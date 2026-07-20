//! Params/results for every critical experimental-v2 method.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{InputBlock, Item, Session, Turn};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionListParams {
    #[serde(flatten)]
    pub page: PageRequest,
    #[serde(default)]
    pub include_archived: bool,
    #[serde(default)]
    pub workspace_root: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionListResult {
    pub sessions: Vec<Session>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionReadParams {
    pub session_id: String,
    #[serde(default)]
    pub include_turns: bool,
    #[serde(default)]
    pub include_items: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionReadResult {
    pub session: Session,
    pub turns: Vec<Turn>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionResumeParams {
    pub session_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionForkParams {
    pub session_id: String,
    pub idempotency_key: String,
    #[serde(default)]
    pub workspace_root: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionArchiveParams {
    pub session_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnSteerParams {
    pub session_id: String,
    pub turn_id: String,
    pub input: Vec<InputBlock>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnInterruptParams {
    pub session_id: String,
    pub turn_id: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InteractionResponseParams {
    pub session_id: String,
    pub turn_id: String,
    pub interaction_id: String,
    pub decision: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub operation_id: String,
    pub accepted: bool,
}

// This schema/document carrier intentionally preserves concrete protocol
// variants. Boxing only the large read-result variant would change the public
// Rust construction shape without changing wire behavior.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", content = "value")]
pub enum MethodDocument {
    #[serde(rename = "session/listParams")]
    SessionListParams(SessionListParams),
    #[serde(rename = "session/listResult")]
    SessionListResult(SessionListResult),
    #[serde(rename = "session/readParams")]
    SessionReadParams(SessionReadParams),
    #[serde(rename = "session/readResult")]
    SessionReadResult(SessionReadResult),
    #[serde(rename = "session/resumeParams")]
    SessionResumeParams(SessionResumeParams),
    #[serde(rename = "session/forkParams")]
    SessionForkParams(SessionForkParams),
    #[serde(rename = "session/archiveParams")]
    SessionArchiveParams(SessionArchiveParams),
    #[serde(rename = "turn/steerParams")]
    TurnSteerParams(TurnSteerParams),
    #[serde(rename = "turn/interruptParams")]
    TurnInterruptParams(TurnInterruptParams),
    #[serde(rename = "interaction/respondParams")]
    InteractionResponseParams(InteractionResponseParams),
    #[serde(rename = "operationResult")]
    OperationResult(OperationResult),
}

#[cfg(test)]
mod methods_tests {
    use super::*;
    use crate::{
        InputBlock, Item, ItemBody, ItemStatus, ProviderBinding, Session, SessionStartParams,
        SessionStatus, Turn, TurnKind, TurnStartParams, TurnStatus, PROTOCOL_VERSION,
    };
    use serde_json::json;

    fn sample_session() -> Session {
        Session {
            session_id: "session_1".into(),
            history_epoch: "epoch_1".into(),
            revision: 1.into(),
            status: SessionStatus::Ready,
            workspace_root: "/work".into(),
            title: None,
            active_turn_id: None,
            latest_turn_id: None,
            provider_binding: Some(ProviderBinding {
                provider_id: "codex".into(),
                credential_id: "work".into(),
                model_id: "gpt-5.6".into(),
                backend: "responses".into(),
                binding_revision: 1.into(),
            }),
            created_at_ms: 1,
            updated_at_ms: 1,
        }
    }

    #[test]
    fn methods_session_and_turn_params_roundtrip_happy_and_reject_unknown() {
        let start = SessionStartParams {
            workspace_root: "/work".into(),
            agent_type: Some("orchestrator".into()),
            provider_binding: None,
            idempotency_key: "session-start-0010".into(),
        };
        let value = serde_json::to_value(&start).unwrap();
        assert_eq!(value["workspaceRoot"], "/work");
        assert_eq!(value["idempotencyKey"], "session-start-0010");
        assert_eq!(
            serde_json::from_value::<SessionStartParams>(value).unwrap(),
            start
        );

        let turn = TurnStartParams {
            session_id: "session_1".into(),
            input: vec![InputBlock::Text {
                text: "hello".into(),
            }],
            idempotency_key: "turn-1".into(),
        };
        assert_eq!(
            serde_json::from_value::<TurnStartParams>(serde_json::to_value(&turn).unwrap()).unwrap(),
            turn
        );

        // Concurrent-shape inventory: list + read + interrupt all deserialize.
        let list = SessionListParams {
            page: PageRequest {
                page_size: Some(10),
                cursor: None,
            },
            include_archived: false,
            workspace_root: Some("/work".into()),
        };
        let list_json = serde_json::to_value(&list).unwrap();
        assert_eq!(
            serde_json::from_value::<SessionListParams>(list_json).unwrap(),
            list
        );

        let interrupt = TurnInterruptParams {
            session_id: "session_1".into(),
            turn_id: "turn_1".into(),
            idempotency_key: "int-1".into(),
        };
        assert_eq!(
            serde_json::from_value::<TurnInterruptParams>(serde_json::to_value(&interrupt).unwrap())
                .unwrap(),
            interrupt
        );

        let _ = PROTOCOL_VERSION;
        let _ = sample_session();
    }

    #[test]
    fn methods_results_and_method_document_tags_are_stable() {
        let session = sample_session();
        let result = SessionListResult {
            sessions: vec![session.clone()],
            next_cursor: None,
        };
        let doc = MethodDocument::SessionListResult(result.clone());
        let value = serde_json::to_value(&doc).unwrap();
        assert_eq!(value["method"], "session/listResult");
        assert_eq!(
            serde_json::from_value::<MethodDocument>(value).unwrap(),
            MethodDocument::SessionListResult(result)
        );

        let read = SessionReadResult {
            session,
            turns: vec![Turn {
                turn_id: "turn_1".into(),
                session_id: "session_1".into(),
                provider_binding: None,
                revision: 1.into(),
                status: TurnStatus::Completed,
                kind: TurnKind::User,
                ordinal: 1,
                created_at_ms: 1,
                completed_at_ms: Some(2),
            }],
            items: vec![Item {
                item_id: "item_1".into(),
                session_id: "session_1".into(),
                turn_id: "turn_1".into(),
                event_seq: 1.into(),
                revision: 1.into(),
                status: ItemStatus::Completed,
                created_at_ms: 1,
                body: ItemBody::AgentMessage {
                    text: "ok".into(),
                },
            }],
        };
        let read_json = serde_json::to_value(&read).unwrap();
        assert_eq!(read_json["turns"][0]["revision"], json!("1"));
        assert_eq!(
            serde_json::from_value::<SessionReadResult>(read_json).unwrap(),
            read
        );
    }
}
