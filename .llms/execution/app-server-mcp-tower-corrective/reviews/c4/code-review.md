# C4-C Independent Code Review (GLM `glm-5.2`)

| Field | Value |
|---|---|
| Wave | C4-B (Real MCP Streamable HTTP server — items 26–28) |
| Review mode | `implementation` (read-only) |
| Reviewer | GLM `glm-5.2` |
| Date | 2026-07-18 |
| Handoff | `HANDOFF-C4-C-code-review.md` |
| Implementer handoff | `HANDOFF-C4-B-mcp-streamable-http.md` |
| Implementer RESULT | GREEN — real axum `/mcp` server, 23 black-box tests + 12 lib tests; composition wiring PARTIAL |
| Branch | `goblin-implement-epic-tree` |
| Changed surface | `transport/http_server.rs` (new), `transport/mod.rs`, `lib.rs`, `Cargo.toml`, `tests/streamable_http.rs` (new) |

## Verdicts

- **IMPLEMENTATION_OR_ARTIFACT: PASS**
- **AGENT_BEHAVIOR: PASS**
- **HANDOFF_QUALITY: PASS**
- **GOAL_GATE: N/A** (wave-level implementation review; final-goal gate not in scope)

The server is genuinely REAL (not helper-only): `run_mcp_http_server` binds a
real `TcpListener` and runs `axum::serve` over a real `Router` with
`POST/GET/DELETE /mcp` + `/healthz`. Every `tools/call` routes through the
shared semantic core `invoke_tower_tool` — there is no second tool
implementation and no local MCP self-loop. All five handoff acceptance
criteria are proven by code evidence plus the captured GREEN log. No
Critical/High finding remains. Eight findings are recorded (one Medium, seven
Low); none block C4-B acceptance. The "23 black-box tests" claim overcounts
by one (one is a static source-inspection `#[test]`, not black-box HTTP), and
the public `McpHttpConfig::default()` is an unsafe-default footgun (empty
bearer + `require_auth: true` silently accepts unauthenticated requests) —
non-blocking today only because the product bin does not yet wire the listener.

## Review packet completeness

- Wave ID / review mode: C4-B / implementation ✓
- Goal/ledger + acceptance: `waves/c4-mcp-streamable-http.md`,
  `HANDOFF-C4-B-mcp-streamable-http.md` §Acceptance ✓
- Original child handoff: `HANDOFF-C4-B-mcp-streamable-http.md` ✓
- Child RESULT: GREEN — real axum server, 23 tests + 12 lib; composition PARTIAL ✓
- Changed surface: `http_server.rs`, `transport/mod.rs`, `lib.rs`, `Cargo.toml`,
  `tests/streamable_http.rs` ✓
- Claimed commands/results: `cargo test -p xai-grok-mcp-server
  --features streamable-http` (23 integration + 12 lib green); default-features
  11/12 green; `cargo test -p xai-grok-tower-tools -p xai-grok-tower` green ✓
- Prior findings / fix mapping: none for C4 (C3 reviews PASS_WITH_FINDINGS on a
  different crate/surface; C4-A map is the RED baseline) ✓

## Checks actually run by this reviewer

- Static read of the full new surface: `http_server.rs` (727 lines),
  `transport/mod.rs`, `lib.rs`, `Cargo.toml`, `tests/streamable_http.rs`
  (751 lines), `transport/http.rs` (helpers consumed by the server).
- Static cross-check of the 23 integration test names + 12 lib tests in the
  GREEN log against the source (`#[tokio::test]` × 22 + `#[test]` × 1).
- `grep` for `xai_grok_mcp` / `xai_grok_shell` / `McpClient` / `register_self`
  in `http_server.rs` (only the negative canary assertions at `:719-727`).
- `grep` for `register_self` / `127.0.0.1:8788/mcp` in
  `app_server_composition.rs` (no matches → composition self-loop guard passes
  honestly by absence).
- Cross-check of `invoke_tower_tool` signature + `tower_agent_start` result
  shape against the HTTP `dispatch_jsonrpc` `tools/call` arm and the test
  assertions (`state == "completed"`, `sessionId` present).
- Cross-check of `validate_http_bearer` empty-token semantics against
  `McpHttpConfig::default()` (F-2).
- Cross-check of `FakeRuntime::start_session` event production against the SSE
  test assertions (`session_changed`, `id: 1`).
