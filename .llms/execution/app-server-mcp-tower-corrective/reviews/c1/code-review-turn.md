# C1-H Independent Code Review — Turn Lifecycle (GLM `glm-5.2`)

| Field | Value |
|---|---|
| Wave | C1-G (turn lifecycle wiring via `SessionHandle` channels) |
| Review mode | read-only (no implementation) |
| Reviewer | GLM `glm-5.2` |
| Date | 2026-07-18 |
| Handoff | `handoffs/HANDOFF-C1-H-code-review.md` |
| Implementer handoff | `handoffs/HANDOFF-C1-G-turn-lifecycle.md` |
| Implementer wave note | `waves/c1-turn-lifecycle.md` |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

No Critical or High finding. The implementation satisfies the corrective
contract non-negotiables (single `SessionActor` authority, no Fake hybrid,
Tower≠Shell, no second turn state machine), the tests are non-vacuous, the
RED→GREEN cycle is real, and the production PARTIAL status is honest. The
findings below are Medium/Low fidelity or honesty gaps that do not block C1-G
acceptance; most are explicitly deferred PARTIAL items already enumerated in
the wave note.

## Severity table

| ID | Severity | Confidence | Summary |
|---|---|---|---|
| F-1 | Medium | High | `steer_turn` synthesized `Item.event_seq` is a wall-clock timestamp (`now_ms()`), not a monotonic sequence — semantically wrong wire shape, inconsistent with the replay projector. |
| F-2 | Medium | Medium | `next_ordinal` resets to 1 on runtime recreation (process restart); the adapter does not seed from `Summary.num_messages`. Ordinal collisions possible across restarts. Not called out in the wave note. |
| F-3 | Medium | Medium | Fire-and-forget `steer_turn`/`interrupt_turn` return `Ok`/synthesized `Item` after `cmd_tx.send` succeeds with no liveness proof. A panicked actor leaves `current_prompt_id` stale; the guard would pass on a dead actor. Matches Shell fire-and-forget semantics but the adapter adds no reap/liveness check (and omits the `SessionThread` reaping the handoff recommended). |
| F-4 | Medium | High | `ensure_resident` has a TOCTOU: concurrent calls for the same session can double-spawn; the second spawned consumer task leaks (its `cmd_rx` is never consumed). `or_insert` prevents map corruption but the leaked task is never reaped. |
| F-5 | Medium | High | Test fixture (`TestActorSpawner`/`HeldTurnSpawner`) does NOT replicate the real actor's `dispatch_lock`, `parse_prompt`, or `pending_interjections` drain. The wave note claims `c1_turn_concurrent_starts_serialize_through_single_mailbox` "mirrors the real actor's `dispatch_lock` + single-threaded mailbox" — only the mailbox ordering is replicated, not the `dispatch_lock`. Minor honesty gap. |
| F-6 | Low | High | `interrupt_turn` hardcodes `rewind_if_pristine: false`. The real `MvpAgent::cancel` uses `rewind_if_pristine: true` on some paths. The adapter silently drops the rewind capability. Safe (no data loss) but not faithful to C0-B §1.2. |
| F-7 | Low | High | `start_turn` hardcodes `send_now: false`, `verbatim: false`, `prompt_mode: PromptMode::default()`, `persist_ack: None`, `parsed_prompt_tx: None`. The real actor receives these from client meta. Acceptable for the minimal path but diverges from production. |
| F-8 | Low | High | `InputBlock::Mention`/`Skill` flatten to `ContentBlock::Text` with only `name`, dropping `path`. Acceptable per handoff ("faithful wire shape") but loses structured identity the real `parse_prompt` would render. |
| F-9 | Low | High | `Turn.revision` and synthesized `Item.revision` hardcoded to `WireCounter::new(1)`. The real actor increments per item. PARTIAL projection. |
| F-10 | Low | Medium | Test fixture persists only `AgentMessageChunk` to `updates.jsonl`; it does not append the user message to `chat_history.jsonl`. The persistence assertion (`!loaded.updates.is_empty()`) proves the agent reply side effect, not the user-message persistence. The real actor persists both. Test-fidelity gap, not a production bug. |
| F-11 | Low | High | C1-D tests `c1_real_adapter_{start_turn,steer_turn,interrupt_turn}_returns_unsupported_actor_gap` still pass because they use `ShellSessionActorRuntime::new` (production spawner → no resident → `unsupported`). The test names now overpromise: they assert `unsupported` which only holds for the no-resident path, not as a blanket "actor gap". No regression; minor naming honesty gap. |

