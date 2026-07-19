# C3-A — WebSocket transport surface map

| Field | Value |
|---|---|
| Handoff | `HANDOFF-C3-A-ws-surface-map.md` |
| Agent | `repo-explore` (read-only, GLM `glm-5.2`) |
| Branch | `goblin-implement-epic-tree` |
| Wave | C3 prep (items 20–21 inputs) |
| Verdict | **GO** for C3-B listener slice after C1-G; resync test deferred to C3-22/23 |

All evidence is `file:line` against the working tree on `goblin-implement-epic-tree`.
No product code was edited. C3 is **not** marked PASS.

---

## 1. Current state of `websocket.rs`

`crates/codegen/xai-grok-app-server/src/transport/websocket.rs` is a **helper
module, not a listener**. There is no `TcpListener`, no HTTP upgrade, no
handshake, no subprotocol negotiation, no frame loop, no ping/pong, no
disconnect/drain, and no per-connection bounded writer.

What actually exists (all in `websocket.rs`):

| Symbol | Line | Role |
|---|---|---|
| `WebSocketAuth { bearer_token }` | 9 | Struct only; never constructed by any caller. |
| `validate_bearer_header(header, expected)` | 16 | Constant-time `Bearer `/`bearer ` prefix check; returns `ProcessorError` code `-32001` on miss/missing. |
| `constant_time_eq(candidate, expected)` | 37 | Private; scans `expected.len()` bytes + folds longer candidate tail so length/prefix mismatches do not short-circuit. |
| `reject_credentials_in_url(url)` | 55 | Rejects `@`, `token=`, `access_token=` in WS URLs; error `-32600`. |
| `handle_ws_text(processor, text)` | 67 | `validate_ws_text_frame` then `processor.handle_line(text)`. This is the only glue to the shared processor. |
| `validate_ws_text_frame(text)` | 73 | 1 MiB cap (`-32021`), JSON-RPC batch rejection (`[...]` → `-32600`). |

Existing tests (`websocket.rs:99`) call `validate_bearer_header`,
`reject_credentials_in_url`, and `handle_ws_text` **directly** — they are not
black-box. `lib.rs:188` `websocket_conformance_tests` also calls
`handle_ws_text` directly and compares shape against `p_stdio.handle_line`.
Requirement matrix rows AS104-01 / AS104-05 / AS104-06
(`waves/c0-requirement-matrix.md:99`) confirm: only the helper exists; no
listener/handshake/ping-pong/cap; conformance is not black-box; network
attacker/slow-client/oversize tests absent.

Cargo evidence the listener is missing:

- `crates/codegen/xai-grok-app-server/Cargo.toml:1` declares a `websocket`
  feature (line 21) and `remote-control = ["websocket"]` (line 22), but the
  feature gates **nothing**. Dependencies are only `async-trait`, `serde`,
  `serde_json`, `xai-grok-app-server-protocol`, `xai-grok-tower`
  (lines 11–15). `tokio` is a **dev-dependency only** (line 18). There is no
  `tokio-tungstenite`, `axum`, `tokio/net`, or `hyper` dependency in this crate.
- A precedent listener exists elsewhere: `xai-grok-shell/src/agent/server.rs`
  (lines 17–30) uses `axum::extract::ws::{Message, WebSocket, WebSocketUpgrade}`,
  `tokio::net::TcpListener`, `futures::{SinkExt, StreamExt}`, with
  `MAX_BUFFER_SIZE = 8 * 1024 * 1024` and `KEEPALIVE_INTERVAL_SECS = 15`
  (lines 49–60). The shell crate pulls `axum` with the `ws` feature and
  `tokio-tungstenite` (`xai-grok-shell/Cargo.toml:88,91`). C3-B can mirror this
  pattern but inside `xai-grok-app-server`.

---

## 2. Shared processor entry points (reused by stdio / in-process / ws)

All three transports converge on one processor. Evidence:

| Entry | Location | Used by |
|---|---|---|
| `FacadeProcessor::new(runtime: Arc<dyn GrokRuntimeFacade>)` | `processor.rs:31` | composition root |
| `FacadeProcessor::handle_line(&str) -> Result<Option<String>, ProcessorError>` | `processor.rs:53` | stdio, ws, in-process (via `InProcessClient`) |
| `FacadeProcessor::handle_value(Value) -> Result<Option<String>, ProcessorError>` | `processor.rs:59` | typed path |
| `FacadeProcessor::dispatch(method, params) -> Result<Value, ProcessorError>` | `processor.rs:109` | internal; runs `classify_pre_init` gate then method match |
| `AppServerProcessor::process(&self, method, params)` trait | `lib.rs:27` (trait), `processor.rs:269` (impl) | typed callers |
| `FacadeProcessor::is_initialized(&self) -> bool` | `processor.rs:46` | gate introspection |

