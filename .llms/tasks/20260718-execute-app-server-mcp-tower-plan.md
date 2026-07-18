# Execute the grok-oss App Server, Tower, MCP, Tools, SDK, and Provider Plan

## Goal Metadata

- Goal type: `implementation-program`
- Version: `20260718-architecture-v2`
- Owner/repo: `nonexphere/grok-build`
- Local path: `/home/guilherme/github/grok-goblin`
- Primary branch: `goblin`
- Task file: `/home/guilherme/github/grok-goblin/.llms/tasks/20260718-execute-app-server-mcp-tower-plan.md`
- Expected duration: `multi-day / multi-PR`

## Objective

Implement the approved `.llms/grok-build/` epic tree through the complete v1
release path for provider foundations, Tower, App Server, Tower Agent tools,
MCP control plane, and TypeScript SDK. Preserve the existing `SessionActor`,
leader/ACP wire behavior, dashboard, session files, provider request binding,
and fork branch policy. Deliver real vertical behavior—not additional empty
scaffolds—using strict RED/GREEN/REFACTOR, independent subagent reviews, and
non-vacuous automated validation after every bounded change and wave.

The implementation target is experimental protocol
`2026-07-18.experimental-v2`. Do not silently preserve the superseded v1 wire
shapes for `ProviderBinding`, wire counters, replay pages, or subscriptions.

## Success Criteria

- Every unblocked checkbox in the in-scope epic `tasks.md` files is implemented,
  tested, checked, and updated with durable evidence.
- Every D-* requirement assigned to an in-scope epic is satisfied by code,
  contract tests, or a precisely recorded HUMAN/external blocker.
- There remains exactly one authoritative runtime: the existing
  `xai-grok-shell` leader/`SessionActor` path.
- `xai-grok-pager-bin` is the composition root and injects the Shell runtime
  adapter into Tower/App Server/tools; Tower never imports Shell.
- Session/Turn/Item, structured `ProviderBinding`, decimal-string wire counters,
  replay, Interaction, idempotency, controller leases, and failure semantics
  pass Rust/schema/TypeScript conformance.
- The exact nine Tower Agent tools work through one semantic core in-process
  and through MCP, with fail-closed ACL and no local self-MCP loop.
- The first complete vertical slice works locally through in-process and stdio
  before WebSocket or remote MCP is accepted.
- WebSocket, MCP stdio/Streamable HTTP, SDK, reconnect, replay and security
  surfaces pass their conformance suites before release hardening.
- Filtered test commands cannot pass by executing zero tests.
- Package checks, relevant workspace checks, CLI build, black-box conformance,
  schema drift, security tests, and smoke tests pass.
- Every implementation wave receives an independent architecture/code review
  and an independent test review by subagents that did not author that wave.
- Product PRs target `goblin`, never `main`; `main` remains an upstream mirror.
- Final documentation and status claims match observed behavior and validation.

## Target And Context

- Repository/path: `/home/guilherme/github/grok-goblin`
- Canonical plan root: `/home/guilherme/github/grok-goblin/.llms/grok-build`
- Root execution order: `.llms/grok-build/README.md`
- Requirement matrix: `.llms/grok-build/_shared/INDEX.md`
- Architecture corrections: `.llms/grok-build/_shared/ARCHITECTURE_CORRECTIONS.md`
- Traceability: `.llms/grok-build/TRACEABILITY.md`
- TDD rules: `.llms/grok-build/TDD.md`
- Repository governance: `AGENTS.md`, `GOBLIN.md`, `task.md`
- Runtime authority: `_shared/runtime-ownership.md`,
  `_shared/runtime-facade.md`, `_shared/source-of-truth.md`
- Crate boundaries: `_shared/crate-map.md`
- Identity/protocol: `_shared/session-turn-item-identity.md` and
  `30-app-server/v1-01-session-protocol/contracts/`
- Tower lifecycle: `_shared/tower-instance-lifecycle.md`
- Tools: `_shared/tower-agent-tools.md`
- Security: `_shared/control-plane-security.md`
- MCP/CLI: `_shared/mcp-server-transport-cli.md`
- SDK: `_shared/typescript-sdk.md`
- Provider behavior: `_shared/provider-contract.md`, `GOBLIN.md`, `task.md`
- Existing runtime evidence: `crates/codegen/xai-grok-shell/src/leader/`,
  `crates/codegen/xai-grok-shell/src/session/`, session registry and ACP paths.
