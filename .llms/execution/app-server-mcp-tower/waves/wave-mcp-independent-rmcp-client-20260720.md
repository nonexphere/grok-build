# Wave: independent MCP client interoperability — 2026-07-20

## Objective

Prove that the public Streamable HTTP listener can be consumed by an MCP client
that does not use the repository's private resolver or managed MCP client.

## Change

- Added a feature-gated integration test using `rmcp` 2.1's
  `StreamableHttpClientTransport` and its own `reqwest` 0.13 client.
- The test starts the real axum listener, authenticates with the configured
  bearer token, performs the actual MCP initialize handshake, and calls
  `tools/list` through the external SDK.
- It asserts nine published tools and non-empty input schemas, then shuts the
  client down through the SDK lifecycle.
- A second test invokes `tower_agent_start` through the SDK and verifies
  `structuredContent.state=completed`; it then sends invalid arguments and
  verifies the public tool-level error shape (`isError=true`,
  `structuredContent.code=invalid_arguments`).

## Evidence

```text
cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http independent_rmcp_client_lists_tools_from_real_listener -- --nocapture
1 passed
```

## Result

MCP104-08 is partial: HTTP initialization, authenticated discovery,
structured tool results, and tool-level errors are proven with an independent
client. Stdio, protocol errors, reconnect, and cross-transport parity still
require coverage.

## Adjacent audit

`cargo check -p xai-grok-mcp-server --features streamable-http` produced no
package warnings. The only warning was a pre-existing workspace-level target
configuration warning in `xai-grok-pager-bin`; `process_mcp_stdio_batch` is
actively used by the stdio transport and parity tests and was not removed.

## Stdio evidence

```text
scripts/smoke/tower-mcp-stdio.sh
tower MCP stdio smoke: PASS (tools/list=9, start=completed, stdout=JSON-RPC-only, EOF=stderr)
```

The dependency-free smoke remains useful for stdout/stderr framing. In
addition, `mcp_stdio_rmcp.rs` now proves the same binary with the independent
`rmcp::TokioChildProcess` client transport.

```text
cargo test -p xai-grok-pager-bin --features mcp-stdio --test mcp_stdio_rmcp -- --nocapture
1 passed; mcp stdio eof
```

Reconnect and the full cross-transport matrix remain open.
