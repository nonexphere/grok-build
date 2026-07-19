# Wave C7-B — Shared FakeRuntime vs real adapter conformance suite

| Field | Value |
|---|---|
| Handoff | `handoffs/HANDOFF-C7-B-conformance-suite.md` |
| Role / Model | build / `glm-5.2` |
| Branch | `goblin-implement-epic-tree` |
| Status | **GREEN** — 18/18 `c7_conformance_*` pass |
| Evidence | `tests/c7/c7_conformance_{RED,GREEN,GREEN_gate}.log` |
| SCRATCH | `SCRATCH/waves/c7-b.md` |

## Goal

One normalized conformance suite that runs the same facade scenarios against
`FakeRuntime` and `ShellSessionActorRuntime` (storage + command-routing where
resident inject possible) and compares normalized results.

## Owned

- `crates/codegen/xai-grok-shell/tests/c7_conformance.rs` (NEW, 18 tests)
- `waves/c7-conformance.md` (this file), `tests/c7/*`

## Scenarios (handoff minimum) — all covered

- list / start / read / fork / replay — storage-backed, normalized comparison.
- turn start / steer / interrupt — real has resident via injected test
  spawner (`AutoCompleteSpawner` / `HeldTurnSpawner`); real `cmd_tx` consumer
  routes `SessionCommand::{Prompt,Interject,Cancel}` and persists through the
  real `JsonlStorageAdapter` (NOT FakeRuntime).
- unsupported archive honesty — real returns `unsupported` (no data loss);
  Fake supports it. Divergence documented.

## Must NOT

- Require live credentials or full production spawn — the real adapter's
  no-spawner path is exercised separately (`start_turn_without_resident`
  returns `unsupported`); the turn scenarios use the test spawner, not
  production spawn.

## Normalization

Non-deterministic fields (session/turn/item ids, timestamps, revision
counters) are stripped; semantic shape (status, workspace, epoch, counts,
event kinds, error codes, body types) is kept. Conformance is asserted
where it holds; divergences are asserted with the exact expected difference
and documented reason.

## Conformance verdict

CONFORM on: idempotency (dedup + conflict), invalid workspace, list count +
workspace set, read (fresh), resume (same id + unknown), replay (snapshot +
epoch mismatch + after-turn non-empty), interrupt (running + unknown),
steer (Item status + turn_id present), start_turn (kind).

DIVERGE (documented, honest): archive (R6 product decision), fresh-session
status, turn status snapshot timing, turn ordinal offset, steer body type,
replay turn-lifecycle projection, start_turn without resident (C1-J/C2-A
HUMAN creds BLOCKER).

## Validation

```bash
bash scripts/run-rust-test-gate.sh c7_conformance \
  cargo test -p xai-grok-shell --test c7_conformance
cargo test -p xai-grok-shell --test c1_shell_port --test c1_turn_lifecycle
```

Results: 18/18 `c7_conformance_*` GREEN; 18/18 `c1_real_adapter_*` + 9/9
`c1_turn_*` GREEN (no regression).

## Residual

None for this slice. Closing the documented divergences is owned by:
- C1-J / C2-A — production `spawn_session_on_thread` assembly (HUMAN creds).
- C3-F — turn-lifecycle projection (Shell writes no turn events).
- R6 — archive product decision (keep-on-disk vs delete).
