//! JSON-RPC processor: initialize gate + method dispatch over GrokRuntimeFacade.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use xai_grok_app_server_protocol::{
    classify_pre_init, gate_error, protocol_defaults, EnvelopeKind, InitializeParams,
    InitializeResult, InteractionResponseParams, ProtocolLimits, ServerCapabilities,
    SessionArchiveParams, SessionCapabilities, SessionForkParams, SessionListParams,
    SessionListResult, SessionReadParams, SessionResumeParams, SessionStartParams,
    SubscribeParams, TurnInterruptParams, TurnStartParams, TurnSteerParams, ClientInfo,
    ItemCapabilities, TurnCapabilities, InteractionCapabilities, PROTOCOL_VERSION,
    parse_envelope, InitializeGateClass,
};
use xai_grok_tower::{GrokRuntimeFacade, RuntimeError, RuntimeEvent};

use crate::{AppServerProcessor, ProcessorError};

pub struct FacadeProcessor {
    runtime: Arc<dyn GrokRuntimeFacade>,
    initialized: AtomicBool,
    server_instance_id: String,
    outbound_queue_cap: u64,
    slow_client_events: AtomicU64,
}

impl FacadeProcessor {
    pub fn new(runtime: Arc<dyn GrokRuntimeFacade>) -> Self {
        Self {
            runtime,
            initialized: AtomicBool::new(false),
            server_instance_id: format!(
                "tower_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            ),
            outbound_queue_cap: protocol_defaults::OUTBOUND_QUEUE_EVENTS,
            slow_client_events: AtomicU64::new(0),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    pub async fn handle_line(&self, line: &str) -> Result<Option<String>, ProcessorError> {
        let value: Value = serde_json::from_str(line).map_err(|e| ProcessorError {
            code: -32700,
            message: format!("Parse error: {e}"),
        })?;
        self.handle_value(value).await
    }

    pub async fn handle_value(&self, value: Value) -> Result<Option<String>, ProcessorError> {
        let envelope = parse_envelope(&value).map_err(|e| ProcessorError {
            code: e.spec.numeric,
            message: e.spec.message.into(),
        })?;
        match envelope {
            EnvelopeKind::Request { id, method, params } => {
                match self.dispatch(&method, params).await {
                    Ok(result) => Ok(Some(
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result,
                        })
                        .to_string(),
                    )),
                    Err(err) => Ok(Some(
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": err.code,
                                "message": err.message,
                                "data": {
                                    "code": domain_code(err.code),
                                    "retryable": retryable(err.code),
                                }
                            }
                        })
                        .to_string(),
                    )),
                }
            }
            EnvelopeKind::Notification { method, params } => {
                if method == "initialized" {
                    // Client ack; already initialized after successful initialize result path.
                    let _ = params;
                    return Ok(None);
                }
                Err(ProcessorError {
                    code: -32601,
                    message: format!("Method not found: {method}"),
                })
            }
            EnvelopeKind::Success { .. } | EnvelopeKind::Failure { .. } => Err(ProcessorError {
                code: -32600,
                message: "Server does not accept client responses as input".into(),
            }),
        }
    }

    async fn dispatch(&self, method: &str, params: Value) -> Result<Value, ProcessorError> {
        let already = self.initialized.load(Ordering::SeqCst);
        match classify_pre_init(method, already) {
            InitializeGateClass::AlreadyInitialized => {
                return Err(spec_error(
                    xai_grok_app_server_protocol::errors::ALREADY_INITIALIZED,
                ));
            }
            InitializeGateClass::NotInitialized => {
                return Err(spec_error(
                    xai_grok_app_server_protocol::errors::NOT_INITIALIZED,
                ));
            }
            InitializeGateClass::HealthAllowed => {
                return Ok(json!({"ok": true}));
            }
            InitializeGateClass::AllowedInitialize => {}
        }

        match method {
            "initialize" => self.initialize(params).await,
            "session/start" => {
                let p: SessionStartParams = deser(params)?;
                let session = self.runtime.start_session(p).await.map_err(map_runtime)?;
                Ok(json!({ "session": session }))
            }
            "session/list" => {
                let p: SessionListParams = deser(params).unwrap_or(SessionListParams {
                    page: Default::default(),
                    include_archived: false,
                    workspace_root: None,
                });
                let mut sessions = self.runtime.list_sessions().await.map_err(map_runtime)?;
                // R6 / C7-B: hide archived sessions from the default list unless
                // the caller explicitly opts in with `include_archived`.
                if !p.include_archived {
                    sessions.retain(|s| {
                        !matches!(
                            s.status,
                            xai_grok_app_server_protocol::SessionStatus::Archived
                        )
                    });
                }
                if let Some(root) = p.workspace_root.as_ref() {
                    sessions.retain(|s| &s.workspace_root == root);
                }
                Ok(serde_json::to_value(SessionListResult {
                    sessions,
                    next_cursor: None,
                })
                .unwrap())
            }
            "session/read" => {
                let p: SessionReadParams = deser(params)?;
                let result = self.runtime.read_session(p).await.map_err(map_runtime)?;
                Ok(serde_json::to_value(result).unwrap())
            }
            "session/resume" => {
                let p: SessionResumeParams = deser(params)?;
                let session = self.runtime.resume_session(p).await.map_err(map_runtime)?;
                Ok(json!({ "session": session }))
            }
            "session/fork" => {
                let p: SessionForkParams = deser(params)?;
                let session = self.runtime.fork_session(p).await.map_err(map_runtime)?;
                Ok(json!({ "session": session }))
            }
            "session/archive" => {
                let p: SessionArchiveParams = deser(params)?;
                self.runtime.archive_session(p).await.map_err(map_runtime)?;
                Ok(json!({"operationId": "archive", "accepted": true}))
            }
            "session/subscribe" => {
                let p: SubscribeParams = deser(params)?;
                let page = self.runtime.replay(p).await.map_err(map_runtime)?;
                // Bounded queue characterization for slow clients.
                if page.events.len() as u64 > self.outbound_queue_cap {
                    self.slow_client_events.fetch_add(1, Ordering::SeqCst);
                }
                let events: Vec<Value> = page.events.iter().map(runtime_event_json).collect();
                Ok(json!({
                    "subscriptionId": format!("sub_{}", self.server_instance_id),
                    "replay": {
                        "events": events,
                        "replayedThrough": page.replayed_through,
                        "nextCursor": page.next_cursor,
                    }
                }))
            }
            "turn/start" => {
                let p: TurnStartParams = deser(params)?;
                let turn = self.runtime.start_turn(p).await.map_err(map_runtime)?;
                Ok(json!({ "turn": turn }))
            }
            "turn/steer" => {
                let p: TurnSteerParams = deser(params)?;
                let item = self.runtime.steer_turn(p).await.map_err(map_runtime)?;
                Ok(json!({ "item": item }))
            }
            "turn/interrupt" => {
                let p: TurnInterruptParams = deser(params)?;
                self.runtime.interrupt_turn(p).await.map_err(map_runtime)?;
                Ok(json!({"operationId": "interrupt", "accepted": true}))
            }
            "interaction/respond" => {
                let p: InteractionResponseParams = deser(params)?;
                self.runtime
                    .respond_interaction(p)
                    .await
                    .map_err(map_runtime)?;
                Ok(json!({"operationId": "interaction", "accepted": true}))
            }
            other => Err(ProcessorError {
                code: -32601,
                message: format!("Method not found: {other}"),
            }),
        }
    }

    async fn initialize(&self, params: Value) -> Result<Value, ProcessorError> {
        let params: InitializeParams = deser(params)?;
        if params.protocol_version != PROTOCOL_VERSION {
            return Err(spec_error(
                xai_grok_app_server_protocol::errors::PROTOCOL_VERSION_UNSUPPORTED,
            ));
        }
        self.initialized.store(true, Ordering::SeqCst);
        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.into(),
            server_info: ClientInfo {
                name: "grok-oss-app-server".into(),
                version: "0.0.0-experimental".into(),
            },
            server_instance_id: self.server_instance_id.clone(),
            capabilities: ServerCapabilities {
                sessions: SessionCapabilities {
                    list: true,
                    read: true,
                    start: true,
                    resume: true,
                    fork: true,
                    archive: true,
                    subscribe: true,
                },
                turns: TurnCapabilities {
                    start: true,
                    steer: true,
                    interrupt: true,
                },
                items: ItemCapabilities {
                    lifecycle: true,
                    deltas: true,
                },
                interactions: InteractionCapabilities {
                    approvals: true,
                    questions: true,
                    mcp_elicitation: false,
                },
                experimental: vec!["experimental-v2".into()],
            },
            limits: ProtocolLimits {
                max_message_bytes: protocol_defaults::MAX_MESSAGE_BYTES,
                max_page_size: protocol_defaults::MAX_PAGE_SIZE,
                replay_window_events: protocol_defaults::REPLAY_WINDOW_EVENTS,
                outbound_queue_events: protocol_defaults::OUTBOUND_QUEUE_EVENTS,
                initialize_timeout_ms: protocol_defaults::INITIALIZE_TIMEOUT_MS,
            },
        };
        Ok(serde_json::to_value(result).unwrap())
    }
}

