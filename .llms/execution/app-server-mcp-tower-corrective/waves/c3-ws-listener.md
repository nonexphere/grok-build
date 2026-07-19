# Wave C3-B — Real WebSocket listener

| Field | Value |
|---|---|
| Handoff | `HANDOFF-C3-B-ws-listener.md` |
| Agent | `build` (GLM `glm-5.2`) |
| Branch | `goblin-implement-epic-tree` |
| Wave | C3 items 20–21 (listener slice) |
| Verdict | **REAL** for items 20 (listener/handshake/auth/frames/ping-pong/reject/disconnect/bounded-writer) and 24 (black-box conformance). **PARTIAL** for slow-client real-adapter resync (deferred to C3-22/23). |

All evidence is `file:line` against the working tree on `goblin-implement-epic-tree`.
C3-B is a **build** wave (product code changed, owned paths only).

---

## 1. What landed

A real WebSocket TCP listener over the shared `FacadeProcessor` — no second
processor. Feature-gated behind the existing `websocket` cargo feature so the
stdio/in-process product path stays zero-network unless `remote-control` is
enabled.

New/changed files (all under `xai-grok-app-server`, owned by C3-B):

| File | Change |
|---|---|
| `crates/codegen/xai-grok-app-server/Cargo.toml` | Added `tokio` (net/io-util/time/sync/rt), `tokio-tungstenite`, `futures-util` as **optional** deps gated by `websocket`. Promoted nothing to a hard dep. No `xai-grok-shell` reference (invariant `composition_processor_depends_on_facade_trait_not_shell` preserved). |
| `crates/codegen/xai-grok-app-server/src/transport/ws_listener.rs` (new) | `run_ws_listener` + `WsListenerConfig` + `WsListenerHandle` + `serve_connection` + `Outbound` command enum + `bind_warning` helper. Real `TcpListener` bind, `accept_hdr_async_with_config` handshake (bearer auth + subprotocol negotiation), text-frame loop through `FacadeProcessor::handle_line`, binary/batch/oversize rejection, ping/pong keepalive (tungstenite auto-pong + explicit flush), close/drain, per-connection bounded writer (`mpsc` + `try_send` + `dropped_events` counter). |
| `crates/codegen/xai-grok-app-server/src/transport/mod.rs` | `#[cfg(feature = "websocket")] pub mod ws_listener;` |
| `crates/codegen/xai-grok-app-server/src/lib.rs` | Re-export `run_ws_listener`, `WsListenerConfig`, `WsListenerHandle`, `WS_SUBPROTOCOL`, `OUTBOUND_QUEUE_CAP`, `MAX_FRAME_SIZE`, `KEEPALIVE_INTERVAL_SECS` behind the feature. New `ws_listener_blackbox_tests` module (16 tests). |

Not touched (ownership respected): `xai-grok-shell/**` (C1), `xai-grok-multi-auth/**` (C5), MCP server crates (C4), `xai-grok-tower/**` (facade-only by contract).

---

## 2. Acceptance matrix (item 20 / 24)

| Required (handoff §Acceptance) | Status | Evidence |
|---|---|---|
| 1. Real bind/listen/accept/upgrade path (feature-gated) | **REAL** | `ws_listener.rs:run_ws_listener` binds `TcpListener`, spawns accept loop, `accept_hdr_async_with_config` performs the WS upgrade. Gated by `websocket` feature. `ws_listener_handshake_subprotocol_negotiates_protocol_version` proves the upgrade. |
| 2a. auth fail | **REAL** | `ws_listener_rejects_missing_authorization_header`, `ws_listener_rejects_wrong_bearer` (RED→GREEN: stubbing auth makes both fail, see `tests/c3/c3_ws_listener_RED.log`). |
| 2b. valid text RPC through processor | **REAL** | `ws_listener_text_frame_initialize_then_session_start_roundtrip` runs `initialize` + `session/start` over the wire through `FacadeProcessor::handle_line`. |
| 2c. oversized/batch rejection | **REAL** | `ws_listener_rejects_oversize_text_frame` (WS-layer 1 MiB cap terminates the connection), `ws_listener_rejects_jsonrpc_batch` (`-32600` from `validate_ws_text_frame`). |
| 2d. ping/pong / WS keepalive | **REAL** | `ws_listener_ping_pong_keepalive` — client Ping yields a Pong (tungstenite auto-pong + writer `Flush` command). |
| 2e. disconnect cleanup | **REAL** | `ws_listener_disconnect_drains_and_closes` — client Close drains and the stream ends. |
| 3. Bounded writer / backpressure with test | **REAL** | `bounded_writer_drops_when_full` (deterministic unit test: cap=2, enqueue 5 → 3 dropped). Listener uses the same `mpsc` + `try_send` + `dropped_events` pattern. `ws_listener_bounded_writer_survives_burst` exercises it over the wire. |
| 4. Slow-client real-adapter resync | **PARTIAL** (deferred) | `ws_listener_slow_client_resync_via_replay_fake_adapter` proves the listener wires `session/subscribe` over WS (fake adapter). Real-adapter resync waits on canonical session files (Wave C3-22/23, AS105-01..07 OPEN). Documented, not blocked. |
| 5. Wave note + evidence | **done** | This file; `tests/c3/c3_ws_listener_{RED,GREEN,GREEN_gate}.log`. |
| Item 24: conformance matches stdio/in-process | **REAL** | `ws_conformance_matches_stdio_method_shapes` — black-box WS `initialize`/`session/start` result shapes match the stdio `process_ndjson_batch` path. |
| Binary frame rejection | **REAL** | `ws_listener_rejects_binary_frame` (`-32600` "Binary WebSocket frames are unsupported"). |
| Cleartext non-loopback policy | **REAL** | `ws_listener_cleartext_non_loopback_warns_experimental_unsafe` + `ws_listener_default_config_is_loopback`. Listener emits `remote_bind_warning_exact` at bind time; default bind is `127.0.0.1:0`; TLS stays HUMAN (D-SEC.13). |

