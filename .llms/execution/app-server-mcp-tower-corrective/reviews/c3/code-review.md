# C3-C Independent Code Review (GLM `glm-5.2`)

| Field | Value |
|---|---|
| Wave | C3-B (Real WebSocket listener — items 20/21/24) |
| Review mode | `implementation` (read-only) |
| Reviewer | GLM `glm-5.2` |
| Date | 2026-07-18 |
| Handoff | `HANDOFF-C3-C-code-review.md` |
| Implementer handoff | `waves/c3-ws-listener.md` (C3-B wave note) + `HANDOFF-C3-B-ws-listener.md` |
| Implementer RESULT | REAL for items 20/21/24; PARTIAL for real-adapter slow-client resync (deferred C3-22/23) |
| Branch | `goblin-implement-epic-tree` |
| Changed surface | `transport/ws_listener.rs` (new), `transport/mod.rs`, `lib.rs`, `Cargo.toml` |

## Verdicts

- **IMPLEMENTATION_OR_ARTIFACT: PASS_WITH_FINDINGS**
- **AGENT_BEHAVIOR: PASS**
- **HANDOFF_QUALITY: PASS_WITH_FINDINGS**
- **GOAL_GATE: N/A** (wave-level implementation review; final-goal gate not in scope)

The listener is genuinely REAL (not helper-only): it binds a real `TcpListener`,
performs a real `accept_hdr_async_with_config` WebSocket upgrade, drives a real
frame loop through the shared `FacadeProcessor::handle_line`, and owns a real
per-connection bounded writer. Every handoff acceptance criterion (1–5) is
proven by code evidence plus captured RED→GREEN logs. No Critical/High finding
remains. Six Low findings and two informational notes are recorded; none block
C3-B acceptance. The "16 black-box tests" claim overcounts by one (one of the
16 is a non-network unit test), and two small observability/dead-code issues
exist in the listener — all non-blocking.

## Review packet completeness

- Wave ID / review mode: C3-B / implementation ✓
- Goal/ledger + acceptance: `waves/c3-ws-listener.md`, `HANDOFF-C3-B-ws-listener.md` §Acceptance ✓
- Original child handoff: `HANDOFF-C3-B-ws-listener.md` ✓
- Child RESULT: REAL items 20/21/24, PARTIAL resync ✓
- Changed surface: `ws_listener.rs`, `mod.rs`, `lib.rs`, `Cargo.toml` ✓
- Claimed commands/results: `cargo test -p xai-grok-app-server --features websocket`
  (42 passed), gate `scripts/run-rust-test-gate.sh ws_listener` exit 0 ✓
- Prior findings / fix mapping: none for C3 (C1 reviews PASS_WITH_FINDINGS, unrelated surface) ✓

## Checks actually run by this reviewer

- Static read of the full diff: `ws_listener.rs` (430 lines), `transport/mod.rs`,
  `lib.rs` (re-exports + `ws_listener_blackbox_tests`), `Cargo.toml`, `websocket.rs`,
  `security.rs`, `processor.rs` (handle_line contract).
- Static cross-check of the 16 test names in `lib.rs` against the GREEN log.
- `grep` for `xai-grok-shell` / `xai_grok_shell` in the app-server crate (tower≠shell).
- `grep` for `KEEPALIVE_INTERVAL_SECS` usage (dead-code check).
- Cross-check of RED log failure messages against the auth code path.
- **Skipped:** re-running `cargo test` / the gate script. This read-only review
  harness exposes no shell-execution tool, so I could not re-execute the suite
  fresh. I verified the captured logs (`tests/c3/c3_ws_listener_{RED,GREEN,GREEN_gate}.log`)
  against the current code statically; the test names, counts, and asserted
  behaviors match the source. The GREEN log is internally consistent (16/16
  ws_listener tests, 42/42 package) and the RED log shows the two auth tests
  failing for the documented stub — consistent with the auth code at
  `ws_listener.rs:196-204`.

## Acceptance criteria — proof matrix