#[async_trait]
impl AppServerProcessor for FacadeProcessor {
    async fn process(&self, method: &str, params: Value) -> Result<Value, ProcessorError> {
        self.dispatch(method, params).await
    }
}

fn deser<T: serde::de::DeserializeOwned>(params: Value) -> Result<T, ProcessorError> {
    serde_json::from_value(params).map_err(|e| ProcessorError {
        code: -32602,
        message: format!("Invalid params: {e}"),
    })
}

fn map_runtime(err: RuntimeError) -> ProcessorError {
    let numeric = match err.code {
        "session_not_found" => -32010,
        "turn_not_found" => -32011,
        "epoch_mismatch" => -32012,
        "invalid_workspace" => -32020,
        "resource_budget_admission" => -32022,
        "invalid_state" => -32016,
        "unauthorized" => -32001,
        _ => -32603,
    };
    ProcessorError {
        code: numeric,
        message: err.message,
    }
}

fn spec_error(spec: xai_grok_app_server_protocol::ErrorSpec) -> ProcessorError {
    ProcessorError {
        code: spec.numeric,
        message: spec.message.into(),
    }
}

fn domain_code(numeric: i64) -> &'static str {
    xai_grok_app_server_protocol::lookup_error_numeric(numeric)
        .map(|s| s.code)
        .unwrap_or("internal_error")
}

