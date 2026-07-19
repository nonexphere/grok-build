# C3-D Independent Test Review — `ws_listener` black-box suite

| Field | Value |
|---|---|
| Review mode | implementation (test-artifact review) |
| Wave | C3-B (real WebSocket listener, items 20/21/24) |
| Handoff | `HANDOFF-C3-D-test-review.md` |
| Reviewer | GLM `glm-5.2` (independent review, read-only) |
| Date | 2026-07-18 |
| Artifacts reviewed | `crates/codegen/xai-grok-app-server/src/lib.rs` (`ws_listener_blackbox_tests`, lines 259-669), `crates/codegen/xai-grok-app-server/src/transport/ws_listener.rs` (`ws_listener_unit_tests`, lines 388-429), `crates/codegen/xai-grok-app-server/src/transport/websocket.rs` (`validate_bearer_header`, `validate_ws_text_frame`), `crates/codegen/xai-grok-app-server/Cargo.toml`, `src/transport/mod.rs`, `.llms/execution/app-server-mcp-tower-corrective/tests/c3/{c3_ws_listener_RED.log,c3_ws_listener_GREEN.log,c3_ws_listener_GREEN_gate.log,README.md}`, `waves/c3-ws-listener.md`, `scripts/run-rust-test-gate.sh` |
| Re-run executed | **No** — this read-only review subagent has no shell/command-execution tool. Captured RED/GREEN/GREEN-gate logs were inspected line-by-line and cross-checked against the current source (test names, counts, feature gates, and asserted behaviors all match). |

## Verdicts

- `IMPLEMENTATION_OR_ARTIFACT: PASS_WITH_FINDINGS` — the test suite is genuinely black-box, RED-non-vacuous for the security-critical auth path, feature-gated, and GREEN. One labeling overcount and a few weak/under-asserted edges; none blocking.
- `AGENT_BEHAVIOR: PASS`
- `HANDOFF_QUALITY: PASS`
- `GOAL_GATE: N/A` (subtask review, not final-goal)

## Acceptance criteria mapping (handoff §Acceptance)

| Required | Claim | Test evidence | Verdict |
|---|---|---|---|
| 1. Real bind/listen/accept/upgrade (feature-gated) | REAL | `ws_listener_handshake_subprotocol_negotiates_protocol_version` (lib.rs:364) — real `connect_async` upgrade against an ephemeral listener, asserts `sec-websocket-protocol` echo. Feature gate verified: `transport/mod.rs:6` + `Cargo.toml:30` (`websocket = ["dep:…"]`). | PROVEN |
| 2a. auth fail | REAL | `ws_listener_rejects_missing_authorization_header` (lib.rs:377), `ws_listener_rejects_wrong_bearer` (lib.rs:389). **RED-proven**: `c3_ws_listener_RED.log:21,28` show both FAIL when auth stubbed (`if require_auth && false`); GREEN log shows both `ok`. | PROVEN (RED-non-vacuous) |
| 2b. valid text RPC through processor | REAL | `ws_listener_text_frame_initialize_then_session_start_roundtrip` (lib.rs:409) — sends `initialize`+`session/start` over the wire, asserts `protocolVersion` + `session.status=="ready"` + `workspaceRoot`. Routes through `FacadeProcessor::handle_line` (ws_listener.rs:328 `dispatch_text`). | PROVEN |
| 2c. oversized/batch rejection | REAL | `ws_listener_rejects_oversize_text_frame` (lib.rs:478, 1.5 MiB > 1 MiB cap), `ws_listener_rejects_jsonrpc_batch` (lib.rs:468, asserts `-32600` + "batch"). | PROVEN (see F4 for oversize assertion shape) |
| 2d. ping/pong / WS keepalive | REAL | `ws_listener_ping_pong_keepalive` (lib.rs:433) — client Ping → asserts Pong received within 2s. Impl flushes auto-pong via `Outbound::Flush` (ws_listener.rs:299-303). | PROVEN |
| 2e. disconnect cleanup | REAL | `ws_listener_disconnect_drains_and_closes` (lib.rs:509) — client Close → asserts stream ends (closed=true). | PROVEN |
| 3. Bounded writer / backpressure with test | REAL | `bounded_writer_drops_when_full` (ws_listener.rs:399, deterministic: cap=2, enqueue 5 → 2 sent, 3 dropped) + `ws_listener_bounded_writer_survives_burst` (lib.rs:537, over-the-wire cap=4). | PROVEN (see F5 for burst assertion weakness) |
| 4. Slow-client real-adapter resync | PARTIAL (deferred) | `ws_listener_slow_client_resync_via_replay_fake_adapter` (lib.rs:561) — fake-adapter only; asserts `replay.events` is array + `subscriptionId` is string. Real-adapter deferred to C3-22/23. | PROVEN (honest PARTIAL) |
| 5. Wave note + evidence | done | `waves/c3-ws-listener.md` + 3 logs under `tests/c3/`. | PROVEN |
| Item 24: conformance matches stdio | REAL | `ws_conformance_matches_stdio_method_shapes` (lib.rs:613) — compares WS vs `process_ndjson_batch` result shapes for `initialize`+`session/start`. | PROVEN |
| Binary frame rejection | REAL | `ws_listener_rejects_binary_frame` (lib.rs:457) — asserts `-32600` + "Binary". | PROVEN |
| Cleartext non-loopback policy | REAL | `ws_listener_cleartext_non_loopback_warns_experimental_unsafe` (lib.rs:593) + `ws_listener_default_config_is_loopback` (lib.rs:598). | PROVEN |

