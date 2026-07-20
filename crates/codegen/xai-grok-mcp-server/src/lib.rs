//! MCP server adapter over the shared tower tool semantic core.
//! The existing `xai-grok-mcp` remains the external-server MCP client only.

pub mod transport;

use std::sync::Arc;

use serde_json::{json, Value};
use xai_grok_tower::GrokRuntimeFacade;
use xai_grok_tower_tools::{
    invoke_tower_tool, tool_schema, TowerToolDescriptor, TOWER_TOOL_DESCRIPTORS,
};

pub use xai_grok_tower_tools::{
    TowerToolDescriptor as McpToolDescriptor, TOWER_TOOL_DESCRIPTORS as MCP_TOOL_DESCRIPTORS,
    TOWER_TOOL_NAMES as MCP_TOOL_NAMES, TOWER_TOOL_NAMES,
};

#[cfg(feature = "streamable-http")]
pub use transport::http_server::{
    McpHttpConfig, McpHttpHandle, McpHttpState, McpSession, McpSessionEvent,
    DEFAULT_MAX_MESSAGE_BYTES, DEFAULT_MAX_SESSION_EVENTS, MCP_PROTOCOL_VERSION_HEADER,
    MCP_SESSION_HEADER, bind_warning, run_mcp_http_server,
};

/// MCP wire protocol version advertised by `initialize` and enforced by the
/// Streamable HTTP `protocol-version` gate. Distinct from the App Server
/// protocol version (`xai_grok_app_server_protocol::PROTOCOL_VERSION`).
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTransport {
    Stdio,
    StreamableHttp,
}

/// List tools for MCP `tools/list`.
pub fn list_tools() -> Vec<TowerToolDescriptor> {
    TOWER_TOOL_DESCRIPTORS.to_vec()
}

/// Call a tool through the same semantic core as in-process adapters.
/// Returns a flat `code: message` string for backward compatibility with the
/// stdio adapter. See [`call_tool_typed`] for structured error codes.
pub async fn call_tool(
    runtime: Arc<dyn GrokRuntimeFacade>,
    agent_type: &str,
    explicit_opt_in: bool,
    name: &str,
    arguments: Value,
) -> Result<Value, String> {
    call_tool_typed(runtime, agent_type, explicit_opt_in, name, arguments)
        .await
        .map_err(|e| format!("{}: {}", e.code, e.message))
}

/// Typed variant of [`call_tool`] that preserves the stable Tower error code
/// so the HTTP adapter can emit `isError: true` structured content with the
/// canonical code (parity with in-process `ToolError`).
pub async fn call_tool_typed(
    runtime: Arc<dyn GrokRuntimeFacade>,
    agent_type: &str,
    explicit_opt_in: bool,
    name: &str,
    arguments: Value,
) -> Result<Value, xai_grok_tower_tools::ToolError> {
    invoke_tower_tool(runtime, agent_type, explicit_opt_in, name, arguments).await
}

/// Minimal MCP-looking JSON-RPC dispatcher for stdio/Streamable HTTP adapters.
pub async fn handle_mcp_jsonrpc(
    runtime: Arc<dyn GrokRuntimeFacade>,
    agent_type: &str,
    explicit_opt_in: bool,
    request: Value,
) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    match method {
        "tools/list" => json!({
            "jsonrpc":"2.0",
            "id": id,
            "result": {
                "tools": TOWER_TOOL_DESCRIPTORS.iter().map(|d| json!({
                    "name": d.name,
                    "description": d.description,
                    "inputSchema": tool_schema(d.name, false).unwrap_or_else(|| json!({"type":"object"})),
                })).collect::<Vec<_>>()
            }
        }),
        "tools/call" => {
            let name = request["params"]["name"].as_str().unwrap_or("");
            let args = request["params"]["arguments"].clone();
            match call_tool_typed(runtime, agent_type, explicit_opt_in, name, args).await {
                Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text": result.to_string()}],"structuredContent": result}}),
                Err(err) => {
                    // Parity with in-process ToolError: stable Tower code is
                    // preserved in structuredContent and `isError: true` so
                    // clients can branch on the canonical code instead of
                    // parsing a free-text message. The operation id is echoed
                    // back via the JSON-RPC `id`.
                    json!({
                        "jsonrpc":"2.0","id":id,
                        "result":{
                            "content":[{"type":"text","text": format!("{}: {}", err.code, err.message)}],
                            "structuredContent": xai_grok_tower_tools::tool_error_json(&err),
                            "isError": true
                        }
                    })
                }
            }
        }
        "initialize" => json!({
            "jsonrpc":"2.0",
            "id": id,
            "result": {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {"tools": {}},
                "serverInfo": {"name":"grok-oss-mcp-server","version":"0.0.0-experimental"}
            }
        }),
        other => json!({
            "jsonrpc":"2.0",
            "id": id,
            "error": {"code": -32601, "message": format!("Method not found: {other}")}
        }),
    }
}

