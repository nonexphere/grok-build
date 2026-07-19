# C0-B — SessionActor / leader command map (evidence-backed)

> Handoff C0-B deliverable. Read-only characterization. Every row cites a real
> Shell symbol with `file:fn` evidence. `UNVERIFIED` marks gaps where no
> existing symbol maps cleanly to a facade method and C1-D must add a thin
> adapter (not a second actor).

Branch: `goblin-implement-epic-tree`
Scope: `crates/codegen/xai-grok-shell/**` + composition root
`crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs`

## 0. Ownership invariants (proven)

- **Tower defines no `SessionActor` and never imports Shell.**
  `crates/codegen/xai-grok-tower/src/lib.rs:36-65` declares the
  `GrokRuntimeFacade` trait; `crates/codegen/xai-grok-tower/src/lib.rs:91-132`
  (`leader_characterization_tower_has_no_second_actor_type`) asserts Tower
  sources contain no `struct SessionActor`/`enum SessionActor` and the crate
  Cargo.toml does not depend on `xai-grok-shell`.
- **The only real `SessionActor` lives in Shell.**
  `crates/codegen/xai-grok-shell/src/session/acp_session.rs:564`
  `pub(crate) struct SessionActor` (with `impl` blocks spread across
  `session/acp_session_impl/**`). It is `!Send` and runs on a dedicated
  thread + `LocalSet` — see §3.
- **Current inject seam is `ShellRuntimeAdapter` (Shell side), still backed by
  `FakeRuntime` in the product composition root.**
  `crates/codegen/xai-grok-shell/src/app_server_runtime/mod.rs:33-52`
  `pub struct ShellRuntimeAdapter { inner: Arc<dyn GrokRuntimeFacade>, registry: Mutex<SessionRegistry> }`.
  `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs:12-16`
  `experimental_app_server_processor()` injects `Arc::new(FakeRuntime::new())`.
  This is the C1-D switch point — **must not** stay FakeRuntime in product.
- **No hybrid Fake+JSONL authority exists** (grep for
  `SessionStorageHybridRuntime` returns no matches — confirmed by C0-C review).
  The adapter must keep that property: one authority per session.

## 1. Facade → existing Shell symbol map

Columns: Facade method | Existing Shell symbol (file:fn) | Message/command type | Persistence touch | Permission/interaction? | Test entrypoints | Risk

### 1.1 Session lifecycle

