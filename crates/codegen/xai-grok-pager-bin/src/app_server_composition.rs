//! Experimental App Server composition root.
//!
//! Until SessionActor-backed `GrokRuntimeFacade` lands, tests inject FakeRuntime
//! only. Production mutations must not mix real storage list with fake mutation
//! authority (see adversarial audit F-01 / corrective contract).

use std::sync::Arc;

use xai_grok_app_server::FacadeProcessor;
use xai_grok_shell::app_server_runtime::ShellRuntimeAdapter;
use xai_grok_tower::FakeRuntime;

/// Build the experimental App Server processor for composition-root smoke tests.
pub fn experimental_app_server_processor() -> FacadeProcessor {
    let adapter = ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()));
    FacadeProcessor::new(Arc::new(adapter))
}

/// Resolve Tower instance id: explicit arg > env > default. No ambient last-used.
pub fn select_tower_instance_id(explicit: Option<&str>) -> String {
    if let Some(v) = explicit.filter(|s| !s.is_empty()) {
        return v.to_owned();
    }
    if let Ok(v) = std::env::var("GROK_TOWER_INSTANCE") {
        if !v.is_empty() {
            return v;
        }
    }
    "default".to_owned()
}

#[cfg(test)]
mod composition_tests {
    use super::*;
    use serde_json::json;
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;

    #[tokio::test]
    async fn composition_root_initialize_session_turn() {
        let processor = experimental_app_server_processor();
        let init = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize",
                    "params":{
                        "protocolVersion": PROTOCOL_VERSION,
                        "clientInfo":{"name":"pager-bin","version":"0"},
                        "capabilities":{}
                    }
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        assert!(init.contains(PROTOCOL_VERSION));
        let start = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":2,"method":"session/start",
                    "params":{"workspaceRoot":"/work","idempotencyKey":"comp-1"}
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        assert!(start.contains("sessionId"));
    }
}

#[cfg(test)]
mod tower_selection_tests {
    use super::*;

    #[test]
    fn tower_selection_prefers_explicit_then_env_then_default() {
        assert_eq!(select_tower_instance_id(Some("branch-a")), "branch-a");
        let id = select_tower_instance_id(Some(""));
        assert!(!id.is_empty());
        assert_eq!(select_tower_instance_id(Some("x")), "x");
    }
}
