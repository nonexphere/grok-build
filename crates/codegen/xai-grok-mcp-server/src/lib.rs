//! MCP server adapter scaffold owned by `40-mcp-control-plane/v1-01..02`.
//! The existing `xai-grok-mcp` remains the external-server MCP client.

pub use xai_grok_tower_tools::{TowerToolDescriptor, TOWER_TOOL_DESCRIPTORS, TOWER_TOOL_NAMES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTransport {
    Stdio,
    StreamableHttp,
}
