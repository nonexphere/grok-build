# C1-J — Production spawn seam + Medium finding fixes (wave note)

| Field | Value |
|---|---|
| Handoff | `handoffs/HANDOFF-C1-J-production-spawn.md` |
| Branch | `goblin-implement-epic-tree` |
| Implementer | GLM `glm-5.2` (build) |
| Date | 2026-07-18 |
| Predecessor | C1-G (turn lifecycle) + C1-H/I reviews PASS_WITH_FINDINGS |

## 1. What landed

This wave closes the C1-G production-spawn residual as far as the bounded
`app_server_runtime/**` scope allows and fixes the accepted Medium findings
from C1-H (F-1..F-5) where cheap and correct.

### Production spawn seam (C1-G residual)

The full `spawn_session_on_thread` factory (`session/acp_session_impl/spawn.rs`)
requires ~80 arguments — HUMAN `Credentials`, `AgentDefinition`, `ToolContext`,
`GatewaySender`, `ModelsManager`, `PersistenceHandle`, MCP servers,
`WorkspaceOps`, `PluginRegistry`, `AuthManager`, `SamplingConfig`, and a
dedicated OS thread + `LocalSet` — that cannot be assembled hermetically
inside `app_server_runtime/**` without HUMAN-provided auth and without
editing the composition root (owned by handoff C2-A). This slice therefore
implements the **largest real path available within scope** and documents the
remaining BLOCKER honestly:

- New `RealSpawnFn` type alias + `ProductionSpawner::with_real_spawn(real)` +
  `ShellSessionActorRuntime::with_production_spawn(root, real)`. When a
  `RealSpawnFn` is injected, `ProductionSpawner::spawn` delegates to it; the
  facade method bodies (`start_session`/`resume_session`/`start_turn`/...) are
  unchanged. This is the production-grade seam the composition root (C2-A)
  wires to a real `spawn_session_on_thread`-backed closure.
- `ProductionSpawner::new()` (default, used by `ShellSessionActorRuntime::new`)
  has no real spawn function and returns `unsupported` enumerating the exact
  missing production dependencies (credentials, `AgentDefinition`,
  `ToolContext`, `GatewaySender`, `ModelsManager`, `PersistenceHandle`,
  `McpServers`, `WorkspaceOps`, `PluginRegistry`, `AuthManager`,
  `SamplingConfig`, `spawn_session_on_thread`, C2-A).
- `ensure_resident` now records the spawner's error message per session in
  `last_spawn_error`; the turn methods' `no resident` error surfaces that
  message so a caller sees WHY there is no resident (actionable BLOCKER),
  not a generic "no resident" string.
- `mod.rs` re-exports `RealSpawnFn`.

A test (`c1_prod_spawn_seam_routes_real_resident_when_spawn_fn_injected`)
injects a **real offline `cmd_tx` consumer** (NOT `FakeRuntime`) via
`with_production_spawn` and proves `start_session` → `start_turn` obtains a
real resident `SessionHandle` with a real disk side effect through the real
command path. This is the "minimal offline/test spawn path" the handoff
describes — REAL for the seam + command routing, PARTIAL for the production
`spawn_session_on_thread` assembly.

### Medium finding fixes (C1-H)

- **F-1 (Medium):** `steer_turn` synthesized `Item.event_seq` is now a
  per-session monotonic sequence (`Resident::next_event_seq`, an
  `AtomicU64`), not a wall-clock timestamp (`now_ms()`). Seeded from
  `Summary.num_messages + 1` on resident bring-up so synthesized events stay
  above the persisted replay range. Test
  `c1_f1_steer_turn_event_seq_is_monotonic_not_wall_clock` asserts strict
  monotonicity across 3 steers.
- **F-2 (Medium):** `next_ordinal` is now seeded from
  `Summary.num_messages.max(1)` on resident bring-up (in `ensure_resident`
  after a successful spawn) so ordinals do not collide across process
  restarts. Test `c1_f2_next_ordinal_seeds_from_summary_on_resume` persists
  `num_messages = 5`, resumes on a fresh runtime over the same disk, and
  asserts the first turn's ordinal is `> 5`.
