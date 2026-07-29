//! Shell-owned `GrokRuntimeFacade` backed by the real JSONL storage adapter.
//!
//! This is the C1-D real port. It maps every facade method to an *existing*
//! Shell symbol (see `waves/c0-session-actor-command-map.md` §1) so the
//! composition root can switch off `FakeRuntime` for the experimental product
//! path without introducing a second `SessionActor`, a second permission
//! engine, a second replay buffer, or a hybrid Fake+JSONL authority.
//!
//! ## Authority
//! One authority per session: the real `JsonlStorageAdapter`. Reads and
//! writes go through the same on-disk `summary.json` / `updates.jsonl` /
//! `chat_history.jsonl` layout that the live `SessionActor` uses. `FakeRuntime`
//! is NOT mixed in here (corrective contract §2 / audit F-13).
//!
//! ## What is real vs PARTIAL (C1-D)
//! - **REAL (storage-backed):** `list_sessions`, `read_session` (session row),
//!   `start_session` (persists `summary.json` via `init_session`),
//!   `resume_session` (loads `summary.json`), `fork_session`
//!   (`copy_session_data`), `replay` (projects `updates.jsonl`).
//! - **PARTIAL (actor fixture gap):** `start_turn`, `steer_turn`,
//!   `interrupt_turn` require a live `SessionActor` (`!Send`, dedicated thread
//!   + `LocalSet` + auth/credentials/tool-context). The adapter returns
//!   `unsupported` for these and documents the gap in `waves/c1-shell-port.md`.
//! - **REAL (R6 hide-not-delete):** `archive_session` writes `archived.flag`
//!   and projects `SessionStatus::Archived` without deleting session data.
//! - **REAL (R10 / R5-09 delivery channel):** `respond_interaction` is a
//!   delivery channel into the existing pending-interaction surface (C6-B):
//!   it checks `pending_interactions` membership, removes the entry
//!   (first-answer-wins), and delivers the decision string via the shared
//!   process-local oneshot hub (`SessionHandle::interaction_delivery_hub` /
//!   `ResidentHandle::delivery_hub`). It does NOT re-evaluate allow/deny
//!   policy. The live actor parks reverse-request oneshots on that hub at
//!   spawn (`ask_user_question` dual-waits hub vs ACP). Fixture/tests may
//!   also pre-seed the hub. Without a resident, the method returns
//!   `unsupported` honestly. Product turns still need a real spawn factory
//!   (EXTERNAL credentials) — that is separate from this delivery seam.
//! - **PARTIAL (projection):** `read_session` Turn/Item projection (R2) and
//!   `replay` full `RuntimeEvent` projection (R11) — Shell has no first-class
//!   `Turn`/`Item`/`RuntimeEvent` projector; the adapter builds a shared one
//!   over `updates.jsonl` (C3-F). REAL: user/agent/thought chunks, tool call
//!   lifecycle (`ItemStarted`→`ItemCompleted` correlated via `tool_call_id`),
//!   plan. PARTIAL: `TurnChanged` not emitted (Shell writes no turn lifecycle);
//!   turn status inferred `Completed` from persistence; item grouping across
//!   streaming chunks not performed; `InteractionRequested` not projected
//!   (in-memory only); `created_at_ms` is 0 (`UpdatesIterator` drops the
//!   envelope timestamp); xAI extension updates skipped.
//!
//! `MvpAgent`/`SessionActor` are NOT reinvented here. The turn lifecycle
//! forwards to the existing `SessionCommand::{Prompt,Interject,Cancel}` once
//! the actor fixture is wired (Wave C1 follow-on).

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::sync::{Mutex as TokioMutex, mpsc, oneshot, watch};
use tokio::time::{Duration, timeout};
use xai_grok_app_server_protocol::{
    InteractionResponseParams, Item, ItemBody, ItemStatus, ProviderBinding, Session,
    SessionArchiveParams, SessionForkParams, SessionReadParams, SessionReadResult,
    SessionResumeParams, SessionStartParams, SessionStatus, SubscribeParams, Turn,
    TurnInterruptParams, TurnKind, TurnStartParams, TurnStatus, TurnSteerParams, WireCounter,
};
use xai_grok_tower::{
    GrokRuntimeFacade, ReplayPage, RuntimeCapabilities, RuntimeError, RuntimeEvent,
};

use crate::app_server_runtime::acp_host::AcpCommandHandle;
use crate::session::commands::{
    PromptCompletionKind, PromptTurnOk, PromptTurnResult, SessionCommand,
};
use crate::session::handle::SessionHandle;
use crate::session::info::Info;
use crate::session::pending_interaction::PendingInteractions;
use crate::session::persistence::{Summary, default_model_id};
use crate::session::plan_mode::PromptMode;
use crate::session::storage::{
    JsonlStorageAdapter, SessionUpdate, StorageAdapter, UpdatesIterator,
};

async fn receive_prompt_response(
    response: oneshot::Receiver<PromptTurnResult>,
    deadline: Duration,
) -> Result<Result<PromptTurnResult, oneshot::error::RecvError>, tokio::time::error::Elapsed> {
    timeout(deadline, response).await
}

/// History epoch fallback only when a session has no durable epoch file yet
/// (legacy sessions). New sessions always write a unique epoch (R5-04).
const HISTORY_EPOCH_DEFAULT: &str = "epoch_1";
const HISTORY_EPOCH_FILE: &str = "history_epoch";

/// Replay page size (events per page). Matches the Fake conformance bound.
const REPLAY_PAGE_SIZE: usize = 100;

/// Process-local delivery hub: `interaction_id` (= `tool_call_id`) → oneshot
/// for a parked reverse-request future (R10 / R5-09).
pub type InteractionDeliveryHub = Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>;

/// The `Send`-able projection of a live `SessionHandle` needed to route turn
/// lifecycle commands.
///
/// This is **not** a second `SessionActor` — it holds only the command channel
/// (`cmd_tx`) and the shared `current_prompt_id` slot that the real actor
/// updates. The real `SessionActor` remains the single authority on its
/// dedicated thread + `LocalSet`; this struct lets the `Send + Sync` facade
/// enqueue `SessionCommand::{Prompt,Interject,Cancel}` without moving the
/// `!Send` actor across threads.
///
/// Construct from a real `SessionHandle` via [`ResidentHandle::from_handle`],
/// or directly from a channel for a real `cmd_tx` consumer fixture (tests).
pub struct ResidentHandle {
    /// Command channel into the live session actor (or a real test consumer).
    pub cmd_tx: mpsc::UnboundedSender<SessionCommand>,
    /// Shared running-turn id, mirrored from the actor. `None` when idle.
    pub current_prompt_id: Arc<Mutex<Option<String>>>,
    /// Open blocking reverse-requests (permission / question / plan-approval),
    /// keyed by `tool_call_id`. Projected from the real `SessionHandle` so
    /// `respond_interaction` can check membership without a second permission
    /// engine. `None` when the resident was constructed without a real
    /// pending-interactions surface (e.g. a minimal test spawner that does
    /// not exercise the interaction delivery channel).
    pub pending_interactions: Option<PendingInteractions>,
    /// Delivery hub for `respond_interaction`: maps `interaction_id`
    /// (= `tool_call_id`) → `oneshot::Sender<String>` for the parked future.
    /// The actor registers here when it parks on a blocking reverse-request;
    /// `respond_interaction` delivers the decision string through this hub
    /// **without re-evaluating allow/deny policy**. Shared with
    /// [`SessionHandle::interaction_delivery_hub`] so the live actor and the
    /// facade use the same map (R5-09).
    pub delivery_hub: Option<InteractionDeliveryHub>,
    /// ACP permission decision channel for the production resident bridge.
    /// `None` for hermetic/local fixtures that use `delivery_hub` directly.
    pub permission_responder: Option<AcpCommandHandle>,
}

impl ResidentHandle {
    /// Project the `Send`-able command-routing surface off a real
    /// `SessionHandle`. The actor stays on its own thread; only the channel
    /// and the shared prompt-id slot / pending maps are retained here.
    pub fn from_handle(handle: SessionHandle) -> Self {
        Self {
            cmd_tx: handle.cmd_tx,
            current_prompt_id: handle.current_prompt_id,
            pending_interactions: Some(handle.pending_interactions),
            // R5-09: share the actor's hub — never invent a disconnected map.
            delivery_hub: Some(handle.interaction_delivery_hub),
            permission_responder: None,
        }
    }

    /// Read the current running turn id (snapshot). `None` when no turn is in
    /// flight. Exposed for tests that observe the running-turn slot.
    pub fn current_turn(&self) -> Option<String> {
        self.current_prompt_id.lock().ok().and_then(|g| g.clone())
    }
}

/// Spawn hook for a live session actor.
///
/// The default production implementation ([`ProductionSpawner`] with no real
/// spawn function) returns `unsupported` because the full
/// `spawn_session_on_thread` path requires HUMAN credentials, an
/// `AgentDefinition`, MCP/tool context, a `GatewaySender`, `ModelsManager`,
/// `PersistenceHandle`, `WorkspaceOps`, plugin registry, and a dedicated
/// thread + `LocalSet` that cannot be assembled hermetically without
/// HUMAN-provided auth. The structure is in place so the composition root
/// (handoff C2-A owns composition wiring) can inject a real spawn function
/// — either by implementing this trait or by passing a [`RealSpawnFn`] to
/// [`ProductionSpawner::with_real_spawn`] — without touching the facade
/// method bodies.
///
/// Tests inject a real `cmd_tx` consumer (NOT `FakeRuntime`) that actually
/// processes `SessionCommand::{Prompt,Interject,Cancel}` and persists side
/// effects to disk, proving command routing against a real actor path.
#[async_trait]
pub trait SessionSpawner: Send + Sync {
    /// Spawn (or attach to) a live actor for `info` and return its
    /// `Send`-able command-routing handle.
    async fn spawn(
        &self,
        info: &Info,
        model_id: &agent_client_protocol::ModelId,
    ) -> Result<ResidentHandle, RuntimeError>;
}

/// Concrete real-spawn function injected by the composition root (C2-A).
///
/// When present, [`ProductionSpawner`] delegates `spawn` to it; when absent,
/// `spawn` returns `unsupported` enumerating the missing production spawn
/// dependencies. This lets the composition root wire a real
/// `spawn_session_on_thread`-backed factory (assembling credentials,
/// `AgentDefinition`, tool context, MCP servers, `ModelsManager`, etc.) without
/// implementing the full [`SessionSpawner`] trait, and keeps the facade method
/// bodies unchanged.
pub type RealSpawnFn = Arc<
    dyn Fn(
            Info,
            agent_client_protocol::ModelId,
        ) -> Pin<Box<dyn Future<Output = Result<ResidentHandle, RuntimeError>> + Send>>
        + Send
        + Sync,
>;

