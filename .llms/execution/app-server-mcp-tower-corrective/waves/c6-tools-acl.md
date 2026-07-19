# C6-A — Tower tools ACL + cross-surface parity (wave note)

| Field | Value |
|---|---|
| Handoff | `handoffs/HANDOFF-C6-A-tools-acl.md` |
| Wave | C6-A (items 41–43 partial) |
| Agent | build (glm-5.2) |
| Branch | `goblin-implement-epic-tree` |
| Status | **REAL** for the nine-tool ACL + invoke-path + parity slice |
| Owned | `xai-grok-tower-tools` (src + tests) + ledger `tests/c6/*`, `waves/c6-tools-acl.md` |
| Must NOT | shell session actor rewrites; WS/MCP server rewrites (tools call through facade only) |

## Scope delivered

1. **All nine tools proven through the shared semantic core** —
   `invoke_tower_tool` over `FakeRuntime` (a real `GrokRuntimeFacade`
   adapter). A new integration test file
   `crates/codegen/xai-grok-tower-tools/tests/c6_tools_acl.rs` exercises an
   invoke (happy) path for each of the nine tools with structured-output
   assertions, plus an ACL-deny path per tool.
2. **Fail-closed ACL** — `is_authorized` denies every built-in
   non-orchestrator agent (`build`, `review`, `explore`, `repo-explore`,
   `architect`, `general`, unknown) by default; only `orchestrator` or an
   explicit `tower_access=true` opt-in is allowed. ACL is evaluated before
   any target lookup: every tool returns `forbidden` with identical code for
   existing vs missing targets (no existence leak).
3. **Idempotency / limits** — retry with the same key replays the original
   `sessionId`/`turnId`; the same key with diverging input returns
   `idempotency_conflict`; swarm of N independent sessions works without a
   hub entity.
4. **Cross-surface parity** — the in-process path returns structured JSON
   (no JSON-RPC envelope); `xai-grok-tower-tools` has no dependency on
   `xai-grok-mcp-server` (no local self-MCP edge); the `tower_agent_hub`
   symbol is absent from names and production source. Both adapters share
   the same semantic core and descriptors.
5. **Contract fix** — `tower_agent_wait` output was diverging from the
   normative schema: `events` was a count (number) instead of an array of
   objects, and `wakeReason` returned `"events"` (not in the schema enum).
   Fixed to forward projected `RuntimeEvent` objects as a JSON array and
   report a schema-valid reason (`"event"`/`"timeout"`). The optional
   `historyEpoch` argument is now honored.

## Files changed

### Product code
- `crates/codegen/xai-grok-tower-tools/src/lib.rs` —
  `tower_agent_wait` now emits `events` as an array of projected
  `RuntimeEvent` objects and a schema-valid `wakeReason`; honors optional
  `historyEpoch`. Added `project_runtime_event_to_json` helper that
  forwards facade events as opaque structured objects (no re-interpretation).

### Tests
- `crates/codegen/xai-grok-tower-tools/tests/c6_tools_acl.rs` — **new**;
  24 integration tests covering all nine tools (invoke path + ACL deny),
  fail-closed default, idempotency (start/send replay + conflict), swarm
  limits, forbidden-hub absence, and no-MCP-loop parity.

### Ledger / evidence
- `.llms/execution/app-server-mcp-tower-corrective/tests/c6/README.md` —
  RED→GREEN evidence summary.
- `.llms/execution/app-server-mcp-tower-corrective/tests/c6/c6_tools_acl_RED.log`
  — 23 passed / 1 failed (`c6_tower_agent_wait_invoke_path` schema
  divergence).
- `.llms/execution/app-server-mcp-tower-corrective/tests/c6/c6_tools_acl_GREEN.log`
  — 24 passed / 0 failed.
- `.llms/execution/app-server-mcp-tower-corrective/tests/c6/c6_vertical_GREEN.log`
  — protocol 22, tower 22 + 4 isolation, tower-tools 11 + 24 c6, mcp-server
  11 (default features).
- `.llms/execution/app-server-mcp-tower-corrective/tests/c6/c6_mcp_server_GREEN.log`
  — mcp-server 15 lib + 27 streamable_http (streamable-http feature).
- `.llms/execution/app-server-mcp-tower-corrective/waves/c6-tools-acl.md` —
  this wave note.

## Validation commands and results

| Command | Result |
|---|---|
| `cargo test -p xai-grok-tower-tools --test c6_tools_acl` | **24 passed; 0 failed** |
| `cargo test -p xai-grok-tower-tools` | **11 lib + 24 c6 = 35 passed; 0 failed** |
| `cargo test -p xai-grok-app-server-protocol -p xai-grok-tower -p xai-grok-tower-tools -p xai-grok-mcp-server` | protocol 22; tower 22 + 4; tower-tools 11 + 24; mcp-server 11 — all green |
| `cargo test -p xai-grok-mcp-server --features streamable-http` | **15 lib + 27 streamable_http; 0 failed** |
| `cargo clippy -p xai-grok-tower-tools --test c6_tools_acl` | no new warnings in C6 target |

## Acceptance mapping

| Criterion | Evidence |
|---|---|
| Tests cover all nine tools (invoke path + ACL deny) | `c6_tools_acl.rs` tests 4–15 (invoke) + test 3 (ACL deny for all nine) |
| Fail-closed default | `c6_acl_is_fail_closed_by_default` + `is_authorized` |
| Idempotency/limits if already in contract | `c6_idempotency_*` (3 tests) + `c6_swarm_n_sessions_without_hub` |
| Wave note + evidence | this file + `tests/c6/*` logs |

## Assumptions

- `FakeRuntime` is the authoritative real semantic-core adapter for tests
  (production injects a Shell-backed facade; this slice does not touch
  Shell). This matches the C2-A / TA101 contract.
- The normative schema
  `xai-grok-app-server-protocol/schemas/tower-tools.schema.json` is the
  source of truth for output shapes; the `tower_agent_wait` fix realigns
  the implementation to it.
- MCP-server parity is validated by the MCP server's own streamable-http
  suite (27 tests green) consuming the same descriptors; a live external
  MCP-client differential run is out of scope for this slice.

## Residual (outside this slice)

- Live external MCP-client differential fixture execution (C4-B/C4-E
  surface) — not re-run here; the MCP server suite is green.
- `tower_agent_history` `maxBytes` byte-accounting + redaction canary
  fixtures belong to the projection slice (TA101-03, done).
- `tower_agent_list` filter/pagination/cursor semantics (workspaceRoot,
  agentType, status, includeArchived, pageSize, opaque cursor) are not
  asserted in this slice; the facade `list_sessions` returns all sessions
  and the tool does not yet apply filters. This is a follow-on within
  C6 if the contract is re-opened.
- No commits, pushes, or PRs (per harness policy; orchestrator owns
  integration).