| Handoff AC | Evidence | Status |
|---|---|---|
| 1. Real bind/listen/accept/upgrade path (feature-gated) | `ws_listener.rs:148` `TcpListener::bind(&config.bind)`, `:152` `listener.local_addr()`, `:156-166` accept loop spawning `serve_connection` per connection, `:226-237` `accept_hdr_async_with_config` performs the WS upgrade. Gated by `websocket` feature: `transport/mod.rs:6` `#[cfg(feature = "websocket")] pub mod ws_listener;`, `lib.rs:13-18` re-exports behind `#[cfg(feature = "websocket")]`, `Cargo.toml:28` `websocket = ["dep:tokio", "dep:tokio-tungstenite", "dep:futures-util"]`. | PROVEN |
| 2a. Black-box auth fail | `ws_listener.rs:196-204` calls `validate_bearer_header` in the handshake callback; `unauthorized_response` returns HTTP 401 (`:362-368`). Tests `ws_listener_rejects_missing_authorization_header` + `ws_listener_rejects_wrong_bearer` (`lib.rs:376-393`) connect a real `tokio-tungstenite` client and assert rejection. RED log proves they fail when auth is stubbed. | PROVEN |
| 2b. Valid text RPC through processor | `ws_listener.rs:272-283` dispatches `Message::Text` via `dispatch_text` → `validate_ws_text_frame` + `processor.handle_line` (`:325-333`). Test `ws_listener_text_frame_initialize_then_session_start_roundtrip` (`lib.rs:407-427`) runs `initialize` + `session/start` over the wire. | PROVEN |
| 2c. Oversized/batch rejection | Oversize: `ws_listener.rs:227-229` `WebSocketConfig::max_message_size`/`max_frame_size` = `MAX_FRAME_SIZE` (1 MiB); test `ws_listener_rejects_oversize_text_frame` (`lib.rs:478-505`) sends 1.5 MiB and asserts no RPC response. Batch: `validate_ws_text_frame` rejects `[...]` (`websocket.rs:84-89`); test `ws_listener_rejects_jsonrpc_batch` (`lib.rs:468-476`) asserts `-32600`. | PROVEN |
| 2d. Ping/pong / WS keepalive | `ws_listener.rs:298-306` handles `Message::Ping` by enqueuing `Outbound::Flush`; tungstenite auto-queues the pong; writer flushes (`:252-261`). Test `ws_listener_ping_pong_keepalive` (`lib.rs:432-453`) sends a Ping and asserts a Pong arrives. | PROVEN |
| 2e. Disconnect cleanup | `ws_listener.rs:308` `Message::Close(_) => break`, `:318-320` drops `outbound_tx` and awaits writer drain; writer closes the sink (`:263`). Test `ws_listener_disconnect_drains_and_closes` (`lib.rs:508-538`) asserts the stream ends without hang. | PROVEN |
| 3. Bounded writer / backpressure with test | `ws_listener.rs:240-243` per-connection `mpsc::channel::<Outbound>(config.outbound_queue_cap)`; `:279-281` `try_send` + `dropped_events.fetch_add(1, SeqCst)` on full. Deterministic unit test `bounded_writer_drops_when_full` (`:399-428`, cap=2, enqueue 5 → 3 dropped) proves the drop guarantee; `ws_listener_bounded_writer_survives_burst` (`lib.rs:538-557`) exercises it over the wire. | PROVEN |
| 4. Slow-client real-adapter resync | Fake-adapter variant `ws_listener_slow_client_resync_via_replay_fake_adapter` (`lib.rs:559-591`) wires `session/subscribe` over WS. Real-adapter resync honestly deferred to C3-22/23 (canonical session files); documented in wave note §6 R-WS-3 and STATUS.md. | PARTIAL (accepted by handoff) |
| 5. Wave note + evidence | `waves/c3-ws-listener.md` present with file:line evidence; `tests/c3/c3_ws_listener_{RED,GREEN,GREEN_gate}.log` present and consistent with code. | PROVEN |
| Item 24: conformance matches stdio | `ws_conformance_matches_stdio_method_shapes` (`lib.rs:613-665`) compares real-listener `initialize`/`session/start` result shapes against `process_ndjson_batch` stdio path. | PROVEN |
| Binary frame rejection | `ws_listener.rs:286-295` rejects `Message::Binary` with `-32600` error envelope; test `ws_listener_rejects_binary_frame` (`lib.rs:455-466`). | PROVEN |
| Cleartext non-loopback policy | `ws_listener.rs:142-145` emits `bind_warning(host)` = `security::remote_bind_warning_exact` (`:356-358`) for non-loopback; default bind `127.0.0.1:0` (`:97`); TLS not advertised. Tests `ws_listener_cleartext_non_loopback_warns_experimental_unsafe` + `ws_listener_default_config_is_loopback` (`lib.rs:593-614`). | PROVEN |

## Handoff-specific checks (from `HANDOFF-C3-C-code-review.md`)

