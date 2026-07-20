# Tasks — capability truth and product conformance

- [ ] AS109-01 [F-05,F-10] Build typed capability registry in xai-grok-app-server from product dependencies/features; run ./scripts/run-rust-test-gate.sh capability_truth cargo test -p xai-grok-app-server -p xai-grok-pager-bin capability_truth; accept initialize exactly matches executable methods.
- [ ] AS109-02 [F-05,F-07] Unify OperationResult and RpcErrorData in protocol/schema/processors; run ./scripts/run-rust-test-gate.sh operation_error_contract cargo test -p xai-grok-app-server-protocol -p xai-grok-app-server operation_error_contract; accept code/message/retryable/operationId parity.
- [ ] AS109-03 [F-01,F-08] Run shared product-backed fixtures through in-process, real stdio subprocess and real WS listener; run ./scripts/run-rust-test-gate.sh product_transport_conformance cargo test -p xai-grok-app-server product_transport_conformance; accept normalized success/error/state equality.
- [ ] AS109-04 [F-10] Wire all Interaction kinds to parked actor delivery and controller lease; run ./scripts/run-rust-test-gate.sh product_interaction_delivery cargo test -p xai-grok-shell -p xai-grok-app-server product_interaction_delivery; accept reconnect/deny/timeout/closed receiver without double effect.
- [ ] AS109-05 [F-03,F-10] Prove replay/live/reconnect against canonical files and actor events; run ./scripts/run-rust-test-gate.sh product_replay_continuity cargo test -p xai-grok-shell -p xai-grok-app-server product_replay_continuity; accept exact epoch/cursors and no gap/duplicate.
- [ ] AS109-06 [F-10] Complete session fork/resume/archive and turn steer/interrupt product paths or mark capability unavailable; run ./scripts/run-rust-test-gate.sh all_announced_methods cargo test -p xai-grok-app-server all_announced_methods; accept no unsupported for advertised methods.
- [ ] AS109-07 [F-07] Generate protocol goldens/errors/capabilities consumed by SDK and MCP; run cargo run -p xai-grok-app-server-protocol --example generate-schema -- --check; accept byte-for-byte clean generation.
- [ ] AS109-08 [F-08] Add CI jobs named fake-conformance and product-integration; accept reports cannot merge or relabel SKIP as PASS.
- [ ] AS109-09 [DEAD] Inventory experimental, placeholder, unused and helper-only paths in target crates; run cargo clippy/check plus rg ledger; remove proven obsolete code or record owner/defer reason.
- [ ] AS109-10 [COMPAT] Run native Session versus Codex Thread adapter differential fixtures; accept adapter translates without owning state or changing core semantics.
- [ ] AS109-11 [TD] Capture human product smoke for WS start→turn→items→interaction→interrupt→archive.

