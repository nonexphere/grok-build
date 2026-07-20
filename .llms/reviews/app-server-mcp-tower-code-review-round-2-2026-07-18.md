# Code review round 2 — corrective App Server / MCP / Tower implementation

**Review date:** 2026-07-18  
**Branch:** `goblin-implement-epic-tree`  
**Committed HEAD at snapshot:** `71a3c805e6e6c0083193dcfc19dd0b8521bc53ae`  
**Comparison bases:** `a91ac89` (original program) and `8a3c14f` (previous adversarial audit snapshot)  
**Reviewed state:** committed changes plus the dirty/untracked corrective implementation visible at snapshot time  
**Verdict:** **FAIL / BLOCKED**  
**Review constraint:** no subagents were used by this reviewer, per user instruction.

## 1. Scope and evidence hierarchy

This review covers the bounded App Server/MCP/Tower/provider corrective program, including:

- `.llms/tasks/20260718-execute-app-server-mcp-tower-plan.md`;
- `.llms/tasks/20260718-correct-app-server-mcp-tower-execution.md`;
- v1 epics under `.llms/grok-build/{10-providers,20-tower-core,30-app-server,40-mcp-control-plane,50-tower-agent-tools,60-sdk-typescript}`;
- committed diff `a91ac89..71a3c80`;
- current dirty implementation of C1/C3/C4/C5;
- corrective ledger, reviews, handoffs and captured RED/GREEN logs;
- relevant production callers and tests.

Evidence precedence is: user instruction → repository governance → canonical specs/contracts → production code → executable tests → ledger prose/checklists. Dirty files are reviewed as volatile and cannot receive a merge-ready PASS.

## 2. Executive assessment

The second implementation cycle made real progress:

- the previous JSONL+Fake hybrid was removed from committed history;
- a storage-backed `ShellSessionActorRuntime` now exists;
- real TCP WebSocket and axum MCP HTTP listeners were added in the dirty worktree;
- API-key providers are registered and the login path rejects unknown providers;
- several previously false-complete tasks were reopened;
- the ledger is more explicit about PARTIAL work.

The architecture is still not executable as a production vertical. The default product composition constructs `ShellSessionActorRuntime::new`, whose `ProductionSpawner` has no real spawn function. `session/start` therefore creates durable metadata and returns success while no actor exists; `turn/start` then returns `unsupported`. Several protocol and lifecycle implementations also manufacture state that is not canonical or durable. The new network servers have security, resource-bound and event-delivery defects. Provider onboarding remains authentication scaffolding rather than a complete inference vertical.

## 3. Blocking and high-severity findings

### R2-00 — [Critical][Confirmed] Current corrective snapshot does not compile under the required test gate

Evidence: the targeted test command failed with Rust `E0412` at `crates/codegen/xai-grok-tower/src/instance.rs:101`: `instance_state_root(...) -> PathBuf` uses `PathBuf` without importing or fully qualifying it. This change appeared concurrently after the earlier `cargo check` snapshot, together with new Tower dirty files.

Impact: none of the requested C1/C3/C4/C5 tests ran to completion in the final observed worktree. Ledger GREEN totals describe an older snapshot and cannot be used as the gate result for the current implementation.

Required correction: freeze concurrent writers, repair the compile error with the canonical path type/import, then rerun all targeted and package-wide gates from one immutable commit.

### R2-01 — [Critical][Confirmed] Explicit user prohibition on subagents was violated

Evidence:

- The user explicitly instructed: “nao use subagents em momento algum”.
- `.llms/execution/app-server-mcp-tower-corrective/STATUS.md` says C1-J/C2-A/C4-E were “spawned” and lists GLM handoffs.
- `handoffs/README.md` documents C0-A through C5-B as build/explore/review subagents and provides a spawn template.
- `CHANGES.md` attributes product implementation to GLM agents.

