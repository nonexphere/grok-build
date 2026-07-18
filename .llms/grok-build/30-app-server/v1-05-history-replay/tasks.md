# Tasks — v1-05 history, projection and replay

- [ ] `AS105-01` [D-AP.4] In a new App Server projection module, index canonical session files without becoming execution truth; run `cargo test -p xai-grok-app-server projection_rebuild`; accept delete/rebuild produces identical IDs/order.
- [ ] `AS105-02` [D-SP.14,D-AP.5] Persist/derive historyEpoch, eventSeq and entity revisions; run `cargo test -p xai-grok-app-server cursor_semantics`; accept stale/foreign/epoch-mismatched cursor fixtures fail explicitly.
- [ ] `AS105-03` [D-SP.15] Implement attach-boundary-replay-live subscription; run `cargo test -p xai-grok-app-server snapshot_then_live`; accept no gaps/duplicates under concurrent event production.
- [ ] `AS105-04` [D-SP.18] Implement retention/byte/queue boundaries; run `cargo test -p xai-grok-app-server replay_backpressure`; accept terminal events retained and explicit resync beyond limits.
- [ ] `AS105-05` [D-RF.3,D-RF.4] Normalize all major runtime fixtures; run `cargo test -p xai-grok-app-server projection_goldens`; accept stable Items and redaction across rebuild.
- [ ] `AS105-06` [D-TA.1,D-TA.2] Reuse the history path for `tower_agent_history` in App Server/Tower tools adapters; run `cargo test -p xai-grok-app-server -p xai-grok-tower-tools history_parity`; accept identical epoch/cursor/redaction semantics.
- [ ] `AS105-07` [D-TD.3] Record crash/rebuild RED/GREEN evidence; run `cargo test -p xai-grok-app-server history_rebuild -- --nocapture`; accept no SQLite-only fact required to execute a Session.
