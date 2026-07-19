# C1-E Independent Code Review (GLM `glm-5.2`)

| Field | Value |
|---|---|
| Wave | C1-D (Shell SessionActor-backed facade port) |
| Review mode | `implementation` (read-only) |
| Reviewer | GLM `glm-5.2` |
| Date | 2026-07-18 |
| Handoff | `HANDOFF-C1-E-code-review.md` |
| Implementer handoff | `waves/c1-shell-port.md` (C1-D wave note) |
| Branch | `goblin-implement-epic-tree` |

## Verdicts

- **IMPLEMENTATION_OR_ARTIFACT: PASS**
- **AGENT_BEHAVIOR: PASS**
- **HANDOFF_QUALITY: PASS**
- **GOAL_GATE: N/A** (wave-level review; final-goal gate not in scope)

Every applicable acceptance criterion is proven by code evidence + captured
GREEN logs. No blocking finding remains. Residual risks and an evidence gap
are recorded below; none blocks C1-D acceptance.

## Acceptance criteria — proof matrix

| AC | Evidence | Status |
|---|---|---|
| No hybrid Fake+JSONL mutation split | `shell_session_actor_runtime.rs` production section (lines 1-453) contains no `FakeRuntime::new`, no `use xai_grok_tower::FakeRuntime`, no `: FakeRuntime` (grep confirmed — only doc-comment mentions and the `#[cfg(test)]` guard). No `SessionStorageHybridRuntime` symbol anywhere. Composition root (`app_server_composition.rs:31-32`) injects `ShellSessionActorRuntime::new(root)`, not `FakeRuntime`. | PROVEN |
| Composition injects real port, not FakeRuntime, for product path | `app_server_composition.rs:20-33`: `experimental_app_server_processor()` → `grok_home()` → `experimental_app_server_processor_with_root(root)` → `Arc::new(ShellSessionActorRuntime::new(root))` wrapped in `ShellRuntimeAdapter`. `FakeRuntime` appears only in doc comments (lines 5-6) and a test name (line 90), never in product code. | PROVEN |
| PARTIAL methods honestly return `unsupported` (not fake success) | `shell_session_actor_runtime.rs`: `archive_session` (357-364), `start_turn` (366-372), `steer_turn` (374-380), `interrupt_turn` (382-388), `respond_interaction` (391-399) all return `Err(RuntimeError { code: "unsupported", .. })` with explanatory messages. No silent no-op, no fake `Ok`. `archive_session` test (c1_shell_port.rs:255-285) additionally asserts the session is still on disk afterward. | PROVEN |
| No second `SessionActor` | grep for `struct SessionActor`/`enum SessionActor` in `shell_session_actor_runtime.rs` matches only inside `#[cfg(test)]` (lines 465-466, the guard itself). Static guard `shell_session_actor_runtime_defines_no_session_actor` splits on `#[cfg(test)]` and asserts the production section is clean. `mod.rs` production section is likewise clean (guard `app_server_runtime_defines_no_session_actor_state_machine`). The only real actor remains `session/acp_session.rs:564` (per C0-B). | PROVEN |
| Tower still no Shell dep | `crates/codegen/xai-grok-tower/Cargo.toml` lists only `async-trait`, `serde_json`, `xai-grok-app-server-protocol` (deps) and `tempfile`, `tokio` (dev-deps). No `xai-grok-shell`. Tower guard `leader_characterization_tower_has_no_second_actor_type` (`xai-grok-tower/src/lib.rs:91-120`) asserts both the source and Cargo.toml invariants. | PROVEN |
| Gaps R6/R10/R11/actor-fixture documented honestly | `waves/c1-shell-port.md` §2 (method map with REAL/PARTIAL per method), §3 (R6/R10/R11/R2 design sketches), §7 (honest remaining gaps enumerating R2-R11 + provider_binding). Doc comments in `shell_session_actor_runtime.rs` lines 1-40 mirror the same PARTIAL/REAL split. README in `tests/c1/` honestly states the build subagent could not run tests itself. | PROVEN |

## Test evidence reviewed

| Log | Result |
|---|---|
| `tests/c1/c1_shell_port.txt` | 18/18 `c1_real_adapter_*` passed; `test result: ok. 18 passed; 0 failed` |
| `tests/c1/composition.txt` | 2/2 composition tests passed across all three bin targets (`goblin`, `grok_oss`, `xai_grok_pager`); `test result: ok. 2 passed; 0 failed` (×3) |
| `tests/c1/README.md` | Honestly documents that the build subagent lacked a command-execution tool; lists the commands the parent/reviewer must run. |