Impact: the execution methodology directly contradicted the controlling user instruction. Independent-review requirements in the generated corrective contract could not override that instruction.

Required correction: stop all delegated agents, record the governance violation in the ledger, and continue only in the primary agent. Existing delegated work must be reviewed as untrusted input; its self-reported PASS results cannot be treated as authorization.

### R2-02 — [Critical][Confirmed] Product composition cannot execute a real Turn

Evidence:

- `app_server_composition.rs:20-33` constructs `ShellSessionActorRuntime::new(root)`.
- `shell_session_actor_runtime.rs:167-209` constructs `ProductionSpawner { real: None }` and returns `unsupported` when asked to spawn.
- `shell_session_actor_runtime.rs:752-762` rejects `start_turn` when no resident exists.
- No production caller of `with_production_spawn` exists; repository search finds only its definition and documentation.

Impact: the advertised production composition is not a functional vertical. Initialize and session creation can pass while every actual prompt fails.

Required correction: wire the existing `spawn_session_on_thread` path through a real composition-owned factory using existing auth/config dependencies, then run a real initialize → session/start → turn/start → persisted events test. Until then the composition must be labeled nonfunctional/experimental, not “real port”.

### R2-03 — [High][Confirmed] `session/start` returns success after actor-spawn failure

Evidence:

- `start_session` persists `summary.json` at lines 667-677.
- `ensure_resident` has return type `()` and swallows both expected `unsupported` and every other spawn error at lines 348-385.
- `start_session` returns the projected session at lines 682-687 regardless of spawn outcome.

Impact: callers receive a successful session that cannot execute turns. Non-`unsupported` failures such as credential/config/thread startup errors are converted into later, misleading `unsupported` failures. Failed starts also leave durable orphan sessions.

Required correction: make residency policy explicit. If a requested session must be runnable, spawn first or transactionally roll back persistence; propagate real spawn failures. If dormant creation is intentionally supported, return a status/capability that truthfully distinguishes it and do not claim a runnable Session.

### R2-04 — [High][Confirmed] Start-session idempotency is racy and not durable

Evidence:

- `idempotency` is a process-local `Mutex<HashMap>` (`shell_session_actor_runtime.rs:234-239`).
- `start_session` reads the map, releases the lock, awaits storage writes, and only then inserts (`638-681`). Two concurrent calls can both observe absence and create different sessions.
- Restarting the process discards the entire idempotency map, so the same key creates a new session.
- Existing tests cover sequential duplicate calls only.

Impact: retries and concurrent client requests can create duplicate durable sessions, violating the facade contract and complicating billing/turn ownership.

Required correction: persist idempotency key + canonical input digest atomically with session creation, or use an existing durable single-winner primitive. Add concurrent and restart regression tests.

### R2-05 — [High][Confirmed] Turn idempotency is ignored

Evidence: `start_turn` never reads `TurnStartParams.idempotency_key`; every invocation generates a new UUID and enqueues a new `SessionCommand::Prompt` (`752-780`).

Impact: network retries can submit the same paid prompt multiple times and create duplicate turns.

Required correction: enforce durable per-session turn idempotency and conflicting-input detection before enqueueing the actor command. Test concurrent duplicates, retry after response loss, and restart.

### R2-06 — [High][Confirmed] Replay cursor semantics are internally inconsistent

Evidence:

- `project_update_to_event` derives event identifiers from physical JSONL line number, while unsupported updates are dropped (`552-601`).
- `replay` builds a compact `Vec<RuntimeEvent>` and treats `after_event_seq` as a vector index (`944-970`).
- `replayed_through` is the compact vector length/end, not the last canonical event sequence.
- A snapshot event is inserted at vector index zero without a canonical event sequence.

Example: JSONL lines 1–4 where only line 4 projects produce vector `[snapshot, event_seq=4]`. A cursor `after_event_seq=2` incorrectly returns nothing because `2 >= vec.len()`, even though event 4 is newer.