- Existing scaffolds: six new Rust crates under `crates/codegen/` and
  `packages/grok-oss-app-server/`.
- External systems/checks: local Cargo/npm toolchains; GitHub fork and CI when
  pushing PRs; live provider tests only when credentials are explicitly present.

Read every applicable `AGENTS.md` and the complete files above before editing.
For each epic, reread its README, contracts, SPECS if present, and `tasks.md` at
the start of the epic. Never execute from this task summary alone.

## Operating Mode

- Continue until all in-scope unblocked work meets the stop conditions or a
  genuine listed blocker is reached.
- Prefer the next safe action when contracts and dependencies are explicit.
- Ask the human only for the HUMAN decisions enumerated below or a newly proven
  product/security/ownership ambiguity with materially different outcomes.
- Maintain a live execution ledger at
  `.llms/execution/app-server-mcp-tower/STATUS.md`.
- After compaction or a new session, reread this task file, STATUS, the current
  epic README/tasks, the current branch diff, and the last review artifacts.
- Do not mark an epic or wave complete because code compiles; require its
  observable behavior, tests, review, and documentation gates.
- If this file is missing, unreadable, or internally conflicting with more
  authoritative repository governance, report `BLOCKED` with evidence.

## Invariants

- `AGENTS.md` and more-local instructions always win.
- `main` mirrors `origin/main`; all product work and PRs target `goblin`.
- Never use `git reset --hard`, force-checkout, or stash another actor’s WIP.
- Use isolated worktrees for rebases, conflict resolution, parallel feature
  branches, or any operation that could disturb the shared worktree.
- Do not stage or commit unrelated files.
- Preserve public/legacy behavior unless the experimental-v2 contract explicitly
  changes it or a migration is approved.
- Existing `SessionActor` and canonical session files remain execution truth.
- App Server, MCP, tools, SDK, projections and databases remain adapters/indexes.
- `xai-grok-pager-bin` is the composition root. Tower does not depend on Shell.
- No second actor, duplicate registry authority, `tower_agent_hub`, self-MCP
  injection, native public `Thread`, or Goal v2 coupling is permitted.
- `ProviderBinding` contains identifiers only and is immutable for an in-flight
  request/Turn; no bearer/provider secret may cross protocol projection.
- Wire revisions/event sequences/cursors are canonical decimal strings.
- Replay is bounded/paged; lifecycle events are never silently dropped.
- Multiple subscriptions for one Session are keyed by `subscriptionId`.
- No arbitrary product Session cap is introduced, but measured safety budgets
  must protect residents, Turns, pending loads, queues, FDs, memory and disk.
- Loopback is the default. Non-loopback cleartext is only
  `experimental/unsafe`; public production readiness requires TLS termination
  plus the HUMAN security gate.
- Every behavior change begins with a failing real test for the expected reason.
- A filtered Rust test gate must prove at least one matching test ran.
- Generated artifacts are changed only through their generator/source pipeline.
- Secrets, credentials and private data never enter logs, fixtures or commits.

## Allowed Actions

- Read and modify files inside the repository required by the current epic.
- Add behavior, contract, integration, property, black-box and E2E tests.
- Create isolated worktrees and `goblin-*` feature branches based on `goblin`.
- Commit coherent reviewed slices, push them to `fork`, and open draft/product
  PRs into `goblin` using `@create-pr` when a wave/PR slice is ready.
- Update epic checkboxes, specs, traceability, changelog and execution evidence
  only after implementation and validation prove the claim.
- Create durable issues for confirmed out-of-scope defects or blockers according
  to repository issue governance.
- Use subagents for bounded exploration, implementation on non-overlapping
  ownership, independent code review, security review and test review.
- Run local tools, Cargo/npm commands, schema generation and safe Git/GitHub
  inspection required for implementation and validation.

## Forbidden Actions

- Do not push product commits to `main` or open a product PR with base `main`.
- Do not reset/rewrite `goblin`, force-push shared integration history, or merge
  unreviewed work merely to progress the goal.
- Do not implement future programs: `30/v2-01`, `50/v2-01`, Goal v2 (`70/*`),
  Telegram (`80/*`) or voice (`90/*`) unless a separate explicit goal authorizes it.
- Do not migrate the dashboard/TUI to App Server in MVP.
- Do not invent provider contracts, credentials, TLS infrastructure, npm naming,
  token UX, or business decisions absent from canonical evidence.
