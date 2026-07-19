# Tasks — leader characterization and promotion

- [x] `TW101-01` [D-TW.11,D-TW.12] Add byte fixtures/tests under `xai-grok-shell/src/leader/`; run `./scripts/run-rust-test-gate.sh leader cargo test -p xai-grok-shell leader`; accept unchanged discovery/handshake bytes.
- [x] `TW101-02` [D-TW.4] Add the single-winner race fixture under `xai-grok-shell/src/leader/lock.rs`; run `./scripts/run-rust-test-gate.sh connect_or_spawn_has_single_winner cargo test -p xai-grok-shell connect_or_spawn_has_single_winner`; accept one spawned leader and every contender connected to it.
- [x] `TW101-03` [D-CR.5,D-RF.7] Implement only the adapter seam in `xai-grok-tower`; run `cargo test -p xai-grok-tower`; accept no second actor type.
- [ ] `TW101-04` [D-TW.13,D-TW.14] Add registry ownership tests in `xai-grok-tower/tests/one_actor.rs` plus the shell adapter tests; run `./scripts/run-rust-test-gate.sh one_actor cargo test -p xai-grok-tower -p xai-grok-shell one_actor`; accept mutations serialized by the existing actor. <!-- C0-A PARTIAL: F-02 test injects FakeRuntime; "mutations serialized by existing actor" not proven; opaque token registry. Wave C1-10. -->
- [ ] `TW101-05` [D-TD.3] Record RED/GREEN commands and observed failure in PR evidence; run `./scripts/run-rust-test-gate.sh leader_characterization cargo test -p xai-grok-tower leader_characterization`; accept reproducible test names. <!-- C0-A PARTIAL: F-09 named RED/GREEN evidence missing. -->
- [ ] [D-TW.11,D-TW.12] `(HUMAN, manual-verify, non-blocking)` compare characterization fixture to a real local leader before merge.
