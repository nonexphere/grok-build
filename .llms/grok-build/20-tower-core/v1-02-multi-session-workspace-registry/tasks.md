# Tasks — multi-session workspace registry

- [x] `TW102-01` [D-TW.6,D-TW.7] Add session metadata types in `xai-grok-tower`; run `./scripts/run-rust-test-gate.sh session_metadata cargo test -p xai-grok-tower session_metadata`; accept metadata points to canonical files without copying transcripts.
- [x] `TW102-02` [D-TW.8] Add canonical path/symlink race regression in `xai-grok-tower/tests/workspace_policy.rs`; run `./scripts/run-rust-test-gate.sh workspace_symlink cargo test -p xai-grok-tower workspace_symlink`; accept fail-closed authorization when resolution changes.
- [x] `TW102-03` [D-TW.6,D-RF.1] Implement list/read facade adapter in `xai-grok-shell/src/app_server_runtime/`; run `./scripts/run-rust-test-gate.sh app_server_multi_workspace cargo test -p xai-grok-shell app_server_multi_workspace`; accept N workspaces and stable Session IDs.
- [x] `TW102-04` [D-TW.9] Add current/peak session/resident metrics in `xai-grok-tower/src/telemetry.rs`; run `./scripts/run-rust-test-gate.sh telemetry_peaks cargo test -p xai-grok-tower telemetry_peaks`; accept deterministic increments, decrements and peak retention.
- [x] `TW102-05` [D-TW.10] Add no-arbitrary-product-cap regression in `xai-grok-tower/tests/lifecycle.rs`; run `./scripts/run-rust-test-gate.sh resource_budget_admission cargo test -p xai-grok-tower resource_budget_admission`; accept dormant Sessions remain listable while resident/Turn/load admission fails explicitly at measured safety budgets.
- [x] `TW102-06` [D-TD.1] Run `cargo check -p xai-grok-tower`; accept warning-free package scaffold/integration.
