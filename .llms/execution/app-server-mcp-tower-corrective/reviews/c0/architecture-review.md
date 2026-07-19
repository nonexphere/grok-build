# C0-C Architecture Review — Wave C1 GO/NO-GO (GLM, read-only)

| Field | Value |
|---|---|
| Reviewer | GLM `glm-5.2` (independent, read-only) |
| Wave | C0 item 6 (architecture review) → gates Wave C1 |
| Mode | `implementation` review (no product code mutation) |
| Branch audited | `goblin-implement-epic-tree` |
| Inputs | HANDOFF-C0-C, c0-requirement-matrix.md, c0-session-actor-command-map.md (full), adversarial audit 2026-07-18, `app_server_composition.rs`, `app_server_runtime/mod.rs`, plus independent grep verification of Tower/Shell/hybrid invariants |
| Date | 2026-07-18 |

## 1. Verdict

**GO for Wave C1** — conditional on the preconditions in §5. No Critical or High
finding blocks C1-D from starting implementation against the C0-B map. The
C0-B SessionActor/leader command map is evidence-backed enough that C1-D can
implement every facade method by forwarding to a **real, existing** Shell
symbol without inventing a second `SessionActor`, a second permission engine,
a second replay buffer, or a hybrid Fake+JSONL authority.

The two architectural blockers from the adversarial audit that *would* have
blocked C1 are resolved at the evidence level:

- **F-13 (hybrid `SessionStorageHybridRuntime` split authority): NOT PRESENT.**
  `rg SessionStorageHybridRuntime crates/` returns no matches. The volatile
  post-snapshot dirty state described in the audit was not merged. The
  composition root is honestly Fake-only (see §3, finding A1).
- **F-01 (composition injects `FakeRuntime`): STILL PRESENT, but it is exactly
  the C1-D work surface.** `app_server_composition.rs:15` constructs
  `ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()))`. This is the
  documented switch point, not a regression. C1-D replaces the inner
  `Arc<dyn GrokRuntimeFacade>` with a real Shell-owned port; the outer
  `ShellRuntimeAdapter` wrapper may remain.

## 2. Independent invariant verification (re-checked this turn)

| Invariant | Evidence | Status |
|---|---|---|
| Tower has no Shell dependency | `crates/codegen/xai-grok-tower/Cargo.toml` — `rg xai-grok-shell` returns no matches; guard at `xai-grok-tower/src/lib.rs:118-122` asserts it | PASS |
| Tower defines no `SessionActor` type | `rg "struct SessionActor\|enum SessionActor"` over `crates/codegen/xai-grok-tower` matches only the *test* `leader_characterization_tower_has_no_second_actor_type` at `lib.rs:95`; production sources (split on `#[cfg(test)]`) are asserted clean at `lib.rs:101-114` | PASS |
| The only real `SessionActor` lives in Shell | `crates/codegen/xai-grok-shell/src/session/acp_session.rs:564` `pub(crate) struct SessionActor` | PASS |
| No hybrid Fake+JSONL authority exists | `rg SessionStorageHybridRuntime crates/` → no matches | PASS |
| Composition root is Fake-only (not hybrid) | `app_server_composition.rs:15` injects `FakeRuntime::new()`; module doc lines 1-4 explicitly forbids mixing real storage list with fake mutation | PASS (honest block, not a violation) |
| `ShellRuntimeAdapter` registry is opaque tokens only | `app_server_runtime/mod.rs:38-39` `registry: Mutex<SessionRegistry>`; `get_or_insert_with` at lines 70-72, 79-82, 88-90 records `Ok(())` tokens, not actor state | PASS |
| `project_active_session_row` is a dormant stub, not a real projection | `app_server_runtime/mod.rs:137-158` hardcodes `history_epoch:"epoch_1"`, `revision: WireCounter::new(0)`, `status: Dormant` | PASS (correctly flagged as stub; C1-D must replace per C0-B §1.1 R1) |

## 3. Findings (Critical / High / Medium / Low)