## RED non-vacuity check

**RED is genuine and non-vacuous for the security-critical auth path.**

- `tests/c3/c3_ws_listener_RED.log` was produced with the handshake auth guard stubbed to `if require_auth && false` (per the README and wave note). Under that stub:
  - `ws_listener_rejects_missing_authorization_header` FAILS (lib.rs:379 `unwrap_err()` on an `Ok` 101 upgrade — log line 30).
  - `ws_listener_rejects_wrong_bearer` FAILS (lib.rs:392 `assert!(connect_async(req).await.is_err())` — log line 38).
- Both failures prove the tests actually assert rejection, not just exercise a happy path. With the real `if require_auth { validate_bearer_header(...) }` guard (ws_listener.rs:198-204), both pass (GREEN log lines 28-29).
- The current source confirms the guard is real, not stubbed: `validate_bearer_header` (websocket.rs:17) uses `constant_time_eq` (websocket.rs:33) — no short-circuit, no placeholder.

**Scope of RED evidence:** only the 2 auth-rejection tests are RED-proven. The other 14 tests rely on discriminating assertions (error codes, status fields, shape equality) rather than a RED reproduction. This is acceptable for a test-adequacy review — the assertions are anchored to real wire behavior and would fail under a stubbed/missing implementation — but a broader RED sweep (e.g. stub `dispatch_text`, stub the bounded writer, stub the oversize cap) would strengthen the record. See F3.

## Feature-off still green

- `ws_listener` module is `#[cfg(feature = "websocket")]` (`transport/mod.rs:6`).
- `ws_listener_blackbox_tests` is `#[cfg(all(test, feature = "websocket"))]` (`lib.rs:259`).
- `ws_listener_unit_tests` lives inside `ws_listener.rs`, so it is transitively gated.
- `Cargo.toml` `default = ["in-process","stdio"]`; `websocket` is opt-in (`Cargo.toml:30`). `tokio`/`tokio-tungstenite`/`futures-util` are `optional = true` (Cargo.toml:18-21).
- **Internal consistency:** GREEN-gate log shows 42 passed with `--features websocket`; the wave note claims 26 passed without it. 42 − 16 = 26, matching the 16 ws_listener tests added by the feature. The 26 default-features tests are visible in the GREEN-gate log (conformance/controller/processor/security/replay/stdio/in-process/websocket helper tests).
- **Gap (F2):** no separate feature-off log was captured under `tests/c3/`. The claim lives only in the wave note. Source gating + arithmetic consistency verify it, but a `c3_ws_listener_feature_off_GREEN.log` artifact would close the traceability gap.

## "16 black-box claims" count

The GREEN log runs 16 tests total. Breakdown:

