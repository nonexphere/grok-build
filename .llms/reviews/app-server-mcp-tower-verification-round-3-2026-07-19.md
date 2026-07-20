# Verification round 3 — App Server / MCP / Tower corrective program

**Date:** 2026-07-19  
**Branch:** `goblin-implement-epic-tree`  
**Committed HEAD:** `71a3c805e6e6c0083193dcfc19dd0b8521bc53ae`  
**Compared against:** `.llms/reviews/app-server-mcp-tower-code-review-round-2-2026-07-18.md`  
**Reviewed state:** committed HEAD plus current staged, unstaged and untracked implementation  
**Verdict:** **FAIL / BLOCKED**  
**Method:** primary-agent-only code review; no subagents used by this verifier.

## 1. Result summary

The previous compile failure was corrected and the current feature-enabled `cargo check` passes. Tower instance resolution and MCP empty-bearer handling improved. Those corrections are outweighed by unresolved production blockers and a new security regression: both new network startup paths print the complete bearer token to stderr.

The worktree remains highly volatile and uncommitted. It contains staged and unstaged changes in the same files (`MM`) plus many untracked production files and tests. Ledger PASS/GREEN claims therefore do not identify one immutable reviewed artifact.

## 2. New findings

### V3-01 — [Critical][Confirmed] Bearer credentials are printed verbatim to stderr

Evidence:

- `xai-grok-pager-bin/src/main.rs:124-127` prints the App Server WebSocket token.
- `main.rs:163-166` prints the MCP HTTP token.
- Both functions run during listener startup (`main.rs:1414` and `1458`).

Impact: credentials can enter terminal capture, CI logs, service-manager journals, support bundles, telemetry attachments and shared shell history/screens. Anyone with log access can control the exposed agent surface.

Required correction: never print the token. Print only where it was loaded/generated and a non-secret fingerprint if operationally necessary. Deliver generated credentials through a permission-restricted token file or explicit secure output mechanism that the user knowingly requests.

### V3-02 — [High][Confirmed] WebSocket product path still accepts an empty bearer

Evidence:

- `WsListenerConfig::default` and the product builder use `require_auth: true` with caller-provided token.
- `run_ws_listener` has no empty/whitespace token validation before bind (`ws_listener.rs:126-146`).
- Constant-time equality accepts `Authorization: Bearer ` when expected token is empty.
- The new product composition tests only assert `require_auth == true`; they do not test empty-token refusal. The empty-bearer test at composition lines 861+ is for MCP, not WebSocket.

Impact: `GROK_OSS_APP_SERVER=1` with an empty secret starts an effectively unauthenticated listener.

Required correction: add the same bind-time fail-closed validation already implemented for MCP, plus direct and product-composition regression tests.

### V3-03 — [High][Confirmed] Product network composition advertises a real runtime but still cannot execute prompts

Evidence:

- WS product path calls `experimental_app_server_processor()` → `ShellSessionActorRuntime::new` (`app_server_composition.rs:21-34`, `106-112`).
- MCP product path calls `experimental_mcp_http_runtime_with_root` → `ShellSessionActorRuntime::new` (`183-196`).
- Neither path uses `with_production_spawn`; repository search still finds no `spawn_session_on_thread` assembly in product composition.
- Ledger explicitly says the factory assembly is PARTIAL.

Impact: network clients can initialize and create durable session metadata, but real Turn execution remains unsupported. The new black-box composition tests stop after `session/start`/`tower_agent_start`, so they avoid the failing boundary.

Required correction: either wire the existing actor factory and test a real prompt, or prevent these product listener modes from claiming/returning runnable sessions.

### V3-04 — [High][Confirmed] Invalid Tower configuration is silently downgraded in the actual CLI path

Evidence:

- `resolve_tower_instance_id` correctly fails on invalid input.
- `select_tower_instance_id` catches every error and returns `default` (`app_server_composition.rs:292-302`).
- MCP CLI startup calls this fail-soft wrapper (`main.rs:1412-1415`).
- No product `--tower <id>` argument is wired; STATUS acknowledges this.

Impact: a typo or invalid `GROK_OSS_TOWER` can connect the operator to the default Tower, violating fail-closed isolation and potentially targeting the wrong sessions.

Required correction: use the fallible resolver in the CLI and surface a clear configuration error. Remove the legacy/fail-soft wrapper after migration or confine it to non-security-sensitive display code.

### V3-05 — [High][Confirmed] History projection expanded but cursor loss bug remains

Evidence:

- `project_updates` increments `seq` for every physical line but omits unsupported/corrupt updates from `events`.
- `replay` still converts `after_event_seq` to a vector index and slices the compact event vector (`shell_session_actor_runtime.rs:1273-1297`).
- `replayed_through` remains the vector end, not the last canonical line/event sequence.

Impact: reconnect can permanently skip newer projected events whenever earlier JSONL lines were unprojected. Adding more projector variants reduces frequency but does not fix the model.

Required correction: carry canonical sequence alongside every projected event and filter by sequence rather than vector position.

### V3-06 — [Medium][Confirmed] Network feature selection is implicit and order-dependent

Evidence: in `AgentCmd::Serve`, MCP is checked first and returns; WebSocket is checked second. If both `GROK_OSS_MCP_HTTP` and `GROK_OSS_APP_SERVER` are truthy, MCP silently wins.

Impact: configuration mistakes produce surprising exposure and the operator may believe both or the other service started.

Required correction: reject mutually enabled modes or introduce one explicit CLI enum selecting exactly one transport.

### V3-07 — [Medium][Confirmed] Environment-mutating tests are not globally serialized