| # | Check | Result |
|---|---|---|
| 1 | Real bind/accept/upgrade (not helper-only)? | **YES** — `TcpListener::bind` + `accept_hdr_async_with_config`; not a helper wrapper. |
| 2 | Reuses `FacadeProcessor` only? | **YES** — `dispatch_text` (`:325-333`) calls `processor.handle_line`; no second processor, no `xai-grok-shell` import. |
| 3 | Auth constant-time; no token in URL? | **YES** — `validate_bearer_header` (`websocket.rs:16-32`) is constant-time over the full expected length; bind credential guard at `ws_listener.rs:136-140` rejects `@`/`token=` in the bind string; server validates the `Authorization` header (not the client URL). |
| 4 | Cleartext non-loopback experimental/unsafe preserved? | **YES** — `bind_warning` reuses `remote_bind_warning_exact`; default loopback; TLS stays HUMAN (D-SEC.13); no production TLS claim. |
| 5 | Bounded writer real? | **YES** — real `mpsc::channel` + `try_send` + shared `dropped_events` counter; reader/writer are separate tasks. |
| 6 | Tower ≠ Shell? | **YES** — `Cargo.toml` deps are `xai-grok-app-server-protocol` + `xai-grok-tower` only; grep found no `xai-grok-shell` import (only a negative test assertion at `processor.rs:476`). Composition invariant `composition_processor_depends_on_facade_trait_not_shell` passes (GREEN gate log). |
| 7 | Security footguns? | No Critical/High. Minor: dropped error envelopes are not counted (F-3); `KEEPALIVE_INTERVAL_SECS` is dead code implying an active keepalive that does not exist (F-4); dropped RPC responses are silently lost with no client signal (F-5). None weaken auth, redaction, or the cleartext gate. |

## Findings

### F-1 [Low][Confirmed] — "16 black-box tests" overcounts (one is a non-black-box unit test)

The handoff/STATUS/CHANGES claim "16 black-box tests". The `ws_listener_blackbox_tests`
module (`lib.rs:259-665`) contains **15** tests; the 16th is
`bounded_writer_drops_when_full` in `ws_listener_unit_tests` (`lib.rs:399-428`),
a pure `mpsc` channel test with no network/listener — it is a *unit* test, not
black-box. The GREEN log confirms the split: the unit test is listed under
`transport::ws_listener::ws_listener_unit_tests::`, the other 15 under
`ws_listener_blackbox_tests::`.

Evidence: `lib.rs:390-429` (`mod ws_listener_unit_tests`); GREEN log lines 18-19.
Severity Low: the overclaim is cosmetic; the unit test genuinely proves the
drop guarantee the listener relies on. Fix: restate as "15 black-box + 1
deterministic unit test" in STATUS/wave note.

### F-2 [Low][Confirmed] — IPv6 loopback host parsing mis-classifies `[::1]` as non-loopback

`ws_listener.rs:142`:
```rust
let host = config.bind.split(':').next().unwrap_or("127.0.0.1");
```
For an IPv6 loopback bind like `[::1]:0`, `split(':')` yields `"["`, which is
non-loopback, so `bind_warning` would emit a spurious
`experimental/unsafe` warning and the bind would be mislabeled. Only IPv4
`host:port` and bare `localhost` are handled correctly. The default bind is
IPv4 loopback, so this is not exercised today, but it is a latent footgun for
any operator binding `[::1]`.

Evidence: `ws_listener.rs:141-145`; `security.rs:23` (`remote_bind_label` only
matches `127.0.0.1`/`::1`/`localhost` — `"["` is non-loopback).
Severity Low: no default impact; IPv6 loopback is a reasonable operator
choice. Fix: parse the host with `SocketAddr`/`ToSocketAddrs` or strip a
leading `[...]` before splitting on `:`.

### F-3 [Low][Confirmed] — `dropped_events` not incremented for dropped error envelopes

The module doc (`ws_listener.rs:26-39`) states "the response is dropped and the
per-listener `dropped_events` counter is incremented". The success path honors
this (`:279-281`), but the error-envelope paths use `let _ = outbound_tx.try_send(...)`
without counting:
- binary frame rejection (`:286-295`, line 294)
- processor error envelope (`:283-285`, line 285)

So under backpressure, dropped *error* responses are silently lost and not
observable in `dropped_events`, contradicting the documented contract.

Evidence: `ws_listener.rs:285` and `:294` vs `:279-281`.
Severity Low: error envelopes are rare and the channel is rarely full at that
instant, but the observability contract is inconsistent. Fix: increment
`dropped_events` on those two `try_send` failures too, or weaken the doc to
"successful responses".

### F-4 [Low][Confirmed] — `KEEPALIVE_INTERVAL_SECS` is dead code; implies an active keepalive that does not exist

`ws_listener.rs:70` declares and `lib.rs:17` re-exports
`KEEPALIVE_INTERVAL_SECS = 15`, but it is never used anywhere (grep confirmed:
only the declaration and the re-export). The server never schedules outbound
`Ping`s; it only auto-responds to client `Ping`s via tungstenite. The constant
and its doc (`:67-69`) imply an active keepalive timer that is not implemented,
which is misleading for consumers who might rely on it.

