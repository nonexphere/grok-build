//! Params/results for every critical experimental-v2 method.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{InputBlock, Item, Session, Turn};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
