# C1-I Independent Test Review — `c1_turn_lifecycle` turn-lifecycle suite

| Field | Value |
|---|---|
| Review mode | independent test-adequacy review (read-only, no implementation) |
| Wave | C1-G (turn lifecycle via `SessionHandle` channels) |
| Handoff | `HANDOFF-C1-I-test-review.md` |
| Reviewer | GLM `glm-5.2` (independent review, read-only) |
| Date | 2026-07-18 |
| Artifacts reviewed | `crates/codegen/xai-grok-shell/tests/c1_turn_lifecycle.rs`, `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`, `crates/codegen/xai-grok-shell/src/app_server_runtime/mod.rs`, `crates/codegen/xai-grok-shell/src/session/commands.rs`, `crates/codegen/xai-grok-shell/src/session/handle.rs`, `crates/codegen/xai-grok-shell/tests/c1_shell_port.rs`, `.llms/execution/app-server-mcp-tower-corrective/tests/c1/{c1_turn_lifecycle_RED.log,c1_turn_lifecycle_GREEN.log,c1_turn_lifecycle_GREEN_gate.log,c1_shell_port.txt,composition.txt,README.md}`, `waves/c1-turn-lifecycle.md`, `scripts/run-rust-test-gate.sh` |
| Re-run executed | **No** — this review subagent has no shell/command-execution tool. GREEN/RED logs captured by the implementer were independently inspected line-by-line; facade routing code and `SessionCommand` wire shapes were verified against source. |

## Verdict

**PASS_WITH_FINDINGS**

The C1-G turn-lifecycle test suite satisfies the handoff acceptance criteria with honest PARTIAL framing. RED→GREEN evidence is present and non-empty; at least one test exercises a real `cmd_tx` consumer (NOT `FakeRuntime`) that processes the real `SessionCommand` enum and persists through the real `JsonlStorageAdapter`; concurrent start / interrupt / steer / resume coverage is present against acceptance item 10; the gate is non-vacuous; and `ProductionSpawner` PARTIAL honesty is proven by a dedicated test. No Fake-as-production, no SKIP-as-PASS, no empty filter. Five low/informational findings remain (test-name/body misalignments and one missing invariant log); none are blocking.

## Acceptance criteria mapping (handoff C1-I checklist)