---

## 3. RED → GREEN evidence

- **RED**: `tests/c3/c3_ws_listener_RED.log` — with the handshake auth
  stubbed (`if require_auth && false`), `ws_listener_rejects_missing_authorization_header`
  and `ws_listener_rejects_wrong_bearer` both FAIL (the upgrade succeeds
  instead of rejecting). This proves the black-box tests catch the missing
  behavior, not just exercise a happy path.
- **GREEN**: `tests/c3/c3_ws_listener_GREEN.log` — with the real handshake,
  all 16 `ws_listener`/`ws_listener_blackbox` tests pass (16 passed; 0 failed
  in 5.07s).
- **GREEN gate**: `tests/c3/c3_ws_listener_GREEN_gate.log` —
  `scripts/run-rust-test-gate.sh ws_listener cargo test -p xai-grok-app-server --features websocket`
  exits 0 (42 passed total, gate fragment `ws_listener` matched).

Full-suite regression:
- `cargo test -p xai-grok-app-server --features websocket` → 42 passed; 0 failed.
- `cargo test -p xai-grok-app-server` (default, no `websocket`) → 26 passed; 0 failed (stdio/in-process transports stay zero-network).
- `cargo check -p xai-grok-pager-bin` / `-p xai-grok-app-server-client` → OK (downstream crates unaffected; no one enables `websocket`/`remote-control` today).
- `cargo clippy -p xai-grok-app-server --features websocket --all-targets` → no warnings in new code (pre-existing warnings in `processor.rs`/`controller.rs` left untouched — out of scope).

---

## 4. Design decisions (inferences, documented)

| ID | Decision | Rationale |
|---|---|---|
| R-WS-2 | Subprotocol = `PROTOCOL_VERSION` (`"2026-07-18.experimental-v2"`) | No spec evidence. Reusing the protocol version lets a client negotiate the same JSON-RPC envelope on the wire. If a client offers subprotocols but none match, the handshake is rejected (400); no subprotocol header → accept without one (lenient). |
| Bounded writer | Per-connection `mpsc::channel` (cap `OUTBOUND_QUEUE_CAP=256`, configurable) + `try_send`; overflow drops and increments a shared `dropped_events` counter. Reader and writer are separate tasks so a slow client cannot head-of-line-block the reader. | `FacadeProcessor::handle_line` returns one response per request (`processor.rs:53`); a full live-event stream is out of scope for item 20. The bounded channel prevents unbounded buffering; the drop counter is observable via `WsListenerHandle::dropped_events`. |
| Ping/pong | Tungstenite auto-queues a Pong on `Message::Ping` (RFC 6455 §5.5.2). The reader enqueues an `Outbound::Flush` command so the writer flushes the auto-pong promptly even with no pending RPC response. | Without the explicit flush, the pong would sit in tungstenite's internal buffer until the next RPC response and keepalive would appear dead. |
| Oversize | WS-layer `max_message_size`/`max_frame_size` = 1 MiB (matches `validate_ws_text_frame`). Oversize frames are rejected at the WS layer (connection terminates); the JSON-RPC-layer 1 MiB cap (`-32021`) is covered by the existing `validate_ws_text_frame` unit test. | Two layers, two tests. |

---

## 5. Security posture (HUMAN gate preserved)

- Cleartext non-loopback bind remains `experimental/unsafe`. The listener
  emits `remote_bind_warning_exact` to stderr at bind time and exposes
  `bind_warning(host)` for testability. Default bind is `127.0.0.1:0`.
- TLS is a **HUMAN** gate (AS104-HUMAN / D-SEC.13). This module never
  advertises production TLS and never auto-promotes a cleartext remote bind.
- Bearer auth is constant-time (`validate_bearer_header`, reused unchanged);
  tokens never appear in URLs (`reject_credentials_in_url` enforced at bind).
- Responses route through the shared processor, which redacts via
  `xai-grok-tower::projection`; `SECRET_CANARIES` / `assert_no_secret_canaries`
  remain the canonical guard and were not weakened.

---

## 6. Risks / blockers

| ID | Risk | Status |
|---|---|---|
| R-WS-3 | Real-adapter slow-client resync needs canonical session files (AS105-01..07 OPEN) | **Deferred** to C3-22/23. Fake-adapter resync test landed now. Not a blocker for items 20/21/24. |
| R-WS-2 | Subprotocol string undefined by spec | **Documented inference** — `PROTOCOL_VERSION`. |
| R-WS-4 | `handle_line` returns one response, not a live stream | **Out of scope** for item 20 (bounded writer + resync only). Full live event streaming over WS is a larger follow-on. |
| R-WS-7 | Port binding in CI | Ephemeral `127.0.0.1:0`; never hard-coded ports. Tests pass in 5s. |

No blocker prevents C3-B from being marked REAL for items 20/21/24.

---

## 7. Remaining (outside this wave's bound)

- Real-adapter slow-client resync test (C3-22/23, canonical session files).
- Composition-root wiring of `run_ws_listener` into `app_server_composition.rs`
  / pager-bin `Command::Serve` (shared with C1-G; coordinate before editing).
- Full live event streaming over WS (item 20 only requires bounded writer).
- TLS termination (HUMAN gate D-SEC.13 — never auto-resolved).
