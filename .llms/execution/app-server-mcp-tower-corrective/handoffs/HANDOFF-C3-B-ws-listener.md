# Handoff C3-B — Real WebSocket listener (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Wave | C3 items 20–21 (listener slice) |
| Capability | read-write under owned paths |
| Depends on | C3-A map; C1-G landed (non-file-dep for listener itself) |
| Branch | `goblin-implement-epic-tree` |

## Goal

Implement a **real** WebSocket TCP listener/lifecycle over the shared `FacadeProcessor`, with black-box RED→GREEN tests for handshake, auth header, text frames, ping/pong, binary/batch/oversize rejection, disconnect, and a bounded writer. Helper-only validation is not enough.

## Read first

- `.llms/execution/app-server-mcp-tower-corrective/waves/c3-ws-surface-map.md` (full)
- Corrective contract § Wave C3 items 20–21
- `crates/codegen/xai-grok-app-server/src/transport/websocket.rs`
- `crates/codegen/xai-grok-app-server/src/{processor,security,lib}.rs`

## Non-negotiables

- Reuse `FacadeProcessor::handle_line` — no second processor
- Cleartext non-loopback remains `experimental/unsafe`; do **not** claim production TLS (HUMAN)
- Do not edit `xai-grok-shell/**` (C1 ownership) or multi-auth (C5) or MCP server
- RED→GREEN evidence under `tests/c3/`
- Tower ≠ Shell dependency

## Owned files

- `crates/codegen/xai-grok-app-server/src/transport/**` (incl. new `ws_listener.rs` if needed)
- `crates/codegen/xai-grok-app-server/Cargo.toml` (feature-gated tokio-tungstenite etc.)
- Tests under app-server for WS black-box
- Ledger: `waves/c3-ws-listener.md`, `tests/c3/*`, STATUS/CHANGES

## Acceptance

1. Real bind/listen/accept/upgrade path (feature-gated `websocket` OK).
2. Black-box tests cover: auth fail, valid text RPC through processor, oversized/batch rejection, ping/pong or documented WS keepalive, disconnect cleanup.
3. Bounded writer or documented backpressure behavior with test.
4. Slow-client **real-adapter** resync may stay PARTIAL (defer to C3 history) — document.
5. Wave note + evidence.

## Report back

Files, RED/GREEN, REAL vs PARTIAL, risks.
