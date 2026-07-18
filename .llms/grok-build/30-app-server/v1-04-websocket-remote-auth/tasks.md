# Tasks — v1-04 WebSocket and remote bearer

- [ ] `AS104-01` [D-TR.2] In `xai-grok-app-server/src/transport/websocket.rs`, implement subprotocol, text frames, ping/pong and 1 MiB cap; run `cargo test -p xai-grok-app-server websocket_transport`; accept binary/batch/oversized frames rejected.
- [ ] `AS104-02` [D-SEC.1..3] In a new App Server auth module, implement token-file validation and header-only constant-time bearer check; run `cargo test -p xai-grok-app-server bearer_auth`; accept the complete failure matrix has generic 401 responses.
- [ ] `AS104-03` [D-SEC.5..7] In config/CLI adapter modules, implement loopback defaults and explicit remote warning without Origin/scopes/TLS enforcement; run `cargo test -p xai-grok-app-server remote_bind`; accept locked permissive behavior and exact high-risk warning.
- [ ] `AS104-04` [D-SEC.8,D-SEC.9] Add redacted connection/audit fields; run `cargo test -p xai-grok-app-server redaction_canary`; accept all canary lengths absent from every output sink.
- [ ] `AS104-05` [D-SP.19,D-MCP.7] Extend black-box conformance to WebSocket; run `cargo test -p xai-grok-app-server websocket_conformance`; accept equality with in-process/stdio method semantics.
- [ ] `AS104-06` [D-SEC.10..12] Add network attacker/slow client/oversize tests; run `cargo test -p xai-grok-app-server control_plane_security`; accept bounded queues and safe failures.
- [ ] [D-SEC.13] `(HUMAN, manual-verify, blocking: remote release)` accept `_shared/control-plane-security.md` before advertising non-loopback release readiness.
