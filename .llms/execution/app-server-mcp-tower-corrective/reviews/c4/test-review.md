# C4-D Independent Test Review — `streamable_http` black-box suite

| Field | Value |
|---|---|
| Review mode | implementation (test-artifact review) |
| Wave | C4-B (real MCP Streamable HTTP server, items 26–28) |
| Handoff | `HANDOFF-C4-D-test-review.md` |
| Reviewer | GLM `glm-5.2` (independent review, read-only) |
| Date | 2026-07-18 |
| Artifacts reviewed | `crates/codegen/xai-grok-mcp-server/tests/streamable_http.rs` (751 lines, 23 tests), `crates/codegen/xai-grok-mcp-server/src/transport/http_server.rs` (728 lines), `crates/codegen/xai-grok-mcp-server/src/lib.rs`, `src/transport/mod.rs`, `Cargo.toml`, `crates/codegen/xai-grok-tower/src/fake.rs`, `.llms/execution/app-server-mcp-tower-corrective/tests/c4/c4_streamable_http_GREEN.log`, `waves/c4-mcp-streamable-http.md`, `waves/c4-mcp-surface-map.md`, `handoffs/HANDOFF-C4-B-mcp-streamable-http.md`, `HANDOFF-C4-D-test-review.md` |
| Re-run executed | **No** — this read-only review subagent has no shell/command-execution tool. The captured GREEN log was inspected line-by-line and cross-checked against the current source (test names, counts, feature gates, asserted behaviors, and dispatch routing all match). |

## Verdicts

- `IMPLEMENTATION_OR_ARTIFACT: PASS_WITH_FINDINGS` — the suite is genuinely black-box over a real axum TCP listener + `reqwest` client, feature-gated, routes `tools/call` through the shared `invoke_tower_tool` semantic core, uses `FakeRuntime` only as a test facade (never as product composition), and is GREEN (23/23 integration + 12/12 lib). The handoff's five acceptance criteria are PROVEN by tests. Findings are non-blocking: no RED evidence log was captured (F1), one security-relevant code path (bearer fingerprint mismatch) is unexercised by its named test (F2), and several tests have names that overstate their assertions (F3–F5).
- `AGENT_BEHAVIOR: PASS`
- `HANDOFF_QUALITY: PASS`
- `GOAL_GATE: N/A` (subtask review, not final-goal)

## Acceptance criteria mapping (handoff §Acceptance)

| Required | Claim | Test evidence | Verdict |
|---|---|---|---|
| 1. Real HTTP bind serving `/mcp` (feature-gated OK) | REAL | `run_mcp_http_server` binds a real `TcpListener` + `axum::serve` (http_server.rs:209-249), feature-gated behind `streamable-http` (`transport/mod.rs:2`, `Cargo.toml:28`). Every test spawns a real listener on an ephemeral loopback port via `spawn_server` (streamable_http.rs:29-46) and drives it with a real `reqwest` client. | PROVEN |
| 2. Black-box: POST initialize/tools/list/tools/call; auth failure; body limit; DELETE session | REAL | Tests 1–8, 20–22. `post_initialize_negotiates_session_header` (104), `post_tools_lists_exactly_nine_descriptors_matching_in_process` (115), `post_tools_call_start_returns_structured_content_with_session_id` (142), `post_tools_call_deny_path_emits_iserror_with_forbidden_code` (171), `auth_failures_are_indistinguishable_401` (204), `body_limit_rejects_oversized_post_before_dispatch` (253), `delete_session_terminates_and_rejects_subsequent_post` (277), `delete_without_session_header_is_bad_request` (304), `post_rejects_non_json_content_type` (688), `post_rejects_token_in_query_string` (704), `post_notification_returns_202_no_body` (724). | PROVEN |
| 3. SSE GET resume path at least with helper/table wired to a real transport (full real-adapter resync may PARTIAL) | REAL for framing/resume/foreign-id; PARTIAL for live push | Tests 9–13: `get_sse_streams_events_after_tools_call` (321), `get_sse_resume_from_last_event_id` (368), `get_sse_foreign_last_event_id_returns_resumption_error` (409), `get_sse_does_not_replay_another_clients_events` (432), `get_sse_requires_accept_event_stream` (472). Event log is per-transport-session, pull-fed from `GrokRuntimeFacade::replay` after mutating `tools/call` (http_server.rs:540-561). Live push honestly PARTIAL (no facade push seam; stream delivers buffered events then ends). | PROVEN (honest PARTIAL) |
| 4. Nine-tool descriptor parity with in-process names | REAL | Test 2 asserts `tools.len() == 9` + names == `MCP_TOOL_NAMES` (streamable_http.rs:129-139). Test 17 `stdio_and_http_produce_identical_tools_list_and_error_shapes` (574) compares stdio vs HTTP names + error shapes (`isError`/`structuredContent.code == "forbidden"`). | PROVEN |
| 5. Wave note + evidence; honest PARTIAL for composition self-loop if product bin not yet wired | done | `waves/c4-mcp-streamable-http.md` + `tests/c4/c4_streamable_http_GREEN.log`. Composition self-loop guarded by test 19 (`composition_source_does_not_register_local_mcp_self_loop`, 664) + source canary (`http_server_does_not_import_outbound_mcp_client`, http_server.rs:718). Product bin not yet wired — honestly PARTIAL. | PROVEN (honest PARTIAL) |