| # | Test | Module | Black-box (real network)? |
|---|---|---|---|
| 1 | `ws_listener_handshake_subprotocol_negotiates_protocol_version` | blackbox | yes |
| 2 | `ws_listener_rejects_missing_authorization_header` | blackbox | yes |
| 3 | `ws_listener_rejects_wrong_bearer` | blackbox | yes |
| 4 | `ws_listener_rejects_credentials_in_url_at_bind` | blackbox | yes (bind path) |
| 5 | `ws_listener_text_frame_initialize_then_session_start_roundtrip` | blackbox | yes |
| 6 | `ws_listener_ping_pong_keepalive` | blackbox | yes |
| 7 | `ws_listener_rejects_binary_frame` | blackbox | yes |
| 8 | `ws_listener_rejects_jsonrpc_batch` | blackbox | yes |
| 9 | `ws_listener_rejects_oversize_text_frame` | blackbox | yes |
| 10 | `ws_listener_disconnect_drains_and_closes` | blackbox | yes |
| 11 | `ws_listener_bounded_writer_survives_burst` | blackbox | yes |
| 12 | `ws_listener_slow_client_resync_via_replay_fake_adapter` | blackbox | yes (fake adapter) |
| 13 | `ws_listener_cleartext_non_loopback_warns_experimental_unsafe` | blackbox | no network (pure helper) |
| 14 | `ws_listener_default_config_is_loopback` | blackbox | no network (config) |
| 15 | `ws_conformance_matches_stdio_method_shapes` | blackbox | yes |
| 16 | `bounded_writer_drops_when_full` | `ws_listener_unit_tests` | **no** — deterministic in-process `mpsc` test, no network |

So **15 tests live in `ws_listener_blackbox_tests`**; the 16th (`bounded_writer_drops_when_full`) is a focused unit test of the drop guarantee. The handoff/wave phrasing "16 black-box tests" / "16 `ws_listener`/`ws_listener_blackbox` tests" slightly overcounts the black-box label (the unit test is not black-box). This is a labeling imprecision, not a coverage defect — the unit test is the right tool for the deterministic drop guarantee and the listener-level burst test exercises the same pattern over the wire. This matches the parallel C3-C code review's F-1. See F1.

## Non-vacuous gate check (`scripts/run-rust-test-gate.sh`)

The gate is **non-vacuous** (re-checked against `scripts/run-rust-test-gate.sh`):
- Requires `cargo test` as the command form (lines 8-12).
- `set -euo pipefail` (line 2): any non-zero cargo exit (any test failure) aborts before the grep.
- Grep `^test .*${expected_test}.* \.\.\. ok$` (line 21) is anchored to a `test … ok` line and requires ≥1 passing test matching the fragment.
- The GREEN-gate run used fragment `ws_listener`, which matches all 16 ws_listener tests. Combined with `set -e`, the gate effectively requires the whole `cargo test -p xai-grok-app-server --features websocket` suite (42 tests) green AND ≥1 ws_listener test present. `c3_ws_listener_GREEN_gate.log:62` confirms `test result: ok. 42 passed; 0 failed`.

Minor (informational): the gate only requires *one* matching test to pass (grep succeeds on first match); it does not count tests. Acceptable because `set -e`+`pipefail` already forces the entire `cargo test` invocation to succeed.

## Findings

### F1 — "16 black-box tests" overcounts (severity: low, confidence: high)
The handoff and wave note say "16 black-box tests" / "16 `ws_listener`/`ws_listener_blackbox` tests". 15 are black-box; the 16th (`bounded_writer_drops_when_full`, ws_listener.rs:399) is a non-network unit test of the `mpsc` drop guarantee. The unit test is appropriate (deterministic proof of the drop guarantee the listener relies on), but the label is imprecise.
- Evidence: `ws_listener.rs:388-429`; GREEN log line 19 (`transport::ws_listener::ws_listener_unit_tests::bounded_writer_drops_when_full ... ok`).
- Required fix: relabel as "15 black-box + 1 focused unit test (16 total)" in the wave note / README. Not blocking — coverage is sound; this is a labeling correction. (Corroborated by C3-C code review F-1.)

