# Adversarial execution audit — App Server / MCP / Tower

**Audit date:** 2026-07-18  
**Audited branch:** `goblin-implement-epic-tree`  
**Audited HEAD:** `8a3c14f`  
**Baseline:** `a91ac89`  
**Primary contract:** `.llms/tasks/20260718-execute-app-server-mcp-tower-plan.md`  
**Execution report:** `.llms/execution/app-server-mcp-tower/FINAL_REPORT.md`  
**Verdict:** **FAIL / BLOCKED — substantial FakeRuntime scaffolding is green, but the planned production architecture and several claimed epic acceptance criteria were not delivered.**

**Post-snapshot dirty state:** while this audit was being finalized, the active executor modified `app_server_composition.rs` and `app_server_runtime/mod.rs` without a new commit. Those volatile edits are not included in the green test result below. They introduce `SessionStorageHybridRuntime`, reading list/read from JSONL while continuing every mutation and replay through `FakeRuntime`.

## Scope and method

This is a read-only adversarial review of commits `a91ac89..8a3c14f`, the execution ledger, epic task files, production code and tests. The audit treats source and executable tests as stronger evidence than checkboxes. It does not convert an external/HUMAN skip into PASS and does not treat a helper or fake-backed test as proof of a real transport or runtime integration.

The implementation agent eventually reported the program as `BLOCKED`, which is correct and avoids a false global completion claim. That honesty does not repair the inconsistent task state or satisfy acceptance criteria already marked complete.

## Executive result

What is genuinely delivered:

- a versioned protocol surface with schema/golden checks;
- a useful, stateful `FakeRuntime` and facade-oriented processor;
- local in-process and stdio slices over the fake;
- Tower registry/lifecycle/projection primitives;
- nine Tower tool descriptors and a semantic adapter;
- WebSocket framing/auth helpers and MCP HTTP/SSE helper primitives;
- security canaries and package-local unit tests;
- a final report that explicitly identifies the production `SessionActor` blocker.

What is not delivered:

- a production `GrokRuntimeFacade` backed by the existing Shell leader/`SessionActor`;
- production composition that executes real sessions and turns;
- a WebSocket listener/server or a Streamable HTTP MCP server;
- complete OpenRouter, Groq and Cloudflare provider verticals;
- real history rebuild/replay against canonical persisted sessions;
- transport-wide black-box conformance, dual-process leader proof, and final independent reviews for later waves;
- the complete execution ledger and TDD evidence required by the contract.

## Findings

### F-01 — BLOCKER: production composition still injects `FakeRuntime`

Evidence: `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs:1-17` documents and constructs `ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()))`. No product path maps the facade onto the existing leader or `SessionActor` command path.

Impact: the green initialize/session/turn composition test proves only the fake. The principal architectural invariant — one existing Shell runtime authority — is not implemented.

Required correction: implement a Shell-owned actor-backed port for every facade method, inject it at the composition root, and run the same conformance suite against fake and real adapters.

### F-02 — HIGH: `RF102-02` and `RF102-05` are checked despite contradicting their acceptance criteria

Evidence: `30-app-server/v1-02-runtime-facade-projection/tasks.md` marks both tasks complete. The adapter at `xai-grok-shell/src/app_server_runtime/mod.rs:38-141` only delegates to another facade and maintains an independent token registry. The test named `single_actor_owns_turn_mutation` at lines 225-261 injects `FakeRuntime`, allows all eight concurrent starts, and proves only that one opaque registry token exists.

Impact: neither “each facade call sends one existing SessionActor/leader command” nor “ordered mutations” is tested. The test name overstates its evidence.

Required correction: reopen both tasks; test real actor command routing, foreground-turn exclusivity/steering semantics, deterministic ordering and cancellation.

### F-03 — HIGH: transport epics claim servers, but only pure helpers exist