- **F-3 (Medium):** stale-handle risk reduced. When the actor mailbox is
  detected gone (`cmd_tx.send` fails, or the `start_turn` oneshot is
  dropped), the adapter now clears the stale `current_prompt_id` slot
  (`clear_current_turn`) so the turn-id guard stays honest — a subsequent
  steer/interrupt against the dead turn returns `turn_not_found` instead of
  falsely matching or returning `session_closed` forever. Test
  `c1_f3_dead_actor_clears_stale_current_prompt_id_and_returns_session_closed`
  proves the stale slot is cleared after `session_closed`.
- **F-4 (Medium):** TOCTOU in `ensure_resident` eliminated. A per-session
  async lock (`spawn_locks: Mutex<HashMap<String, Arc<TokioMutex<()>>>>`)
  serializes concurrent `ensure_resident` calls for the same session with
  double-checked locking (fast path → per-session lock → re-check → spawn).
  The `residents` map lock is never held across the spawn `await`. Test
  `c1_f4_concurrent_ensure_resident_does_not_double_spawn` runs 8 concurrent
  `resume_session` calls for the same session and asserts the spawner is
  invoked exactly once. (NOTE: the TOCTOU race is timing-dependent; the test
  proves the lock holds under contention but the RED log for F-4 is
  race-dependent and not deterministically reproducible — see §5.)
- **F-5 (Medium):** honesty fix in the test comment. The
  `c1_turn_concurrent_starts_serialize_through_single_mailbox` comment now
  says "mirrors the single-threaded mailbox ordering; `dispatch_lock`
  foreground exclusivity is not replicated in the test fixture" (was
  overclaiming `dispatch_lock`).

### Files
- `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`
  — added `RealSpawnFn`, `ProductionSpawner::{new, with_real_spawn}`,
  `ShellSessionActorRuntime::with_production_spawn`, `last_spawn_error`
  surfacing, `no_resident_error` helper, `Resident::next_event_seq`,
  `next_event_seq`/`clear_current_turn` helpers, per-session `spawn_locks`,
  summary-seeded ordinals/event_seq in `ensure_resident`, F-3 stale-slot
  clearing in `start_turn`/`steer_turn`/`interrupt_turn`.
- `crates/codegen/xai-grok-shell/src/app_server_runtime/mod.rs` — re-export
  `RealSpawnFn`.
- `crates/codegen/xai-grok-shell/tests/c1_production_spawn.rs` (new) — 7
  integration tests (seam routes/seam BLOCKER message/resume, F-1/F-2/F-3/F-4)
  with a real `cmd_tx` consumer fixture (`CountingActorSpawner`,
  `HeldTurnSpawner`, `DropableActorSpawner`).
- `crates/codegen/xai-grok-shell/tests/c1_turn_lifecycle.rs` — F-5 comment
  honesty fix.

## 2. REAL vs PARTIAL summary

### REAL (proven)
- **Production spawn seam:** `with_production_spawn` + `RealSpawnFn` +
  `ProductionSpawner::with_real_spawn` route a real resident `SessionHandle`
  when a real spawn function is injected. Test-proven with a real offline
  `cmd_tx` consumer (NOT `FakeRuntime`) that persists a real disk side effect
  through the real `JsonlStorageAdapter` via the real `SessionCommand` path.
- **BLOCKER message surfacing:** when no spawn function is injected, turn
  methods return `unsupported` with the exact missing production dependency
  list (credentials, `AgentDefinition`, `ToolContext`, `GatewaySender`,
  `ModelsManager`, `PersistenceHandle`, `McpServers`, `WorkspaceOps`,
  `PluginRegistry`, `AuthManager`, `SamplingConfig`,
  `spawn_session_on_thread`, C2-A). Test asserts the message names these.
- **F-1 monotonic `event_seq`:** per-session `AtomicU64`, strict monotonicity
  proven across 3 steers.
- **F-2 ordinal seeding:** seeded from `Summary.num_messages` on resume;
  proven across a process-restart simulation (fresh runtime, same disk).
- **F-3 stale-slot clearing:** proven — dead actor's stale
  `current_prompt_id` is cleared, subsequent steer/interrupt return
  `turn_not_found`.
- **F-4 no double-spawn:** proven under 8-way concurrent contention —
  spawner invoked exactly once.
