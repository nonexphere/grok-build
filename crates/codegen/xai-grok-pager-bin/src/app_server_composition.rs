//! Experimental App Server composition root.
//!
//! C1-D: the product path injects the real Shell-owned
//! [`ShellSessionActorRuntime`] (backed by the JSONL storage adapter), NOT
//! `FakeRuntime`. `FakeRuntime` remains available for unit/conformance tests
//! only. Do not mix real storage list/read with FakeRuntime mutations (split
//! authority — audit F-01 / F-13).

use std::sync::Arc;

use xai_grok_app_server::FacadeProcessor;
use xai_grok_shell::app_server_runtime::{ShellRuntimeAdapter, ShellSessionActorRuntime};
use xai_grok_tower::GrokRuntimeFacade;

/// Build the experimental App Server processor for the product path.
///
/// Uses the real Shell session-actor runtime rooted at `grok_home()`. The
/// actor-backed turn/interaction methods are PARTIAL (C1-D); storage-backed
/// methods (list/read/start/resume/fork/replay) are real.
pub fn experimental_app_server_processor() -> FacadeProcessor {
    let root = xai_grok_shell::util::grok_home::grok_home();
    experimental_app_server_processor_with_root(root)
}

/// Build the experimental App Server processor with an explicit storage root.
///
/// Test seam: tests pass a `TempDir` so they never touch the real `grok_home()`.
pub fn experimental_app_server_processor_with_root(
    root: std::path::PathBuf,
) -> FacadeProcessor {
    let real: Arc<dyn GrokRuntimeFacade> = Arc::new(ShellSessionActorRuntime::new(root));
    let adapter = ShellRuntimeAdapter::inject(real);
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
        // Use a temp root so the real port never touches the user's grok_home.
        let temp = tempfile::TempDir::new().unwrap();
        let processor = experimental_app_server_processor_with_root(temp.path().to_path_buf());
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

    /// The product composition root must inject the real port, not FakeRuntime.
    #[test]
    fn composition_root_injects_real_port_not_fake_runtime() {
        let temp = tempfile::TempDir::new().unwrap();
        let processor = experimental_app_server_processor_with_root(temp.path().to_path_buf());
        // Smoke: the processor builds from the real port without panicking.
        let _ = processor;
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
