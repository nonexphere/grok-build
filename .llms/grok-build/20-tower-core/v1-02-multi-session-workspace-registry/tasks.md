# Tasks — multi-session workspace registry

- [ ] `TW102-01` [D-TW.6,D-TW.7] Add session metadata types in `xai-grok-tower`; run `cargo test -p xai-grok-tower session_metadata`; accept metadata points to canonical files without copying transcripts.
- [ ] `TW102-02` [D-TW.8] Add canonical path/symlink race regression in `xai-grok-tower/tests/workspace_policy.rs`; run `cargo test -p xai-grok-tower workspace_symlink`; accept fail-closed authorization when resolution changes.
- [ ] `TW102-03` [D-TW.6,D-RF.1] Implement list/read facade adapter in `xai-grok-shell/src/app_server_runtime/`; run `cargo test -p xai-grok-shell app_server_multi_workspace`; accept N workspaces and stable Session IDs.
- [ ] `TW102-04` [D-TW.9] Add current/peak session/resident metrics in `xai-grok-tower/src/telemetry.rs`; run `cargo test -p xai-grok-tower telemetry_peaks`; accept deterministic increments, decrements and peak retention.
- [ ] `TW102-05` [D-TW.10] Add no-hard-cap regression in `xai-grok-tower/tests/lifecycle.rs`; run `cargo test -p xai-grok-tower no_implicit_session_cap`; accept telemetry records growth without admission rejection.
- [ ] `TW102-06` [D-TD.1] Run `cargo check -p xai-grok-tower`; accept warning-free package scaffold/integration.