/// Build a real ACP-backed command bridge for one session.
///
/// This is an integration seam, not the default product factory yet: the ACP
/// host owns inference and JSONL notification persistence, while this bridge
/// owns only the `Send` command mailbox and running-prompt slot. Interaction
/// policy, item projection, and multi-command concurrency still require the
/// remaining product gates before composition can inject it.
pub fn experimental_acp_resident_spawn(root: PathBuf) -> RealSpawnFn {
    Arc::new(
        move |info: Info, model_id: agent_client_protocol::ModelId| {
            let root = root.clone();
            Box::pin(async move {
                use crate::agent::config::Config as AgentConfig;
                use crate::app_server_runtime::acp_host::spawn_acp_host;
                use agent_client_protocol::Agent as _;

                let config = AgentConfig::default();
                let auth_manager = Arc::new(config.create_auth_manager());
                let mut host =
                    spawn_acp_host(config, auth_manager, None, None).map_err(|error| {
                        RuntimeError {
                            code: "spawn_failed",
                            message: error.to_string(),
                        }
                    })?;
                let init = host
                    .initialize(
                        agent_client_protocol::InitializeRequest::new(
                            agent_client_protocol::ProtocolVersion::V1,
                        )
                        .client_capabilities(
                            agent_client_protocol::ClientCapabilities::new()
                                .fs(agent_client_protocol::FileSystemCapabilities::new())
                                .terminal(false),
                        ),
                    )
                    .await
                    .map_err(|error| RuntimeError {
                        code: "spawn_failed",
                        message: error.to_string(),
                    })?;
                let auth_method = init
                    .auth_methods
                    .iter()
                    .find(|method| method.id().0.as_ref() == "xai.api_key")
                    .ok_or_else(|| RuntimeError {
                        code: "auth_unavailable",
                        message: "ACP host did not advertise xai.api_key".into(),
                    })?;
                host.authenticate(agent_client_protocol::AuthenticateRequest::new(
                    auth_method.id().clone(),
                ))
                .await
                .map_err(|error| RuntimeError {
                    code: "auth_failed",
                    message: error.to_string(),
                })?;
                let session = host
                    .new_session(
                        agent_client_protocol::NewSessionRequest::new(PathBuf::from(&info.cwd))
                            .mcp_servers(vec![])
                            .meta(
                                serde_json::json!({
                                    "sessionId": info.id.0.to_string(),
                                    "modelId": model_id.0.to_string(),
                                })
                                .as_object()
                                .cloned(),
                            ),
                    )
                    .await
                    .map_err(|error| RuntimeError {
                        code: "session_spawn_failed",
                        message: error.to_string(),
                    })?;
                if session.session_id != info.id {
                    return Err(RuntimeError {
                        code: "session_identity_mismatch",
                        message: "ACP host returned a different session identity".into(),
                    });
                }
                let storage = JsonlStorageAdapter::with_root(root);
                storage
                    .init_session(&info, model_id.clone())
                    .await
                    .map_err(|error| RuntimeError {
                        code: "persistence_failed",
                        message: error.to_string(),
                    })?;
                host.start_persistence(storage, info.clone())
                    .map_err(|error| RuntimeError {
                        code: "persistence_failed",
                        message: error.to_string(),
                    })?;

                let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SessionCommand>();
                let acp_commands = host.command_handle();
                let permission_responder = acp_commands.clone();
                let mut prompt_tasks = tokio::task::JoinSet::new();
                let current_prompt_id = Arc::new(Mutex::new(None::<String>));
                let pending_interactions: PendingInteractions =
                    Arc::new(Mutex::new(HashMap::new()));
                let delivery_hub: InteractionDeliveryHub = Arc::new(Mutex::new(HashMap::new()));
                let current_clone = current_prompt_id.clone();
                let session_id = info.id.clone();
                tokio::spawn(async move {
                    let (cancel_tx, _) = watch::channel(0_u64);
                    while let Some(command) = cmd_rx.recv().await {
                        match command {
                            SessionCommand::Prompt {
                                prompt_id,
                                prompt_blocks,
                                respond_to,
                                ..
                            } => {
                                if let Ok(mut current) = current_clone.lock() {
                                    *current = Some(prompt_id.clone());
                                }
                                let acp_commands = acp_commands.clone();
                                let session_id = session_id.clone();
                                let current = current_clone.clone();
                                let mut cancel_rx = cancel_tx.subscribe();
                                prompt_tasks.spawn(async move {
                                let result = tokio::select! {
                                    response = acp_commands.prompt_with_context(
                                        agent_client_protocol::PromptRequest::new(
                                            session_id,
                                            prompt_blocks,
                                        ),
                                        prompt_id.clone(),
                                    ) => response
                                        .map(|response| PromptTurnOk {
                                            stop_reason: response.stop_reason,
                                            total_tokens: 0,
                                            turn_snapshot: None,
                                            completion_kind: PromptCompletionKind::Completed,
                                            structured_output: None,
                                            usage: None,
                                        })
                                        .map_err(|error| {
                                            agent_client_protocol::Error::internal_error()
                                                .data(error.to_string())
                                        }),
                                    _ = cancel_rx.changed() => Ok(PromptTurnOk {
                                        stop_reason: agent_client_protocol::StopReason::Cancelled,
                                        total_tokens: 0,
                                        turn_snapshot: None,
                                        completion_kind: PromptCompletionKind::Cancelled {
                                            category: None,
                                            context: None,
                                        },
                                        structured_output: None,
                                        usage: None,
                                    }),
                                };
                                let _ = respond_to.send(result);
                                if let Ok(mut current) = current.lock() {
                                    *current = None;
                                }
                            });
                            }
                            SessionCommand::Interject { text, .. } => {
                                let acp_commands = acp_commands.clone();
                                let session_id = session_id.clone();
                                prompt_tasks.spawn(async move {
                                    let _ = acp_commands
                                        .prompt(agent_client_protocol::PromptRequest::new(
                                            session_id.clone(),
                                            vec![agent_client_protocol::ContentBlock::Text(
                                                agent_client_protocol::TextContent::new(text),
                                            )],
                                        ))
                                        .await;
                                });
                            }
                            SessionCommand::Cancel { .. } => {
                                let next_cancel = cancel_tx.borrow().wrapping_add(1);
                                let _ = cancel_tx.send(next_cancel);
                                let _ = acp_commands
                                    .cancel(agent_client_protocol::CancelNotification::new(
                                        session_id.clone(),
                                    ))
                                    .await;
                                if let Ok(mut current) = current_clone.lock() {
                                    *current = None;
                                }
                            }
                            SessionCommand::Shutdown => {
                                while prompt_tasks.join_next().await.is_some() {}
                                break;
                            }
                            _ => {}
                        }
                    }
                    let _ = host.shutdown().await;
                });
                Ok(ResidentHandle {
                    cmd_tx,
                    current_prompt_id,
                    pending_interactions: Some(pending_interactions),
                    delivery_hub: Some(delivery_hub),
                    permission_responder: Some(permission_responder),
                })
            })
        },
    )
}

/// **Test/fixture-only** offline turn factory (R5-01).
///
/// Spawns a real `cmd_tx` consumer that processes
/// `SessionCommand::{Prompt,Interject,Cancel}`, persists agent message chunks
/// via the JSONL storage adapter, and resolves prompt oneshots with a
/// deterministic offline echo. This is **not** a product spawn path and must
/// not be injected by the composition root. Use it only in hermetic tests that
/// need turn routing without live model credentials.
///
/// Product composition uses [`ShellSessionActorRuntime::new`] (no factory):
/// missing credentials yield explicit `unsupported` / spawn errors — never an
/// echo presented as production behavior.
///
/// Full `spawn_session_on_thread` with live credentials remains the EXTERNAL
/// production path when the composition root injects a real [`RealSpawnFn`].
pub fn experimental_local_turn_spawn(root: PathBuf) -> RealSpawnFn {
    Arc::new(
        move |info: Info, _model_id: agent_client_protocol::ModelId| {
            let root = root.clone();
            Box::pin(async move {
                let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SessionCommand>();
                let current_prompt_id = Arc::new(std::sync::Mutex::new(None::<String>));
                let pending_interactions: PendingInteractions =
                    Arc::new(std::sync::Mutex::new(HashMap::new()));
                let delivery_hub: InteractionDeliveryHub = Arc::new(Mutex::new(HashMap::new()));
                let storage = JsonlStorageAdapter::with_root(root);
                let info_clone = info.clone();
                let current_clone = current_prompt_id.clone();
                tokio::spawn(async move {
                    while let Some(cmd) = cmd_rx.recv().await {
                        match cmd {
                            SessionCommand::Prompt {
                                prompt_id,
                                respond_to,
                                ..
                            } => {
                                if let Ok(mut g) = current_clone.lock() {
                                    *g = Some(prompt_id.clone());
                                }
                                let notification = agent_client_protocol::SessionNotification::new(
                                    info_clone.id.clone(),
                                    agent_client_protocol::SessionUpdate::AgentMessageChunk(
                                        agent_client_protocol::ContentChunk::new(
                                            agent_client_protocol::ContentBlock::Text(
                                                agent_client_protocol::TextContent::new(format!(
                                                    "experimental-local-reply-{prompt_id}"
                                                )),
                                            ),
                                        ),
                                    ),
                                );
                                let _ = storage
                                    .append_update(
                                        &info_clone,
                                        &SessionUpdate::Acp(Box::new(notification)),
                                    )
                                    .await;
                                let _ = respond_to.send(Ok(PromptTurnOk {
                                    stop_reason: agent_client_protocol::StopReason::EndTurn,
                                    total_tokens: 0,
                                    turn_snapshot: None,
                                    completion_kind: PromptCompletionKind::Completed,
                                    structured_output: None,
                                    usage: None,
                                }));
                                if let Ok(mut g) = current_clone.lock() {
                                    *g = None;
                                }
                            }
                            SessionCommand::Interject { .. } => {}
                            SessionCommand::Cancel { .. } => {
                                if let Ok(mut g) = current_clone.lock() {
                                    *g = None;
                                }
                            }
                            _ => {}
                        }
                    }
                });
                Ok(ResidentHandle {
                    cmd_tx,
                    current_prompt_id,
                    pending_interactions: Some(pending_interactions),
                    delivery_hub: Some(delivery_hub),
                    permission_responder: None,
                })
            })
        },
    )
}

/// Outcome of an atomic durable idempotency claim (R5-03).
///
/// The exclusive create of the claim file is the single authority across
/// runtimes/processes sharing the same storage root. The loser must load the
/// winner's session and discard any speculative session it created.
enum IdempotencyClaim {
    Won,
    Existing { session_id: String },
}

/// Default production spawner. Honest PARTIAL when no real spawn function is
/// injected: the full `spawn_session_on_thread` path is not assembled in this
/// slice (requires HUMAN credentials + agent/tool context + ~80 args wired at
/// the composition root, owned by handoff C2-A). Returning `unsupported` keeps
/// `start_session`/`resume_session` working for storage-backed methods while
/// leaving turn methods `unsupported` until C2-A injects a real spawn function.
///
/// When a [`RealSpawnFn`] is injected via [`ProductionSpawner::with_real_spawn`],
/// `spawn` delegates to it — this is the production-grade seam the composition
/// root uses. The default [`ProductionSpawner::new`] (used by
/// [`ShellSessionActorRuntime::new`]) has no real spawn function and returns
/// `unsupported` with the exact missing-dependency list.
struct ProductionSpawner {
    real: Option<RealSpawnFn>,
}

impl ProductionSpawner {
    /// Build the default production spawner with no real spawn function.
    /// `spawn` returns `unsupported` enumerating the missing dependencies.
    fn new() -> Self {
        Self { real: None }
    }

    /// Build a production spawner backed by a real spawn function injected by
    /// the composition root (C2-A). When `spawn` is called, it delegates to
    /// `real`; the facade method bodies are unchanged.
    fn with_real_spawn(real: RealSpawnFn) -> Self {
        Self { real: Some(real) }
    }
}

#[async_trait]
impl SessionSpawner for ProductionSpawner {
    async fn spawn(
        &self,
        info: &Info,
        model_id: &agent_client_protocol::ModelId,
    ) -> Result<ResidentHandle, RuntimeError> {
        match &self.real {
            Some(real) => real(info.clone(), model_id.clone()).await,
            None => Err(RuntimeError {
                code: "unsupported",
                message: "live SessionActor spawn not assembled in this slice \
                    (C1-J PARTIAL): the composition root (C2-A) must inject a real \
                    spawn function via ProductionSpawner::with_real_spawn or \
                    ShellSessionActorRuntime::with_production_spawn. Missing \
                    production dependencies: HUMAN credentials (api_key / auth \
                    token), AgentDefinition, ToolContext (cwd + permissions), \
                    GatewaySender, ModelsManager, PersistenceHandle, McpServers, \
                    WorkspaceOps, PluginRegistry, AuthManager, SamplingConfig, and \
                    the dedicated thread + LocalSet that spawn_session_on_thread \
                    requires."
                    .into(),
            }),
        }
    }
}