## Checklist answers (from handoff)

1. **Does every new path still use a single SessionActor authority?**
   YES. `ResidentHandle` (`shell_session_actor_runtime.rs:72-107`) holds only
   `cmd_tx: mpsc::UnboundedSender<SessionCommand>` and
   `current_prompt_id: Arc<Mutex<Option<String>>>` — the `Send`-able projection
   of `SessionHandle` (`session/handle.rs:38-164`). It is NOT a
   `SessionActor`: no `JoinHandle`, no `LocalSet`, no actor state, no
   `dispatch_lock`. The static guard
   `shell_session_actor_runtime_defines_no_session_actor` (lines 815-823)
   splits on `#[cfg(test)]` and asserts the production section contains no
   `struct SessionActor`/`enum SessionActor`. The only real `SessionActor`
   remains `session/acp_session.rs:564` (per C0-B). NOTE: the handoff
   recommended a `SessionThread` (`JoinHandle`) for reaping — the implementer
   omitted it. That is a reaping gap (F-3), not a second actor.

2. **Are `SessionCommand::{Prompt,Interject,Cancel}` used correctly?**
   YES, with fidelity caveats (F-6/F-7/F-8).
   - `start_turn` (lines 598-665) constructs `SessionCommand::Prompt` with a
     fresh `prompt_id` (UUIDv7), `prompt_blocks` from
     `input_blocks_to_content_blocks`, a oneshot `respond_to`, and awaits
     `PromptTurnResult`. `PromptTurnOk.completion_kind` → `TurnStatus` mapping
     is correct: `Completed→Completed`, `Cancelled→Interrupted`,
     `Rewound→Interrupted`, `MaxTurnsReached→Failed`,
     `RemovedFromQueue→Declined`, `Err→Failed`. Matches C0-B §1.2.
   - `steer_turn` (lines 666-714) sends `SessionCommand::Interject { text, id,
     images }` with `id = Some(params.idempotency_key)`. `turn_id` is verified
     against `current_prompt_id` (mismatch → `turn_not_found`). Synthesizes an
     `ItemBody::AgentMessage` envelope because Shell `Interject` is
     fire-and-forget. Matches C0-B §1.2 R8.
   - `interrupt_turn` (lines 716-745) sends `SessionCommand::Cancel` with
     `cancel_subagents: true, kill_background_tasks: false,
     rewind_if_pristine: false, trigger: Some("interrupt_turn")`. `turn_id`
     verified against `current_prompt_id`. Matches C0-B §1.2 R9, except
     `rewind_if_pristine` is hardcoded `false` (F-6).

3. **Any Send/Sync / await-across-lock hazards?**
   NO blocking hazard found.
   - `ensure_resident` (lines 224-256) drops the `residents` MutexGuard before
     awaiting `spawner.spawn` (the `contains_key` check is in a nested block
     that releases at `}`). The re-lock after `spawn` is a separate critical
     section. Send-safe.
   - `start_session` (lines 485-535) drops the `idempotency` MutexGuard before
     any await (lines 488-490). Send-safe.
   - `resident()` (lines 262-272) locks, clones `cmd_tx` + `current_prompt_id`
     Arc, returns — no await. `current_turn()` (lines 96-99) uses
     `.lock().ok()` to handle poison. `next_ordinal` (lines 276-281) locks,
     `fetch_add`, returns — no await.
   - `start_turn`/`steer_turn`/`interrupt_turn` call `resident()` (no await),
     then `cmd_tx.send` (no await), then await oneshot `rx` (start_turn only).
     No await-across-lock.
   - `ResidentHandle` is `Send + Sync` (`mpsc::UnboundedSender` + `Arc<Mutex<_>>`).
     `Resident` adds `AtomicU64` — still `Send + Sync`. `ShellSessionActorRuntime`
     is `Send + Sync`. `SessionSpawner: Send + Sync`. All consistent.
   - F-4 (TOCTOU in `ensure_resident`) is a correctness race, not a
     Send/Sync hazard.