Evidence: `xai-grok-app-server/src/transport/websocket.rs:65-91` processes a supplied text value; it has no listener, handshake, socket lifecycle, ping/pong implementation, bounded writer or disconnect handling. `xai-grok-mcp-server/src/transport/http.rs:43-77` contains an in-memory cursor table and response constructor; it has no POST/GET/DELETE router, HTTP listener, SSE stream lifecycle or session deletion. The final report also concedes “helpers only”.

Impact: checked tasks `AS104-01`, `AS104-05`, `AS104-06` and `MCP101-03` are unsupported. Network behavior, backpressure and remote auth are untested end to end.

Required correction: either implement real transports and black-box tests or reopen/re-scope every affected task without calling helpers a server.

### F-04 — HIGH: provider verticals were reduced to descriptors and skips were checked as done

Evidence: `providers/byok/mod.rs:14-49` defines constants and string helpers, not registered provider implementations, model discovery or inference. OpenRouter/Groq/Cloudflare task files mark live Turn smoke `SKIP without credentials` as `[x]`, while the contract explicitly says skip is never PASS. The final report separately admits provider verticals remain open.

Impact: onboarding is not functional and progress reporting is internally inconsistent.

Required correction: restore implementation-ready provider tasks; wire registry, credential selection, request auth, catalog discovery and a real Turn; leave credential-dependent checks SKIP/open until executed.

### F-05 — HIGH: API-key login accepts unregistered providers and hardcodes ephemeral metadata

Evidence: `login_coordinator.rs:209-212` discards the result of `registry.get`; any syntactically constructed provider ID can proceed. Lines 239-251 store metadata with `SecretBackendKind::Ephemeral` regardless of the injected credential store. Lines 227-233 contain a no-op XAI fallback branch.

Impact: unknown providers can receive credentials, persisted-store metadata can misrepresent the backend, and the claimed fallback prohibition is not enforced by this path.

Required correction: require a registered API-key-capable descriptor, derive backend from the actual store/record policy, remove the no-op branch, and test rejection through the public login path.

### F-06 — HIGH: history/replay acceptance is proven only against volatile fake state

Evidence: tests are explicitly named `projection_rebuild_via_replay_is_stable_for_fake` and `snapshot_then_live_no_gap_on_fake`. No canonical session-file index/rebuild path appears in the App Server implementation. Yet all `AS105-01..07` tasks are checked.

Impact: crash recovery, durable epoch semantics and delete/rebuild equivalence are not established.

Required correction: reopen the epic and implement tests over canonical persisted session artifacts, including crash/restart, stale/foreign cursors and replay/live race boundaries.

### F-07 — MEDIUM: Tower instance selection uses the wrong/unvalidated contract surface

Evidence: `app_server_composition.rs:60-85` reads `GROK_TOWER_INSTANCE`, returns arbitrary strings without `TowerInstanceId` validation, and its fallback test merely asserts non-empty because ambient environment is uncontrolled. The planned/canonical surface uses `GROK_OSS_TOWER` and an explicit validated selector.

Impact: invalid IDs can cross the composition boundary and behavior depends on an undocumented environment variable.

Required correction: use the canonical configuration name/type and hermetic precedence tests.

### F-08 — HIGH: completion ledger and reviews do not meet the execution contract

Evidence: the contract required `CHANGES.md`, `BLOCKERS.md`, `DECISIONS.md`, per-wave evidence/test/review artifacts and independent code/test review for each slice. Only `STATUS.md`, `BLOCKERS.md`, `FINAL_REPORT.md`, one combined wave 0–2 report and one wave 0–2 review pair exist. No `CHANGES.md`, `DECISIONS.md`, later-wave reports or later-wave independent reviews exist.

Impact: later changes lack independent acceptance and resumable decision history.

Required correction: reconstruct traceability per epic/commit and run fresh independent reviews after implementation; never self-author an “independent” verdict.

### F-09 — HIGH: strict TDD is not auditable

