//! Experimental App Server composition root.
//!
//! Injects a Shell-owned [`ShellRuntimeAdapter`] into the App Server processor.
//! Until the SessionActor command port is complete, the default inject uses the
//! faithful [`FakeRuntime`] so local stdio/in-process slices remain testable
//! from the binary crate without a second actor type.

use std::sync::Arc;

use xai_grok_app_server::FacadeProcessor;
use xai_grok_shell::app_server_runtime::ShellRuntimeAdapter;
use xai_grok_tower::FakeRuntime;

/// Build the experimental App Server processor for composition-root smoke tests.
pub fn experimental_app_server_processor() -> FacadeProcessor {
    let adapter = ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()));
    FacadeProcessor::new(Arc::new(adapter))
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