Compilation of `xai-grok-shell`, `xai-grok-pager`, `xai-grok-pager-bin`,
`xai-grok-update`, `xai-grok-pager-minimal` all succeeded (composition.txt
lines 47-49). Only pre-existing unrelated warnings (`xai-grok-multi-auth`
unused `AuthProvider`, `xai-grok-sampling-types` dead code) — none in the
changed surface.

## Independent static verification (re-checked, not trusted from implementer)

1. **No hybrid authority.** grep `FakeRuntime` in
   `shell_session_actor_runtime.rs` → only doc comments (lines 5, 12, 77, 469,
   470-471) and the `#[cfg(test)]` guard (477-480). Production code (pre-`#[cfg(test)]`)
   has zero `FakeRuntime` usage.
2. **No second actor.** grep `struct SessionActor`/`enum SessionActor` in the
   runtime file → only the test-guard assertions. `mod.rs` production section
   clean.
3. **Tower independence.** `xai-grok-tower/Cargo.toml` has no `xai-grok-shell`
   dependency; Tower guard test asserts the same.
4. **Dormant stub removed.** grep `project_active_session_row` in
   `app_server_runtime/mod.rs` → no matches. `list_sessions` now projects the
   real `Summary` via `project_summary_to_session` (line 140-176).
5. **Real symbols exist.** `JsonlStorageAdapter::with_root`
   (`storage/jsonl/mod.rs:51`), `StorageAdapter::{init_session, load_summary,
   list_sessions, copy_session_data, updates_file_path}`
   (`storage/mod.rs:515,638,642,678,702`), `grok_home::grok_home()`
   (`util/grok_home`, referenced from `util/config/mcp.rs:1311` and
   `util/config/resolve/version.rs:17`). All compile (GREEN logs).
6. **PARTIAL honesty.** All five actor/interaction methods return
   `code: "unsupported"` with messages naming the gap (R6/R10/actor-fixture).
   None return `Ok(..)` with fake data.

## Findings (severity / confidence / evidence)

### F-1 — Medium / Medium — Invariant-guard execution log not captured
The two static invariant guards
(`shell_session_actor_runtime_defines_no_session_actor`,
`shell_session_actor_runtime_does_not_use_fake_runtime`) and the `mod.rs`
guards are the primary automated proof for "no second SessionActor" and "no
hybrid authority". The README documents the command
(`cargo test -p xai-grok-shell --lib app_server_runtime`) but no
`green-invariants.log` was captured in `tests/c1/`. The two captured logs only
run the integration test binary and the pager-bin lib — neither exercises the
shell lib unit tests.

I independently verified the assertions hold against current source (see §
"Independent static verification" above), so the guards would PASS if run.
But the evidence packet is incomplete for these specific guards.

**Required fix (evidence, not code):** capture
`cargo test -p xai-grok-shell --lib app_server_runtime -- --nocapture` into
`tests/c1/green-invariants.log` and confirm the guard tests pass. Not blocking
because the invariants are independently proven by static inspection.

### F-2 — Low / High — `composition_root_injects_real_port_not_fake_runtime` is a weak smoke test
`app_server_composition.rs:88-96` builds the processor and immediately drops
it; it asserts nothing about the inner runtime type. The real protection is
the static guard in `shell_session_actor_runtime.rs` (F-1) plus source
inspection of `app_server_composition.rs:31-32`. The test name overpromises
relative to its body.

**No fix required.** The AC is satisfied by source evidence + the static
guard. A stronger test would assert the processor handles a real
`session/start` round-trip (already covered by
`composition_root_initialize_session_turn`), which is present and GREEN.

### F-3 — Low / Medium — `c1_real_adapter_no_hybrid_authority_real_list_with_fake_mutation_rejected` test name overpromises
`c1_shell_port.rs:477-496` only asserts `list_sessions().len() == 1` for an
isolated TempDir. It does not exercise any "fake mutation rejection" path
(there is no fake mutation path in the real port to reject — by design). The
real hybrid-authority protection is the static guard in the port module. The
test is a documentation smoke test, not a behavioral rejection test.

