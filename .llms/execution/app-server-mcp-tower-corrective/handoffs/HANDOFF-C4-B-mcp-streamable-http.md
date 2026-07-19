# Handoff C4-B — Real MCP Streamable HTTP server (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Wave | C4 items 26–28 (framing + real server) |
| Capability | read-write under owned paths |
| Depends on | C4-A map; C1-G command routing landed |
| Branch | `goblin-implement-epic-tree` |

## Goal

Implement an actual Streamable HTTP server/router for POST/GET/DELETE `/mcp` over the shared Tower tool semantic core (`invoke_tower_tool`). Helper-only pure functions are insufficient.

## Read first

- `.llms/execution/app-server-mcp-tower-corrective/waves/c4-mcp-surface-map.md`
- Corrective contract § Wave C4
- `xai-grok-mcp-server` transport helpers
- Tower tools `invoke_tower_tool` + nine tool names

## Non-negotiables

- Shared semantic core via `GrokRuntimeFacade` / `invoke_tower_tool`
- No local MCP self-loop in production composition (add guard/test)
- Do not edit `xai-grok-shell/app_server_runtime/**`, multi-auth, or app-server WS listener
- RED→GREEN under `tests/c4/`
- For tool black-box, inject FakeRuntime or test facade — do not require live SessionActor

## Owned files

- `crates/codegen/xai-grok-mcp-server/**` (and related MCP server package paths from map)
- New integration tests for Streamable HTTP
- Ledger waves/tests for c4

## Acceptance

1. Real HTTP bind serving `/mcp` (feature-gated OK).
2. Black-box: POST initialize/tools/list/tools/call; auth failure; body limit; DELETE session.
3. SSE GET resume path at least with helper/table wired to a real transport (full real-adapter resync may PARTIAL).
4. Nine-tool descriptor parity with in-process names.
5. Wave note + evidence; honest PARTIAL for composition self-loop if product bin not yet wired.

## Report back

Files, RED/GREEN, REAL vs PARTIAL, risks.