/// Resident bookkeeping per session: the command handle + the next turn
/// ordinal + the next synthesized `event_seq`.
///
/// `next_ordinal` is seeded from `Summary.num_messages` on resident bring-up
/// (C1-H F-2 fix) so ordinals do not collide across process restarts.
/// `next_event_seq` is a per-resident monotonic counter used for synthesized
/// `Item.event_seq` values (C1-H F-1 fix: monotonic, not wall-clock). It is
/// seeded from `Summary.num_messages + 1` so synthesized events stay above
/// the persisted replay range when one exists.
struct Resident {
    handle: ResidentHandle,
    next_ordinal: AtomicU64,
    next_event_seq: AtomicU64,
}

/// Shell-owned `GrokRuntimeFacade` backed by the real JSONL storage adapter.
///
/// Construct with [`ShellSessionActorRuntime::new`] (product uses `grok_home()`;
/// tests pass a `TempDir` root). The adapter is `Send + Sync` because
/// `JsonlStorageAdapter` is `Clone + Send + Sync` and all actor state lives on
/// the actor's own thread (not held here).
pub struct ShellSessionActorRuntime {
    /// Product/storage root (for durable idempotency claims + epoch files).
    root: PathBuf,
    storage: JsonlStorageAdapter,
    /// `idempotency_key -> (session_id, input_digest)` for start-session dedup.
    /// Mirrored on disk under `{root}/app_server_idempotency/` (R4-05).
    idempotency: Mutex<HashMap<String, (String, String)>>,
    /// Per-key async locks for concurrent start-session idempotency claims.
    idempotency_locks: Mutex<HashMap<String, Arc<TokioMutex<()>>>>,
    residents: Mutex<HashMap<String, Resident>>,
    spawn_locks: Mutex<HashMap<String, Arc<TokioMutex<()>>>>,
    spawner: Arc<dyn SessionSpawner>,
    /// True only when a resident factory capable of routing live commands was
    /// injected. Storage-only ports must remain fail-closed for turn methods.
    mutations_enabled: bool,
    last_spawn_error: Mutex<HashMap<String, String>>,
}

impl ShellSessionActorRuntime {
    fn sync_directory_metadata(dir: &Path) -> std::io::Result<()> {
        // Unix exposes directory handles for durable metadata flushes. The
        // Windows standard library has no portable equivalent; the published
        // file itself is still synced there, so do not make every durable
        // update fail merely because the directory handle cannot be opened.
        #[cfg(unix)]
        {
            std::fs::File::open(dir).and_then(|dir_file| dir_file.sync_all())
        }
        #[cfg(not(unix))]
        {
            let _ = dir;
            Ok(())
        }
    }

