# Tasks — Tower operations hardening

- [ ] `TW104-01` [D-TW.15] Implement drain state machine in `xai-grok-tower/src/lifecycle.rs`; run `./scripts/run-rust-test-gate.sh drain cargo test -p xai-grok-tower drain`; accept new work rejected and deadline-bounded cleanup.
- [ ] `TW104-02` [D-TW.15,D-SP.13] Add crash/restart fixture in `xai-grok-tower/tests/lifecycle.rs`; run `./scripts/run-rust-test-gate.sh restart_epoch cargo test -p xai-grok-tower restart_epoch`; accept stable Session ID and explicit resync when epoch changes.
- [ ] `TW104-03` [D-TW.9] Add lifecycle/latency/peak telemetry in `xai-grok-tower/src/telemetry.rs`; run `./scripts/run-rust-test-gate.sh lifecycle_metrics cargo test -p xai-grok-tower lifecycle_metrics`; accept bounded labels and no secrets.
- [ ] `TW104-04` [D-SEC.9] Add structured audit/redaction in Tower composition logging; run `./scripts/run-rust-test-gate.sh audit_canary cargo test -p xai-grok-tower audit_canary`; accept canary absent from all fields.
- [ ] `TW104-05` [D-TW.4] Add stale metadata matrix in `xai-grok-tower/tests/lifecycle.rs`; run `./scripts/run-rust-test-gate.sh stale_metadata cargo test -p xai-grok-tower stale_metadata`; accept no deletion based only on PID.
- [ ] `TW104-06` [D-TD.6] Validate Tower/shell/workspace integration paths; run `cargo check -p xai-grok-tower -p xai-grok-shell -p xai-grok-workspace`; accept every package gate green.
