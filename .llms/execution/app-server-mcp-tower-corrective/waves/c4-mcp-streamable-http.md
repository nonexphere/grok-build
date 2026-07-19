# C4-B — Real MCP Streamable HTTP server (GLM build)

| Field | Value |
|---|---|
| Agent | `build` (glm-5.2) |
| Wave | C4 items 26–28 (framing + real server) |
| Branch | `goblin-implement-epic-tree` |
| Verdict | **GREEN** — real axum `/mcp` server, 23 black-box tests + 12 lib tests; composition wiring PARTIAL |

## What landed (REAL)

A real Streamable HTTP server bound to a loopback TCP socket, serving
`POST/GET/DELETE /mcp` over axum 0.8. Every `tools/call` routes through the
shared semantic core `invoke_tower_tool` — no second tool implementation, no
local MCP self-loop.

- `xai-grok-mcp-server/src/transport/http_server.rs` — new, feature-gated
  behind `streamable-http`. Binds `TcpListener`, runs `axum::serve`, owns
  per-transport-session state (bearer fingerprint + Tower instance binding +
  per-session event log fed by `GrokRuntimeFacade::replay`).
- `xai-grok-mcp-server/Cargo.toml` — `streamable-http` feature now pulls
  `axum`, `tokio` (net/io-util/time/sync/rt), `futures-util` (all workspace
  deps, no new supply-chain). Added `xai-grok-app-server-protocol` as a real
  dep (for `SubscribeParams`/`WireCounter` in the SSE event feed).
- `xai-grok-mcp-server/src/lib.rs` — added `MCP_PROTOCOL_VERSION` constant
  and `call_tool_typed` (returns `ToolError` so the HTTP/stdio error shape can
  carry the stable Tower code). `handle_mcp_jsonrpc` `tools/call` error arm
  now emits `isError: true` + `structuredContent.{code,message}` (parity with
  in-process `ToolError`, closing the MCP102-05 error-shape gap).
- `xai-grok-mcp-server/src/transport/mod.rs` — re-exports `http_server`
  under the feature.
- `xai-grok-mcp-server/tests/streamable_http.rs` — new integration test
  directory (the crate previously had none). 23 black-box tests driving a
  real listener with a real `reqwest` client.

## RED → GREEN

RED: the crate had **no `/mcp` route, no listener, no axum/hyper dep, no
`tests/` dir** (per C4-A map §1). Every Streamable HTTP claim was helper-level.
GREEN: a real `run_mcp_http_server` binds a socket and serves the surface.

Black-box tests (real HTTP, real TCP bind, `reqwest` client, `FakeRuntime`
facade injected per handoff §"For tool black-box"):