    /// Build a real port rooted at `root` (product: `grok_home()`; tests: TempDir).
    ///
    /// Uses the `ProductionSpawner` with no real spawn function, which honestly
    /// returns `unsupported` for live actor spawn until the composition root
    /// (C2-A) injects a real spawn function via
    /// [`Self::with_production_spawn`].
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: root.clone(),
            storage: JsonlStorageAdapter::with_root(root),
            idempotency: Mutex::new(HashMap::new()),
            idempotency_locks: Mutex::new(HashMap::new()),
            residents: Mutex::new(HashMap::new()),
            spawn_locks: Mutex::new(HashMap::new()),
            spawner: Arc::new(ProductionSpawner::new()),
            mutations_enabled: false,
            last_spawn_error: Mutex::new(HashMap::new()),
        }
    }

    /// Build a real port over an explicit storage adapter (test seam).
    pub fn with_storage(storage: JsonlStorageAdapter) -> Self {
        // Storage root is not exposed; use a placeholder under /tmp for claims
        // only — prefer `new`/`with_spawner`/`with_production_spawn` for product.
        let root = std::env::temp_dir().join("grok-shell-session-actor-runtime");
        Self {
            root,
            storage,
            idempotency: Mutex::new(HashMap::new()),
            idempotency_locks: Mutex::new(HashMap::new()),
            residents: Mutex::new(HashMap::new()),
            spawn_locks: Mutex::new(HashMap::new()),
            spawner: Arc::new(ProductionSpawner::new()),
            mutations_enabled: false,
            last_spawn_error: Mutex::new(HashMap::new()),
        }
    }

    /// Build a real port with an injected spawn hook (test seam for a real
    /// `cmd_tx` consumer). Storage is rooted at `root`.
    pub fn with_spawner(root: PathBuf, spawner: Arc<dyn SessionSpawner>) -> Self {
        Self {
            root: root.clone(),
            storage: JsonlStorageAdapter::with_root(root),
            idempotency: Mutex::new(HashMap::new()),
            idempotency_locks: Mutex::new(HashMap::new()),
            residents: Mutex::new(HashMap::new()),
            spawn_locks: Mutex::new(HashMap::new()),
            spawner,
            mutations_enabled: true,
            last_spawn_error: Mutex::new(HashMap::new()),
        }
    }

    /// Build a real port with a production spawn function (R4-03 product seam).
    pub fn with_production_spawn(root: PathBuf, real: RealSpawnFn) -> Self {
        Self {
            root: root.clone(),
            storage: JsonlStorageAdapter::with_root(root),
            idempotency: Mutex::new(HashMap::new()),
            idempotency_locks: Mutex::new(HashMap::new()),
            residents: Mutex::new(HashMap::new()),
            spawn_locks: Mutex::new(HashMap::new()),
            spawner: Arc::new(ProductionSpawner::with_real_spawn(real)),
            mutations_enabled: true,
            last_spawn_error: Mutex::new(HashMap::new()),
        }
    }

    /// Compute the input digest for start-session idempotency (matches Fake).
    fn start_digest(params: &SessionStartParams) -> String {
        format!("{}|{:?}", params.workspace_root, params.provider_binding)
    }

    /// Ensure a resident actor exists for `info` (R4-04).
    ///
    /// Returns `Ok(())` when a resident is present (existing or newly spawned).
    /// Returns `Err` when spawn fails — callers that require residency (e.g.
    /// `start_session` on the product path) must surface this as a structured
    /// error rather than returning a silent non-operational session.
    ///
    /// C1-H F-4: per-session async lock serializes concurrent `ensure_resident`.
    async fn ensure_resident(
        &self,
        info: &Info,
        model_id: &agent_client_protocol::ModelId,
    ) -> Result<(), RuntimeError> {
        // Fast path: already resident.
        if self
            .residents
            .lock()
            .unwrap()
            .contains_key(&info.id.0.to_string())
        {
            return Ok(());
        }
        let lock = {
            let mut guard = self.spawn_locks.lock().unwrap();
            guard
                .entry(info.id.0.to_string())
                .or_insert_with(|| Arc::new(TokioMutex::new(())))
                .clone()
        };
        let _spawn_guard = lock.lock().await;
        if self
            .residents
            .lock()
            .unwrap()
            .contains_key(&info.id.0.to_string())
        {
            return Ok(());
        }
        match self.spawner.spawn(info, model_id).await {
            Ok(handle) => {
                let (seed_ordinal, seed_event_seq) = self
                    .storage
                    .load_summary(info)
                    .await
                    .map(|s| {
                        let n = s.num_messages as u64;
                        (n + 1, n + 1)
                    })
                    .unwrap_or((1, 1));
                let mut guard = self.residents.lock().unwrap();
                guard.entry(info.id.0.to_string()).or_insert(Resident {
                    handle,
                    next_ordinal: AtomicU64::new(seed_ordinal),
                    next_event_seq: AtomicU64::new(seed_event_seq),
                });
                self.last_spawn_error
                    .lock()
                    .unwrap()
                    .remove(&info.id.0.to_string());
                Ok(())
            }
            Err(e) => {
                self.last_spawn_error
                    .lock()
                    .unwrap()
                    .insert(info.id.0.to_string(), e.message.clone());
                tracing::warn!(
                    session_id = %info.id.0,
                    code = %e.code,
                    "resident spawn failed"
                );
                Err(e)
            }
        }
    }

    /// Durable start-session idempotency claim path (R4-05).
    fn idempotency_claim_path(&self, key: &str) -> PathBuf {
        // Use a cryptographic digest for the on-disk name. `DefaultHasher` is
        // not a collision-resistant identity primitive and its algorithm is
        // not a stable cross-process contract. The original key remains in
        // the claim digest payload; BLAKE3 only provides a safe, deterministic
        // filename for that payload.
        let digest = blake3::hash(key.as_bytes()).to_hex();
        let name = format!("{digest}.json");
        // Storage root is private; reconstruct from a sentinel via adapter.
        // We keep claims under the same root as sessions via a relative path
        // on the first session dir's parent of "sessions".
        self.idempotency_root().join(name)
    }

    /// Root directory for durable idempotency claims.
    fn idempotency_root(&self) -> PathBuf {
        self.root.join("app_server_idempotency")
    }

    fn session_dir(&self, info: &Info) -> PathBuf {
        self.storage
            .archived_flag_file(info)
            .parent()
            .expect("archived.flag has a parent session dir")
            .to_path_buf()
    }

    /// Load durable (or in-memory) idempotency claim.
    fn load_idempotency_claim(&self, key: &str) -> Option<(String, String)> {
        if let Some(v) = self.idempotency.lock().unwrap().get(key).cloned() {
            return Some(v);
        }
        let path = self.idempotency_claim_path(key);
        let bytes = std::fs::read_to_string(path).ok()?;
        let v: serde_json::Value = serde_json::from_str(&bytes).ok()?;
        let sid = v.get("session_id")?.as_str()?.to_string();
        let dig = v.get("digest")?.as_str()?.to_string();
        self.idempotency
            .lock()
            .unwrap()
            .insert(key.to_string(), (sid.clone(), dig.clone()));
        Some((sid, dig))
    }

    /// Atomically claim `key` for `(session_id, digest)`.
    ///
    /// Writes via temp + exclusive create of the final path so concurrent
    /// losers never observe a half-written claim when the winner has finished.
    fn claim_idempotency(
        &self,
        key: &str,
        session_id: &str,
        digest: &str,
    ) -> Result<IdempotencyClaim, RuntimeError> {
        let dir = self.idempotency_root();
        std::fs::create_dir_all(&dir).map_err(|e| RuntimeError {
            code: "internal_error",
            message: format!("idempotency dir: {e}"),
        })?;
        let path = self.idempotency_claim_path(key);
        let body = serde_json::json!({
            "session_id": session_id,
            "digest": digest,
        })
        .to_string();
        // Write fully to a unique temp, then exclusive create via hard-link or
        // rename-into-place. On Linux, `create_new` + write is not crash-safe
        // for readers; we write temp then `create_new` the final by renaming
        // only if the final does not exist (atomic claim).
        // The temporary path must be unique per *attempt*, not merely per
        // process: multiple independent runtimes can claim the same key from
        // concurrent Tokio tasks in one process. Reusing a PID-only path lets
        // one writer overwrite another writer's fully prepared body before
        // the exclusive hard-link claim is attempted.
        let tmp = dir.join(format!(
            ".{}.tmp-{}",
            path.file_name().and_then(|s| s.to_str()).unwrap_or("claim"),
            uuid::Uuid::now_v7()
        ));
        std::fs::write(&tmp, &body).map_err(|e| RuntimeError {
            code: "internal_error",
            message: format!("idempotency temp write: {e}"),
        })?;
        // The exclusive link below makes the winner selection atomic, but it
        // does not by itself flush the claim body to stable storage. Sync the
        // fully-written temporary file before publishing its name so a crash
        // cannot leave a visible winner whose JSON is still only in page cache.
        if let Err(e) = std::fs::File::open(&tmp).and_then(|file| file.sync_all()) {
            let _ = std::fs::remove_file(&tmp);
            return Err(RuntimeError {
                code: "internal_error",
                message: format!("idempotency temp sync: {e}"),
            });
        }
        match std::fs::hard_link(&tmp, &path).or_else(|_| {
            // Fallback when hard_link unsupported: exclusive create + copy.
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut f) => {
                    use std::io::Write;
                    f.write_all(body.as_bytes()).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                    })?;
                    // A fallback exclusive-create is still a durable claim
                    // contract: do not report Won if the published body
                    // could not be flushed to stable storage.
                    f.sync_all().map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                    })?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }) {
            Ok(()) => {
                // The claim body is synced above, but the directory entry
                // also needs a metadata flush before a restart can reliably
                // rediscover the winner by filename.
                if let Err(e) = Self::sync_directory_metadata(&dir) {
                    // The claim was already linked into the namespace, so a
                    // failed directory flush must undo that publication too;
                    // removing only the temp would leave a durable-looking
                    // winner whose caller is about to roll back the session.
                    let _ = std::fs::remove_file(&path);
                    let _ = std::fs::remove_file(&tmp);
                    return Err(RuntimeError {
                        code: "internal_error",
                        message: format!("idempotency directory sync: {e}"),
                    });
                }
                let _ = std::fs::remove_file(&tmp);
                self.idempotency.lock().unwrap().insert(
                    key.to_string(),
                    (session_id.to_string(), digest.to_string()),
                );
                Ok(IdempotencyClaim::Won)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let _ = std::fs::remove_file(&tmp);
                // Winner already claimed — load durable claim (retry briefly
                // if the winner is still writing via the fallback path).
                let (sid, dig) =
                    self.load_idempotency_claim_retry(key)
                        .ok_or_else(|| RuntimeError {
                            code: "internal_error",
                            message: "idempotency claim exists but could not be read".into(),
                        })?;
                if dig != digest {
                    return Err(RuntimeError {
                        code: "idempotency_conflict",
                        message: "The idempotency key was already used with different input."
                            .into(),
                    });
                }
                Ok(IdempotencyClaim::Existing { session_id: sid })
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(RuntimeError {
                    code: "internal_error",
                    message: format!("idempotency create: {e}"),
                })
            }
        }
    }

    /// Load a claim with short retries for concurrent winner write races.
    fn load_idempotency_claim_retry(&self, key: &str) -> Option<(String, String)> {
        for _ in 0..20 {
            if let Some(v) = self.load_idempotency_claim(key) {
                return Some(v);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        None
    }

    /// Per-session history epoch (R4-07 / R5-04).
    fn history_epoch_for(&self, info: &Info) -> Result<String, RuntimeError> {
        let path = self.session_dir(info).join(HISTORY_EPOCH_FILE);
        match std::fs::read_to_string(&path) {
            Ok(s) => {
                let t = s.trim();
                if t.is_empty() {
                    return Err(RuntimeError {
                        code: "internal_error",
                        message: "history epoch file is empty".into(),
                    });
                }
                Ok(t.to_string())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Legacy sessions may predate the durable epoch sidecar.
                Ok(HISTORY_EPOCH_DEFAULT.to_string())
            }
            Err(e) => Err(RuntimeError {
                code: "internal_error",
                message: format!("history epoch read: {e}"),
            }),
        }
    }

    /// Persist a history epoch before exposing it in a projected session.
    ///
    /// Epochs are part of the replay contract, so silently swallowing an I/O
    /// error would let callers advertise an identity that cannot survive a
    /// restart. Write and sync a unique temporary file, then atomically
    /// replace the epoch file.
    fn write_history_epoch(&self, info: &Info, epoch: &str) -> Result<(), RuntimeError> {
        let path = self.session_dir(info).join(HISTORY_EPOCH_FILE);
        let parent = path.parent().ok_or_else(|| RuntimeError {
            code: "internal_error",
            message: "history epoch path has no parent".into(),
        })?;
        let tmp = parent.join(format!(".history_epoch.{}.tmp", uuid::Uuid::now_v7()));
        let result = (|| {
            let mut file = std::fs::File::create(&tmp).map_err(|e| RuntimeError {
                code: "internal_error",
                message: format!("history epoch create: {e}"),
            })?;
            use std::io::Write;
            file.write_all(format!("{epoch}\n").as_bytes())
                .map_err(|e| RuntimeError {
                    code: "internal_error",
                    message: format!("history epoch write: {e}"),
                })?;
            file.sync_all().map_err(|e| RuntimeError {
                code: "internal_error",
                message: format!("history epoch sync: {e}"),
            })?;
            if let Err(e) = std::fs::rename(&tmp, &path) {
                // Windows does not replace an existing destination on rename.
                // Preserve the same API there with a narrow remove-and-rename
                // fallback; Unix keeps the atomic replacement path above.
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    std::fs::remove_file(&path).map_err(|remove_err| RuntimeError {
                        code: "internal_error",
                        message: format!("history epoch replace: {remove_err}"),
                    })?;
                    std::fs::rename(&tmp, &path).map_err(|rename_err| RuntimeError {
                        code: "internal_error",
                        message: format!("history epoch publish: {rename_err}"),
                    })?;
                } else {
                    return Err(RuntimeError {
                        code: "internal_error",
                        message: format!("history epoch publish: {e}"),
                    });
                }
            }
            Self::sync_directory_metadata(parent).map_err(|e| RuntimeError {
                code: "internal_error",
                message: format!("history epoch directory sync: {e}"),
            })?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        result
    }

    /// Ensure a unique history epoch file exists for a new session stream.
    fn ensure_history_epoch(&self, info: &Info) -> Result<String, RuntimeError> {
        let path = self.session_dir(info).join(HISTORY_EPOCH_FILE);
        if path.is_file() {
            // Return the value we validated instead of forcing callers to do
            // another fallible read after publishing an idempotency claim.
            return self.history_epoch_for(info);
        }
        let epoch = format!("epoch_{}", uuid::Uuid::now_v7());
        self.write_history_epoch(info, &epoch)?;
        Ok(epoch)
    }

    /// Rotate the history epoch (invalidates prior cursors) when the stream
    /// identity changes — rewrite, truncate, or reconstruction (R5-04).
    pub fn rotate_history_epoch(&self, info: &Info) -> Result<String, RuntimeError> {
        let epoch = format!("epoch_{}", uuid::Uuid::now_v7());
        self.write_history_epoch(info, &epoch)?;
        Ok(epoch)
    }

    /// Apply residency result consistently (R5-02): hard spawn failures
    /// propagate; intentional `unsupported` (no factory) is soft — storage
    /// methods remain available and turn methods surface the spawn detail.
    fn residency_result(result: Result<(), RuntimeError>) -> Result<(), RuntimeError> {
        match result {
            Ok(()) => Ok(()),
            Err(e) if e.code == "unsupported" => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Load an existing session row after an idempotent claim win by another
    /// runtime/process, re-residing best-effort.
    async fn load_existing_session_row(&self, session_id: &str) -> Result<Session, RuntimeError> {
        let info = self.find_info(session_id).await?;
        let summary = self
            .storage
            .load_summary(&info)
            .await
            .map_err(io_err_to_runtime)?;
        let binding = self.load_binding(&info).await;
        let archived = self.storage.is_archived(&info);
        let epoch = self.history_epoch_for(&info)?;
        Self::residency_result(self.ensure_resident(&info, &default_model_id()).await)?;
        Ok(project_summary_to_session(
            &summary,
            binding.as_ref(),
            archived,
            &epoch,
        ))
    }

    /// Look up the resident handle for a session id. Returns `None` when no
    /// live actor is resident (production PARTIAL — turn methods return
    /// `unsupported`). Exposed for tests that need to observe the running-turn
    /// slot; not part of the `GrokRuntimeFacade` trait.
    pub fn resident(&self, session_id: &str) -> Option<ResidentHandle> {
        self.residents
            .lock()
            .unwrap()
            .get(session_id)
            .map(|r| ResidentHandle {
                cmd_tx: r.handle.cmd_tx.clone(),
                current_prompt_id: r.handle.current_prompt_id.clone(),
                pending_interactions: r.handle.pending_interactions.clone(),
                delivery_hub: r.handle.delivery_hub.clone(),
                permission_responder: r.handle.permission_responder.clone(),
            })
    }

    /// Allocate the next turn ordinal for a resident session.
    ///
    /// The atomic holds the *next* ordinal to assign. `fetch_add(1)` returns
    /// the current value then increments, so a seed of `1` yields first
    /// ordinal `1` (matches FakeRuntime / C7-B F-2).
    fn next_ordinal(&self, session_id: &str) -> u64 {
        let mut guard = self.residents.lock().unwrap();
        match guard.get_mut(session_id) {
            Some(r) => r.next_ordinal.fetch_add(1, Ordering::Relaxed),
            None => 1,
        }
    }

    /// Allocate the next synthesized `event_seq` for a resident session
    /// (C1-H F-1 fix: monotonic per session, not wall-clock). Returns 0 when
    /// no resident exists (the caller checks residency first).
    fn next_event_seq(&self, session_id: &str) -> u64 {
        let mut guard = self.residents.lock().unwrap();
        match guard.get_mut(session_id) {
            Some(r) => r.next_event_seq.fetch_add(1, Ordering::Relaxed),
            None => 0,
        }
    }

    /// Clear the running-turn slot for a resident session (C1-H F-3 fix:
    /// reduce stale-handle risk). Called when the actor command channel is
    /// detected closed (fire-and-forget `send` failed, or the `start_turn`
    /// oneshot was dropped) — a dead actor never clears `current_prompt_id`
    /// itself, so the adapter clears it to keep the turn-id guard honest.
    fn clear_current_turn(&self, session_id: &str) {
        if let Some(r) = self.residents.lock().unwrap().get(session_id)
            && let Ok(mut g) = r.handle.current_prompt_id.lock()
        {
            *g = None;
        }
    }

    /// Drop a speculative resident created before an idempotency loser is
    /// discarded. Dropping the last command sender lets the actor thread
    /// observe channel closure; the durable session row is removed separately
    /// by the caller.
    fn remove_resident(&self, session_id: &str) {
        self.residents.lock().unwrap().remove(session_id);
        self.last_spawn_error.lock().unwrap().remove(session_id);
    }

    /// Build the `unsupported` error returned by turn methods when no resident
    /// actor exists. Includes the last spawn error message for the session
    /// (the actionable BLOCKER — exact missing production deps) when the
    /// spawner returned an error, so the caller sees WHY there is no
    /// resident rather than a generic "no resident" message.
    fn no_resident_error(&self, session_id: &str) -> RuntimeError {
        let detail = self
            .last_spawn_error
            .lock()
            .unwrap()
            .get(session_id)
            .cloned()
            .unwrap_or_else(|| {
                "no resident SessionActor for this session (no spawn attempted yet).".to_string()
            });
        RuntimeError {
            code: "unsupported",
            message: format!(
                "no resident SessionActor for this session (C1-J PARTIAL: spawn not assembled). \
                 Composition root (C2-A) must inject a real spawn function via \
                 ShellSessionActorRuntime::with_production_spawn. Spawn error: {detail}"
            ),
        }
    }

    /// Find a session's `Info` by id across all workspaces (O(n) scan).
    ///
    /// `SessionResumeParams` / `SubscribeParams` carry no `cwd`, but the JSONL
    /// layout is `{root}/sessions/{urlencoded(cwd)}/{session_id}/`. Resolving
    /// the cwd requires reading the summary, which requires the path, which
    /// requires the cwd — so we scan all summaries and match by id (R4).
    async fn find_info(&self, session_id: &str) -> Result<Info, RuntimeError> {
        let summaries = self
            .storage
            .list_sessions(None)
            .await
            .map_err(io_err_to_runtime)?;
        for s in summaries {
            if s.info.id.0.as_ref() == session_id {
                return Ok(s.info);
            }
        }
        Err(RuntimeError {
            code: "session_not_found",
            message: "Session was not found.".into(),
        })
    }

    /// Load the identifier-only [`ProviderBinding`] sidecar for `info`, if any
    /// (C5-C). Best-effort: returns `None` when the sidecar is absent (session
    /// started without a binding, or before C5-C landed) or unreadable
    /// (corrupt/missing file). The sidecar contains **no** secret material —
    /// only `provider_id` / `credential_id` / `model_id` / `backend` /
    /// `binding_revision`. Reads happen on a blocking thread pool via
    /// `spawn_blocking` because the sidecar is a small JSON file.
    async fn load_binding(&self, info: &Info) -> Option<ProviderBinding> {
        let path = self.storage.provider_binding_file(info);
        tokio::task::spawn_blocking(move || {
            if !path.is_file() {
                return None;
            }
            match std::fs::read_to_string(&path) {
                Ok(bytes) => serde_json::from_str::<ProviderBinding>(&bytes).ok(),
                Err(_) => None,
            }
        })
        .await
        .ok()
        .flatten()
    }

    /// Persist the identifier-only [`ProviderBinding`] sidecar for `info`
    /// (C5-C). Overwrites any existing sidecar (start-session is the
    /// authority; a re-start with the same idempotency key returns the
    /// existing session without re-writing). Writes pretty JSON on a blocking
    /// thread. The sidecar contains **no** secret material by contract
    /// (`ProviderBinding` is identifier-only — see
    /// `provider_binding_is_structured_and_contains_no_secret_material`).
    async fn write_binding(
        &self,
        info: &Info,
        binding: &ProviderBinding,
    ) -> Result<(), RuntimeError> {
        let path = self.storage.provider_binding_file(info);
        let bytes = serde_json::to_vec_pretty(binding).map_err(|e| RuntimeError {
            code: "storage_error",
            message: format!("failed to serialize provider binding: {e}"),
        })?;
        tokio::task::spawn_blocking(move || {
            // The session dir is created by `init_session` before we get here,
            // so `write` is safe. If the dir is somehow missing, create it
            // best-effort so the sidecar is not silently dropped.
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(&path, bytes)
        })
        .await
        .map_err(|e| RuntimeError {
            code: "storage_error",
            message: format!("provider binding write join failed: {e}"),
        })?
        .map_err(io_err_to_runtime)
    }

    /// Project the canonical `updates.jsonl` for `info` into the shared
    /// [`ProjectedHistory`] (events for `replay`, turns + items for
    /// `read_session`). Returns an empty history when the updates file is
    /// absent or unreadable (honest — no data, no projection). This is the
    /// single projection surface shared by R2 and R11; it is NOT a second
    /// replay buffer.
    ///
    /// `binding` is the session's identifier-only [`ProviderBinding`] (C5-C),
    /// projected onto every inferred `Turn` row so `read_session` turns
    /// carry the same binding as the parent `Session`. `None` when no
    /// sidecar exists.
    fn project_history(
        &self,
        info: &Info,
        session_id: &str,
        binding: Option<ProviderBinding>,
    ) -> ProjectedHistory {
        let Some(path) = self.storage.updates_file_path(info) else {
            return ProjectedHistory {
                events: Vec::new(),
                turns: Vec::new(),
                items: Vec::new(),
            };
        };
        let Ok(Some(iter)) = UpdatesIterator::open(&path) else {
            return ProjectedHistory {
                events: Vec::new(),
                turns: Vec::new(),
                items: Vec::new(),
            };
        };
        project_updates(session_id, iter, binding)
    }
}

fn io_err_to_runtime(e: std::io::Error) -> RuntimeError {
    RuntimeError {
        code: "storage_error",
        message: e.to_string(),
    }
}

/// Convert facade `InputBlock`s into ACP `ContentBlock`s for `SessionCommand::Prompt`.
///
/// Minimal real conversion: `Text` → `ContentBlock::Text`; `Mention`/`Skill`
/// flatten to their `name` as text (the actor's `parse_prompt` does the rich
/// rendering in production; the adapter only needs a faithful wire shape to
/// enqueue the command). This is NOT a second parser — it preserves intent.
fn input_blocks_to_content_blocks(
    input: &[xai_grok_app_server_protocol::InputBlock],
) -> Vec<agent_client_protocol::ContentBlock> {
    use agent_client_protocol as acp;
    input
        .iter()
        .map(|b| match b {
            xai_grok_app_server_protocol::InputBlock::Text { text } => {
                acp::ContentBlock::Text(acp::TextContent::new(text.clone()))
            }
            xai_grok_app_server_protocol::InputBlock::Mention { name, .. } => {
                acp::ContentBlock::Text(acp::TextContent::new(name.clone()))
            }
            xai_grok_app_server_protocol::InputBlock::Skill { name, .. } => {
                acp::ContentBlock::Text(acp::TextContent::new(name.clone()))
            }
        })
        .collect()
}

/// Flatten facade `InputBlock`s to a single text string for `Interject`.
fn input_blocks_to_text(input: &[xai_grok_app_server_protocol::InputBlock]) -> String {
    input
        .iter()
        .map(|b| match b {
            xai_grok_app_server_protocol::InputBlock::Text { text } => text.clone(),
            xai_grok_app_server_protocol::InputBlock::Mention { name, .. } => name.clone(),
            xai_grok_app_server_protocol::InputBlock::Skill { name, .. } => name.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Project a Shell `Summary` into a protocol `Session` (C0-B §1.1 R1).
///
/// This replaces the dormant `project_active_session_row` stub: status,
/// workspace, timestamps, and title come from the real on-disk summary, not
/// hardcoded `epoch_1`/revision 0/`Dormant`.
///
/// `binding` is the identifier-only [`ProviderBinding`] persisted in the
/// `provider_binding.json` sidecar (C5-C). It is projected verbatim onto the
/// `Session.provider_binding` field when present, and `None` when no sidecar
/// exists (e.g. a session started before C5-C, or a session whose
/// `SessionStartParams` carried no binding). The binding contains **no**
/// secret material — only `provider_id` / `credential_id` / `model_id` /
/// `backend` / `binding_revision`.
fn project_summary_to_session(
    summary: &Summary,
    binding: Option<&ProviderBinding>,
    archived: bool,
    history_epoch: &str,
) -> Session {
    let title = summary.display_title();
    let title = if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    };
    // Shell `Summary` has no live-status field; a persisted session with no
    // actor resident is `Ready` (resumable). Archive is a reversible hide
    // marker on disk (R6), not delete. `Running`/`WaitingForInput` require
    // the live actor (PARTIAL — not represented here).
    let status = if archived {
        SessionStatus::Archived
    } else if summary.num_messages == 0 {
        SessionStatus::Starting
    } else {
        SessionStatus::Ready
    };
    let revision = WireCounter::new(summary.num_messages.max(1) as u64);
    Session {
        session_id: summary.info.id.0.to_string(),
        history_epoch: history_epoch.to_string(),
        revision,
        status,
        workspace_root: summary.info.cwd.clone(),
        title,
        active_turn_id: None,
        latest_turn_id: None,
        // C5-C: project the identifier-only `ProviderBinding` from the
        // `provider_binding.json` sidecar (no secrets). `None` when the
        // session was started without a binding or before C5-C landed.
        provider_binding: binding.cloned(),
        created_at_ms: summary.created_at.timestamp_millis() as u64,
        updated_at_ms: summary.updated_at.timestamp_millis() as u64,
    }
}

/// Projected history from `updates.jsonl` — the single projection surface
/// shared by `read_session` (Turn/Item) and `replay` (RuntimeEvent).
///
/// This is **NOT a second replay buffer**: it reads `updates.jsonl` once via
/// `UpdatesIterator` and derives all three views (`events`, `turns`, `items`)
/// from the same pass. `read_session` and `replay` share this projector so
/// there is one projection truth over the canonical `updates.jsonl`, not a
/// second execution authority.
///
/// ## What is REAL vs PARTIAL (C3-F, R2/R11)
/// - **REAL:** `UserMessageChunk` → `ItemCompleted(UserMessage)`;
///   `AgentMessageChunk` → `ItemDelta` (replay) + `ItemCompleted(AgentMessage)`
///   (read_session, per-chunk — no grouping); `AgentThoughtChunk` →
///   `ItemCompleted(ReasoningSummary)`; `ToolCall` → `ItemStarted(ToolCall)`;
///   `ToolCallUpdate` with a status → `ItemCompleted(ToolCall)` correlated to
///   the original `ToolCall` via `tool_call_id`; `Plan` → `ItemCompleted(Plan)`.
/// - **PARTIAL (Shell never writes these):**
///   - `TurnChanged` is **not emitted** in replay — Shell writes no turn
///     lifecycle events. `read_session.turns` are inferred from
///     `UserMessageChunk` boundaries with `status: Completed` (inferred from
///     persistence; crash-mid-turn not detected).
///   - Item grouping across streaming chunks is not performed (each chunk is
///     a separate item — Shell writes no item-id correlation for text
///     chunks).
///   - `InteractionRequested` is not projected — Shell interaction requests
///     are in-memory only (`pending_interaction.rs`), never persisted to
///     `updates.jsonl`.
///   - `created_at_ms` is `0` — `UpdatesIterator` parses `SessionUpdate`
///     (discarding the envelope `timestamp`); exposing it would require
///     changing the shared `UpdatesIterator` symbol (out of scope).
///   - xAI extension updates (`RewindMarker`, `AutoCompact*`, `Memory*`,
///     `Subagent*`, etc.) are skipped — they have no `RuntimeEvent`
///     representation; rewind/compaction/subagent projection is deferred.
struct ProjectedHistory {
    /// `RuntimeEvent` stream (events 1..N from `updates.jsonl`; the
    /// `SessionChanged` snapshot at index 0 is prepended separately by
    /// `replay`).
    events: Vec<RuntimeEvent>,
    /// Turns inferred from `UserMessageChunk` boundaries.
    turns: Vec<Turn>,
    /// Items projected per-line in order.
    items: Vec<Item>,
}

/// Map an ACP `ToolCallStatus` to the protocol `ItemStatus`.
fn tool_call_status_to_item_status(status: &agent_client_protocol::ToolCallStatus) -> ItemStatus {
    use agent_client_protocol as acp;
    match status {
        acp::ToolCallStatus::Pending => ItemStatus::Pending,
        acp::ToolCallStatus::InProgress => ItemStatus::InProgress,
        acp::ToolCallStatus::Completed => ItemStatus::Completed,
        acp::ToolCallStatus::Failed => ItemStatus::Failed,
        // `ToolCallStatus` is `non_exhaustive`; unknown future variants map to
        // `Pending` (the ACP default) so projection never panics on new
        // statuses Shell may start writing. PARTIAL — status fidelity for
        // unknown variants.
        _ => ItemStatus::Pending,
    }
}

/// Project one `SessionUpdate` line into a replay `RuntimeEvent` and/or a
/// `read_session` `Item`. `turn_id` is the current turn (the caller resolves
/// turn boundaries from `UserMessageChunk` before calling this). Returns
/// `(event, item)`; either may be `None` when the update has no projection
/// (honestly PARTIAL — see [`ProjectedHistory`] docs).
fn project_line(
    session_id: &str,
    seq: u64,
    turn_id: &str,
    update: &crate::session::storage::SessionUpdate,
) -> (Option<RuntimeEvent>, Option<Item>) {
    use agent_client_protocol as acp;
    let event_seq = WireCounter::new(seq);
    let item_id = format!("item_{seq}");
    match update {
        crate::session::storage::SessionUpdate::Acp(n) => match &n.update {
            acp::SessionUpdate::UserMessageChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    let item = Item {
                        item_id,
                        session_id: session_id.to_string(),
                        turn_id: turn_id.to_string(),
                        event_seq,
                        revision: WireCounter::new(1),
                        status: ItemStatus::Completed,
                        created_at_ms: 0,
                        body: ItemBody::UserMessage {
                            content: vec![xai_grok_app_server_protocol::InputBlock::Text {
                                text: text.text.clone(),
                            }],
                        },
                    };
                    (Some(RuntimeEvent::ItemCompleted(item.clone())), Some(item))
                } else {
                    (None, None)
                }
            }
            acp::SessionUpdate::AgentMessageChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    let delta_event = RuntimeEvent::ItemDelta {
                        session_id: session_id.to_string(),
                        turn_id: turn_id.to_string(),
                        item_id: item_id.clone(),
                        revision: WireCounter::new(1),
                        delta: text.text.clone(),
                    };
                    // read_session: each chunk as a completed message fragment
                    // (PARTIAL — no item grouping across streaming chunks).
                    let item = Item {
                        item_id,
                        session_id: session_id.to_string(),
                        turn_id: turn_id.to_string(),
                        event_seq,
                        revision: WireCounter::new(1),
                        status: ItemStatus::Completed,
                        created_at_ms: 0,
                        body: ItemBody::AgentMessage {
                            text: text.text.clone(),
                        },
                    };
                    (Some(delta_event), Some(item))
                } else {
                    (None, None)
                }
            }
            acp::SessionUpdate::AgentThoughtChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    let item = Item {
                        item_id,
                        session_id: session_id.to_string(),
                        turn_id: turn_id.to_string(),
                        event_seq,
                        revision: WireCounter::new(1),
                        status: ItemStatus::Completed,
                        created_at_ms: 0,
                        body: ItemBody::ReasoningSummary {
                            summary: text.text.clone(),
                        },
                    };
                    (Some(RuntimeEvent::ItemCompleted(item.clone())), Some(item))
                } else {
                    (None, None)
                }
            }
            acp::SessionUpdate::ToolCall(tc) => {
                // item_id correlates ToolCall + ToolCallUpdate via tool_call_id
                // so the replay lifecycle is ItemStarted → ItemCompleted with
                // the same item_id (no second buffer — derived from the ACP
                // id already present in the update).
                let tc_item_id = format!("tc_{}", tc.tool_call_id);
                let item = Item {
                    item_id: tc_item_id,
                    session_id: session_id.to_string(),
                    turn_id: turn_id.to_string(),
                    event_seq,
                    revision: WireCounter::new(1),
                    status: tool_call_status_to_item_status(&tc.status),
                    created_at_ms: 0,
                    body: ItemBody::ToolCall {
                        tool_name: tc.title.clone(),
                        arguments: tc.raw_input.clone().unwrap_or(serde_json::Value::Null),
                    },
                };
                (Some(RuntimeEvent::ItemStarted(item.clone())), Some(item))
            }
            acp::SessionUpdate::ToolCallUpdate(tcu) => {
                // Correlate with the original ToolCall via tool_call_id
                // (item_id = "tc_{tool_call_id}"). Emit ItemCompleted when a
                // status is present; otherwise skip (PARTIAL — non-status
                // updates have no RuntimeEvent lifecycle representation).
                if let Some(status) = &tcu.fields.status {
                    let tc_item_id = format!("tc_{}", tcu.tool_call_id);
                    let item = Item {
                        item_id: tc_item_id,
                        session_id: session_id.to_string(),
                        turn_id: turn_id.to_string(),
                        event_seq,
                        revision: WireCounter::new(1),
                        status: tool_call_status_to_item_status(status),
                        created_at_ms: 0,
                        body: ItemBody::ToolCall {
                            tool_name: tcu
                                .fields
                                .title
                                .clone()
                                .unwrap_or_else(|| tcu.tool_call_id.to_string()),
                            arguments: tcu
                                .fields
                                .raw_input
                                .clone()
                                .unwrap_or(serde_json::Value::Null),
                        },
                    };
                    (Some(RuntimeEvent::ItemCompleted(item.clone())), Some(item))
                } else {
                    (None, None)
                }
            }
            acp::SessionUpdate::Plan(plan) => {
                let steps: Vec<serde_json::Value> = plan
                    .entries
                    .iter()
                    .map(|e| serde_json::to_value(e).unwrap_or(serde_json::Value::Null))
                    .collect();
                let content = steps
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                let item = Item {
                    item_id,
                    session_id: session_id.to_string(),
                    turn_id: turn_id.to_string(),
                    event_seq,
                    revision: WireCounter::new(1),
                    status: ItemStatus::Completed,
                    created_at_ms: 0,
                    body: ItemBody::Plan { content, steps },
                };
                (Some(RuntimeEvent::ItemCompleted(item.clone())), Some(item))
            }
            // SessionInfoUpdate / AvailableCommandsUpdate / CurrentModeUpdate /
            // ConfigOptionUpdate are session-meta updates, not item lifecycle
            // events. Skipped honestly (PARTIAL — no RuntimeEvent variant).
            _ => (None, None),
        },
        crate::session::storage::SessionUpdate::Xai(_) => {
            // xAI extension updates (RewindMarker, AutoCompact, Memory,
            // Subagent*, etc.) are not ACP item lifecycle events and have no
            // RuntimeEvent representation. Skipped honestly (PARTIAL —
            // rewind / compaction / subagent projection deferred).
            (None, None)
        }
    }
}

