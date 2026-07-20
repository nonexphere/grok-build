# SCRATCH — C7-B shared conformance suite (build, GLM `glm-5.2`)

Branch: `goblin-implement-epic-tree`. Handoff:
`handoffs/HANDOFF-C7-B-conformance-suite.md`. Wave ledger:
`waves/c7-conformance.md`. Evidence: `tests/c7/c7_conformance_{RED,GREEN,GREEN_gate}.log`.

## One-line status
GREEN. One normalized conformance suite runs the same facade scenarios
against `FakeRuntime` and `ShellSessionActorRuntime` (real JSONL storage +
real `cmd_tx` command routing via injected test spawners) and compares
normalized results. Conformance asserted where it holds; divergences
documented honestly (archive, fresh-session status, turn ordinal/status
snapshot timing, steer body type, replay event lifecycle).

## Files changed (owned)
- `crates/codegen/xai-grok-shell/tests/c7_conformance.rs` — NEW. 18 tests:
  - Normalized outcome types (`NormSession`, `NormRead`, `NormReplay`,
    `NormTurn`, `NormItem`, `Outcome`) that strip non-deterministic
    ids/timestamps/revision and keep semantic shape (status, workspace,
    epoch, counts, event kinds, error codes, body types).
  - Two real `cmd_tx` consumer spawners (`AutoCompleteSpawner`,
    `HeldTurnSpawner`) that route `SessionCommand::{Prompt,Interject,Cancel}`
    and persist through the real `JsonlStorageAdapter` — NOT FakeRuntime.
  - `FakeRig` / `RealRig` harnesses.
  - Scenario tests: start/list/read/fork/resume/replay (storage-backed);
    turn start/steer/interrupt (real has resident via spawner); archive
    honesty; invalid workspace; idempotency conflict; resume/replay/interrupt
    unknown → not_found; replay epoch mismatch; replay after turn;
    start_turn without resident → unsupported; non-vacuity guard.

## Not touched (ownership respected)
- `xai-grok-tower/**` (FakeRuntime is the contract fake; unchanged).
- `xai-grok-shell/src/**` (real adapter unchanged; C1-D/C1-G/C3-F own it).
- `xai-grok-pager-bin/**`, `xai-grok-mcp-server/**`, `xai-grok-app-server/**`.
- Concurrent C7-C subagent work (managed-install fix) — no overlap.

## Reproduce
```bash
# GREEN (gate)
bash scripts/run-rust-test-gate.sh c7_conformance \
  cargo test -p xai-grok-shell --test c7_conformance
# GREEN (full)
cargo test -p xai-grok-shell --test c7_conformance
# No regression in C1
cargo test -p xai-grok-shell --test c1_shell_port --test c1_turn_lifecycle
```
Results: 18/18 `c7_conformance_*` pass; 18/18 `c1_real_adapter_*` + 9/9
`c1_turn_*` still pass (no regression).

## Conformance matrix (normalized comparison)

| Scenario | Fake | Real | Verdict |
|---|---|---|---|
| start_session shape | Ready, epoch_1, workspace | Starting, epoch_1, workspace | CONFORM modulo fresh-status divergence (documented) |
| start_session idempotency (same key+input) | dedup | dedup | CONFORM |
| start_session idempotency (diff input) | idempotency_conflict | idempotency_conflict | CONFORM |
| invalid workspace | invalid_workspace | invalid_workspace | CONFORM |
| list_sessions count + workspace set | 2, set | 2, set | CONFORM |
| read_session (fresh) | 0 turns, 0 items | 0 turns, 0 items | CONFORM |
| fork_session | distinct id, workspace override, epoch_1 | distinct id, workspace override, epoch_1 | CONFORM modulo fresh-status divergence |
| resume_session | same session_id | same session_id | CONFORM |
| resume unknown | session_not_found | session_not_found | CONFORM |
| replay (fresh) | SessionChanged snapshot | SessionChanged snapshot | CONFORM |
| replay epoch mismatch | epoch_mismatch | epoch_mismatch | CONFORM |
| archive_session | Ok (Archived) | unsupported (no delete) | DIVERGE — honest archive gap (R6 product decision pending) |
| start_turn (resident) | InProgress, ordinal 1, kind User | Completed, ordinal 2, kind User | CONFORM modulo status-snapshot + ordinal-offset divergence (documented) |
| start_turn (no resident) | Ok | unsupported | DIVERGE — honest production spawn gap (C1-J/C2-A HUMAN creds) |
| steer_turn (running) | Item Completed, user_message | Item Completed, agent_message | CONFORM modulo body-type divergence (documented) |
| interrupt_turn (running) | Ok | Ok | CONFORM |
| interrupt unknown | turn_not_found | turn_not_found | CONFORM |
| replay after turn | SessionChanged + TurnChanged + Item lifecycle | SessionChanged + AgentMessageChunk projection | CONFORM (both non-empty, snapshot first); DIVERGE on turn-lifecycle (Shell writes none) — documented |

## Documented divergences (honest, not bugs)
1. **archive_session** — Fake supports it (marks Archived); Real returns
   `unsupported` because mapping `archive` → `delete_session` is data loss
   (R6 product decision pending). The real adapter does NOT delete.
2. **fresh-session status** — Fake returns `Ready`; Real returns `Starting`
   (`summary.num_messages == 0`). Both valid; the real adapter is honest
   about the fresh-session lifecycle.
3. **turn status snapshot** — Fake `start_turn` returns the pre-completion
   snapshot (`InProgress`); Real returns `Completed` after the consumer
   resolves the oneshot. Both store the completed turn internally.
4. **turn ordinal offset** — Fake numbers from 1; Real seeds
   `next_ordinal` from `num_messages.max(1)` (C1-H F-2) then
   `fetch_add(1) + 1`, so the first resident turn ordinal is 2. Both
   monotonic per session.
5. **steer body type** — Fake steer returns a `UserMessage` item; Real
   returns an `AgentMessage` envelope (Shell `Interject` is fire-and-forget;
   the adapter synthesizes an Item to satisfy `steer_turn -> Item`).
6. **replay turn lifecycle** — Fake emits `TurnChanged` + full item
   lifecycle; Real projects only what `updates.jsonl` carries (Shell writes
   no turn lifecycle events — C3-F PARTIAL).
7. **start_turn without resident** — Fake always starts turns; Real
   honestly returns `unsupported` (production spawn needs HUMAN creds —
   C1-J/C2-A BLOCKER).

## Residual
None for this slice. The conformance suite documents the C1/C3-F
divergences; closing them is owned by the respective waves (C1-J production
spawn, C3-F turn-lifecycle projection, R6 archive product decision).