- **Skipped:** fresh re-execution of `cargo test`. This read-only review
  harness exposes no shell-execution tool, so I could not re-run the suite.
  I verified the captured log (`tests/c4/c4_streamable_http_GREEN.log`)
  against the current code statically: the 23 integration + 12 lib test names,
  counts, and asserted behaviors match the source, and the log is internally
  consistent (23/23, 12/12, 0 failed). The one compiler warning
  (`unused import: process_mcp_stdio_batch` in `stdio.rs:9`) is pre-existing
  and unrelated to the C4-B surface.

## Acceptance criteria — proof matrix

| Handoff AC | Evidence | Status |
|---|---|---|
| 1. Real HTTP bind serving `/mcp` (feature-gated OK) | `http_server.rs:208` `run_mcp_http_server`, `:246` `TcpListener::bind(&config.bind).await`, `:248` `listener.local_addr()`, `:250-253` `tokio::spawn` + `axum::serve(listener, app)`. Router at `:239-243` registers `POST/GET/DELETE /mcp` + `GET /healthz`. Feature-gated: `Cargo.toml:28` `streamable-http = ["dep:axum","dep:tokio","dep:futures-util"]`; `transport/mod.rs:3` `#[cfg(feature="streamable-http")] pub mod http_server`; `lib.rs:19-21` re-exports behind the feature. | PROVEN |
| 2. Black-box POST initialize/tools/list/tools/call; auth failure; body limit; DELETE session | initialize: `post_initialize_negotiates_session_header` (`tests:104`). tools/list: `post_tools_lists_exactly_nine_descriptors_matching_in_process` (`tests:115`). tools/call: `post_tools_call_start_returns_structured_content_with_session_id` (`tests:142`) + deny `post_tools_call_deny_path_emits_iserror_with_forbidden_code` (`tests:171`). auth: `auth_failures_are_indistinguishable_401` (`tests:204`). body limit: `body_limit_rejects_oversized_post_before_dispatch` (`tests:253`). DELETE: `delete_session_terminates_and_rejects_subsequent_post` (`tests:277`) + `delete_without_session_header_is_bad_request` (`tests:304`). All drive a real `reqwest` client against a real bound listener. | PROVEN (22 black-box HTTP tests; see F-1 for the 23rd) |
| 3. SSE GET resume path wired to a real transport (full real-adapter resync may PARTIAL) | `get_mcp` (`http_server.rs:384-441`) streams `McpSession::events_after` with `Last-Event-ID` resume; foreign id → `resumption_error` (`:416-422`); per-transport-session id space + event log fed by `pull_facade_events` (`:538-561`) polling `GrokRuntimeFacade::replay`. Tests 9–13 (`tests:321,368,409,432,472`) prove framing/resume/foreign-id/isolation. Live push is PARTIAL (snapshot-then-close; see F-5) — accepted by handoff AC 3. | PROVEN (framing); PARTIAL (live push, accepted) |
| 4. Nine-tool descriptor parity with in-process names | `dispatch_jsonrpc` `tools/list` (`http_server.rs:483-489`) emits `TOWER_TOOL_DESCRIPTORS` with `name/description/inputSchema`. `post_tools_lists_exactly_nine_descriptors_matching_in_process` asserts `len == 9` and names == `MCP_TOOL_NAMES`. `stdio_and_http_produce_identical_tools_list_and_error_shapes` (`tests:574`) compares stdio vs HTTP names. | PROVEN |
| 5. Wave note + evidence; honest PARTIAL for composition self-loop if product bin not yet wired | `waves/c4-mcp-streamable-http.md` present with file:line evidence + honest PARTIAL section. `tests/c4/c4_streamable_http_GREEN.log` present and consistent. Composition wiring PARTIAL documented; `app_server_composition.rs` has no `run_mcp_http_server`/`McpHttpConfig` reference (grep confirmed). | PROVEN |

## Handoff-specific checks (from `HANDOFF-C4-C-code-review.md`)

