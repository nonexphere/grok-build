# Resolution of residual reviews C3-G / C4-F / C7-B

Date: 2026-07-19  
Source reviews: `c3-g-ws-composition.md`, `c4-f-mcp-composition.md`, `c7-b-conformance.md`

## C3-G WS composition

| Finding | Resolution |
|---|---|
| F-1 real-adapter slow-client resync | **DONE** — `app_server_ws_real_adapter_slow_client_resync_via_subscribe` over real `ShellSessionActorRuntime` + WS (pager-bin, feature `app-server-ws`) |
| F-2 production spawn BLOCKER | **DOCUMENTED residual HUMAN** — still requires credentials + `RealSpawnFn` factory; not silently faked |
| F-3 is_managed_install | **DONE earlier (C7-C)** — test uses `PRODUCT_BIN_NAME` / `grok-oss` |
| F-4 env gate without serial | **DONE** — `#[serial_test::serial]` on WS env gate test |

## C4-F MCP composition

| Finding | Resolution |
|---|---|
| F-1 SSE live push + disconnect-cancels-turn | **DONE** — long-lived SSE via mpsc + `Notify`; `DELETE` and SSE consumer drop call `interrupt_active_turn` |
| F-2 production spawn BLOCKER | **DOCUMENTED residual HUMAN** (same as C3-G F-2) |
| F-3 CLI `--mcp` matrix | **DONE** — `GROK_OSS_MCP=off\|stdio\|http` + legacy `GROK_OSS_MCP_HTTP=1` alias; tests cover matrix |
| F-4 is_managed_install | **DONE** (C7-C) |
| F-5 both envs set, MCP wins silently | **DONE** — `eprintln!` warning when both `GROK_OSS_MCP_HTTP` and `GROK_OSS_APP_SERVER` enabled |

## C7-B conformance

| Finding | Resolution |
|---|---|
| F-1 archive divergence | **DONE** — real `archive_session` hide via `archived.flag`; both adapters return `Archived`; no delete |
| F-2 ordinal + status | **DONE** — real ordinal seed `num_messages+1` / `fetch_add`; Fake returns `Completed` after synthetic turn |
| F-3 steer body + replay TurnChanged | **DONE** steer `UserMessage` item; TurnChanged still PARTIAL (Shell writes none) — documented remaining |

## Validation (sample)

```text
cargo test -p xai-grok-shell --test c7_conformance --test c1_shell_port → green
cargo test -p xai-grok-mcp-server --features streamable-http → 27 integration green
cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws → 4/4
cargo test -p xai-grok-pager-bin --features mcp-streamable-http mcp_http → 5/5
```

## Remaining (honest, not claimed fixed)

- Production `spawn_session_on_thread` + HUMAN credentials (C3-G F-2 / C4-F F-2).
- Shell does not write `TurnChanged` into `updates.jsonl` (C7-B F-3 partial for replay lifecycle).
- TLS HUMAN gate unchanged.