Transport-side glue:

- stdio: `run_stdio_loop(processor, reader, writer, stderr)` (`stdio.rs:12`)
  loops `read_line` → `processor.handle_line` → one JSON object per stdout
  line; EOF drains; parse errors (`-32700`) emit a failure envelope with
  `id: null` (`stdio.rs:46`). `process_ndjson_batch` (`stdio.rs:65`) is the
  test/embedded helper.
- in-process: `InProcessClient::request` (`in_process.rs:31`) builds a JSON-RPC
  envelope and calls `processor.handle_line`; `initialize` helper at
  `in_process.rs:64`.
- ws: `handle_ws_text(processor, text)` (`websocket.rs:67`) →
  `validate_ws_text_frame` + `processor.handle_line`. This is the only WS
  integration point today.

Composition root (real adapter, not FakeRuntime):

- `experimental_app_server_processor()` (`app_server_composition.rs:19`) →
  `experimental_app_server_processor_with_root(root)` (line 28) builds
  `ShellSessionActorRuntime::new(root)` wrapped in `ShellRuntimeAdapter::inject`
  and feeds it to `FacadeProcessor::new`. This is the processor a real WS
  listener must serve.

Initialize gate / protocol version:

- `PROTOCOL_VERSION = "2026-07-18.experimental-v2"`
  (`xai-grok-app-server-protocol/src/lib.rs:26`).
- `classify_pre_init(method, already)` (`processor.rs:113`) gates all methods
  except `initialize`/health before init; errors `-32002` not_initialized /
  already-initialized.

Backpressure surface (partial):

- `FacadeProcessor` holds `outbound_queue_cap` (`processor.rs:36`, set to
  `protocol_defaults::OUTBOUND_QUEUE_EVENTS` line 39) and a `slow_client_events`
  `AtomicU64` (line 37). `session/subscribe` increments `slow_client_events`
  when `page.events.len() > outbound_queue_cap` (`processor.rs:175`). This is a
  counter only — there is **no per-connection bounded queue or writer**.
- `replay::replay_all_pages` (`replay.rs:6`) pages through `replay()` via
  `next_cursor`; used by `snapshot_then_live_tests` (`lib.rs:209`).

---

## 3. Missing pieces for black-box acceptance (item 20)

Item 20 (`tasks/20260718-correct-app-server-mcp-tower-execution.md:70`):
handshake/subprotocol, header auth, text frames, ping/pong, binary/batch/
oversize rejection, disconnect, bounded writer, slow-client resync.

