# Wave C3-G — Wire WS listener into product composition (GLM build)

| Field | Value |
|---|---|
| Handoff | `HANDOFF-C3-G-ws-composition.md` |
| Agent | **build** (GLM `glm-5.2`) |
| Branch | `goblin-implement-epic-tree` |
| Wave | C3 item 22 (composition wiring of `run_ws_listener`) |
| Verdict | **REAL** for composition wiring (bind + auth + `handle_line` over the real `ShellSessionActorRuntime`). **PARTIAL** for TLS (HUMAN gate D-SEC.13, deferred by contract). |

All evidence is `file:line` against the working tree on `goblin-implement-epic-tree`.
C3-G is a **build** wave (product code changed, owned paths only).

---

## 1. What landed

The experimental App Server WebSocket listener (`run_ws_listener`, landed in
C3-B) is now wired into the `xai-grok-pager-bin` product composition root over
the **real** `ShellSessionActorRuntime` (not `FakeRuntime`). The path is
feature-gated and env-gated so the default build and the default `agent serve`
behavior are unchanged.

New/changed files (all under `xai-grok-pager-bin`, owned by C3-G):

| File | Change |
|---|---|
| `crates/codegen/xai-grok-pager-bin/Cargo.toml` | Added optional `tokio-tungstenite` + `futures-util` deps and a new `app-server-ws` feature that enables `xai-grok-app-server/websocket` (the listener) plus the WS test client. The default feature set does **not** enable it; the stdio/in-process product path stays zero-network on the app-server side. |
| `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` | Added `APP_SERVER_SERVE_ENV`, `app_server_serve_env_enabled()`, `app_server_ws_listener_config()`, `run_app_server_ws()`, `run_app_server_ws_with_root()` (all `#[cfg(feature = "app-server-ws")]` except the env gate, which is always available). New `app_server_ws_composition_tests` module (3 tests, feature-gated). |
| `crates/codegen/xai-grok-pager-bin/src/main.rs` | Env-gated dispatch in the `Some(AgentCmd::Serve(a))` arm: when `GROK_OSS_APP_SERVER=1` and the `app-server-ws` feature is on, starts `run_app_server_ws` instead of the shell agent server. Added `print_app_server_ws_startup_info()` and `AppServerWsGuard` (RAII abort of the accept loop on drop). Without the feature, the env gate is a no-op and the shell agent server path runs unchanged. |

Not touched (ownership respected): `xai-grok-shell/**` (C1/C3-F owns the shell
runtime + projection), `xai-grok-app-server/src/transport/ws_listener.rs`
(C3-B owns the listener), `xai-grok-mcp-server/**` (C4 owns MCP HTTP),
`xai-grok-multi-auth/**` (C5), `xai-grok-tower/**` (facade-only by contract).

---

## 2. Acceptance matrix (handoff §Acceptance)

| Required | Status | Evidence |
|---|---|---|
| 1. Documented CLI/env path starts real listener on 127.0.0.1 | **REAL** | `app_server_composition.rs:run_app_server_ws` builds the real `experimental_app_server_processor()` (→ `ShellSessionActorRuntime::new(grok_home())`) and calls `xai_grok_app_server::run_ws_listener`. CLI: `grok agent serve --bind 127.0.0.1:0 --secret <token>` with `GROK_OSS_APP_SERVER=1` (`main.rs` Serve arm). `app_server_ws_composition_bind_auth_and_handle_line_roundtrip` asserts `handle.addr.ip() == 127.0.0.1`. |
| 2. Black-box/composition test proves bind + auth + handle_line | **REAL** | `app_server_ws_composition_bind_auth_and_handle_line_roundtrip` spawns `run_app_server_ws_with_root(TempDir, "127.0.0.1:0", TOKEN)`, connects a real `tokio-tungstenite` client, asserts (a) bind is loopback, (b) wrong bearer is rejected at the handshake, (c) `initialize` returns `PROTOCOL_VERSION`, (d) `session/start` returns a `sessionId` — all over the real `ShellSessionActorRuntime` (not FakeRuntime). RED→GREEN proof below. |
| 3. Honest PARTIAL for TLS HUMAN | **PARTIAL (by contract)** | `print_app_server_ws_startup_info` prints `"TLS: not provided (HUMAN gate D-SEC.13 — cleartext only)"`. The listener (`run_ws_listener`, C3-B) emits `remote_bind_warning_exact` for non-loopback cleartext. TLS itself stays a HUMAN gate (AS104-HUMAN / D-SEC.13); this wave never advertises production TLS and never auto-promotes a cleartext remote bind. |