| # | Check | Result |
|---|---|---|
| 1 | Real bind/serve vs helper-only? | **REAL** — `TcpListener::bind` + `axum::serve` in a spawned task; not a helper/pure-function wrapper. The pre-existing `transport/http.rs` helpers (`validate_http_bearer`, `reject_token_query`, `enforce_body_limit`, `SseResumeTable`, `post_mcp_response`) are now *consumed* by the real server (`http_server.rs:59,599,617`), not the surface. `SseResumeTable` remains unused by the server (the server owns its own per-session event log) — see F-8. |
| 2 | Shared `invoke_tower_tool` only? | **YES** — `dispatch_jsonrpc` `tools/call` (`http_server.rs:496-500`) calls `invoke_tower_tool(state.runtime.clone(), &state.agent_type, state.explicit_opt_in, name, args)`. No second tool implementation. `tools/list` reuses `TOWER_TOOL_DESCRIPTORS`. The JSON-RPC *framing* is duplicated (see F-3), but the semantic core is single-sourced. |
| 3 | Auth/body limits/session binding? | **YES** — auth `require_auth` (`:586-595`) via `validate_http_bearer` (constant-time-ish, indistinguishable 401). Body limit `check_body_limit` (`:617-622`) via `enforce_body_limit` before dispatch. Session binding `lookup_session` (`:638-664`): `Mcp-Session-Id` required, bearer fingerprint match, and `tower_instance_id` match. `initialize` opens a session (`:322-329`). |
| 4 | Self-loop guards? | **YES, three levels** — symbol canary `http_server_does_not_import_outbound_mcp_client` (`:718-727`) asserts no `xai_grok_mcp::`/`McpClient`/`register_self` in production source; composition guard `composition_source_does_not_register_local_mcp_self_loop` (`tests:664`); runtime guard `post_tools_call_does_not_reenter_via_managed_mcp_client` (`tests:634`). Pre-existing canaries `no_local_self_injection_in_production_source`/`no_self_mcp_loop_tool_names` retained. |
| 5 | Security (token query, TLS honesty)? | **YES** — query-token rejection `check_query` (`:598-604`) via `reject_token_query` (test `post_rejects_token_in_query_string`). Bind credential guard rejects `@`/`token=` in bind string (`:213-218`). `bind_warning` (`:255-262`) emits canonical `experimental/unsafe` for non-loopback; TLS stays HUMAN (D-SEC.13); no production TLS claim. Loopback is the default (`:188`). |

## Findings

### F-1 [Low][Confirmed] — "23 black-box tests" overcounts (one is a static source guard, not black-box HTTP)

The handoff/STATUS claim "23 black-box tests". `tests/streamable_http.rs`
contains **22** `#[tokio::test]` black-box HTTP tests + **1** `#[test]`
static source-inspection guard: `composition_source_does_not_register_local_mcp_self_loop`
(`tests:664-684`), which does `include_str!` on
`app_server_composition.rs` and asserts string absence — no listener, no HTTP
client. The GREEN log lists all 23 under the integration binary, but the
`#[test]` (line 664) is not black-box.

Evidence: `tests/streamable_http.rs:664` (`#[test]`, not `#[tokio::test]`);
grep counts 22 `#[tokio::test]` + 1 `#[test]`.
Severity Low: the overclaim is cosmetic; the source guard genuinely proves the
composition self-loop tripwire. Fix: restate as "22 black-box HTTP + 1
composition source guard" in STATUS/wave note. (Same shape as C3 F-1.)

### F-2 [Medium][Confirmed] — `McpHttpConfig::default()` is an unsafe default (empty bearer silently authenticates)

`McpHttpConfig::default()` (`http_server.rs:186-193`) sets
`bearer_token: String::new()` and `require_auth: true`. With an empty expected
token, `validate_http_bearer` (`transport/http.rs:12-33`) accepts a request
with no/empty `Authorization` header: `token == ""`, `expected == ""` →
`diff` initializes to 0 (equal lengths), the `expected.bytes()` loop runs zero
iterations, and `token.bytes().skip(0)` yields no bytes, so `diff == 0` → `Ok`.
A consumer that constructs `McpHttpConfig::default()` without setting a
non-empty `bearer_token` gets `require_auth: true` (a false sense of security)
while actually accepting unauthenticated requests.

Evidence: `http_server.rs:186-193`; `transport/http.rs:12-33` (empty/empty → Ok).
Severity Medium: real security-misconfiguration vector in the public API. No
production exposure today only because `app_server_composition.rs` does not
wire the listener (PARTIAL); all tests set a non-empty `TOKEN`, so the gap is
not caught. Fix: either default `require_auth` to `false` when `bearer_token`
is empty, or assert a non-empty `bearer_token` at construction when
`require_auth` is true, or document `default()` as test-only and provide a
`McpHttpConfig::secured(token)` constructor.

