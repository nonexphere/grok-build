//! Safe projection of runtime events to protocol Items/lifecycle events.

use xai_grok_app_server_protocol::{
    Item, ItemBody, ItemStatus, Session, Turn, WireCounter,
};

use crate::RuntimeEvent;

const SECRET_CANARIES: &[&str] = &[
    "sk-",
    "Bearer ",
    "access_token",
    "refresh_token",
    "client_secret",
    "XAI_API_KEY",
];

/// Redact contracted secret material from free-form text projected to clients.
/// When a canary prefix is found, the entire secret token (prefix + trailing
/// non-whitespace run) is replaced with a constant marker — never a revealing
/// suffix.
pub fn redact_text(input: &str) -> String {
    let mut out = input.to_owned();
    for canary in SECRET_CANARIES {
        while let Some(start) = out.find(canary) {
            let rest = &out[start + canary.len()..];
            let token_extra = rest
                .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ',' | '}' | ']'))
                .unwrap_or(rest.len());
            let end = start + canary.len() + token_extra;
            out.replace_range(start..end, "[REDACTED]");
        }
    }
    out
}

pub fn contains_secret_canary(input: &str) -> bool {
    SECRET_CANARIES.iter().any(|c| input.contains(c))
}

/// Map a runtime event into a safe projected form. Unknown event classes become
/// diagnostic Items — never silent drops.
pub fn project_runtime_event(event: RuntimeEvent) -> RuntimeEvent {
    match event {
        RuntimeEvent::ItemDelta {
            session_id,
            turn_id,
            item_id,
            revision,
            delta,
        } => RuntimeEvent::ItemDelta {
            session_id,
            turn_id,
            item_id,
            revision,
            delta: redact_text(&delta),
        },
        RuntimeEvent::ItemStarted(item) => RuntimeEvent::ItemStarted(redact_item(item)),
        RuntimeEvent::ItemCompleted(item) => RuntimeEvent::ItemCompleted(redact_item(item)),
        RuntimeEvent::SessionChanged(session) => RuntimeEvent::SessionChanged(session),
        RuntimeEvent::TurnChanged(turn) => RuntimeEvent::TurnChanged(turn),
        RuntimeEvent::InteractionRequested(mut req) => {
            req.prompt = redact_text(&req.prompt);
            RuntimeEvent::InteractionRequested(req)
        }
    }
}

fn redact_item(mut item: Item) -> Item {
    item.body = match item.body {
        ItemBody::AgentMessage { text } => ItemBody::AgentMessage {
            text: redact_text(&text),
        },
        ItemBody::CommandExecution {
            command,
            argv,
            cwd,
            output,
            exit_code,
        } => ItemBody::CommandExecution {
            command: redact_text(&command),
            argv: argv.into_iter().map(|a| redact_text(&a)).collect(),
            cwd,
            output: redact_text(&output),
            exit_code,
        },
        ItemBody::Error { code, message } => ItemBody::Error {
            code,
            message: redact_text(&message),
        },
        ItemBody::ProviderError {
            provider_id,
            code,
            safe_message,
        } => ItemBody::ProviderError {
            provider_id,
            code,
            safe_message: redact_text(&safe_message),
        },
        other => other,
    };
    item
}

/// Unknown runtime diagnostics become an explicit extension Item.
pub fn project_unknown_diagnostic(
    session_id: &str,
    turn_id: &str,
    event_seq: u64,
    kind: &str,
) -> Item {
    Item {
        item_id: format!("diag_{event_seq}"),
        session_id: session_id.into(),
        turn_id: turn_id.into(),
        event_seq: WireCounter::new(event_seq),
        revision: WireCounter::new(1),
        status: ItemStatus::Completed,
        created_at_ms: 0,
        body: ItemBody::Extension {
            extension_type: "unknown_runtime_event".into(),
            payload: serde_json::json!({
                "kind": kind,
                "safe": true,
            }),
        },
    }
}

#[allow(dead_code)]
pub fn project_session_identity(session: &Session) -> &str {
    &session.session_id
}

#[allow(dead_code)]
pub fn project_turn_identity(turn: &Turn) -> &str {
    &turn.turn_id
}

#[cfg(test)]
mod projection_tests {
    use super::*;
    use xai_grok_app_server_protocol::ItemStatus;

    #[test]
    fn projection_maps_item_delta_and_redacts_secrets() {
        let event = RuntimeEvent::ItemDelta {
            session_id: "s".into(),
            turn_id: "t".into(),
            item_id: "i".into(),
            revision: 1.into(),
            delta: "token Bearer sk-secret-value".into(),
        };
        let projected = project_runtime_event(event);
        match projected {
            RuntimeEvent::ItemDelta { delta, .. } => {
                assert!(!contains_secret_canary(&delta), "{delta}");
                assert!(delta.contains("[REDACTED]"));
                assert!(
                    !delta.contains("secret-value"),
                    "suffix must not leak: {delta}"
                );
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn projection_redaction_unknown_events_become_safe_diagnostics() {
        let item = project_unknown_diagnostic("s", "t", 9, "weird.event.v9");
        assert_eq!(item.status, ItemStatus::Completed);
        match &item.body {
            ItemBody::Extension {
                extension_type,
                payload,
            } => {
                assert_eq!(extension_type, "unknown_runtime_event");
                assert_eq!(payload["kind"], "weird.event.v9");
                assert_eq!(payload["safe"], true);
            }
            other => panic!("expected extension, got {other:?}"),
        }
        assert!(!contains_secret_canary(&format!("{item:?}")));
    }
}
