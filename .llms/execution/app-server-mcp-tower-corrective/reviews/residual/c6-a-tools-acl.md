# Residual review — C6-A Tower tools ACL + cross-surface parity

| Field | Value |
|---|---|
| Wave | C6-A (items 41–43 partial) |
| Mode | implementation review (residual) |
| Reviewer | review harness (read-only, glm-5.2) |
| Date | 2026-07-19 |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

All nine tools proven through the shared semantic core with fail-closed ACL,
normalized error shapes, and in-process vs MCP parity. A genuine contract
divergence in `tower_agent_wait` was caught by RED and fixed (events array +
schema-valid `wakeReason`). Findings are Low; the slice is honestly scoped.

## Severity summary

- Critical: 0
- High: 0
- Medium: 0
- Low: 3 (F-1, F-2, F-3)

## Contract non-negotiables (re-checked)

- **No `tower_agent_hub` / forbidden hub.** `c6_forbidden_hub_symbol_absent`
  + `c6_unknown_tool_is_method_not_found` pass; symbol absent from names and
  production source (`xai-grok-tower-tools/src/lib.rs`). PASS.
- **No local MCP self-loop.** `c6_in_process_path_has_no_mcp_loop` asserts
  the in-process path returns structured JSON (no JSON-RPC envelope) and the
  crate manifest has no `xai-grok-mcp-server` dependency. PASS.
- **No second actor / Tower ≠ Shell.** No shell session actor rewrites; tools
  call through the facade only. `FakeRuntime` is the authoritative test
  adapter over `GrokRuntimeFacade` (matches C2-A / TA101 contract); the
  production path injects a Shell-backed facade (not touched here). PASS.
- **Fail-closed ACL.** `c6_acl_is_fail_closed_by_default` denies every
  built-in non-orchestrator agent + unknown; only `orchestrator` or explicit
  `tower_access=true` allowed. `c6_acl_denies_every_tool_before_target_lookup`
  asserts identical `forbidden` code for existing vs missing targets (no
  existence leak). PASS.
- **Secrets.** Invoke-path tests assert no provider credentials in
  `tower_agent_start` / `tower_agent_status` outputs. PASS.

## Evidence reviewed

- Wave note: `.llms/execution/app-server-mcp-tower-corrective/waves/c6-tools-acl.md`
- Handoff: `.llms/.../handoffs/HANDOFF-C6-A-tools-acl.md`
- RED: `.llms/.../tests/c6/c6_tools_acl_RED.log` (23 passed / 1 failed —
  `c6_tower_agent_wait_invoke_path` schema divergence: `events` was a count,
  `wakeReason` was `"events"` not in enum).
- GREEN: `.llms/.../tests/c6/c6_tools_acl_GREEN.log` (24/24 pass).
- Vertical: `tests/c6/c6_vertical_GREEN.log` (protocol 22, tower 22+4,
  tower-tools 11+24, mcp-server 11).
- MCP server streamable-http: `tests/c6/c6_mcp_server_GREEN.log` (15 lib +
  27 streamable_http).

## Findings

### F-1 — `tower_agent_list` filter/pagination not asserted (Low, high confidence)
The facade `list_sessions` returns all sessions; the tool does not yet apply
`workspaceRoot`/`agentType`/`status`/`includeArchived`/`pageSize`/cursor
filters. The wave documents this as a follow-on within C6 if the contract is
re-opened. Acceptable for this slice (filter semantics were not in the
acceptance bound), but the descriptor advertises these params.

### F-2 — `FakeRuntime` is the test adapter, not production (Low, high confidence)
The 24 tests exercise `invoke_tower_tool` over `FakeRuntime` (a real
`GrokRuntimeFacade` adapter). Production injects a Shell-backed facade; this
slice does not touch Shell. This matches the C2-A / TA101 contract and is
honestly stated. Residual: a live Shell-backed differential run is out of
scope; the MCP server's own 27-test streamable-http suite is green and
consumes the same descriptors.

### F-3 — Live external MCP-client differential not re-run (Low, medium confidence)
The wave notes live external MCP-client differential fixture execution
(C4-B/C4-E surface) is not re-run here; the MCP server suite is green.
Acceptable, but the cross-surface parity claim rests on the shared
descriptors + semantic core, not on an external client differential.

## Required fixes

None for this wave's bounded scope.

## Residual risk / dependencies

- `tower_agent_list` filter/pagination/cursor semantics if the contract is
  re-opened.
- `tower_agent_history` `maxBytes` byte-accounting + redaction canary
  fixtures belong to the projection slice (TA101-03, done) — not duplicated.

## Commands / results

- `cargo test -p xai-grok-tower-tools --test c6_tools_acl` → 24 passed; 0 failed (`c6_tools_acl_GREEN.log`).
- `cargo test -p xai-grok-tower-tools` → 11 lib + 24 c6 = 35 passed; 0 failed.
- `cargo test -p xai-grok-mcp-server --features streamable-http` → 15 lib + 27 streamable_http; 0 failed.
- `cargo clippy -p xai-grok-tower-tools --test c6_tools_acl` → no new warnings in C6 target.