| # | Test | Proves |
|---|---|---|
| 1 | `post_initialize_negotiates_session_header` | POST initialize returns `Mcp-Session-Id` |
| 2 | `post_tools_lists_exactly_nine_descriptors_matching_in_process` | nine-tool descriptor parity with `MCP_TOOL_NAMES` |
| 3 | `post_tools_call_start_returns_structured_content_with_session_id` | tools/call reaches `invoke_tower_tool`; `structuredContent.sessionId` present |
| 4 | `post_tools_call_deny_path_emits_iserror_with_forbidden_code` | `isError: true` + `code: forbidden` for build agent (fail-closed) |
| 5 | `auth_failures_are_indistinguishable_401` | missing/empty/wrong/malformed bearer all → 401 + `WWW-Authenticate: Bearer` |
| 6 | `body_limit_rejects_oversized_post_before_dispatch` | 2 MiB POST → 413 before dispatch |
| 7 | `delete_session_terminates_and_rejects_subsequent_post` | DELETE 200; subsequent POST → 404 |
| 8 | `delete_without_session_header_is_bad_request` | DELETE without session header → 400 |
| 9 | `get_sse_streams_events_after_tools_call` | GET /mcp `text/event-stream` delivers `session_changed` events with `id:` |
| 10 | `get_sse_resume_from_last_event_id` | `Last-Event-ID: 1` does not replay event 1 |
| 11 | `get_sse_foreign_last_event_id_returns_resumption_error` | `Last-Event-ID: 999` → `resumption_error` event |
| 12 | `get_sse_does_not_replay_another_clients_events` | transport session B never sees A's events |
| 13 | `get_sse_requires_accept_event_stream` | non-SSE Accept → 406 |
| 14 | `protocol_version_gate_rejects_unsupported_before_dispatch` | `protocol-version: 9999-99-99` → 400 `-32006` before dispatch |
| 15 | `session_id_from_tower_a_rejected_by_tower_b` | cross-Tower session id → 404 |
| 16 | `session_bearer_fingerprint_mismatch_rejects` | non-initialize without session header → 400 |
| 17 | `stdio_and_http_produce_identical_tools_list_and_error_shapes` | MCP102-05 parity (stdio vs HTTP) |
| 18 | `post_tools_call_does_not_reenter_via_managed_mcp_client` | exactly one transport session; no re-entry |
| 19 | `composition_source_does_not_register_local_mcp_self_loop` | pager-bin composition has no `http://127.0.0.1:8788/mcp` self-registration |
| 20 | `post_rejects_non_json_content_type` | non-JSON → 415 |
| 21 | `post_rejects_token_in_query_string` | `?token=secret` → 400 |
| 22 | `post_notification_returns_202_no_body` | notification (no `id`) → 202 |
| 23 | `healthz_returns_ok_without_auth` | `GET /healthz` → 200 no auth |

Plus the existing 11 lib/transport unit tests still pass (default features),
and a new `self_loop_canary::http_server_does_not_import_outbound_mcp_client`
asserts the HTTP module never imports `xai_grok_mcp::` / `McpClient` /
`register_self`.

## Acceptance mapping (handoff §Acceptance)

1. **Real HTTP bind serving `/mcp` (feature-gated OK).** ✅ REAL —
   `run_mcp_http_server` binds `TcpListener` + `axum::serve`, feature-gated
   behind `streamable-http`.
2. **Black-box: POST initialize/tools/list/tools/call; auth failure; body
   limit; DELETE session.** ✅ REAL — tests 1–8, 20–22.
3. **SSE GET resume path at least with helper/table wired to a real transport
   (full real-adapter resync may PARTIAL).** ✅ REAL for framing/resume/foreign-id
   (tests 9–13). PARTIAL for full real-adapter resync: the SSE event log is
   fed by polling `GrokRuntimeFacade::replay` after mutating `tools/call`,
   not by a live push subscription (the facade has no push seam today; this
   matches the map §2 note that the facade's event surface is pull-only via
   `replay`). A real Shell-backed adapter with live events would resync
   through the same `replay` cursor — no new contract needed.
4. **Nine-tool descriptor parity with in-process names.** ✅ REAL — test 2 +
   test 17 (stdio parity).
5. **Wave note + evidence; honest PARTIAL for composition self-loop if
   product bin not yet wired.** ✅ — this note + GREEN log. Composition
   self-loop is guarded (test 19 + canary) but the product bin does not yet
   bind the HTTP listener (PARTIAL, see below).

## PARTIAL / out-of-scope (honest)

- **Product bin wiring.** `xai-grok-pager-bin` does not yet bind
  `run_mcp_http_server` for `--mcp http://ADDR`. The composition root
  (`app_server_composition.rs`) still only builds the App Server processor.
  Wiring the MCP HTTP listener into the daemon co-start matrix
  (`--mcp off|stdio|http://ADDR`, default `http://127.0.0.1:8788`) is
  owned by the composition/CLI surface and was not in the C4-B file bound
  (map §7 lists it as "optional: coordinate with C3-B"). The self-loop guard
  test (19) and source canary ensure the composition does not self-register
  when wiring lands.
