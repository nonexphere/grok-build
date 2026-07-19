# C1-G — Turn lifecycle via SessionHandle (wave note)

| Field | Value |
|---|---|
| Handoff | `handoffs/HANDOFF-C1-G-turn-lifecycle.md` |
| Branch | `goblin-implement-epic-tree` |
| Implementer | GLM `glm-5.2` (build) |
| Date | 2026-07-18 |
| Predecessor | C1-D (storage-backed port) + C1-E/F reviews PASS |

## 1. What landed

The actor-fixture gap for turn methods is closed for the **command-routing
path**. `ShellSessionActorRuntime` now maintains a resident map of
`session_id → ResidentHandle` (the `Send`-able projection of a live
`SessionHandle`: `cmd_tx` + `current_prompt_id`). `start_turn` / `steer_turn` /
`interrupt_turn` no longer unconditionally return `unsupported` when a
resident handle exists; they enqueue the real `SessionCommand::{Prompt,
Interject, Cancel}` through the actor's command channel and map the result
back to the protocol `Turn` / `Item`.

The production spawn path is **PARTIAL** (the full `spawn_session_on_thread`
factory requires HUMAN credentials + agent/tool context that cannot be
assembled hermetically in this slice). The structure is in place via an
injectable `SessionSpawner` trait so a later handoff can replace
`ProductionSpawner` with the real factory without touching the facade method
bodies. Tests inject a **real `cmd_tx` consumer** (NOT `FakeRuntime`) that
processes the real `SessionCommand` enum and persists side effects to disk
through the real `JsonlStorageAdapter`, proving command routing against a
real actor path.

### Files
- `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`
  — added `ResidentHandle`, `SessionSpawner` trait, `ProductionSpawner`,
  resident map (`Mutex<HashMap<String, Resident>>`), `with_spawner` test
  seam, `ensure_resident` / `resident` / `next_ordinal` helpers, and the
  `start_turn` / `steer_turn` / `interrupt_turn` implementations. `start_session`
  and `resume_session` now attempt to re-resident the actor.
- `crates/codegen/xai-grok-shell/src/app_server_runtime/mod.rs` — re-exports
  `ResidentHandle` and `SessionSpawner` from the module root.
- `crates/codegen/xai-grok-shell/tests/c1_turn_lifecycle.rs` (new) — 9
  integration tests with a real `cmd_tx` consumer fixture
  (`TestActorSpawner` / `HeldTurnSpawner`).

## 2. Facade method → real Shell symbol map (C1-G)

| Facade method | Real symbol used | Status |
|---|---|---|
| `start_turn` | `SessionCommand::Prompt` (oneshot `PromptTurnResult`) via resident `cmd_tx`; `InputBlock` → `ContentBlock` conversion; `PromptTurnOk.completion_kind` → `TurnStatus` | **REAL** command routing + persistence (test-proven via real `cmd_tx` consumer); **PARTIAL** production spawn (needs creds) |
| `steer_turn` | `SessionCommand::Interject` (fire-and-forget) via resident `cmd_tx`; `turn_id` verified against shared `current_prompt_id` (mismatch → `turn_not_found`); adapter synthesizes protocol `Item` envelope (`AgentMessage`) since Shell `Interject` returns none | **REAL** command routing + turn-id guard; **PARTIAL** production spawn |
| `interrupt_turn` | `SessionCommand::Cancel` (fire-and-forget) via resident `cmd_tx`; `turn_id` verified against `current_prompt_id` (mismatch → `turn_not_found`) | **REAL** command routing + turn-id guard; **PARTIAL** production spawn |
| `respond_interaction` | — | **PARTIAL** — `unsupported` (R10 out of scope for C1-G) |
| `archive_session` | — | **PARTIAL** — `unsupported` (R6 product decision) |

## 3. REAL vs PARTIAL summary

### REAL (proven)
- **Command routing:** `start_turn` / `steer_turn` / `interrupt_turn` enqueue
  the real `SessionCommand` variants through the resident `cmd_tx`. A test
  with a real `cmd_tx` consumer (not FakeRuntime) proves the prompt is
  received, resolved via oneshot, and mapped to a protocol `Turn`.
- **Persistence through the command path:** the real consumer appends an
  `AgentMessageChunk` to `updates.jsonl` via `JsonlStorageAdapter` on
  `Prompt`; the test asserts `load_session` sees the update (real disk side
  effect through the real command path, not a fake).
- **Turn-id guard (R8/R9):** `steer_turn` / `interrupt_turn` verify
  `turn_id == current_prompt_id`; mismatch returns `turn_not_found`. Proven
  by `*_turn_id_mismatch_returns_turn_not_found` tests and the
  `*_against_running_turn_*` tests (which target a held running turn).
- **Foreground serialization (item 10):** `c1_turn_concurrent_starts_serialize_through_single_mailbox`
  proves two concurrent `start_turn`s both complete with distinct turn ids
  through the single consumer mailbox (mirrors the real actor's
  `dispatch_lock` + single-threaded mailbox).
- **Resume re-resident (R4 command path):** `c1_turn_resume_re_residents_actor_and_routes_turn`
  proves `resume_session` re-residents the actor and a subsequent turn
  routes. (R4 drain/replay of the old thread remains PARTIAL.)
- **Honest unsupported when no resident:** `c1_turn_start_turn_without_resident_returns_unsupported`
  proves the production path (default `ProductionSpawner`) returns
  `unsupported` for `start_turn` rather than faking a turn.
- **Invariants preserved:** the static guards
  `shell_session_actor_runtime_defines_no_session_actor` and
  `..._does_not_use_fake_runtime` still pass. `ResidentHandle` is a thin
  `Send`-able projection (channel + shared slot), NOT a second `SessionActor`.