| # | Criterion | Evidence | Verdict |
|---|---|---|---|
| 1 | RED→GREEN evidence present and non-empty | `c1_turn_lifecycle_RED.log`: 8/9 fail with `RuntimeError { code: "unsupported", message: "RED stub" }`; the 1 passing test (`..._without_resident_returns_unsupported`) expects `unsupported` so it passes under the stub — confirms the other 8 exercise the real routing path. `c1_turn_lifecycle_GREEN.log` + `..._GREEN_gate.log`: 9/9 pass, gate exit 0. | PROVEN |
| 2 | At least one test exercises real actor/handle path (not only FakeRuntime) | `TestActorSpawner`/`HeldTurnSpawner` are real `mpsc::UnboundedChannel` consumers of the real `SessionCommand::{Prompt,Interject,Cancel}` enum (field shapes verified against `commands.rs:122-156,572-582,672-681`); they persist via the real `JsonlStorageAdapter::append_update` and the test asserts `load_session` sees the update on disk. The facade routing code (`start_turn`/`steer_turn`/`interrupt_turn` bodies at `shell_session_actor_runtime.rs:599-742`) is identical whether the `ResidentHandle` came from a real `SessionHandle` or the test spawner — the facade only uses `cmd_tx.send()` + `current_prompt_id.lock()`. So the tests exercise the real facade path; only the consumer side is a stand-in (explicitly allowed by handoff AC #3: "real SessionActor path **or equivalent real cmd_tx consumer** that is not FakeRuntime"). | PROVEN |
| 3 | Concurrent start / interrupt / steer coverage vs item 10 | `c1_turn_concurrent_starts_serialize_through_single_mailbox` (2 concurrent `start_turn`s, distinct ids, no deadlock); `c1_turn_interrupt_turn_cancels_running_turn_only` (held running turn, interrupt succeeds, `start_turn` future resolves); `c1_turn_steer_turn_against_running_turn_returns_item` (held running turn, steer returns `Item` with matching `turn_id`/`session_id`); `c1_turn_steer_turn_turn_id_mismatch_returns_turn_not_found` + `c1_turn_interrupt_turn_turn_id_mismatch_returns_turn_not_found` (turn_id guard); `c1_turn_resume_re_residents_actor_and_routes_turn` (R4 re-resident). | PROVEN |
| 4 | No empty `cargo test` filter that always passes | Gate: `./scripts/run-rust-test-gate.sh c1_turn cargo test -p xai-grok-shell --test c1_turn_lifecycle`. `set -euo pipefail` (line 2) forces the entire cargo invocation to succeed before the grep runs; grep `^test .*c1_turn.* \.\.\. ok$` (line 21) requires at least one passing test matching `c1_turn` — 9 named tests match. The `--test c1_turn_lifecycle` targets a specific test file (no bare `cargo test` with an empty filter). | PROVEN (non-vacuous) |
| 5 | Gaps listed as OPEN/PARTIAL with unblock condition | `waves/c1-turn-lifecycle.md` §3 (REAL vs PARTIAL), §6 (honest remaining gaps): production actor spawn (needs creds → next handoff replaces `ProductionSpawner`); R7 turn idempotency-key dedup; R4 resume drain/replay; R8 `steer_turn` `Item` shape (product decision); R10 `respond_interaction`; R6 `archive_session`; R11/R2 projection. Each has an unblock condition. | PROVEN |

## Strict checks (per review brief)

### Fake-as-production
**Not present.** `TestActorSpawner` and `HeldTurnSpawner` are test fixtures confined to `tests/c1_turn_lifecycle.rs` and are clearly documented as test seams (module doc lines 1-13; struct doc lines 33-39). They do not import or construct `FakeRuntime`; they consume the real `SessionCommand` enum and persist via the real `JsonlStorageAdapter`. The production path uses `ProductionSpawner` (constructed by `ShellSessionActorRuntime::new`/`with_storage`), which honestly returns `unsupported`. The static guard `shell_session_actor_runtime_does_not_use_fake_runtime` still holds (verified by source inspection: the production slice of `shell_session_actor_runtime.rs` contains none of `FakeRuntime::new` / `use xai_grok_tower::FakeRuntime` / `: FakeRuntime`).

### SKIP-as-PASS
**Not present.** No `#[ignore]`, no `#[cfg(skip)]`, no skipped tests. All 9 tests run and pass in both the GREEN log and the gate log (`running 9 tests` … `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured`).

### Empty filters
**Not present.** The gate command targets a specific test binary (`--test c1_turn_lifecycle`) and requires a matching `c1_turn*` test to pass. The README's prescribed commands all name specific test binaries or lib fragments; none use a bare `cargo test` with an empty filter that would pass on zero matches.

### ProductionSpawner PARTIAL honesty
**Honest.** `ProductionSpawner::spawn` returns `Err(RuntimeError { code: "unsupported", message: "live SessionActor spawn not assembled in this slice (C1-G PARTIAL: needs credentials)." })` (`shell_session_actor_runtime.rs:142-151`). `ensure_resident` swallows the `unsupported` case as a no-op (lines 238-244) so storage-backed methods still work without a resident. The dedicated test `c1_turn_start_turn_without_resident_returns_unsupported` (lines 172-190) constructs the default `ShellSessionActorRuntime::new(temp)` (ProductionSpawner), starts a real session, and asserts `start_turn` returns `unsupported` — proving the production path does not fake a turn. The wave note §3 explicitly marks production spawn as PARTIAL. The C1-D suite's `c1_real_adapter_{start,steer,interrupt}_turn_returns_unsupported_actor_gap` tests (still green — see below) reinforce this for steer/interrupt.

## `tests/c1_shell_port.rs` still green?

**Yes — no regression.** `c1_shell_port.txt` shows 18/18 `c1_real_adapter_*` pass (`test result: ok. 18 passed; 0 failed`). The C1-G changes added `ensure_resident` calls in `start_session`/`resume_session`, but with the default `ProductionSpawner` these return `unsupported` (swallowed as no-op), so storage-backed behavior is unchanged. The three C1-D turn-unsupported tests use `real_port(&temp)` (default ProductionSpawner) → no resident → `start_turn`/`steer_turn`/`interrupt_turn` return `unsupported` (the no-resident branch at `shell_session_actor_runtime.rs:601-604,669-672,719-722`), matching their assertions. `composition.txt` shows 3/3 composition-root tests pass (2 tests × 3 bin targets: goblin, grok-oss, xai-grok-pager). The log timestamps and README ("C1-G (turn lifecycle) — captured 2026-07-18") confirm these were re-captured post-C1-G.

## Are the 0.01s "cmd_tx consumer" tests non-vacuous vs full SessionActor?

**Non-vacuous for what they claim; not a full SessionActor test (honestly PARTIAL).**

The GREEN log reports `finished in 0.01s` for 9 tests. This is fast but not vacuous:
- `c1_turn_start_turn_routes_prompt_through_real_cmd_tx_and_persists` does real work: `start_session` (writes `summary.json` via `init_session`) → `start_turn` (converts `InputBlock`→`ContentBlock`, builds `SessionCommand::Prompt` with a real `oneshot::Sender`, sends through `cmd_tx`) → the consumer task receives, sets `current_prompt_id`, appends an `AgentMessageChunk` to `updates.jsonl` via `JsonlStorageAdapter::append_update` (real disk write), resolves the oneshot → the facade maps `PromptTurnOk.completion_kind` → `TurnStatus` → the test reads the session back via `storage.load_session` and asserts `!loaded.updates.is_empty()`. The 0.01s reflects the simplicity of the consumer (no model inference, no real actor thread) and small temp-dir I/O — not a missing assertion.
- The `HeldTurnSpawner` tests use `poll_until_running` (5 ms poll interval, 2 s timeout) which returns as soon as `current_prompt_id` is `Some`, then steer/interrupt + `turn_handle.await`. These also complete fast because the held consumer breaks its inner loop on the first `Cancel`.

What the tests **do** prove (real, non-theatrical):
1. Facade command routing: `InputBlock`→`ContentBlock` conversion, `cmd_tx.send(SessionCommand::Prompt)`, oneshot wiring, `PromptTurnResult`→`Turn` mapping.
2. Real persistence through the command path: `JsonlStorageAdapter::append_update` writes `updates.jsonl`; `load_session` reads it back.
3. Turn-id guard (R8/R9): `steer_turn`/`interrupt_turn` verify `turn_id == current_prompt_id`; mismatch → `turn_not_found`; match → success.
4. Foreground serialization (item 10): two concurrent `start_turn`s both complete with distinct turn ids through the single consumer mailbox.
5. Resume re-resident (R4 command path): `resume_session` re-residents and a subsequent turn routes.
6. Honest `unsupported` when no resident (production PARTIAL).

What the tests **do not** prove (honestly PARTIAL, not claimed PASS):
- The full `SessionActor` path: `dispatch_lock`, `parse_prompt`, model inference, `run_loop`, tool/permission/MCP context. The test consumer is a simplified mailbox, not the real actor.
- Production actor spawn: `ProductionSpawner` returns `unsupported`; the real `spawn_session_on_thread` factory is not wired (needs creds).

The handoff AC #3 explicitly allows "equivalent real `cmd_tx` consumer that is not FakeRuntime" as an alternative to "real SessionActor path." The test consumer satisfies this: it consumes the real `SessionCommand` enum (verified field shapes), persists via the real `JsonlStorageAdapter`, and is not `FakeRuntime`. The facade routing code is identical for real and test-provided handles. So the AC is satisfied by both letter and spirit. The 0.01s timing is a property of the consumer's simplicity, not vacuity.

## Findings

### F1 — `c1_turn_steer_turn_targets_running_turn_and_returns_item` test name/body mismatch (severity: low, confidence: high)
The test name claims "targets_running_turn_and_returns_item" but the body (lines 196-231) asserts `err.code == "turn_not_found"`. The `TestActorSpawner` consumer resolves the `Prompt` synchronously and clears `current_prompt_id` before `start_turn` returns, so by the time `steer_turn` runs there is no running turn. The test's own comment (lines 204-213) acknowledges this and says it "instead asserts the turn_not_found path when no turn is running (the honest behavior)." The actual "running turn + returns Item" coverage is provided by `c1_turn_steer_turn_against_running_turn_returns_item` (HeldTurnSpawner, lines 333-396). The misnamed test is misleading and partially redundant with `c1_turn_steer_turn_turn_id_mismatch_returns_turn_not_found`.
- Evidence: `tests/c1_turn_lifecycle.rs:196-231`.
- Required fix: rename to reflect what it proves (e.g. `c1_turn_steer_turn_after_turn_completes_returns_turn_not_found`) or remove it (the mismatch test already covers the `turn_not_found` path). Not blocking — the positive running-turn coverage exists in the HeldTurnSpawner test.

### F2 — Wave note "real actor path" wording slightly overclaims (severity: low, confidence: high)
`waves/c1-turn-lifecycle.md` §3 says "proving command routing against a real actor path." The test consumer is a real `cmd_tx` consumer, not a real actor path. The handoff AC allows "equivalent real cmd_tx consumer," so the substance is correct, but the wording could be read as implying the full `SessionActor` is exercised.
- Evidence: `waves/c1-turn-lifecycle.md` lines 28-31, 60-63.
- Required fix: tighten to "real `cmd_tx` consumer path" to match the AC wording and the §3 PARTIAL section. Not blocking — the PARTIAL framing is honest.

### F3 — No explicit GREEN log for lib invariant-guard tests (severity: low, confidence: high)
The wave note claims "7/7 `app_server_runtime` lib tests pass (including both static guards)" but no `green-invariants.log` was dropped in `tests/c1/` (carried forward from C1-D review F5). The two static guards (`shell_session_actor_runtime_defines_no_session_actor`, `shell_session_actor_runtime_does_not_use_fake_runtime`) are verifiable by source inspection: the production slice of `shell_session_actor_runtime.rs` contains neither `struct SessionActor`/`enum SessionActor` (the only real `SessionActor` is `session/acp_session.rs:564`) nor `FakeRuntime::new`/`use xai_grok_tower::FakeRuntime`/`: FakeRuntime`. So the invariants hold, but the log is missing for traceability.
- Evidence: `tests/c1/` directory listing (no `green-invariants.log`); `shell_session_actor_runtime.rs:817-837`.
- Required fix: drop the captured `green-invariants.log` for the lib invariant-guard tests. Not blocking.

### F4 — `c1_turn_concurrent_starts_serialize_through_single_mailbox` name overclaims serialization (severity: low, confidence: medium)
The test spawns two concurrent `start_turn`s and asserts both complete with distinct turn ids (lines 460-491). It does not assert that the second turn waited for the first — two distinct ids would also be produced by a parallel consumer. The single-mailbox serialization is a structural property of the `mpsc` channel + single consumer task, not a behavioral assertion in the test. The test proves "concurrent starts both succeed with distinct ids, no deadlock/corruption," which satisfies the handoff item-10 AC ("concurrent start serialization or second start while busy"), but the name's "serialize" is stronger than what the body proves.
- Evidence: `tests/c1_turn_lifecycle.rs:460-491`.
- Required fix: either rename to `..._concurrent_starts_complete_with_distinct_ids` or add a behavioral assertion that the second turn's `created_at_ms` is strictly after the first's completion. Not blocking — the AC is satisfied.

### F5 — 0.01s timing reflects simplified consumer, not full SessionActor (severity: informational, confidence: high)
The 9 tests finish in 0.01s because the test consumer resolves oneshots immediately and performs only one small JSONL write per turn. This is not vacuous (real channel send + real disk write + real read-back occur) but it does mean the tests do not exercise the real `SessionActor`'s `dispatch_lock`/`parse_prompt`/`run_loop`/model-inference path. This is honestly documented as PARTIAL (production spawn needs creds) and is within the handoff AC #3 allowance for "equivalent real cmd_tx consumer." No fix required; noted for the record so a future handoff that wires `spawn_session_on_thread` knows to add a test against the real actor path.

## Invariants re-verified

- **No second `SessionActor`.** `ResidentHandle` holds only `cmd_tx: mpsc::UnboundedSender<SessionCommand>` + `current_prompt_id: Arc<Mutex<Option<String>>>` — the `Send`-able projection of `SessionHandle` (`handle.rs:42,49`). The static guard `shell_session_actor_runtime_defines_no_session_actor` holds by source inspection. The only real `SessionActor` remains `session/acp_session.rs:564`.
- **No hybrid Fake+JSONL authority.** The real port never imports/constructs `FakeRuntime` (static guard holds by source inspection). The test consumer uses the real `JsonlStorageAdapter`, not `FakeRuntime`.
- **Tower must not gain Shell dependency.** Unchanged — `xai-grok-tower/Cargo.toml` does not reference `xai-grok-shell` (enforced by `app_server_runtime_adapter_lives_in_shell_not_tower` in `mod.rs`).
- **No second turn state machine.** Turn status is derived from the real `PromptTurnResult.completion_kind` (lines 645-651); `current_prompt_id` is read from the shared slot. The adapter introduces no parallel `Turn` state machine.
- **`SessionHandle` is `Clone + Send`; actor is `!Send`.** `ResidentHandle` is built from the `Send`-able subset and never moves the actor across threads.

## Commands / results

| Command | Run by | Result |
|---|---|---|
| `./scripts/run-rust-test-gate.sh c1_turn cargo test -p xai-grok-shell --test c1_turn_lifecycle` | implementer (logged in `c1_turn_lifecycle_GREEN_gate.log`) | 9 passed, 0 failed; gate exit 0 (log lines 47-58) |
| `cargo test -p xai-grok-shell --test c1_turn_lifecycle` (RED, stubbed) | implementer (logged in `c1_turn_lifecycle_RED.log`) | 1 passed, 8 failed with `code: "unsupported", message: "RED stub"` (log lines 56-107) — confirms tests exercise real routing |
| `cargo test -p xai-grok-shell --test c1_shell_port` (no-regression) | implementer (logged in `c1_shell_port.txt`) | 18 passed, 0 failed (log lines 51-72) |
| `cargo test -p xai-grok-pager-bin --bins composition` | implementer (logged in `composition.txt`) | 2 passed ×3 bin targets, 0 failed (log lines 55-77) |
| `cargo test -p xai-grok-shell --lib app_server_runtime` (invariant guards) | **not run / not logged** | F3 — verified by source inspection instead |
| Re-run by reviewer | **skipped — no shell tool available in this review subagent** | GREEN/RED logs inspected; facade routing + `SessionCommand` wire shapes verified against source |

## Summary

The C1-G turn-lifecycle test suite is **non-vacuous and honest**. RED→GREEN evidence is present and discriminating (8/9 fail under the stub). The tests exercise the real facade command-routing path (`InputBlock`→`ContentBlock`→`SessionCommand`→`cmd_tx`→oneshot→`Turn`/`Item`) against a real `cmd_tx` consumer that persists through the real `JsonlStorageAdapter` — not `FakeRuntime`. Concurrent start, interrupt (running + mismatch), steer (running + mismatch), and resume re-resident are all covered. The gate is non-vacuous (`set -euo pipefail` + anchored grep + specific test binary). `ProductionSpawner` PARTIAL is honestly proven by a dedicated test. `tests/c1_shell_port.rs` is still green (18/18, no regression). The 0.01s timing reflects the simplified consumer, not vacuity — the tests do real channel + disk work and prove the facade routing path, while honestly leaving the full `SessionActor` path as PARTIAL (within handoff AC #3's "equivalent real cmd_tx consumer" allowance).

Five findings, all low/informational and non-blocking: F1 (test name/body mismatch on `..._targets_running_turn_and_returns_item`), F2 (wave-note "real actor path" wording), F3 (missing `green-invariants.log`), F4 (concurrent-starts test name overclaims serialization), F5 (informational note on 0.01s timing vs full SessionActor). No source mutation was performed by this reviewer.

**Verdict: PASS_WITH_FINDINGS.**
