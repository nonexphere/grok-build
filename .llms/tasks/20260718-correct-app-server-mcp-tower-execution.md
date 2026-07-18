# Corrective execution contract — App Server / MCP / Tower

## 1. Objective and authority

Correct the incomplete implementation produced under `.llms/tasks/20260718-execute-app-server-mcp-tower-plan.md` until the production path, transports, providers, persistence semantics, tests and execution evidence satisfy their literal epic acceptance criteria.

This contract is subordinate to repository `AGENTS.md`, the canonical specs under `.llms/grok-build/`, and shared contracts under `.llms/grok-build/_shared/`. It supersedes checkboxes that conflict with executable evidence. The adversarial findings in `.llms/reviews/app-server-mcp-tower-adversarial-audit-2026-07-18.md` are mandatory inputs.

## 2. Non-negotiable methodology

- Work from a `goblin-*` feature branch based on `goblin`; never target `main`.
- Preserve one runtime authority: existing Shell leader/`SessionActor`. Tower must not define a second actor or depend on Shell.
- Do not create a hybrid facade that reads real JSONL state while mutating or replaying `FakeRuntime`; all operations for a Session must share one canonical authority.
- Use strict Red-Green-Refactor for every behavior change. Capture the named RED before production edits and the GREEN afterward.
- A helper test is not a transport test; a fake-backed test is not production integration evidence; SKIP is never PASS.
- Use independent subagents for read-only code review and test review after every bounded epic/slice. Reviewers must not implement the slice they review. The primary executor triages findings and reruns gates.
- Run one implementation slice at a time. Do not let parallel agents edit overlapping files. Subagents may review in parallel only after the slice is stable.
- Do not mark a task complete unless its literal acceptance criterion, named tests, required review and ledger evidence all exist.
- Do not use live credentials unless explicitly supplied through approved secret mechanisms. Never print or commit them.
- Do not claim remote production readiness without the HUMAN TLS/threat acceptance gate.

## 3. Required ledger

Maintain these files throughout execution:

- `.llms/execution/app-server-mcp-tower-corrective/STATUS.md`
- `.llms/execution/app-server-mcp-tower-corrective/CHANGES.md`
- `.llms/execution/app-server-mcp-tower-corrective/BLOCKERS.md`
- `.llms/execution/app-server-mcp-tower-corrective/DECISIONS.md`
- `.llms/execution/app-server-mcp-tower-corrective/waves/<wave>.md`
- `.llms/execution/app-server-mcp-tower-corrective/tests/<wave>/<gate>.txt`
- `.llms/execution/app-server-mcp-tower-corrective/reviews/<wave>/code-review.md`
- `.llms/execution/app-server-mcp-tower-corrective/reviews/<wave>/test-review.md`
- `.llms/execution/app-server-mcp-tower-corrective/FINAL_REPORT.md`

Every wave report must map task IDs to files, tests, RED/GREEN logs, review findings, fixes, commits, skips and blockers.

## 4. Epics and sequence

### Wave C0 — Reconcile truth before implementation

1. Pin baseline commit, branch, dirty state and all original/corrective contracts.
2. Build a requirement matrix for every v1 task in providers, Tower, App Server, MCP, tools and SDK.
3. Reopen every `[x]` whose literal criterion is not supported by production code and a non-vacuous test; explicitly reopen at least RF102-02/05, AS104 network tasks, AS105 persistence tasks, MCP101-03 and credential-dependent provider smoke tasks.
3a. Remove or reject the volatile `SessionStorageHybridRuntime` approach unless it is replaced by one coherent actor-backed authority; add a regression test that a listed session is readable, mutable and replayable through the same runtime.
4. Label each task `OPEN`, `PARTIAL`, `BLOCKED`, `SKIP`, `HUMAN` or `PASS`; only `PASS` may use `[x]`.
5. Characterize existing leader/`SessionActor` commands, lifecycle, permission interactions, persistence and composition entry points without changing behavior.
6. Ask an independent architecture-review subagent to validate the command mapping and ownership boundary; resolve all Critical/High findings before Wave C1.

### Wave C1 — Real Shell runtime authority

7. Write failing contract tests that run every `GrokRuntimeFacade` method against a real Shell-owned actor-backed adapter.
8. Implement the smallest Shell port that maps list/read/start/resume/fork/archive, turn start/steer/interrupt, interaction response and replay to existing leader/`SessionActor` commands; do not add a second state machine.
9. Replace the opaque `SessionRegistry<()>` proof with evidence of actual actor identity/residency and authoritative command routing.
10. Define and test foreground-turn rules: concurrent starts, idempotent retries, steering, interruption, ordering, cancellation and actor failure.
11. Replace hardcoded projection epoch/revision values with canonical persisted/runtime values or remove the projection until they are available.
12. Switch the `grok-oss` composition root from `FakeRuntime` to the real Shell port; retain the fake only for conformance/unit tests.
13. Run the same conformance suite against FakeRuntime and the real adapter and compare normalized results.
14. Commission separate independent code-review and test-review subagents; fix all Critical/High and accepted Medium findings, then rerun the complete Wave C1 gate.

### Wave C2 — Configuration, lifecycle and process ownership

15. Replace `GROK_TOWER_INSTANCE` with the canonical `GROK_OSS_TOWER` contract unless a spec decision explicitly says otherwise.
16. Parse and validate all selected IDs through `TowerInstanceId`; add hermetic explicit > env > default tests without ambient environment dependence.
17. Add a true dual-OS-process leader/flock test, handshake mismatch cases, stale metadata safety and restart/drain behavior against the composition path.
18. Prove two Tower instances have isolated directories, registries, ports and sessions without shared hidden state.
19. Obtain independent code/test reviews and close findings before continuing.

### Wave C3 — Real App Server transports and persistence

