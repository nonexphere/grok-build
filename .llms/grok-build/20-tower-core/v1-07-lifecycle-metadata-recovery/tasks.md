# Tasks — lifecycle metadata and recovery

- [ ] TW107-01 [F-02,F-04] Define explicit SessionLifecycle × Residency transition table in xai-grok-tower and protocol tests; run ./scripts/run-rust-test-gate.sh session_lifecycle_matrix cargo test -p xai-grok-tower session_lifecycle_matrix; accept all valid and invalid transitions enumerated.
- [ ] TW107-02 [F-02] Extend canonical metadata with agentType, workspace display/resolved path, provider binding, sandbox profile, activeTurn and timestamps; run ./scripts/run-rust-test-gate.sh canonical_session_metadata cargo test -p xai-grok-tower -p xai-grok-shell canonical_session_metadata; accept roundtrip/rebuild without secrets.
- [ ] TW107-03 [F-02] Replace unknown/resident projections in tools/App Server with facade metadata; run ./scripts/run-rust-test-gate.sh canonical_session_rows cargo test -p xai-grok-tower-tools -p xai-grok-app-server canonical_session_rows; accept schema-valid rows.
- [ ] TW107-04 [F-04] Implement archive drain/detach and dormant resume state gates; run ./scripts/run-rust-test-gate.sh archive_resume_transitions cargo test -p xai-grok-shell -p xai-grok-tower archive_resume_transitions; accept archive never resident and resume only dormant.
- [ ] TW107-05 [F-04] Add crash-mid-turn recovery policy and terminal diagnostic projection; run ./scripts/run-rust-test-gate.sh crash_mid_turn cargo test -p xai-grok-shell crash_mid_turn; accept no permanently starting/running phantom.
- [ ] TW107-06 [F-02,F-04] Add concurrent archive/resume/start/interrupt property tests; run ./scripts/run-rust-test-gate.sh lifecycle_races cargo test -p xai-grok-tower -p xai-grok-shell lifecycle_races; accept one legal terminal outcome and one actor.
- [ ] TW107-07 [F-02] Implement stable updatedAt-desc/sessionId ordering and filter-bound opaque cursor primitives; run ./scripts/run-rust-test-gate.sh session_list_cursor cargo test -p xai-grok-tower session_list_cursor; accept foreign/filter-mismatched cursor errors.
- [ ] TW107-08 [F-04] Add restart/rebuild smoke with archived, dormant, failed and active fixtures; run ./scripts/run-rust-test-gate.sh lifecycle_recovery cargo test -p xai-grok-shell lifecycle_recovery; accept identity/epoch rules and no data loss.
- [ ] TW107-09 [TD] Update runbook and state diagram; validate every transition has a named test and error.