| Required | Status | Evidence / gap |
|---|---|---|
| Real TCP listener (loopback default; non-loopback explicit) | **missing** | No `TcpListener` in app-server crate. `security::remote_bind_label` (`security.rs:22`) and `remote_bind_warning_exact` (`security.rs:54`) exist but are not called by any bind path. |
| WebSocket handshake / HTTP upgrade | **missing** | No upgrade handler. `transport/mod.rs:1` defines `TransportKind::WebSocket` and `ProtocolConnection` trait (line 22) but no WS impl of it. |
| Subprotocol negotiation | **missing** | `PROTOCOL_VERSION` exists (`protocol/lib.rs:26`) but no subprotocol name constant, registry, or `Sec-WebSocket-Protocol` handling anywhere in the crate (grep found none). **Implementer must decide the subprotocol string** — recommend reusing `PROTOCOL_VERSION` and documenting it; no spec evidence either way. |
| Header auth at handshake | **partial** | `validate_bearer_header` (`websocket.rs:16`) is correct and constant-time but is **never wired into a handshake**; no `Authorization` header extraction at upgrade. `reject_credentials_in_url` (`websocket.rs:55`) unused by any listener. |
| Text frame loop → processor | **partial** | `handle_ws_text` (`websocket.rs:67`) maps one text frame to `processor.handle_line`, but no receive loop, no `Message::Text` dispatch, no response `Message::Text` send. |
| Ping/pong + keepalive | **missing** | No `Message::Ping`/`Pong` handling, no timeout. Shell precedent: `agent/server.rs:60` `KEEPALIVE_INTERVAL_SECS = 15`. |
| Binary frame rejection | **missing** | `validate_ws_text_frame` rejects batches + oversize text, but there is no `Message::Binary` rejection at the WS layer. |
| Batch rejection | **exists (helper only)** | `validate_ws_text_frame` rejects `[...]` (`websocket.rs:84`, `-32600`); needs wiring into the frame loop. |
| Oversize rejection | **exists (helper only)** | 1 MiB cap (`websocket.rs:78`, `-32021`); needs wiring + WS frame size limit. |
| Disconnect / drain / close | **missing** | No `Message::Close` handling, no drain, no cleanup. `run_stdio_loop` has an EOF drain (`stdio.rs:29`) as a pattern. |
| Bounded writer (per-connection outbound queue) | **missing** | Only a global `slow_client_events` counter (`processor.rs:37`); no per-connection bounded channel. `ProtocolConnection::send` (`transport/mod.rs:26`) is a trait method with no WS impl. |
| Slow-client resync (replay-to-live) | **blocked on C3-22/23** | `replay_all_pages` (`replay.rs:6`) exists but real replay depends on canonical session files (Wave C3-22/23, rows AS105-01..07 `c0-requirement-matrix.md:100`). Today replay tests use `FakeRuntime` only. A listener-level resync test can be written against the fake adapter; a real-adapter resync test must wait for C3-22/23. |
| `run_ws_listener` / `serve_ws` entry point | **missing** | No function exists. `lib.rs:8` re-exports stdio/in-process but not any WS listener. |

---

## 4. Security

Cleartext non-loopback policy (evidence):

- `security::remote_bind_label(host, cleartext)` (`security.rs:22`):
  `127.0.0.1`/`::1`/`localhost` → `"loopback"`; non-loopback cleartext →
  `"experimental/unsafe-cleartext-remote"`; non-loopback TLS →
  `"remote-tls-required"`.
- `remote_bind_warning_exact(host)` (`security.rs:54`) returns the
  `"WARNING: non-loopback cleartext bind is experimental/unsafe"` string for
  non-loopback only.
- Epic `v1-04-websocket-remote-auth/README.md:36` business rule: loopback bind
  default; non-loopback explicit; `ws://` permitted; **no scopes/Origin/TLS
  enforcement in MVP**, with warning/threat docs. `README.md:58` flags
  `[HIGH][Confirmed] cleartext remote full-control` as an accepted MVP tradeoff.

Auth header path (evidence):

- `validate_bearer_header` (`websocket.rs:16`) accepts `Authorization: Bearer
  <token>` (case-insensitive `bearer ` prefix), constant-time over the full
  expected length; generic `-32001` "Authentication required." on any miss
  (`websocket.rs:19`). `bearer_auth_rejects_prefix_and_length_mismatches_identically`
  (`websocket.rs:115`) proves prefix/length mismatches are indistinguishable.
- `reject_credentials_in_url` (`websocket.rs:55`) keeps tokens out of URLs.

Threat notes / HUMAN gates:

- AS104-HUMAN (`v1-04 tasks.md:9`, `c0-requirement-matrix.md:101`):
  `[D-SEC.13]` TLS/threat acceptance for non-loopback is a **HUMAN,
  manual-verify, blocking** gate. C3-B must **not** auto-promote cleartext
  non-loopback to production-ready; it must remain labeled
  `experimental/unsafe`. TLS itself stays HUMAN.
- `SECRET_CANARIES` + `assert_no_secret_canaries` (`security.rs:6,13`) must
  remain absent from every output sink; the WS listener must route responses
  through the same processor (which already redacts via
  `xai-grok-tower::projection` — `projection.rs:43`).

---

## 5. Suggested RED test names (crate `xai-grok-app-server`)

Black-box: spawn a real listener on an ephemeral loopback port, connect with a
real WS client (`tokio-tungstenite`), assert wire behavior. Use
`experimental_app_server_processor_with_root(TempDir)` so tests never touch
real `grok_home()` (pattern: `app_server_composition.rs:58`).

Item 20 / AS104-01 / AS104-05 / AS104-06:

