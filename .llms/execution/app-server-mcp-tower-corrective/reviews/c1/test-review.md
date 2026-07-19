# C1-F Independent Test Review — `c1_shell_port` real-adapter suite

| Field | Value |
|---|---|
| Review mode | implementation (test-artifact review) |
| Wave | C1-D (Shell `SessionActor`-backed facade port) |
| Handoff | `HANDOFF-C1-F-test-review.md` |
| Reviewer | GLM `glm-5.2` (independent review, read-only) |
| Date | 2026-07-18 |
| Artifacts reviewed | `crates/codegen/xai-grok-shell/tests/c1_shell_port.rs`, `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`, `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs`, `.llms/execution/app-server-mcp-tower-corrective/tests/c1/{c1_shell_port.txt,composition.txt,README.md}`, `waves/c1-shell-port.md`, `scripts/run-rust-test-gate.sh` |
| Re-run executed | **No** — this subagent has no shell/command-execution tool. GREEN logs captured by the implementer were independently inspected line-by-line; storage symbols were verified against source. |

## Verdicts

- `IMPLEMENTATION_OR_ARTIFACT: PASS` (test quality for the C1-D *claimed* storage-backed REAL surface is sound; one vacuous pagination test and two overclaiming test names must be fixed — see F6/F2/F3)
- `AGENT_BEHAVIOR: PASS`
- `HANDOFF_QUALITY: PASS`
- `GOAL_GATE: N/A` (subtask review, not final-goal)

## Acceptance criteria mapping

| C1-D claimed surface | Claim | Test evidence | Verdict |
|---|---|---|---|
| `list_sessions` | REAL (dormant stub removed) | `c1_real_adapter_list_sessions_reads_jsonl_summaries_not_dormant_stub` — seeds via `JsonlStorageAdapter` directly, asserts real cwd, `assert_ne!(status, Dormant)`, real `history_epoch` | PROVEN |
| `read_session` row | REAL row, PARTIAL turns/items (R2) | `c1_real_adapter_read_session_projects_session_row_from_summary` — asserts projected row + empty turns/items (honest R2) | PROVEN |
| `start_session` | REAL `summary.json` write, UUIDv7, idempotency | `..._persists_summary_and_returns_real_id` (36-char id + `storage.load_summary` on disk), `..._idempotency_key_dedups_same_session_id`, `..._idempotency_conflict_on_different_input` | PROVEN |
| `resume_session` | REAL summary load | `..._loads_persisted_summary`, `..._unknown_returns_not_found` | PROVEN |
| `fork_session` | REAL `copy_session_data` | `..._copies_history_to_new_cwd` — asserts new id/cwd AND `storage.load_session` non-empty updates on disk | PROVEN (non-theatrical: verifies the copy, not just the return value) |
| `replay` snapshot | REAL snapshot + minimal projector (R11 PARTIAL) | `..._projects_updates_jsonl_into_runtime_events` (snapshot present), `..._epoch_mismatch_returns_error` | PROVEN |
| `replay` cursor pagination | Claimed implemented (page size 100) | `..._cursor_pagination_advances_after_event_seq` | **NOT PROVEN by test — see F6** |
| `archive_session` | PARTIAL `unsupported`, no delete | `..._returns_unsupported_not_delete` — asserts `unsupported` AND session still listed | PROVEN (honest, anti-data-loss) |
| `start_turn`/`steer_turn`/`interrupt_turn`/`respond_interaction` | PARTIAL `unsupported` | Four tests assert `err.code == "unsupported"` | PROVEN (honest gap) |
| No hybrid Fake+JSONL authority | Invariant | Static guard `shell_session_actor_runtime_does_not_use_fake_runtime` + `c1_real_adapter_no_hybrid_authority...` | PROVEN via static guard (runtime test is weak — F2) |
| Composition root injects real port | REAL port, not FakeRuntime | `composition_root_initialize_session_turn`, `composition_root_injects_real_port_not_fake_runtime` | PROVEN via source (test is smoke-only — F3) |

## Findings

