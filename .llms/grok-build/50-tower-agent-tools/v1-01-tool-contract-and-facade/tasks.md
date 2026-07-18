# Tasks — Tower tool contract and facade

- [x] `TA101-01` [D-TA.1,D-TA.2] Validate 18 input/output definitions in `xai-grok-app-server-protocol/schemas/tower-tools.schema.json`; run `./scripts/run-rust-test-gate.sh all_nine_tower_tool cargo test -p xai-grok-app-server-protocol all_nine_tower_tool`; accept nine complete valid example pairs.
- [x] `TA101-02` [D-RF.1,D-RF.2] Complete facade methods in `xai-grok-tower/src/lib.rs` and thin shell adapter; run `./scripts/run-rust-test-gate.sh facade_conformance cargo test -p xai-grok-tower -p xai-grok-shell facade_conformance`; accept exact existing SessionActor operation mapping.
- [x] `TA101-03` [D-RF.3,D-RF.4] Implement event projector in `xai-grok-tower/src/projection.rs`; run `./scripts/run-rust-test-gate.sh projection cargo test -p xai-grok-tower projection`; accept every Item mapping and redaction canary absence.
- [x] `TA101-04` [D-TA.3,D-TA.4] Implement shared errors/idempotency in `xai-grok-tower-tools`; run `./scripts/run-rust-test-gate.sh idempotency cargo test -p xai-grok-tower-tools idempotency`; accept stable error examples, retry equality and conflict rejection.
- [x] `TA101-05` [D-TA.11,D-TA.12] Add swarm/limit fixtures under `xai-grok-tower-tools/tests/`; run `./scripts/run-rust-test-gate.sh swarm_limits cargo test -p xai-grok-tower-tools swarm_limits`; accept bounded waits and N independent Sessions without a hub entity.
- [x] `TA101-06` [D-TD.3] Record RED/GREEN per mutation; run `./scripts/run-rust-test-gate.sh mutations cargo test -p xai-grok-tower-tools mutations`; accept evidence for each behavior.