4. **Any silent data loss (archive, delete, truncate)?**
   NO silent data loss.
   - `archive_session` (lines 586-591) returns `unsupported` — no `delete_session`
     call, no truncation. Honest R6 stub.
   - `interrupt_turn` hardcodes `rewind_if_pristine: false` (F-6) — the adapter
     never triggers the rewind/truncate path. This is the safe (no-data-loss)
     choice, even though it diverges from the real actor.
   - `steer_turn` synthesizes an `Item` but does NOT persist it to
     `updates.jsonl`. The real Shell `Interject` is fire-and-forget and merges
     into the next turn's user message on drain; the adapter does not replicate
     that persistence. No silent loss — the interjection text is delivered to
     the actor via `cmd_tx`, and the synthesized `Item` is a protocol envelope
     only.
   - `start_turn` does not persist the user message itself (the real actor
     does via `persist_ack`). The adapter sends the command; persistence is the
     actor's responsibility. No loss in production; the test fixture does not
     replicate user-message persistence (F-10).

5. **Tests non-vacuous? Prove real routing not string-matching alone?**
   NON-VACUOUS. The tests exercise the real `SessionCommand` routing path:
   - `c1_turn_start_turn_routes_prompt_through_real_cmd_tx_and_persists`
     (lines 144-171) sends a real `SessionCommand::Prompt` through `cmd_tx`;
     the `TestActorSpawner` consumer (lines 41-117) matches on the real
     `SessionCommand::Prompt` variant, sets `current_prompt_id`, appends a real
     `AgentMessageChunk` to `updates.jsonl` via the real
     `JsonlStorageAdapter::append_update`, resolves the oneshot with
     `PromptTurnOk { completion_kind: Completed }`, then clears the slot. The
     test asserts `!loaded.updates.is_empty()` — a real disk side effect
     through the real command path, not string-matching.
   - `c1_turn_steer_turn_against_running_turn_returns_item` (lines 332-396)
     uses `HeldTurnSpawner` (lines 252-330) which holds the turn running until
     a `Cancel` arrives, then polls `current_prompt_id` until `Some`, steers
     with the matching `turn_id`, asserts the returned `Item.turn_id` matches
     the running id, then interrupts and asserts the start_turn future
     resolves. This proves the turn-id guard and the steer path against a
     live running turn, not a string match.
   - `c1_turn_interrupt_turn_cancels_running_turn_only` (lines 398-443)
     proves the cancel path releases the held turn and the start_turn future
     resolves with the same `turn_id`.
   - `c1_turn_*_turn_id_mismatch_returns_turn_not_found` (lines 232-254,
     444-455) prove the turn-id guard rejects mismatches with `turn_not_found`.
   - `c1_turn_concurrent_starts_serialize_through_single_mailbox`
     (lines 458-491) proves two concurrent `start_turn`s both complete with
     distinct turn ids through the single consumer mailbox. NOTE (F-5): this
     proves mpsc mailbox ordering, not the real `dispatch_lock`.
   - `c1_turn_resume_re_residents_actor_and_routes_turn` (lines 494-519)
     proves `resume_session` re-residents (via `ensure_resident`) and a
     subsequent turn routes.
   - `c1_turn_start_turn_without_resident_returns_unsupported` (lines 176-191)
     proves the production path (default `ProductionSpawner`) returns
     `unsupported` honestly.
   - RED log (`c1_turn_lifecycle_RED.log`): 8/9 fail when the methods are
     stubbed back to `unsupported`/`"RED stub"`; the no-resident test still
     passes. Confirms the tests exercise the real routing path (a vacuous test
     would not flip on the implementation).
   - GREEN log (`c1_turn_lifecycle_GREEN.log`, `..._GREEN_gate.log`): 9/9 pass.
   - The consumer is a real `cmd_tx` consumer processing the real
     `SessionCommand` enum, NOT `FakeRuntime`. This satisfies handoff AC #3
     ("equivalent real `cmd_tx` consumer that is not FakeRuntime").

6. **Residual PARTIAL claims honest?**
   YES. The wave note `waves/c1-turn-lifecycle.md` §3 enumerates REAL vs
   PARTIAL honestly:
   - REAL: command routing, persistence through the command path, turn-id
     guard, foreground serialization (caveat F-5), resume re-resident,
     honest `unsupported` when no resident, invariant guards.
   - PARTIAL: production actor spawn (`ProductionSpawner` returns
     `unsupported`), R7 idempotency-key dedup, `InputBlock`→`ContentBlock`
     minimal conversion, `steer_turn` `Item` shape, R4 resume drain/replay,
     `respond_interaction` (R10), `archive_session` (R6).
   - STATUS.md and CHANGES.md are updated with the C1-G row and the honest
     "command-routing REAL; production spawn PARTIAL" framing.
   - F-2 (ordinal reset across restarts) is the one PARTIAL gap NOT
     explicitly called out in the wave note — see finding.