Evidence: `ws_listener.rs:67-70`; grep `KEEPALIVE_INTERVAL_SECS` → only
`ws_listener.rs:70` and `lib.rs:17`.
Severity Low: no behavior impact, but a misleading public API. Fix: either
implement an active ping interval (e.g., a `tokio::time::interval` driving
`Outbound::Flush`/`Ping`) or remove the constant and document that keepalive
is purely reactive (client-initiated ping → auto-pong).

### F-5 [Low][Possible] — Dropped RPC responses are silently lost; client receives no signal

When the bounded writer is full, a successful response is dropped and counted
(`:279-281`), but the connection is **not** closed and no error is sent to the
client. A JSON-RPC client waiting on that `id` will hang indefinitely (until
its own timeout). This is the documented backpressure tradeoff (wave note §4),
but it is a real RPC-semantics hazard: the protocol loses a response without
telling the peer. The `ws_listener_bounded_writer_survives_burst` test only
asserts `responses >= 1`, not that no request is silently dropped.

Evidence: `ws_listener.rs:278-282`; test `lib.rs:538-557`.
Severity Low/Possible: acceptable for the MVP bounded-writer scope, but it
should be explicitly called out in the public doc as "dropped requests are not
NACKed". Residual risk for any client that does not set its own per-request
timeout.

### F-6 [Low][Confirmed] — `Message::Frame(_)` arm terminates the connection despite comment saying "ignore"

`ws_listener.rs:311-312`:
```rust
// Raw frames are not surfaced to users by tungstenite's read path,
// but the enum is non-exhaustive — ignore any other data variant.
Some(Ok(Message::Frame(_))) => break,
```
The comment says "ignore" but the arm `break`s, terminating the connection.
Tungstenite does not surface `Message::Frame` on the read path, so this is
effectively dead today, but the code contradicts the comment and would close
a connection on any future non-data variant.

Evidence: `ws_listener.rs:308-313`.
Severity Low: dead path today. Fix: change to `continue` to match the comment,
or delete the arm and rely on the catch-all.

### Informational notes (non-blocking, no fix required)

- **R-WS-2 (subprotocol inference):** honestly documented as inference with no
  spec evidence; reuses `PROTOCOL_VERSION`. Acceptable for an experimental
  transport.
- **Real-adapter resync PARTIAL:** honestly deferred to C3-22/23 with a
  fake-adapter variant landed now. Accepted by the handoff (AC 4 explicitly
  permits PARTIAL). Not a finding.

## Required fixes

None blocking. Recommended (non-blocking) follow-ups for a future wave or
`@implementation-loop` pass:

1. F-1: Correct the "16 black-box" wording to "15 black-box + 1 unit".
2. F-2: Parse bind host robustly (IPv6 `[::1]:port`).
3. F-3: Increment `dropped_events` on the two error-envelope `try_send` drops
   (or weaken the doc).
4. F-4: Remove or implement `KEEPALIVE_INTERVAL_SECS`.
5. F-5: Document that dropped responses are not NACKed to the client.
6. F-6: Align the `Message::Frame` arm with its comment (`continue` or delete).

## Residual risk

- Dropped-response silence (F-5) is the only behavioral residual; clients must
  set their own per-request timeouts. Documented as a design choice.
- Real-adapter slow-client resync remains PARTIAL (deferred C3-22/23); the
  fake-adapter variant does not exercise canonical session-file replay.
- TLS remains a HUMAN gate (D-SEC.13); this module correctly does not resolve
  it and never advertises production TLS.
- No composition-root wiring of `run_ws_listener` into `app_server_composition.rs`
  / `pager-bin Command::Serve` yet (out of scope for C3-B; flagged in wave note
  §7). The listener is currently only exercised by tests.

## Commands / results (as captured by the implementer; not re-run by this reviewer)

- `cargo test -p xai-grok-app-server --features websocket` → 42 passed; 0
  failed (`tests/c3/c3_ws_listener_GREEN_gate.log`).
- `cargo test -p xai-grok-app-server` (default, no `websocket`) → 26 passed; 0
  failed (per wave note; not re-verified here).
- `bash scripts/run-rust-test-gate.sh ws_listener cargo test -p xai-grok-app-server --features websocket` → exit 0; gate fragment `ws_listener` matched.
- RED: `tests/c3/c3_ws_listener_RED.log` — 2 auth tests FAIL with stubbed
  handshake (`if require_auth && false`), proving the black-box tests catch
  missing auth.
- **Skipped by reviewer:** fresh re-execution of the above. No shell tool in
  this read-only harness. Static cross-check of logs vs. source is consistent.

## Verification checklist

- [x] Base/head, specs, and full changed surface identified.
- [x] Every acceptance criterion has explicit status and evidence.
- [x] Findings cite file/line evidence and include a concrete fix.
- [~] Required checks ran: static review + log cross-check done; fresh
      `cargo test` re-run skipped (no shell tool) — reported honestly.
- [x] Verdict follows the stated gate; no source files were modified.