Evidence: the App Server env-gate test mutates process-wide environment and states it is not serial because it is the only test “here”. Other tests/process modules can read the same environment concurrently, and the corrective program adds several environment-based Tower/transport tests.

Impact: flaky tests and cross-test configuration contamination are possible.

Required correction: centralize environment guards and serialize every test touching the same process environment.

## 3. Resolution matrix for round-2 findings

| Prior finding | Current status | Verification |
|---|---|---|
| R2-00 compile failure | **FIXED** | `PathBuf` imported; feature-enabled `cargo check` exit 0 |
| R2-01 subagent prohibition | **OPEN / worsened** | More GLM handoffs C3-G/C4-F/C6-A/C7-A were added |
| R2-02 no real production Turn | **OPEN** | Product still constructs `ShellSessionActorRuntime::new` |
| R2-03 success after spawn failure | **OPEN** | `ensure_resident` still returns `()` and errors are only recorded/logged |
| R2-04 Session idempotency racy/volatile | **OPEN** | Same process-local check-await-insert map |
| R2-05 Turn idempotency ignored | **OPEN** | `start_turn` still ignores `idempotency_key` |
| R2-06 replay cursor mismatch | **OPEN** | Projector expanded; vector-index cursor remains |
| R2-07 constant history epoch | **OPEN** | `HISTORY_EPOCH = "epoch_1"` remains global |
| R2-08 WS empty bearer | **OPEN** | No bind-time validation; see V3-02 |
| R2-09 WS silent response drop | **OPEN** | `try_send` still discards responses |
| R2-10 unbounded MCP state | **OPEN** | Unbounded sessions map and event vectors remain |
| R2-11 finite MCP SSE | **OPEN** | GET still uses finite `stream::iter`; one replay page only |
| R2-12 MCP envelope/session leak | **OPEN** | Strict JSON-RPC validation/allocation order unchanged |
| R2-13 incomplete providers | **OPEN** | Empty catalog, default Cloudflare account, ephemeral metadata, no Turn binding |
| R2-14 artificial actor fixture | **OPEN** | New tests still use custom command consumers, not actual SessionActor |
| R2-15 first ordinal off by one | **OPEN** | seed 1 plus `fetch_add + 1` still yields 2 |
| R2-16 synthetic steer Item | **OPEN** | Still fabricated, unacknowledged and unpersisted |
| R2-17 spawn-lock/resident growth | **OPEN** | No eviction/removal lifecycle added |
| R2-18 negative timestamp wrap | **OPEN** | Direct signed-to-u64 cast remains |
| R2-19 IPv6 host parsing | **OPEN** | `split(':').next()` remains in listeners/startup output |
| R2-20 WS accept busy-spin | **OPEN** | Persistent accept errors still `continue` immediately |
| R2-21 MCP DELETE lifecycle | **OPEN** | Removes transport map entry only |
| R2-22 TDD chronology | **OPEN** | New RED logs again describe deliberately stubbed behavior |
| R2-23 stale ledger | **OPEN** | BLOCKERS still lists WS/MCP/providers broadly despite partial completion |
| R2-24 Tower instance contract | **PARTIAL** | Canonical resolver/type added; CLI remains fail-soft/no `--tower` |
| R2-25 warnings | **OPEN** | Current check emits new unused-import warnings |
| R2-26 content-type substring | **OPEN** | MIME check unchanged |
| R2-27 ignored registry errors | **OPEN** | Registration still discards errors with `.ok()` |

Summary: **1 fixed, 1 partial, 26 open/worsened** from the prior 28 findings.

## 4. Validation

Executed:

```text
cargo check \
  -p xai-grok-shell \
  -p xai-grok-tower \
  -p xai-grok-app-server --features xai-grok-app-server/websocket \
  -p xai-grok-mcp-server --features xai-grok-mcp-server/streamable-http \
  -p xai-grok-multi-auth \
  -p xai-grok-pager-bin --features app-server-ws,mcp-streamable-http
```

Result: **PASS, exit 0** in 14.06s. Warnings remain for unused imports in multi-auth and MCP stdio, plus sampling and multi-bin warnings.

A targeted chained test gate was also executed:

```text
cargo test -p xai-grok-shell \
  --test c1_shell_port --test c1_turn_lifecycle \
  --test c1_production_spawn --test c3_history_projection --no-fail-fast
cargo test -p xai-grok-app-server --features websocket ws_listener --no-fail-fast
cargo test -p xai-grok-mcp-server --features streamable-http \
  --test streamable_http --no-fail-fast
cargo test -p xai-grok-multi-auth --test byok_providers --no-fail-fast
cargo test -p xai-grok-tower --lib --test tower_instance_isolation --no-fail-fast
```

Result: **PASS, exit 0 — 136 tests passed**: Shell 50, WebSocket-filtered App Server 16, MCP HTTP 27, BYOK 17, Tower 26. Warnings remain. These tests validate their bounded fixtures and network helpers, but do not invalidate the production-boundary findings: several suites intentionally assert `unsupported`, use synthetic command consumers/FakeRuntime, omit empty-bearer WS, and do not test durable idempotency or canonical cursor gaps.

## 5. Gate decision

**FAIL / BLOCKED.** The compile regression was repaired, but the implementation remains unsafe to merge or expose. Immediate priority:

1. remove all bearer-token printing;
2. fail-close WebSocket on empty credentials;
3. stop further subagent execution per user instruction;
4. freeze and commit one reviewable snapshot;
5. wire or explicitly disable the nonfunctional production Turn path;
6. fix durable idempotency and replay sequence semantics;
7. bound MCP state and implement replay-to-live delivery;
8. rerun tests and review from the immutable commit.

No production code was modified by this verification. Only this report was added.