Feature gating (handoff "Feature-gate if needed"):

| Concern | Status | Evidence |
|---|---|---|
| `app-server-ws` feature enables `websocket` on the app-server dep | **REAL** | `Cargo.toml`: `app-server-ws = ["xai-grok-app-server/websocket", "dep:tokio-tungstenite", "dep:futures-util"]`. |
| Default build unchanged (zero-network app-server side) | **REAL** | `cargo check -p xai-grok-pager-bin` (no feature) → OK; `cargo test -p xai-grok-pager-bin` (no feature) → 39 passed, 1 pre-existing failure (`is_managed_install_*`, unrelated — see §5); the WS composition tests are `#[cfg(all(test, feature = "app-server-ws"))]` and are filtered out without the feature. |
| Default `agent serve` behavior unchanged | **REAL** | The env gate `app_server_serve_env_enabled()` is falsy when `GROK_OSS_APP_SERVER` is unset/empty/falsy; the dispatch falls through to the existing `xai_grok_shell::agent::run_agent_server` path. `app_server_serve_env_gate_default_is_off` proves the gate. |

---

## 3. RED → GREEN evidence

- **RED**: `tests/c3/c3_ws_composition_RED.log` — with `require_auth` stubbed to
  `false` in `app_server_ws_listener_config`,
  `app_server_ws_composition_bind_auth_and_handle_line_roundtrip` FAILS at the
  `wrong bearer must be rejected at the handshake` assertion (the wrong-token
  upgrade succeeds instead of rejecting). This proves the composition test
  catches missing auth, not just a happy path.
- **GREEN**: `tests/c3/c3_ws_composition_GREEN.log` — with the real
  `require_auth: true`, all 3 `app_server_ws_composition_tests` pass
  (3 passed; 0 failed).
- **GREEN gate**: `tests/c3/c3_ws_composition_GREEN_gate.log` —
  `scripts/run-rust-test-gate.sh app_server_ws cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws`
  exits 0 (gate fragment `app_server_ws` matched).

Full-suite regression:
- `cargo test -p xai-grok-pager-bin --features app-server-ws` → 42 passed; 1
  failed (`is_managed_install_matches_only_the_bin_grok_target` — pre-existing,
  see §5). The 3 new composition tests pass.
- `cargo test -p xai-grok-pager-bin` (default, no `app-server-ws`) → 39 passed;
  1 failed (same pre-existing). WS composition tests filtered out.
- `cargo test -p xai-grok-app-server --features websocket` → 42 passed; 0
  failed (C3-B listener unaffected).
- `cargo check -p xai-grok-pager-bin --features app-server-ws` → OK.
- `cargo clippy -p xai-grok-pager-bin --features app-server-ws --all-targets`
  → no new warnings in C3-G code (pre-existing warnings in other crates left
  untouched — out of scope).

---

## 4. Design decisions (inferences, documented)

| ID | Decision | Rationale |
|---|---|---|
| R-C3G-1 | Env-gated path on the existing `agent serve` (`GROK_OSS_APP_SERVER=1`) rather than a new CLI subcommand | The handoff scopes ownership to `xai-grok-pager-bin/**` and explicitly permits a "CLI/env path". Reusing the existing `ServeArgs` (`--bind`, `--secret`) keeps the change entirely inside pager-bin (no `xai-grok-pager` library CLI edit), respects concurrent work, and the env gate makes the experimental swap explicit (no surprise: default is the shell agent server). |
| R-C3G-2 | New `app-server-ws` cargo feature (not reusing `remote-control`) | pager-bin did not previously depend on the app-server `websocket` feature. A dedicated feature makes the product opt-in explicit and keeps the default build zero-network on the app-server side. The feature forwards to `xai-grok-app-server/websocket` and pulls the WS test client. |
| R-C3G-3 | `run_app_server_ws` wraps `experimental_app_server_processor()` (real `ShellSessionActorRuntime`), not FakeRuntime | The product path must inject the real port (C1-D invariant). The composition test uses `run_app_server_ws_with_root(TempDir)` so it never touches `grok_home()`. |
| R-C3G-4 | RAII `AppServerWsGuard` aborts the accept loop on drop | The listener's accept loop runs on a `JoinHandle`; the guard ensures the process can exit cleanly after a signal without relying on the runtime dropping the task. |
| R-C3G-5 | `app_server_serve_env_enabled()` is always compiled (not feature-gated) | The env gate is a pure stdlib env-var read; keeping it always available means the dispatch decision is uniform and the gate test runs in the default suite. The actual listener call is `#[cfg(feature = "app-server-ws")]`. |