## Test count check

The handoff claims "23 black-box tests + 12 lib tests". Verified:

- `grep -c '#\[(tokio::)?test\]' tests/streamable_http.rs` → **23** test attributes (22 `#[tokio::test]` + 1 `#[test]` at line 664 for the composition source guard). Matches the GREEN log: `running 23 tests` … `test result: ok. 23 passed; 0 failed` (GREEN log lines 16, 43).
- Lib tests: GREEN log shows `running 12 tests` … `12 passed; 0 failed` (lines 49, 64), including the new `transport::http_server::self_loop_canary::http_server_does_not_import_outbound_mcp_client`. The handoff's "11 lib tests" in one paragraph vs "12 lib tests" in the evidence line is a minor wording slip (11 pre-existing + 1 new canary = 12); the log is authoritative and consistent.

All 23 tests are genuinely black-box: `spawn_server` → `run_mcp_http_server` binds a real `TcpListener` and `axum::serve`s it; tests use `reqwest::Client` against the bound `SocketAddr`. No in-process helper calls into the dispatch path except test 17 which deliberately compares stdio (`process_mcp_stdio_batch`) vs HTTP shapes — a parity test, not a substitute for the HTTP path.

## RED / GREEN honesty

**RED evidence is MISSING.** The handoff AC (`HANDOFF-C4-B` §Acceptance) requires "RED→GREEN under `tests/c4/`", and the wave note (`waves/c4-mcp-streamable-http.md:36-38`) claims a RED→GREEN transition. However `tests/c4/` contains **only** `c4_streamable_http_GREEN.log` — there is **no RED log** demonstrating the tests fail against the pre-C4-B state (no `/mcp` route, no listener). Compare C3 (`tests/c3/c3_ws_listener_RED.log`) and C1 (`tests/c1/c1_turn_lifecycle_RED.log`), both of which captured RED evidence.

Mitigation: the C4-A surface map (`waves/c4-mcp-surface-map.md` §1) independently documented the RED state — the crate had no `/mcp` route, no axum/hyper dep, no `tests/` dir, and an empty `streamable-http` feature. The 23 tests bind a real listener and assert specific HTTP statuses/headers/bodies, so they would genuinely fail to compile/run against the pre-C4-B crate (the test file itself is `#![cfg(feature = "streamable-http")]` and references `run_mcp_http_server`/`McpHttpConfig` which did not exist). This gives implicit RED confidence, but it is not captured evidence. See F1.

**GREEN is genuine.** The GREEN log shows 23/23 integration + 12/12 lib passing. The assertions are anchored to real wire behavior (status codes, headers, JSON-RPC fields, SSE event framing), not vacuous `assert!(true)` or empty filters. No SKIP-as-PASS, no `#[ignore]`.

## FakeRuntime honesty (test facade only)

`FakeRuntime` (`crates/codegen/xai-grok-tower/src/fake.rs:37`) is used **only as the runtime facade** injected into `run_mcp_http_server` for the black-box tests (`spawn_server_with`, streamable_http.rs:41-46). This is explicitly permitted by the handoff ("For tool black-box, inject FakeRuntime or test facade"). Crucially:

