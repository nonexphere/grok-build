# C3 — WebSocket listener test evidence

| File | What |
|---|---|
| `c3_ws_listener_RED.log` | RED: handshake auth stubbed (`if require_auth && false`); `ws_listener_rejects_missing_authorization_header` + `ws_listener_rejects_wrong_bearer` FAIL (upgrade succeeds instead of rejecting). Proves the black-box tests catch missing auth. |
| `c3_ws_listener_GREEN.log` | GREEN: real handshake; all 16 `ws_listener`/`ws_listener_blackbox` tests pass (16 passed; 0 failed in ~5s). |
| `c3_ws_listener_GREEN_gate.log` | `scripts/run-rust-test-gate.sh ws_listener cargo test -p xai-grok-app-server --features websocket` → exit 0 (42 passed total; gate fragment `ws_listener` matched). |
| `c3_ws_composition_RED.log` | RED (C3-G): `require_auth` stubbed `false` in `app_server_ws_listener_config`; `app_server_ws_composition_bind_auth_and_handle_line_roundtrip` FAILS at "wrong bearer must be rejected at the handshake". Proves the composition test catches missing auth on the product path. |
| `c3_ws_composition_GREEN.log` | GREEN (C3-G): real `require_auth: true`; all 3 `app_server_ws_composition_tests` pass (3 passed; 0 failed). |
| `c3_ws_composition_GREEN_gate.log` | `scripts/run-rust-test-gate.sh app_server_ws cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws` → exit 0 (gate fragment `app_server_ws` matched). |

Reproduce:

```bash
cargo test -p xai-grok-app-server --features websocket ws_listener
bash scripts/run-rust-test-gate.sh ws_listener cargo test -p xai-grok-app-server --features websocket

# C3-G composition (pager-bin product path)
cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws
bash scripts/run-rust-test-gate.sh app_server_ws \
  cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws
```

Scope: items 20/21/24 (C3-B listener) + item 22 (C3-G composition wiring).
PARTIAL: real-adapter slow-client resync deferred to C3-22/23 (canonical
session files); fake-adapter variant `ws_listener_slow_client_resync_via_replay_fake_adapter`
landed in C3-B. TLS is a HUMAN gate (D-SEC.13) — never auto-resolved.

## C3-F — history / RuntimeEvent projection (R2/R11)

| File | What |
|---|---|
| `c3_history_projection_RED.log` | RED: source reverted to C1-J (empty turns/items, minimal projection). 9/16 FAIL (turn/item/tool-call-lifecycle/thought/plan projections missing). 7 PASS = honest-PARTIAL absence tests (no TurnChanged, no InteractionRequested, skips xAI, cursor beyond end, snapshot event 0, empty updates, agent delta). |
| `c3_history_projection_GREEN.log` | GREEN: 16/16 pass with the shared `project_updates` projector. |
| `c3_history_projection_GREEN_gate.log` | `scripts/run-rust-test-gate.sh c3_read_session cargo test -p xai-grok-shell --test c3_history_projection` → exit 0 (16 passed; gate fragment `c3_read_session` matched). |

Reproduce:

```bash
cargo test -p xai-grok-shell --test c3_history_projection
bash scripts/run-rust-test-gate.sh c3_read_session \
  cargo test -p xai-grok-shell --test c3_history_projection
```

Scope: C3 items 22–23 residual (R2/R11). REAL: user/agent/thought chunks,
tool-call lifecycle correlated via `tool_call_id`, plan. PARTIAL: `TurnChanged`
not emitted (Shell writes no turn lifecycle); turn status inferred Completed
from persistence; item grouping not performed; `InteractionRequested` not
projected (in-memory only); xAI extension updates skipped.