/// Project all `updates.jsonl` lines into `events` (replay), `turns` and
/// `items` (read_session) in one pass. This is the single projection surface
/// — `read_session` and `replay` share it without a second replay buffer.
///
/// Turn boundaries are inferred from `UserMessageChunk` (Shell writes no
/// explicit turn-start event). Items before the first user message, if any,
/// attach to a synthetic `turn_0` (rare in practice — Shell writes the user
/// message first).
fn project_updates(
    session_id: &str,
    updates: impl Iterator<Item = std::io::Result<crate::session::storage::SessionUpdate>>,
    binding: Option<ProviderBinding>,
) -> ProjectedHistory {
    use agent_client_protocol as acp;
    let mut events: Vec<RuntimeEvent> = Vec::new();
    let mut turns: Vec<Turn> = Vec::new();
    let mut items: Vec<Item> = Vec::new();
    let mut seq = 1u64;
    let mut current_turn_id: String = "turn_0".to_string();
    let mut turn_ordinal: u64 = 0;

    for update in updates {
        let update = match update {
            Ok(u) => u,
            Err(_) => {
                seq += 1;
                continue;
            }
        };
        // Turn boundary: a UserMessageChunk starts a new turn. Shell writes
        // no explicit turn-start event, so turn_id is synthesized from the
        // user message's line number. Turn status is inferred `Completed`
        // from persistence (PARTIAL — crash-mid-turn not detected).
        let starts_new_turn = matches!(
            &update,
            crate::session::storage::SessionUpdate::Acp(n)
                if matches!(n.update, acp::SessionUpdate::UserMessageChunk(_))
        );
        if starts_new_turn {
            turn_ordinal += 1;
            current_turn_id = format!("turn_{seq}");
            turns.push(Turn {
                turn_id: current_turn_id.clone(),
                session_id: session_id.to_string(),
                // C5-C: project the session's identifier-only ProviderBinding
                // onto every inferred turn row (no secrets). `None` when the
                // session has no binding sidecar.
                provider_binding: binding.clone(),
                revision: WireCounter::new(1),
                status: TurnStatus::Completed,
                kind: TurnKind::User,
                ordinal: turn_ordinal,
                created_at_ms: 0,
                completed_at_ms: None,
            });
        }
        let (event, item) = project_line(session_id, seq, &current_turn_id, &update);
        if let Some(e) = event {
            events.push(e);
        }
        if let Some(i) = item {
            items.push(i);
        }
        seq += 1;
    }
    ProjectedHistory {
        events,
        turns,
        items,
    }
}

