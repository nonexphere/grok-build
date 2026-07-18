# Tasks — v1-06 interactions and controller lease

- [ ] `AS106-01` [D-AP.1] In App Server controller modules, implement UNOWNED/HELD/RELEASED/RESOLVED lease transitions; run `cargo test -p xai-grok-app-server controller_lease`; accept revisioned exactly-one controller behavior.
- [ ] `AS106-02` [D-AP.2,D-SP.13] Keep Interaction ID distinct from connection request ID; run `cargo test -p xai-grok-app-server interaction_identity`; accept retry/reconnect preserves Interaction identity.
- [ ] `AS106-03` [D-AP.3] Implement disconnect/expiry policy; run `cargo test -p xai-grok-app-server controller_disconnect`; accept never auto-allow and explicit configured auto-deny only.
- [ ] `AS106-04` [D-SP.16] Add idempotent Interaction response storage; run `cargo test -p xai-grok-app-server interaction_idempotency`; accept duplicate replay and conflicting terminal response error.
- [ ] `AS106-05` [D-RF.2] Map response through `xai-grok-shell/src/app_server_runtime/` to the existing permission/elicitation command path; run `cargo test -p xai-grok-shell interaction_facade`; accept no second permission engine.
- [ ] `AS106-06` [D-MCP.7] Run Interaction conformance on in-process/stdio/WS; run `cargo test -p xai-grok-app-server interaction_conformance`; accept equal requests/errors and lease effects.
- [ ] [D-AP.6] `(HUMAN, product-decision, blocking: headless release policy)` approve default wait vs explicit auto-deny timeout; auto-allow is not an option.