### F-3 [Medium][Likely] — Duplicated JSON-RPC framing creates a parity-drift risk

`dispatch_jsonrpc` (`http_server.rs:466-533`) duplicates the
`initialize`/`tools/list`/`tools/call`/error/`method-not-found` JSON-RPC
response shaping already present in `handle_mcp_jsonrpc`
(`lib.rs:71-126`). The semantic core is shared (`invoke_tower_tool`), but the
framing is a second hand-maintained copy. The two copies are byte-identical
today (verified: `initialize` result, `tools/list` descriptor shape,
`tools/call` success `content`/`structuredContent`, error `isError` +
`structuredContent.{code,message}`, and `-32601` method-not-found all match),
but parity is enforced only by `stdio_and_http_produce_identical_tools_list_and_error_shapes`
(`tests:574`), which compares **names** only and exercises the stdio error
shape — not the HTTP error shape, full descriptor shape, or method-not-found.

Evidence: `http_server.rs:476-533` vs `lib.rs:78-125`; `tests:574-627` (names
only).
Severity Medium/Likely: maintainability + parity-drift hazard, not a current
correctness defect. The split is understandable (`dispatch_jsonrpc` needs
session context for `tower_session_id` binding that `handle_mcp_jsonrpc` does
not take), but the framing could be factored into a shared helper that both
transports call, passing session context separately. Fix: extract a shared
`shape_jsonrpc_response`/`shape_tools_list`/`shape_tool_error` module and call
it from both dispatchers; add a parity test that compares full descriptor +
error shapes (not just names).

### F-4 [Low][Confirmed] — IPv6 loopback bind host parsing mis-classifies as non-loopback

`http_server.rs:221`:
```rust
let host = config.bind.split(':').next().unwrap_or("127.0.0.1");
```
For an IPv6 loopback bind like `[::1]:8788`, `split(':')` yields `"["`; for
`::1:8788` it yields `""`. Neither is recognized by `is_loopback_host`
(`:264-271`), so `bind_warning` emits a spurious `experimental/unsafe`
warning and the bind is mislabeled. IPv4 `host:port` and bare `localhost`
work. The default bind is IPv4 loopback, so this is not exercised today.

Evidence: `http_server.rs:221,255-271`.
Severity Low: no default impact; IPv6 loopback is a reasonable operator choice.
Fix: parse the host with `SocketAddr`/`ToSocketAddrs` or strip a leading
`[...]` bracket before splitting on `:`. (Identical to C3 F-2.)

### F-5 [Low][Confirmed] — SSE stream is snapshot-then-close, not a long-lived stream

`get_mcp` (`http_server.rs:411-439`) builds the SSE body from a finite
`stream::iter(events)` (or a single `resumption_error` event). Once the
buffered events are delivered, the inner stream ends and the SSE response
completes; `KeepAlive` (15s) only keeps the connection open *during*
iteration, it does not block-wait for future events. A client that opens
`GET /mcp` and waits for events produced by a *later* `tools/call` will see
the connection close after the snapshot + keepalive. `pull_facade_events`
runs only after a mutating `tools/call` (`:344-348`), so the event log is
only refreshed on demand.

Evidence: `http_server.rs:411-439,344-348,538-561`; tests use
`timeout(5s, resp.text())` and expect termination (`tests:357-365,399-402`).
Severity Low: honest PARTIAL — the handoff documents "SSE live push" as
missing and the facade has no push seam today. Behavioral limitation for
clients expecting a long-lived stream; acceptable for the experimental
transport. Fix: document explicitly in the module doc that `GET /mcp`
delivers currently-buffered events then closes (snapshot semantics), and
that a real push subscription requires a facade event-stream seam.

### F-6 [Low][Confirmed] — `new_session_id()` is non-cryptographic and predictable

`http_server.rs:702-710` derives the `Mcp-Session-Id` from
`DefaultHasher::new()` over `(SystemTime::now(), AtomicU64 counter)`.
`DefaultHasher` is SipHash with fixed keys; the inputs are low-entropy and
predictable. The session id is not an auth token (auth is the bearer), and
the bearer-fingerprint binding (`:648-657`) means a guessed session id is
still rejected unless the bearer matches the session's opening bearer. So
the practical impact is limited to reducing the barrier to session hijacking
when a bearer has been leaked.