#[async_trait]
impl GrokRuntimeFacade for ShellSessionActorRuntime {
    fn capabilities(&self) -> RuntimeCapabilities {
        // Storage-backed session/replay paths are executable today. Turn
        // mutation is advertised only when a live resident factory was
        // injected; Interaction/items remain fail-closed until their own
        // product gates are complete.
        RuntimeCapabilities {
            session_list: true,
            session_read: true,
            session_start: true,
            session_resume: true,
            session_fork: true,
            session_archive: true,
            session_subscribe: true,
            turn_start: self.mutations_enabled,
            turn_steer: self.mutations_enabled,
            turn_interrupt: self.mutations_enabled,
            interaction_respond: false,
            item_lifecycle: false,
            item_deltas: false,
        }
    }

    async fn list_sessions(&self) -> Result<Vec<Session>, RuntimeError> {
        let summaries = self
            .storage
            .list_sessions(None)
            .await
            .map_err(io_err_to_runtime)?;
        // C5-C: project the identifier-only ProviderBinding sidecar onto each
        // session row. This is N best-effort sidecar reads (one per session);
        // the sidecar is a small JSON file and absent for pre-C5-C sessions.
        // Sessions whose sidecar is missing/corrupt project `None` (honest).
        let mut sessions = Vec::with_capacity(summaries.len());
        for s in &summaries {
            let binding = self.load_binding(&s.info).await;
            let archived = self.storage.is_archived(&s.info);
            let epoch = self.history_epoch_for(&s.info)?;
            sessions.push(project_summary_to_session(
                s,
                binding.as_ref(),
                archived,
                &epoch,
            ));
        }
        Ok(sessions)
    }

    async fn read_session(
        &self,
        params: SessionReadParams,
    ) -> Result<SessionReadResult, RuntimeError> {
        let info = self.find_info(&params.session_id).await?;
        let summary = self
            .storage
            .load_summary(&info)
            .await
            .map_err(io_err_to_runtime)?;
        // C5-C: load the identifier-only ProviderBinding sidecar (no secrets)
        // and project it onto the Session row and every inferred Turn row.
        let binding = self.load_binding(&info).await;
        let archived = self.storage.is_archived(&info);
        let epoch = self.history_epoch_for(&info)?;
        let session = project_summary_to_session(&summary, binding.as_ref(), archived, &epoch);
        // R2: project Turn/Item from `updates.jsonl` via the shared projector
        // (same surface as R11 `replay` — no second buffer). PARTIAL: turn
        // status inferred `Completed` from persistence (Shell writes no turn
        // lifecycle events); item grouping across streaming chunks not
        // performed; `created_at_ms` is 0 (`UpdatesIterator` does not expose
        // the envelope timestamp); xAI extension updates skipped.
        let (turns, items) = if params.include_turns || params.include_items {
            let history = self.project_history(&info, &params.session_id, binding.clone());
            (
                if params.include_turns {
                    history.turns
                } else {
                    Vec::new()
                },
                if params.include_items {
                    history.items
                } else {
                    Vec::new()
                },
            )
        } else {
            (Vec::new(), Vec::new())
        };
        Ok(SessionReadResult {
            session,
            turns,
            items,
        })
    }