### F1 — No RED evidence captured (severity: low, confidence: high)
`tests/c1/README.md` explicitly states the build subagent had no command-execution tool and that "RED baseline … optional, for the record … Skip if not reproducing; the GREEN run below is the authoritative gate." Only GREEN logs exist (`c1_shell_port.txt`, `composition.txt`).
- Evidence: `tests/c1/README.md` lines 5-26.
- Mitigation: the storage-backed tests are discriminating enough that they would fail under the prior dormant stub (e.g. `assert_ne!(row.status, SessionStatus::Dormant)`, `assert_eq!(s.session_id.len(), 36)`, on-disk `load_summary`/`load_session` assertions), giving implicit RED confidence. For the explicitly-PARTIAL surface this is acceptable; for the REAL storage-backed surface, the GREEN evidence is sufficient because the assertions are anchored to on-disk artifacts, not return values alone.
- Required fix: none (informational); a future RED reproduction (revert port impl, re-run) would strengthen the record but is not blocking.

### F2 — `c1_real_adapter_no_hybrid_authority_real_list_with_fake_mutation_rejected` overclaims (severity: low, confidence: high)
The test name claims to reject a FakeRuntime mutation path, but the body (lines 476-494) only starts a real session and asserts `sessions.len() == 1` in an isolated temp root. It never constructs `FakeRuntime`, never attempts a mutation through it, and never asserts rejection. The actual no-hybrid-authority guarantee is enforced by the static guard `shell_session_actor_runtime_does_not_use_fake_runtime` (which is sound).
- Evidence: `tests/c1_shell_port.rs:476-494`.
- Required fix: rename to reflect what it actually proves (e.g. `..._real_port_isolates_to_temp_root`), or strengthen the body to attempt a `FakeRuntime`-backed write into the same root and assert the real port's `list_sessions` does not reflect it. Not blocking — the invariant is covered by the static guard.

### F3 — `composition_root_injects_real_port_not_fake_runtime` is a smoke test (severity: low, confidence: high)
The composition test (lines 92-98) builds the processor and discards it; it asserts nothing about the inner port type. The "not fake runtime" claim is enforced by the composition-root source (`app_server_composition.rs:33` constructs `ShellSessionActorRuntime::new(root)`), not by this test's assertions.
- Evidence: `app_server_composition.rs:92-98`.
- Required fix: align the test name with the body (smoke/compile test), or add a typed assertion that the injected `FacadeProcessor` is backed by `ShellSessionActorRuntime` (e.g. via a downcast seam or a behavior that `FakeRuntime` would not exhibit). Not blocking — the source is the authority.

### F4 — `steer_turn`/`interrupt_turn`/`respond_interaction` tests use bogus session ids (severity: low, confidence: medium)
These three tests pass session_id `"s"` / `"t"` / `"ix"` without starting a real session first. Because the impl returns `unsupported` unconditionally (without a session lookup), the tests pass but do not distinguish "unsupported (actor gap)" from "unsupported (session not found)". `start_turn` correctly starts a real session first, so it is the only turn test that actually ties the `unsupported` to a real session context.
- Evidence: `tests/c1_shell_port.rs:310-357`; impl at `shell_session_actor_runtime.rs:368-399`.
- Required fix: for consistency and to make the "actor gap" framing honest, start a real session in `steer_turn`/`interrupt_turn` before asserting `unsupported` (matching `start_turn`). Not blocking — the impl is unconditionally `unsupported` so no false success is possible.

### F5 — No explicit GREEN log for the lib invariant-guard tests (severity: low, confidence: high)
`tests/c1/README.md` command #4 prescribes `cargo test -p xai-grok-shell --lib app_server_runtime` for `shell_session_actor_runtime_defines_no_session_actor` and `..._does_not_use_fake_runtime`, but no `green-invariants.log` was dropped in `tests/c1/`. The `c1_shell_port.txt` GREEN run compiled the shell crate (so the lib compiled), and the invariant tests are static `include_str!` checks that hold by direct inspection of the source (the production slice contains neither `struct SessionActor`/`enum SessionActor` nor `FakeRuntime::new`/`use xai_grok_tower::FakeRuntime`/`: FakeRuntime`).
- Evidence: `tests/c1/` directory listing (only `c1_shell_port.txt`, `composition.txt`, `README.md`); `shell_session_actor_runtime.rs:459-484`.
- Required fix: drop the captured `green-invariants.log` for traceability. Not blocking — invariants verified by source inspection.

