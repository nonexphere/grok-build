# Wave: product transport conformance — 2026-07-20

## Evidence

### Real App Server WebSocket + Shell runtime

```text
cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws_ -- --nocapture
4 passed per binary target; 0 failed
```

The tests bind a real loopback WS listener, enforce bearer auth, execute
`initialize` and `session/start` through the Shell-backed runtime, and cover
real replay/resync behavior.

### Real MCP HTTP + Shell runtime

```text
cargo test -p xai-grok-pager-bin --features mcp-streamable-http \
  mcp_http_composition_bind_auth_and_dispatch_roundtrip -- --nocapture
1 passed per binary target; 0 failed
```

This covers real listener bind/auth, MCP initialize, nine-tool discovery, and
`tower_agent_start` through the Shell-backed runtime.

### Real product stdio + independent SDK

```text
cargo test -p xai-grok-pager-bin --features mcp-stdio \
  --test mcp_stdio_rmcp -- --nocapture
1 passed; mcp stdio eof
```

## Remaining scope

AS109-03 remains partial until one shared fixture normalizes success, error,
state, and operation identity across in-process, real stdio, real WS, and MCP
transport legs. The current evidence proves the product legs separately.
