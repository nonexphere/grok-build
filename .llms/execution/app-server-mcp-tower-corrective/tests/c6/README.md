# C6-A Tower tools ACL + cross-surface parity — RED→GREEN evidence

| Field | Value |
|---|---|
| Wave | C6-A (items 41–43 partial) |
| Agent | build (glm-5.2) |
| Branch | `goblin-implement-epic-tree` |
| Date | 2026-07-19 |
| Owned crate | `xai-grok-tower-tools` |
| Adapter | `FakeRuntime` (real semantic core via `GrokRuntimeFacade`) |

## Scope

Prove all nine `tower_agent_*` tools through the shared semantic core
(`invoke_tower_tool`) with fail-closed ACL, normalized error shapes, and
in-process vs MCP parity notes. No shell session actor rewrites; no WS/MCP
server rewrites — tools call through the facade only.

## RED (pre-implementation)

`cargo test -p xai-grok-tower-tools --test c6_tools_acl`
→ **23 passed; 1 failed** (see `c6_tools_acl_RED.log`).

The failing test, `c6_tower_agent_wait_invoke_path`, exposed a contract
divergence in the `tower_agent_wait` tool implementation:

- `events` was emitted as a **count** (`page.events.len()`, a JSON number)
  but the normative schema `tower_agent_wait_output` requires `events` to be
  an **array of objects**.
- `wakeReason` returned the literal `"events"` (plural), which is **not** a
  member of the schema enum
  `["event","terminal","interaction","timeout","resync_required"]`.

The other 23 tests (ACL deny for all nine tools, invoke paths for the other
eight tools, fail-closed default, idempotency, swarm, forbidden-hub absence,
no-MCP-loop) passed against the pre-existing implementation, confirming the
ACL boundary and parity invariants were already in place; only the
`tower_agent_wait` output shape diverged from the schema.

## GREEN (post-implementation)

Fix: `tower_agent_wait` now forwards the projected `RuntimeEvent` objects as
a JSON array (via `project_runtime_event_to_json`) and reports a
schema-valid `wakeReason` (`"event"` when events are present, `"timeout"`
otherwise). It also honors the optional `historyEpoch` argument.

`cargo test -p xai-grok-tower-tools --test c6_tools_acl`
→ **24 passed; 0 failed** (see `c6_tools_acl_GREEN.log`).

Full tower-tools crate:
`cargo test -p xai-grok-tower-tools`
→ **11 lib + 24 c6 = 35 passed; 0 failed**.

Vertical contract:
`cargo test -p xai-grok-app-server-protocol -p xai-grok-tower -p xai-grok-tower-tools -p xai-grok-mcp-server`
→ protocol 22, tower 22 + 4 isolation, tower-tools 11 + 24 c6, mcp-server 11
(all green; see `c6_vertical_GREEN.log`).

MCP server with streamable-http feature:
`cargo test -p xai-grok-mcp-server --features streamable-http`
→ **15 lib + 27 streamable_http; 0 failed** (see `c6_mcp_server_GREEN.log`).

`cargo clippy -p xai-grok-tower-tools --test c6_tools_acl` → no new warnings
in the C6 test target (pre-existing warnings in other crates and in the
pre-existing `adapter_parity_mcp_and_in_process_normalized` test are
unchanged and out of scope).

## Tests (24) — contract coverage

| # | Test | Asserts |
|---|---|---|
| 1 | `c6_all_nine_descriptors_have_input_and_output_schema` | 18 `$defs` resolve; nine unique names |
| 2 | `c6_acl_is_fail_closed_by_default` | built-in non-orchestrator + unknown agents denied; orchestrator + explicit opt-in allowed |
| 3 | `c6_acl_denies_every_tool_before_target_lookup` | every tool returns `forbidden` for both existing and missing targets; identical code (no existence leak) |
| 4 | `c6_tower_agent_list_invoke_path` | `sessions` array + `nextCursor` |
| 5 | `c6_tower_agent_start_invoke_path` | `operationId`/`state`/`sessionId`; no provider credentials |
| 6 | `c6_tower_agent_send_new_turn_invoke_path` | `turnId` string returned |
| 7 | `c6_tower_agent_send_new_turn_rejects_turn_id` | `new_turn` + `turnId` → `invalid_params` |
| 8 | `c6_tower_agent_send_steer_active_invoke_path` | steer returns same `turnId` |
| 9 | `c6_tower_agent_send_steer_active_requires_turn_id` | steer without `turnId` → `invalid_params` |
| 10 | `c6_tower_agent_history_invoke_path` | `historyEpoch`/`items`/`redacted=true` |
| 11 | `c6_tower_agent_resume_invoke_path` | `operationId`/`state`/`sessionId` |
| 12 | `c6_tower_agent_wait_invoke_path` | `events` array + schema-valid `wakeReason` |
| 13 | `c6_tower_agent_interrupt_invoke_path` | interrupt active turn → `completed` |
| 14 | `c6_tower_agent_archive_invoke_path` | archive → `completed` |
| 15 | `c6_tower_agent_status_invoke_path` | sessionRow shape; no provider credentials |
| 16 | `c6_custom_explicit_opt_in_is_allowed` | custom agent with `tower_access=true` allowed |
| 17 | `c6_idempotency_start_replays_same_session` | same key → same `sessionId` |
| 18 | `c6_idempotency_send_replays_same_turn` | same key → same `turnId` |
| 19 | `c6_idempotency_key_conflict_on_diverging_input` | same key + different workspace → `idempotency_conflict` |
| 20 | `c6_swarm_n_sessions_without_hub` | 5 independent sessions; no hub entity |
| 21 | `c6_forbidden_hub_symbol_absent` | no `tower_agent_hub` in names or production source |
| 22 | `c6_in_process_path_has_no_mcp_loop` | core returns structured JSON (not RPC envelope); no MCP dependency in manifest |
| 23 | `c6_unknown_tool_is_method_not_found` | `tower_agent_hub` → `method_not_found` |
| 24 | `c6_invalid_params_when_workspace_root_missing` | start without `workspaceRoot` → `invalid_params` |

## Parity notes (in-process vs MCP)

- Both adapters call the same `invoke_tower_tool` semantic core over the
  same `GrokRuntimeFacade`; neither reinterprets success or errors.
- The in-process path returns structured JSON values directly (no JSON-RPC
  envelope), asserted by `c6_in_process_path_has_no_mcp_loop`.
- `xai-grok-tower-tools` has no dependency on `xai-grok-mcp-server`
  (asserted via the Cargo manifest), so the product cannot form a local
  self-MCP edge.
- The `tower_agent_hub` symbol is absent from both the tool name list and
  the production source of `lib.rs`.

## Residual (outside this slice)

- Live MCP-server differential fixture execution against an external MCP
  client (C4-B/C4-E surface) is not re-run here; the MCP server's own
  streamable-http suite (27 tests) is green and consumes the same
  descriptors.
- `tower_agent_history` `maxBytes` enforcement and redaction canary fixtures
  belong to the projection slice (TA101-03, already done) and are not
  duplicated here.
