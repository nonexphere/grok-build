# Residual review — C3-G WS listener composition wiring

| Field | Value |
|---|---|
| Wave | C3-G (C3 item 22 — composition wiring of `run_ws_listener`) |
| Mode | implementation review (residual) |
| Reviewer | review harness (read-only, glm-5.2) |
| Date | 2026-07-19 |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

The WS listener is wired into the `xai-grok-pager-bin` product composition
root over the **real** `ShellSessionActorRuntime` (not `FakeRuntime`).
Feature-gated + env-gated so the default build and default `agent serve`
behavior are unchanged. Bind + auth + `handle_line` roundtrip proven over a
real `tokio-tungstenite` client. TLS is honestly PARTIAL (HUMAN gate
D-SEC.13). Findings are Medium/Low.

## Severity summary

- Critical: 0
- High: 0
- Medium: 2 (F-1, F-2)
- Low: 2 (F-3, F-4)

## Contract non-negotiables (re-checked)

- **No Fake hybrid on product path.** `run_app_server_ws` builds the real
  `experimental_app_server_processor()` (→ `ShellSessionActorRuntime::new`);
  the composition test uses `run_app_server_ws_with_root(TempDir)` so it
  never touches `grok_home()`. PASS.
- **No second actor / Tower ≠ Shell.** No shell runtime, tower, or mcp-
  server edits; only pager-bin composition + main.rs dispatch. PASS.
- **Auth fail-closed on product path.** `app_server_ws_listener_config` sets
  `require_auth: true`; `app_server_ws_config_requires_auth_by_default`
  guards it. RED log proves the composition test catches missing auth
  (wrong bearer upgrade succeeds when `require_auth` stubbed false). PASS.
- **Loopback default; cleartext non-loopback experimental/unsafe.** Bind
  defaults to `127.0.0.1`; composition test asserts `addr.ip() == 127.0.0.1`.
  Non-loopback cleartext emits `remote_bind_warning_exact`. PASS.
- **TLS is a HUMAN gate.** `print_app_server_ws_startup_info` prints
  `"TLS: not provided (HUMAN gate D-SEC.13 — cleartext only)"`; never
  auto-promotes a cleartext remote bind. PASS.
- **Secrets.** Bearer is the per-invocation `--secret`/`GROK_AGENT_SECRET`;
  responses route through the shared `FacadeProcessor` with tower projection
  redaction; `SECRET_CANARIES`/`assert_no_secret_canaries` not weakened. PASS.

## Evidence reviewed

- Wave note: `.llms/execution/app-server-mcp-tower-corrective/waves/c3-ws-composition.md`
- Handoff: `.llms/.../handoffs/HANDOFF-C3-G-ws-composition.md`
- SCRATCH: `.llms/.../SCRATCH/waves/c3-g.md`
- GREEN gate: `.llms/.../tests/c3/c3_ws_composition_GREEN_gate.log`
  (3/3 `app_server_ws_composition_tests` pass across all three bin targets;
  gate exit 0).
- RED: `tests/c3/c3_ws_composition_RED.log` (wrong-bearer assertion fails
  when `require_auth` stubbed false).

## Findings

### F-1 — Real-adapter slow-client resync over WS deferred (Medium, high confidence)
The composition test exercises `initialize` + `session/start` over the real
adapter; slow-client resync is deferred to C3-22/23 (canonical session
files). Acceptable per C3-B, but the WS path is not yet proven under
slow-client/reconnect conditions.

### F-2 — Production spawn BLOCKER surfaces here too (Medium, high confidence)
The composition uses the real `ShellSessionActorRuntime` facade, whose turn
methods surface the C1-J/C2-A BLOCKER honestly via `no_resident_error`
(`unsupported`). The WS composition test only reaches `session/start`, not a
real turn, so it does not trip the BLOCKER — but a real `tools/call`/turn
over WS would. This is the cross-wave C1-J dependency, not a C3-G defect.

### F-3 — Pre-existing `is_managed_install_*` failure (Low, high confidence)
`tests::is_managed_install_matches_only_the_bin_grok_target` fails because
the test hardcodes `home/bin/grok` but `PRODUCT_BIN_NAME` is now `"grok-oss"`
(grok-oss identity cutover). Pre-existing, not C3-G; tracked by the identity
cutover workstream. Verified not touched by C3-G.

### F-4 — Env-gate test mutates `GROK_OSS_APP_SERVER` without `#[serial]` (Low, medium confidence)
`app_server_serve_env_gate_default_is_off` saves/restores the prior value
and is the only test in the module reading that var, so no `#[serial]` is
used. Acceptable within the module, but fragile if another test in the same
binary later reads `GROK_OSS_APP_SERVER`. Documented as R-C3G-7.

## Required fixes

None for this wave's bounded scope.

## Residual risk / dependencies

- Real-adapter slow-client resync over WS (C3-22/23).
- TLS termination (HUMAN gate D-SEC.13 — never auto-resolved).
- Production `spawn_session_on_thread` assembly (C1-J/C2-A BLOCKER) for real
  turns over WS.
- A dedicated CLI subcommand for the app-server is a follow-on; the env gate
  is the documented CLI/env path per handoff acceptance.

## Commands / results

- `cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws` → 3/3 pass (GREEN gate log).
- `bash scripts/run-rust-test-gate.sh app_server_ws cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws` → exit 0.
- `cargo test -p xai-grok-pager-bin` (default, no feature) → 39 passed; 1 pre-existing failure (unrelated); WS composition tests filtered out.
- `cargo test -p xai-grok-app-server --features websocket` → 42 passed; 0 failed (C3-B listener unaffected).
- `cargo check -p xai-grok-pager-bin --features app-server-ws` → OK.