## Independent static verification (re-checked, not trusted from implementer)

1. **No second `SessionActor`.** grep `struct SessionActor`/`enum SessionActor`
   in `shell_session_actor_runtime.rs` → only inside `#[cfg(test)]` (the guard
   at lines 815-823). `ResidentHandle` (lines 72-107) holds only `cmd_tx` +
   `current_prompt_id`. No `JoinHandle`, no `LocalSet`, no actor state. PASS.
2. **No Fake hybrid.** grep `FakeRuntime` in the production section (pre-
   `#[cfg(test)]`) → only doc-comment mentions (lines 5, 12, 77). No
   `FakeRuntime::new`, no `use xai_grok_tower::FakeRuntime`, no
   `: FakeRuntime`. The static guard
   `shell_session_actor_runtime_does_not_use_fake_runtime` (lines 825-835)
   asserts the same. The test consumer uses `JsonlStorageAdapter`, not
   `FakeRuntime`. PASS.
3. **Tower≠Shell.** Unchanged from C1-D — Tower still does not import Shell
   (guard in `mod.rs:147-156`). C1-G adds no Tower edits. PASS.
4. **No second turn state machine.** Turn state (`current_prompt_id`,
   `completion_kind → TurnStatus`) is read from the real actor's shared slot
   and the real `PromptTurnResult`. The adapter introduces no parallel `Turn`
   state machine. PASS.
5. **`SessionHandle` is `Clone + Send`; actor is `!Send`.** `ResidentHandle`
   is built from the `Send`-able subset (`cmd_tx`, `current_prompt_id`) via
   `ResidentHandle::from_handle` (lines 99-104) or directly from a channel
   for tests. The actor never moves across threads. PASS.
6. **Mapping fidelity to C0-B §1.2.** `start_turn`→`SessionCommand::Prompt`
   (oneshot), `steer_turn`→`SessionCommand::Interject` (fire-and-forget +
   turn_id guard + synthesized `Item`), `interrupt_turn`→`SessionCommand::Cancel`
   (fire-and-forget + turn_id guard). All three map correctly. The only
   fidelity divergence is `rewind_if_pristine: false` hardcoded in
   `interrupt_turn` (F-6).
7. **No regression.** `c1_shell_port.txt` shows 18/18 `c1_real_adapter_*`
   pass, including `c1_real_adapter_start_turn_returns_unsupported_actor_gap`
   (line 62), `..._steer_turn_...` (line 54), `..._interrupt_turn_...` (line 54).
   These still pass because `real_port()` uses `ShellSessionActorRuntime::new`
   (production spawner → no resident → `unsupported`). The C1-G change is
   behavior-preserving for the production path.

## Findings (detail)

### F-1 — Medium — `steer_turn` `Item.event_seq` is a wall-clock timestamp
`shell_session_actor_runtime.rs:704`: `event_seq: WireCounter::new(now)` where
`now = now_ms()` (line 703). `WireCounter` is an opaque wire counter used as a
monotonic event sequence for replay pagination (`after_event_seq` in
`SubscribeParams`). The replay projector uses sequential `event_seq =
WireCounter::new(seq)` (line 779). The synthesized `Item` from `steer_turn`
uses a wall-clock timestamp, which is semantically wrong and inconsistent
with the replay ordering. The synthesized `Item` is returned directly to the
caller and is NOT appended to `updates.jsonl`, so it does not corrupt replay
pagination — but the wire shape is wrong and a client that orders Items by
`event_seq` would interleave this Item incorrectly.

**No fix required for C1-G.** The synthesized `Item` is an acknowledged
PARTIAL (R8 shape pending product decision). Flag for the steer `Item` shape
follow-on: use a per-session monotonic sequence, not `now_ms()`.

