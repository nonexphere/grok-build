# Handoff C3-C — Independent code review of C3-B WS listener (GLM review)

| Field | Value |
|---|---|
| Agent role | **review** |
| Model | `glm-5.2` |
| Capability | read-only |
| Start after | C3-B stable |

## Goal

Independent code review of real WebSocket listener. Do not implement.

## Scope

- `crates/codegen/xai-grok-app-server/src/transport/ws_listener.rs`
- `websocket.rs`, `mod.rs`, `lib.rs`, `Cargo.toml` feature gates
- wave `c3-ws-listener.md`, evidence `tests/c3/*`
- map `c3-ws-surface-map.md` acceptance

## Checks

1. Real bind/accept/upgrade (not helper-only)?
2. Reuses FacadeProcessor only?
3. Auth constant-time; no token in URL?
4. Cleartext non-loopback experimental/unsafe preserved?
5. Bounded writer real?
6. Tower≠Shell?
7. Security footguns?

## Deliverable

`.llms/execution/app-server-mcp-tower-corrective/reviews/c3/code-review.md`
Verdict PASS | PASS_WITH_FINDINGS | FAIL.
