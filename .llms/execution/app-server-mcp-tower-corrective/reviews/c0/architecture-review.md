# C0-C — Independent architecture review (GLM, read-only)

| Field | Value |
|---|---|
| Reviewer | GLM `glm-5.2` (independent review subagent) |
| Wave | C0 item 6 |
| Mode | read-only architecture / GO-NO-GO gate for Wave C1 |
| Branch | `goblin-implement-epic-tree` |
| Inputs | adversarial audit, FINAL_REPORT, corrective contract, C0-A/B handoffs, tower/shell/composition code |
| Must NOT | implement fixes |

## 1. Verdict

**NO-GO for Wave C1.**

The ownership boundary itself is sound and the volatile hybrid authority has
been removed, but Wave C0 is **not complete**. Two of its three deliverables are
missing or inadequate, and the C1 implementer would have to invent the
SessionActor command mapping that C0-B was supposed to produce. Permitting C1-D
to start now would violate corrective contract §2 ("Do not mark a task
complete unless its literal acceptance criterion, named tests, required review
and ledger evidence all exist") and §4 Wave C0 item 6 ("resolve all Critical/High
findings before Wave C1").

One-liner: **NO-GO — boundary is clean and hybrid removed, but C0-A matrix is
missing, C0-B map is non-evidential, and the product composition root still
injects FakeRuntime; C1 cannot start without an evidence-backed command map.**

## 2. Findings

### CRITICAL

#### C-1 — C0-A requirement matrix was never produced
- **Severity:** Critical / **Confidence:** High
- **Evidence:** `waves/` contains only `c0-session-actor-map.md`; no
  `c0-requirement-matrix.md` exists
  (`/home/guilherme/github/grok-goblin/.llms/execution/app-server-mcp-tower-corrective/waves/`).
  `HANDOFF-C0-A-requirement-matrix.md:21` requires
  `waves/c0-requirement-matrix.md` with one row per v1 task ID and the status
  enum `OPEN | PARTIAL | BLOCKED | SKIP | HUMAN | PASS`. `STATUS.md:16` still
  lists C0-A as `running`. Corrective contract §4 Wave C0 item 2–4 makes the
  matrix a precondition for item 6 (this review).
- **Impact:** There is no reconciled truth surface mapping every v1 task to
  evidence/gap/next. The audit's F-02..F-05 reopen list (RF102-02/05, AS104
  network, AS105 persistence, MCP101-03, OR-02/GQ-02/CF-02 live smokes) is not
  reflected in any ledger artifact. A C1 implementer cannot tell which tasks
  are genuinely PASS vs PARTIAL, and the contract's "only PASS may remain `[x]`"
  rule is unenforced.
- **Required fix (for parent, not this reviewer):** Produce the matrix before
  any C1 spawn; reopen every unsupported `[x]` listed in the audit and the C0-A
  handoff.