- **Invariants preserved:** the static guards
  `shell_session_actor_runtime_defines_no_session_actor` and
  `..._does_not_use_fake_runtime` still pass. `ResidentHandle` is still the
  thin `Send`-able projection (channel + shared slot); no `JoinHandle`, no
  `LocalSet`, no second `SessionActor`. The new `spawn_locks` and
  `last_spawn_error` are bookkeeping, not actor state.

### PARTIAL (honest — not claimed DONE)
- **Production `spawn_session_on_thread` assembly:** the real factory
  requires HUMAN credentials + ~80 args assembled at the composition root
  (C2-A owns composition wiring). This slice provides the seam
  (`with_production_spawn` + `RealSpawnFn`); C2-A must inject a real
  `spawn_session_on_thread`-backed closure. **BLOCKER:** HUMAN credentials
  (api_key / auth token) + the composition-root assembly of
  `AgentDefinition`/`ToolContext`/`GatewaySender`/`ModelsManager`/MCP
  servers/`WorkspaceOps`/`PluginRegistry`/`AuthManager`/`SamplingConfig`.
  This is the principal C1-G residual and remains PARTIAL.
- **F-3 full `SessionThread` reaping:** this slice reduces stale-handle risk
  by clearing `current_prompt_id` when the mailbox is detected gone, but it
  does not add a `JoinHandle`/`SessionThread` that auto-evicts the dead
  resident from the `residents` map. A dead resident's `cmd_tx` stays in the
  map (it just fails on next send). Full reaping is a larger design follow-on
  (the handoff explicitly allowed documenting this).
- **F-4 RED log is race-dependent:** the TOCTOU race window is narrow; the
  F-4 test proves the lock holds under contention, but reverting the F-4
  fix does not deterministically reproduce a double-spawn in CI (the RED
  log for F-4 is therefore not captured; F-1/F-2/F-3 REDs are deterministic
  and captured).
- **R7** turn `idempotency_key` dedup (unchanged from C1-G).
- **R8** `steer_turn` `Item` shape is an adapter-side envelope (product
  decision pending; F-1 only fixed `event_seq` monotonicity, not the shape).
- **R4** resume drain/replay of the old actor thread (unchanged).
- **R10** `respond_interaction`, **R6** `archive_session` (unchanged).
- **R11/R2** full `RuntimeEvent`/Turn/Item projection (unchanged).

## 3. Invariants preserved (re-verified)

- **No second `SessionActor`.** `ResidentHandle` still holds only `cmd_tx` +
  `current_prompt_id`. The new `spawn_locks` (per-session async locks) and
  `last_spawn_error` (per-session error strings) are bookkeeping, not actor
  state. The static guard
  `shell_session_actor_runtime_defines_no_session_actor` still passes. The
  only real `SessionActor` remains `session/acp_session.rs:564`.
- **No Fake hybrid.** The real port never imports/constructs `FakeRuntime`.
  The static guard `shell_session_actor_runtime_does_not_use_fake_runtime`
  still passes. The test consumers use `JsonlStorageAdapter`, not
  `FakeRuntime`.
- **Tower ≠ Shell.** Unchanged — Tower still does not import Shell; the
  adapter is injected at the composition root. No Tower edits in this wave.
- **No second turn state machine.** Turn state is still read from the real
  actor's shared slot and the real `PromptTurnResult`. `next_event_seq` is a
  wire-counter allocator for synthesized envelopes, not a turn state machine.
- **`SessionHandle` is `Clone + Send`; actor is `!Send`.** `ResidentHandle`
  is still the `Send`-able subset. The `RealSpawnFn` returns a
  `ResidentHandle` (channel + shared slot), never the `!Send` actor.
- **No await-across-`std::sync::Mutex`-guard.** `ensure_resident` releases
  the `residents`/`spawn_locks` `std::sync::Mutex` guards before awaiting the
  per-session `TokioMutex` and the spawner. `next_event_seq`/`next_ordinal`/
  `clear_current_turn`/`no_resident_error` lock, read, release — no await.

## 4. RED / GREEN evidence