### F-2 — Medium — `next_ordinal` resets across process restarts
`shell_session_actor_runtime.rs:276-281`: `next_ordinal` allocates from a
per-resident `AtomicU64` starting at 1. The `residents` map is in-memory only;
on runtime recreation (process restart), a fresh resident starts at ordinal 1
even though the disk already has turns from the previous process. The real
actor persists turn count in `Summary.num_messages`. The adapter does not seed
`next_ordinal` from `Summary.num_messages`. Ordinal collisions possible across
restarts.

The wave note §3 does not call this out. The other PARTIAL items (R7, R8, R4)
are enumerated; ordinal seeding is not.

**No fix required for C1-G.** Document as a known PARTIAL limitation in the
wave note. Flag for the production-spawn follow-on: seed `next_ordinal` from
`Summary.num_messages` (or the real actor's turn count) when re-residenting.

### F-3 — Medium — Fire-and-forget commands have no liveness proof
`steer_turn` (lines 666-714) and `interrupt_turn` (lines 716-745) return
`Ok(())`/synthesized `Item` after `cmd_tx.send` succeeds. `mpsc::UnboundedSender::send`
succeeds as long as the receiver has not been dropped — it does NOT prove the
actor processed the command. If the actor panicked, `current_prompt_id` may
remain `Some(turn_id)` (the actor never cleared it), and the adapter's
`current_turn()` guard would pass on a dead actor. The send succeeds, the
adapter returns success, but nobody processes the command.

This matches real Shell semantics (`Interject`/`Cancel` are fire-and-forget
in `run_loop.rs:734`/`:420`), so it is not a new hazard. But the handoff
recommended a `SessionThread` (`JoinHandle`) for reaping a panicked actor;
the implementer omitted it. Without reaping, a dead actor's stale
`current_prompt_id` is never cleared, and the adapter has no way to detect
the actor is gone (the `cmd_tx` send succeeds until the receiver task exits,
which may lag the panic).

**No fix required for C1-G.** The behavior matches Shell. Flag for the
production-spawn follow-on: add the `SessionThread`/`JoinHandle` reaping the
handoff recommended, and clear `current_prompt_id` on actor death.

### F-4 — Medium — `ensure_resident` TOCTOU can leak a spawned consumer task
`shell_session_actor_runtime.rs:224-256`: the fast-path checks
`guard.contains_key(&info.id.0.to_string())` in a scoped block, drops the
guard, awaits `spawner.spawn`, then re-locks and uses
`guard.entry(...).or_insert(...)`. Two concurrent `ensure_resident` calls for
the same session can both observe "not resident", both spawn, and the second
`or_insert` is a no-op (the first wins). The second spawned `ResidentHandle`
is dropped — its `cmd_rx` consumer task (`tokio::spawn` in the test spawner)
leaks with no one to consume its channel.

`start_turn`/`steer_turn`/`interrupt_turn` do NOT call `ensure_resident` (they
only look up existing residents), so the race only occurs between concurrent
`start_session`/`resume_session` for the same session — an unusual pattern.
The real actor's `dispatch_lock` would serialize this in production; the
adapter does not replicate that lock.

**No fix required for C1-G.** The race window is narrow and the consequence is
a leaked test task (not map corruption). Flag for the production-spawn
follow-on: hold the `residents` lock across the spawn (the spawner is
`Send+Sync`), or use a `DashMap::entry`-style atomic insert.

### F-5 — Medium — Test fixture does not replicate `dispatch_lock`
`TestActorSpawner` (lines 41-117) and `HeldTurnSpawner` (lines 252-330) process
the `mpsc::UnboundedReceiver` sequentially in a single `tokio::spawn` task.
This replicates the actor's single-threaded mailbox ordering but NOT the real
actor's `dispatch_lock` (per-session `Rc<tokio::sync::Mutex<()>>` held across
the prompt at `acp_agent.rs:2127-2128`, per C0-B §1.2). The wave note §3 claims
`c1_turn_concurrent_starts_serialize_through_single_mailbox` "mirrors the real
actor's `dispatch_lock` + single-threaded mailbox" — only the mailbox
ordering is mirrored, not the `dispatch_lock` foreground exclusivity.

The test still proves the two concurrent `start_turn`s both complete with
distinct turn ids through the single consumer mailbox — that is a real
property. The honesty gap is in the wave note's framing, not the test.

**No fix required for C1-G.** Update the wave note to say "mirrors the
single-threaded mailbox ordering; `dispatch_lock` foreground exclusivity is
not replicated in the test fixture" for accuracy.

