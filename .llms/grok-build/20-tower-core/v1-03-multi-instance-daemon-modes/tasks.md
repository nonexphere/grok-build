# Tasks — multi-instance daemon modes

- [ ] `TW103-01` [D-TW.1,D-TW.2] Implement validated ID/layout in `xai-grok-tower/src/instance.rs`; run `./scripts/run-rust-test-gate.sh tower_instance cargo test -p xai-grok-tower tower_instance`; accept valid format and isolated secure roots.
- [ ] `TW103-02` [D-TW.3] Wire explicit/env/default selection in the narrow `xai-grok-pager-bin` CLI adapter; run `./scripts/run-rust-test-gate.sh tower_selection cargo test -p xai-grok-pager-bin tower_selection`; accept no ambient last-used state.
- [ ] `TW103-03` [D-TW.4,D-TW.5] Add two-instance fixture in `xai-grok-tower/tests/lifecycle.rs`; run `./scripts/run-rust-test-gate.sh two_instances cargo test -p xai-grok-tower two_instances`; accept disjoint endpoint/lock/token/state and no session steal.
- [ ] `TW103-04` [D-TR.8] Add co-start parser tests in App Server/MCP composition config; run `./scripts/run-rust-test-gate.sh co_start cargo test -p xai-grok-app-server -p xai-grok-mcp-server co_start`; accept every valid matrix row and reject dual stdio.
- [ ] `TW103-05` [D-SEC.6] Add bind-warning fixture in App Server config tests; run `./scripts/run-rust-test-gate.sh remote_bind_warning_exact cargo test -p xai-grok-app-server remote_bind_warning_exact`; accept loopback default and explicit non-loopback warning only.
- [ ] `TW103-06` [D-TD.3] Capture RED/GREEN for contention and isolation; run `./scripts/run-rust-test-gate.sh instance_contention cargo test -p xai-grok-tower instance_contention`; accept reproducible evidence.
