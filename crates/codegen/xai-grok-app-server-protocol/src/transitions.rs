//! Pure Session/Turn/Item status transition validation (no runtime side effects).

use crate::{ItemStatus, SessionStatus, TurnStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    InvalidSessionTransition,
    InvalidTurnTransition,
    InvalidItemTransition,
}

pub fn session_can_transition(from: SessionStatus, to: SessionStatus) -> bool {
    use SessionStatus::*;
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        (Starting, Ready)
            | (Starting, Failed)
            | (Ready, Running)
            | (Ready, Dormant)
            | (Ready, Archived)
            | (Ready, Failed)
            | (Running, Ready)
            | (Running, WaitingForInput)
            | (Running, Completed)
            | (Running, Failed)
            | (WaitingForInput, Running)
            | (WaitingForInput, Ready)
            | (WaitingForInput, Failed)
            | (Dormant, Ready)
            | (Dormant, Archived)
            | (Completed, Archived)
            | (Failed, Archived)
    )
}

pub fn turn_can_transition(from: TurnStatus, to: TurnStatus) -> bool {
    use TurnStatus::*;
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        (Queued, InProgress)
            | (Queued, Interrupted)
            | (Queued, Declined)
            | (Queued, Failed)
            | (InProgress, WaitingForApproval)
            | (InProgress, WaitingForInput)
            | (InProgress, Completed)
            | (InProgress, Interrupted)
            | (InProgress, Failed)
            | (WaitingForApproval, InProgress)
            | (WaitingForApproval, Declined)
            | (WaitingForApproval, Interrupted)
            | (WaitingForApproval, Failed)
            | (WaitingForInput, InProgress)
            | (WaitingForInput, Interrupted)
            | (WaitingForInput, Failed)
    )
}

pub fn item_can_transition(from: ItemStatus, to: ItemStatus) -> bool {
    use ItemStatus::*;
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        (Pending, InProgress)
            | (Pending, Cancelled)
            | (Pending, Failed)
            | (InProgress, WaitingForApproval)
            | (InProgress, WaitingForInput)
            | (InProgress, Completed)
            | (InProgress, Failed)
            | (InProgress, Declined)
            | (InProgress, Cancelled)
            | (InProgress, Backgrounded)
            | (WaitingForApproval, InProgress)
            | (WaitingForApproval, Declined)
            | (WaitingForApproval, Cancelled)
            | (WaitingForApproval, Failed)
            | (WaitingForInput, InProgress)
            | (WaitingForInput, Cancelled)
            | (WaitingForInput, Failed)
            | (Backgrounded, Completed)
            | (Backgrounded, Failed)
            | (Backgrounded, Cancelled)
    )
}

pub fn assert_session_transition(
    from: SessionStatus,
    to: SessionStatus,
) -> Result<(), TransitionError> {
    session_can_transition(from, to)
        .then_some(())
        .ok_or(TransitionError::InvalidSessionTransition)
}

pub fn assert_turn_transition(from: TurnStatus, to: TurnStatus) -> Result<(), TransitionError> {
    turn_can_transition(from, to)
        .then_some(())
        .ok_or(TransitionError::InvalidTurnTransition)
}

pub fn assert_item_transition(from: ItemStatus, to: ItemStatus) -> Result<(), TransitionError> {
    item_can_transition(from, to)
        .then_some(())
        .ok_or(TransitionError::InvalidItemTransition)
}

#[cfg(test)]
mod entity_transition_tests {
    use super::*;
    use crate::{Item, ItemBody, ItemStatus, Session, SessionStatus, Turn, TurnKind, TurnStatus};
    use serde_json::json;

