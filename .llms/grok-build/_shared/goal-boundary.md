# Goal runtime future boundary

[provenance: handoff §13.5, Goal plan, review D-GO.1..4]

Goal redesign is outside this pass and outside the App Server/Tower MVP. No Goal
v2 crate, module, schema or runtime stub is authorized here.

Future selection contract:

```toml
goal_runtime = "disabled" # or "v1" or "v2"
```

The eventual rollout epic owns default/migration. V1 and v2 cannot write one
authoritative record without an explicit dual-write/rollback contract.

## Hot-path inventory template

Every App Server implementation epic records whether it touches:

| Mechanism | Existing path/evidence | Goal coupling | Boundary action |
|---|---|---|---|
| SessionActor construction | path + test | none/read/write | keep behind runtime facade |
| prompt/turn queue | path + test | none/read/write | event projection only |
| tool registration | path + test | none/read/write | versioned Goal port later |
| continuation scheduling | path + test | none/read/write | no implicit Goal completion |
| completion/blocker evaluation | path + test | none/read/write | remain owned by selected runtime |
| session persistence/recovery | path + test | none/read/write | preserve unknown Goal artifacts |
| ACP/TUI/headless projection | path + test | none/read/write | version-neutral representation |

## Dual-version test strategy

When Goal work is separately approved: characterize v1 first; run identical
fixtures under disabled/v1/v2; verify ordinary Sessions unchanged; verify v1/v2
state isolation, rollback and recovery; fuzz transitions and crash points; and
require evidence-backed completion audits. These are Goal epic tests, not App
Server scaffold tests.