### F6 — `c1_real_adapter_replay_cursor_pagination_advances_after_event_seq` is vacuous (severity: medium, confidence: high)
The test seeds 2 updates (3 total events: snapshot + 2 deltas) and calls `replay` with `after_event_seq=0`. `REPLAY_PAGE_SIZE = 100` (`shell_session_actor_runtime.rs:66`), so `end = min(0+100, 3) = 3`; `next_cursor = if end < total` → `3 < 3` is false → `next_cursor = None`. The test's pagination-advancement assertion is inside `if let Some(next) = first.next_cursor { ... }` (lines 444-455), which is **never entered**. The test passes without exercising a second page or asserting `replayed_through` advances across a page boundary. The wave note claims "Cursor pagination over `after_event_seq` is implemented (page size 100)" — the implementation is correct by code inspection (`next_cursor` is computed at lines 440-444), but the *test* does not prove it.
- Evidence: `tests/c1_shell_port.rs:419-455`; `shell_session_actor_runtime.rs:66,440-444`.
- Required fix (blocking for the pagination sub-claim): make the test actually exercise pagination — either seed >100 updates (e.g. 102) so `next_cursor` is `Some` and a second `replay` call returns a strictly advanced `replayed_through`, or reduce `REPLAY_PAGE_SIZE` via a test seam. Without this, the "cursor pagination advances" AC is unproven by tests.

## Non-vacuous gate check (`scripts/run-rust-test-gate.sh`)

The gate script is **non-vacuous**:
- Requires `cargo test` as the command form (lines 8-12).
- `set -euo pipefail` (line 2) means any non-zero cargo exit (any test failure) aborts before the grep runs.
- The grep `^test .*${expected_test}.* \.\.\. ok$` (line 21) is anchored to a `test … ok` line and requires at least one passing test matching the fragment.
- Combined effect: the full targeted test suite must pass AND a matching test must be present. The `c1_real_adapter` fragment matches all 18 real-adapter tests, so the gate effectively requires the whole `c1_shell_port` suite green.

Minor: the gate only requires *one* matching test to be present and passing (grep succeeds on first match); it does not explicitly count tests. This is acceptable because `set -e`+`pipefail` already forces the entire `cargo test` invocation to succeed.

## Real-adapter vs FakeRuntime-only check

The suite is genuinely real-adapter, not FakeRuntime-only:
- `real_port(temp)` constructs `ShellSessionActorRuntime::new(temp.path())` — the real port backed by `JsonlStorageAdapter` (`tests/c1_shell_port.rs:28-30`).
- Tests seed real on-disk state via `JsonlStorageAdapter` directly (`seed_update`, lines 32-50) and assert on-disk artifacts (`storage.load_summary`, `storage.load_session`, `list_sessions` contents) — not just return values.
- `c1_real_adapter_start_session_persists_summary_and_returns_real_id` asserts the summary exists on disk via `storage.load_summary` (lines 124-131).
- `c1_real_adapter_fork_session_copies_history_to_new_cwd` asserts the forked `updates.jsonl` is non-empty on disk via `storage.load_session` (lines 248-256) — proves a real copy, not a return-value stub.
- Static guards (`shell_session_actor_runtime.rs:459-484`) enforce the real port neither defines a `SessionActor` nor uses `FakeRuntime`.
- The real storage symbols referenced (`init_session`, `list_sessions`, `copy_session_data`, `load_summary`, `updates_file_path`, `UpdatesIterator::open`) all exist in `crates/codegen/xai-grok-shell/src/session/storage/` (verified by grep) — the "REAL" claims are not theatrical.

## "unsupported" honesty check

- `archive_session`: the test asserts `err.code == "unsupported"` AND that the session is still listed afterward (lines 270-283). This is **honest and anti-theatrical** — it proves the `unsupported` is not a disguised `delete_session` (no data loss). Aligns with wave-note R6.
- `start_turn`: starts a real session first, then asserts `unsupported` (lines 286-308). Honest — the gap is tied to a real session, not a missing one.
- `steer_turn`/`interrupt_turn`/`respond_interaction`: assert `unsupported` with bogus ids (F4). Honest about the gap but slightly weaker than `start_turn` (no real session context). No canned-success mock; no silent fake.

