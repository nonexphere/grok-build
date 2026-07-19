# C6-C interaction_conformance — RED→GREEN evidence

## Gate (default features: in-process + stdio + WS frame adapter)

Command:
```
./scripts/run-rust-test-gate.sh interaction_conformance cargo test -p xai-grok-app-server interaction_conformance
```
Result: **GREEN** (exit 0). 5 tests pass:
- `interaction_conformance_accept_shape_matches_across_transports`
- `interaction_conformance_request_envelope_shape_is_stable`
- `interaction_conformance_not_initialized_error_shape_matches`
- `interaction_conformance_invalid_params_error_shape_matches`
- `interaction_conformance_suite_covers_minimum_matrix`

Log: `c6_interaction_conformance_GATE.log` (and `c6_interaction_conformance_default_GREEN.log`).

## RED (pre-existing gap, F-11)

Before C6-C, the requirement matrix (wave c0-requirement-matrix row 118)
recorded AS106-06 as PARTIAL with finding F-11:

> F-11: WS leg is helper-level (`handle_ws_text`), not black-box. Wave C3-24/C6-40.

There was NO test proving `interaction/respond` produces EQUAL accept/error
shapes across in-process, stdio, AND a real WS listener. The WS frame adapter
(`handle_ws_text`) existed but the real-listener black-box path for
`interaction/respond` was unproven, and no cross-transport shape-parity
assertion existed. This is the RED state: the conformance contract was
unproven, so AS106-06 could not be marked GREEN.

## GREEN (this wave)

Added two inline test modules in
`crates/codegen/xai-grok-app-server/src/lib.rs`:

1. `interaction_conformance_tests` (default features) — proves accept and
   error shape parity across **in-process** (`InProcessClient`), **stdio**
   (`process_ndjson_batch`), and the **WS frame adapter** (`handle_ws_text`):
   - accept shape: `{"operationId":"interaction","accepted":true}` identical
     across all three transports.
   - not-initialized error: numeric `-32002`, domain `not_initialized`
     identical across all three transports.
   - invalid-params error (missing `sessionId`): numeric `-32602`, domain
     `invalid_params` identical across all three transports.
   - request envelope shape pinned (camelCase params keys).

2. `interaction_conformance_ws_listener` (feature-gated `websocket`) — proves
   the **real WS listener** black-box path (real `TcpListener` + WS upgrade +
   `tokio-tungstenite` client) produces shapes matching the stdio reference:
   - `interaction_conformance_ws_listener_accept_shape_matches_stdio`
   - `interaction_conformance_ws_listener_not_initialized_error_matches_stdio`
   - `interaction_conformance_ws_listener_invalid_params_error_matches_stdio`

This closes F-11: the WS leg is now black-box, not helper-level.

## Real WS listener (feature websocket)

Command:
```
cargo test -p xai-grok-app-server --features websocket interaction_conformance
```
Result: **GREEN** (8 tests pass: 5 default + 3 real WS listener).
Log: `c6_interaction_conformance_ws_listener_GREEN.log`.

## Regression

- `cargo test -p xai-grok-app-server` (default) → 39 passed; 0 failed.
  Log: `c6_interaction_conformance_default_GREEN.log`.
- `cargo test -p xai-grok-app-server --features websocket` → 58 passed; 0 failed.
  Log: `c6_interaction_conformance_full_ws_GREEN.log`.

## Scope note

The AS106-06 task text also mentions "lease effects." The controller lease
(UNOWNED/HELD/RELEASED/RESOLVED) is owned by AS106-01/02/03 (already GREEN)
and lives in `controller.rs`; `interaction/respond` at the processor/facade
layer does not mutate the lease directly (the processor dispatch returns
`{"operationId":"interaction","accepted":true}` after the runtime facade
acknowledges delivery). This handoff's bound is shape parity across
transports, which is proven. Lease-effect conformance at the controller is
covered by the controller lease tests and is out of scope for C6-C.
