# Tasks — MCP remote security conformance

- [x] `MCP102-01` [D-SEC.1,D-SEC.2] Implement token-file/header auth in MCP server auth module; run `./scripts/run-rust-test-gate.sh bearer_header_only cargo test -p xai-grok-mcp-server bearer_header_only`; accept 0600 validation and header-only extraction.
- [x] `MCP102-02` [D-SEC.3,D-SEC.12] Add failure matrix in `xai-grok-mcp-server/tests/security.rs`; run `./scripts/run-rust-test-gate.sh auth_failures cargo test -p xai-grok-mcp-server auth_failures`; accept indistinguishable 401 and stable audit outcome.
- [x] `MCP102-03` [D-SEC.8,D-SEC.10] Enforce body/queue/SSE limits in MCP transports; run `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http`; accept explicit resync and no silent drop. Real body, buffer-expiry, queue-cap and resumption-error cases pass.
- [x] `MCP102-04` [D-SEC.9,D-SEC.11] Add audit canary/threat fixtures in MCP security tests; run `./scripts/run-rust-test-gate.sh redaction_canary cargo test -p xai-grok-mcp-server redaction_canary`; accept secrets absent from every sink.
- [~] `MCP102-05` [D-MCP.7] Run shared fixtures against HTTP and stdio drivers under `tests/adapter_parity.rs`; run `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http`; real HTTP/stdio list and error parity pass, while independent SDK/client interop remains open.
- [ ] [D-SEC.13] `(HUMAN, manual-verify, blocking: remote release)` accept threat model and verify TLS termination before public bind production release; cleartext remains experimental/unsafe.
