//! Shell-owned ACP host used by the product runtime bridge.
//!
//! ACP connections and `MvpAgent` are intentionally kept on a Tokio
//! current-thread `LocalSet`. The public handle contains only `Send` command
//! messages and an event sink, so Tower/App Server never receives or moves the
//! `!Send` ACP connection across its boundary.

use std::sync::{Arc, Mutex};
use std::thread;

use agent_client_protocol as acp;
use agent_client_protocol::Agent as _;
use indexmap::IndexMap;
use tokio::io::duplex;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _};

use crate::agent::app::spawn_agent_local;
use crate::agent::config::{Config as AgentConfig, ModelEntry};
use crate::auth::AuthManager;
use crate::session::info::Info;
use crate::session::storage::{JsonlStorageAdapter, SessionUpdate, StorageAdapter};

/// Errors crossing the host's thread boundary. ACP error details are kept
/// local and reduced to a safe string at the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpHostError(pub String);

impl std::fmt::Display for AcpHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AcpHostError {}

/// Notifications emitted by the real ACP agent.
#[derive(Clone)]
pub struct AcpNotificationSink {
    updates: Arc<Mutex<Vec<acp::SessionNotification>>>,
    live: broadcast::Sender<acp::SessionNotification>,
}

impl AcpNotificationSink {
    pub fn new() -> Self {
        let (live, _) = broadcast::channel(256);
        Self {
            updates: Arc::new(Mutex::new(Vec::new())),
            live,
        }
    }

    pub fn snapshot(&self) -> Vec<acp::SessionNotification> {
        self.updates.lock().map(|updates| updates.clone()).unwrap_or_default()
    }

    /// Subscribe before starting a prompt to receive every subsequent ACP
    /// notification in order. Lag is explicit via `broadcast::RecvError` so a
    /// persistence bridge cannot silently claim a complete history.
    pub fn subscribe(&self) -> broadcast::Receiver<acp::SessionNotification> {
        self.live.subscribe()
    }

    fn push(&self, notification: acp::SessionNotification) {
        if let Ok(mut updates) = self.updates.lock() {
            updates.push(notification.clone());
        }
        let _ = self.live.send(notification);
    }
}

/// Persist a live ACP notification stream into the canonical Shell JSONL
/// authority. Broadcast lag is fatal: silently dropping events would make
/// replay/history incomplete while the actor still appears healthy.
pub async fn persist_notifications(
    mut receiver: broadcast::Receiver<acp::SessionNotification>,
    storage: JsonlStorageAdapter,
    info: Info,
) -> Result<(), AcpHostError> {
    loop {
        match receiver.recv().await {
            Ok(notification) => storage
                .append_update(&info, &SessionUpdate::Acp(Box::new(notification)))
                .await
                .map_err(|error| AcpHostError(format!("persist ACP notification: {error}")))?,
            Err(broadcast::error::RecvError::Closed) => return Ok(()),
            Err(broadcast::error::RecvError::Lagged(count)) => {
                return Err(AcpHostError(format!(
                    "ACP notification persistence lagged by {count} events"
                )));
            }
        }
    }
}

struct HostClient {
    sink: AcpNotificationSink,
}

#[async_trait::async_trait(?Send)]
impl acp::Client for HostClient {
    async fn request_permission(
        &self,
        _args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        // A product caller must explicitly provide a permission policy. The
        // bootstrap host is fail-closed until that policy is wired.
        Ok(acp::RequestPermissionResponse::new(
            acp::RequestPermissionOutcome::Cancelled,
        ))
    }

    async fn session_notification(&self, args: acp::SessionNotification) -> acp::Result<()> {
        self.sink.push(args);
        Ok(())
    }
}

enum Command {
    Initialize {
        request: acp::InitializeRequest,
        response: oneshot::Sender<Result<acp::InitializeResponse, AcpHostError>>,
    },
    Authenticate {
        request: acp::AuthenticateRequest,
        response: oneshot::Sender<Result<acp::AuthenticateResponse, AcpHostError>>,
    },
    NewSession {
        request: acp::NewSessionRequest,
        response: oneshot::Sender<Result<acp::NewSessionResponse, AcpHostError>>,
    },
    Prompt {
        request: acp::PromptRequest,
        response: oneshot::Sender<Result<acp::PromptResponse, AcpHostError>>,
    },
    Cancel {
        request: acp::CancelNotification,
        response: oneshot::Sender<Result<(), AcpHostError>>,
    },
    Shutdown {
        response: oneshot::Sender<()> ,
    },
}