### F2 — No feature-off log captured (severity: low, confidence: high)
The wave note claims `cargo test -p xai-grok-app-server` (default features) → 26 passed; 0 failed, but no log artifact was dropped in `tests/c3/`. The "feature off still green" AC is verifiable by source gating + arithmetic consistency (42 − 16 = 26) but not by a captured log.
- Evidence: `tests/c3/` contains only `RED`, `GREEN`, `GREEN_gate` logs; wave note §3.
- Required fix: drop a `c3_ws_listener_feature_off_GREEN.log` for traceability. Not blocking — gating is correct by source inspection.

### F3 — RED evidence covers only 2/16 tests (severity: low, confidence: medium)
The RED run stubs only the auth guard, proving non-vacuity for the 2 auth-rejection tests. The other 14 tests (oversize, batch, binary, ping/pong, disconnect, bounded writer, conformance, subprotocol) have discriminating assertions but no RED reproduction. Their assertions are anchored to real wire behavior (error codes, status fields, shape equality, Pong presence, stream-end) and would fail under a stubbed implementation, so they are not vacuous — but a broader RED sweep would raise confidence.
- Evidence: `c3_ws_listener_RED.log` (only 6 tests run, 2 FAIL); remaining 14 tests GREEN-only.
- Required fix: none (informational). A future RED sweep stubbing `dispatch_text` / the oversize cap / the bounded writer would strengthen the record. Not blocking.

### F4 — `ws_listener_rejects_oversize_text_frame` is a negative-only assertion (severity: low, confidence: medium)
The test asserts `!got_response` (lib.rs:504) — i.e. no `Message::Text` was delivered. It proves the server does not *process* the oversize frame, but it does not assert the connection terminates with a specific close/error signal. A server that silently dropped the frame without closing would also pass. The WS-layer cap (`max_message_size = 1 MiB`, ws_listener.rs:227-229) does terminate the connection by tungstenite contract, but the test does not observe that.
- Evidence: `lib.rs:478-506`; ws_listener.rs:227-229.
- Required fix: optionally also assert the stream ends (`Ok(None)` or `Ok(Some(Ok(Message::Close(_))))` / `Ok(Some(Err(_)))`) within the timeout loop, so the test distinguishes "rejected + closed" from "silently dropped". Not blocking — the rejection is real by impl contract.

### F5 — `ws_listener_bounded_writer_survives_burst` lower bound is weak (severity: low, confidence: medium)
With cap=4 and 16 requests, the test asserts `responses >= 1` (lib.rs:555). This proves the connection survives and delivers *something*, but it would pass even if 15/16 responses were dropped. The drop guarantee itself is proven deterministically by `bounded_writer_drops_when_full` (cap=2 → exactly 2 sent, 3 dropped), so the listener-level test is a survival smoke, not a drop-count assertion. Acceptable as a complement to the unit test, but the name "survives_burst" slightly overstates vs. the assertion.
- Evidence: `lib.rs:537-557`; `ws_listener.rs:399-428`.
- Required fix: none (informational). Optionally assert `responses` is within `[cap, 16]` or observe `dropped_events > 0` to tie the over-the-wire behavior to the drop counter. Not blocking.

### F6 — `ws_conformance_matches_stdio_method_shapes` uses independent processors (severity: low, confidence: medium)
The test builds `p_stdio` (lib.rs:624) and a separate listener-backed processor (`spawn_listener` creates its own `FacadeProcessor::new(FakeRuntime::new())`, lib.rs:288). It compares result *shapes*, not shared-state results, so the two processors being independent is fine for shape conformance — but the test name implies a single shared processor. The shape comparison (protocolVersion, capabilities.sessions.start, session.status, workspaceRoot) is valid and non-vacuous against `FakeRuntime` determinism.
- Evidence: `lib.rs:613-668`; `spawn_listener` at lib.rs:283.
- Required fix: none — shape conformance is the intended AC (item 24). Informational only; the test is correct for what it claims.

## Real-adapter vs Fake-only check

