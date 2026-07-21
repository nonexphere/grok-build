//! Shell-owned ACP host used by the product runtime bridge.
//!
//! ACP connections and `MvpAgent` are intentionally kept on a Tokio
//! current-thread `LocalSet`. The public handle contains only `Send` command
//! messages and an event sink, so Tower/App Server never receives or moves the
//! `!Send` ACP connection across its boundary.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use agent_client_protocol as acp;
use agent_client_protocol::Agent as _;
use indexmap::IndexMap;
use serde_json::Value;
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
    permission_requests: broadcast::Sender<AcpPermissionRequest>,
}

/// A lossless observation of an ACP permission reverse request. The raw ACP
/// payload is retained until the App Server contract can map every permission
/// option without dropping provider-specific fields.
#[derive(Clone, Debug, PartialEq)]
pub struct AcpPermissionRequest {
    pub session_id: String,
    pub turn_id: Option<String>,
    pub item_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub payload: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcpPermissionDecision {
    Selected(String),
    Cancelled,
}

impl AcpNotificationSink {
    pub fn new() -> Self {
        let (live, _) = broadcast::channel(256);
        let (permission_requests, _) = broadcast::channel(256);
        Self {
            updates: Arc::new(Mutex::new(Vec::new())),
            live,
            permission_requests,
        }
    }

    pub fn snapshot(&self) -> Vec<acp::SessionNotification> {
        self.updates
            .lock()
            .map(|updates| updates.clone())
            .unwrap_or_default()
    }

    /// Subscribe before starting a prompt to receive every subsequent ACP
    /// notification in order. Lag is explicit via `broadcast::RecvError` so a
    /// persistence bridge cannot silently claim a complete history.
    pub fn subscribe(&self) -> broadcast::Receiver<acp::SessionNotification> {
        self.live.subscribe()
    }

    pub fn subscribe_permission_requests(&self) -> broadcast::Receiver<AcpPermissionRequest> {
        self.permission_requests.subscribe()
    }

    fn push(&self, notification: acp::SessionNotification) {
        if let Ok(mut updates) = self.updates.lock() {
            updates.push(notification.clone());
        }
        let _ = self.live.send(notification);
    }

