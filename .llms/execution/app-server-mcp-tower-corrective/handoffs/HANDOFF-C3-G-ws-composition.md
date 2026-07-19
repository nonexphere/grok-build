# Handoff C3-G — Wire WS listener into product composition (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Branch | `goblin-implement-epic-tree` |

## Goal

Wire `run_ws_listener` (or equivalent) into `xai-grok-pager-bin` experimental Serve path so product can start a real WebSocket app-server over `ShellSessionActorRuntime` / FacadeProcessor. Feature-gate if needed. Default bind loopback; cleartext non-loopback experimental/unsafe.

## Read first

- `waves/c3-ws-listener.md`, `c3-ws-surface-map.md`
- `app_server_composition.rs`
- `ws_listener.rs`
- pager-bin Serve command surface

## Owned

- `xai-grok-pager-bin/**` composition/CLI for Serve WS only
- Feature flags to enable `websocket` on app-server dep
- tests + `waves/c3-ws-composition.md` + `tests/c3/*`

## Must NOT

- Rewrite shell runtime (C3-F may touch projection concurrently — avoid shell)
- MCP HTTP product wiring (separate C4-F)
- multi-auth

## Acceptance

1. Documented CLI/env path starts real listener on 127.0.0.1
2. At least one black-box or composition test proves bind + auth + handle_line path
3. Honest PARTIAL for TLS HUMAN

## Report

Files, RED/GREEN, residual.