- Do not add compatibility shims for the unused experimental-v1 wire contract.
- Do not use canned-success fakes, tests that exercise only mocks, snapshots that
  omit real schema fields, or production-only methods added for tests.
- Do not accept `cargo test <filter>` as proof when zero tests ran.
- Do not weaken ACL, redaction, sandbox, workspace trust or bearer validation to
  make tests pass.
- Do not expose tokens in URLs, argv, query strings, structured logs or fixtures.
- Do not let subagents edit overlapping files concurrently or merge their output
  without primary-agent diff review.
- Do not claim DONE, PASS, release-ready or production-safe for skipped,
  unavailable, flaky or HUMAN-blocked validation.

## Mode-Specific Rules

This is a multi-wave implementation program.

### Branches, worktrees, commits, and PRs

1. Inspect current branch, status, remotes and divergence before work.
2. If the shared worktree is dirty, identify ownership; do not absorb or discard
   unrelated changes.
3. Base each coherent feature branch on current `goblin`. Prefer one PR per epic
   or tightly coupled vertical slice; do not create one unreviewable mega-PR.
4. Use isolated worktrees for concurrent implementation or rebasing.
5. Keep commits behavior-coherent and include tests with production changes.
6. Before push: targeted tests, package gates, diff review and independent review.
7. Open PRs on `nonexphere/grok-build` with base `goblin` using `@create-pr`.
8. Do not mark a wave complete until required PRs are merged or the execution
   ledger explicitly records why reviewed local commits are the authorized state.
9. Record rollback notes for protocol, persistence, auth, lifecycle and transport
   changes in each PR.

### Mandatory TDD loop for every behavior task

1. Read the exact D-* IDs, contract section, owner path and acceptance criterion.
2. Write the smallest real behavior/regression/contract test first.
3. Run it and record RED: command, nonzero result and expected failure reason.
4. If it passes, fails for another reason or selects zero tests, fix the test
   before production code.
5. Implement the smallest robust production change.
6. Run GREEN with `scripts/run-rust-test-gate.sh` for named Rust tests.
7. Refactor only with the focused suite green.
8. Run the package gate and all directly affected consumer/contract suites.
9. Review the diff for ownership, secrets, generated files, compatibility and
   accidental edits.
10. Update task/evidence status only after all prior steps pass.

For non-behavioral docs or deterministic generation changes, run the smallest
relevant structural, link, formatting, typecheck or drift validation instead of
inventing a behavior test.

### Mandatory subagent protocol

The primary agent owns sequencing, integration, final decisions and all status
claims. Use subagents deliberately; delegation never transfers accountability.

For every epic or bounded slice:

1. Assign implementation subagents only concrete, non-overlapping file/module
   ownership. Tell each agent it is not alone, must preserve others’ edits, must
   not stage unrelated files, and must report exact commands/results.
2. Parallelize only independent domains. Shared protocol/schema, workspace
   manifests and composition-root files have one writer at a time.
3. After implementation, assign a fresh read-only reviewer subagent that did not
   author the slice. It must compare diff to D-* IDs, contracts, invariants and
   current runtime evidence; return PASS/FAIL with severity and file evidence.
4. Assign a separate test/conformance subagent that did not author the slice.
   It must inspect test quality, prove named tests are non-vacuous, run the
   required targeted/package/integration gates, and report skipped coverage.
5. For auth, bearer, workspace, file access, shell execution, remote bind,
   redaction or ACL changes, add a distinct security-review subagent or explicitly
   expand the reviewer’s security mandate.
6. Reviewers and testers are read-only by default. If a reviewer must implement
   a fix, close that review round and require a new independent review afterward.
7. The primary agent triages every finding, fixes confirmed issues, reruns tests,
   and requests re-review until no blocking/high finding remains.
8. Store reports under
   `.llms/execution/app-server-mcp-tower/reviews/<wave>/<slice>-review.md` and
   `.../<slice>-tests.md`.
9. Never use a subagent’s prose as the only evidence; preserve command output,
   diff references and machine-readable results where available.

Recommended concurrency: primary agent plus at most three subagents—one bounded
implementer, one reviewer and one tester—or two non-overlapping implementers
before the review phase. Avoid parallel review of a moving diff.

### Definition of done for one task