    async fn start_session(&self, params: SessionStartParams) -> Result<Session, RuntimeError> {
        let digest = Self::start_digest(&params);
        // R4-05: serialize concurrent same-key starts + durable claim.
        let key_lock = {
            let mut g = self.idempotency_locks.lock().unwrap();
            g.entry(params.idempotency_key.clone())
                .or_insert_with(|| Arc::new(TokioMutex::new(())))
                .clone()
        };
        let _key_guard = key_lock.lock().await;

        if let Some((existing_id, prev_digest)) =
            self.load_idempotency_claim(&params.idempotency_key)
        {
            if prev_digest != digest {
                return Err(RuntimeError {
                    code: "idempotency_conflict",
                    message: "The idempotency key was already used with different input.".into(),
                });
            }
            return self.load_existing_session_row(&existing_id).await;
        }
        if params.workspace_root.is_empty() || !params.workspace_root.starts_with('/') {
            return Err(RuntimeError {
                code: "invalid_workspace",
                message: "The workspace cannot be opened.".into(),
            });
        }
        let session_id = uuid::Uuid::now_v7().to_string();
        let info = Info {
            id: agent_client_protocol::SessionId::new(session_id.clone()),
            cwd: params.workspace_root.clone(),
        };
        let summary = self
            .storage
            .init_session(&info, default_model_id())
            .await
            .map_err(io_err_to_runtime)?;
        let epoch = match self.ensure_history_epoch(&info) {
            Ok(epoch) => epoch,
            Err(e) => {
                let _ = self.storage.delete_session(&info).await;
                return Err(e);
            }
        };
        if let Some(binding) = &params.provider_binding {
            if let Err(e) = self.write_binding(&info, binding).await {
                let _ = self.storage.delete_session(&info).await;
                return Err(e);
            }
        }
        // R5-02: hard spawn failures roll back; intentional `unsupported`
        // (no factory) keeps a storage-only session. Spawn before publishing
        // the claim so another runtime cannot observe a winner that is being
        // rolled back.
        match self.ensure_resident(&info, &default_model_id()).await {
            Ok(()) => {}
            Err(e) if e.code == "unsupported" => {}
            Err(e) => {
                let _ = self.storage.delete_session(&info).await;
                return Err(RuntimeError {
                    code: "spawn_failed",
                    message: format!(
                        "session start rolled back: resident spawn failed: {}",
                        e.message
                    ),
                });
            }
        }
        // R5-03: claim is the atomic authority after speculative creation has
        // reached a stable residency state. On Existing, discard both the
        // speculative resident and its durable session row before returning
        // the winner's row.
        let claim = match self.claim_idempotency(&params.idempotency_key, &session_id, &digest) {
            Ok(claim) => claim,
            Err(e) => {
                self.remove_resident(&session_id);
                let _ = self.storage.delete_session(&info).await;
                return Err(e);
            }
        };
        match claim {
            IdempotencyClaim::Existing {
                session_id: winner_id,
            } => {
                self.remove_resident(&session_id);
                let _ = self.storage.delete_session(&info).await;
                return self.load_existing_session_row(&winner_id).await;
            }
            IdempotencyClaim::Won => {}
        }
        Ok(project_summary_to_session(
            &summary,
            params.provider_binding.as_ref(),
            false,
            &epoch,
        ))
    }

    async fn resume_session(&self, params: SessionResumeParams) -> Result<Session, RuntimeError> {
        let info = self.find_info(&params.session_id).await?;
        let summary = self
            .storage
            .load_summary(&info)
            .await
            .map_err(io_err_to_runtime)?;
        let binding = self.load_binding(&info).await;
        // R5-02: same residency contract as start — hard errors propagate;
        // `unsupported` leaves storage-only session.
        Self::residency_result(self.ensure_resident(&info, &summary.current_model_id).await)?;
        let archived = self.storage.is_archived(&info);
        let epoch = self.history_epoch_for(&info)?;
        Ok(project_summary_to_session(
            &summary,
            binding.as_ref(),
            archived,
            &epoch,
        ))
    }

    async fn fork_session(&self, params: SessionForkParams) -> Result<Session, RuntimeError> {
        let source_info = self.find_info(&params.session_id).await?;
        let source_summary = self
            .storage
            .load_summary(&source_info)
            .await
            .map_err(io_err_to_runtime)?;
        let new_cwd = params
            .workspace_root
            .clone()
            .unwrap_or_else(|| source_summary.info.cwd.clone());
        let new_session_id = uuid::Uuid::now_v7().to_string();
        let target_info = Info {
            id: agent_client_protocol::SessionId::new(new_session_id.clone()),
            cwd: new_cwd.clone(),
        };
        // REAL symbol: `copy_session_data` (the primitive `fork_session`
        // calls internally). No second authority.
        let options = crate::session::storage::CopySessionOptions {
            parent_session_id: Some(params.session_id.clone()),
            session_kind: Some("fork".to_string()),
            ..Default::default()
        };
        if let Err(e) = self
            .storage
            .copy_session_data(&source_info, &target_info, options)
            .await
        {
            let _ = self.storage.delete_session(&target_info).await;
            return Err(io_err_to_runtime(e));
        }
        let summary = match self.storage.load_summary(&target_info).await {
            Ok(summary) => summary,
            Err(e) => {
                let _ = self.storage.delete_session(&target_info).await;
                return Err(io_err_to_runtime(e));
            }
        };
        // C5-C: the fork copy copies the `provider_binding.json` sidecar
        // (identifier-only, no secrets) from the source session dir, so the
        // forked session inherits the parent's binding. Re-load it from the
        // target's sidecar to project the inherited binding onto the forked
        // Session row (the copy is the authority; this read is the projection).
        let binding = self.load_binding(&target_info).await;
        // R5-04: fork is a new stream identity — rotate the copied epoch
        // atomically instead of removing it and ignoring an I/O failure.
        let epoch = match self.rotate_history_epoch(&target_info) {
            Ok(epoch) => epoch,
            Err(e) => {
                let _ = self.storage.delete_session(&target_info).await;
                return Err(e);
            }
        };
        // Forks start unarchived even if the source was archived (new identity).
        Ok(project_summary_to_session(
            &summary,
            binding.as_ref(),
            false,
            &epoch,
        ))
    }

    async fn archive_session(&self, params: SessionArchiveParams) -> Result<(), RuntimeError> {
        // R6: reversible hide-not-delete. Write `archived.flag` on disk so the
        // session is projected as `SessionStatus::Archived` and can be filtered
        // from default lists, but the session dir is never removed (no data
        // loss). Idempotent: re-archive of an already-archived session is Ok.
        let info = self.find_info(&params.session_id).await?;
        self.storage
            .mark_archived(&info)
            .map_err(io_err_to_runtime)?;
        Ok(())
    }

    async fn start_turn(&self, params: TurnStartParams) -> Result<Turn, RuntimeError> {
        // Route through the live actor's command channel. If no resident
        // handle exists (production PARTIAL — spawner returned `unsupported`,
        // or the session was never started/resumed here), return `unsupported`
        // honestly instead of faking a turn.
        let resident = self
            .resident(&params.session_id)
            .ok_or_else(|| self.no_resident_error(&params.session_id))?;
        let turn_id = uuid::Uuid::now_v7().to_string();
        let blocks = input_blocks_to_content_blocks(&params.input);
        // C5-C: project the session's identifier-only ProviderBinding sidecar
        // (no secrets) onto the new Turn row. Best-effort: `None` when no
        // sidecar exists (e.g. a session started without a binding). The
        // lookup is a scan (`find_info`) because `TurnStartParams` carries no
        // cwd; this matches `read_session`/`resume_session` resolution.
        let turn_binding = match self.find_info(&params.session_id).await {
            Ok(info) => self.load_binding(&info).await,
            Err(_) => None,
        };
        let (tx, rx) = oneshot::channel();
        let prompt = SessionCommand::Prompt {
            prompt_id: turn_id.clone(),
            prompt_blocks: blocks,
            prompt_mode: PromptMode::default(),
            artifact_upload_ctx: None,
            client_identifier: None,
            screen_mode: None,
            verbatim: false,
            traceparent: None,
            json_schema: None,
            send_now: false,
            respond_to: tx,
            persist_ack: None,
            parsed_prompt_tx: None,
        };
        if resident.cmd_tx.send(prompt).is_err() {
            // C1-H F-3 fix: the actor mailbox is gone — clear the stale
            // running-turn slot so the turn-id guard stays honest.
            self.clear_current_turn(&params.session_id);
            return Err(RuntimeError {
                code: "session_closed",
                message: "Session actor command channel is closed.".into(),
            });
        }
        let result = receive_prompt_response(
            rx,
            Duration::from_millis(xai_grok_app_server_protocol::errors::defaults::TOOL_WAIT_MAX_MS),
        )
        .await
        .map_err(|_| {
            self.clear_current_turn(&params.session_id);
            let _ = resident.cmd_tx.send(SessionCommand::Cancel {
                cancel_subagents: true,
                kill_background_tasks: false,
                rewind_if_pristine: false,
                trigger: Some("prompt_deadline".into()),
            });
            RuntimeError {
                code: "runtime_unavailable",
                message: "Session actor did not complete the turn before the configured deadline."
                    .into(),
            }
        })?
        .map_err(|_| {
            // C1-H F-3 fix: the actor dropped the oneshot (likely
            // panicked) — clear the stale running-turn slot.
            self.clear_current_turn(&params.session_id);
            RuntimeError {
                code: "session_closed",
                message: "Session actor dropped the prompt response.".into(),
            }
        })?;
        let ordinal = self.next_ordinal(&params.session_id);
        let now = now_ms();
        let status = match &result {
            Ok(ok) => match ok.completion_kind {
                PromptCompletionKind::Completed => TurnStatus::Completed,
                PromptCompletionKind::Cancelled { .. } => TurnStatus::Interrupted,
                PromptCompletionKind::Rewound => TurnStatus::Interrupted,
                PromptCompletionKind::MaxTurnsReached { .. } => TurnStatus::Failed,
                PromptCompletionKind::RemovedFromQueue => TurnStatus::Declined,
            },
            Err(_) => TurnStatus::Failed,
        };
        let completed_at_ms = if matches!(
            status,
            TurnStatus::Completed
                | TurnStatus::Interrupted
                | TurnStatus::Failed
                | TurnStatus::Declined
        ) {
            Some(now)
        } else {
            None
        };
        Ok(Turn {
            turn_id,
            session_id: params.session_id,
            // C5-C: project the session's identifier-only ProviderBinding
            // (no secrets) onto the Turn row.
            provider_binding: turn_binding,
            revision: WireCounter::new(1),
            status,
            kind: TurnKind::User,
            ordinal,
            created_at_ms: now,
            completed_at_ms,
        })
    }

    async fn steer_turn(&self, params: TurnSteerParams) -> Result<Item, RuntimeError> {
        let resident = self
            .resident(&params.session_id)
            .ok_or_else(|| self.no_resident_error(&params.session_id))?;
        // Verify the target turn is the running turn (R8). Shell `Interject`
        // targets the running turn implicitly via `current_prompt_id`; the
        // adapter enforces the explicit `turn_id` match here.
        let current = resident.current_turn();
        if current.as_deref() != Some(params.turn_id.as_str()) {
            return Err(RuntimeError {
                code: "turn_not_found",
                message: "No running turn matches the given turn_id.".into(),
            });
        }
        let steer_text = input_blocks_to_text(&params.input);
        let interject = SessionCommand::Interject {
            text: steer_text.clone(),
            id: Some(params.idempotency_key.clone()),
            images: Vec::new(),
        };
        if resident.cmd_tx.send(interject).is_err() {
            // C1-H F-3 fix: actor mailbox gone — clear the stale running-turn
            // slot so the turn-id guard stays honest.
            self.clear_current_turn(&params.session_id);
            return Err(RuntimeError {
                code: "session_closed",
                message: "Session actor command channel is closed.".into(),
            });
        }
        // Shell `Interject` is fire-and-forget (no response channel); the
        // adapter synthesizes a protocol `Item` envelope for the steer so the
        // facade contract (`steer_turn -> Item`) is satisfied without a
        // second actor or a parallel turn state machine.
        let now = now_ms();
        // C1-H F-1 fix: `event_seq` is a per-session monotonic sequence, not a
        // wall-clock timestamp (which was semantically wrong and inconsistent
        // with the replay projector's sequential `event_seq`).
        let event_seq = WireCounter::new(self.next_event_seq(&params.session_id));
        Ok(Item {
            item_id: uuid::Uuid::now_v7().to_string(),
            session_id: params.session_id,
            turn_id: params.turn_id,
            event_seq,
            revision: WireCounter::new(1),
            status: ItemStatus::Completed,
            created_at_ms: now,
            // C7-B F-3 / R8: steer synthesizes a UserMessage item (user input
            // injected mid-turn), matching FakeRuntime — not an AgentMessage
            // envelope. Shell Interject is fire-and-forget; the adapter only
            // provides the protocol Item envelope.
            body: ItemBody::UserMessage {
                content: params.input.clone(),
            },
        })
    }