| Facade method | Existing Shell symbol (file:fn) | Message/command type | Persistence touch | Permission/interaction? | Test entrypoints | Risk |
|---|---|---|---|---|---|---|
| `list_sessions` | `JsonlStorageAdapter::list_sessions` — `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs:1340` (async) → `list_sessions_sync` at `:166`; roster path `MvpAgent::resident_roster_entry` / `emit_roster_changed` via `session_lifecycle.rs:97-130`; unified list `build_unified_list` — `crates/codegen/xai-grok-shell/src/session/unified_list/mod.rs` | Storage `StorageAdapter::list_sessions` trait — `crates/codegen/xai-grok-shell/src/session/storage/mod.rs:642` | Read-only scan of `summary.json` per session dir (`jsonl/mod.rs:166-188`); no writes | None | `crates/codegen/xai-grok-shell/benches/session_list.rs` (warm bench over 9864 summaries); `crates/codegen/xai-grok-shell/tests/test_xai_session_update.rs` | R1: `project_active_session_row` (`app_server_runtime/mod.rs:137-158`) hardcodes `epoch_1`/revision 0/Dormant — **not** the JSONL summary. C1-D must call `JsonlStorageAdapter::list_sessions` and project `Summary` → protocol `Session`, not the dormant stub. |
| `read_session` | `StorageAdapter::load_session` / `load_session_without_updates` / `load_summary` — `crates/codegen/xai-grok-shell/src/session/storage/mod.rs:628-644`; JSONL impl `jsonl/mod.rs` (`load_summary`/`load_session`); persistence entry `crate::session::persistence::load_light` — `crates/codegen/xai-grok-shell/src/session/persistence.rs:2369` | Storage trait method | Reads `summary.json`, `chat_history.jsonl`, `updates.jsonl`, plan/signals/announcement state | None | `crates/codegen/xai-grok-shell/tests/session_load_perf.rs`; `crates/codegen/xai-grok-shell/src/session/persistence_tests.rs` | R2: `SessionReadResult` carries `turns: Vec<Turn>` + `items: Vec<Item>` (`methods.rs:48-54`), but Shell `Summary`/`PersistedData` has no first-class `Turn`/`Item` projection. C1-D must build the projection from `updates.jsonl` lines (same source as `replay`). **UNVERIFIED** whether a clean `Turn`/`Item` projector exists today — likely new code in the adapter, not a second actor. |
| `start_session` | `MvpAgent::new_session` (ACP handler) — `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs:853`; spawns actor via `spawn_and_register_session` — `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs:2911`; thread spawn `spawn_session_on_thread` — `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs:1662` | ACP `session/new` → builds `SessionInfo`, resolves model/MCP, persists summary, spawns actor | Writes `summary.json` (new session dir); registers in `active_sessions.json` via `active_sessions::register` — `crates/codegen/xai-grok-shell/src/active_sessions.rs:28` | None at create time | `crates/codegen/xai-grok-shell/tests/test_agent_type_invariant.rs`; `app_server_runtime/mod.rs:182-216` `app_server_runtime_registers_one_actor_token_per_session` (Fake-backed) | R3: `new_session` takes `acp::NewSessionRequest` (cwd, mcp_servers, meta), not `SessionStartParams` (workspace_root, agent_type, provider_binding, idempotency_key — `methods.rs:14-19` + tower `lib.rs:48`). Adapter must translate params and honor `idempotency_key` (Fake dedups; Shell `new_session` generates UUIDv7 unless `_meta.sessionId` set at `acp_agent.rs:907-919`). |
| `resume_session` | `MvpAgent::load_session` (ACP handler) — `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs:1239`; reconnect path flushes live actor (`flush_session` — `agent_ops.rs:1950`), drains old thread (`drain_old_session_thread` — `agent_ops.rs:1728`), replays updates (`replay_session_updates` — `mvp_agent/mod.rs:1446`) | ACP `session/load` | Reads `summary.json` + `chat_history.jsonl` + `updates.jsonl`; writes nothing on load itself (replay is forward-only) | None at resume; parked plan-approval re-issued via `SessionCommand::RestorePlanApproval` — `commands.rs:118` | `crates/codegen/xai-grok-shell/tests/test_leader_stdio_integration.rs`; `crates/codegen/xai-grok-shell/tests/test_leader_death_repro.rs`; `tests/test_xai_session_update.rs` | R4: `resume_session` params (`SessionResumeParams{session_id, idempotency_key}` — `methods.rs:33-39`) carry no `cwd`/`mcp_servers`/`meta`. Shell `load_session` requires `cwd` (`acp_agent.rs:1273`). Adapter must resolve cwd from `summary.json` (read first) — extra storage round-trip, or extend params. **UNVERIFIED** which is intended. |
| `fork_session` | `session::fork::fork_session` — `crates/codegen/xai-grok-shell/src/session/fork.rs:66`; uses `JsonlStorageAdapter::copy_session_data` (`storage/mod.rs:678`); request type `ForkSessionRequest` — `fork.rs:13-37` | Free function (no actor); copies `chat_history.jsonl` + `updates.jsonl` + plan state to a new session dir with new UUIDv7 id (`fork.rs:60`) | Writes new session dir + `summary.json` (no actor spawned) | None | `crates/codegen/xai-grok-shell/tests/test_fork_session.rs` | R5: `SessionForkParams{session_id, idempotency_key, workspace_root?}` (`methods.rs:43-50`) has no `target_prompt_index`/`new_model_id`/`source_cwd`. `fork_session` requires `source_cwd` + `new_cwd` (`fork.rs:21-29`). Adapter must read source `summary.json` for cwd and use `workspace_root` as `new_cwd`. Idempotency via `idempotency_key` is **UNVERIFIED** — `fork_session` generates a fresh UUIDv7 each call (no dedup). |
| `archive_session` | `StorageAdapter::delete_session` — `crates/codegen/xai-grok-shell/src/session/storage/mod.rs:647`; JSONL impl `jsonl/mod.rs:1347` (idempotent `remove_dir_all`); also `persistence::delete_session_history` — `persistence.rs:2517`; explicit close `MvpAgent::close_session_explicit` — `session_lifecycle.rs:58` (finalizes cloud replica + `remove_session_terminal(Completed)`) | Storage trait method / ACP `x.ai/session/close` | Deletes entire session dir (permanent) — **destructive** | None | No direct test for `delete_session` JSONL path found in `tests/` (grep: `tests/test_fork_session.rs` covers copy, not delete) | R6: Facade `archive_session` semantically means *archive* (hide, resumable), but the only existing destructive symbol is `delete_session` (irreversible). `close_session_explicit` finalizes the cloud replica but keeps disk. **No archive/hide-only path exists today.** C1-D must either (a) map to `delete_session` and document the destructive semantics, or (b) add a `hidden` flag on `Summary` (new field). **UNVERIFIED** — product decision required; default to `delete_session` is risky (data loss). |