- **SSE live push.** The event log is pull-fed from `replay` after each
  mutating `tools/call`. A long-lived GET stream stays open via axum
  `KeepAlive` (15s) but only delivers events that exist at GET time; it does
  not block-wait for future events. A real push subscription requires a
  facade event-stream seam that does not exist today (map §2/§8 risk 2).
- **TLS.** Cleartext non-loopback bind emits the canonical
  `experimental/unsafe` warning; TLS termination remains a HUMAN gate
  (`D-SEC.13` / `MCP102-HUMAN`). Not claimed PASS.
- **Disconnect-cancels-turn.** HTTP disconnect cleanup is handled by axum
  task drop; a turn in flight is not actively interrupted via
  `tower_agent_interrupt` on disconnect (the facade has no per-turn handle
  exposed to the HTTP layer). Marked PARTIAL; the map §6.6 test is not
  included.
- **`stdio` conflict rule.** `--stdio` cannot coexist with MCP stdio
  (stdout framing owner). This is a CLI-matrix concern owned by the
  composition/CLI surface, not the server crate.

## Self-loop guard

Production composition must not register the local `/mcp` URL into the
session's MCP client pool. Enforced at three levels:

1. Symbol: `http_server_does_not_import_outbound_mcp_client` (this crate) +
   the pre-existing `no_local_self_injection_in_production_source` and
   `no_self_mcp_loop_tool_names` canaries.
2. Composition: `composition_source_does_not_register_local_mcp_self_loop`
   asserts pager-bin's `app_server_composition.rs` contains no
   `http://127.0.0.1:8788/mcp` literal and no `register_self` symbol.
3. Runtime: `post_tools_call_does_not_reenter_via_managed_mcp_client`
   asserts a `tower_agent_start` over HTTP produces exactly one transport
   session (no re-entry).

## Evidence

- `tests/c4/c4_streamable_http_GREEN.log` — 23 integration tests + 12 lib
  tests, all green.
- `cargo test -p xai-grok-mcp-server` (default features) — 11 tests green
  (no regression to stdio/helper tests).
- `cargo test -p xai-grok-tower-tools -p xai-grok-tower` — green (shared
  semantic core unaffected).

## Risks

- The SSE event log is per-transport-session and pull-fed; two transport
  sessions that bind to the same Tower session id (via the same idempotency
  key) each pull the same facade events into their own id space. Last-Event-ID
  is scoped per transport session, so a foreign id always yields a
  `resumption_error` (test 11) and another client's events are never replayed
  (test 12). This satisfies the spec's isolation requirement, but operators
  should be aware the event *data* can overlap across transport sessions
  bound to the same Tower session.
- The bearer fingerprint uses `DefaultHasher` (non-cryptographic). It is a
  binding fingerprint, not an auth check — auth is done by
  `validate_http_bearer` (constant-time-ish compare) against the expected
  token. The fingerprint only prevents a session negotiated with one bearer
  from being reused after the server is reconfigured for another.

## C4-E corrective — review fixes (F-2 + fingerprint test)

C4-E triages the C4-C/C4-D Medium findings against the same crate surface.
No new product behavior beyond the two fixes; the suite grows from 23 → 27
integration tests and 12 → 15 lib tests.

- **F-2 fail-closed auth (C4-C F-2).** `run_mcp_http_server` now refuses to
  bind when `require_auth: true` and `bearer_token` is empty/whitespace,
  returning `InvalidInput` with a `fail-closed` message. This closes the
  empty-bearer footgun where `McpHttpConfig::default()` (require_auth=true,
  empty token) would silently accept unauthenticated requests because two
  empty strings compare equal in `validate_http_bearer`. The default config
  can no longer bind; operators must provide a non-empty bearer or set
  `require_auth: false` explicitly. Documented on the `McpHttpConfig` struct
  and field doc-comments.