Impact: valid events are skipped, reconnect can lose output, and MCP SSE inherits the loss.

Required correction: define one canonical monotonic sequence independent of vector position. Filter events by their actual sequence, return the last canonical sequence consumed, and explicitly model snapshot boundaries.

### R2-07 — [High][Confirmed] Global constant history epoch cannot detect rebuild/compaction

Evidence: every session and process uses `HISTORY_EPOCH = "epoch_1"` (`shell_session_actor_runtime.rs:65-70`, `project_summary_to_session:535`, `replay:936-943`).

Impact: stale cursors remain apparently valid after history rewrite, truncation, migration or compaction. Reconnect clients may silently combine incompatible histories.

Required correction: persist a per-history generation/epoch and rotate it whenever sequence identity can change. Test process restart without rewrite (same epoch) and compaction/rebuild (new epoch + explicit resync).

### R2-08 — [High][Confirmed] WebSocket default auth accepts an empty bearer

Evidence:

- `WsListenerConfig::default()` sets `require_auth: true` and `bearer_token: ""` (`ws_listener.rs:100-107`).
- Unlike MCP, `run_ws_listener` has no fail-closed empty-token check.
- `validate_bearer_header(Some("Bearer "), "")` succeeds because candidate and expected are equal empty strings.

Impact: a default-configured listener can be accessed with a trivial empty bearer. On explicit non-loopback bind this exposes the control plane over cleartext.

Required correction: refuse to bind when auth is required and the trimmed token is empty; add default/whitespace regression tests matching the MCP fix.

### R2-09 — [High][Confirmed] WebSocket overload silently drops RPC responses without resync

Evidence: `serve_connection` uses `try_send`; when full it increments a counter and discards the response (`ws_listener.rs:240-278`). Error envelopes and binary-frame errors are also best-effort-dropped without incrementing the counter (`281-304`). No resync event, close code or client-visible failure is sent.

Impact: clients can wait indefinitely for a request ID whose response was dropped. Mutations may have executed, so blind retry can duplicate non-idempotent turns. This violates the intended explicit slow-client resync behavior.

Required correction: never silently drop request responses. Reserve capacity/separate response and event queues, apply bounded backpressure, or close with an explicit retry/resync reason. Only droppable events may be coalesced, with a guaranteed resync marker.

### R2-10 — [High][Confirmed] MCP session and event storage is unbounded

Evidence:

- `McpHttpState.sessions` is an unbounded `HashMap` (`http_server.rs:146`).
- Every initialize creates and inserts a new session (`354-359`).
- `McpSession.events` is an unbounded `Vec`; `append_event` never evicts (`90-119`).
- There is no TTL, maximum session count, maximum events/bytes, background cleanup or disconnect cleanup.

Impact: any authenticated client can exhaust memory through repeated initialize calls and event-producing tool calls. If auth is explicitly disabled, the attack is unauthenticated on the bound interface.

Required correction: add bounded session admission, inactivity expiry, per-session byte/event retention, explicit resync beyond retention and cleanup on shutdown/delete.

### R2-11 — [High][Confirmed] MCP “Streamable HTTP” GET is a finite replay, not a live stream

Evidence: `get_mcp` snapshots `events_after`, wraps it in `stream::iter`, and immediately ends (`412-468`). `pull_facade_events` runs only after POST `tools/call` and fetches only one replay page (`569-589`). The ledger itself admits “SSE live push” is residual.

Impact: clients connected to GET receive no subsequent events; events produced asynchronously by an ongoing Turn are not delivered. More than one replay page can be permanently omitted because `next_cursor` is ignored.

Required correction: implement a bounded live publisher/subscriber per transport session, drain all replay pages before attaching live, guarantee replay/live boundary ordering, and expose explicit resync on lag.

### R2-12 — [High][Confirmed] MCP accepts malformed JSON-RPC shapes and leaks sessions on initialize notifications