### PARTIAL (honest, not claimed PASS)
- **Production actor spawn:** `ProductionSpawner` returns `unsupported`
  because the full `spawn_session_on_thread` factory needs credentials, an
  `AgentDefinition`, MCP/tool context, and a dedicated thread + `LocalSet`.
  Wiring that hermetically is a follow-on handoff (C1-G residual). The
  `SessionSpawner` trait + `with_spawner` seam are in place so the real
  factory drops in without facade-method changes.
- **Idempotency-key dedup for turns (R7):** `start_turn` / `steer_turn` /
  `interrupt_turn` do not dedup by `idempotency_key` (Shell `prompt` dedups
  via `dispatch_lock` + `send_now`, not by key). Adapter-side key dedup is
  deferred.
- **`InputBlock` → `ContentBlock` conversion:** minimal (Text/Mention/Skill
  flatten to `ContentBlock::Text`). The actor's `parse_prompt` does the rich
  rendering in production; the adapter only needs a faithful wire shape.
- **`steer_turn` `Item` shape (R8):** the adapter synthesizes an
  `AgentMessage` envelope because Shell `Interject` is fire-and-forget. The
  real `Item` representation for a steer is a product decision.
- **R4 resume drain/replay:** `resume_session` re-residents via the spawner
  but does not drain/replay the old actor thread (the test spawner attaches
  fresh). Full R4 drain/replay is out of scope.
- **`respond_interaction` (R10), `archive_session` (R6):** unchanged — still
  `unsupported` (out of scope for C1-G).

## 4. Invariants preserved (re-verified)

- **No second `SessionActor`.** `ResidentHandle` holds only `cmd_tx` +
  `current_prompt_id` (the `Send`-able projection of `SessionHandle`). The
  static guard `shell_session_actor_runtime_defines_no_session_actor` still
  passes. The only real `SessionActor` remains
  `session/acp_session.rs:564`.
- **No hybrid Fake+JSONL authority.** The real port never imports or
  constructs `FakeRuntime`. The static guard
  `shell_session_actor_runtime_does_not_use_fake_runtime` still passes. The
  test consumer uses the real `JsonlStorageAdapter`, not `FakeRuntime`.
- **Tower must not gain Shell dependency.** Unchanged — Tower still does not
  import Shell; the adapter is injected at the composition root.
- **No second turn state machine.** Turn state (`current_prompt_id`,
  completion kind → `TurnStatus`) is read from the real actor's shared slot
  and the real `PromptTurnResult`; the adapter does not introduce a parallel
  `Turn` state machine.
- **`SessionHandle` is `Clone + Send`; actor is `!Send`.** `ResidentHandle`
  is built from the `Send`-able subset of `SessionHandle` and never moves the
  actor across threads.

## 5. RED / GREEN evidence

Tests live in `crates/codegen/xai-grok-shell/tests/c1_turn_lifecycle.rs`.
Evidence logs are captured under
`.llms/execution/app-server-mcp-tower-corrective/tests/c1/`.

**Validation commands** (run from repo root):
```bash
# C1-G turn lifecycle GREEN gate:
./scripts/run-rust-test-gate.sh c1_turn \
  cargo test -p xai-grok-shell --test c1_turn_lifecycle

# Existing C1-D real-adapter gate (no regression):
./scripts/run-rust-test-gate.sh c1_real_adapter \
  cargo test -p xai-grok-shell --test c1_shell_port

# Invariant guards:
./scripts/run-rust-test-gate.sh shell_session_actor_runtime \
  cargo test -p xai-grok-shell --lib app_server_runtime

# Composition root (bin target):
cargo test -p xai-grok-pager-bin --bins composition
```

### Results
- **RED** (`c1_turn_lifecycle_RED.log`): 8 of 9 tests fail when the turn
  methods are stubbed back to `unsupported`; the no-resident test still
  passes (it expects `unsupported`). Confirms the tests exercise the real
  routing path.
- **GREEN** (`c1_turn_lifecycle_GREEN.log`, `c1_turn_lifecycle_GREEN_gate.log`):
  9/9 `c1_turn_*` tests pass.
- **No regression:** 18/18 `c1_real_adapter_*` pass; 7/7 `app_server_runtime`
  lib tests pass (including both static guards); 3/3 composition-root tests
  pass.
- **Pre-existing unrelated failures:** 2 shell lib tests
  (`claude_import::tests::gate_load_claude_env_returns_empty_when_marker_set`,
  `upload::trace::tests::classify_workspace_non_project_for_tmp`) fail on the
  baseline (env/`/tmp`-dependent) and are NOT caused by C1-G — verified by
  stashing the C1-G source changes and reproducing the same 2 failures.

## 6. Honest remaining gaps (PARTIAL — not claimed PASS)

- **R7** turn `idempotency_key` dedup not implemented.
- **Production actor spawn** (`spawn_session_on_thread` wiring with creds)
  — the next handoff replaces `ProductionSpawner`.
- **R4** resume drain/replay of the old actor thread.
- **R8** `steer_turn` `Item` shape is an adapter-side envelope (product
  decision pending).
- **R10** `respond_interaction` delivery channel (out of scope).
- **R6** `archive_session` product decision (out of scope).
- **R11** full `RuntimeEvent` projection; **R2** `read_session` Turn/Item
  projection (unchanged from C1-D).

## 7. What did NOT change (out of scope)

- No MCP HTTP / WS server work (C3/C4).
- No provider vertical edits (C5).
- No protocol crate changes.
- No `MvpAgent` / `SessionActor` / `spawn_session_on_thread` source edits —
  only the `app_server_runtime` adapter and tests. The spawn hook is a new
  trait in the adapter, not a redesign of the spawn path.
- `FakeRuntime` retained for unit/conformance.