- **Fingerprint mismatch test (C4-D F-2).** Replaced the misleading
  `session_bearer_fingerprint_mismatch_rejects` (which only asserted
  missing-session-header → 400) with a real test that exercises the
  `lookup_session` fingerprint branch: a session opened under bearer A is
  injected into a server reconfigured for bearer B (same Tower instance id),
  and a request with bearer B (valid for B → auth passes) + A's session id
  is rejected with 401 because the fingerprint of B ≠ the stored fingerprint
  of A. Added `session_bearer_fingerprint_match_accepts` as the positive
  control. The missing-session-header path is now covered by a renamed
  `non_initialize_request_without_session_header_is_bad_request`.

### C4-E RED → GREEN

- RED (`tests/c4/c4e_fail_closed_auth_RED.log`): with the fail-closed gate
  disabled, the three new fail-closed tests fail — `default_config_refuses_
  to_bind_with_empty_bearer` and `empty_bearer_with_require_auth_refuses_
  to_bind` (lib) panic on `expect_err` (the server binds), and
  `run_mcp_http_server_refuses_empty_bearer_when_require_auth_true`
  (integration) panics on `expect_err`. The fingerprint tests pass in RED,
  confirming they cover pre-existing behavior (coverage additions, not a
  behavior change) — no RED is needed for them.
- GREEN (`tests/c4/c4e_fail_closed_auth_GREEN.log`): with the gate
  re-enabled, 15 lib + 27 integration + 0 doctest, all green.
- Feature-off (`tests/c4/c4e_feature_off_GREEN.log`): default features → 11
  lib tests green (the new `fail_closed_auth_canary` module is feature-gated
  with `http_server`; no regression to the stdio/helper path).

### C4-E acceptance mapping (HANDOFF-C4-E §Acceptance)

1. **Default config is fail-closed for auth.** ✅ —
   `run_mcp_http_server` returns `InvalidInput` for
   `McpHttpConfig::default()`; proven by
   `default_config_refuses_to_bind_with_empty_bearer` (lib) +
   `run_mcp_http_server_refuses_empty_bearer_when_require_auth_true`
   (integration).
2. **Fingerprint mismatch test GREEN.** ✅ —
   `session_bearer_fingerprint_mismatch_rejects` now exercises the real
   fingerprint branch (401 on mismatch); positive control
   `session_bearer_fingerprint_match_accepts` confirms the matching path
   accepts.
3. **Full streamable-http suite still green.** ✅ — 27 integration + 15 lib
   with `--features streamable-http`; 11 lib default-features; shared core
   `xai-grok-tower-tools`/`xai-grok-tower` green; clippy clean on new code
   (only the pre-existing `stdio.rs:9` unused-import warning).
4. **Wave note update / CHANGES.** ✅ — this section + CHANGES.md C4-E row.

## C4-F corrective — product composition wiring (GLM build)

C4-F wires `run_mcp_http_server` into the `xai-grok-pager-bin` product
composition root over the real `ShellSessionActorRuntime`, mirroring the
C3-G WS pattern with a distinct env (`GROK_OSS_MCP_HTTP=1`) and a distinct
feature (`mcp-streamable-http`). Fail-closed bearer; no self-loop; C3-G WS
wiring intact.

### What landed

| File | Change |
|---|---|
| `crates/codegen/xai-grok-pager-bin/Cargo.toml` | Optional `xai-grok-mcp-server` + `reqwest` deps; new `mcp-streamable-http` feature → `xai-grok-mcp-server/streamable-http` + `dep:reqwest`. Default build stays zero-network on the MCP side. |
| `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` | `MCP_HTTP_SERVE_ENV` (`GROK_OSS_MCP_HTTP`), `mcp_http_serve_env_enabled()`, `experimental_mcp_http_runtime[_with_root]()` (real `Arc<dyn GrokRuntimeFacade>` from `ShellSessionActorRuntime::new(root)`, NOT FakeRuntime — the MCP listener takes the facade directly, not a `FacadeProcessor`), `mcp_http_server_config()` (always `require_auth: true`, fail-closed), `run_mcp_http[_with_root]()` + 5-test `mcp_http_composition_tests` module (feature-gated). |
| `crates/codegen/xai-grok-pager-bin/src/main.rs` | Env-gated Serve dispatch to `run_mcp_http` under `GROK_OSS_MCP_HTTP=1` + `mcp-streamable-http`; resolves Tower instance via existing `select_tower_instance_id`; `print_mcp_http_startup_info` (honest TLS HUMAN gate D-SEC.13/MCP102-HUMAN); `McpHttpGuard` (RAII abort). Inserted before the C3-G WS block; distinct env so no collision. |