- Contract and D-* requirement identified.
- Correct RED observed and recorded when behavior changes.
- Production implementation complete without workaround.
- Focused test ran at least one case and passed.
- Package and affected-consumer gates passed.
- Independent review has no unresolved blocking/high finding.
- Test reviewer confirms coverage and non-vacuous execution.
- Docs/schema/generated artifacts are synchronized.
- Task checkbox/evidence is updated truthfully.

### Definition of done for one epic

- Every unblocked `tasks.md` item meets task definition of done.
- Epic README/SPECS match actual behavior.
- All required fixtures, security cases and recovery paths pass.
- No unresolved P0/P1 finding or accidental public-contract drift remains.
- PR/diff is reviewable and has rollback notes.
- Epic review and test reports are stored in the execution ledger.

## Execution Model

### Phase 0 — Intake, recovery, and baseline

1. Read governance and every canonical artifact listed above.
2. Inspect branch/status/remotes, existing commits/PRs and current scaffold.
3. Create/update STATUS with artifact inventory, current wave, branches, PRs,
   commands, known blockers and HUMAN gates.
4. Run baseline checks without editing:
   - `cargo metadata --no-deps --format-version 1`
   - package-scoped checks for the six new crates;
   - protocol tests and schema generation check;
   - SDK typecheck/test/drift;
   - relevant existing Shell/leader characterization tests;
   - `git diff --check`.
5. Record preexisting failures separately; never misattribute them to new work.

### Wave 0 — Freeze evidence and contracts

Execute, in dependency-aware parallel branches where safe:

- `10-providers/v1-01-codex-readiness-hygiene`
- `20-tower-core/v1-01-leader-characterization-promotion`
- `30-app-server/v1-01-session-protocol`

Required gate: existing Codex/provider behavior is honestly characterized;
leader/connect/spawn/ACP bytes and single-actor ownership have real fixtures;
experimental-v2 Rust/generated schema/operational schema/TS/goldens have zero
critical drift. This wave must not create a second daemon/runtime or implement
remote transports.

### Wave 1 — Provider seam, registry, and runtime facade

After Wave 0 gates:

- `10-providers/v1-02-api-key-provider-foundation`
- `20-tower-core/v1-02-multi-session-workspace-registry`
- `30-app-server/v1-02-runtime-facade-projection`

Required gate: provider selection remains request-bound; one actor owns each
loaded Session; N Sessions/workspaces work; resource safety budgets fail
explicitly; real and faithful-fake facade suites match; projection redacts
secrets and maps unknown events safely.

### Wave 2 — First complete local vertical slice

Execute only after the facade/registry gates:

- `30-app-server/v1-03-core-in-process-stdio`

Deliver initialize → Session start/read/list → Turn start → Item lifecycle/delta
→ final transcript through the same processor using in-process and stdio NDJSON.
Prove idempotency, cancellation, backpressure, concurrent Sessions and no
duplicate actor load. Do not begin remote WS/MCP to mask an incomplete local
slice.

### Wave 3 — Lifecycle, replay, interactions, and in-process tools contract

After the local vertical slice, execute independently where contracts permit:

- `20-tower-core/v1-03-multi-instance-daemon-modes`
- `30-app-server/v1-05-history-replay`
- `30-app-server/v1-06-approvals-control`
- `50-tower-agent-tools/v1-01-tool-contract-and-facade`

Required gate: two Tower instances remain isolated; replay snapshot/boundary/live
has no gaps and uses bounded `ReplayPage`; reconnect/epoch/resync works;
controller leases and Interaction races are deterministic; exact nine tools
invoke the facade directly with stable errors and schemas.

### Wave 4 — WebSocket, then MCP transports

Order is mandatory:

1. `30-app-server/v1-04-websocket-remote-auth`
2. `40-mcp-control-plane/v1-01-server-transports`

WebSocket must reuse the processor and pass parity with in-process/stdio before
MCP begins. MCP stdio and Streamable HTTP/SSE adapt the same tool/facade core;
the existing `xai-grok-mcp` client is not the server semantic implementation.
Loopback is the default. Cleartext remote stays experimental/unsafe.

### Wave 5 — Adapter parity, SDK, and provider verticals

After Wave 4:

- `50-tower-agent-tools/v1-02-in-process-acl-mcp-parity`
- `60-sdk-typescript/v1-01-generated-sdk-client-examples`
- `10-providers/v1-03-openrouter-onboarding`
- `10-providers/v1-04-groq-onboarding`
- `10-providers/v1-05-cloudflare-onboarding`