/// Process NDJSON MCP stdio batch (one request object per line).
pub async fn process_mcp_stdio_batch(
    runtime: Arc<dyn GrokRuntimeFacade>,
    agent_type: &str,
    explicit_opt_in: bool,
    input: &str,
) -> Vec<String> {
    let mut out = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(req) = serde_json::from_str::<Value>(trimmed) {
            let resp = handle_mcp_jsonrpc(runtime.clone(), agent_type, explicit_opt_in, req).await;
            out.push(resp.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_tower::FakeRuntime;

    #[test]
    fn mcp_lists_exactly_nine_tools() {
        assert_eq!(list_tools().len(), 9);
        assert_eq!(TOWER_TOOL_NAMES.len(), 9);
    }

    #[tokio::test]
    async fn mcp_stdio_tools_call_shares_semantic_core_with_in_process() {
        use xai_grok_tower_tools::is_authorized;
        let rt = Arc::new(FakeRuntime::new());
        assert!(!is_authorized("build", false));
        let denied = call_tool(rt.clone(), "build", false, "tower_agent_list", json!({}))
            .await
            .unwrap_err();
        assert!(denied.contains("forbidden"));

        let batch = process_mcp_stdio_batch(
            rt,
            "orchestrator",
            false,
            &format!(
                "{}\n{}\n",
                json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
                json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"tower_agent_start","arguments":{"workspaceRoot":"/work","agentType":"build","idempotencyKey":"mcp-start-0001"}}})
            ),
        )
        .await;
        assert_eq!(batch.len(), 2);
        let list: Value = serde_json::from_str(&batch[0]).unwrap();
        assert_eq!(list["result"]["tools"].as_array().unwrap().len(), 9);
        let start: Value = serde_json::from_str(&batch[1]).unwrap();
        assert!(start["result"]["structuredContent"]["sessionId"].is_string());
    }

    #[test]
    fn no_self_mcp_loop_tool_names() {
        // Forbidden product name: a hub tool that re-enters MCP against self.
        let forbidden = ["tower", "agent", "hub"].join("_");
        assert!(!TOWER_TOOL_NAMES.contains(&forbidden.as_str()));
        let production = include_str!("lib.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(!production.contains(&forbidden));
    }

    #[test]
    fn tools_list_publishes_self_contained_input_schemas() {
        let response = handle_mcp_jsonrpc;
        let _ = response;
        let schemas: Vec<Value> = TOWER_TOOL_DESCRIPTORS
            .iter()
            .map(|d| tool_schema(d.name, false).expect("canonical input schema"))
            .collect();
        assert_eq!(schemas.len(), 9);
        assert!(schemas.iter().all(|schema| schema["type"] == "object"));
        assert!(schemas.iter().all(|schema| schema.get("$ref").is_none()));
    }
}

#[cfg(test)]
mod mcp_extra_tests {
    use super::*;

    #[test]
    fn tool_descriptors_exact_nine() {
        let tools = list_tools();
        assert_eq!(tools.len(), 9);
        let names: Vec<_> = tools.iter().map(|t| t.name).collect();
        assert_eq!(names, TOWER_TOOL_NAMES.to_vec());
    }

    #[test]
    fn no_local_self_injection_in_production_source() {
        let production = include_str!("lib.rs").split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("xai_grok_mcp::"));
        assert!(!production.contains("McpClient"));
    }

    #[tokio::test]
    async fn adapter_parity_list_start_via_mcp_jsonrpc() {
        let rt = Arc::new(xai_grok_tower::FakeRuntime::new());
        let list = handle_mcp_jsonrpc(
            rt.clone(),
            "orchestrator",
            false,
            serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        )
        .await;
        assert_eq!(list["result"]["tools"].as_array().unwrap().len(), 9);
    }
}
