#![cfg(feature = "mcp-stdio")]

use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use tokio::process::Command;

#[tokio::test]
async fn rmcp_child_process_consumes_real_grok_oss_stdio_launcher() {
    let root = std::env::temp_dir().join(format!(
        "grok-mcp-rmcp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_grok-oss"));
    command
        .arg("tower")
        .arg("--stdio")
        .env("XAI_API_KEY", "test-api-key")
        .env("GROK_HOME", &root)
        .env("GROK_OSS_TOWER_AGENT_TYPE", "orchestrator");
    let transport = TokioChildProcess::new(command).expect("spawn grok-oss tower stdio");
    let client = ().serve(transport).await.expect("rmcp initialize");

    let tools = client
        .peer()
        .list_all_tools()
        .await
        .expect("rmcp tools/list");
    assert_eq!(tools.len(), 9);

    let result = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("tower_agent_start").with_arguments(
                serde_json::json!({
                    "workspaceRoot": root.to_string_lossy(),
                    "agentType": "orchestrator",
                    "idempotencyKey": format!("rmcp-stdio-{}", std::process::id())
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("rmcp tools/call");
    assert_ne!(result.is_error, Some(true));
    assert_eq!(
        result.structured_content.as_ref().unwrap()["state"],
        "completed"
    );

    client.cancel().await.expect("rmcp shutdown");
    std::fs::remove_dir_all(root).unwrap();
}
