# Wave — MCP structured error envelope (2026-07-19)

## Scope

Align MCP stdio and Streamable HTTP `tools/call` failures with the canonical
Tower error shape without inventing operation IDs for failed operations.

## Changes

- Added shared `tool_error_json` projection in `xai-grok-tower-tools`.
- Both MCP adapters now emit `code`, `message`, `retryable`, and
  `operationId: null` in `structuredContent`.
- Retryability is fail-closed and only true for explicitly transient codes:
  `operation_timeout`, `runtime_unavailable`, `tower_draining`, and
  `resync_required`.
- Added a unit test covering stable projection and fail-closed defaults.

## Evidence

```text
cargo test -p xai-grok-mcp-server --features streamable-http --no-fail-fast
20 unit tests passed
38 Streamable HTTP tests passed
```

The shared Tower tools suite also remained green with the new projection.

## Remaining gap

Failed tool calls still have no canonical operation identity, and full error
catalog/retryability parity with App Server remains a cross-boundary task.