1. `ws_listener_handshake_subprotocol_negotiates_protocol_version`
2. `ws_listener_rejects_missing_authorization_header` (AS104-02 matrix)
3. `ws_listener_rejects_wrong_bearer_constant_time` (no prefix/length leakage)
4. `ws_listener_rejects_credentials_in_url`
5. `ws_listener_text_frame_initialize_then_session_start_roundtrip`
6. `ws_listener_ping_pong_keepalive`
7. `ws_listener_rejects_binary_frame`
8. `ws_listener_rejects_jsonrpc_batch`
9. `ws_listener_rejects_oversize_text_frame` (>1 MiB)
10. `ws_listener_disconnect_drains_and_closes`
11. `ws_listener_bounded_writer_drops_slow_client_events` (cap = `OUTBOUND_QUEUE_EVENTS`)
12. `ws_listener_slow_client_resync_via_replay` (defer real-adapter variant to C3-22/23; fake-adapter variant can land now)
13. `ws_listener_cleartext_non_loopback_warns_experimental_unsafe` (loopback default; non-loopback explicit + warning, never production)
14. `ws_conformance_matches_stdio_and_in_process_method_shapes` (item 24, `lib.rs:188` already a non-black-box version — replace/augment with the real listener)

Gate script: `scripts/run-rust-test-gate.sh` matches `^test .*<fragment>.* \.\.\. ok$`
(`run-rust-test-gate.sh:20`). Suggested gate fragments:
`websocket_listener`, `bearer_auth`, `remote_bind`, `redaction_canary`,
`control_plane_security`, `websocket_conformance`.

---

## 6. Owned files for C3-B (non-overlapping with C1-G)

C1-G owns `xai-grok-shell/src/app_server_runtime/**` and the `SessionActor`
turn lifecycle wiring (`HANDOFF-C1-G-turn-lifecycle.md:82` explicitly excludes
"WebSocket transport implementation" from C1-G and assigns mapping only to
C3). C3-B must **not** edit those files.

C3-B owns (app-server transport layer only):

| File | Action |
|---|---|
| `crates/codegen/xai-grok-app-server/src/transport/websocket.rs` | Extend with listener / frame loop / ping-pong / close; keep existing auth + `validate_ws_text_frame` helpers. |
| `crates/codegen/xai-grok-app-server/src/transport/ws_listener.rs` (new) | Recommended new module for `run_ws_listener`/`serve_ws` + bounded writer + handshake/subprotocol. |
| `crates/codegen/xai-grok-app-server/src/transport/mod.rs` | Re-export the listener; optionally add a WS `ProtocolConnection` impl. |
| `crates/codegen/xai-grok-app-server/src/lib.rs` | Re-export `run_ws_listener` (mirrors stdio re-export at line 9). |
| `crates/codegen/xai-grok-app-server/Cargo.toml` | Add `tokio-tungstenite` (and `tokio` with `net`/`io-util`/`time` features, or `axum` with `ws`) behind the existing `websocket` feature; promote `tokio` from dev-dep to optional dep gated by `websocket`. Workspace already provides these crates (`xai-grok-shell/Cargo.toml:88,91`, `xai-grok-workspace/Cargo.toml:103`). |
| `crates/codegen/xai-grok-app-server/src/security.rs` | Reuse as-is; possibly add a `bind_host_label` helper used by the listener. |

Shared / coordinate:

- `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` — only if
  C3-B wires a `run_ws_server` entry into the composition root. This file is
  shared with C1-G's composition assertions; edit minimally and coordinate.

Do **not** touch: `xai-grok-shell/src/app_server_runtime/**`,
`xai-grok-shell/src/agent/**`, `xai-grok-tower/**` (tower is facade-only by
contract — `xai-grok-tower/src/lib.rs:4` "never imports Shell").

---

## 7. Risks / blockers