/// `Send` handle to the ACP host. The connection itself never leaves its
/// dedicated current-thread runtime.
pub struct AcpHostHandle {
    commands: mpsc::UnboundedSender<Command>,
    notifications: AcpNotificationSink,
    join: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    persistence: Option<tokio::task::JoinHandle<Result<(), AcpHostError>>>,
}

/// Cloneable command-only view. It deliberately does not own the ACP thread,
/// persistence task, or join handle; those remain owned by `AcpHostHandle`.
#[derive(Clone)]
pub struct AcpCommandHandle {
    commands: mpsc::UnboundedSender<Command>,
}

impl AcpCommandHandle {
    pub async fn prompt(
        &self,
        request: acp::PromptRequest,
    ) -> Result<acp::PromptResponse, AcpHostError> {
        let (response, receive) = oneshot::channel();
        self.commands
            .send(Command::Prompt { request, response })
            .map_err(|_| AcpHostError("ACP host is stopped".into()))?;
        receive
            .await
            .map_err(|_| AcpHostError("ACP host stopped before prompt completed".into()))?
    }

    pub async fn cancel(&self, request: acp::CancelNotification) -> Result<(), AcpHostError> {
        let (response, receive) = oneshot::channel();
        self.commands
            .send(Command::Cancel { request, response })
            .map_err(|_| AcpHostError("ACP host is stopped".into()))?;
        receive
            .await
            .map_err(|_| AcpHostError("ACP host stopped before cancel completed".into()))?
    }
}

impl Drop for AcpHostHandle {
    fn drop(&mut self) {
        // Best-effort rollback for bootstrap failures. Normal product shutdown
        // consumes the handle and joins the thread; this path only guarantees
        // that a partially-created host receives a termination command instead
        // of leaving its LocalSet running until process exit.
        let (response, _receive) = oneshot::channel();
        let _ = self.commands.send(Command::Shutdown { response });
    }
}

impl AcpHostHandle {
    pub fn command_handle(&self) -> AcpCommandHandle {
        AcpCommandHandle {
            commands: self.commands.clone(),
        }
    }
    pub fn notifications(&self) -> AcpNotificationSink {
        self.notifications.clone()
    }

    /// Attach the canonical JSONL persistence consumer to this host. Ownership
    /// transfers to the handle and is awaited by `shutdown(self)`.
    pub fn start_persistence(
        &mut self,
        storage: JsonlStorageAdapter,
        info: Info,
    ) -> Result<(), AcpHostError> {
        if self.persistence.is_some() {
            return Err(AcpHostError("ACP host persistence already started".into()));
        }
        self.persistence = Some(tokio::spawn(persist_notifications(
            self.notifications.subscribe(),
            storage,
            info,
        )));
        Ok(())
    }

    pub async fn initialize(
        &self,
        request: acp::InitializeRequest,
    ) -> Result<acp::InitializeResponse, AcpHostError> {
        self.call(|response| Command::Initialize { request, response })
            .await
    }

    pub async fn new_session(
        &self,
        request: acp::NewSessionRequest,
    ) -> Result<acp::NewSessionResponse, AcpHostError> {
        self.call(|response| Command::NewSession { request, response })
            .await
    }

    pub async fn authenticate(
        &self,
        request: acp::AuthenticateRequest,
    ) -> Result<acp::AuthenticateResponse, AcpHostError> {
        self.call(|response| Command::Authenticate { request, response })
            .await
    }

    pub async fn prompt(
        &self,
        request: acp::PromptRequest,
    ) -> Result<acp::PromptResponse, AcpHostError> {
        self.call(|response| Command::Prompt { request, response })
            .await
    }

    pub async fn cancel(&self, request: acp::CancelNotification) -> Result<(), AcpHostError> {
        self.call(|response| Command::Cancel { request, response })
            .await
    }

    pub async fn shutdown(mut self) -> Result<(), AcpHostError> {
        let (response, receive) = oneshot::channel();
        self.commands
            .send(Command::Shutdown { response })
            .map_err(|_| AcpHostError("ACP host is stopped".into()))?;
        receive
            .await
            .map_err(|_| AcpHostError("ACP host stopped before shutdown".into()))?;
        if let Some(join) = self.join.lock().ok().and_then(|mut join| join.take()) {
            join.join()
                .map_err(|_| AcpHostError("ACP host thread panicked during shutdown".into()))?;
        }
        // The host's internal ACP client has now dropped its sink. Drop this
        // last public clone before waiting for the broadcast consumer to
        // observe channel closure.
        let notifications = std::mem::replace(&mut self.notifications, AcpNotificationSink::new());
        drop(notifications);
        if let Some(persistence) = self.persistence.take() {
            persistence
                .await
                .map_err(|_| AcpHostError("ACP persistence task panicked".into()))??;
        }
        Ok(())
    }

