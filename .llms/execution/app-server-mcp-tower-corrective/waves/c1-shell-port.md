# C1-D — Shell SessionActor-backed facade port (wave note)

| Field | Value |
|---|---|
| Handoff | `HANDOFF-C1-D-shell-port-impl.md` |
| Branch | `goblin-implement-epic-tree` |
| Implementer | GLM `glm-5.2` (build) |
| Date | 2026-07-18 |
| Predecessor | C0-C architecture review = **GO** (preconditions R6/R10/R11) |

## 1. What landed

A real Shell-owned `GrokRuntimeFacade` — `ShellSessionActorRuntime` — that
maps the storage-backed facade methods to **existing** Shell symbols (C0-B §1)
and switches the composition root off `FakeRuntime` for the experimental
product path. `FakeRuntime` is retained for unit/conformance only.

### Files
- `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`
  (new) — the real port.
- `crates/codegen/xai-grok-shell/src/app_server_runtime/mod.rs` — declares the
  module, re-exports `ShellSessionActorRuntime`, **deletes the dormant
  `project_active_session_row` stub** (review A2), drops now-unused imports.
- `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` —
  composition root now injects `ShellSessionActorRuntime` rooted at
  `grok_home()`; adds `experimental_app_server_processor_with_root` test seam
  so tests never touch the real `grok_home()`.
- `crates/codegen/xai-grok-pager-bin/Cargo.toml` — adds `tempfile` dev-dep.
- `crates/codegen/xai-grok-shell/tests/c1_shell_port.rs` (new) — real-adapter
  integration tests.

## 2. Facade method → real Shell symbol map (as implemented)

| Facade method | Real symbol used | Status |
|---|---|---|
| `list_sessions` | `JsonlStorageAdapter::list_sessions` (`storage/jsonl/mod.rs:1340`) → `Summary` projected via `project_summary_to_session` | **REAL** (replaces dormant stub) |
| `read_session` | `StorageAdapter::load_summary` (`storage/mod.rs:628`); session row from real summary | **REAL** row; **PARTIAL** turns/items (R2 — empty until projector lands) |
| `start_session` | `StorageAdapter::init_session` (`storage/mod.rs:515`) writes `summary.json`; UUIDv7 id; adapter-side idempotency-key dedup | **REAL** storage write; **PARTIAL** — no `SessionActor` spawn (actor fixture gap) |
| `resume_session` | `find_info` (scan) + `load_summary` | **REAL** row; **PARTIAL** — no actor drain/replay (R4) |
| `fork_session` | `StorageAdapter::copy_session_data` (`storage/mod.rs:678`, the primitive `fork::fork_session` calls) | **REAL** copy; **PARTIAL** — no idempotency-key dedup (R5) |
| `archive_session` | — | **PARTIAL** — returns `unsupported` (R6 product decision pending; safest reversible no-op stub, NOT `delete_session`) |
| `start_turn` | — | **PARTIAL** — returns `unsupported` (requires live `SessionActor` + `SessionCommand::Prompt`) |
| `steer_turn` | — | **PARTIAL** — returns `unsupported` (`SessionCommand::Interject`) |
| `interrupt_turn` | — | **PARTIAL** — returns `unsupported` (`SessionCommand::Cancel`) |
| `respond_interaction` | — | **PARTIAL** — returns `unsupported` (R10 delivery-channel design pending) |
| `replay` | `StorageAdapter::load_summary` + `UpdatesIterator::open` over `updates.jsonl` → minimal `RuntimeEvent` projector | **REAL** snapshot + AgentMessageChunk/UserMessageChunk projection; **PARTIAL** — full Turn/ToolCall/Interaction lifecycle projection (R11) deferred |

## 3. Design sketches (preconditions R6 / R10 / R11 / R2)

### R6 — archive_session
**Decision: safest reversible no-op stub returning `unsupported`.** The only
existing destructive symbol is `StorageAdapter::delete_session` (irreversible
`remove_dir_all`); `close_session_explicit` keeps disk but finalizes the cloud
replica. Mapping `archive` → `delete` is data loss; adding a `hidden` flag on
`Summary` is a schema change. Per review §5.1, the default must NOT be silent
`delete_session`. The adapter returns `RuntimeError { code: "unsupported", .. }`
and the test `c1_real_adapter_archive_session_returns_unsupported_not_delete`
asserts the session is still on disk afterward. **Product decision required
before this becomes real** (hide-flag vs delete vs close).

### R10 — respond_interaction
**Decision: delivery-channel design deferred; returns `unsupported`.** The
parked oneshot is resolved via the leader's ACP response forwarding
(`leader/server.rs:492`), not a `SessionCommand`. The intended design (when
implemented): map `interaction_id` → `tool_call_id` and complete the parked
`PendingInteractionGuard` oneshot (`session/pending_interaction.rs:80-145`)
**without re-evaluating permission policy** — a delivery channel, not a second
permission engine. May require a new `SessionCommand::ResolveInteraction` or a
direct channel into the actor's pending registry. This is the C1 follow-on.