### A1 — Medium / High confidence — Composition root still injects FakeRuntime (expected C1-D surface)
**Evidence:** `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs:15`
`let adapter = ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()));`
plus `app_server_runtime/mod.rs` `impl GrokRuntimeFacade for ShellRuntimeAdapter`
(lines 56-132) where every method delegates to `self.inner.*` (the Fake).
**Impact:** This is the primary C1-D work surface, not a regression. It is
correctly documented in both module headers and the C0-B map §4. No action
required before C1 starts; C1-D must replace the inner port.
**Severity rationale:** Medium (not Critical) because it is the *declared*
scope of C1-D, the codebase is honest about it, and no product claim depends on
the fake path today (RF102-02/05 are OPEN/PARTIAL in the matrix, not PASS).

### A2 — Medium / High confidence — `project_active_session_row` dormant stub must not survive into C1
**Evidence:** `app_server_runtime/mod.rs:137-158` hardcodes `epoch_1` /
revision 0 / `Dormant`. C0-B §1.1 R1 explicitly calls this out: the real
`list_sessions` must call `JsonlStorageAdapter::list_sessions`
(`session/storage/jsonl/mod.rs:1340`/`:166`) and project `Summary` → protocol
`Session`, not this stub.
**Impact:** If C1-D leaves this stub in place, `list_sessions` returns
synthetic metadata and `historyEpoch` semantics break (audit F-13 root cause
was a similar synthetic epoch). C1-D RED test
`c1_real_adapter_list_sessions_reads_jsonl_summaries_not_dormant_stub`
(C0-B §9 #1) must fail against this stub and force its removal.
**Required fix (C1-D):** delete or replace `project_active_session_row`; do
not retain it as a fallback path.

### A3 — Medium / High confidence — `GROK_TOWER_INSTANCE` env name still uncanonical
**Evidence:** `app_server_composition.rs:24` reads `std::env::var("GROK_TOWER_INSTANCE")`.
Global `AGENTS.md` / `GOBLIN.md` canonical name is `GROK_OSS_TOWER`
(C0-B §4 M-1; audit F-07; matrix TW103-02 PARTIAL). No `TowerInstanceId`
validation is applied to the returned string.
**Impact:** Invalid IDs cross the composition boundary; behavior depends on an
undocumented env var. Not a C1-D blocker (C1-D owns the runtime port, not the
selector), but C1-D must not *extend* this surface. Flag for Wave C2-15/16.
**Required fix (C2, not C1):** rename to `GROK_OSS_TOWER`, validate via
`TowerInstanceId`, hermetic precedence tests.

### A4 — Low / High confidence — `single_actor_owns_turn_mutation` test still Fake-backed and over-named
**Evidence:** `app_server_runtime/mod.rs:219-261` injects `FakeRuntime`,
spawns 8 concurrent `start_turn`s, asserts all 8 succeed (`ok == 8`), and
checks `registry_len() == 1`. The test name claims "single actor owns turn
mutation" but proves only that one opaque registry token exists and the Fake
allows 8 concurrent starts.
**Impact:** This is the F-02 finding, still live. It does not block C1 (the
matrix correctly has RF102-05 as OPEN and TW101-04 as PARTIAL). C1-D must add
the real-adapter RED/GREEN tests from C0-B §9 (#8, #9, #18) and not treat
this Fake test as evidence of foreground-turn exclusivity.
**Required fix (C1-D):** add real-actor exclusivity test; do not delete the
Fake conformance test (it remains valid as a Fake-behavior contract per
RF102-06 PASS).

### A5 — Low / Medium confidence — `start_turn` registry auto-insert is permissive
**Evidence:** `app_server_runtime/mod.rs:99-116` — if `registry.get(session_id)`
is `None`, it silently inserts a token via `get_or_insert_with` before
delegating `start_turn` to the inner port. This means a `start_turn` for an
unknown session id does not return `session_not_found` at the adapter layer;
it relies on the inner port to reject.
**Impact:** With `FakeRuntime` inner, the Fake likely accepts any session id.
C1-D's real port must reject unknown sessions (the real `MvpAgent::prompt`
path requires a registered `SessionHandle`). The adapter's silent auto-insert
could mask a missing rejection if the real port is permissive. Low risk
because the real port is the authority, but C1-D should verify the
unknown-session rejection path (RED test recommended).
**Required fix (C1-D):** confirm real port returns `session_not_found` for
unknown session ids; consider removing the silent auto-insert or making it
conditional on a successful inner-port result.

## 4. Checkbox honesty — which PASS claims are still false

Re-confirming the audit's reopen set against the current matrix
(`c0-requirement-matrix.md`):

- **RF102-02 / RF102-05: OPEN** (not PASS). The adapter delegates to Fake;
  real SessionActor command routing and foreground-turn exclusivity are not
  implemented. **Confirmed still false.** C1-D owns the fix.
- **TW101-04 / TW102-03: PARTIAL.** `ShellRuntimeAdapter` list/read seam
  exists but maps to Fake, not `JsonlStorageAdapter`/leader roster.
  **Confirmed still false as PASS.**
- **AS106-05: PARTIAL.** `respond_interaction` delegates to Fake; not mapped
  to `PendingInteractionGuard`/leader routing. **Confirmed still false.**
  C1-D §1.3 R10 owns the resolution channel design.
- **TA101-02: OPEN.** Facade methods incomplete; shell adapter fake-backed.
  **Confirmed still false.**
- **AS104-01/05/06, MCP101-03, MCP102-05: OPEN/PARTIAL.** Transport "server"
  claims are helpers only (audit F-03). These are **out of scope for C1**
  (Wave C3/C4) but remain correctly non-PASS.
- **TW103-02: PARTIAL.** `GROK_TOWER_INSTANCE` vs canonical `GROK_OSS_TOWER`
  (finding A3). **Confirmed still false.** Wave C2 owns the fix.

No PASS checkbox in the matrix contradicts its evidence for the C1-scoped
rows. The matrix's reopen of 19 previously-`[x]` rows to PARTIAL is honest
and matches the code.

## 5. Preconditions that must be true before C1-D coding starts

Only these remain; all are documentation/decision preconditions, not code
gates (C1-D writes the code):

1. **R6 archive_session product decision (C0-B §1.1 R6 / §8 #3).** The only
   existing destructive symbol is `StorageAdapter::delete_session`
   (`session/storage/mod.rs:647`, irreversible). `close_session_explicit`
   (`session_lifecycle.rs:58`) keeps disk. Mapping `archive` → `delete` is
   data loss; adding a `hidden` flag is a `Summary` schema change. **A product
   decision is required before C1-D implements `archive_session`.** Default
   must not be silent `delete_session`. If undecided by C1 start, C1-D must
   implement the safest reversible interpretation (hide flag or no-op stub
   returning `unsupported`) and record the decision in `waves/c1-shell-port.md`.
2. **R10 interaction resolution channel design (C0-B §1.3 R10 / §8 #2).**
   `respond_interaction` has no existing `SessionCommand`. The parked oneshot
   is resolved via the leader's ACP response forwarding
   (`leader/server.rs:492`), not a session command. C1-D must design a
   resolution channel that reuses `PendingInteractionGuard`'s registry
   (`session/pending_interaction.rs:80-145`) keyed by `tool_call_id` (mapping
   `interaction_id` → `tool_call_id`) **without re-evaluating permission
   policy**. The design must be written into `waves/c1-shell-port.md` before
   the `respond_interaction` RED test is greened.
3. **R11 replay `RuntimeEvent` projection design (C0-B §1.3 R11 / §8 #1).**
   No existing `updates.jsonl` line → `RuntimeEvent` projector exists. C1-D
   must build one (the largest new code surface) that reuses
   `MvpAgent::replay_session_updates` (`mvp_agent/mod.rs:1446`) and
   `session::replay_events::SessionNotification` as the source, projecting
   into `RuntimeEvent::{SessionChanged, TurnChanged, ItemStarted, ItemDelta,
   ItemCompleted, InteractionRequested}` (tower `lib.rs:74-86`) with cursor
   pagination over `after_event_seq`/`WireCounter`. **Must not introduce a
   second replay buffer.** Design sketch required in `waves/c1-shell-port.md`.
4. **R2 read_session Turn/Item projection design (C0-B §1.1 R2).**
   `SessionReadResult` carries `Vec<Turn>` + `Vec<Item>`; Shell `Summary`/
   `PersistedData` has no first-class `Turn`/`Item`. C1-D must project from
   `updates.jsonl` (same parser surface as R11, but synchronous and
   paginated). Reuse the R11 projector where possible to avoid two parsers.
5. **C1-D must run RED tests first** (C0-B §9, 20 names) against a real
   `JsonlStorageAdapter` fixture (TempDir) + a real spawned `SessionActor`
   via `spawn_session_on_thread` (`session/acp_session_impl/spawn.rs:1662`).
   All RED before any GREEN. Per-behavior RED/GREEN evidence must be captured
   (audit F-09; matrix TW101-05/RF102-07/AS103-07/TA101-06 PARTIAL).
6. **Composition root switch is the terminal C1-D step**, not the first.
   Build the real `ShellSessionActorRuntime: GrokRuntimeFacade` behind RED
   tests first; switch `app_server_composition.rs:15` from `FakeRuntime` to
   the real port only after GREEN. Keep `FakeRuntime` for unit/conformance
   only (RF102-06 PASS). Do not introduce a hybrid at any intermediate step
   (audit F-13; the current clean state must be preserved).

## 6. Is C0-B evidence-backed enough that C1-D can implement without inventing SessionCommand APIs?

**Yes.** Every one of the 11 facade methods in C0-B §1 maps to a real,
existing Shell symbol with `file:fn` evidence:

| Facade method | Existing Shell symbol (file:fn) | Invent new SessionCommand? |
|---|---|---|
| `list_sessions` | `JsonlStorageAdapter::list_sessions` `jsonl/mod.rs:1340`/`:166` | No |
| `read_session` | `StorageAdapter::load_session`/`load_summary` `storage/mod.rs:628-644`; `persistence::load_light` `persistence.rs:2369` | No (but new Turn/Item projector — R2) |
| `start_session` | `MvpAgent::new_session` `acp_agent.rs:853`; `spawn_and_register_session` `agent_ops.rs:2911` | No |
| `resume_session` | `MvpAgent::load_session` `acp_agent.rs:1239`; `drain_old_session_thread` `agent_ops.rs:1728` | No |
| `fork_session` | `session::fork::fork_session` `fork.rs:66` | No |
| `archive_session` | `StorageAdapter::delete_session` `storage/mod.rs:647` / `close_session_explicit` `session_lifecycle.rs:58` | No (but product decision — R6) |
| `start_turn` | `MvpAgent::prompt` `acp_agent.rs:2017` → `SessionCommand::Prompt` `commands.rs:113` | No |
| `steer_turn` | `SessionCommand::Interject` `commands.rs:669` | No |
| `interrupt_turn` | `MvpAgent::cancel` `acp_agent.rs:3094` → `SessionCommand::Cancel` `commands.rs:566` | No |
| `respond_interaction` | `PendingInteractionGuard` `pending_interaction.rs:80-145`; leader routing `leader/server.rs:492` | **Possibly** a new `SessionCommand::ResolveInteraction` or direct channel — but this reuses the existing pending registry, not a second permission engine (R10) |
| `replay` | `MvpAgent::replay_session_updates` `mvp_agent/mod.rs:1446`; `replay_events::SessionNotification` `replay_events.rs:14` | No (but new `RuntimeEvent` projector — R11) |

The only place a *new* `SessionCommand` variant may be needed is
`respond_interaction` (R10), and even there the C0-B map is explicit that it
must reuse `PendingInteractionGuard`'s registry — it is a *delivery channel*,
not a second permission/elicitation engine. The two "new code" surfaces (R2
Turn/Item projector, R11 `RuntimeEvent` projector) are projections over
existing `updates.jsonl` data, not new actor commands.

**Conclusion: C1-D can implement every facade method by composing existing
Shell symbols. The UNVERIFIED markers (R2, R4, R5, R6, R7, R8, R9, R10, R11)
are adapter-side design decisions and projections, not inventions of
SessionActor APIs. No second actor, second permission engine, second replay
buffer, or hybrid authority is required or permitted.**

## 7. Out of scope reminders (do not pull into C1)

- **70/80/90 provider verticals** (OpenRouter/Groq/Cloudflare): Wave C5.
  C1-D must not wire provider inference; `provider_binding` flows through
  `SessionStartParams` to the real port unchanged.
- **Real WebSocket listener / Streamable HTTP MCP server** (audit F-03;
  AS104-01/05/06, MCP101-03, MCP102-05): Wave C3/C4. C1-D does not touch
  transport.
- **Dashboard migration / dual-OS-process leader flock** (TW103-03): Wave
  C2-17.
- **Canonical history rebuild over persisted sessions** (AS105-01..07,
  audit F-06): Wave C3-22/23. C1-D's `replay` projects `updates.jsonl` for a
  *live* session via the actor; durable crash/rebuild is later.
- **TLS / threat acceptance for non-loopback** (AS104-HUMAN, AS107-HUMAN,
  MCP102-HUMAN): HUMAN gates, never PASS without sign-off.
- **npm publish** (TS101-HUMAN, `TO_RELEASE_NPM.md`): external `NPM_TOKEN`,
  HUMAN gate.

## 8. Required fixes, residual risk, commands/results

### Required fixes (C1-D, not pre-C1)
- Replace `FakeRuntime` inner port at `app_server_composition.rs:15` with a
  real Shell-owned `GrokRuntimeFacade` impl after RED tests pass.
- Delete or replace `project_active_session_row` (`app_server_runtime/mod.rs:137-158`);
  do not retain the dormant `epoch_1` stub as a fallback (finding A2).
- Add real-adapter RED/GREEN for `respond_interaction`, `replay`,
  `read_session` Turn/Item projection, `list_sessions` JSONL projection
  (C0-B §9 #1, #2, #14, #15, #16, #17).
- Resolve R6 archive semantics before implementing `archive_session`
  (precondition §5.1).
- Do not introduce a hybrid Fake+JSONL authority at any intermediate step
  (audit F-13; current clean grep state must be preserved).

### Residual risk
- **R11 replay projection** is the highest-risk new surface — largest new
  code, easiest to accidentally second-engineer. C1-D must keep
  `replay_session_updates` as the single source.
- **R10 interaction channel** risks building a second permission engine if
  the adapter re-evaluates policy instead of just completing the parked
  oneshot. Design review at GREEN.
- **Idempotency-key dedup** (R3/R7/R8/R9) is UNVERIFIED in Shell's existing
  `new_session`/`prompt`/`Interject`/`Cancel`. C1-D must implement dedup in
  the adapter without weakening `dispatch_lock` exclusivity.
- **`InputBlock` ≠ `ContentBlock`** wire-shape conversion (R7) is a concrete
  translation risk at the `start_turn` boundary.

### Commands / results executed by this review
- `rg SessionStorageHybridRuntime crates/` → no matches (F-13 clean).
- `rg "struct SessionActor|enum SessionActor"` over `crates/codegen/xai-grok-tower` → only the test guard at `lib.rs:95`; production sources clean.
- `rg "pub(crate) struct SessionActor|struct SessionActor"` over `crates/codegen/xai-grok-shell/src/session` → `acp_session.rs:564` (single real actor).
- `rg xai-grok-shell` over `crates/codegen/xai-grok-tower/Cargo.toml` → no matches (no Shell dependency).
- `rg FakeRuntime` over `app_server_composition.rs` → lines 3, 11, 15 (composition root is Fake-only, honestly documented).
- Read `xai-grok-tower/src/lib.rs:85-130` (leader characterization guard).
- Read `app_server_composition.rs` and `app_server_runtime/mod.rs` in full.

No build/test commands were run; this is a read-only architecture review and
the invariants are statically provable by grep + source inspection. C1-D's
own `./scripts/run-rust-test-gate.sh` and package-scoped `cargo test` are the
authoritative gate at GREEN time.

## 9. One-line GO/NO-GO

**GO for Wave C1.** C0-B is evidence-backed: all 11 facade methods map to real
existing Shell symbols, no SessionCommand API invention is required (only
adapter-side projections and one delivery channel for `respond_interaction`),
the F-13 hybrid is absent, and the remaining preconditions are product
decisions (R6 archive) and adapter design sketches (R10/R11/R2), not code
blockers. C1-D may start RED tests immediately after recording the R6
decision and the R10/R11/R2 design sketches in `waves/c1-shell-port.md`.