---

## 5. Security posture (HUMAN gate preserved)

- Default bind is loopback (`--bind` defaults to `127.0.0.1:2419`; the
  composition test uses `127.0.0.1:0` for an ephemeral loopback port).
  `app_server_ws_composition_bind_auth_and_handle_line_roundtrip` asserts the
  bound address is `127.0.0.1`.
- Non-loopback cleartext remains `experimental/unsafe`: the listener
  (`run_ws_listener`, C3-B) emits `remote_bind_warning_exact` at bind time, and
  `print_app_server_ws_startup_info` prints the warning for non-loopback hosts.
- TLS is a **HUMAN** gate (AS104-HUMAN / D-SEC.13). This wave never advertises
  production TLS and never auto-promotes a cleartext remote bind. The startup
  banner explicitly states `"TLS: not provided (HUMAN gate D-SEC.13 —
  cleartext only)"`.
- Bearer auth is required on the product path (`require_auth: true` in
  `app_server_ws_listener_config`); `app_server_ws_config_requires_auth_by_default`
  guards this invariant. The token is the per-invocation `--secret` /
  `GROK_AGENT_SECRET` already used by `agent serve`.
- Responses route through the shared `FacadeProcessor`, which redacts via
  `xai-grok-tower::projection`; `SECRET_CANARIES` / `assert_no_secret_canaries`
  remain the canonical guard and were not weakened.

---

## 6. Risks / blockers

| ID | Risk | Status |
|---|---|---|
| R-C3G-6 | `is_managed_install_matches_only_the_bin_grok_target` fails (1 test) | **Pre-existing, unrelated.** The test hardcodes `home/bin/grok` but `xai_grok_config::paths::PRODUCT_BIN_NAME` is now `"grok-oss"` (the grok-oss identity cutover in progress, per `AGENTS.md`). The test exists unchanged in the staged tree (pre-C3-G). C3-G does not touch `is_managed_install` or its test. Tracked by the identity-cutover workstream, not C3-G. |
| R-C3G-7 | Composition test mutates `GROK_OSS_APP_SERVER` env | `app_server_serve_env_gate_default_is_off` saves/restores the prior value. It is the only test in the module reading that var, so no `#[serial]` is needed within this module. Other env-mutating tests in the file use different vars (`GROK_OSS_TOWER` / `GROK_TOWER_INSTANCE`). |
| R-C3G-8 | Real-adapter slow-client resync over WS | **Deferred** to C3-22/23 (canonical session files), per C3-B. The composition test exercises `initialize` + `session/start` over the real adapter; resync is out of scope for C3-G. |

No blocker prevents C3-G from being marked REAL for composition wiring.

---

## 7. Remaining (outside this wave's bound)

- MCP HTTP product wiring (C4-F owns it; C3-G explicitly does not wire MCP HTTP).
- Real-adapter slow-client resync over WS (C3-22/23, canonical session files).
- TLS termination (HUMAN gate D-SEC.13 — never auto-resolved).
- A dedicated CLI subcommand for the app-server (e.g. `grok agent app-server`)
  is a follow-on if the env-gated path is deemed insufficient; the env gate is
  the documented CLI/env path per the handoff acceptance.
- The pre-existing `is_managed_install_matches_only_the_bin_grok_target`
  failure (grok-oss identity cutover) is owned by the identity-cutover
  workstream.