The suite is genuinely over-the-wire, not Fake-only:
- `spawn_listener` (lib.rs:283) calls `run_ws_listener` which binds a real `TcpListener` (ws_listener.rs:147) and performs `accept_hdr_async_with_config` (ws_listener.rs:226). Tests connect with `tokio_tungstenite::connect_async` (lib.rs:327) — real TCP + WS handshake.
- The processor is `FakeRuntime`-backed, which is correct for a transport-layer test: the AC is listener/handshake/auth/frames/backpressure, not session-actor behavior (that is C1's domain). The slow-client resync test honestly uses the fake adapter and labels the real-adapter variant as PARTIAL/deferred (lib.rs:562-565).
- No `xai-grok-shell` import in the listener or tests (Tower≠Shell preserved; matches wave note §1 and the C3-C code review).

## "unsupported"/PARTIAL honesty check

- Slow-client real-adapter resync: explicitly PARTIAL, deferred to C3-22/23, documented in the test body (lib.rs:562-565) and wave note §6 R-WS-3. Honest — no fake success.
- No test claims production TLS; the cleartext non-loopback policy is labeled `experimental/unsafe` and TLS stays HUMAN (ws_listener.rs:13-16; test at lib.rs:593). Honest.

## Required fixes

1. **F1 (low)** — Relabel "16 black-box tests" → "15 black-box + 1 unit test (16 total)" in `waves/c3-ws-listener.md` and `tests/c3/README.md`.
2. **F2 (low)** — Capture `c3_ws_listener_feature_off_GREEN.log` (default features) under `tests/c3/` for traceability.
3. **F4 (low, optional)** — Strengthen `ws_listener_rejects_oversize_text_frame` to also assert the stream terminates (close/error/None), not just `!got_response`.
4. **F5 (low, optional)** — Strengthen `ws_listener_bounded_writer_survives_burst` to observe `dropped_events > 0` or bound `responses`, tying the wire test to the drop counter.

None are blocking. F1 and F2 are documentation/traceability corrections; F4/F5 are optional test-strengthening. No behavior defect in the implementation-under-test.

## Residual risk

- The core REAL claims (bind/listen/upgrade, auth fail, text RPC, oversize/batch/binary rejection, ping/pong, disconnect cleanup, bounded writer, conformance) are well-proven and low-risk.
- RED non-vacuity is proven for the auth path only (F3); the remaining tests rely on discriminating assertions — acceptable but not RED-reproduced.
- Feature-off green is verified by source gating + arithmetic, not a captured log (F2).
- The 16-vs-15 black-box label is imprecise (F1) but coverage is sound.
- No source mutation was performed by this reviewer.

## Commands / results

| Command | Run by | Result |
|---|---|---|
| `cargo test -p xai-grok-app-server --features websocket ws_listener` (RED, auth stubbed) | implementer (`c3_ws_listener_RED.log`) | 4 passed, 2 failed (`ws_listener_rejects_missing_authorization_header`, `ws_listener_rejects_wrong_bearer`) — independently inspected: log lines 21-39 |
| `cargo test -p xai-grok-app-server --features websocket ws_listener` (GREEN, real auth) | implementer (`c3_ws_listener_GREEN.log`) | 16 passed, 0 failed (5.09s) — independently inspected: log lines 19-33 |
| `bash scripts/run-rust-test-gate.sh ws_listener cargo test -p xai-grok-app-server --features websocket` | implementer (`c3_ws_listener_GREEN_gate.log`) | exit 0; 42 passed, 0 failed (5.07s); gate fragment `ws_listener` matched — independently inspected: log lines 18-62 |
| `cargo test -p xai-grok-app-server` (default features, feature-off) | implementer (wave note §3) | 26 passed, 0 failed (claimed) — **not logged** (F2); verified by source gating + 42−16=26 consistency |
| Re-run by reviewer | **skipped — no shell tool available in this read-only review subagent** | Logs inspected line-by-line; source gates and assertions cross-checked |

## Summary

The C3-B `ws_listener` test suite is **PASS_WITH_FINDINGS**. It is genuinely black-box over a real TCP+WS listener, RED-non-vacuous for the security-critical auth path, feature-gated so the default stdio/in-process path stays zero-network, and GREEN (16/16 ws_listener, 42/42 package). The gate script is non-vacuous. The handoff's five acceptance criteria plus item 24 conformance are all PROVEN by tests. Findings are low-severity: a "16 black-box" labeling overcount (F1), a missing feature-off log (F2), RED coverage limited to the auth path (F3), and two under-asserted edges (F4 oversize negative-only, F5 burst weak lower bound). None are blocking; the required fixes are labeling/traceability corrections plus optional test-strengthening. No source was modified by this reviewer.
