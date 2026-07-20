# SCRATCH — C4-F mcp-composition (build, GLM `glm-5.2`)

Branch: `goblin-implement-epic-tree`. Handoff:
`handoffs/HANDOFF-C4-F-mcp-composition.md`. Wave note:
`waves/c4-mcp-streamable-http.md` (C4-F section appended).
Evidence: `tests/c4/c4f_mcp_composition_{RED,GREEN,GREEN_gate}.log`.

## One-line status
REAL for composition wiring (bind + fail-closed bearer auth + `initialize` →
`tools/list` → `tools/call` over the real `ShellSessionActorRuntime` on
loopback). PARTIAL for TLS (HUMAN gate D-SEC.13 / MCP102-HUMAN, by contract).

## Files changed (owned)
- `crates/codegen/xai-grok-pager-bin/Cargo.toml` — optional
  `xai-grok-mcp-server` + `reqwest` deps; new `mcp-streamable-http` feature →
  `xai-grok-mcp-server/streamable-http` + `dep:reqwest`.
- `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` —
  `MCP_HTTP_SERVE_ENV` (`GROK_OSS_MCP_HTTP`), `mcp_http_serve_env_enabled()`,
  `experimental_mcp_http_runtime[_with_root]()` (real
  `ShellSessionActorRuntime` as `Arc<dyn GrokRuntimeFacade>`, NOT FakeRuntime),
  `mcp_http_server_config()` (always `require_auth: true`, fail-closed),
  `run_mcp_http[_with_root]()` + 5 composition tests
  (`mcp_http_composition_tests`, feature-gated).
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — env-gated Serve dispatch
  to `run_mcp_http` under `GROK_OSS_MCP_HTTP=1` + `mcp-streamable-http`;
  `print_mcp_http_startup_info` (honest TLS HUMAN gate); `McpHttpGuard`
  (RAII abort). Inserted before the C3-G WS block; distinct env so no collision.

## Not touched (ownership respected)
- `xai-grok-mcp-server/**` (C4-B/E owns the HTTP server).
- `xai-grok-shell/**` (C1 / C3-F shell runtime + projection).
- `xai-grok-app-server/**` (C3-B WS listener; C3-G WS composition intact).
- `xai-grok-multi-auth/**` (C5), `xai-grok-tower/**` (facade-only).

## Reproduce
```bash
# GREEN
cargo test -p xai-grok-pager-bin --features mcp-streamable-http mcp_http
bash scripts/run-rust-test-gate.sh mcp_http \
  cargo test -p xai-grok-pager-bin --features mcp-streamable-http mcp_http
# default (no feature) — MCP composition tests filtered out
cargo test -p xai-grok-pager-bin app_server_composition
# C3-G WS wiring still intact
cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws
# mcp-server self-loop guard still passes (composition source has no
# contiguous register_self / http://127.0.0.1:8788/mcp literal)
cargo test -p xai-grok-mcp-server --features streamable-http \
  composition_source_does_not_register
# RED (stub require_auth=false in mcp_http_server_config, then revert)
cargo test -p xai-grok-pager-bin --features mcp-streamable-http mcp_http
```

## Product path (documented CLI/env)
```
GROK_OSS_MCP_HTTP=1 grok agent serve --bind 127.0.0.1:0 --secret <token>
```
Default bind loopback; cleartext non-loopback `experimental/unsafe`; TLS HUMAN
(D-SEC.13 / MCP102-HUMAN). Fail-closed bearer: an empty
`--secret`/`GROK_AGENT_SECRET` makes `run_mcp_http_server` refuse to bind
(F-2). Without the env var, `agent serve` runs the shell agent server
unchanged. Distinct from `GROK_OSS_APP_SERVER` (C3-G WS) — the two gates are
independent; if both are set, the MCP HTTP block (checked first) wins and the
WS block is skipped, but each alone triggers only its own path.

## Self-loop guard (three layers, preserved)
1. Symbol: `xai-grok-mcp-server` `http_server_does_not_import_outbound_mcp_client`
   + `no_local_self_injection_in_production_source` / `no_self_mcp_loop_tool_names`.
2. Composition: `composition_source_does_not_register_local_mcp_self_loop`
   (mcp-server integration suite) scans `app_server_composition.rs` for the
   contiguous literals `register_self` and `http://127.0.0.1:8788/mcp`. The
   local `mcp_http_composition_does_not_self_register_local_mcp` guard
   reconstructs those forbidden tokens from parts (`format!("{}{}",
   "register_", "self")`) so the guard does not itself introduce the
   contiguous literal it scans for — verified by the mcp-server suite passing.
3. Runtime: `post_tools_call_does_not_reenter_via_managed_mcp_client`
   (C4-B) asserts exactly one transport session (no re-entry).

## Pre-existing failure (not C4-F)
`tests::is_managed_install_matches_only_the_bin_grok_target` fails because the
test hardcodes `home/bin/grok` but `PRODUCT_BIN_NAME` is now `"grok-oss"`
(grok-oss identity cutover in progress, per `AGENTS.md`). Exists in the staged
tree pre-C4-F; C4-F does not touch `is_managed_install`. Documented in C3-G
§6 R-C3G-6.

## Residual
- TLS termination → HUMAN gate D-SEC.13 / MCP102-HUMAN.
- SSE live push + disconnect-cancels-turn → C4-B residuals (facade has no
  push seam / per-turn handle exposed to the HTTP layer).
- CLI `--mcp off|stdio|http://ADDR` matrix (vs env gate) → composition/CLI
  follow-on; the env gate is the documented CLI/env path per handoff
  acceptance.
- Production `spawn_session_on_thread` assembly → C1-J/C2-A BLOCKER
  (credentials + ~80 args); the composition uses the real
  `ShellSessionActorRuntime` facade, whose turn methods surface the BLOCKER
  honestly via `no_resident_error`.
