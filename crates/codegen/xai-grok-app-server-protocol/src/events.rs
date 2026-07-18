//! Notifications and server-initiated interaction requests.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Item, Session, Turn, WireCounter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    pub session_id: String,
    pub history_epoch: String,
    pub event_seq: WireCounter,
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
    pub revision: WireCounter,
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

#[cfg(test)]
mod events_tests {
    use super::*;
    use crate::{
        Item, ItemBody, ItemStatus, Session, SessionStatus, Turn, TurnKind, TurnStatus, WireCounter,
    };
    use serde_json::json;

    fn meta(seq: u64) -> EventMeta {
        EventMeta {
            subscription_id: Some("sub_1".into()),
            session_id: "session_1".into(),
            history_epoch: "epoch_1".into(),
            event_seq: WireCounter::new(seq),
            timestamp_ms: 1_000 + seq,
        }
    }

    #[test]
    fn events_snapshot_replay_live_ordering_shapes_roundtrip() {
        let session = Session {
            session_id: "session_1".into(),
            history_epoch: "epoch_1".into(),
            revision: 2.into(),
            status: SessionStatus::Running,
            workspace_root: "/work".into(),
            title: None,
            active_turn_id: Some("turn_1".into()),
            latest_turn_id: Some("turn_1".into()),
            provider_binding: None,
            created_at_ms: 1,
            updated_at_ms: 2,
        };
        let turn = Turn {
            turn_id: "turn_1".into(),
            session_id: "session_1".into(),
            provider_binding: None,
            revision: 1.into(),
            status: TurnStatus::InProgress,
            kind: TurnKind::User,
            ordinal: 1,
            created_at_ms: 1,
            completed_at_ms: None,
        };
        let item = Item {
            item_id: "item_1".into(),
            session_id: "session_1".into(),
            turn_id: "turn_1".into(),
            event_seq: 3.into(),
            revision: 1.into(),
            status: ItemStatus::InProgress,
            created_at_ms: 1,
            body: ItemBody::AgentMessage {
                text: "partial".into(),
            },
        };

        // Snapshot boundary then deltas then live terminal — event_seq strictly increases.
        let ordered = vec![
            EventDocument::SessionUpdated(SessionEvent {
                meta: meta(1),
                session: session.clone(),
            }),
            EventDocument::TurnUpdated(TurnEvent {
                meta: meta(2),
                turn: turn.clone(),
            }),
            EventDocument::ItemStarted(ItemEvent {
                meta: meta(3),
                item: item.clone(),
            }),
            EventDocument::ItemDelta(ItemDeltaEvent {
                meta: meta(4),
                turn_id: "turn_1".into(),
                item_id: "item_1".into(),
                revision: 2.into(),
                delta: " more".into(),
                stream: "text".into(),
            }),
            EventDocument::ItemCompleted(ItemEvent {
                meta: meta(5),
                item: Item {
                    revision: 3.into(),
                    status: ItemStatus::Completed,
                    body: ItemBody::AgentMessage {
                        text: "partial more".into(),
                    },
                    ..item
                },
            }),
        ];
        let mut last = 0u64;
        for event in &ordered {
            let value = serde_json::to_value(event).unwrap();
            let seq: WireCounter = match event {
                EventDocument::SessionUpdated(e) => e.meta.event_seq.clone(),
                EventDocument::TurnUpdated(e) => e.meta.event_seq.clone(),
                EventDocument::ItemStarted(e) | EventDocument::ItemCompleted(e) => {
                    e.meta.event_seq.clone()
                }
                EventDocument::ItemDelta(e) => e.meta.event_seq.clone(),
                EventDocument::InteractionRequest(_) => unreachable!(),
            };
            assert!(seq.as_u64() > last);
            last = seq.as_u64();
            // Wire counters remain strings.
            let wire = serde_json::to_value(&seq).unwrap();
            assert!(wire.is_string());
            assert_eq!(
                serde_json::from_value::<EventDocument>(value.clone()).unwrap(),
                *event,
                "{value}"
            );
        }
    }

    #[test]
    fn events_invalid_epoch_and_interaction_request_shapes() {
        let interaction = InteractionRequest {
            interaction_id: "ix_1".into(),
            session_id: "session_1".into(),
            turn_id: "turn_1".into(),
            item_id: "item_1".into(),
            kind: InteractionKind::Approval,
            prompt: "run?".into(),
            choices: vec!["allow".into(), "deny".into()],
            expires_at_ms: Some(9_999),
        };
        let doc = EventDocument::InteractionRequest(interaction.clone());
        let value = serde_json::to_value(&doc).unwrap();
        assert_eq!(value["event"], "interaction/request");
        assert_eq!(
            serde_json::from_value::<EventDocument>(value).unwrap(),
            EventDocument::InteractionRequest(interaction)
        );

        // Cursor/epoch mismatch is represented by SubscribeParams + domain codes (wire only).
        let bad_epoch = json!({
            "sessionId": "session_1",
            "afterEventSeq": "10",
            "historyEpoch": "epoch_stale"
        });
        let params: crate::SubscribeParams = serde_json::from_value(bad_epoch).unwrap();
        assert_eq!(params.history_epoch.as_deref(), Some("epoch_stale"));
        assert_eq!(params.after_event_seq.as_u64(), 10);
    }
}