### 1.2 Turn lifecycle

| Facade method | Existing Shell symbol (file:fn) | Message/command type | Persistence touch | Permission/interaction? | Test entrypoints | Risk |
|---|---|---|---|---|---|---|
| `start_turn` | `MvpAgent::prompt` (ACP handler) — `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs:2017`; sends `SessionCommand::Prompt` — `crates/codegen/xai-grok-shell/src/session/commands.rs:113`; actor dispatch `run_loop.rs:281`; foreground exclusivity via per-session `dispatch_lock` — `session_lifecycle.rs:52` (`Rc<tokio::sync::Mutex<()>>`, held across the prompt at `acp_agent.rs:2127-2128`) | ACP `session/prompt` → `SessionCommand::Prompt` (oneshot `PromptTurnResult`) | Appends user msg to `chat_history.jsonl` (persist_ack oneshot — `commands.rs:148-152`); turn writes stream to `updates.jsonl` via persistence actor | None at turn start; permission/question/plan-approval reverse-requests arise *during* the turn | `crates/codegen/xai-grok-shell/tests/test_xai_session_update.rs`; `mvp_agent/tests.rs:2979` `cancel_does_not_forward_to_bridge_in_local_mode`, `:3007` `cancel_never_overtakes_in_flight_prompt_intake`; `app_server_runtime/mod.rs:219-261` `single_actor_owns_turn_mutation` (Fake-backed, 8 concurrent starts) | R7: `TurnStartParams{session_id, input: Vec<InputBlock>, idempotency_key}` (`methods.rs:91-99` + tower `lib.rs:53`) vs `acp::PromptRequest{session_id, prompt: Vec<ContentBlock>, meta}`. `InputBlock` ≠ `ContentBlock` — adapter must convert. `idempotency_key` dedup is **UNVERIFIED** in Shell `prompt` (it dedups via `dispatch_lock` + `send_now`, not by key). |
| `steer_turn` | `SessionCommand::Interject` — `crates/codegen/xai-grok-shell/src/session/commands.rs:669`; actor handler `run_loop.rs:734` (`broadcast_interjection` + push to `pending_interjections` if turn running, else `queue_interjection_fallback_prompt`); also `InterjectQueuedPrompt` — `commands.rs:545` (atomic remove-from-queue + interject) | `SessionCommand::Interject{text, id, images}` (fire-and-forget, no oneshot) | No direct persistence (interjection merged into next turn's user msg on drain) | None | `crates/codegen/xai-grok-shell/src/session/acp_session_tests/interjection_actor_tests.rs`; `interjection_tests.rs` | R8: `TurnSteerParams{session_id, turn_id, input, idempotency_key}` (`methods.rs:74-81`) targets a specific `turn_id` and returns `Item`. Shell `Interject` has no `turn_id` (it targets the running turn implicitly via `current_prompt_id`) and returns no `Item`. Adapter must (a) verify `turn_id` matches `current_prompt_id` (else `turn_not_found`), (b) synthesize/return an `Item` — **UNVERIFIED** what `Item` should represent for a steer. Likely a new adapter-side `Item` envelope, not a second actor. |
| `interrupt_turn` | `MvpAgent::cancel` (ACP handler) — `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs:3094`; sends `SessionCommand::Cancel` — `commands.rs:566`; actor handler `run_loop.rs:420`; foreground lock held (`acp_agent.rs:3126-3127`) | ACP `session/cancel` notification → `SessionCommand::Cancel{cancel_subagents, kill_background_tasks, rewind_if_pristine, trigger}` (fire-and-forget) | Cancel may rewind if pristine (`rewind_if_pristine`) → truncates `chat_history.jsonl` + `updates.jsonl` | None | `mvp_agent/tests.rs:2979,3007`; `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | R9: `TurnInterruptParams{session_id, turn_id, idempotency_key}` (`methods.rs:83-90`) targets a `turn_id`. Shell `Cancel` targets the running turn implicitly. Adapter must verify `turn_id == current_prompt_id` (else `turn_not_found`). `idempotency_key` dedup **UNVERIFIED** in Shell cancel. |

### 1.3 Interaction + replay

| Facade method | Existing Shell symbol (file:fn) | Message/command type | Persistence touch | Permission/interaction? | Test entrypoints | Risk |
|---|---|---|---|---|---|---|
| `respond_interaction` | Reverse-request resolution: leader routes `interaction_resolved` — `crates/codegen/xai-grok-shell/src/leader/server.rs:492` `extract_interaction_resolved_tool_call_id`; pending registry `session::pending_interaction::PendingInteractionGuard` (Drop removes + broadcasts) — `crates/codegen/xai-grok-shell/src/session/pending_interaction.rs:80-145`; kinds `PendingKind::{Permission, Question, PlanApproval}` — `pending_interaction.rs:32-46`; shared interaction routing `leader/server.rs:455` `is_interaction_request` | ACP reverse-request response (`session/request_permission`, `x.ai/ask_user_question`, `x.ai/exit_plan_mode`) — **shared/broadcast, first-answer-wins** | Never persisted (pending interactions are in-memory only — `pending_interaction.rs:1-9`) | **This IS the interaction surface** | `crates/codegen/xai-grok-shell/src/session/pending_interaction.rs:148-230` (guard insert/remove, parked plan-approval, poisoned lock); `leader/server.rs:2800-2870` (interaction detection tests) | R10: `InteractionResponseParams{session_id, turn_id, interaction_id, decision, idempotency_key}` (`methods.rs:82-101`) uses `interaction_id`. Shell keys pending by `tool_call_id` (`pending_interaction.rs:28`). Adapter must map `interaction_id` → `tool_call_id`. The actual resolution mechanism (who completes the parked oneshot) lives in the ACP client/leader forwarding layer, **not** in a `SessionCommand`. **UNVERIFIED** how the adapter delivers the decision back to the parked future without reusing the leader's ACP response path — likely needs a new `SessionCommand::ResolveInteraction` or a direct channel. C1-D must not invent a second permission engine. |
| `replay` | `MvpAgent::replay_session_updates` — `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs:1446` (reads `updates.jsonl`, filters by cursor via `session::storage::prepare_replay_lines`); delta path `replay_session_updates_from_offset_enqueue` — `mvp_agent/mod.rs:1541`; replay buffer `session::replay_events::SessionNotification` — `replay_events.rs:14`; bounded page helper `xai_grok_app_server::replay::replay_all_pages` — `crates/codegen/xai-grok-app-server/src/replay.rs:7` | Reads `updates.jsonl` from disk + live replay buffer; `SubscribeParams{session_id, after_event_seq, history_epoch}` (`app_server_protocol`) | Read-only disk scan of `updates.jsonl` (`mvp_agent/mod.rs:1462-1469`) | None | `crates/codegen/xai-grok-shell/tests/trace_replay.rs`; `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs`; `app_server/src/replay.rs:40` (cursor semantics) | R11: `replay` returns `ReplayPage{events: Vec<RuntimeEvent>, replayed_through, next_cursor}` (tower `lib.rs:66-87`). Shell `replay_session_updates` returns `(last_tokens, end_offset, unfinished_subagents)` and forwards raw `updates.jsonl` lines over the ACP gateway — no `RuntimeEvent` projection exists. Adapter must (a) parse each `updates.jsonl` line into one of `RuntimeEvent::{SessionChanged, TurnChanged, ItemStarted, ItemDelta, ItemCompleted, InteractionRequested}` (tower `lib.rs:74-86`), (b) implement cursor pagination over `after_event_seq`/`WireCounter`. **UNVERIFIED** projection exists today — this is the largest new code surface in C1-D, but it is a *projection*, not a second actor. |

## 2. One SessionActor per loaded Session (enforcement today)

- **One dedicated OS thread + `LocalSet` per session.**
  `spawn_session_on_thread` — `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs:1662`
  constructs the `!Send` `SessionActor` inside the thread and sends a
  `SessionHandle` back via oneshot (`spawn.rs:1648-1656`).
- **`MvpAgent::sessions: RefCell<HashMap<SessionId, SessionHandle>>`** is the
  resident registry; `spawn_and_register_session` (`agent_ops.rs:2911`) inserts
  after spawn. `remove_session` (`session_lifecycle.rs:33`) evicts.
- **`SessionThread`** (`spawn.rs:1632`) holds the `JoinHandle` separately
  (not `Clone`); stored in `session_threads`. `drain_old_session_thread`
  (`agent_ops.rs:1728`) waits for the old actor to flush before reload — this
  is the mechanism that prevents two actors for one session on reconnect.
- **Foreground turn exclusivity** = per-session `dispatch_lock`:
  `session_lifecycle.rs:52` `dispatch_lock` returns
  `Rc<tokio::sync::Mutex<()>>`; `prompt` acquires it at `acp_agent.rs:2127`
  and `cancel` at `acp_agent.rs:3126`. This serializes prompt/cancel per
  session — the actor mailbox itself is single-threaded so this is belt-and-
  suspenders for the leader-mode multi-client case.
- **Idle-unload no-evict keystone**: `session_has_live_work`
  (`session_lifecycle.rs:384`) checks `current_prompt_id` (sync) +
  parked plan-approval (`pending_interaction::has_parked_plan_approval`)
  + `SessionHandle::is_busy` (`handle.rs:232`) before any unload.
- **Tower-side registry is opaque tokens only**:
  `ShellRuntimeAdapter::registry` (`app_server_runtime/mod.rs:38`) calls
  `SessionRegistry::get_or_insert_with` per session id — a `u64` token, not a
  state machine. `app_server_runtime_registers_one_actor_token_per_session`
  (`app_server_runtime/mod.rs:182`) proves one token per session id. **This
  registry does NOT enforce single-actor; the Shell `sessions` map +
  `drain_old_session_thread` do.** C1-D must keep the Tower registry opaque.

## 3. Foreground turn exclusivity / interrupt / steer hooks

- **Steer** = `SessionCommand::Interject` (`commands.rs:669`); handler at
  `run_loop.rs:734`: if `current_prompt_id.is_some()` → push to
  `pending_interjections` (`InterjectionBuffer`, drained at safe points in
  `process_conversation_turn`); else `queue_interjection_fallback_prompt` +
  `maybe_start_running_task`. `InterjectQueuedPrompt` (`commands.rs:545`,
  handler `prompt_queue.rs:484`) atomically promotes a queued prompt to an
  interjection (single mailbox op — prevents double-run).
- **Interrupt** = `SessionCommand::Cancel` (`commands.rs:566`); handler at
  `run_loop.rs:420`. `cancel_subagents` / `kill_background_tasks` /
  `rewind_if_pristine` / `trigger` fields. `dispatch_lock` held in
  `MvpAgent::cancel` (`acp_agent.rs:3126`).
- **Foreground exclusivity** = `dispatch_lock` per session
  (`session_lifecycle.rs:52`), held across `prompt` (`acp_agent.rs:2127`)
  and `cancel` (`acp_agent.rs:3126`). The actor's single-threaded mailbox is
  the primary serialization; the lock bounds leader-mode cross-client races.

## 4. Composition root injection point

- **Canonical injection site**: `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs:12`
  `experimental_app_server_processor()` — currently
  `ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()))`.
- C1-D must replace the `FakeRuntime` argument with a real Shell-owned
  `GrokRuntimeFacade` impl that forwards to the symbols in §1. The
  `ShellRuntimeAdapter` wrapper (`app_server_runtime/mod.rs:33`) already
  records Tower registry tokens and may stay as the outer wrapper; the **inner**
  `Arc<dyn GrokRuntimeFacade>` is what must become real.
- Tower instance selection: `select_tower_instance_id`
  (`app_server_composition.rs:19`) — explicit > `GROK_TOWER_INSTANCE` env >
  `"default"`. **Note (M-1 from C0-C review)**: global AGENTS.md / GOBLIN.md
  uses `GROK_OSS_TOWER` as the canonical env name; `GROK_TOWER_INSTANCE` here
  may need alignment. Flag for C1-D.

## 5. What must NOT be reinvented

- **No second `SessionActor`.** Tower's
  `leader_characterization_tower_has_no_second_actor_type`
  (`xai-grok-tower/src/lib.rs:91`) guards this. The real actor is
  `session/acp_session.rs:564`.
- **No hybrid Fake+JSONL authority.** `FakeRuntime` mutations + real
  `JsonlStorageAdapter::list_sessions` reads in one facade = split authority
  (forbidden by corrective contract §2 / F-01). The real adapter must own
  both read and write paths consistently.
- **No second permission/elicitation engine.** `PendingInteractionGuard`
  (`pending_interaction.rs:80`) + the leader's shared interaction routing
  (`leader/server.rs:455`) are the only surface. `respond_interaction` must
  resolve into the existing parked oneshot, not re-evaluate permissions.
- **No second replay buffer.** `session::replay_events::SessionNotification`
  + `MvpAgent::replay_session_updates` over `updates.jsonl` are the source.
  `replay` must project these into `RuntimeEvent`, not buffer again.
- **No second turn state machine.** `SessionCommand::Prompt`/`Cancel`/
  `Interject` + `dispatch_lock` + `current_prompt_id` are the turn authority.
  `start_turn`/`steer_turn`/`interrupt_turn` must map onto these, not
  introduce a parallel `Turn` state machine in Tower.

## 6. Existing tests that already touch each path

| Path | Test file:fn |
|---|---|
| list_sessions (storage) | `crates/codegen/xai-grok-shell/benches/session_list.rs` (warm bench); `active_sessions.rs:218` `register_is_idempotent`, `:236` `collect_crashed_partitions_by_pid_liveness` |
| read_session (load) | `crates/codegen/xai-grok-shell/tests/session_load_perf.rs`; `session/persistence_tests.rs`; `mvp_agent/mod.rs:871-917` `read_session_or_init_meta_str_*` |
| start_session (new) | `tests/test_agent_type_invariant.rs`; `app_server_runtime/mod.rs:182` `app_server_runtime_registers_one_actor_token_per_session`; `app_server_runtime/mod.rs:265` `app_server_multi_workspace_stable_session_ids` |
| resume_session (load) | `tests/test_leader_stdio_integration.rs`; `tests/test_leader_death_repro.rs`; `tests/test_xai_session_update.rs`; `acp_session_tests/reverse_request_session_id_tests.rs` |
| fork_session | `tests/test_fork_session.rs` |
| archive_session (delete) | **NONE** — gap. `delete_session` JSONL path has no direct test (grep finds no `tests/` reference). |
| start_turn (prompt) | `tests/test_xai_session_update.rs`; `mvp_agent/tests.rs:2979` `cancel_does_not_forward_to_bridge_in_local_mode`, `:3007` `cancel_never_overtakes_in_flight_prompt_intake`; `app_server_runtime/mod.rs:219` `single_actor_owns_turn_mutation` |
| steer_turn (interject) | `acp_session_tests/interjection_actor_tests.rs`; `interjection_tests.rs`; `acp_session_tests/prompt_queue_actor_tests.rs` |
| interrupt_turn (cancel) | `acp_session_tests/cancel_running_task_tests.rs`; `mvp_agent/tests.rs:2979,3007` |
| respond_interaction | `session/pending_interaction.rs:148-230` (guard unit tests); `leader/server.rs:2800-2870` (interaction detection) |
| replay | `tests/trace_replay.rs`; `acp_session_tests/replay_buffer_send_update_tests.rs`; `app-server/src/replay.rs:40` (cursor semantics) |

## 7. Smallest C1 implementation slice (3–5 steps)

1. **RED tests first** (one per facade method, §8) under
   `crates/codegen/xai-grok-shell/tests/c1/` against a real
   `JsonlStorageAdapter` fixture (TempDir) + a real spawned `SessionActor`.
   Run via `./scripts/run-rust-test-gate.sh`. All RED initially.
2. **Build the real inner port** (`ShellSessionActorRuntime: GrokRuntimeFacade`)
   in `app_server_runtime/` that owns a `MvpAgent`-like harness (or a thin
   facade over an existing `MvpAgent` test fixture) and forwards each method
   per §1. Reuse `JsonlStorageAdapter` for list/read/archive,
   `spawn_session_on_thread` for start/resume, `fork::fork_session` for fork,
   `SessionCommand::{Prompt, Interject, Cancel}` for turns, the pending-
   interaction registry for `respond_interaction`, and
   `replay_session_updates` + a new `RuntimeEvent` projector for `replay`.
3. **Switch the composition root** at `app_server_composition.rs:12` from
   `FakeRuntime` to the real port. Keep `FakeRuntime` for unit/conformance
   only (Tower tests, processor tests that don't need the actor).
4. **GREEN the RED tests** with real-adapter evidence; capture logs under
   `tests/c1/`. Do not mark epic PASS without real-adapter GREEN.
5. **Honesty gate**: document remaining gaps (R2 Turn/Item projection, R6
   archive semantics, R10 interaction resolution channel, R11 replay
   projection) in `waves/c1-shell-port.md` and update corrective STATUS.

## 8. Top 5 integration risks

1. **R11 / replay projection is the largest new surface.** No existing
   `updates.jsonl` line → `RuntimeEvent` projector exists. C1-D must build one
   without introducing a second replay buffer. Highest risk of accidental
   second-engineering.
2. **R10 / `respond_interaction` has no `SessionCommand` today.** The parked
   oneshot is resolved via the leader's ACP response forwarding, not a session
   command. The adapter needs a new resolution channel that reuses
   `PendingInteractionGuard`'s registry without re-evaluating permission
   policy — easy to accidentally build a second permission engine.
3. **R6 / `archive_session` semantics undefined.** Only `delete_session`
   (irreversible) and `close_session_explicit` (keeps disk) exist. Mapping
   `archive` → `delete` is data loss; adding a `hidden` flag is a schema
   change to `Summary`. Product decision required before C1-D.
4. **R2 / `read_session` Turn/Item projection.** `SessionReadResult` needs
   `Vec<Turn>` + `Vec<Item>`, which Shell does not have first-class. Must be
   projected from `updates.jsonl` — same parser risk as R11, but synchronous
   and paginated.
5. **R3 + R7 / param translation + idempotency.** `SessionStartParams`/
   `TurnStartParams`/`TurnSteerParams`/`TurnInterruptParams` carry
   `idempotency_key`, but Shell `new_session`/`prompt`/`Interject`/`Cancel`
   dedup via different mechanisms (`_meta.sessionId`, `dispatch_lock`,
   `send_now`). The adapter must implement idempotency-key dedup without
   weakening existing exclusivity. `InputBlock` ≠ `ContentBlock` conversion
   is a concrete wire-shape risk.

## 9. Recommended C1 RED test names (names only)

1. `c1_real_adapter_list_sessions_reads_jsonl_summaries_not_dormant_stub`
2. `c1_real_adapter_read_session_projects_turns_and_items_from_updates_jsonl`
3. `c1_real_adapter_start_session_spawns_actor_and_persists_summary`
4. `c1_real_adapter_start_session_idempotency_key_dedups_same_session_id`
5. `c1_real_adapter_resume_session_drains_old_thread_before_replay`
6. `c1_real_adapter_fork_session_copies_history_to_new_cwd`
7. `c1_real_adapter_archive_session_semantics_match_product_decision`
8. `c1_real_adapter_start_turn_acquires_dispatch_lock_and_runs_prompt`
9. `c1_real_adapter_start_turn_idempotency_key_dedups_concurrent_starts`
10. `c1_real_adapter_steer_turn_targets_running_turn_and_returns_item`
11. `c1_real_adapter_steer_turn_turn_id_mismatch_returns_turn_not_found`
12. `c1_real_adapter_interrupt_turn_cancels_running_turn_only`
13. `c1_real_adapter_interrupt_turn_turn_id_mismatch_returns_turn_not_found`
14. `c1_real_adapter_respond_interaction_resolves_parked_pending_interaction`
15. `c1_real_adapter_respond_interaction_unknown_interaction_id_returns_not_found`
16. `c1_real_adapter_replay_projects_updates_jsonl_into_runtime_events`
17. `c1_real_adapter_replay_cursor_pagination_advances_after_event_seq`
18. `c1_real_adapter_one_actor_per_session_no_second_actor_on_reconnect`
19. `c1_real_adapter_no_hybrid_authority_real_list_with_fake_mutation_rejected`
20. `c1_composition_root_injects_real_port_not_fake_runtime`

## 10. Report-back summary

- **Full map**: §1 (11 facade methods → real Shell symbols with `file:fn`).
- **Top 5 risks**: §8 (replay projection, interaction resolution channel,
  archive semantics, read Turn/Item projection, param/idempotency translation).
- **Recommended C1 RED tests**: §9 (20 names).
- **UNVERIFIED markers**: R2 (Turn/Item projector), R4 (resume cwd source),
  R5 (fork idempotency), R6 (archive semantics — product decision),
  R7 (idempotency-key dedup in Shell), R8 (steer `Item` shape),
  R9 (interrupt idempotency), R10 (interaction resolution channel),
  R11 (replay `RuntimeEvent` projection). These are the C1-D implementer's
  required design decisions, not second-actor inventions.
- **No production code changed.** Read-only exploration only.