Evidence: `http_server.rs:702-710`; `:648-657` (fingerprint mitigation).
Severity Low: mitigated by bearer fingerprint binding. Fix: generate the
session id from a CSPRNG (e.g., `rand` / `getrandom`) if session-id secrecy
is desired, or document that the id is a non-secret correlation handle
protected by the bearer fingerprint.

### F-7 [Low][Confirmed] — Composition self-loop guard is a string-literal absence check, not semantic

`composition_source_does_not_register_local_mcp_self_loop` (`tests:664-684`)
asserts that `app_server_composition.rs` contains no `http://127.0.0.1:8788/mcp`
literal and no `register_self` symbol. It passes today because the
composition does not wire MCP HTTP at all (PARTIAL, honestly documented). A
future self-registration via a different URL form (e.g.
`http://localhost:8788/mcp`, `127.0.0.1:8788` constructed by string
interpolation, or a `format!`) would evade the guard.

Evidence: `tests:664-684`; grep confirms no `127.0.0.1:8788`/`register_self`
in `app_server_composition.rs`.
Severity Low: acceptable tripwire for the current PARTIAL state, but limited
once product wiring lands. Fix: when the bin wires the listener, replace the
string guard with a semantic test that asserts the composition's MCP client
pool is not populated with the server's own bound address (e.g., inspect the
built `mcpServers` config), or add an explicit opt-in flag guard at the
composition root.

### F-8 [Low][Confirmed] — `SseResumeTable` is now dead code; `McpHttpState::Debug` locks the sessions Mutex

(a) `transport/http.rs:46-67` `SseResumeTable` (the pre-C4 helper cursor table)
is no longer used by any production path: the real server owns its own
per-session event log (`McpSession::events`/`events_after`,
`http_server.rs:117-134`). It is still exercised by its own unit test
(`http.rs:80-98`) but contributes nothing to the Streamable HTTP surface.
(b) `McpHttpState::Debug` (`http_server.rs:149-158`) calls
`self.sessions.lock().unwrap().keys().collect()` inside `fmt`, which locks
the same `Mutex` that `lookup_session`/`initialize`/`delete_mcp` lock; under
contention a `Debug` print can block or, if ever formatted while the same
thread holds the lock, deadlock (single `Mutex`, so re-entrant lock would
poison).

Evidence: `transport/http.rs:46-67`; `http_server.rs:149-158`.
Severity Low: no behavior impact today. Fix: remove `SseResumeTable` (or
gate it behind a `#[cfg(test)]`/`#[allow(dead_code)]` with a comment), and
have `McpHttpState::Debug` read `sessions` via `try_lock` or snapshot only
the count without holding the lock across formatting.

## Informational notes (non-blocking, no fix required)

- **FakeRuntime for tests only:** all black-box tests inject
  `FakeRuntime::new()` as the runtime facade (`tests:36-37`), permitted by the
  handoff §"For tool black-box". The `tools/call` path still routes through
  `invoke_tower_tool`, so the tests prove HTTP framing reaches the shared
  semantic core, but do not exercise a real Shell-backed adapter over HTTP.
  Honest PARTIAL aligned with C4-A map §9 (GREEN for turn-affecting tools over
  HTTP waits on a real adapter; framing uses FakeRuntime). Not a finding.
- **No RED log file in `tests/c4/`:** only `c4_streamable_http_GREEN.log` is
  captured. The RED state (no `/mcp` route, no listener, no axum dep, no
  `tests/` dir) is documented narratively in the C4-A map §1 rather than as a
  captured failing-log artifact. Acceptable given the map is the RED baseline,
  but C3/C1 captured explicit RED logs; a captured RED log would strengthen
  the evidence. (Mirrors C4-D test-review F1.)
- **`stdio.rs:9` unused import** (`process_mcp_stdio_batch`): pre-existing
  compiler warning, unrelated to the C4-B surface. Non-blocking.
- **TLS honestly HUMAN-gated:** `bind_warning` emits the canonical
  `experimental/unsafe` string; the module never advertises production TLS and
  never auto-promotes a cleartext remote bind. D-SEC.13 / MCP102-HUMAN
  preserved. ✓