Evidence: the ledger contains final green outputs and some early failures, but does not provide named RED/GREEN evidence for each behavior as mandated. Several task files claim `D-TD.3` complete without the required evidence documents. Earlier aggregate logs also contain vacuous `running 0 tests` invocations.

Impact: successful current tests do not prove test-first development or non-vacuous execution of every named gate.

Required correction: for every remaining behavior, capture the exact failing test and reason before production edits, then green/refactor output. Existing unproven work must be covered by characterization/regression tests and must not be relabeled retrospectively as TDD.

### F-10 — MEDIUM: public task state conflicts with the authoritative final report

Evidence: 46 checked versus 8 open tasks were observed in the scoped tree before v2/out-of-scope separation, including complete flags for known helpers, fake-only behavior and skipped live tests. `FINAL_REPORT.md:61-64` acknowledges over-checked files.

Impact: a future executor cannot safely resume from checkboxes and may skip missing production work.

Required correction: reopen every task whose literal acceptance criterion is not evidenced; add PARTIAL/SKIP/BLOCKED annotations instead of encoding them as `[x]`.

### F-11 — MEDIUM: test names and scope produce false confidence

Evidence: “websocket conformance” calls `handle_ws_text` directly; “streamable_http” tests an auth helper and `Mutex<HashMap>`; “single_actor” uses the fake; “projection rebuild” uses fake replay. The tests pass, but they do not cross the named production boundary.

Impact: package totals are real but cannot support the corresponding architecture/release claims.

Required correction: reserve transport/production/conformance names for black-box boundary tests; rename helper tests and add real suites.

### F-12 — LOW: validation is green with a warning

Evidence: the independent audit command passed 91 library tests, but `xai-grok-app-server/src/controller.rs:52` has an unused `lease` variable and the pager manifest reports one source in three bin targets.

Impact: no functional failure demonstrated, but warning-free gates were not achieved.

Required correction: resolve the new warning and document whether the multi-bin layout is intentional.

### F-13 — BLOCKER (volatile post-snapshot): proposed hybrid runtime creates split authority

Evidence: the dirty worktree adds `SessionStorageHybridRuntime`: list/read consult real JSONL summaries, while start/resume/fork/archive/turn/interaction/replay still delegate to `FakeRuntime`. It also synthesizes `history_epoch = "epoch_1"`, revision zero and empty turns/items for disk sessions. The composition comment explicitly admits mutations remain fake.

Impact: this is not an incremental production adapter; it creates mutually inconsistent read and mutation worlds. A listed disk session may be unreadable by the fake mutation port, newly created fake sessions are not persisted by JSONL, replay cannot represent disk history, and synthetic counters can violate cursor/revision semantics. It conflicts with the single-authority design and repository stop conditions against partial implementations presented as progress.

Required correction: do not merge the hybrid. Implement one actor-backed facade with canonical read, mutation and replay semantics, or keep the product composition blocked on `FakeRuntime` while that complete port is built behind tests.

## Validation executed by this audit

```text
cargo test -p xai-grok-app-server-protocol -p xai-grok-tower \
  -p xai-grok-app-server -p xai-grok-tower-tools \
  -p xai-grok-mcp-server --lib --no-fail-fast
```

Result: **PASS, 91 tests** (22 protocol, 21 Tower, 26 App Server, 11 Tower tools, 11 MCP), with the warning described in F-12. This validates the implemented unit-level/fake-backed slice, not the missing production boundaries.

The 91-test command completed before the post-snapshot hybrid edits appeared. Those two dirty files require their own RED/GREEN evidence and full gate if retained.

## Gate decision

**Implementation program: FAIL / BLOCKED.** Do not merge or advertise the program as production-capable. The current branch is a useful experimental foundation and can be continued, but task checkboxes must first be reconciled with literal acceptance criteria. Completion requires satisfying the corrective execution contract in `.llms/tasks/20260718-correct-app-server-mcp-tower-execution.md` and obtaining fresh independent code and test review.