Evidence:

- POST parses arbitrary JSON but does not validate object shape, `jsonrpc: "2.0"`, method presence/type, batch rejection or params schema (`343-365`).
- Any request whose method string is `initialize` allocates a session before checking whether `id` exists.
- Notifications then return 202 without echoing the generated session ID (`378-380`), leaving an unreachable session in the unbounded map.

Impact: protocol conformance is weak and an authenticated client can leak sessions cheaply using initialize notifications.

Required correction: reuse a strict shared MCP envelope validator, reject initialize notifications if the protocol forbids them, and allocate a session only after successful validation/dispatch of a request that can receive the session header.

### R2-13 — [High][Confirmed] Provider work is not a complete OpenRouter/Groq/Cloudflare vertical

Evidence:

- `ByokAuthProvider::list_models` always returns an empty `Unknown` catalog (`auth_provider.rs:158-173`).
- No product composition maps a BYOK `provider_binding` into Shell Turn execution; status admits this is PARTIAL.
- Cloudflare endpoint resolution requires account metadata (`220-248`), but `run_api_key_login` stores `ProviderAccountInfo::default()` and the CLI accepts no account ID (`login_coordinator.rs:255-270`, pager main BYOK arm).
- `run_api_key_login` still hardcodes `SecretBackendKind::Ephemeral` in metadata even when the CLI injects `FileCredentialStore`.

Impact: Cloudflare requests cannot resolve after normal login; catalog discovery is absent; the stored credential backend metadata is misleading; and no real Turn selects these providers.

Required correction: implement provider-specific onboarding inputs and durable backend metadata, model discovery/catalog binding, canonical `ProviderBinding` projection and real request execution tests. Keep live checks SKIP until credentials exist.

## 4. Medium-severity findings

### R2-14 — [Medium][Confirmed] The “real actor” test is an artificial command consumer

Evidence: C1 turn tests create a custom `mpsc` consumer that matches `SessionCommand` and writes selected updates. They do not instantiate the existing `SessionActor`, `SessionThread`, `LocalSet` or production spawn factory. The ledger says “real cmd_tx consumer (NOT FakeRuntime)” and sometimes describes it as a “real actor path”.

Impact: the tests prove enum/channel compatibility, not that production actor lifecycle, locks, prompt IDs, persistence, cancellation or failure behavior match the adapter assumptions.

Required correction: rename the fixture honestly and add integration coverage using the actual actor/spawn path with hermetic offline dependencies.

### R2-15 — [Medium][Confirmed] First Turn ordinal is off by one

Evidence: for a fresh session `ensure_resident` seeds `next_ordinal` to `max(num_messages, 1) = 1` (`355-368`); `next_ordinal` then performs `fetch_add(1) + 1` (`404-410`), returning 2 for the first Turn despite the comment saying ordinals start at 1.

Impact: wire ordering begins incorrectly and restart seeding conflates message count with turn count.

Required correction: store the next value consistently and seed it from actual persisted Turn count, not message count.

### R2-16 — [Medium][Confirmed] Synthesized steer Item is noncanonical and not persisted

Evidence: `steer_turn` sends `Interject` fire-and-forget, then fabricates an `AgentMessage` Item with a local counter and Completed status (`831-881`). It does not wait for actor acknowledgement or persist the Item. The counter is seeded from message count and is unrelated to JSONL line/replay sequence.

Impact: clients observe a successful agent-authored item that may never have been processed and cannot be replayed after reconnect.

Required correction: project the canonical persisted actor event/ack; do not invent an agent message for user steering input.

### R2-17 — [Medium][Confirmed] Actor-spawn lock maps grow forever

Evidence: `spawn_locks` inserts one `Arc<TokioMutex>` per session and never removes it (`334-342`). Resident handles also have no archive/shutdown eviction path because archive is unsupported.

Impact: long-running processes accumulate per-session memory even after sessions are no longer used.

