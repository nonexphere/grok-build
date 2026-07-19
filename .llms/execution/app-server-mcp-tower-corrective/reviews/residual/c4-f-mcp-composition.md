# Residual review — C4-F MCP Streamable HTTP composition wiring

| Field | Value |
|---|---|
| Wave | C4-F (product composition wiring of `run_mcp_http_server`) |
| Mode | implementation review (residual) |
| Reviewer | review harness (read-only, glm-5.2) |
| Date | 2026-07-19 |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

`run_mcp_http_server` is wired into the `xai-grok-pager-bin` product
composition root over the **real** `ShellSessionActorRuntime` (as
`Arc<dyn GrokRuntimeFacade>`, NOT `FakeRuntime`). Distinct env
(`GROK_OSS_MCP_HTTP`) + distinct feature (`mcp-streamable-http`); fail-closed
bearer; three-layer self-loop guard preserved; C3-G WS wiring intact. TLS
honestly PARTIAL (HUMAN gate D-SEC.13 / MCP102-HUMAN). Findings Medium/Low.

## Severity summary

- Critical: 0
- High: 0
- Medium: 2 (F-1, F-2)
- Low: 3 (F-3, F-4, F-5)

## Contract non-negotiables (re-checked)

- **No Fake hybrid on product path.** `experimental_mcp_http_runtime` returns
  a real `Arc<dyn GrokRuntimeFacade>` from `ShellSessionActorRuntime::new(root)`,
  not `FakeRuntime`. The MCP listener takes the facade directly (not a
  `FacadeProcessor`), routing `tools/call` through the shared
  `invoke_tower_tool`. PASS.
- **No local MCP self-loop (three layers).**
  1. Symbol: `http_server_does_not_import_outbound_mcp_client` at
     `xai-grok-mcp-server/src/transport/http_server.rs:751`.
  2. Composition: `composition_source_does_not_register_local_mcp_self_loop`
     at `xai-grok-mcp-server/tests/streamable_http.rs:756` scans
     `app_server_composition.rs` for contiguous `register_self` /
     `http://127.0.0.1:8788/mcp` literals; the local
     `mcp_http_composition_does_not_self_register_local_mcp` guard
     reconstructs those tokens from parts so it does not self-trip.
  3. Runtime: `post_tools_call_does_not_reenter_via_managed_mcp_client`
     (C4-B) asserts exactly one transport session.
  All pass. PASS.
- **Fail-closed bearer.** `mcp_http_server_config` always sets
  `require_auth: true`; the listener (C4-E F-2) refuses to bind on
  empty/whitespace bearer. RED log proves 3/5 composition tests fail when
  `require_auth` stubbed false. PASS.
- **No second actor / Tower ≠ Shell.** No shell, tower, mcp-server, or
  app-server source edits; only pager-bin composition + main.rs. PASS.
- **TLS is a HUMAN gate.** `print_mcp_http_startup_info` prints
  `"TLS: not provided (HUMAN gate D-SEC.13 / MCP102-HUMAN — cleartext only)"`;
  never auto-promotes a cleartext remote bind. PASS.
- **Secrets.** Bearer is `--secret`/`GROK_AGENT_SECRET` (auto-generated when
  not supplied, non-empty); responses redact via tower projection;
  `SECRET_CANARIES` not weakened. PASS.

## Evidence reviewed

- Wave note: `.llms/execution/app-server-mcp-tower-corrective/waves/c4-mcp-streamable-http.md` (C4-F section appended).
- Handoff: `.llms/.../handoffs/HANDOFF-C4-F-mcp-composition.md`
- SCRATCH: `.llms/.../SCRATCH/waves/c4-f.md`
- GREEN gate: `.llms/.../tests/c4/c4f_mcp_composition_GREEN_gate.log`
  (5/5 `mcp_http_composition_tests` pass across all three bin targets; gate
  exit 0).
- RED: `tests/c4/c4f_mcp_composition_RED.log` (3/5 fail when `require_auth`
  stubbed false).
- Source guards: `http_server.rs:751`, `streamable_http.rs:756`.

## Findings

### F-1 — SSE live push + disconnect-cancels-turn deferred (Medium, high confidence)
The SSE event log is pull-fed from `GrokRuntimeFacade::replay` after each
mutating `tools/call`; a long-lived GET stream stays open via axum
`KeepAlive` but only delivers events that exist at GET time — it does not
block-wait for future events. Disconnect cleanup is via axum task drop; a
turn in flight is not actively interrupted via `tower_agent_interrupt` (the
facade has no per-turn handle exposed to the HTTP layer). These are C4-B
residuals, not C4-F defects, but they cap the composition's liveness
guarantees.

### F-2 — Production spawn BLOCKER surfaces on real turns (Medium, high confidence)
The composition uses the real `ShellSessionActorRuntime` facade, whose turn
methods surface the C1-J/C2-A BLOCKER via `no_resident_error` (`unsupported`).
The composition test reaches `initialize` → `tools/list` → `tools/call`
dispatch but a real resident turn would trip the BLOCKER. Cross-wave C1-J
dependency, not a C4-F defect.

### F-3 — CLI `--mcp off|stdio|http://ADDR` matrix not implemented (Low, high confidence)
The env gate (`GROK_OSS_MCP_HTTP=1`) is the documented CLI/env path per
handoff acceptance; the full CLI matrix is a composition/CLI follow-on.
Acceptable.

### F-4 — Pre-existing `is_managed_install_*` failure (Low, high confidence)
Same grok-oss identity-cutover failure as C3-G; pre-existing, not C4-F.
Verified not touched.

### F-5 — If both `GROK_OSS_MCP_HTTP` and `GROK_OSS_APP_SERVER` set, MCP wins (Low, medium confidence)
The MCP HTTP block is checked first; if both envs are set, the WS block is
skipped. Documented as R-C4F-1. Acceptable (independent gates), but an
operator setting both gets only MCP HTTP with no warning. Minor operational
surprise.

## Required fixes

None for this wave's bounded scope.

## Residual risk / dependencies

- SSE live push + disconnect-cancels-turn (C4-B residuals — facade has no
  push seam / per-turn handle).
- TLS termination (HUMAN gate D-SEC.13 / MCP102-HUMAN).
- CLI `--mcp` matrix (composition/CLI follow-on).
- Production `spawn_session_on_thread` assembly (C1-J/C2-A BLOCKER).

## Commands / results

- `cargo test -p xai-grok-pager-bin --features mcp-streamable-http mcp_http` → 5/5 pass (GREEN gate log).
- `bash scripts/run-rust-test-gate.sh mcp_http cargo test -p xai-grok-pager-bin --features mcp-streamable-http mcp_http` → exit 0.
- `cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws` → 3/3 (C3-G WS intact).
- `cargo test -p xai-grok-mcp-server --features streamable-http composition_source_does_not_register` → self-loop guard passes.
- `cargo check -p xai-grok-pager-bin --features mcp-streamable-http,app-server-ws` → OK.
