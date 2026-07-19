# Handoff C4-A — MCP Streamable HTTP surface map (GLM explore, read-only)

| Field | Value |
|---|---|
| Agent role | **repo-explore** (read-only) |
| Model | `glm-5.2` |
| Wave | C4 prep (items 26–29 inputs) |
| Capability | **read-only** — no product code edits |
| Parallel with | C1-G, C3-A, C5-A |
| Branch | `goblin-implement-epic-tree` |

## Goal

Map the real vs stub MCP Streamable HTTP path and the nine-tool semantic core so C4 implementer can write black-box RED tests for POST/GET/DELETE `/mcp`, session lifecycle, SSE resume, and parity with in-process tools.

## Read first

- Corrective contract § Wave C4
- `crates/codegen/xai-grok-mcp/src/{servers,wire,acp_transport,mcp_http_client,liveness,lib}.rs`
- Tower tools crate (`xai-grok-tower-tools` or equivalent) for tool descriptors
- Any existing `/mcp` router or axum setup
- `.llms/grok-build/` MCP epic tasks (MCP101-03 etc.)

## Deliverable

`.llms/execution/app-server-mcp-tower-corrective/waves/c4-mcp-surface-map.md`

Must include:

1. **Current state** of Streamable HTTP (helper only vs real server).
2. **Tool catalog**: exact nine tools — names, schema locations (`file:fn`).
3. **Parity path**: how in-process vs MCP adapters share semantic core (or don't today).
4. **Self-loop risk**: production composition must not double-execute tools via local MCP.
5. **Missing black-box behaviors**: POST/GET/DELETE `/mcp`, SSE resume, auth failure equivalence, body limits, cancellation, disconnect.
6. **Suggested RED tests** + owning crate.
7. **Files for C4-B implementer** (must not overlap Shell C1-G ownership).

## Must NOT

- Edit product code
- Claim MCP PASS
- Introduce credentials

## Report back

Path to map + executive summary + GO/NO-GO for C4-B after C1/C3 sequencing.