### F-6 — Low — `interrupt_turn` hardcodes `rewind_if_pristine: false`
`shell_session_actor_runtime.rs:730-733`: `Cancel { cancel_subagents: true,
kill_background_tasks: false, rewind_if_pristine: false, trigger:
Some("interrupt_turn") }`. The real `MvpAgent::cancel` (`acp_agent.rs:3094`)
sets `rewind_if_pristine` based on the request. The adapter hardcodes `false`,
meaning it NEVER triggers the rewind-if-pristine path even when the real
actor would. This is the safe (no-data-loss) choice — `rewind_if_pristine`
truncates `chat_history.jsonl` + `updates.jsonl`. But it is a silent
behavioral divergence from C0-B §1.2.

The protocol `TurnInterruptParams` carries no `rewind_if_pristine` field, so
the adapter cannot faithfully forward it. `false` is the conservative default.

**No fix required.** Safe choice. Flag for the R9 follow-on: if the protocol
gains a `rewind_if_pristine` field, forward it; otherwise document the
hardcoded default.

### F-7 — Low — `start_turn` hardcodes prompt meta
`shell_session_actor_runtime.rs:612-622`: `prompt_mode: PromptMode::default()`,
`artifact_upload_ctx: None`, `client_identifier: None`, `screen_mode: None`,
`verbatim: false`, `traceparent: None`, `json_schema: None`, `send_now: false`,
`persist_ack: None`, `parsed_prompt_tx: None`. The real `MvpAgent::prompt`
receives these from the client request meta. The protocol `TurnStartParams`
carries only `session_id`, `input`, `idempotency_key` — no prompt meta. The
adapter cannot faithfully forward what the protocol does not carry.

`send_now: false` means a new prompt queues behind a running turn rather than
cancel-and-replace. The real actor's `send_now` is client-driven. The
adapter's default is the conservative queue-behind behavior.

**No fix required.** Acceptable for the minimal path. Flag for the protocol
follow-on: if `TurnStartParams` gains prompt-meta fields, forward them.

### F-8 — Low — `InputBlock::Mention`/`Skill` drop `path`
`shell_session_actor_runtime.rs:319-336`: `Mention { name, .. }` and
`Skill { name, .. }` flatten to `ContentBlock::Text(TextContent::new(name))`,
dropping the `path: Option<String>` field. The real `parse_prompt` does the
rich rendering (`@mention` resolution, skill invocation). The handoff
explicitly allows "minimal real conversion" with "faithful wire shape" — the
adapter only needs to enqueue the command, and the actor's `parse_prompt`
re-renders in production. The `name` alone preserves intent.

**No fix required.** Acceptable per handoff. Flag for the production-spawn
follow-on: if the actor's `parse_prompt` requires `path` for resolution,
extend the conversion.

### F-9 — Low — `Turn.revision` and synthesized `Item.revision` hardcoded to 1
`shell_session_actor_runtime.rs:660` (`Turn.revision: WireCounter::new(1)`)
and `:706` (`Item.revision: WireCounter::new(1)`). The real actor increments
revision per item/turn. The adapter has no access to the conversation revision
without a round-trip through the actor. PARTIAL projection.

**No fix required.** Honest PARTIAL. Flag for the R2/R11 projection follow-on.

### F-10 — Low — Test fixture does not persist user message to `chat_history.jsonl`
`TestActorSpawner` (lines 60-95) appends an `AgentMessageChunk` to
`updates.jsonl` on `Prompt` but does not append the user message to
`chat_history.jsonl` (the real actor does via `persist_ack`). The test
`c1_turn_start_turn_routes_prompt_through_real_cmd_tx_and_persists` asserts
`!loaded.updates.is_empty()` — which passes on the agent reply, not the user
message. The persistence proof is real (real `JsonlStorageAdapter`, real
disk) but partial (agent reply only).

**No fix required.** The test proves the command-routing path produces a real
disk side effect, which is the AC. The user-message persistence is the real
actor's job in production. Flag for the production-spawn follow-on: when the
real actor is wired, the user message will persist through `persist_ack`.

### F-11 — Low — C1-D turn-test names now overpromise
`c1_shell_port.rs:285` (`c1_real_adapter_start_turn_returns_unsupported_actor_gap`),
and the steer/interrupt siblings, assert `unsupported` for the turn methods.
After C1-G, these methods return `unsupported` ONLY when no resident exists
(production path). The C1-D tests use `ShellSessionActorRuntime::new`
(production spawner → no resident), so they still pass — no regression. But
the test names imply a blanket "actor gap" that no longer holds for the
resident path.