Required gate: exact tool parity through MCP and in-process paths; ACL defaults
fail closed; SDK supports stdio/WS, multiple subscriptions by subscription ID,
typed errors and reconnect/replay; Rust → generated schema → operational schema
→ TypeScript drift passes; each provider vertical proves catalog/binding/Turn/
logout without cross-account leakage. Live provider checks are opt-in and never
reported PASS when credentials are absent.

### Wave 6 — Remote security, release hardening, and operations

Execute in dependency order:

1. `40-mcp-control-plane/v1-02-remote-security-conformance`
2. `30-app-server/v1-07-release-hardening`
3. `20-tower-core/v1-04-operations-hardening`

Run the full threat matrix, token permission/rotation/redaction canaries,
oversize/slow-client/resource/fault/restart/drain tests, cross-transport
conformance, CLI smoke and runbooks. Non-loopback production readiness remains
blocked until TLS termination and HUMAN security acceptance are evidenced.

### Phase 7 — Final cross-program audit and delivery

1. Reread all in-scope epics and 157-ID INDEX.
2. Verify code, tasks, README/SPECS, TRACEABILITY and status agree.
3. Run fresh independent architecture/code, security and test reviews against
   the final integrated diff/branch—not only accumulated per-wave reports.
4. Resolve findings and rerun affected plus full gates.
5. Build the human-facing binary and execute local smoke flows.
6. Produce final delivery report with PRs/commits/tests/HUMAN blockers/risks.
7. Do not begin future/backlog epics under this goal.

## Issue, Decision, And Blocker Rules

- Classify every discovery as:
  - confirmed actionable issue;
  - decision-needed item;
  - duplicate/resolved/non-issue;
  - external blocker.
- Fix in-scope confirmed correctness/security/test gaps before progressing.
- Materialize durable out-of-scope findings with evidence and link them from
  STATUS; do not silently broaden the current epic.
- Mark `Human decision required: yes` only for product intent, architecture,
  ownership, credentials or external policy that blocks safe progress.
- Existing HUMAN gates include:
  - public npm package name/publication;
  - final token create/list/revoke CLI UX if still conditional;
  - compatibility behavior for a future Codex adapter accepting missing
    `jsonrpc`;
  - approval timeout policy where the contract leaves it HUMAN-gated;
  - final Tower CLI flag names if not already approved;
  - non-loopback production threat-model acceptance and verified TLS termination;
  - live provider credentials/authorization and release sign-off.
- A HUMAN gate blocks only the dependent release/task. Continue independent
  local work and record the dependency precisely.
- If implementation exposes a cross-repository consumer not present in the
  workspace, stop that contract change and report the missing owner/schema.

## Validation Plan

### Per task

- Exact RED and GREEN commands from the epic task.
- Named Rust gates through:
  `./scripts/run-rust-test-gate.sh <fragment> cargo test ...`.
- Whole-package test when the acceptance is the complete package.
- `cargo check` for directly affected crates and consumers.
- `git diff --check` and focused diff review.

### Protocol/schema/SDK

- `cargo run -q -p xai-grok-app-server-protocol --example generate-schema -- --check`
- `cargo test -p xai-grok-app-server-protocol`
- `jq empty crates/codegen/xai-grok-app-server-protocol/schemas/*.json`
- Validate every line in every protocol golden as JSON.
- `npm --prefix packages/grok-oss-app-server run typecheck`
- `npm --prefix packages/grok-oss-app-server test`
- `npm --prefix packages/grok-oss-app-server run check:drift`

### Core Rust packages

- `cargo check -p xai-grok-app-server-protocol -p xai-grok-app-server-client -p xai-grok-app-server -p xai-grok-tower -p xai-grok-tower-tools -p xai-grok-mcp-server`
- `cargo test -p xai-grok-app-server-protocol -p xai-grok-app-server-client -p xai-grok-app-server -p xai-grok-tower -p xai-grok-tower-tools -p xai-grok-mcp-server`
- Relevant `xai-grok-shell`, leader, pager, auth, sampler, workspace and MCP
  consumer tests for the changed seams.

### Cross-transport conformance

- Run identical normalized fixtures through in-process, stdio, WebSocket and MCP.
- Cover initialize gates, all critical Session/Turn methods, Items/deltas,
  replay/resync, Interactions, errors, cancellation and slow subscribers.
