//! Pure wire types for the experimental grok-oss App Server protocol.
//!
//! This crate intentionally owns no transport, runtime actor, persistence, or
//! authorization behavior. Those boundaries consume these serializable types.

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, str::FromStr};

pub mod envelope;
pub mod errors;
pub mod events;
pub mod methods;
pub mod transitions;
pub use envelope::*;
pub use errors::{
    classify_pre_init, gate_error, lookup as lookup_error, lookup_numeric as lookup_error_numeric,
    DomainErrorData, ErrorSpec, InitializeGateClass, ALL as ERROR_CATALOG,
};
pub use errors::defaults as protocol_defaults;
pub use events::*;
pub use methods::*;
pub use transitions::*;

pub const PROTOCOL_VERSION: &str = "2026-07-18.experimental-v2";

/// Lossless JSON representation for monotonic counters shared with JavaScript.
/// The wire form is a canonical unsigned decimal string, never a JSON number.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct WireCounter(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCounterError;

impl fmt::Display for WireCounterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("wire counter must be a canonical unsigned decimal string")
    }
}

impl WireCounter {
    pub fn new(value: u64) -> Self {
        Self(value.to_string())
    }
    pub fn as_u64(&self) -> u64 {
        self.0.parse().expect("validated wire counter")
    }
}