## RED/GREEN for production-readiness claims

The C1-D surface is explicitly **PARTIAL**, not production-ready (wave note §7). The storage-backed methods claimed REAL have solid GREEN evidence with on-disk assertions. RED is missing (F1) but is acknowledged and mitigated by discriminating assertions. No behavior is claimed production-ready without GREEN evidence; the actor-backed methods are honestly `unsupported`.

## Coverage gaps for turn methods (expected PARTIAL)

Turn-method coverage is appropriate for the PARTIAL claim:
- One `unsupported` assertion per turn method (`start_turn`/`steer_turn`/`interrupt_turn`) and `respond_interaction`.
- No idempotency-key dedup tests for turns — consistent with wave note §7 (not implemented).
- `read_session` turns/items empty asserted (R2 PARTIAL) — honest.
- `replay` full lifecycle projection deferred (R11 PARTIAL); tests assert snapshot + AgentMessageChunk delta only — honest.
- Gap: F4 (turn tests other than `start_turn` don't start a real session) and F6 (pagination vacuous). No other turn-coverage gaps; the PARTIAL framing is not used to hide a fake success.

## Required fixes (blocking for the named sub-claims)

1. **F6 (medium)** — Strengthen `c1_real_adapter_replay_cursor_pagination_advances_after_event_seq` to actually cross a page boundary (seed >100 updates or expose a page-size seam) and assert `replayed_through` strictly advances on the second `replay` call. Until then, the "cursor pagination advances" AC is unproven by tests.
2. **F2 (low)** — Rename or strengthen `c1_real_adapter_no_hybrid_authority_real_list_with_fake_mutation_rejected` so the body matches the name (the invariant is already covered by the static guard).
3. **F3 (low)** — Align `composition_root_injects_real_port_not_fake_runtime` test name with its smoke-test body, or add a typed assertion.
4. **F4 (low)** — Start a real session in `steer_turn`/`interrupt_turn` tests before asserting `unsupported`, for consistency with `start_turn`.
5. **F5 (low)** — Drop `green-invariants.log` for the lib invariant-guard tests.

## Residual risk

- The core storage-backed REAL claims (list/read/start/resume/fork/replay-snapshot) are well-proven and low-risk.
- The pagination sub-claim (F6) is the only AC with a vacuous test; the implementation is correct by inspection, so residual risk is a test-coverage gap, not a behavior defect.
- No second `SessionActor` and no hybrid authority — enforced by static guards and verified by source inspection.
- `provider_binding` left `None` and `history_epoch` hardcoded to `epoch_1` are documented PARTIAL (wave note §7) — not a test gap.

## Commands / results

| Command | Run by | Result |
|---|---|---|
| `./scripts/run-rust-test-gate.sh c1_real_adapter cargo test -p xai-grok-shell --test c1_shell_port …` | implementer (logged in `c1_shell_port.txt`) | 18 passed, 0 failed (independently inspected: log lines 51-72 show all 18 `c1_real_adapter_*` tests `ok`, `test result: ok. 18 passed; 0 failed`) |
| `cargo test -p xai-grok-pager-bin --lib composition …` | implementer (logged in `composition.txt`) | 2 passed ×3 binary targets (goblin, grok-oss, xai-grok-pager), 0 failed (log lines 55-77) |
| `cargo test -p xai-grok-shell --lib app_server_runtime` (invariant guards) | **not run / not logged** | F5 — verified by source inspection instead |
| Re-run by reviewer | **skipped — no shell tool available in this review subagent** | GREEN logs inspected; storage symbols verified by grep |

## Summary

Test quality for the C1-D *claimed* storage-backed REAL surface is **PASS**: the suite is genuinely real-adapter (not FakeRuntime-only), asserts on-disk artifacts, the `unsupported` PARTIAL surface is honest and anti-data-loss, and the gate is non-vacuous. One medium finding (F6: vacuous pagination test) leaves the "cursor pagination advances" sub-claim unproven by tests, and three low findings (F2/F3/F4) are test-name/body misalignments. No blocking defect in the implementation-under-test; the required fixes are test-strengthening only. No source mutation was performed by this reviewer.