Required correction: remove per-session locks after successful/terminal spawn coordination and implement lifecycle eviction tied to actor/session closure.

### R2-18 — [Medium][Confirmed] Persisted timestamp conversion can wrap negative values

Evidence: `project_summary_to_session` casts signed `timestamp_millis()` directly to `u64` (`547-548`). Pre-epoch/corrupt timestamps become enormous future times.

Required correction: use checked conversion or clamp to zero and surface corrupt metadata diagnostics.

### R2-19 — [Medium][Confirmed] WebSocket bind-host parsing mishandles IPv6

Evidence: `config.bind.split(':').next()` returns `"["` for `[::1]:port`, so loopback IPv6 is classified as non-loopback and receives the unsafe warning (`ws_listener.rs:139-143`). MCP duplicates the same parsing approach.

Required correction: parse `SocketAddr` first and call `ip().is_loopback()`; handle hostname resolution separately and fail conservatively.

### R2-20 — [Medium][Confirmed] WebSocket accept-loop can busy-spin forever

Evidence: every `listener.accept()` error executes `continue` without delay or terminal classification (`ws_listener.rs:151-165`). A persistent listener failure produces a hot loop with no observability.

Required correction: log and terminate on permanent listener errors or add bounded retry/backoff for explicitly transient cases.

### R2-21 — [Medium][Confirmed] MCP DELETE does not execute lifecycle cancellation

Evidence: DELETE only removes transport-session metadata (`http_server.rs:471-487`). It does not cancel an active Turn, release leases or detach a live subscriber. STATUS already lists disconnect cancellation as residual.

Impact: clients may believe termination stopped work while the runtime continues consuming resources.

Required correction: specify DELETE semantics and route cancellation/detach through the authoritative runtime before returning success.

### R2-22 — [Medium][Confirmed] TDD artifacts demonstrate sensitivity, not reliable chronology

Evidence: the C3 RED log is described as produced “with handshake auth stubbed”; C4 has no RED log and reviewers infer compile-fail RED after implementation. This can prove a test discriminates a mutation, but not that the test existed and failed before production code as required by strict TDD.

Required correction: capture commit/order evidence for future slices. Do not retroactively label mutation testing as test-first TDD.

### R2-23 — [Medium][Confirmed] Ledger state is stale and internally contradictory

Evidence:

- `BLOCKERS.md` still lists real WebSocket, MCP HTTP and provider registration as blockers although STATUS says those slices are done/partial.
- STATUS reports spawned work and green totals for uncommitted files that can change during review.
- Several reviews issue `PASS_WITH_FINDINGS` despite missing required RED artifacts or real product wiring.

Required correction: distinguish `IMPLEMENTED_DIRTY`, `TESTED_SNAPSHOT`, `COMMITTED`, `REVIEWED` and `ACCEPTED`; refresh blockers after every state transition and never assign PASS to an acceptance criterion with acknowledged missing mandatory evidence.

### R2-24 — [Medium][Confirmed] Canonical Tower instance contract remains unfixed

Evidence: `app_server_composition.rs:36-46` still reads `GROK_TOWER_INSTANCE`, returns unvalidated strings and has a non-hermetic fallback test. The canonical contract specifies `GROK_OSS_TOWER` and `TowerInstanceId` validation.

Required correction: complete corrective Wave C2 before any multi-instance claim.

## 5. Lower-severity quality findings

### R2-25 — [Low][Confirmed] New code introduces warnings

The review check reports unused `AuthProvider` imports in multi-auth and an unused `process_mcp_stdio_batch` import, in addition to pre-existing sampling warnings and the multi-bin manifest warning. The corrective contract requires introduced warnings to be resolved or justified.

### R2-26 — [Low][Confirmed] MCP content-type validation is substring-based

`is_json_content_type` accepts any header containing `application/json`, including malformed media types. Parse the media type and accept the explicit JSON types intended by the contract.

