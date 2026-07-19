# Residual review — C1-J Production spawn seam + Medium finding fixes

| Field | Value |
|---|---|
| Wave | C1-J (C1-G production-spawn residual + C1-H F-1..F-5) |
| Mode | implementation review (residual) |
| Reviewer | review harness (read-only, glm-5.2) |
| Date | 2026-07-19 |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

The production spawn **seam** is REAL and proven with a real offline `cmd_tx`
consumer (NOT `FakeRuntime`). The production `spawn_session_on_thread`
**assembly** is honestly PARTIAL/BLOCKER (HUMAN credentials + composition-root
assembly owned by C2-A). Medium findings F-1/F-2/F-3 are fixed with
deterministic RED→GREEN; F-4 is GREEN-proven but its RED is race-dependent
(honestly not captured); F-5 is a comment honesty fix. No second actor, no
Fake hybrid.

## Severity summary

- Critical: 0
- High: 1 (F-1 — principal program BLOCKER, but correctly scoped/deferred)
- Medium: 2 (F-2, F-3)
- Low: 2 (F-4, F-5)

## Contract non-negotiables (re-checked against source)

- **No second `SessionActor`.** Static guard
  `shell_session_actor_runtime_defines_no_session_actor` present at
  `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs:1308`;
  asserts production source contains no `struct SessionActor` / `enum
  SessionActor`. The only real `SessionActor` remains `session/acp_session.rs:564`.
  New `spawn_locks` / `last_spawn_error` are bookkeeping, not actor state.
  PASS.
- **No Fake hybrid.** Static guard
  `shell_session_actor_runtime_does_not_use_fake_runtime` at
  `shell_session_actor_runtime.rs:1319` asserts production source has no
  `FakeRuntime::new` / `use xai_grok_tower::FakeRuntime` / `: FakeRuntime`.
  The C1-J test consumers use `JsonlStorageAdapter`, not `FakeRuntime`.
  PASS.
- **`SessionHandle` is `Clone + Send`; actor is `!Send`.** `RealSpawnFn`
  returns a `ResidentHandle` (channel + shared slot), never the `!Send` actor.
  No `JoinHandle`/`LocalSet` added. PASS.
- **No await across `std::sync::Mutex` guard.** `ensure_resident` releases
  the `residents`/`spawn_locks` `std::sync::Mutex` guards before awaiting the
  per-session `TokioMutex` and the spawner. PASS.
- **Tower ≠ Shell.** No Tower edits in this wave. PASS.
- **Secrets.** No secrets introduced; the BLOCKER message enumerates missing
  HUMAN credentials but does not embed any. PASS.

## Evidence reviewed

- Wave note: `.llms/execution/app-server-mcp-tower-corrective/waves/c1-production-spawn.md`
- Handoff: `.llms/.../handoffs/HANDOFF-C1-J-production-spawn.md`
- GREEN gate: `.llms/.../tests/c1/c1_production_spawn_GREEN_gate.log`
  (c1_production_spawn 7/7, c1_turn_lifecycle 9/9, c1_shell_port 18/18,
  app_server_runtime invariants 7/7, composition root 11/11).
- RED: `.llms/.../tests/c1/c1_production_spawn_RED.log` (3/7 fail
  deterministically with F-1/F-2/F-3 stubbed back to C1-G behavior).
- Source guards: `shell_session_actor_runtime.rs:1308,1319`.

## Findings

### F-1 — Production `spawn_session_on_thread` assembly BLOCKER (High, high confidence)
The real factory requires HUMAN credentials + ~80 args assembled at the
composition root (C2-A owns composition wiring). This slice provides the
  seam (`with_production_spawn` + `RealSpawnFn`); C2-A must inject a real
`spawn_session_on_thread`-backed closure. Until then, `ProductionSpawner::new()`
returns `unsupported` enumerating the exact missing production dependencies,
and turn methods surface that BLOCKER via `no_resident_error`. This is the
principal C1-G residual and remains PARTIAL. The wave correctly does NOT
claim production spawn DONE. **This is the dominant program-level blocker;
it cascades into C3-G/C4-F/C7-B "start_turn without resident → unsupported"
divergences.**

### F-2 — F-3 full `SessionThread` reaping not implemented (Medium, high confidence)
The slice clears the stale `current_prompt_id` slot when the mailbox is
detected gone, so subsequent steer/interrupt return `turn_not_found` instead
of falsely matching. But a dead resident's `cmd_tx` stays in the `residents`
map (it just fails on next send); there is no `JoinHandle`/`SessionThread`
auto-evict. The handoff explicitly allowed documenting this. Acceptable but
leaves a slow leak of dead residents in long-running processes.

### F-3 — F-4 RED log is race-dependent (Medium, medium confidence)
The TOCTOU race window in `ensure_resident` is narrow; the F-4 GREEN test
proves the per-session async lock holds under 8-way contention (spawner
invoked exactly once), but reverting the F-4 fix does not deterministically
reproduce a double-spawn in CI. The F-4 RED is therefore not captured; F-1/
F-2/F-3 REDs are deterministic. Honest, but the F-4 guard is not
regression-proof against a future revert.

### F-4 — `next_ordinal` offset divergence (Low, high confidence)
`next_ordinal` is seeded from `Summary.num_messages.max(1)` then
`fetch_add(1) + 1`, so the first resident turn ordinal is 2 (not 1). This is
monotonic per session and matches the C7-B documented divergence, but it is
an off-by-one relative to the Fake adapter. Acceptable; documented.

### F-5 — R7/R8/R4/R10/R6/R11/R2 residuals unchanged (Low, high confidence)
Turn `idempotency_key` dedup (R7), steer `Item` shape (R8), resume
drain/replay (R4), `respond_interaction` (R10), `archive_session` (R6),
full `RuntimeEvent`/Turn/Item projection (R11/R2) remain unchanged from
C1-G. Honestly listed.

## Required fixes

None for this wave's bounded scope. The High finding (F-1) is a tracked
cross-wave dependency on C2-A + HUMAN credentials, not a defect in C1-J.

## Residual risk / dependencies

- C2-A must inject a real `spawn_session_on_thread`-backed closure at the
  composition root, and a live-actor integration test must pass with real
  creds before production spawn is claimed DONE.
- Full `SessionThread` reaping (F-3 follow-on) for long-running processes.

## Commands / results

- `cargo test -p xai-grok-shell --test c1_production_spawn` → 7/7 pass (GREEN gate log).
- `cargo test -p xai-grok-shell --test c1_turn_lifecycle` → 9/9 (no regression).
- `cargo test -p xai-grok-shell --test c1_shell_port` → 18/18 (no regression).
- `cargo test -p xai-grok-shell --lib app_server_runtime` → 7/7 (both static guards pass).
- `cargo test -p xai-grok-pager-bin --bins composition` → 11/11.