    fn push_permission_request(&self, request: AcpPermissionRequest) {
        let _ = self.permission_requests.send(request);
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
    prompt_context: Arc<Mutex<Option<PromptContext>>>,
    decisions: Arc<Mutex<HashMap<String, oneshot::Sender<AcpPermissionDecision>>>>,
    permission_timeout: std::time::Duration,
}

#[derive(Clone, Debug)]
struct PromptContext {
    session_id: String,
    turn_id: Option<String>,
}

#[async_trait::async_trait(?Send)]
impl acp::Client for HostClient {
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        let payload = serde_json::to_value(&args).unwrap_or(Value::Null);
        let tool_call_id = payload
            .pointer("/toolCall/toolCallId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let context = self
            .prompt_context
            .lock()
            .ok()
            .and_then(|value| value.clone());
        let session_id = context
            .as_ref()
            .map(|value| value.session_id.clone())
            .or_else(|| {
                payload
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_default();
        let item_id = tool_call_id.as_ref().map(|id| format!("tc_{id}"));
        self.sink.push_permission_request(AcpPermissionRequest {
            session_id,
            turn_id: context.as_ref().and_then(|value| value.turn_id.clone()),
            item_id,
            tool_call_id: tool_call_id.clone(),
            payload,
        });
        let Some(tool_call_id) = tool_call_id else {
            return Ok(acp::RequestPermissionResponse::new(
                acp::RequestPermissionOutcome::Cancelled,
            ));
        };
        let (sender, receiver) = oneshot::channel();
        if let Ok(mut decisions) = self.decisions.lock() {
            if decisions.contains_key(&tool_call_id) {
                return Ok(acp::RequestPermissionResponse::new(
                    acp::RequestPermissionOutcome::Cancelled,
                ));
            }
            decisions.insert(tool_call_id.clone(), sender);
        } else {
            return Ok(acp::RequestPermissionResponse::new(
                acp::RequestPermissionOutcome::Cancelled,
            ));
        }
        // Publishing first makes the request observable before the ACP
        // reverse call parks. A missing/late decision remains fail-closed.
        let decision = match tokio::time::timeout(self.permission_timeout, receiver).await {
            Ok(Ok(decision)) => decision,
            Ok(Err(_)) | Err(_) => {
                if let Ok(mut decisions) = self.decisions.lock() {
                    decisions.remove(&tool_call_id);
                }
                AcpPermissionDecision::Cancelled
            }
        };
        let outcome = match decision {
            AcpPermissionDecision::Selected(option_id) => acp::RequestPermissionOutcome::Selected(
                acp::SelectedPermissionOutcome::new(option_id),
            ),
            AcpPermissionDecision::Cancelled => acp::RequestPermissionOutcome::Cancelled,
        };
        Ok(acp::RequestPermissionResponse::new(outcome))
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
        context: PromptContext,
        response: oneshot::Sender<Result<acp::PromptResponse, AcpHostError>>,
    },
    Cancel {
        request: acp::CancelNotification,
        response: oneshot::Sender<Result<(), AcpHostError>>,
    },
    RespondPermission {
        tool_call_id: String,
        decision: AcpPermissionDecision,
        response: oneshot::Sender<Result<(), AcpHostError>>,
    },
    Shutdown {
        response: oneshot::Sender<()>,
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
        let context = PromptContext {
            session_id: request.session_id.to_string(),
            turn_id: None,
        };
        self.commands
            .send(Command::Prompt {
                request,
                context,
                response,
            })
            .map_err(|_| AcpHostError("ACP host is stopped".into()))?;
        receive
            .await
            .map_err(|_| AcpHostError("ACP host stopped before prompt completed".into()))?
    }

    pub async fn prompt_with_context(
        &self,
        request: acp::PromptRequest,
        turn_id: String,
    ) -> Result<acp::PromptResponse, AcpHostError> {
        let (response, receive) = oneshot::channel();
        let context = PromptContext {
            session_id: request.session_id.to_string(),
            turn_id: Some(turn_id),
        };
        self.commands
            .send(Command::Prompt {
                request,
                context,
                response,
            })
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

    pub async fn respond_permission(
        &self,
        tool_call_id: String,
        decision: AcpPermissionDecision,
    ) -> Result<(), AcpHostError> {
        let (response, receive) = oneshot::channel();
        self.commands
            .send(Command::RespondPermission {
                tool_call_id,
                decision,
                response,
            })
            .map_err(|_| AcpHostError("ACP host is stopped".into()))?;
        receive
            .await
            .map_err(|_| AcpHostError("ACP host stopped before permission response".into()))?
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

    pub async fn respond_permission(
        &self,
        tool_call_id: String,
        decision: AcpPermissionDecision,
    ) -> Result<(), AcpHostError> {
        self.command_handle()
            .respond_permission(tool_call_id, decision)
            .await
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
        let context = PromptContext {
            session_id: request.session_id.to_string(),
            turn_id: None,
        };
        self.call(|response| Command::Prompt {
            request,
            context,
            response,
        })
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
            let result =
                local.block_on(&runtime, async move {
                    let (client_to_agent, agent_input) = duplex(8 * 1024 * 1024);
                    let (agent_to_client, client_input) = duplex(8 * 1024 * 1024);
                    let (commands, mut command_rx) = mpsc::unbounded_channel();
                    let sink = AcpNotificationSink::new();
                    let prompt_context = Arc::new(Mutex::new(None));
                    let decisions = Arc::new(Mutex::new(HashMap::new()));
                    let client = HostClient {
                        sink: sink.clone(),
                        prompt_context: prompt_context.clone(),
                        decisions: decisions.clone(),
                        permission_timeout: std::time::Duration::from_secs(300),
                    };
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
                                let _ =
                                    response.send(connection.initialize(request).await.map_err(
                                        |e| AcpHostError(format!("ACP initialize failed: {e}")),
                                    ));
                            }
                            Command::Authenticate { request, response } => {
                                let _ =
                                    response.send(connection.authenticate(request).await.map_err(
                                        |e| AcpHostError(format!("ACP authenticate failed: {e}")),
                                    ));
                            }
                            Command::NewSession { request, response } => {
                                let _ =
                                    response.send(connection.new_session(request).await.map_err(
                                        |e| AcpHostError(format!("ACP new session failed: {e}")),
                                    ));
                            }
                            Command::Prompt {
                                request,
                                context,
                                response,
                            } => {
                                if let Ok(mut current) = prompt_context.lock() {
                                    *current = Some(context);
                                }
                                let _ =
                                    response.send(connection.prompt(request).await.map_err(|e| {
                                        AcpHostError(format!("ACP prompt failed: {e}"))
                                    }));
                                if let Ok(mut current) = prompt_context.lock() {
                                    *current = None;
                                }
                            }
                            Command::Cancel { request, response } => {
                                let _ =
                                    response.send(connection.cancel(request).await.map_err(|e| {
                                        AcpHostError(format!("ACP cancel failed: {e}"))
                                    }));
                            }
                            Command::RespondPermission {
                                tool_call_id,
                                decision,
                                response,
                            } => {
                                let result = decisions
                                    .lock()
                                    .ok()
                                    .and_then(|mut pending| pending.remove(&tool_call_id))
                                    .map(|sender| {
                                        sender.send(decision).map_err(|_| {
                                            AcpHostError(
                                                "permission request is no longer waiting".into(),
                                            )
                                        })
                                    })
                                    .unwrap_or_else(|| {
                                        Err(AcpHostError("permission request not found".into()))
                                    });
                                let _ = response.send(result);
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
    use agent_client_protocol::Client as _;

    #[test]
    fn permission_request_sink_preserves_identity_and_payload() {
        let sink = AcpNotificationSink::new();
        let mut receiver = sink.subscribe_permission_requests();
        let request = AcpPermissionRequest {
            session_id: "session-1".into(),
            turn_id: Some("turn-1".into()),
            item_id: Some("tc-call-1".into()),
            tool_call_id: Some("call-1".into()),
            payload: serde_json::json!({"toolCall":{"toolCallId":"call-1"}}),
        };
        sink.push_permission_request(request.clone());
        assert_eq!(receiver.try_recv().expect("permission request"), request);
    }

    #[test]
    fn notification_sink_is_thread_safe_and_preserves_order() {
        let sink = AcpNotificationSink::new();
        let first = acp::SessionNotification::new(
            acp::SessionId::new("s1"),
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
                acp::TextContent::new("one"),
            ))),
        );
        let second = acp::SessionNotification::new(
            acp::SessionId::new("s1"),
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
                acp::TextContent::new("two"),
            ))),
        );
        sink.push(first.clone());
        sink.push(second.clone());
        assert_eq!(sink.snapshot(), vec![first, second]);
    }

    #[tokio::test]
    async fn permission_reverse_request_waits_for_selected_decision() {
        let sink = AcpNotificationSink::new();
        let mut requests = sink.subscribe_permission_requests();
        let decisions = Arc::new(Mutex::new(HashMap::new()));
        let client = HostClient {
            sink,
            prompt_context: Arc::new(Mutex::new(Some(PromptContext {
                session_id: "session-1".into(),
                turn_id: Some("turn-1".into()),
            }))),
            decisions: decisions.clone(),
            permission_timeout: std::time::Duration::from_secs(1),
        };
        let request = acp::RequestPermissionRequest::new(
            acp::SessionId::new("session-1"),
            acp::ToolCallUpdate::new(
                acp::ToolCallId::new("call-1"),
                acp::ToolCallUpdateFields::new()
                    .kind(Some(acp::ToolKind::Other))
                    .title(Some("test tool".into())),
            ),
            vec![acp::PermissionOption::new(
                "allow-once",
                "Allow once",
                acp::PermissionOptionKind::AllowOnce,
            )],
        );

        let response = tokio::task::LocalSet::new()
            .run_until(async move {
                let task =
                    tokio::task::spawn_local(
                        async move { client.request_permission(request).await },
                    );
                let observed =
                    tokio::time::timeout(std::time::Duration::from_secs(1), requests.recv())
                        .await
                        .expect("permission request timeout")
                        .expect("permission request broadcast");
                assert_eq!(observed.session_id, "session-1");
                assert_eq!(observed.turn_id.as_deref(), Some("turn-1"));
                assert_eq!(observed.item_id.as_deref(), Some("tc_call-1"));
                assert_eq!(observed.tool_call_id.as_deref(), Some("call-1"));

                let sender = decisions
                    .lock()
                    .expect("decision mutex")
                    .remove("call-1")
                    .expect("permission waiter");
                sender
                    .send(AcpPermissionDecision::Selected("allow-once".into()))
                    .expect("decision delivery");
                task.await
                    .expect("permission task")
                    .expect("permission response")
            })
            .await;
        assert!(matches!(
            response.outcome,
            acp::RequestPermissionOutcome::Selected(selected)
                if selected.option_id.0.as_ref() == "allow-once"
        ));
    }

    #[tokio::test]
    async fn permission_reverse_request_expires_and_removes_waiter() {
        let sink = AcpNotificationSink::new();
        let mut requests = sink.subscribe_permission_requests();
        let decisions = Arc::new(Mutex::new(HashMap::new()));
        let client = HostClient {
            sink,
            prompt_context: Arc::new(Mutex::new(Some(PromptContext {
                session_id: "session-timeout".into(),
                turn_id: Some("turn-timeout".into()),
            }))),
            decisions: decisions.clone(),
            permission_timeout: std::time::Duration::from_millis(20),
        };
        let request = acp::RequestPermissionRequest::new(
            acp::SessionId::new("session-timeout"),
            acp::ToolCallUpdate::new(
                acp::ToolCallId::new("call-timeout"),
                acp::ToolCallUpdateFields::new().kind(Some(acp::ToolKind::Other)),
            ),
            vec![],
        );
        let response_task = tokio::task::LocalSet::new()
            .run_until(async move {
                let task =
                    tokio::task::spawn_local(
                        async move { client.request_permission(request).await },
                    );
                let observed = requests.recv().await.expect("permission request");
                assert_eq!(observed.tool_call_id.as_deref(), Some("call-timeout"));
                task.await
                    .expect("permission task")
                    .expect("permission response")
            })
            .await;
        assert!(matches!(
            response_task.outcome,
            acp::RequestPermissionOutcome::Cancelled
        ));
        assert!(
            decisions.lock().expect("decision mutex").is_empty(),
            "expiry must remove the waiter so a late response cannot authorize"
        );
    }
}
