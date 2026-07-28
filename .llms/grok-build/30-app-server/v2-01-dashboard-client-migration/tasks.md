# Tasks — v2-01 dashboard migration (backlog boundary only)

- [ ] `UI201-01` [D-UI.1,D-BK.1] Before any implementation, inventory exact pager/dashboard/ACP/roster files and App Server contracts consumed; run `rg -n 'pager|dashboard|ACP|roster' .llms/grok-build/30-app-server/v2-01-dashboard-client-migration`; accept a separately approved v2 spec with no MVP code edit.
- [ ] `UI201-02` [D-UI.2] Characterize current `x.ai/sessions/list` and `x.ai/sessions/changed` in shell/pager fixtures; run `./scripts/run-rust-test-gate.sh roster cargo test -p xai-grok-shell roster` and `./scripts/run-rust-test-gate.sh dashboard cargo test -p xai-grok-pager dashboard`; accept byte/behavior fixtures before client replacement.
- [ ] `UI201-03` [D-UI.3] Design incremental cutover/rollback and parity matrix; run `rg -n 'cutover|rollback|parity|ACP' .llms/grok-build/30-app-server/v2-01-dashboard-client-migration`; accept ACP remains available until all dashboard observations match.
- [ ] [D-UI.3] `(HUMAN, product-decision, blocking: v2 implementation)` approve whether and when dashboard becomes an App Server client.