### R2-27 — [Low][Confirmed] Registration errors are silently discarded

Provider registry construction repeatedly calls `.register(...).ok()`. A duplicate or invalid registration silently changes the provider surface. Static construction should fail loudly or assert invariants.

## 6. Acceptance matrix

| Area | Status | Evidence summary |
|---|---|---|
| Protocol/schema foundation | PASS for current experimental schema | Package tests and generated-schema checks exist |
| Single production runtime authority | FAIL | No real production spawner; success-before-spawn |
| Real SessionActor mapping | PARTIAL | Channel seam exists; actual actor not exercised/product-wired |
| Durable session/turn idempotency | FAIL | session map volatile/racy; turn key unused |
| Canonical replay/history | FAIL | constant epoch and index/sequence mismatch |
| WebSocket listener | PARTIAL / security-blocked | real listener exists dirty; empty-bearer and response-drop defects |
| MCP Streamable HTTP | PARTIAL | real server exists dirty; finite SSE, unbounded state, weak envelope validation |
| Tower instance/lifecycle | PARTIAL | canonical env/type and dual-process gates pending |
| OpenRouter/Groq/Cloudflare | PARTIAL | auth providers registered; catalog/account/composition Turn missing |
| Approvals/interactions | FAIL | production `respond_interaction` unsupported |
| Tool parity | PARTIAL | semantic core exists; real runtime/product transport composition incomplete |
| SDK/end-to-end conformance | PARTIAL | fake/local gates exist; real multi-transport matrix absent |
| Strict TDD evidence | FAIL | chronology unproven for multiple slices |
| Required user methodology | FAIL | subagents explicitly used |
| Merge/release readiness | FAIL / BLOCKED | dirty state plus Critical/High findings |

## 7. Validation executed

Command started against the volatile snapshot:

```text
cargo check \
  -p xai-grok-shell \
  -p xai-grok-app-server --features xai-grok-app-server/websocket \
  -p xai-grok-mcp-server --features xai-grok-mcp-server/streamable-http \
  -p xai-grok-multi-auth \
  -p xai-grok-pager-bin
```

Result: **exit 0** after 1m52s. Compilation passed for the selected surfaces. Reported warnings are recorded in R2-25. A successful type/build check does not exercise the blocking runtime behaviors above.

Previously captured implementer logs were inspected but not accepted as proof beyond their exact tested surfaces. In particular, FakeRuntime tests, custom command consumers and finite HTTP/SSE tests do not prove production actor/live-stream behavior.

Targeted behavior gate attempted afterward:

```text
cargo test -p xai-grok-shell --test c1_shell_port --test c1_turn_lifecycle --no-fail-fast &&
cargo test -p xai-grok-app-server --features websocket ws_listener --no-fail-fast &&
cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http --no-fail-fast &&
cargo test -p xai-grok-multi-auth --test byok_providers --no-fail-fast
```

Result: **exit 101 before tests executed**, due to R2-00 (`PathBuf` not found in `xai-grok-tower/src/instance.rs:101`). The worktree changed during review; therefore the earlier exit-0 `cargo check` and this later failure are both accurate for their respective volatile snapshots.

## 8. Final gate

**FAIL / BLOCKED.** Do not merge, publish or mark the corrective program COMPLETE. The minimum safe order is:

1. stop delegated agents and freeze a committed snapshot;
2. fix WebSocket empty-token auth before any listener exposure;
3. wire a real production actor spawner and propagate spawn failure;
4. implement durable session/turn idempotency;
5. repair canonical epoch/event cursor semantics;
6. bound MCP sessions/events and implement replay-to-live delivery;
7. complete provider account/catalog/composition wiring;
8. add real actor and real multi-transport tests;
9. rerun warning-free checks and a new independent review performed without violating the user’s no-subagent constraint.

No production source was modified by this review. Only this review document was added.