### R11 — replay RuntimeEvent projection
**Decision: minimal projector implemented; full lifecycle projection deferred.**
The event stream is: event 0 = `SessionChanged` snapshot (projected from the
real summary); events 1..N = projected `updates.jsonl` lines. The projector
(`project_update_to_event`) handles `AgentMessageChunk` → `ItemDelta` and
`UserMessageChunk` → `ItemCompleted(UserMessage)`. Full `Turn`/`ToolCall`/
`Interaction` lifecycle projection (grouping chunks into items, turn
boundaries, `InteractionRequested`) is deferred — this is a **projection over
existing `updates.jsonl` data, NOT a second replay buffer**. Cursor pagination
over `after_event_seq` is implemented (page size 100, matches Fake conformance).

### R2 — read_session Turn/Item projection
**Decision: deferred; returns empty turns/items.** Shares the R11 projector
surface. `SessionReadResult.session` is the real projected row; `turns`/`items`
are empty until the `updates.jsonl` → `Turn`/`Item` projector lands (synchronous,
paginated variant of the R11 projector).

## 4. Invariants preserved (re-verified)

- **No second `SessionActor`.** The real port defines no `SessionActor` type
  (static guard `shell_session_actor_runtime_defines_no_session_actor`). The
  only real actor remains `session/acp_session.rs:564`.
- **No hybrid Fake+JSONL authority.** The real port never constructs or imports
  `FakeRuntime` (static guard `shell_session_actor_runtime_does_not_use_fake_runtime`).
  One authority per session: the real `JsonlStorageAdapter`.
- **Tower must not gain Shell dependency.** Unchanged — Tower still does not
  import Shell; the adapter is injected at the composition root.
- **`ShellRuntimeAdapter` registry stays opaque tokens.** The wrapper is
  unchanged; it records one opaque token per session id and delegates to the
  real inner port.
- **`project_active_session_row` dormant stub removed** (review A2). `list_sessions`
  now projects the real `Summary`.

## 5. Composition root switch

`experimental_app_server_processor()` now builds
`ShellSessionActorRuntime::new(grok_home())` wrapped in `ShellRuntimeAdapter`.
`experimental_app_server_processor_with_root(root)` is the test seam (TempDir).
The composition test was updated to use the TempDir seam so it never writes to
the user's real `grok_home()`.

The switch is honest: storage-backed methods (list/read/start/resume/fork/replay)
are real; actor-backed methods (turn/interaction) return `unsupported` and are
documented as PARTIAL. `FakeRuntime` is kept for the Fake conformance tests in
`xai-grok-tower` and the Fake-backed `ShellRuntimeAdapter` tests in
`app_server_runtime/mod.rs` (RF102-06).

## 6. RED / GREEN evidence

Tests live in `crates/codegen/xai-grok-shell/tests/c1_shell_port.rs` (real
adapter) and `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs`
(composition root). Evidence logs are captured under
`.llms/execution/app-server-mcp-tower-corrective/tests/c1/`.

**Validation commands** (run from repo root):
```bash
# Real-adapter GREEN gate:
./scripts/run-rust-test-gate.sh c1_real_adapter \
  cargo test -p xai-grok-shell --test c1_shell_port

# Composition-root GREEN gate:
./scripts/run-rust-test-gate.sh composition_root \
  cargo test -p xai-grok-pager-bin --lib composition

# Invariant guards:
./scripts/run-rust-test-gate.sh shell_session_actor_runtime \
  cargo test -p xai-grok-shell --lib app_server_runtime

# Full shell lib + integration:
cargo test -p xai-grok-shell
cargo test -p xai-grok-pager-bin
```

## 7. Honest remaining gaps (PARTIAL — not claimed PASS)

- **R2** `read_session` Turn/Item projection — empty until projector lands.
- **R3** `start_session` does not spawn a live `SessionActor` — no turn
  lifecycle. Actor fixture gap (the actor is `!Send`; the facade is `Send+Sync`;
  wiring requires a dedicated thread + `LocalSet` + auth/credentials/tool-context,
  which is the C1 follow-on).
- **R4** `resume_session` resolves cwd via O(n) scan and does not drain/replay
  the old actor thread.
- **R5** `fork_session` does not dedup by `idempotency_key` (generates a fresh
  UUIDv7 each call, matching `fork::fork_session`).
- **R6** `archive_session` returns `unsupported` — product decision pending.
- **R7/R8/R9** `start_turn`/`steer_turn`/`interrupt_turn` return `unsupported` —
  require the live actor (`SessionCommand::{Prompt,Interject,Cancel}` +
  `current_prompt_id` match + `dispatch_lock`). Idempotency-key dedup not
  implemented.
- **R10** `respond_interaction` returns `unsupported` — delivery-channel design
  pending.
- **R11** `replay` full `RuntimeEvent` lifecycle projection deferred — only
  snapshot + AgentMessageChunk/UserMessageChunk projection today.
- **`provider_binding`** is left `None` on the projected `Session`; Shell
  `Summary` carries `current_model_id` but not a full `ProviderBinding`
  (credential_id/backend require actor-side resolution).

## 8. What did NOT change (out of scope)

- No MCP HTTP / WS server work (C3/C4).
- No provider vertical edits (C5).
- No protocol crate changes.
- No `MvpAgent` / `SessionActor` source edits — the actor fixture wiring is the
  C1 follow-on.
- `FakeRuntime` retained for unit/conformance.