    async fn interrupt_turn(&self, params: TurnInterruptParams) -> Result<(), RuntimeError> {
        let resident = self
            .resident(&params.session_id)
            .ok_or_else(|| self.no_resident_error(&params.session_id))?;
        // Verify the target turn is the running turn (R9).
        let current = resident.current_turn();
        if current.as_deref() != Some(params.turn_id.as_str()) {
            return Err(RuntimeError {
                code: "turn_not_found",
                message: "No running turn matches the given turn_id.".into(),
            });
        }
        let cancel = SessionCommand::Cancel {
            cancel_subagents: true,
            kill_background_tasks: false,
            rewind_if_pristine: false,
            trigger: Some("interrupt_turn".to_string()),
        };
        if resident.cmd_tx.send(cancel).is_err() {
            // C1-H F-3 fix: actor mailbox gone — clear the stale running-turn
            // slot so the turn-id guard stays honest.
            self.clear_current_turn(&params.session_id);
            return Err(RuntimeError {
                code: "session_closed",
                message: "Session actor command channel is closed.".into(),
            });
        }
        // Shell `Cancel` is fire-and-forget; the adapter returns Ok once the
        // command is accepted by the mailbox. The actor clears
        // `current_prompt_id` when the turn actually tears down.
        Ok(())
    }

    async fn respond_interaction(
        &self,
        params: InteractionResponseParams,
    ) -> Result<(), RuntimeError> {
        // R10 delivery channel: deliver the caller's decision string into the
        // existing pending-interaction surface. This is NOT a second permission
        // engine — we do not re-evaluate allow/deny policy. The caller already
        // decided; we only route the decision to the parked future.
        //
        // 1. Session must exist on disk (storage authority).
        let info = self.find_info(&params.session_id).await?;
        let _ = info; // existence check only; no further use of the summary

        // 2. Require a resident actor. If no resident is loaded, return
        //    `unsupported` honestly — the decision cannot be delivered to a
        //    parked future that lives in the actor's memory.
        let resident = self
            .resident(&params.session_id)
            .ok_or_else(|| RuntimeError {
                code: "unsupported",
                message: "respond_interaction requires a resident session actor; \
                no resident loaded for this session (C1-J PARTIAL: spawn not \
                assembled)."
                    .into(),
            })?;

        // Production ACP residents park the reverse request inside the ACP
        // host rather than in Shell's legacy pending-interaction table. Route
        // the same interaction identity directly to that waiter when present.
        if let Some(permission_responder) = resident.permission_responder {
            let decision = match params.decision.as_str() {
                "cancel" | "cancelled" | "deny" | "denied" => {
                    crate::app_server_runtime::acp_host::AcpPermissionDecision::Cancelled
                }
                option_id => crate::app_server_runtime::acp_host::AcpPermissionDecision::Selected(
                    option_id.to_owned(),
                ),
            };
            return permission_responder
                .respond_permission(params.interaction_id, decision)
                .await
                .map_err(|error| RuntimeError {
                    code: if error.0.contains("not found") {
                        "interaction_not_found"
                    } else {
                        "interaction_not_deliverable"
                    },
                    message: error.0,
                });
        }

        // 3. Require the pending-interactions surface. A resident without
        //    one (e.g. a minimal test spawner) cannot accept interaction
        //    responses — return `unsupported` honestly.
        let pending = resident
            .pending_interactions
            .as_ref()
            .ok_or_else(|| RuntimeError {
                code: "unsupported",
                message: "resident has no pending_interactions surface \
                (production auto-register PARTIAL)."
                    .into(),
            })?;

        // 4. First-answer-wins: remove the entry keyed by `interaction_id`
        //    (= `tool_call_id`). A second call for the same interaction finds
        //    the entry gone → `interaction_not_found`. This makes the delivery
        //    idempotent: only the first response is delivered; duplicates are
        //    rejected without side effects.
        let removed = {
            let mut map = pending.lock().unwrap_or_else(|e| e.into_inner());
            map.remove(&params.interaction_id)
        };
        if removed.is_none() {
            return Err(RuntimeError {
                code: "interaction_not_found",
                message: format!(
                    "No pending interaction with id '{}' for this session.",
                    params.interaction_id
                ),
            });
        }

        // 5. Deliver the decision string via the delivery hub. The hub maps
        //    `interaction_id` → `oneshot::Sender` and is shared with the live
        //    actor via `SessionHandle::interaction_delivery_hub` (R5-09).
        //    If no oneshot is registered, restore the pending entry so a later
        //    retry can deliver it once the actor has parked its future.
        let Some(hub) = resident.delivery_hub.as_ref() else {
            // The resident advertised a pending interaction surface but no
            // delivery hub. Preserve the pending entry so a later retry (or
            // a correctly initialized actor) can still deliver it.
            if let Some(kind) = removed {
                pending
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(params.interaction_id.clone(), kind);
            }
            return Err(RuntimeError {
                code: "unsupported",
                message: "resident has no interaction delivery hub".into(),
            });
        };
        let sender = {
            let mut map = hub.lock().unwrap_or_else(|e| e.into_inner());
            map.remove(&params.interaction_id)
        };
        match sender {
            Some(tx) => {
                if tx.send(params.decision.clone()).is_ok() {
                    Ok(())
                } else {
                    // A sender can remain in the hub after its parked future
                    // has been cancelled. Treat that as a failed delivery,
                    // not success, and restore the pending entry so a fresh
                    // actor park can retry safely.
                    if let Some(kind) = removed {
                        pending
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .insert(params.interaction_id.clone(), kind);
                    }
                    Err(RuntimeError {
                        code: "interaction_not_deliverable",
                        message: format!(
                            "Pending interaction '{}' receiver was closed before delivery.",
                            params.interaction_id
                        ),
                    })
                }
            }
            None => {
                // Do not consume a pending interaction when the actor has not
                // parked its oneshot yet. The caller can retry after the
                // reverse-request park completes; first-answer-wins applies
                // only once a decision was actually deliverable.
                if let Some(kind) = removed {
                    pending
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .insert(params.interaction_id.clone(), kind);
                }
                Err(RuntimeError {
                    code: "interaction_not_deliverable",
                    message: format!(
                        "Pending interaction '{}' had no parked oneshot registered \
                         on the delivery hub (actor must park before respond).",
                        params.interaction_id
                    ),
                })
            }
        }
    }

    async fn replay(&self, cursor: SubscribeParams) -> Result<ReplayPage, RuntimeError> {
        let info = self.find_info(&cursor.session_id).await?;
        let summary = self
            .storage
            .load_summary(&info)
            .await
            .map_err(io_err_to_runtime)?;
        let epoch = self.history_epoch_for(&info)?;
        if let Some(expected) = &cursor.history_epoch {
            if expected.as_str() != epoch {
                return Err(RuntimeError {
                    code: "epoch_mismatch",
                    message: "History epoch does not match.".into(),
                });
            }
        }
        // Build the event stream: event 0 = SessionChanged snapshot; events
        // 1..N = projected updates.jsonl lines via the shared projector (R11).
        let binding = self.load_binding(&info).await;
        let archived = self.storage.is_archived(&info);
        let mut numbered: Vec<(u64, RuntimeEvent)> = vec![(
            0,
            RuntimeEvent::SessionChanged(project_summary_to_session(
                &summary,
                binding.as_ref(),
                archived,
                &epoch,
            )),
        )];
        let history = self.project_history(&info, &cursor.session_id, binding.clone());
        // R4-06: use canonical event_seq on each projected event (not the
        // compacted vector index). Gaps from omitted/corrupt lines are preserved
        // so filter is `seq > after_event_seq`.
        for e in history.events {
            let seq =
                runtime_event_seq(&e).unwrap_or(numbered.last().map(|(s, _)| s + 1).unwrap_or(1));
            numbered.push((seq, e));
        }
        let after = cursor.after_event_seq.as_u64();
        // R4-06: exclusive filter on canonical seq. Special case: after=0 is
        // start-of-stream and includes the SessionChanged snapshot at seq 0
        // (client has not received event 0 yet). For after>0, filter is
        // strictly `seq > after`.
        let filtered: Vec<(u64, RuntimeEvent)> = numbered
            .into_iter()
            .filter(|(seq, _)| after == 0 || *seq > after)
            .collect();
        // R5-05: `more` requires remaining events beyond this page — a full
        // page that is also the final page must not advertise next_cursor.
        let more = filtered.len() > REPLAY_PAGE_SIZE;
        let page: Vec<_> = filtered.into_iter().take(REPLAY_PAGE_SIZE).collect();
        let replayed_through = page.last().map(|(s, _)| *s).unwrap_or(after);
        let events: Vec<_> = page.into_iter().map(|(_, e)| e).collect();
        let next_cursor = if more {
            Some(WireCounter::new(replayed_through))
        } else {
            None
        };
        Ok(ReplayPage {
            events,
            replayed_through: WireCounter::new(replayed_through),
            next_cursor,
        })
    }
}

/// Extract the canonical event sequence from a projected RuntimeEvent (R4-06).
fn runtime_event_seq(event: &RuntimeEvent) -> Option<u64> {
    match event {
        RuntimeEvent::SessionChanged(_) => Some(0),
        RuntimeEvent::TurnChanged(t) => Some(t.revision.as_u64()),
        RuntimeEvent::ItemStarted(i) | RuntimeEvent::ItemCompleted(i) => Some(i.event_seq.as_u64()),
        RuntimeEvent::ItemDelta { item_id, .. } => {
            item_id.strip_prefix("item_").and_then(|s| s.parse().ok())
        }
        RuntimeEvent::InteractionRequested(_) => None,
    }
}

#[cfg(test)]
mod port_invariant_tests {
    use super::*;

    /// The real port must not define a SessionActor type (Tower guard mirrors).
    #[test]
    fn shell_session_actor_runtime_defines_no_session_actor() {
        let src = include_str!("shell_session_actor_runtime.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("struct SessionActor"));
        assert!(!production.contains("enum SessionActor"));
    }

    /// The real port must not construct or import FakeRuntime (no hybrid authority).
    /// Doc comments may mention FakeRuntime by name; this checks for actual
    /// code usage (`FakeRuntime::new` / `use ...FakeRuntime`), not prose.
    #[test]
    fn shell_session_actor_runtime_does_not_use_fake_runtime() {
        let src = include_str!("shell_session_actor_runtime.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        assert!(
            !production.contains("FakeRuntime::new")
                && !production.contains("use xai_grok_tower::FakeRuntime")
                && !production.contains(": FakeRuntime"),
            "real port must not mix FakeRuntime authority"
        );
    }

    #[test]
    fn shell_runtime_capabilities_do_not_advertise_unwired_actor_methods() {
        let temp = tempfile::TempDir::new().unwrap();
        let runtime = ShellSessionActorRuntime::new(temp.path().to_path_buf());
        let capabilities = runtime.capabilities();
        assert!(capabilities.session_start);
        assert!(capabilities.session_subscribe);
        assert!(!capabilities.turn_start);
        assert!(!capabilities.turn_steer);
        assert!(!capabilities.turn_interrupt);
        assert!(!capabilities.interaction_respond);
    }

    #[test]
    fn injected_resident_factory_promotes_only_turn_capabilities() {
        let temp = tempfile::TempDir::new().unwrap();
        let runtime = ShellSessionActorRuntime::with_production_spawn(
            temp.path().to_path_buf(),
            experimental_acp_resident_spawn(temp.path().to_path_buf()),
        );
        let capabilities = runtime.capabilities();
        assert!(capabilities.turn_start);
        assert!(capabilities.turn_steer);
        assert!(capabilities.turn_interrupt);
        assert!(!capabilities.interaction_respond);
        assert!(!capabilities.item_lifecycle);
        assert!(!capabilities.item_deltas);
    }

    #[tokio::test]
    async fn prompt_response_deadline_returns_timeout_without_fake_success() {
        let (_tx, rx) = oneshot::channel::<PromptTurnResult>();
        let result = receive_prompt_response(rx, Duration::from_millis(1)).await;
        assert!(result.is_err(), "a stalled actor must hit the deadline");
    }
}
