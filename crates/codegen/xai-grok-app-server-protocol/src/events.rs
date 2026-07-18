//! Notifications and server-initiated interaction requests.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Item, Session, Turn};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventMeta {
    pub session_id: String,
    pub history_epoch: String,
    pub event_seq: u64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
    #[serde(flatten)]
    pub meta: EventMeta,
    pub session: Session,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnEvent {
    #[serde(flatten)]
    pub meta: EventMeta,
    pub turn: Turn,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemEvent {
    #[serde(flatten)]
    pub meta: EventMeta,
    pub item: Item,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemDeltaEvent {
    #[serde(flatten)]
    pub meta: EventMeta,
    pub turn_id: String,
    pub item_id: String,
    pub revision: u64,
    pub delta: String,
    pub stream: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InteractionKind {
    Approval,
    Question,
    McpElicitation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InteractionRequest {
    pub interaction_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub item_id: String,
    pub kind: InteractionKind,
    pub prompt: String,
    pub choices: Vec<String>,
    pub expires_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", content = "value")]
pub enum EventDocument {
    #[serde(rename = "session/updated")]
    SessionUpdated(SessionEvent),
    #[serde(rename = "turn/updated")]
    TurnUpdated(TurnEvent),
    #[serde(rename = "item/started")]
    ItemStarted(ItemEvent),
    #[serde(rename = "item/delta")]
    ItemDelta(ItemDeltaEvent),
    #[serde(rename = "item/completed")]
    ItemCompleted(ItemEvent),
    #[serde(rename = "interaction/request")]
    InteractionRequest(InteractionRequest),
}