### Acceptance mapping (HANDOFF-C4-F §Acceptance)

1. **Product can start loopback MCP HTTP with required bearer.** ✅ REAL —
   `run_mcp_http` builds the real `experimental_mcp_http_runtime()` (→
   `ShellSessionActorRuntime::new(grok_home())`) and calls
   `xai_grok_mcp_server::run_mcp_http_server`. CLI: `grok agent serve --bind
   127.0.0.1:0 --secret <token>` with `GROK_OSS_MCP_HTTP=1` (`main.rs` Serve
   arm). `mcp_http_composition_bind_auth_and_dispatch_roundtrip` asserts
   `handle.addr.ip() == 127.0.0.1`.
2. **Self-loop guard still holds.** ✅ REAL — three layers preserved (symbol +
   composition + runtime). The composition-level guard
   `composition_source_does_not_register_local_mcp_self_loop` (mcp-server
   integration suite) still passes: the composition source contains no
   contiguous `register_self` / `http://127.0.0.1:8788/mcp` literal. The local
   `mcp_http_composition_does_not_self_register_local_mcp` guard reconstructs
   the forbidden tokens from parts so it does not self-trip. Runtime guard
   `post_tools_call_does_not_reenter_via_managed_mcp_client` (C4-B) unchanged.
3. **Test or documented smoke path.** ✅ REAL — 5 composition tests
   (`mcp_http_composition_tests`): bind/auth/dispatch roundtrip, fail-closed
   empty+whitespace bearer, config-requires-auth invariant, env-gate
   default-off, self-loop guard. Documented CLI/env path in `main.rs` +
   SCRATCH.
4. **PARTIAL TLS HUMAN.** ✅ PARTIAL (by contract) —
   `print_mcp_http_startup_info` prints `"TLS: not provided (HUMAN gate
   D-SEC.13 / MCP102-HUMAN — cleartext only)"`. The listener (`run_mcp_http_server`,
   C4-B) emits `bind_warning` for non-loopback cleartext. TLS itself stays a
   HUMAN gate; this wave never advertises production TLS and never
   auto-promotes a cleartext remote bind.

### C4-F RED → GREEN

- RED (`tests/c4/c4f_mcp_composition_RED.log`): with `require_auth` stubbed to
  `false` in `mcp_http_server_config`, 3/5 tests fail —
  `mcp_http_composition_bind_auth_and_dispatch_roundtrip` (wrong bearer not
  rejected with 401), `mcp_http_composition_fail_closed_on_empty_bearer`
  (empty bearer binds instead of refusing), and
  `mcp_http_config_requires_auth_by_default` (invariant broken). Proves the
  composition tests catch missing auth, not just a happy path.
- GREEN (`tests/c4/c4f_mcp_composition_GREEN.log`): with the real
  `require_auth: true`, all 5 `mcp_http_composition_tests` pass (5 passed; 0
  failed) across all three bin targets.
- GREEN gate (`tests/c4/c4f_mcp_composition_GREEN_gate.log`):
  `scripts/run-rust-test-gate.sh mcp_http cargo test -p xai-grok-pager-bin
  --features mcp-streamable-http mcp_http` exits 0.

### C4-F regression

- `cargo check -p xai-grok-pager-bin` (default) → OK.
- `cargo check -p xai-grok-pager-bin --features app-server-ws` → OK (C3-G WS
  wiring intact).
- `cargo check -p xai-grok-pager-bin --features mcp-streamable-http` → OK.
- `cargo check -p xai-grok-pager-bin --features mcp-streamable-http,app-server-ws`
  → OK.
