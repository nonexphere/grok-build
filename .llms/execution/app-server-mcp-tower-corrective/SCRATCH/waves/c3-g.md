# SCRATCH — C3-G ws-composition (build, GLM `glm-5.2`)

Branch: `goblin-implement-epic-tree`. Wave note: `waves/c3-ws-composition.md`.
Evidence: `tests/c3/c3_ws_composition_{RED,GREEN,GREEN_gate}.log`.

## One-line status
REAL for composition wiring (bind + auth + `handle_line` over the real
`ShellSessionActorRuntime`). PARTIAL for TLS (HUMAN gate D-SEC.13, by contract).

## Files changed (owned)
- `crates/codegen/xai-grok-pager-bin/Cargo.toml` — `app-server-ws` feature
  → `xai-grok-app-server/websocket` + optional `tokio-tungstenite`/`futures-util`.
- `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` —
  `APP_SERVER_SERVE_ENV`, `app_server_serve_env_enabled()`,
  `app_server_ws_listener_config()`, `run_app_server_ws()`,
  `run_app_server_ws_with_root()` + 3 composition tests.
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — env-gated Serve dispatch
  to `run_app_server_ws`; `print_app_server_ws_startup_info`;
  `AppServerWsGuard` (RAII abort).

## Not touched (ownership respected)
- `xai-grok-shell/**` (C1 / C3-F shell runtime + projection).
- `xai-grok-app-server/src/transport/ws_listener.rs` (C3-B listener).
- `xai-grok-mcp-server/**` (C4 MCP HTTP — explicitly not wired here).
- `xai-grok-multi-auth/**` (C5), `xai-grok-tower/**` (facade-only).

## Reproduce
```bash
# GREEN
cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws
bash scripts/run-rust-test-gate.sh app_server_ws \
  cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws
# default (no feature) — WS composition tests filtered out
cargo test -p xai-grok-pager-bin app_server_composition
# RED (stub require_auth=false in app_server_ws_listener_config, then revert)
cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws_composition_bind_auth
```

## Product path (documented CLI/env)
```
GROK_OSS_APP_SERVER=1 grok agent serve --bind 127.0.0.1:0 --secret <token>
```
Default bind loopback; cleartext non-loopback `experimental/unsafe`; TLS HUMAN
(D-SEC.13). Without the env var, `agent serve` runs the shell agent server
unchanged.

## Pre-existing failure (not C3-G)
`tests::is_managed_install_matches_only_the_bin_grok_target` fails because the
test hardcodes `home/bin/grok` but `PRODUCT_BIN_NAME` is now `"grok-oss"`
(grok-oss identity cutover in progress, per `AGENTS.md`). Exists in the staged
tree pre-C3-G; C3-G does not touch `is_managed_install`.

## Residual
- MCP HTTP product wiring → C4-F.
- Real-adapter slow-client resync over WS → C3-22/23.
- TLS termination → HUMAN gate D-SEC.13.
