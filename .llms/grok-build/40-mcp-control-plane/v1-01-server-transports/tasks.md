# Tasks — MCP server transports

- [x] `MCP101-01` [D-MCP.8,D-MCP.9] Keep `xai-grok-mcp-server` separate from client; run `cargo check -p xai-grok-mcp-server`; accept DAG law.
- [x] `MCP101-02` [D-MCP.1] Implement stdio adapter in `xai-grok-mcp-server/src/transport/stdio.rs`; run `./scripts/run-rust-test-gate.sh stdio cargo test -p xai-grok-mcp-server stdio`; accept protocol-only stdout and graceful EOF.
- [x] `MCP101-03` [D-MCP.1,D-MCP.2] Implement POST/GET/DELETE `/mcp` in `xai-grok-mcp-server/src/transport/http.rs`; run `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http`; accept bearer-header auth and safe SSE resume. Real listener suite passes.
- [x] `MCP101-04` [D-MCP.3,D-MCP.6] Register descriptors from `xai-grok-tower-tools` in the MCP server adapter; run `./scripts/run-rust-test-gate.sh tool_descriptors cargo test -p xai-grok-mcp-server tool_descriptors`; accept exact nine names/descriptions/input schemas.
- [x] `MCP101-05` [D-MCP.4] Map facade results/errors in `xai-grok-mcp-server/src/adapter.rs`; run `./scripts/run-rust-test-gate.sh adapter_parity cargo test -p xai-grok-mcp-server adapter_parity`; accept normalized equality with in-process fixtures.
- [x] `MCP101-06` [D-MCP.5] Add composition regression in MCP server tests; run `./scripts/run-rust-test-gate.sh no_local_self_injection cargo test -p xai-grok-mcp-server no_local_self_injection`; accept no local MCP client registration or recursive tool path.
