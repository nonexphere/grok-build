# C6-C interaction conformance — SCRATCH notes

## What was proven

`interaction/respond` produces EQUAL accept and error shapes across all three
App Server transports:

| Transport | Surface | Accept | not-init error | invalid-params error |
|---|---|---|---|---|
| in-process | `InProcessClient::request` | `{"operationId":"interaction","accepted":true}` | -32002 / `not_initialized` | -32602 / `invalid_params` |
| stdio | `process_ndjson_batch` | same | same | same |
| WS frame adapter | `handle_ws_text` | same | same | same |
| real WS listener (feature `websocket`) | `tokio-tungstenite` client over real `TcpListener` | same | same | same |

## Why parity holds

All transports route through the same `FacadeProcessor::handle_line` →
`dispatch("interaction/respond", params)` → `runtime.respond_interaction(p)`.
The processor wraps the result as `{"operationId":"interaction","accepted":true}`
and wraps errors as `{"error":{"code":N,"message":..,"data":{"code":domain,"retryable":..}}}`.
Transports only differ in framing (typed call / NDJSON line / WS text frame /
TCP+WS handshake), not in dispatch. The tests prove this by running the same
request through each surface and comparing the normalized `result`/`error`
fields.

## Files changed

- `crates/codegen/xai-grok-app-server/src/lib.rs` — added
  `interaction_conformance_tests` (default features) and
  `interaction_conformance_ws_listener` (feature-gated `websocket`).

## Evidence logs (tests/c6)

- `c6_interaction_conformance_GATE.log` — gate run (default features).
- `c6_interaction_conformance_default_GREEN.log` — full default suite.
- `c6_interaction_conformance_ws_listener_GREEN.log` — `--features websocket` filtered.
- `c6_interaction_conformance_full_ws_GREEN.log` — full `--features websocket` suite.
- `c6_interaction_conformance_RED_to_GREEN.md` — RED→GREEN narrative.

## Out of scope

- Controller lease effects on `interaction/respond` (owned by AS106-01/02/03,
  already GREEN).
- `interaction/request` event emission (FakeRuntime does not emit
  `InteractionRequested` events; the request-side rendering is shared via
  `runtime_event_json` across all transports by construction — not separately
  re-tested here).