    #[test]
    fn entity_session_turn_item_transitions_and_roundtrip() {
        assert!(session_can_transition(
            SessionStatus::Starting,
            SessionStatus::Ready
        ));
        assert!(!session_can_transition(
            SessionStatus::Archived,
            SessionStatus::Running
        ));
        assert!(turn_can_transition(
            TurnStatus::InProgress,
            TurnStatus::Completed
        ));
        assert!(!turn_can_transition(
            TurnStatus::Completed,
            TurnStatus::InProgress
        ));
        assert!(item_can_transition(
            ItemStatus::InProgress,
            ItemStatus::Completed
        ));
        assert!(!item_can_transition(
            ItemStatus::Completed,
            ItemStatus::Pending
        ));

        let session = Session {
            session_id: "session_1".into(),
            history_epoch: "epoch_1".into(),
            revision: 1.into(),
            status: SessionStatus::Ready,
            workspace_root: "/work".into(),
            title: None,
            active_turn_id: None,
            latest_turn_id: None,
            provider_binding: None,
            created_at_ms: 1,
            updated_at_ms: 1,
        };
        let turn = Turn {
            turn_id: "turn_1".into(),
            session_id: "session_1".into(),
            provider_binding: None,
            revision: 1.into(),
            status: TurnStatus::Queued,
            kind: TurnKind::User,
            ordinal: 1,
            created_at_ms: 1,
            completed_at_ms: None,
        };
        let item = Item {
            item_id: "item_1".into(),
            session_id: "session_1".into(),
            turn_id: "turn_1".into(),
            event_seq: 1.into(),
            revision: 1.into(),
            status: ItemStatus::Pending,
            created_at_ms: 1,
            body: ItemBody::UserMessage {
                content: vec![crate::InputBlock::Text { text: "hi".into() }],
            },
        };
        let session_json = serde_json::to_value(&session).unwrap();
        assert_eq!(session_json["revision"], json!("1"));
        assert_eq!(
            serde_json::from_value::<Session>(session_json).unwrap(),
            session
        );
        assert_eq!(
            serde_json::from_value::<Turn>(serde_json::to_value(&turn).unwrap()).unwrap(),
            turn
        );
        assert_eq!(
            serde_json::from_value::<Item>(serde_json::to_value(&item).unwrap()).unwrap(),
            item
        );
    }

    #[test]
    fn entity_all_mvp_item_body_kinds_roundtrip() {
        let kinds = [
            ItemBody::UserMessage {
                content: vec![crate::InputBlock::Text { text: "a".into() }],
            },
            ItemBody::AgentMessage { text: "b".into() },
            ItemBody::ToolCall {
                tool_name: "t".into(),
                arguments: json!({}),
            },
            ItemBody::ToolResult {
                tool_name: "t".into(),
                output: json!({}),
                is_error: false,
            },
            ItemBody::CommandExecution {
                command: "echo".into(),
                argv: vec!["echo".into()],
                cwd: "/work".into(),
                output: String::new(),
                exit_code: Some(0),
            },
            ItemBody::FileChange {
                changes: vec![],
                summary: None,
            },
            ItemBody::Plan {
                content: "p".into(),
                steps: vec![],
            },
            ItemBody::Subagent {
                subagent_id: "s".into(),
                agent_type: "build".into(),
                description: "d".into(),
                result: None,
            },
            ItemBody::McpToolCall {
                server: "m".into(),
                tool_name: "t".into(),
                arguments: json!({}),
            },
            ItemBody::ReasoningSummary {
                summary: "r".into(),
            },
            ItemBody::Hook {
                hook_name: "h".into(),
                phase: "pre".into(),
                safe_summary: "ok".into(),
            },
            ItemBody::BackgroundTask {
                task_id: "bg".into(),
                safe_summary: "ok".into(),
            },
            ItemBody::Compaction {
                safe_summary: "c".into(),
            },
            ItemBody::ProviderError {
                provider_id: "p".into(),
                code: "e".into(),
                safe_message: "m".into(),
            },
            ItemBody::InteractionRequest {
                interaction_id: "i".into(),
                prompt: "ok?".into(),
                choices: vec!["yes".into()],
            },
            ItemBody::Error {
                code: "e".into(),
                message: "m".into(),
            },
            ItemBody::Extension {
                extension_type: "x".into(),
                payload: json!({}),
            },
        ];
        for body in kinds {
            let item = Item {
                item_id: "item".into(),
                session_id: "session".into(),
                turn_id: "turn".into(),
                event_seq: 0.into(),
                revision: 0.into(),
                status: ItemStatus::Pending,
                created_at_ms: 0,
                body: body.clone(),
            };
            let value = serde_json::to_value(&item).unwrap();
            assert!(value.get("type").is_some(), "{value}");
            assert_eq!(serde_json::from_value::<Item>(value).unwrap().body, body);
        }
    }
}
