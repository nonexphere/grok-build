# Residual review — C3-F Canonical history / RuntimeEvent projection

| Field | Value |
|---|---|
| Wave | C3-F (C3 items 22–23 residual; R2/R11) |
| Mode | implementation review (residual) |
| Reviewer | review harness (read-only, glm-5.2) |
| Date | 2026-07-19 |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

One shared projector over canonical `updates.jsonl` derives both the
`read_session` Turn/Item view (R2) and the `replay` RuntimeEvent stream (R11)
from a single pass — no second replay buffer, no second execution authority,
no FakeRuntime. Honest PARTIALs are documented for events Shell never writes.
Findings are Medium/Low.

## Severity summary

- Critical: 0
- High: 0
- Medium: 2 (F-1, F-2)
- Low: 2 (F-3, F-4)

## Contract non-negotiables (re-checked against source)

- **No second `SessionActor`.** Static guard
  `shell_session_actor_runtime_defines_no_session_actor` at
  `shell_session_actor_runtime.rs:1308` still passes (per GREEN gate). PASS.
- **No hybrid Fake+JSONL authority.** Static guard
  `shell_session_actor_runtime_does_not_use_fake_runtime` at
  `shell_session_actor_runtime.rs:1319` still passes. `read_session` and
  `replay` both call `ShellSessionActorRuntime::project_history`, reading
  `updates.jsonl` once via `UpdatesIterator`. PASS.
- **No second replay buffer.** The projector reads `updates.jsonl` once;
  `read_session` and `replay` share the `ProjectedHistory` result. The
  `tool_call_id → item_id` correlation is derived from the ACP id already in
  the update, not a parallel state machine. PASS.
- **Tower does not gain a Shell dependency.** No Tower crate changes. PASS.
- **Secrets.** Projection forwards facade events as opaque structured
  objects; `SECRET_CANARIES` / `assert_no_secret_canaries` not weakened. PASS.

## Evidence reviewed

- Wave note: `.llms/execution/app-server-mcp-tower-corrective/waves/c3-history-projection.md`
- Handoff: `.llms/.../handoffs/HANDOFF-C3-F-history-projection.md`
- GREEN gate: `.llms/.../tests/c3/c3_history_projection_GREEN_gate.log`
  (16/16 pass; gate fragment `c3_read_session` matched, exit 0).
- RED: `tests/c3/c3_history_projection_RED.log` (9/16 FAIL with source
  reverted to C1-J empty projection; 7 PASS = honest-PARTIAL absence tests).
- Source guards: `shell_session_actor_runtime.rs:1308,1319`.

## Findings

### F-1 — `TurnChanged` not emitted in `replay`; crash-mid-turn not detected (Medium, high confidence)
Shell writes no turn lifecycle events to `updates.jsonl`. `read_session.turns`
are inferred from `UserMessageChunk` boundaries with `status: Completed`
inferred from persistence; crash-mid-turn is NOT detected (proven by
`c3_read_session_crash_mid_turn_inferred_completed_partial`). This is an
honest PARTIAL — the data does not exist in the canonical file. Closing it
requires Shell to write turn lifecycle events (a Shell-side change outside
this wave's owned files). Documented.

### F-2 — `InteractionRequested` not projected; item grouping not performed (Medium, high confidence)
Shell interaction requests are in-memory only (`pending_interaction.rs`),
never persisted to `updates.jsonl`, so they cannot be projected. Streaming
text chunks are not grouped into single items (each chunk is a separate
item) because Shell writes no item-id correlation for text chunks. Both
honest PARTIALs; closing requires Shell-side persistence changes.

### F-3 — `created_at_ms` is 0 (Low, high confidence)
`UpdatesIterator` parses `SessionUpdate` and discards the
`SessionUpdateEnvelope.timestamp`. Exposing it would require changing the
shared `UpdatesIterator` symbol, which is out of scope for this handoff
(owned files are `app_server_runtime/**` projection helpers). Documented.

### F-4 — `read_session` pagination not implemented (Low, high confidence)
`read_session` returns all turns/items; the `SessionReadResult` contract has
no pagination cursor, so this matches the contract. Large histories are
unbounded — flagged for a future paginated contract. Acceptable.

## Required fixes

None for this wave's bounded scope.

## Residual risk / dependencies

- Turn lifecycle + interaction projection require Shell to write those
  events to `updates.jsonl` (Shell-side change, owned by future turn-
  lifecycle work).
- `created_at_ms` requires an `UpdatesIterator` change (shared symbol).
- `provider_binding` on projected `Turn`/`Session` still `None` (depends on
  C1-G/C5 composition-root Turn binding).

## Commands / results

- `bash scripts/run-rust-test-gate.sh c3_read_session cargo test -p xai-grok-shell --test c3_history_projection` → exit 0, 16/16 pass (GREEN gate log).
- `cargo test -p xai-grok-shell --test c1_shell_port --test c1_turn_lifecycle --test c1_production_spawn --test c3_history_projection` → all green (no regression).
- `cargo test -p xai-grok-shell --lib app_server_runtime` → invariant guards pass.
- `cargo test -p xai-grok-tower --lib` → green (no Tower regression).