Tests live in `crates/codegen/xai-grok-shell/tests/c1_production_spawn.rs`
(7 tests) + the existing `c1_turn_lifecycle.rs` (9 tests, F-5 comment fix).
Evidence logs are captured under
`.llms/execution/app-server-mcp-tower-corrective/tests/c1/` and copied to
`/tmp/grok-goal-5598c3040156/implementer/waves/c1-j/`.

**Validation commands** (run from repo root):
```bash
# C1-J production-spawn + Medium fixes GREEN:
cargo test -p xai-grok-shell --test c1_production_spawn

# Existing C1-G turn lifecycle (no regression):
cargo test -p xai-grok-shell --test c1_turn_lifecycle

# Existing C1-D real-adapter (no regression):
cargo test -p xai-grok-shell --test c1_shell_port

# Invariant guards:
cargo test -p xai-grok-shell --lib app_server_runtime

# Composition root (bin target):
cargo test -p xai-grok-pager-bin --bins composition
```

### Results
- **GREEN** (`c1_production_spawn_GREEN.log`,
  `c1_production_spawn_GREEN_gate.log`): 7/7 `c1_*` tests pass; 9/9
  `c1_turn_*` pass; 18/18 `c1_real_adapter_*` pass; 7/7 `app_server_runtime`
  lib tests pass (including both static guards); 11/11 composition-root tests
  pass.
- **RED** (`c1_production_spawn_RED.log`): with F-1/F-2/F-3 stubbed back to
  the C1-G behavior (wall-clock `event_seq`, ordinal reset to 1, no-op
  `clear_current_turn`), 3 of 7 tests fail deterministically:
  - `c1_f1_steer_turn_event_seq_is_monotonic_not_wall_clock` — wall-clock
    `event_seq` collides (`[1784429020846, 1784429020846, 1784429020846]`),
    not strictly increasing.
  - `c1_f2_next_ordinal_seeds_from_summary_on_resume` — ordinal resets to
    `2` (1 + fetch_add), not `> 5`.
  - `c1_f3_dead_actor_clears_stale_current_prompt_id_and_returns_session_closed`
    — stale slot not cleared → second steer returns `session_closed` instead
    of `turn_not_found`.
  - F-4 (`c1_f4_concurrent_ensure_resident_does_not_double_spawn`) and the
    3 seam tests still pass with the stubs (F-4's TOCTOU is race-dependent;
    the seam tests exercise the seam which exists independent of F-1/F-2/F-3).
  This confirms the F-1/F-2/F-3 tests are non-vacuous. F-4's RED is not
  deterministically reproducible (narrow race window) and is documented as
  such; the GREEN test proves the lock holds under 8-way contention.

## 5. Honest remaining gaps (PARTIAL — not claimed DONE)

- **Production `spawn_session_on_thread` assembly** — the principal
  residual. **BLOCKER:** HUMAN credentials + composition-root assembly of
  the ~80 factory args (C2-A owns composition wiring). The
  `with_production_spawn` + `RealSpawnFn` seam is in place; C2-A injects the
  real factory. Do NOT claim production spawn DONE until C2-A wires the real
  factory AND a live-actor integration test passes with real creds.
- **F-3 full `SessionThread` reaping** — `JoinHandle`-based auto-evict of
  dead residents from the `residents` map (larger design).
- **R7** turn `idempotency_key` dedup; **R8** steer `Item` shape (product
  decision); **R4** resume drain/replay; **R10** `respond_interaction`;
  **R6** `archive_session`; **R11/R2** full projection — unchanged from C1-G.
- **F-4 RED log** — race-dependent, not captured; GREEN proves the lock
  holds under contention.

## 6. What did NOT change (out of scope)

- No MCP HTTP / WS server work (C3/C4).
- No provider vertical edits (C5).
- No protocol crate changes.
- No `MvpAgent` / `SessionActor` / `spawn_session_on_thread` source edits —
  only the `app_server_runtime` adapter and tests. The spawn seam is a new
  injection point in the adapter, not a redesign of the spawn path.
- No composition-root (`xai-grok-pager-bin`) edits — C2-A owns composition.
- No mcp-server, multi-auth, app-server transport, or pager-bin composition
  edits (per handoff Must-NOT-edit).
- `FakeRuntime` retained for unit/conformance.
