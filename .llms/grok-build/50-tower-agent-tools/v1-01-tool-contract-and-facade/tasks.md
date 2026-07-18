# Tasks — Tower tool contract and facade

- [ ] `TA101-01` [D-TA.1,D-TA.2] Validate 18 input/output definitions in `xai-grok-app-server-protocol/schemas/tower-tools.schema.json`; run `cargo test -p xai-grok-app-server-protocol all_nine_tower_tool`; accept nine complete valid example pairs.
- [ ] `TA101-02` [D-RF.1,D-RF.2] Complete facade methods in `xai-grok-tower/src/lib.rs` and thin shell adapter; run `cargo test -p xai-grok-tower -p xai-grok-shell facade_conformance`; accept exact existing SessionActor operation mapping.
- [ ] `TA101-03` [D-RF.3,D-RF.4] Implement event projector in `xai-grok-tower/src/projection.rs`; run `cargo test -p xai-grok-tower projection`; accept every Item mapping and redaction canary absence.
- [ ] `TA101-04` [D-TA.3,D-TA.4] Implement shared errors/idempotency in `xai-grok-tower-tools`; run `cargo test -p xai-grok-tower-tools idempotency`; accept stable error examples, retry equality and conflict rejection.
- [ ] `TA101-05` [D-TA.11,D-TA.12] Add swarm/limit fixtures under `xai-grok-tower-tools/tests/`; run `cargo test -p xai-grok-tower-tools swarm_limits`; accept bounded waits and N independent Sessions without a hub entity.
- [ ] `TA101-06` [D-TD.3] Record RED/GREEN per mutation; run `cargo test -p xai-grok-tower-tools mutations -- --nocapture`; accept evidence for each behavior.