- **Disconnect-cancels-turn PARTIAL:** HTTP disconnect cleanup is left to
  axum task drop; a turn in flight is not actively interrupted via
  `tower_agent_interrupt` on disconnect (the facade exposes no per-turn handle
  to the HTTP layer). Honestly documented in the handoff. ✓
- **Bearer fingerprint uses `DefaultHasher`:** non-cryptographic, acknowledged
  in the handoff risks. It is a binding fingerprint, not an auth check (auth
  is `validate_http_bearer`). Acceptable. (See F-6 for the related session-id
  predictability note.)

## Required fixes

None blocking for C4-B acceptance. Recommended (non-blocking) follow-ups for a
future wave or `@implementation-loop` pass, ordered by impact:

1. **F-2:** Make `McpHttpConfig::default()` safe — default `require_auth` to
   `false` when `bearer_token` is empty, or require a non-empty bearer when
   `require_auth` is true, or add a `McpHttpConfig::secured(token)`
   constructor and mark `default()` test-only. (Highest priority — security
   API footgun.)
2. **F-3:** Factor the shared JSON-RPC framing (`initialize`/`tools/list`/
   `tools/call`/error/method-not-found) into a module both `handle_mcp_jsonrpc`
   and `dispatch_jsonrpc` call; add a full-shape parity test (descriptors +
   error + method-not-found), not just names.
3. **F-4:** Parse the bind host robustly (IPv6 `[::1]:port`).
4. **F-1:** Correct the "23 black-box" wording to "22 black-box HTTP + 1
   composition source guard".
5. **F-5:** Document the snapshot-then-close SSE semantics in the module doc.
6. **F-6:** Generate `Mcp-Session-Id` from a CSPRNG, or document it as a
   non-secret correlation handle.
7. **F-7:** When the bin wires the listener, replace the string-literal guard
   with a semantic MCP-client-pool guard.
8. **F-8:** Remove/gate `SseResumeTable`; make `McpHttpState::Debug` lock-free.

## Residual risk

- **F-2** is the only security-relevant residual: the public default config is
  unsafe, but has no production exposure today because the composition root
  does not wire `run_mcp_http_server`. Any future wiring must not use
  `default()` without a non-empty bearer.
- **F-3** parity-drift: the two JSON-RPC framers are identical today but
  unenforced for full shape; a future change to one without the other would
  silently break stdio/HTTP parity (MCP102-05).
- SSE is snapshot-then-close (F-5); clients must not assume a long-lived
  stream. Live push requires a facade event-stream seam that does not exist.
- Real-adapter semantics over HTTP are not exercised (FakeRuntime only);
  disconnect-cancels-turn remains PARTIAL.
- TLS remains a HUMAN gate (D-SEC.13); this module correctly does not resolve
  it and never advertises production TLS.
- No composition-root wiring of `run_mcp_http_server` into
  `app_server_composition.rs` / `pager-bin` yet (out of scope for C4-B;
  flagged in wave note). The server is currently only exercised by tests.

## Commands / results (as captured by the implementer; not re-run by this reviewer)

- `cargo test -p xai-grok-mcp-server --features streamable-http` → 23
  integration passed; 0 failed (`tests/c4/c4_streamable_http_GREEN.log`).
- `cargo test -p xai-grok-mcp-server` (default features) → 12 lib tests
  passed; 0 failed (GREEN log §"lib tests").
- `cargo test -p xai-grok-tower-tools -p xai-grok-tower` → green (shared
  semantic core unaffected; per handoff, not re-verified here).
- **Skipped by reviewer:** fresh re-execution of the above. No shell tool in
  this read-only harness. Static cross-check of the GREEN log vs. source is
  consistent: 23 integration + 12 lib test names and asserted behaviors match
  the code; one pre-existing `unused_imports` warning in `stdio.rs:9`
  (unrelated to C4-B).

## Verification checklist

- [x] Base/head, specs, and full changed surface identified.
- [x] Every acceptance criterion has explicit status and evidence.
- [x] Findings cite file/line evidence and include a concrete fix.
- [~] Required checks ran: static review + log cross-check done; fresh
      `cargo test` re-run skipped (no shell tool) — reported honestly.
- [x] Verdict follows the stated gate; no source files were modified.