impl From<u64> for WireCounter {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
impl Default for WireCounter {
    fn default() -> Self {
        Self::new(0)
    }
}
impl From<WireCounter> for String {
    fn from(value: WireCounter) -> Self {
        value.0
    }
}
impl FromStr for WireCounter {
    type Err = WireCounterError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parsed = value.parse::<u64>().map_err(|_| WireCounterError)?;
        if parsed.to_string() != value {
            return Err(WireCounterError);
        }
        Ok(Self(value.to_owned()))
    }
}
impl TryFrom<String> for WireCounter {
    type Error = WireCounterError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl fmt::Display for WireCounter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Public, immutable inference selection. It contains identifiers only—never
/// tokens, cookies, authorization headers, or other credential material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderBinding {
    pub provider_id: String,
    pub credential_id: String,
    pub model_id: String,
    pub backend: String,
    pub binding_revision: WireCounter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    pub client_info: ClientInfo,
    #[serde(default)]
    pub capabilities: ClientCapabilities,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(default)]
    pub experimental: Vec<String>,
    #[serde(default)]
    pub interactions: bool,
    #[serde(default)]
    pub reconnect: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub sessions: SessionCapabilities,
    pub turns: TurnCapabilities,
    pub items: ItemCapabilities,
    pub interactions: InteractionCapabilities,
    #[serde(default)]
    pub experimental: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionCapabilities {
    pub list: bool,
    pub read: bool,
    pub start: bool,
    pub resume: bool,
    pub fork: bool,
    pub archive: bool,
    pub subscribe: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnCapabilities {
    pub start: bool,
    pub steer: bool,
    pub interrupt: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemCapabilities {
    pub lifecycle: bool,
    pub deltas: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InteractionCapabilities {
    pub approvals: bool,
    pub questions: bool,
    pub mcp_elicitation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub server_info: ClientInfo,
    pub server_instance_id: String,
    pub capabilities: ServerCapabilities,
    pub limits: ProtocolLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolLimits {
    pub max_message_bytes: u64,
    pub max_page_size: u32,
    pub replay_window_events: u64,
    pub outbound_queue_events: u64,
    pub initialize_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Starting,
    Ready,
    Running,
    WaitingForInput,
    Dormant,
    Completed,
    Archived,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub session_id: String,
    pub history_epoch: String,
    pub revision: WireCounter,
    pub status: SessionStatus,
    pub workspace_root: String,
    pub title: Option<String>,
    pub active_turn_id: Option<String>,
    pub latest_turn_id: Option<String>,
    #[serde(default)]
    pub provider_binding: Option<ProviderBinding>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TurnStatus {
    Queued,
    InProgress,
    WaitingForApproval,
    WaitingForInput,
    Completed,
    Interrupted,
    Declined,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TurnKind {
    User,
    Steer,
    Resume,
    Synthetic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Turn {
    pub turn_id: String,
    pub session_id: String,
    pub provider_binding: Option<ProviderBinding>,
    pub revision: WireCounter,
    pub status: TurnStatus,
    pub kind: TurnKind,
    pub ordinal: u64,
    pub created_at_ms: u64,
    #[serde(default)]
    pub completed_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    Pending,
    InProgress,
    WaitingForApproval,
    WaitingForInput,
    Completed,
    Failed,
    Declined,
    Cancelled,
    Backgrounded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputBlock {
    Text { text: String },
    Mention { name: String, path: Option<String> },
    Skill { name: String, path: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemBody {
    UserMessage {
        content: Vec<InputBlock>,
    },
    AgentMessage {
        text: String,
    },
    ToolCall {
        tool_name: String,
        arguments: Value,
    },
    ToolResult {
        tool_name: String,
        output: Value,
        is_error: bool,
    },
    CommandExecution {
        command: String,
        argv: Vec<String>,
        cwd: String,
        output: String,
        exit_code: Option<i32>,
    },
    FileChange {
        changes: Vec<Value>,
        summary: Option<String>,
    },
    Plan {
        content: String,
        steps: Vec<Value>,
    },
    Subagent {
        subagent_id: String,
        agent_type: String,
        description: String,
        result: Option<String>,
    },
    McpToolCall {
        server: String,
        tool_name: String,
        arguments: Value,
    },
    ReasoningSummary {
        summary: String,
    },
    Hook {
        hook_name: String,
        phase: String,
        safe_summary: String,
    },
    BackgroundTask {
        task_id: String,
        safe_summary: String,
    },
    Compaction {
        safe_summary: String,
    },
    ProviderError {
        provider_id: String,
        code: String,
        safe_message: String,
    },
    InteractionRequest {
        interaction_id: String,
        prompt: String,
        choices: Vec<String>,
    },
    Error {
        code: String,
        message: String,
    },
    Extension {
        extension_type: String,
        payload: Value,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub item_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub event_seq: WireCounter,
    pub revision: WireCounter,
    pub status: ItemStatus,
    pub created_at_ms: u64,
    #[serde(flatten)]
    pub body: ItemBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartParams {
    pub workspace_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_binding: Option<ProviderBinding>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnStartParams {
    pub session_id: String,
    pub input: Vec<InputBlock>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionRef {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnRef {
    pub session_id: String,
    pub turn_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeParams {
    pub session_id: String,
    #[serde(default)]
    pub after_event_seq: WireCounter,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_epoch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

pub fn protocol_schema() -> Value {
    serde_json::to_value(schema_for!(ProtocolDocument)).expect("schema serializes")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum ProtocolDocument {
    InitializeParams(InitializeParams),
    InitializeResult(InitializeResult),
    Session(Session),
    Turn(Turn),
    Item(Item),
    SessionStartParams(SessionStartParams),
    TurnStartParams(TurnStartParams),
    SubscribeParams(SubscribeParams),
    RpcError(RpcError),
    Method(methods::MethodDocument),
    Event(events::EventDocument),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_roundtrip_preserves_wire_shape() {
        let value = InitializeParams {
            protocol_version: PROTOCOL_VERSION.into(),
            client_info: ClientInfo {
                name: "fixture".into(),
                version: "1".into(),
            },
            capabilities: ClientCapabilities {
                reconnect: true,
                ..Default::default()
            },
        };
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(
            serde_json::from_str::<InitializeParams>(&json).unwrap(),
            value
        );
    }

    #[test]
    fn item_roundtrip_preserves_tagged_body() {
        let value = Item {
            item_id: "item_1".into(),
            session_id: "session_1".into(),
            turn_id: "turn_1".into(),
            event_seq: 3.into(),
            revision: 1.into(),
            status: ItemStatus::Completed,
            created_at_ms: 1_784_376_000_000,
            body: ItemBody::AgentMessage {
                text: "done".into(),
            },
        };
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json["type"], "agent_message");
        assert_eq!(serde_json::from_value::<Item>(json).unwrap(), value);
    }

    #[test]
    fn provider_binding_is_structured_and_contains_no_secret_material() {
        let value = serde_json::json!({
            "providerId": "codex",
            "credentialId": "work",
            "modelId": "gpt-5.6",
            "backend": "responses",
            "bindingRevision": "7"
        });
        let binding: ProviderBinding = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(binding).unwrap(), value);
        assert!(
            serde_json::from_value::<ProviderBinding>(serde_json::json!({
                "providerId": "codex",
                "credentialId": "work",
                "modelId": "gpt-5.6",
                "backend": "responses",
                "bindingRevision": "7",
                "accessToken": "secret"
            }))
            .is_err()
        );
    }

    #[test]
    fn wire_counters_serialize_as_lossless_decimal_strings() {
        let counter = WireCounter::new(u64::MAX);
        assert_eq!(
            serde_json::to_value(&counter).unwrap(),
            serde_json::json!("18446744073709551615")
        );
        assert_eq!(
            serde_json::from_value::<WireCounter>(serde_json::json!("18446744073709551615"))
                .unwrap(),
            counter
        );
        assert!(serde_json::from_value::<WireCounter>(serde_json::json!(1)).is_err());
        assert!(serde_json::from_value::<WireCounter>(serde_json::json!("01")).is_err());
    }

    #[test]
    fn generated_protocol_schema_compiles() {
        jsonschema::validator_for(&protocol_schema()).expect("valid JSON Schema");
    }

    #[test]
    fn checked_in_generated_schema_matches_rust_types_byte_for_byte() {
        let mut generated = serde_json::to_string_pretty(&protocol_schema()).unwrap();
        generated.push('\n');
        assert_eq!(
            include_str!("../schemas/generated-protocol.schema.json"),
            generated,
            "run: cargo run -p xai-grok-app-server-protocol --example generate-schema"
        );
    }

    #[test]
    fn checked_in_schemas_compile_and_cover_nine_tool_pairs() {
        let app: Value = serde_json::from_str(include_str!("../schemas/app-server.schema.json"))
            .expect("app-server schema is JSON");
        let tools: Value = serde_json::from_str(include_str!("../schemas/tower-tools.schema.json"))
            .expect("tower tools schema is JSON");
        jsonschema::validator_for(&app).expect("app-server schema compiles");
        jsonschema::validator_for(&tools).expect("tower tools schema compiles");
        let definitions = tools["$defs"].as_object().expect("tool definitions");
        let input_count = definitions
            .keys()
            .filter(|key| key.ends_with("_input"))
            .count();
        let output_count = definitions
            .keys()
            .filter(|key| key.ends_with("_output"))
            .count();
        assert_eq!((input_count, output_count), (9, 9));
    }

    #[test]
    fn all_nine_tower_tool_input_and_output_examples_validate() {
        let schema: Value =
            serde_json::from_str(include_str!("../schemas/tower-tools.schema.json")).unwrap();
        let row = serde_json::json!({"sessionId":"session_1","agentType":"build","workspaceRoot":"/work","status":"ready","residency":"resident","activeTurnId":null,"updatedAtMs":1,"safeSummary":null});
        let operation = serde_json::json!({"operationId":"op_1","state":"completed","sessionId":"session_1","turnId":null});
        let cases = [
            (
                "tower_agent_list",
                serde_json::json!({}),
                serde_json::json!({"sessions":[row.clone()],"nextCursor":null}),
            ),
            (
                "tower_agent_start",
                serde_json::json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"start-0001"}),
                operation.clone(),
            ),
            (
                "tower_agent_send",
                serde_json::json!({"sessionId":"session_1","input":[{"type":"text","text":"work"}],"mode":"new_turn","turnId":null,"idempotencyKey":"send-0001"}),
                operation.clone(),
            ),
            (
                "tower_agent_history",
                serde_json::json!({"sessionId":"session_1","mode":"last","maxBytes":4096}),
                serde_json::json!({"sessionId":"session_1","historyEpoch":"epoch_1","items":[],"nextEventSeq":"0","truncated":false,"redacted":true}),
            ),
            (
                "tower_agent_resume",
                serde_json::json!({"sessionId":"session_1","idempotencyKey":"resume-01"}),
                operation.clone(),
            ),
            (
                "tower_agent_wait",
                serde_json::json!({"sessionId":"session_1","afterEventSeq":"0","timeoutMs":1000}),
                serde_json::json!({"sessionId":"session_1","historyEpoch":"epoch_1","events":[],"nextEventSeq":"0","wakeReason":"timeout"}),
            ),
            (
                "tower_agent_interrupt",
                serde_json::json!({"sessionId":"session_1","turnId":"turn_1","idempotencyKey":"interrupt-1"}),
                operation.clone(),
            ),
            (
                "tower_agent_archive",
                serde_json::json!({"sessionId":"session_1","idempotencyKey":"archive-01"}),
                operation,
            ),
            (
                "tower_agent_status",
                serde_json::json!({"sessionId":"session_1"}),
                row,
            ),
        ];
        for (tool, input, output) in cases {
            validate_definition(&schema, &format!("{tool}_input"), &input);
            validate_definition(&schema, &format!("{tool}_output"), &output);
        }
    }

    #[test]
    fn all_four_jsonl_goldens_are_valid_json_objects() {
        let goldens = [
            include_str!("../schemas/goldens/happy-coding.jsonl"),
            include_str!("../schemas/goldens/interrupt.jsonl"),
            include_str!("../schemas/goldens/multi-session.jsonl"),
            include_str!("../schemas/goldens/reconnect.jsonl"),
        ];
        for golden in goldens {
            assert!(golden.lines().count() >= 3);
            for line in golden.lines() {
                assert!(serde_json::from_str::<Value>(line).unwrap().is_object());
            }
        }
    }

    #[test]
    fn golden_params_and_events_validate_against_named_checked_in_definitions() {
        use std::collections::HashMap;

        let schema: Value =
            serde_json::from_str(include_str!("../schemas/app-server.schema.json")).unwrap();
        let definition_for = |method: &str| match method {
            "initialize" => Some("initializeParams"),
            "session/start" => Some("sessionStartParams"),
            "session/list" => Some("sessionListParams"),
            "session/subscribe" => Some("subscribeParams"),
            "turn/start" => Some("turnStartParams"),
            "turn/interrupt" => Some("turnInterruptParams"),
            "item/started" | "item/completed" => Some("itemEvent"),
            "item/delta" => Some("itemDeltaParams"),
            "turn/updated" => Some("turnEvent"),
            "initialized" => None,
            other => panic!("golden method has no schema mapping: {other}"),
        };
        for golden in [
            include_str!("../schemas/goldens/happy-coding.jsonl"),
            include_str!("../schemas/goldens/interrupt.jsonl"),
            include_str!("../schemas/goldens/multi-session.jsonl"),
            include_str!("../schemas/goldens/reconnect.jsonl"),
        ] {
            let envelope_validator = jsonschema::validator_for(&schema).unwrap();
            let mut pending_methods = HashMap::<String, String>::new();
            for line in golden.lines() {
                let message: Value = serde_json::from_str(line).unwrap();
                if let Err(error) = envelope_validator.validate(&message) {
                    panic!("golden envelope invalid: {error}; message={message}");
                }
                let Some(method) = message.get("method").and_then(Value::as_str) else {
                    if let (Some(id), Some(result)) = (message.get("id"), message.get("result")) {
                        let method = pending_methods
                            .remove(&id.to_string())
                            .unwrap_or_else(|| panic!("response has no prior request: {message}"));
                        let result_definition = match method.as_str() {
                            "initialize" => Some("initializeResult"),
                            "session/start" => Some("sessionResult"),
                            "session/list" => Some("sessionListResult"),
                            "session/subscribe" => Some("subscribeResult"),
                            "turn/start" => Some("turnResult"),
                            "turn/interrupt" => Some("operationResult"),
                            _ => None,
                        };
                        if let Some(definition) = result_definition {
                            validate_definition(&schema, definition, result);
                        }
                    }
                    continue;
                };
                if let Some(id) = message.get("id") {
                    pending_methods.insert(id.to_string(), method.to_owned());
                }
                let Some(definition) = definition_for(method) else {
                    continue;
                };
                let params = &message["params"];
                validate_definition(&schema, definition, params);
            }
        }
    }

    fn validate_definition(schema: &Value, definition: &str, instance: &Value) {
        let wrapper = serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$ref": format!("#/$defs/{definition}"),
            "$defs": schema["$defs"].clone(),
        });
        let validator = jsonschema::validator_for(&wrapper).unwrap();
        if let Err(error) = validator.validate(instance) {
            panic!("instance violates {definition}: {error}; instance={instance}");
        }
    }
}