**No fix required.** No regression; the tests still prove the production
no-resident path is honest. Rename to
`..._returns_unsupported_without_resident` for accuracy; non-blocking.

## Required fixes
None blocking. Recommendations (non-blocking):
1. Update the wave note §3 to clarify F-5 (mailbox ordering, not
   `dispatch_lock`) and add F-2 (ordinal reset across restarts) to the
   PARTIAL list.
2. Flag F-1/F-2/F-3/F-4 for the production-spawn follow-on handoff.

## Residual risk
- **Production actor spawn** (`spawn_session_on_thread` + creds) — the
  principal C1-G residual. `ProductionSpawner` returns `unsupported`;
  `start_turn`/`steer_turn`/`interrupt_turn` return `unsupported` when no
  resident. Honest. The `SessionSpawner` trait + `with_spawner` seam are in
  place for the follow-on.
- **R7** turn `idempotency_key` dedup not implemented (deferred).
- **R8** `steer_turn` `Item` shape is an adapter-side envelope (F-1, F-9);
  product decision pending.
- **R4** resume drain/replay of the old actor thread (out of scope).
- **R10** `respond_interaction` delivery channel (out of scope).
- **R6** `archive_session` product decision (out of scope).
- **R11** full `RuntimeEvent` projection; **R2** `read_session` Turn/Item
  projection (unchanged from C1-D).
- **F-3** no `SessionThread` reaping (handoff recommended; implementer
  omitted) — a panicked actor leaves stale `current_prompt_id`; fire-and-forget
  commands would pass the guard on a dead actor.

## Evidence reviewed

| Artifact | Path | Result |
|---|---|---|
| Implementation | `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs` | Reviewed (845 lines) |
| Tests | `crates/codegen/xai-grok-shell/tests/c1_turn_lifecycle.rs` | Reviewed (540 lines, 9 tests) |
| Module re-exports | `crates/codegen/xai-grok-shell/src/app_server_runtime/mod.rs` | Reviewed (270 lines) |
| `SessionHandle` | `crates/codegen/xai-grok-shell/src/session/handle.rs` | Reviewed (164 lines) — confirms `ResidentHandle` is the `Send`-able subset |
| `SessionCommand` | `crates/codegen/xai-grok-shell/src/session/commands.rs` | Reviewed — confirms `Prompt`/`Interject`/`Cancel` shapes |
| Actor dispatch | `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs:281,420,734` | Cross-checked command handling |
| Command map | `waves/c0-session-actor-command-map.md` §1.2 | Mapping fidelity verified |
| Wave note | `waves/c1-turn-lifecycle.md` | Reviewed (190 lines) |
| GREEN log | `tests/c1/c1_turn_lifecycle_GREEN.log` | 9/9 pass |
| GREEN gate log | `tests/c1/c1_turn_lifecycle_GREEN_gate.log` | 9/9 pass |
| RED log | `tests/c1/c1_turn_lifecycle_RED.log` | 8/9 fail (stubbed) — confirms non-vacuous |
| Regression log | `tests/c1/c1_shell_port.txt` | 18/18 `c1_real_adapter_*` pass — no regression |
| STATUS | `STATUS.md` | Updated with C1-G row |
| CHANGES | `CHANGES.md` | Updated with C1-G row |

## Commands / results (as captured)
- `cargo test -p xai-grok-shell --test c1_turn_lifecycle` → 9 passed; 0
  failed (`c1_turn_lifecycle_GREEN.log`, `..._GREEN_gate.log`).
- RED: same command with methods stubbed to `"RED stub"` → 1 passed; 8 failed
  (`c1_turn_lifecycle_RED.log`).
- `cargo test -p xai-grok-shell --test c1_shell_port` → 18 passed; 0 failed
  (`c1_shell_port.txt`) — no regression.

## Checks skipped
- No command-execution tool available to this review subagent. Static
  analysis + captured-log review only. The captured GREEN/RED logs are
  authoritative for the integration surface; the invariant guards are
  independently verified by source inspection (the static guard tests
  `include_str!` the production section and assert no
  `struct SessionActor`/`FakeRuntime::new`).
