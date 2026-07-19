# Handoff C4-F — Wire MCP Streamable HTTP into product composition (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Branch | `goblin-implement-epic-tree` |
| Parallel | C3-G only if carefully split CLI surfaces; prefer after C3-G if same files |

## Goal

Wire `run_mcp_http_server` into pager-bin experimental path (`--mcp` or env) with fail-closed bearer (never empty when require_auth). Shared facade from composition. No self-MCP loop.

## Read first

- `waves/c4-mcp-streamable-http.md`
- mcp-server `http_server.rs`
- composition + main.rs command enum

## Owned

- pager-bin CLI/composition for MCP HTTP only (coordinate if C3-G concurrent — use non-overlapping command paths)
- feature streamable-http on dep
- ledger tests/c4 composition evidence

## Must NOT

- shell app_server_runtime
- multi-auth core (unless CLI login only)

## Acceptance

1. Product can start loopback MCP HTTP with required bearer
2. Self-loop guard still holds
3. Test or documented smoke path
4. PARTIAL TLS HUMAN

## Report

Files, RED/GREEN, residual.