fn retryable(numeric: i64) -> bool {
    xai_grok_app_server_protocol::lookup_error_numeric(numeric)
        .map(|s| s.retryable)
        .unwrap_or(false)
}

fn runtime_event_json(event: &RuntimeEvent) -> Value {
    match event {
        RuntimeEvent::SessionChanged(session) => json!({
            "event": "session/updated",
            "session": session,
        }),
        RuntimeEvent::TurnChanged(turn) => json!({
            "event": "turn/updated",
            "turn": turn,
        }),
        RuntimeEvent::ItemStarted(item) => json!({
            "event": "item/started",
            "item": item,
        }),
        RuntimeEvent::ItemCompleted(item) => json!({
            "event": "item/completed",
            "item": item,
        }),
        RuntimeEvent::ItemDelta {
            session_id,
            turn_id,
            item_id,
            revision,
            delta,
        } => json!({
            "event": "item/delta",
            "sessionId": session_id,
            "turnId": turn_id,
            "itemId": item_id,
            "revision": revision,
            "delta": delta,
        }),
        RuntimeEvent::InteractionRequested(req) => json!({
            "event": "interaction/request",
            "interaction": req,
        }),
    }
}

// Keep gate_error referenced for contract symmetry.
#[allow(dead_code)]
fn _gate(class: InitializeGateClass) -> Option<&'static xai_grok_app_server_protocol::ErrorSpec> {
    gate_error(class)
}

#[cfg(test)]
mod processor_tests {
    use super::*;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn processor_initialize_session_turn_vertical_slice() {
        let rt = Arc::new(FakeRuntime::new());
        let processor = FacadeProcessor::new(rt);
        let init = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0",
                    "id":1,
                    "method":"initialize",
                    "params":{
                        "protocolVersion": PROTOCOL_VERSION,
                        "clientInfo":{"name":"t","version":"1"},
                        "capabilities":{}
                    }
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let init_v: Value = serde_json::from_str(&init).unwrap();
        assert_eq!(init_v["result"]["protocolVersion"], PROTOCOL_VERSION);

        let start = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0",
                    "id":2,
                    "method":"session/start",
                    "params":{
                        "workspaceRoot":"/work",
                        "idempotencyKey":"s1"
                    }
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let start_v: Value = serde_json::from_str(&start).unwrap();
        let session_id = start_v["result"]["session"]["sessionId"].as_str().unwrap();

        let turn = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0",
                    "id":3,
                    "method":"turn/start",
                    "params":{
                        "sessionId": session_id,
                        "input":[{"type":"text","text":"hello"}],
                        "idempotencyKey":"t1"
                    }
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let turn_v: Value = serde_json::from_str(&turn).unwrap();
        assert!(turn_v["result"]["turn"]["turnId"].is_string());

        let sub = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0",
                    "id":4,
                    "method":"session/subscribe",
                    "params":{
                        "sessionId": session_id,
                        "afterEventSeq":"0"
                    }
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let sub_v: Value = serde_json::from_str(&sub).unwrap();
        assert!(
            sub_v["result"]["replay"]["events"]
                .as_array()
                .unwrap()
                .len()
                >= 1
        );
    }

    #[tokio::test]
    async fn processor_rejects_methods_before_initialize() {
        let processor = FacadeProcessor::new(Arc::new(FakeRuntime::new()));
        let line = json!({
            "jsonrpc":"2.0",
            "id":9,
            "method":"session/list",
            "params":{}
        })
        .to_string();
        let resp = processor.handle_line(&line).await.unwrap().unwrap();
        let v: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["error"]["code"], -32002);
        assert_eq!(v["error"]["data"]["code"], "not_initialized");
    }

    #[tokio::test]
    async fn composition_processor_depends_on_facade_trait_not_shell() {
        let cargo = include_str!("../Cargo.toml");
        assert!(!cargo.contains("xai-grok-shell"));
        assert!(cargo.contains("xai-grok-tower"));
        let _p: FacadeProcessor = FacadeProcessor::new(Arc::new(FakeRuntime::new()));
    }
}