- `cargo test -p xai-grok-pager-bin app_server_composition` (default) → 11/11
  (MCP composition tests filtered out without the feature).
- `cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws`
  → 3/3 (C3-G WS composition intact).
- `cargo test -p xai-grok-mcp-server --features streamable-http` → 15 lib +
  27 integration all green (self-loop guard
  `composition_source_does_not_register_local_mcp_self_loop` passes).
- `cargo clippy -p xai-grok-pager-bin --features mcp-streamable-http
  --all-targets` → no new warnings in C4-F code (only pre-existing warnings:
  `main.rs:1249` else-if formatting in `cache_outgoing_acp_state`, the
  multi-bin-target Cargo.toml note, and the pre-existing
  `xai-grok-mcp-server` `stdio.rs:9` unused-import).
- Pre-existing failure `is_managed_install_matches_only_the_bin_grok_target`
  (grok-oss identity cutover) — unchanged, not C4-F.

### C4-F design decisions

| ID | Decision | Rationale |
|---|---|---|
| R-C4F-1 | Distinct env `GROK_OSS_MCP_HTTP` (not reusing `GROK_OSS_APP_SERVER`) | The handoff requires a distinct env so the MCP HTTP and WS experimental paths do not collide. Each gate is independent; if both envs are set, the MCP HTTP block (checked first) wins, but each alone triggers only its own path. Default (both unset) keeps the shell agent server. |
| R-C4F-2 | New `mcp-streamable-http` cargo feature (not reusing `app-server-ws`) | The MCP HTTP listener lives in a different crate (`xai-grok-mcp-server`) with its own `streamable-http` feature. A dedicated pager-bin feature makes the product opt-in explicit and keeps the default build zero-network on the MCP side. |
| R-C4F-3 | `experimental_mcp_http_runtime` returns `Arc<dyn GrokRuntimeFacade>` directly (not a `FacadeProcessor`) | The MCP HTTP listener takes `Arc<dyn GrokRuntimeFacade>` directly (it routes `tools/call` through `invoke_tower_tool` itself), unlike the WS listener which takes `Arc<FacadeProcessor>`. The composition reuses the same real `ShellSessionActorRuntime::new(root)` port as the WS path — single authority, no FakeRuntime on the product path. |
| R-C4F-4 | Fail-closed bearer via the existing F-2 gate in `run_mcp_http_server` | The product config builder always sets `require_auth: true`; the listener refuses to bind on an empty/whitespace bearer (C4-E F-2). The CLI surfaces this as an `anyhow` error with a `fail-closed` hint. The product `--secret`/`GROK_AGENT_SECRET` is non-empty (auto-generated when not supplied). |
| R-C4F-5 | Local self-loop guard reconstructs forbidden tokens from parts | The mcp-server integration guard `composition_source_does_not_register_local_mcp_self_loop` scans the whole `app_server_composition.rs` file (not just non-test) for the contiguous literals `register_self` and `http://127.0.0.1:8788/mcp`. A naive local guard that wrote those literals as assertion arguments would self-trip the integration guard. Reconstructing them via `format!("{}{}", "register_", "self")` keeps the local guard meaningful without breaking the integration guard. |

### C4-F residual (outside this wave's bound)

- TLS termination (HUMAN gate D-SEC.13 / MCP102-HUMAN — never auto-resolved).
- SSE live push + disconnect-cancels-turn (C4-B residuals — the facade has no
  push seam / per-turn handle exposed to the HTTP layer).
- CLI `--mcp off|stdio|http://ADDR` matrix (vs the env gate) — composition/CLI
  follow-on; the env gate is the documented CLI/env path per handoff
  acceptance.
- Production `spawn_session_on_thread` assembly (C1-J/C2-A BLOCKER) — the
  composition uses the real `ShellSessionActorRuntime` facade, whose turn
  methods surface the BLOCKER honestly via `no_resident_error`.

