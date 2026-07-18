# Tasks — MCP remote security conformance

- [ ] `MCP102-01` [D-SEC.1,D-SEC.2] Implement token-file/header auth in MCP server auth module; run `cargo test -p xai-grok-mcp-server bearer_header_only`; accept 0600 validation and header-only extraction.
- [ ] `MCP102-02` [D-SEC.3,D-SEC.12] Add failure matrix in `xai-grok-mcp-server/tests/security.rs`; run `cargo test -p xai-grok-mcp-server auth_failures`; accept indistinguishable 401 and stable audit outcome.
- [ ] `MCP102-03` [D-SEC.8,D-SEC.10] Enforce body/queue/SSE limits in MCP transports; run `cargo test -p xai-grok-mcp-server limits`; accept explicit resync and no silent drop.
- [ ] `MCP102-04` [D-SEC.9,D-SEC.11] Add audit canary/threat fixtures in MCP security tests; run `cargo test -p xai-grok-mcp-server redaction_canary`; accept secrets absent from every sink.
- [ ] `MCP102-05` [D-MCP.7] Run shared fixtures against HTTP and stdio drivers under `tests/adapter_parity.rs`; run `cargo test -p xai-grok-mcp-server conformance`; accept identical tool semantics.
- [ ] [D-SEC.13] `(HUMAN, manual-verify, blocking: remote release)` accept threat model before public bind release.