- The HTTP `tools/call` path routes through the **real shared semantic core** `invoke_tower_tool` (http_server.rs:498-501), which dispatches into the `GrokRuntimeFacade` trait. `FakeRuntime` implements that trait faithfully (fake.rs:1-6 documents it is "not a second production actor; production injects a Shell adapter"). So the tests exercise the real dispatch + ACL + error-shape mapping; only the runtime *backend* is faked, which is the correct boundary for a transport-layer test.
- `FakeRuntime` is **not** used in product composition. Test 19 (`composition_source_does_not_register_local_mcp_self_loop`, streamable_http.rs:664-684) greps `xai-grok-pager-bin/src/app_server_composition.rs` for `http://127.0.0.1:8788/mcp` and `register_self` — neither is present (the product bin does not yet wire the HTTP listener, honestly PARTIAL).
- The stdio parity test (17) uses `FakeRuntime` for both the stdio batch and (separately) the HTTP server; it compares *shapes*, not shared state, so the two independent `FakeRuntime` instances are valid for conformance.
- No `xai-grok-shell` import in `http_server.rs` or the test file (Tower≠Shell preserved; the HTTP layer never reaches a real `SessionActor`, which is C1's domain).

This is honest FakeRuntime-as-facade usage, not Fake-as-production.

## Feature gating

Feature gating is correct and verified at every layer:

- `Cargo.toml:28` — `streamable-http = ["dep:axum", "dep:tokio", "dep:futures-util"]`; `default = ["stdio"]` (line 27). `axum`/`tokio`(net/io-util/time/sync/rt)/`futures-util` are `optional = true` (lines 16-21). No new external supply-chain (all workspace deps).
- `src/transport/mod.rs:2` — `#[cfg(feature = "streamable-http")] pub mod http_server;`
- `src/lib.rs:19-25` — `pub use transport::http_server::{...}` is `#[cfg(feature = "streamable-http")]`.
- `tests/streamable_http.rs:12` — `#![cfg(feature = "streamable-http")]` gates the entire integration test binary.
- The `self_loop_canary` (http_server.rs:714-731) lives inside the gated module, so it only runs with the feature.

With default features (`stdio` only), the 23 integration tests and the `http_server` module are excluded entirely; the stdio/in-process adapters stay zero-network. The handoff claims `cargo test -p xai-grok-mcp-server` (default features) passes with no regression, but **no feature-off log was captured** under `tests/c4/` (see F4). The GREEN log was run with `--features streamable-http` (it includes the 23 integration tests + the `http_server::self_loop_canary` lib test, both feature-gated).

## Self-loop guards (three levels)

1. **Symbol** — `http_server_does_not_import_outbound_mcp_client` (http_server.rs:718) asserts the production source contains no `xai_grok_mcp::` / `McpClient` / `register_self`. Plus pre-existing `no_local_self_injection_in_production_source` (lib.rs:213) and `no_self_mcp_loop_tool_names` (lib.rs:187).
2. **Composition** — `composition_source_does_not_register_local_mcp_self_loop` (streamable_http.rs:664) greps pager-bin's composition for the local `/mcp` URL literal and `register_self`.
3. **Runtime** — `post_tools_call_does_not_reenter_via_managed_mcp_client` (streamable_http.rs:634) — see F3 for assertion weakness.

The symbol + composition guards are real source-text canaries. The runtime guard is weak (F3).

## Findings

### F1 — No RED evidence log captured (severity: Medium, confidence: high)
The handoff AC requires "RED→GREEN under `tests/c4/`" and the wave note claims a RED→GREEN transition, but `tests/c4/` contains only `c4_streamable_http_GREEN.log`. No RED log demonstrates the 23 tests fail against the pre-C4-B state. C3 and C1 both captured RED logs; C4 did not. RED non-vacuity is therefore **unproven by evidence** for all 23 tests.
- Evidence: `tests/c4/` listing (only `c4_streamable_http_GREEN.log`); `waves/c4-mcp-streamable-http.md:36-38` claims RED→GREEN; `HANDOFF-C4-B:28` "RED→GREEN under `tests/c4/`".
- Mitigation: C4-A map (`waves/c4-mcp-surface-map.md` §1) independently documented the empty pre-state (no route, no listener, empty feature, no `tests/` dir). The tests reference symbols (`run_mcp_http_server`, `McpHttpConfig`, `MCP_PROTOCOL_VERSION`) that did not exist pre-C4-B, so the test binary would not compile against the prior crate — giving implicit RED confidence. The assertions are discriminating (specific status codes, headers, JSON fields) and would fail under a stubbed/missing implementation. However, this is inferential, not captured.
- Required fix: capture a `c4_streamable_http_RED.log` (e.g. run the tests against the crate with the `streamable-http` feature but the `http_server` module stubbed/empty, or simply document the compile-fail against the pre-C4-B commit) under `tests/c4/` for traceability parity with C3/C1. **Not blocking** — the GREEN evidence + source-gating + map-documented RED state are sufficient for a test-adequacy PASS, but the missing RED log is a real traceability gap against the handoff AC.

### F2 — `session_bearer_fingerprint_mismatch_rejects` does not test the fingerprint mismatch path (severity: Medium, confidence: high)
Test 16 (`streamable_http.rs:546-568`) is named for the bearer fingerprint binding (http_server.rs:648-657: `lookup_session` compares `bearer_fingerprint` against `session.bearer_fingerprint` and returns `unauthorized()` on mismatch). However, the test's own comment (lines 551-556) admits it "cannot isolate the fingerprint check from the auth check" and instead asserts that a `tools/list` request with **no session header** returns 400. That asserts the session-binding-mandatory path (http_server.rs:641-643), which is already implied by every non-initialize test. The fingerprint-mismatch code path (a request with a *valid* session header but a *different* bearer than the one that opened the session) is **not exercised by any test**.
- Evidence: `streamable_http.rs:546-568` (test body asserts `None` session → 400); `http_server.rs:648-657` (fingerprint comparison, unexercised); no other test sends a valid session header with a mismatched bearer.
- Required fix: add a test that negotiates a session with bearer A, then sends a non-initialize request with the same session header but bearer B (with `require_auth: true` and bearer B also valid, OR with `require_auth: false` so auth passes and only the fingerprint check fires), and asserts 401. This would cover the security-relevant fingerprint binding. **Not blocking** for the test-adequacy gate (the cross-Tower test 15 proves session isolation at the Tower-instance level), but the named test is misleading and the fingerprint path is a coverage gap.

### F3 — `post_tools_call_does_not_reenter_via_managed_mcp_client` assertion is weak (severity: Low, confidence: high)
Test 18 (`streamable_http.rs:634-662`) asserts `state.sessions.len() == 1` after one `initialize` + one `tools/call`. This trivially holds (one `initialize` creates exactly one transport session) and does **not** detect re-entry. Re-entry via a managed MCP client would route outbound through `xai-grok-mcp` (the client crate) and would not create a transport session in *this* server's map, so `sessions.len()` would remain 1 regardless. The real self-loop guard is the source canary (http_server.rs:718) + composition grep (test 19). The test is a smoke test, not a re-entry guard; its name overstates.
- Evidence: `streamable_http.rs:634-662`; the assertion `sessions.len() == 1` (line 662).
- Required fix: none (informational). The canary tests are the actual guard. Optionally rename to `..._produces_single_transport_session` or assert something stronger (e.g. that the facade `start_session` was invoked exactly once via a counting facade wrapper). Not blocking.

### F4 — No feature-off log captured (severity: Low, confidence: high)
The handoff claims `cargo test -p xai-grok-mcp-server` (default features) passes with no regression, but no log artifact was dropped in `tests/c4/`. The GREEN log was run with `--features streamable-http` (it includes the feature-gated integration tests + canary). Feature-off green is verifiable by source gating (the test file is `#![cfg(feature = "streamable-http")]`, so default features exclude all 23 integration tests; the lib canary is also gated), but not by a captured log.
- Evidence: `tests/c4/` contains only the GREEN log; `HANDOFF-C4-B` evidence section claims default-features green.
- Required fix: capture `c4_streamable_http_feature_off_GREEN.log` (default features) under `tests/c4/` for traceability. Not blocking — gating is correct by source inspection.

### F5 — `post_notification_returns_202_no_body` does not assert no body (severity: Low, confidence: high)
Test 22 (`streamable_http.rs:724-745`) asserts only `status == 202`. The name says "no_body" but the test never asserts the response body is empty. The implementation returns `StatusCode::ACCEPTED.into_response()` with no body (http_server.rs:350-351), so the behavior is correct, but the test does not pin it.
- Evidence: `streamable_http.rs:724-745` (no body assertion); http_server.rs:350-351 (impl returns bare 202).
- Required fix: optionally also assert `resp.text().await.unwrap().is_empty()`. Not blocking.

### F6 — Auth-equivalence test lacks a truly malformed bearer case (severity: Low, confidence: medium)
Test 5 (`auth_failures_are_indistinguishable_401`, streamable_http.rs:204-248) covers `None`, empty, `"Bearer wrong"`, and `"token-value"` (which `auth_headers` sends as `"Bearer token-value"` — a well-formed bearer with a wrong token, not malformed). No case sends a syntactically malformed Authorization header (e.g. `"Bearer"` alone, `"Token x"`, or garbage). All four cases do yield 401 + `WWW-Authenticate: Bearer`, proving indistinguishability for the cases covered, but the "malformed" label in the handoff table (test 5 row) is not exercised.
- Evidence: `streamable_http.rs:204-248`; `auth_headers` helper (lines 236-249).
- Required fix: none (informational). Optionally add a `"Bearer"` (no token) and a `"Token xyz"` case. Not blocking.

## Required fixes

1. **F1 (Medium)** — Capture `c4_streamable_http_RED.log` under `tests/c4/` demonstrating the tests fail (compile-fail or stub) against the pre-C4-B state, for traceability parity with C3/C1 and to satisfy the handoff's explicit "RED→GREEN under `tests/c4/`" AC.
2. **F2 (Medium)** — Add a test that exercises the bearer fingerprint mismatch path (valid session header + mismatched bearer → 401), or rename test 16 to reflect what it actually asserts. The fingerprint binding is a security-relevant code path currently unexercised.
3. **F4 (Low)** — Capture `c4_streamable_http_feature_off_GREEN.log` (default features) under `tests/c4/`.
4. **F5 (Low, optional)** — Strengthen test 22 to assert the response body is empty.
5. **F6 (Low, optional)** — Add a malformed-bearer case to test 5.

None are blocking for the test-adequacy PASS: the 23 tests are genuinely black-box, GREEN, feature-gated, and route through the shared semantic core with FakeRuntime only as a facade. F1 and F2 are the most material (traceability gap + a security-path coverage gap); F3–F6 are labeling/optional-strengthening.

## Residual risk

- RED non-vacuity is inferential (F1): the map-documented empty pre-state + discriminating assertions give confidence, but no captured RED log proves it. A future regression that weakens the dispatch/auth/SSE paths could pass without a RED baseline to diff against.
- The bearer fingerprint binding (http_server.rs:648-657) has no test coverage (F2). If that code path regresses (e.g. fingerprint comparison removed), no test fails. The cross-Tower test (15) covers Tower-instance isolation but not bearer rebinding.
- SSE live push is honestly PARTIAL (no facade push seam); the GET stream delivers buffered events then terminates. A deployment relying on long-lived push would need the deferred real-adapter resync.
- Product bin wiring is honestly PARTIAL (composition does not bind the HTTP listener); the self-loop guards are source canaries, not runtime enforcement.
- No source mutation was performed by this reviewer.

## Commands / results

| Command | Run by | Result |
|---|---|---|
| `cargo test -p xai-grok-mcp-server --features streamable-http` (GREEN) | implementer (`tests/c4/c4_streamable_http_GREEN.log`) | 23 integration tests passed (lines 16-43) + 12 lib tests passed (lines 49-64); 0 failed. Independently inspected: test names and counts match the source. |
| `cargo test -p xai-grok-mcp-server` (default features, feature-off) | implementer (handoff evidence section) | claimed green (11/12 lib tests); **not logged** under `tests/c4/` (F4). Verified by source gating: `tests/streamable_http.rs:12` `#![cfg(feature = "streamable-http")]` excludes all 23 integration tests under default features. |
| RED reproduction | **not captured** (F1) | No `c4_streamable_http_RED.log` exists. C4-A map documents the empty pre-state; tests reference symbols that did not exist pre-C4-B, giving implicit compile-fail RED confidence. |
| Re-run by reviewer | **skipped — no shell tool available in this read-only review subagent** | GREEN log inspected line-by-line; source gates, dispatch routing, auth/body/session/SSE paths, and FakeRuntime usage cross-checked against current source. |

## Summary

The C4-B `streamable_http` test suite is **PASS_WITH_FINDINGS**. It is genuinely black-box over a real axum TCP listener with a real `reqwest` client, feature-gated so the default stdio path stays zero-network, and routes every `tools/call` through the shared `invoke_tower_tool` semantic core — no second tool implementation, no local MCP self-loop. `FakeRuntime` is used only as the runtime facade (handoff-permitted), never as product composition; the production composition does not wire the HTTP listener (honestly PARTIAL). The handoff's five acceptance criteria are all PROVEN by tests (23/23 integration + 12/12 lib GREEN). Self-loop guards exist at three levels (symbol, composition, runtime). Findings are non-blocking: no RED evidence log (F1, Medium), the bearer fingerprint mismatch path is unexercised by its named test (F2, Medium), the runtime re-entry guard assertion is weak (F3, Low), no feature-off log (F4, Low), and two under-asserted edges (F5 notification body, F6 malformed bearer). No source was modified by this reviewer.