20. Write failing black-box tests for an actual WebSocket listener: handshake/subprotocol, header auth, text frames, ping/pong, binary/batch/oversize rejection, disconnect, bounded writer and slow-client resync.
21. Implement the real listener/lifecycle using the shared processor; keep cleartext non-loopback experimental/unsafe and blocked from production claims.
22. Write failing persistence tests over canonical session files for rebuild, stable IDs/order, history epoch, entity revisions, crash/restart, stale/foreign cursors and replay-to-live race boundaries.
23. Implement projection/replay without making a second execution truth or relying on volatile fake state.
24. Run one black-box conformance suite across in-process, stdio and WebSocket using the real adapter where feasible.
25. Obtain independent transport/security code review and independent test review; fix findings and rerun gates.

### Wave C4 — Real MCP Streamable HTTP and tool parity

26. Write failing black-box tests for POST/GET/DELETE `/mcp`, session lifecycle, SSE resume, auth failure equivalence, body limits, cancellation and disconnect.
27. Implement an actual Streamable HTTP server/router over the shared Tower tool semantic core; an in-memory cursor helper alone is insufficient.
28. Prove exact nine-tool descriptor/schema parity and normalized result/error parity between in-process and MCP adapters.
29. Prove no local MCP self-loop or duplicate tool execution path in production composition.
30. Run remote security tests at the socket boundary and obtain independent code/test reviews.

### Wave C5 — Provider foundation and three complete verticals

31. Write failing public-path tests that reject unknown/unregistered providers and providers without API-key capability.
32. Make API-key login use registered provider descriptors, the actual credential-store backend policy and an explicit secret source; remove no-op fallback logic.
33. Implement secure TTY paste UX without argv/log leakage and retain non-interactive approved secret input.
34. Register real OpenRouter, Groq and Cloudflare provider implementations with credential selection, request authentication, base URL/account handling, model catalog discovery and inference binding.
35. Reuse the canonical protocol `ProviderBinding`; do not maintain an unconnected duplicate public binding type.
36. Add offline contract tests with schema-faithful HTTP boundary fixtures for each provider, including multiple credentials with the same model slug, auth failures, rate limits and redaction.
37. Run opt-in live model/Turn smoke only when credentials are available. Record missing credentials as SKIP/open, never `[x]` or PASS.
38. Obtain independent provider/security code review and test review; fix findings and rerun gates.

### Wave C6 — Approvals, tools, SDK and cross-surface conformance

39. Map interaction responses to the existing Shell permission/elicitation command path and prove there is no second permission engine.
40. Test lease ownership, reconnect, expiry, duplicate/conflicting terminal responses and explicit auto-deny policy across real transports; never auto-allow.
41. Run every Tower tool through the real adapter, including wait/history/send/archive, ACL denial, idempotency and limits.
42. Regenerate/validate TypeScript contracts from the canonical schema, then test request correlation, error typing, reconnect, epoch validation and replay/live ordering against a real server fixture.
43. Run one normalized conformance matrix across real in-process, stdio, WebSocket and MCP boundaries.
44. Obtain independent code and test reviews for approvals/tools/SDK and resolve findings.

### Wave C7 — Adversarial release gate

45. Run formatting, lint/check, schema/golden drift, all relevant Rust tests, TypeScript typecheck/tests and builds; treat warnings introduced by this program as failures unless explicitly justified.
46. Run adversarial concurrency, malformed input, secret canary, path/symlink, replay/backpressure, cancellation, restart and multi-instance suites.
47. Have one independent subagent review the complete diff against original epics/contracts and another independently audit test adequacy and vacuous filters.
48. Reconcile every checkbox with final executable evidence. No unchecked non-HUMAN, non-external v1 item may remain for COMPLETE.
49. Record HUMAN/external gates separately: TLS/threat acceptance, live credentials, npm naming/publish and other explicit product decisions.
50. Produce `FINAL_REPORT.md` with exact commands, counts, skips, blockers, commits, downstream impact and a strict `COMPLETE` or `BLOCKED` verdict.

## 5. Mandatory acceptance gates

The corrective program is COMPLETE only if all are true:

- product composition does not instantiate `FakeRuntime`;
- every facade method reaches the existing Shell leader/`SessionActor` command path;
- real adapter and fake pass a shared conformance suite;
- real WebSocket and MCP HTTP servers pass black-box tests;
- canonical persisted sessions drive history/replay recovery tests;
- OpenRouter, Groq and Cloudflare are registered functional verticals; missing live credentials remain explicit SKIP only;
- task files, ledger, tests and reviews agree;
- every behavior change has non-vacuous RED/GREEN evidence;
- every wave has independent code and test review with findings triaged;
- all local automated gates are green and no Critical/High finding remains;
- no secret, public contract, downstream consumer or production-readiness claim is left ambiguous.

## 6. Stop conditions

Stop and mark `BLOCKED` rather than weakening the design if completion would require a second actor/state machine, fake-backed production composition, a helper presented as a server, a skipped live test presented as PASS, an undocumented contract break, unverified external ownership, insecure secret handling, or bypassing an unresolved HUMAN security/product decision.

## 7. Prompt for the executing agent

Execute this contract sequentially from Wave C0 through C7. First reconcile task truth; then implement only one bounded slice at a time with strict Red-Green-Refactor. After each slice, dispatch independent read-only subagents for code review and test review, triage their findings, fix all blocking issues and rerun the full slice gate. Keep the corrective ledger current after every meaningful action. Never infer completion from existing checkboxes or package test totals: verify the literal epic acceptance criterion at the real boundary. Continue until every locally executable v1 criterion is proven or a genuine external/HUMAN blocker is precisely documented. End with a strict COMPLETE/BLOCKED report; do not present partial work as complete.