    async fn call<T>(
        &self,
        build: impl FnOnce(oneshot::Sender<Result<T, AcpHostError>>) -> Command,
    ) -> Result<T, AcpHostError> {
        let (response, receive) = oneshot::channel();
        self.commands
            .send(build(response))
            .map_err(|_| AcpHostError("ACP host is stopped".into()))?;
        receive
            .await
            .map_err(|_| AcpHostError("ACP host stopped before command completed".into()))?
    }
}

/// Start the real Shell ACP agent on a dedicated current-thread runtime.
pub fn spawn_acp_host(
    agent_config: AgentConfig,
    auth_manager: Arc<AuthManager>,
    prefetched_models: Option<IndexMap<String, ModelEntry>>,
    memory_config: Option<crate::config::MemoryConfig>,
) -> Result<AcpHostHandle, AcpHostError> {
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
    let thread_result = thread::Builder::new()
        .name("grok-product-acp-host".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = ready_tx.send(Err(AcpHostError(format!(
                        "cannot create ACP host runtime: {error}"
                    ))));
                    return;
                }
            };
            let local = tokio::task::LocalSet::new();
            let result = local.block_on(&runtime, async move {
                let (client_to_agent, agent_input) = duplex(8 * 1024 * 1024);
                let (agent_to_client, client_input) = duplex(8 * 1024 * 1024);
                let (commands, mut command_rx) = mpsc::unbounded_channel();
                let sink = AcpNotificationSink::new();
                let client = HostClient { sink: sink.clone() };
                let (connection, client_io) = acp::ClientSideConnection::new(
                    client,
                    client_to_agent.compat_write(),
                    client_input.compat(),
                    |future| {
                        tokio::task::spawn_local(future);
                    },
                );
                let agent_io = spawn_agent_local(
                    agent_config,
                    auth_manager,
                    prefetched_models,
                    memory_config,
                    agent_to_client.compat_write(),
                    agent_input.compat(),
                );
                tokio::task::spawn_local(client_io);
                tokio::task::spawn_local(agent_io);
                let _ = ready_tx.send(Ok((commands, sink)));

                while let Some(command) = command_rx.recv().await {
                    match command {
                        Command::Initialize { request, response } => {
                            let _ = response.send(connection.initialize(request).await.map_err(|e| {
                                AcpHostError(format!("ACP initialize failed: {e}"))
                            }));
                        }
                        Command::Authenticate { request, response } => {
                            let _ = response.send(connection.authenticate(request).await.map_err(|e| {
                                AcpHostError(format!("ACP authenticate failed: {e}"))
                            }));
                        }
                        Command::NewSession { request, response } => {
                            let _ = response.send(connection.new_session(request).await.map_err(|e| {
                                AcpHostError(format!("ACP new session failed: {e}"))
                            }));
                        }
                        Command::Prompt { request, response } => {
                            let _ = response.send(connection.prompt(request).await.map_err(|e| {
                                AcpHostError(format!("ACP prompt failed: {e}"))
                            }));
                        }
                        Command::Cancel { request, response } => {
                            let _ = response.send(connection.cancel(request).await.map_err(|e| {
                                AcpHostError(format!("ACP cancel failed: {e}"))
                            }));
                        }
                        Command::Shutdown { response } => {
                            let _ = response.send(());
                            break;
                        }
                    }
                }
                Ok::<(), AcpHostError>(())
            });
            if let Err(error) = result {
                tracing::error!(%error, "ACP host stopped");
            }
        })
        .map_err(|error| AcpHostError(format!("cannot spawn ACP host thread: {error}")))?;
    let (commands, notifications) = ready_rx
        .recv()
        .map_err(|_| AcpHostError("ACP host exited during startup".into()))??;
    Ok(AcpHostHandle {
        commands,
        notifications,
        join: Arc::new(Mutex::new(Some(thread_result))),
        persistence: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_sink_is_thread_safe_and_preserves_order() {
        let sink = AcpNotificationSink::new();
        let first = acp::SessionNotification::new(
            acp::SessionId::new("s1"),
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(
                acp::ContentBlock::Text(acp::TextContent::new("one")),
            )),
        );
        let second = acp::SessionNotification::new(
            acp::SessionId::new("s1"),
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(
                acp::ContentBlock::Text(acp::TextContent::new("two")),
            )),
        );
        sink.push(first.clone());
        sink.push(second.clone());
        assert_eq!(sink.snapshot(), vec![first, second]);
    }
}