**No fix required.** The name is misleading but the underlying invariant is
proven by the static guard. Renaming to
`c1_real_adapter_list_returns_only_real_sessions_in_isolated_root` would be
more honest; not blocking.

### F-4 — Low / Medium — `start_session` idempotency has a TOCTOU race
`shell_session_actor_runtime.rs:260-307`: the idempotency check
(`guard.get(&key)`) releases the Mutex, then `init_session` runs (async,
unguarded), then `guard.insert`. Two concurrent `start_session` calls with
the same `idempotency_key` can both observe "no existing", both call
`init_session` (creating two sessions on disk), and the second insert
overwrites the first. Result: two persisted sessions, dedup only honored for
later calls.

This matches the documented PARTIAL status (actor fixture gap; the live
actor's `dispatch_lock` would serialize). It is not claimed as PASS. Residual
risk for the C1 follow-on: idempotency-key dedup must hold the lock across the
storage write, or delegate to a storage-side upsert keyed on
`idempotency_key`.

**No fix required for C1-D.** Document as a known PARTIAL limitation; the
wave note §7 already lists R3/R5 idempotency gaps. Flag for the actor-fixture
follow-on.

### F-5 — Low / High — `replay` materializes the full event stream before paginating
`shell_session_actor_runtime.rs:418-451`: `all_events` is built in full
(`Vec<RuntimeEvent>` over the entire `updates.jsonl`) before slicing
`[after..end]`. For large sessions this is O(N) memory per replay call. It is
NOT a second replay buffer (events are projected from `updates.jsonl` via
`UpdatesIterator` on each call, not buffered persistently), so the "no second
replay buffer" invariant holds. PARTIAL, documented (R11).

**No fix required for C1-D.** A streaming projector (seek to `after_event_seq`
before projecting) is the C1 follow-on optimization.

### F-6 — Low / High — `HISTORY_EPOCH` is a hardcoded constant
`shell_session_actor_runtime.rs:60-62`: `const HISTORY_EPOCH: &str =
"epoch_1"`. Shell `Summary` has no epoch field. The wave note §3 / doc comment
explicitly address the F-13 root cause (synthetic *per-row* epoch) and explain
this is a single stable value, not synthetic per-row. Documented as a
placeholder until a real epoch concept lands in Shell.

**No fix required.** Honest PARTIAL; documented.

## Required fixes
None blocking. Recommendations (non-blocking, evidence-only):
1. Capture `green-invariants.log` (F-1).
2. Optionally rename the overpromising test in F-3.

## Residual risk
- **Actor fixture gap (R3/R7/R8/R9):** turn/interaction methods return
  `unsupported`. This is the C1 follow-on — wiring the `!Send` `SessionActor`
  on a dedicated thread + `LocalSet` + auth/credentials/tool-context. Until
  then, the product path cannot run turns. Honest.
- **Idempotency TOCTOU (F-4):** concurrent same-key `start_session` can create
  duplicate sessions. Matches PARTIAL; fix in the actor-fixture follow-on.
- **R2/R11 projection depth:** only `AgentMessageChunk`/`UserMessageChunk` →
  `ItemDelta`/`ItemCompleted` are projected; full `Turn`/`ToolCall`/
  `Interaction` lifecycle deferred. Honest.
- **R6 archive semantics:** product decision pending; `unsupported` is the
  safest reversible stub (no silent `delete_session`). Honest.
- **R10 delivery channel:** `respond_interaction` returns `unsupported`; the
  parked-oneshot resolution mechanism is the C1 follow-on. Honest.

## Commands / results (as captured)
- `cargo test -p xai-grok-shell --test c1_shell_port` → 18 passed; 0 failed
  (`tests/c1/c1_shell_port.txt`).
- `cargo test -p xai-grok-pager-bin --lib composition` → 2 passed; 0 failed
  (×3 bin targets) (`tests/c1/composition.txt`).
- `cargo test -p xai-grok-shell --lib app_server_runtime` → **NOT captured**
  (F-1); independently statically verified PASS.
- Full `cargo test -p xai-grok-shell` / `-p xai-grok-pager-bin` → not captured
  as separate logs; compilation of both crates succeeded within the
  composition run.

## Checks skipped
- No command-execution tool available to this review subagent. Static
  analysis + captured-log review only. The two captured GREEN logs are
  authoritative for the integration and composition surfaces; the invariant
  guards are independently verified by source inspection (F-1).