| ID | Risk | Evidence | Safe path |
|---|---|---|---|
| R-WS-1 | New runtime deps in app-server crate | `Cargo.toml:11-18` has no net/ws deps; `tokio` is dev-only | Add `tokio-tungstenite` (+ `tokio` net/io/time) behind the existing `websocket` feature; workspace already vendors them. No new external supply-chain. |
| R-WS-2 | Subprotocol string undefined | `PROTOCOL_VERSION` exists (`protocol/lib.rs:26`) but no `Sec-WebSocket-Protocol` constant/registry found anywhere (grep empty) | Implementer picks the subprotocol value; recommend `PROTOCOL_VERSION` and document it in `websocket.rs`. Flag as inference, not spec. |
| R-WS-3 | Slow-client resync needs real replay | `replay_all_pages` (`replay.rs:6`) is fake-only today; AS105-01..07 (`c0-requirement-matrix.md:100`) are OPEN for canonical session files (Wave C3-22/23) | Land listener-level resync test against fake adapter now; defer real-adapter resync test to after C3-22/23. Do not block item 20/21 on it. |
| R-WS-4 | Processor returns one response, not a stream | `handle_line` returns `Result<Option<String>>` (`processor.rs:53`); `session/subscribe` returns a single replay page, not a live stream (`processor.rs:170`) | For black-box roundtrips this is fine. Live event streaming over WS is a larger scope; item 20 only requires bounded writer + resync, not a full live stream. Confirm scope with orchestrator if needed. |
| R-WS-5 | HUMAN TLS gate must remain HUMAN | AS104-HUMAN (`v1-04 tasks.md:9`); `D-SEC.13` | Cleartext non-loopback stays `experimental/unsafe`; never advertise production. TLS is HUMAN. |
| R-WS-6 | Composition root shared with C1-G | `app_server_composition.rs` is referenced by C1-G composition assertions (`composition_tests`, `composition_root_injects_real_port_not_fake_runtime` line 92) | C3-B edits only add a WS entry function; do not alter the real-port injection. Coordinate if C1-G still in flight. |
| R-WS-7 | Port binding in CI | No existing listener tests in app-server crate | Bind `127.0.0.1:0` (ephemeral); shell precedent at `agent/server.rs`. Never hard-code ports. |

No blocker prevents starting C3-B after C1-G. R-WS-3 scopes only the resync
sub-test; R-WS-2 is a documented inference.

---

## Executive summary (10 lines)

1. `websocket.rs` is a helper, not a listener: auth + frame validation + `handle_ws_text` only; no TCP/upgrade/loop/ping-pong/close.
2. The `websocket` cargo feature exists but gates nothing; app-server crate has no `tokio-tungstenite`/`axum`/`tokio-net` deps (`tokio` is dev-only).
3. All transports converge on `FacadeProcessor::handle_line` (`processor.rs:53`); composition injects real `ShellSessionActorRuntime` (`app_server_composition.rs:28`).
4. Auth is real and constant-time (`validate_bearer_header` `websocket.rs:16`) but is not wired into any handshake.
5. `validate_ws_text_frame` (`websocket.rs:73`) gives 1 MiB cap + batch rejection; binary rejection and WS-layer wiring are missing.
6. Security policy exists (`remote_bind_label` `security.rs:22`, `remote_bind_warning_exact` `security.rs:54`); cleartext non-loopback is `experimental/unsafe`, never production.
7. AS104-HUMAN `[D-SEC.13]` TLS gate stays HUMAN; C3-B must not auto-promote cleartext remote.
8. Bounded writer is missing (only a global `slow_client_events` counter, `processor.rs:37`); slow-client resync depends on C3-22/23 real replay.
9. C3-B owns `xai-grok-app-server/src/transport/{websocket.rs, ws_listener.rs (new), mod.rs, lib.rs}` + `Cargo.toml`; must not touch `xai-grok-shell/**` (C1-G).
10. **GO** for C3-B listener slice (items 20/21) after C1-G; defer real-adapter resync test to C3-22/23.

## GO / NO-GO for C3-B after C1-G

**GO** — with one deferral.

- C3-B may start immediately after C1-G lands (so the composition root serves
  the real turn lifecycle). Items 20 and 21 (listener, handshake/subprotocol,
  header auth, text frames, ping/pong, binary/batch/oversize rejection,
  disconnect, bounded writer) are self-contained in the app-server transport
  layer and do not overlap C1-G's `xai-grok-shell` ownership.
- **Defer** the real-adapter slow-client resync test (R-WS-3) to Wave C3-22/23,
  because canonical session-file replay is not implemented yet
  (`replay_all_pages` is fake-only; AS105-01..07 OPEN). A fake-adapter
  resync test may land with C3-B now.
- C3-B must add `tokio-tungstenite` (+ `tokio` net/io/time) behind the existing
  `websocket` feature; workspace already vendors these crates (no new
  supply-chain). Subprotocol string is an implementer decision (R-WS-2) —
  recommend reusing `PROTOCOL_VERSION` and documenting it.
- C3-B must keep cleartext non-loopback labeled `experimental/unsafe` and must
  not resolve the HUMAN TLS gate (AS104-HUMAN / D-SEC.13).

C3 is **not** PASS; this map is read-only prep only.