#### C-2 — C0-B SessionActor command map is non-evidential and incomplete
- **Severity:** Critical / **Confidence:** High
- **Evidence:** `waves/c0-session-actor-map.md` is 22 lines. Its table column is
  literally "Likely existing authority" with vague prose ("ACP new session /
  leader session create", "interjection / steer", "updates JSONL + live
  subscriptions"). It cites **no** `file:fn` evidence, omits the required columns
  (`Message/command type`, `Persistence touch`, `Permission/interaction?`,
  `Test entrypoints`, `Risk`) mandated by `HANDOFF-C0-B-session-actor-map.md:32`,
  and does not document how one-actor-per-session is enforced, foreground-turn
  exclusivity, or the composition injection point. It does not list the actual
  `SessionCommand` variants that exist in the codebase.
- **Counter-evidence in code:** The real command surface is rich and citable:
  - `crates/codegen/xai-grok-shell/src/session/commands.rs:106` defines
    `pub enum SessionCommand` with `Prompt` (line 122), `Cancel` (line 572),
    `Interject` (line 672), `ReplaceSystemPrompt`, `Rewind`, `Shutdown`, etc.
  - `crates/codegen/xai-grok-shell/src/session/handle.rs:38` defines the
    `SessionHandle` proxy (`cmd_tx: mpsc::UnboundedSender<SessionCommand>`).
  - `crates/codegen/xai-grok-shell/src/session/acp_session.rs:564` defines
    `pub(crate) struct SessionActor` with `state: TokioMutex<State>`,
    `pending_interjections`, `current_prompt_id`, `pending_interactions`.
  - Storage authority: `session/storage/jsonl/mod.rs:166 list_sessions_sync`,
    `:198 list_sessions_recent`, `:1241 load_session`, `:1340 list_sessions`;
    trait `session/storage/mod.rs:629 load_session`, `:642 list_sessions`.
  - Fork: `session/fork.rs:65 fork_session`.
  - Resume: `session/worktree.rs:163 resume_session_in_worktree`.
- **Impact:** The C1-D implementer is instructed (`HANDOFF-C1-D-shell-port-impl.md:7`)
  to "map every facade method to existing leader/SessionActor commands" and to
  write RED tests first. Without an evidence-backed map, the implementer must
  reverse-engineer the command surface, which the corrective contract
  explicitly classifies as C0 characterization work (§4 item 5). Starting C1
  now would blur the C0/C1 boundary and risk inventing APIs — the exact failure
  mode the contract §2 forbids ("Do not invent the missing skill's contract";
  knowledge-integrity rule).
- **Required fix:** Re-run C0-B (or have the parent reconstruct it) with real
  `file:fn` citations for all 12 facade methods, the `SessionCommand` variant
  each maps to, the persistence touch, the permission/interaction path, and
  named existing tests. Mark `UNVERIFIED` where evidence is genuinely missing.

### HIGH

#### H-1 — Product composition root still injects FakeRuntime
- **Severity:** High / **Confidence:** High
- **Evidence:** `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs:11-16`
  imports `xai_grok_tower::FakeRuntime` and constructs
  `ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()))` in
  `experimental_app_server_processor()`. The module doc (lines 1-4) admits
  "Until SessionActor-backed `GrokRuntimeFacade` lands, tests inject FakeRuntime
  only." `ShellRuntimeAdapter` (`xai-grok-shell/src/app_server_runtime/mod.rs:39-57`)
  wraps an `inner: Arc<dyn GrokRuntimeFacade>` and delegates every method to it;
  the registry it maintains is an opaque `SessionRegistry<()>` token store, not
  actor residency.
- **Impact:** This is the known C1 blocker (audit F-01, FINAL_REPORT item 1,
  `BLOCKERS.md` C-SA). It is correctly *blocked* and not yet a regression. It is
  listed here because the handoff checklist asks to confirm "FakeRuntime only
  acceptable for unit/conformance, not product claims" — that criterion is **not
  yet satisfied**; the product path is still fake-backed. This is expected at
  C0 (C1-D owns the fix), so it does not by itself block C1, but it must be
  tracked as the first C1 deliverable.
- **Required fix:** C1-D must replace the composition root injection with a
  real Shell actor-backed adapter and run the shared conformance suite against
  both (corrective contract §4 Wave C1 items 7–13).

#### H-2 — RF102-02 / RF102-05 PASS claims remain unsupported
- **Severity:** High / **Confidence:** High
- **Evidence:** Audit F-02. `xai-grok-shell/src/app_server_runtime/mod.rs:219-261`
  `single_actor_owns_turn_mutation` injects `FakeRuntime` (line 220), spawns 8
  concurrent `start_turn` calls (lines 232-247), asserts `ok == 8` (line 249)
  and `registry_len() == 1` (line 251). The registry is `SessionRegistry<()>`
  (`mod.rs:36`); the test proves only that one opaque `u64` token exists per
  session id, not that one `SessionActor` owns turn mutation, not that each
  facade call sends one existing leader/`SessionActor` command, and not
  ordered mutations. `30-app-server/v1-02-runtime-facade-projection/tasks.md`
  marks RF102-02 and RF102-05 complete (per audit F-02).
- **Impact:** The named test overstates its evidence; the literal acceptance
  criteria for RF102-02 ("each facade call sends one existing
  SessionActor/leader command") and RF102-05 ("ordered mutations") are
  unproven. This is a C1 item (contract §4 item 9–10) but the false `[x]` must
  be reopened at C0 per the matrix deliverable.
- **Required fix:** Reopen RF102-02/05 in the C0-A matrix (status PARTIAL/BLOCKED);
  C1-D must replace the opaque-token proof with real actor identity/residency
  and command-routing evidence.

#### H-3 — Transport "server" claims are helpers only (F-03 not closed)
- **Severity:** High / **Confidence:** High
- **Evidence:** Audit F-03. `xai-grok-app-server/src/transport/websocket.rs:65-91`
  processes a supplied text value with no listener/handshake/ping-pong/
  disconnect. `xai-grok-mcp-server/src/transport/http.rs:43-77` is an in-memory
  cursor table and response constructor with no POST/GET/DELETE router or SSE
  lifecycle. These are C3/C4 work, but the affected tasks (AS104-01/05/06,
  MCP101-03) must be reopened at C0 so a future wave does not inherit false PASS.
- **Impact:** Out of scope for C1 implementation, but in scope for C0 truth
  reconciliation. Without the matrix, these remain silently checked.
- **Required fix:** C0-A matrix must mark these OPEN/PARTIAL with evidence
  "helper only, no listener" and next = "C3/C4 real transport".

### MEDIUM

#### M-1 — Tower instance selector uses wrong contract surface
- **Severity:** Medium / **Confidence:** High
- **Evidence:** `app_server_composition.rs:24` reads `GROK_TOWER_INSTANCE`; the
  canonical contract name is `GROK_OSS_TOWER` (audit F-07, corrective contract §4
  Wave C2 item 15). `select_tower_instance_id` returns arbitrary strings without
  `TowerInstanceId` validation (`app_server_composition.rs:19-30`); the
  fallback test (lines 78-83) asserts only non-empty because ambient env is
  uncontrolled. No `GROK_OSS_TOWER` reference exists anywhere in the repo (grep
  across `*.rs` returns no matches).
- **Impact:** C2 work, but the wrong-name/wrong-type selector is a latent
  contract break that C1 should not worsen. C1-D should not touch this surface;
  flag for C2.
- **Required fix:** C2 item 15–16; record in matrix as OPEN with next = C2.

#### M-2 — `project_active_session_row` hardcodes epoch/revision
- **Severity:** Medium / **Confidence:** High
- **Evidence:** `xai-grok-shell/src/app_server_runtime/mod.rs:139-156`
  synthesizes `history_epoch: "epoch_1"`, `revision: WireCounter::new(0)`, empty
  turns/items for any disk session. This is the same synthetic-counter pattern
  the audit flagged in F-13 (volatile hybrid); it survives in the list-path
  helper. Corrective contract §4 Wave C1 item 11 requires replacing hardcoded
  projection epoch/revision with canonical persisted/runtime values or removing
  the projection until available.
- **Impact:** A list of disk sessions will present fabricated epoch/revision
  to clients; replay cursors derived from them can violate cursor/revision
  semantics. C1-D must address this when wiring the real list path.
- **Required fix:** C1 item 11; record in matrix as PARTIAL.

#### M-3 — No `CHANGES.md` / `DECISIONS.md` in corrective ledger
- **Severity:** Medium / **Confidence:** High
- **Evidence:** `app-server-mcp-tower-corrective/` contains `STATUS.md`,
  `BLOCKERS.md`, `handoffs/`, `waves/`, `reviews/`, `tests/`. No `CHANGES.md`
  or `DECISIONS.md` (corrective contract §3 requires both). `STATUS.md` and
  `BLOCKERS.md` exist but are sparse.
- **Impact:** Resumable decision history is absent; the contract §3 ledger is
  incomplete before C1.
- **Required fix:** Parent creates `CHANGES.md` and `DECISIONS.md` and keeps
  them current per §3.

### LOW

#### L-1 — C0-B map gate line duplicates a product rule as if verified
- **Severity:** Low / **Confidence:** High
- **Evidence:** `waves/c0-session-actor-map.md:21` "No production composition may
  inject FakeRuntime. Fake remains unit/conformance only." — stated as a gate
  but the map provides no evidence; the composition root contradicts it today
  (H-1).
- **Impact:** Minor; the rule is correct, but presenting it under a "Gate"
  heading in a characterization doc is misleading until C1-D lands.
- **Required fix:** Move the rule to a "Preconditions" section with explicit
  "not yet satisfied" status.

## 3. Checkbox honesty — PASS claims still false

Per audit F-10 and the C0-A mandate, the following `[x]` claims are not
supported by production code + non-vacuous tests and must be reopened (status
≠ PASS) in the still-missing C0-A matrix:

| Task | Claimed | Actual evidence | Correct status |
|---|---|---|---|
| RF102-02 | done | `single_actor_owns_turn_mutation` injects FakeRuntime; opaque token only (H-2) | PARTIAL |
| RF102-05 | done | no ordered-mutation test against real actor (H-2) | PARTIAL |
| AS104-01/05/06 | done | `websocket.rs` is a text-value helper, no listener (H-3) | PARTIAL |
| MCP101-03 | done | `http.rs` cursor table + response ctor, no router/SSE (H-3) | PARTIAL |
| AS105-01..07 | done | tests named `..._for_fake` / `..._on_fake`; no canonical-file rebuild (audit F-06) | PARTIAL |
| OR-02/GQ-02/CF-02 live smokes | done (SKIP) | SKIP without credentials checked as `[x]` (audit F-04) | SKIP (never `[x]`) |

The audit (F-10) observed 46 checked vs 8 open before v2/out-of-scope separation;
the matrix must reconcile all of these, not just the rows above.

## 4. Preconditions that must be true before C1 coding starts

1. **C0-A matrix exists** at `waves/c0-requirement-matrix.md` with every in-scope
   v1 task ID labeled `OPEN | PARTIAL | BLOCKED | SKIP | HUMAN | PASS`, evidence
   and next columns, and every unsupported `[x]` reopened. (C-1)
2. **C0-B map is evidence-backed** at `waves/c0-session-actor-map.md` (or a
   renamed `c0-session-actor-command-map.md`) with `file:fn` citations for all
   12 facade methods, the `SessionCommand` variant each maps to, persistence
   touch, permission/interaction path, existing test entrypoints, and risk; with
   `UNVERIFIED` markers where evidence is genuinely missing. (C-2)
3. **Hybrid runtime stays removed.** Confirmed today: grep for
   `SessionStorageHybridRuntime` across `crates/` returns no matches. A
   regression test (corrective contract §4 item 3a) that a listed session is
   readable, mutable and replayable through the *same* runtime should be added
   as the first C1 RED test.
4. **`CHANGES.md` and `DECISIONS.md` exist** in the corrective ledger and are
   current. (M-3)
5. **C1-D handoff is re-confirmed** with the corrected map as an input, so the
   implementer does not invent the command surface.
6. **No concurrent writers** on `xai-grok-shell/src/app_server_runtime/**` or
   `xai-grok-pager-bin/src/app_server_composition.rs` while C1-D runs
   (`HANDOFF-C1-D-shell-port-impl.md:25` exclusive-writer list).

## 5. Out of scope reminders

- **70-goal-runtime, 80-channel-gateways, 90-realtime-voice** are excluded
  from the C0 matrix unless referenced by an in-scope task (C0-A handoff §
  Authority). Do not pull them into C1.
- **Dashboard migration (30/v2-01)** and UI freeze remain out of scope; the
  composition root must not grow dashboard coupling.
- **Real transports (WebSocket listener, MCP HTTP router)** are C3/C4, not C1.
  C1-D must not implement them; it must only stop injecting FakeRuntime and
  route facade methods to the existing actor.
- **Provider verticals** are C5; C1 must not touch `providers/byok` or login.
- **HUMAN gates** (TLS/threat, npm publish, live credentials, PC8 live pack) are
  recorded separately and never auto-PASS.

## 6. Required next implementer actions (for parent, not this reviewer)

These are actions for the parent orchestrator to dispatch; this review does not
implement them.

1. Re-dispatch (or reconstruct) **C0-A** to produce the requirement matrix and
   reopen the unsupported `[x]` rows listed in §3.
2. Re-dispatch (or reconstruct) **C0-B** to produce an evidence-backed command
   map with `file:fn` citations. At minimum the map must cite:
   `SessionCommand::{Prompt, Cancel, Interject, ...}` at
   `session/commands.rs:106`, `SessionHandle` at `session/handle.rs:38`,
   `SessionActor` at `session/acp_session.rs:564`, storage at
   `session/storage/jsonl/mod.rs:1241/1340` and `session/storage/mod.rs:629/642`,
   fork at `session/fork.rs:65`, resume at `session/worktree.rs:163`.
3. Create `CHANGES.md` and `DECISIONS.md` in the corrective ledger.
4. After C0-A and C0-B are complete and this review's Critical findings closed,
   re-run this GO/NO-GO gate (or have a fresh independent reviewer do so) before
   spawning C1-D.
5. Only on a fresh **GO**, spawn C1-D with the corrected map as a required
   input and the regression test from precondition 3 as its first RED test.

## 7. Residual risk

- The ownership boundary is clean (Tower does not depend on Shell; Tower defines
  no `SessionActor`; no hybrid split authority). That is the genuine C0
  architectural achievement and it survives this NO-GO.
- The dominant residual risk is **procedural**: starting C1 without the C0-A
  matrix and C0-B map would let the implementer invent the command surface and
  re-introduce the "partial work presented as complete" failure mode the
  corrective program was created to fix.
- A secondary risk is the surviving `project_active_session_row` synthetic
  counters (M-2); if C1-D reuses it for the real list path, cursor/revision
  semantics can break silently.

## 8. Commands / evidence inspected

- Read: `HANDOFF-C0-C-architecture-review.md`, `HANDOFF-C0-A-requirement-matrix.md`,
  `HANDOFF-C0-B-session-actor-map.md`, `HANDOFF-C1-D-shell-port-impl.md`,
  `HANDOFF-C1-E-code-review.md`, `HANDOFF-C1-F-test-review.md`, `README.md`.
- Read: `app-server-mcp-tower-adversarial-audit-2026-07-18.md`,
  `app-server-mcp-tower/FINAL_REPORT.md`, corrective contract
  `20260718-correct-app-server-mcp-tower-execution.md`, `STATUS.md`,
  `BLOCKERS.md`, `waves/c0-session-actor-map.md`.
- Read: `_shared/runtime-facade.md`, `_shared/runtime-ownership.md`,
  `_shared/crate-map.md`, `_shared/security-authority-boundaries.md`.
- Read code: `xai-grok-tower/src/lib.rs`, `xai-grok-shell/src/app_server_runtime/mod.rs`,
  `xai-grok-pager-bin/src/app_server_composition.rs`,
  `xai-grok-shell/src/session/commands.rs`, `session/handle.rs`,
  `session/acp_session.rs` (lines 560-680).
- Grep: `SessionStorageHybridRuntime` (no matches — hybrid removed),
  `FakeRuntime` in pager-bin (matches at `app_server_composition.rs:3,11,15`),
  `xai-grok-shell` in tower (only the test assertion at `lib.rs:120`),
  `struct SessionActor`/`enum SessionActor` in tower (none in production),
  `GROK_OSS_TOWER`/`GROK_TOWER_INSTANCE` (only the latter, at
  `app_server_composition.rs:24`).
- Tests **not** re-run by this reviewer (read-only architecture review; the
  audit's 91-test green result is cited as prior evidence, not re-validated
  here). The parent should re-run the gate after C0-A/B land.

## 9. Separated verdicts

- `IMPLEMENTATION_OR_ARTIFACT: FAIL` — C0-A matrix missing (C-1); C0-B map
  non-evidential (C-2); product composition still FakeRuntime-backed (H-1,
  expected at C0 but blocks the "FakeRuntime only for unit/conformance"
  criterion).
- `AGENT_BEHAVIOR: FAIL` — C0-A and C0-B deliverables were not produced to
  their handoff contracts; STATUS.md still marks both `running`; the corrective
  ledger is missing `CHANGES.md`/`DECISIONS.md` (M-3).
- `HANDOFF_QUALITY: PASS` — the C0-C handoff itself is clear, complete, and
  correctly scoped read-only; inputs and deliverable structure are
  well-specified.
- `GOAL_GATE: NO-GO` — Wave C0 acceptance (contract §4 items 2, 3, 3a, 4, 5, 6)
  is not met: matrix absent, command map inadequate, unsupported `[x]` not
  reconciled, and this review found Critical findings that must be resolved
  before Wave C1.

---

**One-liner: NO-GO for Wave C1.** The ownership boundary is clean and the
hybrid split authority was removed, but C0-A's requirement matrix is missing,
C0-B's command map is non-evidential, and the product composition root still
injects FakeRuntime — C1-D must not start until C0-A and C0-B are complete and
a fresh GO is issued.