- Assert all nine Tower tools have identical normalized semantics through direct
  and MCP adapters.

### Security and failure validation

- Bearer header-only, constant-time rejection, file owner/mode/symlink tests.
- Secret canaries across logs, errors, metrics, tool outputs and panics.
- Workspace canonicalization/symlink races and sandbox/trust checks.
- Oversized messages, queue saturation, replay limits and resource budgets.
- Duplicate resume/start races, interrupt/approval races, crash/restart/epoch,
  stale metadata, drain and multi-instance isolation.
- Cleartext remote labeling and production TLS/HUMAN gate enforcement.

### Product smoke and broad gates

- Build: `cargo build -p xai-grok-pager-bin --bin grok-oss`.
- Confirm binary/version and execute a local safe scripted Session/Turn flow.
- Exercise App Server stdio and, when ready, loopback WS/MCP smoke clients.
- Run workspace-wide format/lint/test only when repository evidence says the
  command is supported and preexisting blockers are accounted for.
- If a validation cannot run, record exact command, reason, affected guarantee,
  risk and substitute evidence. Never label it PASS.

## Memory And Artifacts

- Session/log path: `.llms/execution/app-server-mcp-tower/STATUS.md`
- Per-wave evidence: `.llms/execution/app-server-mcp-tower/waves/<wave>.md`
- Reviews: `.llms/execution/app-server-mcp-tower/reviews/<wave>/`
- Test logs/summaries: `.llms/execution/app-server-mcp-tower/tests/<wave>/`
- Decisions: `.llms/execution/app-server-mcp-tower/DECISIONS.md`
- Blockers: `.llms/execution/app-server-mcp-tower/BLOCKERS.md`
- PR/commit ledger: `.llms/execution/app-server-mcp-tower/CHANGES.md`
- Completion report: `.llms/execution/app-server-mcp-tower/FINAL_REPORT.md`

STATUS must always record current wave/epic/task, branch/worktree, last green
commands, open reviews/findings, next safe action, HUMAN gates and dirty state.
Keep evidence concise and redact secrets.

## Stop Conditions

The goal is complete only when all are true:

- Every unblocked in-scope v1 task in programs 10–60 is implemented and checked.
- Every in-scope D-* requirement is DONE with code/test evidence or explicitly
  remains HUMAN/external-blocked without false completion.
- All wave gates and the final validation matrix pass freshly.
- Independent final architecture/code, security and test reviews have no
  unresolved blocking/high findings.
- App Server local vertical slice, Tower lifecycle/tools, MCP adapters, SDK and
  applicable provider verticals work as specified.
- `grok-oss` builds and required local smoke tests pass.
- PRs/commits, docs, task status and execution artifacts are reconciled.
- Future/backlog programs remain untouched except for proven regression fixtures
  or links explicitly allowed by their freeze contracts.
- FINAL_REPORT provides an honest COMPLETE verdict and residual risks.

## Blocked Conditions

Report `BLOCKED` only when safe independent work is exhausted and one of these
conditions prevents further progress:

- a required HUMAN decision above materially changes the dependent design;
- provider/live validation requires credentials or authorization not available;
- production remote release requires TLS/external setup or human threat acceptance;
- a required producer/consumer contract or owner exists outside the workspace
  and cannot be verified;
- the only path would break leader/ACP/session compatibility, weaken security,
  duplicate runtime authority, fake tests, or introduce a workaround;
- repository corruption/toolchain failure prevents relevant implementation and
  no safe substitute exists.

Before BLOCKED, record the exact unmet requirement, evidence, affected tasks,
safe work already completed, smallest robust resolution and resume command.

## Final Report Requirements

- Final state: COMPLETE or BLOCKED with exact reason.
- Epics/waves completed and remaining.
- Files/areas and architecture boundaries changed.
- Commits, branches, worktrees and PRs with base/head.
- RED/GREEN, package, integration, conformance, security, SDK and smoke commands
  with observed results and test counts.
- Independent subagent reviews, findings and their resolutions.
- D-* coverage and any remaining PARTIAL/HUMAN/external entries.
- Generated/schema/versioning status and compatibility impact.
- Commands not run, why, affected guarantee and risk.
- Residual risks, rollback notes and cross-repository coordination.
- Worktree/staging state and confirmation that unrelated work was preserved.
- Completion or blocked verdict without presenting partial work as complete.
