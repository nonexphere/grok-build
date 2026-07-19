# C4-A — MCP Streamable HTTP surface map

| Field | Value |
|---|---|
| Agent | `repo-explore` (read-only) |
| Model | `glm-5.2` |
| Wave | C4 prep (items 26–29 inputs) |
| Branch | `goblin-implement-epic-tree` |
| Source handoff | `handoffs/HANDOFF-C4-A-mcp-surface-map.md` |
| Verdict | **NO-GO for C4-B until C1-G lands**; C4 surface is helper-only today |

## 1. Current state of Streamable HTTP — HELPER ONLY, no real server

There is **no `/mcp` HTTP route, listener, or axum/hyper router anywhere in the
workspace**. The Streamable HTTP "server" today is a set of pure, unconnected
helper functions plus an in-memory SSE cursor table.

### 1.1 The `xai-grok-mcp-server` crate (server-side adapter)

`crates/codegen/xai-grok-mcp-server/Cargo.toml:18-19` declares a
`streamable-http` cargo feature that is **empty** — no dependencies, no
`cfg(feature = "streamable-http")`-gated code. The crate has no `axum`, `hyper`,
`tokio/net`, or `TcpListener` dependency at all (grep over the crate returns
zero transport-level symbols; only `lib.rs:1-2` doc comments and the
`serverInfo` literal at `lib.rs:76` mention "server").

`crates/codegen/xai-grok-mcp-server/src/transport/http.rs` provides only:

| Symbol | `file:fn` | Role | Wired to a transport? |
|---|---|---|---|
| `validate_http_bearer` | `transport/http.rs:12` | constant-time-ish Bearer header check; rejects query tokens | **No** — no HTTP layer calls it |
| `reject_token_query` | `transport/http.rs:35` | rejects `token=`/`access_token=`/`api_key=` in query | **No** |
| `SseResumeTable` | `transport/http.rs:46` | in-memory `HashMap<stream_id, u64>` cursor map; `resume_from`/`advance` | **No** — not backed by any event log |
| `post_mcp_response` | `transport/http.rs:73` | shapes a `{"jsonrpc","id","result"}` JSON value | **No** — pure JSON builder |
| `enforce_body_limit` | `transport/http.rs:124` | `bytes > max → "message_too_large"` | **No** — no transport applies it |

`crates/codegen/xai-grok-mcp-server/src/lib.rs` exposes a JSON-RPC dispatcher
`handle_mcp_jsonrpc` (`lib.rs:46`) and `process_mcp_stdio_batch` (`lib.rs:88`)
that operate on `serde_json::Value` — **no HTTP framing, no Accept negotiation,
no SSE, no session header**. `McpTransport` enum (`lib.rs:18`) lists `Stdio` and
`StreamableHttp` as variants but nothing constructs the HTTP variant. The stdio
transport `transport/stdio.rs:run_mcp_stdio` is a real line-oriented loop but is
**not** the Streamable HTTP path.

### 1.2 The `xai-grok-app-server` crate has no HTTP transport

`crates/codegen/xai-grok-app-server/src/transport/mod.rs:6-8` defines
`TransportKind::{InProcess, Stdio, WebSocket}` — **no HTTP/Streamable variant**.
The app-server processor (`processor.rs`, `controller.rs`) is reused by stdio
and websocket but has no `/mcp` route registration. The only `mcp` reference in
the crate is `processor.rs:252` (`mcp_elicitation: false` — a capability flag,
unrelated to a server).

### 1.3 The `xai-grok-mcp` crate is the CLIENT, not the server

`crates/codegen/xai-grok-mcp/src/lib.rs:1-30` documents that this crate is the
**external-server MCP client** (rmcp `StreamableHttpClientTransport`,
`TokioChildProcess`). `servers.rs:28-29` imports the rmcp client transport. It
dials **out** to remote MCP servers; it does not **serve** `/mcp`. Any C4
server work must not conflate this crate with `xai-grok-mcp-server`.

### 1.4 No composition wires `/mcp`

`crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` and
`main.rs` reference `mcp_servers_json` (`main.rs:630,696,714,863`) — that is
**MCP client config** (the `mcpServers` block consumers send to the agent),
not the MCP control-plane server. No binary binds a `/mcp` listener.

**Conclusion:** MCP101-03 (`tasks.md:5`) is correctly OPEN. The corrective
blocker `C-MCP-HTTP` (BLOCKERS.md:7) stands. Audit finding F-03 (no
POST/GET/DELETE `/mcp` router/listener/SSE lifecycle) is accurate.

## 2. Tool catalog — exact nine tools

All nine tools are declared once in `xai-grok-tower-tools/src/lib.rs` and
re-exported by `xai-grok-mcp-server/src/lib.rs:13-15`.

### 2.1 Names and descriptors

`xai-grok-tower-tools/src/lib.rs:5-15` — `TOWER_TOOL_NAMES: [&str; 9]`:

| # | Name | Description (`lib.rs:22-77`) | input_schema_ref |
|---|---|---|---|
| 1 | `tower_agent_list` | List Tower-managed Sessions with filters and pagination. | `tower-tools.schema.json#/$defs/tower_agent_list_input` |
| 2 | `tower_agent_start` | Start a top-level Session in a validated workspace. | `…/tower_agent_start_input` |
| 3 | `tower_agent_send` | Start a Turn or steer the named active Turn. | `…/tower_agent_send_input` |
| 4 | `tower_agent_history` | Read redacted full or last Session history within byte limits. | `…/tower_agent_history_input` |
| 5 | `tower_agent_resume` | Make a dormant Session resident without changing identity. | `…/tower_agent_resume_input` |
| 6 | `tower_agent_wait` | Wait after an event cursor without holding runtime locks. | `…/tower_agent_wait_input` |
| 7 | `tower_agent_interrupt` | Idempotently interrupt the named active Turn. | `…/tower_agent_interrupt_input` |
| 8 | `tower_agent_archive` | Archive a Session without deleting its transcript. | `…/tower_agent_archive_input` |
| 9 | `tower_agent_status` | Read a redacted Session status and residency summary. | `…/tower_agent_status_input` |

Each descriptor also carries an `output_schema_ref` (`lib.rs:22-77`,
`descriptor()` at `lib.rs:82`). Schemas resolve against
`crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json`
(verified by `every_descriptor_resolves_exact_input_and_output_definition`,
`tower-tools/src/lib.rs:124`).

### 2.2 Semantic core (single implementation)

`xai-grok-tower-tools/src/lib.rs:172` — `pub async fn invoke_tower_tool(
runtime: Arc<dyn GrokRuntimeFacade>, agent_type, explicit_opt_in, name,
arguments) -> Result<Value, ToolError>`. This is the **only** implementation of
the nine tools; it dispatches by name (`lib.rs:178-401`) into the
`GrokRuntimeFacade` trait (`xai-grok-tower/src/lib.rs:45`).

ACL: `is_authorized` (`tower-tools/src/lib.rs:95`) — `agent_type == "orchestrator"
|| explicit_opt_in`. Fail-closed for `build`/`review`/custom without opt-in
(asserted by `acl_is_fail_closed_by_default`, `lib.rs:101`).

## 3. Parity path — in-process vs MCP share one semantic core

Parity is **structural today, not duplicated**:

```
                          GrokRuntimeFacade (xai-grok-tower/src/lib.rs:45)
                                     │
            ┌────────────────────────┼─────────────────────────────┐
            │                        │                             │
  in-process tower tools      MCP stdio adapter           (future) MCP HTTP
  (50-tower-agent-tools       xai-grok-mcp-server/         ← C4-B must
   over invoke_tower_tool)    src/lib.rs:call_tool:28      route here
                             → invoke_tower_tool:172
```

- MCP server `call_tool` (`mcp-server/src/lib.rs:28`) →
  `invoke_tower_tool` (`tower-tools/src/lib.rs:172`).
- MCP `handle_mcp_jsonrpc` `tools/call` arm (`mcp-server/src/lib.rs:62`) →
  `call_tool` → `invoke_tower_tool`.
- `tools/list` arm (`mcp-server/src/lib.rs:55`) emits the same
  `TOWER_TOOL_DESCRIPTORS` (`tower-tools/src/lib.rs:22`) used by in-process
  registration.

So the **adapter parity** (MCP101-05, MCP102-05) is real for stdio: both paths
funnel through `invoke_tower_tool`. The HTTP path does not exist yet, so
MCP102-05 PARTIAL ("HTTP driver is helper/cursor table, not a real Streamable
HTTP server", `c0-requirement-matrix.md:137`) is accurate. C4-B must add an HTTP
transport whose `tools/call` reaches `invoke_tower_tool` — **no new semantic
code is needed**, only framing + auth + SSE + session lifecycle around the
existing dispatcher.

`FakeRuntime` (`xai-grok-tower/src/fake.rs:37`) is the test facade; the parity
tests (`adapter_parity_mcp_and_in_process_normalized`,
`tower-tools/src/lib.rs:494`) prove identical ACL deny codes for both shapes.

## 4. Self-loop risk — production must not double-execute tools

The handoff calls out that production composition must not double-execute tools
via a local MCP. Today this is **enforced by absence + canary tests**, not by a
runtime guard:

- `no_self_mcp_loop_tool_names` (`mcp-server/src/lib.rs:147`) asserts no tool
  named `tower_agent_hub` exists.
- `no_local_self_injection_in_production_source`
  (`mcp-server/src/lib.rs:174`) asserts the production (non-`#[cfg(test)]`)
  source of `mcp-server/src/lib.rs` contains **no** `xai_grok_mcp::` import and
  no `McpClient` symbol — i.e. the server adapter does not pull in the client
  crate.
- `forbidden_hub_symbol_absent_from_tool_names`
  (`tower-tools/src/lib.rs:487`) asserts the `tower_agent_hub` name is absent
  from `TOWER_TOOL_NAMES` and from production source.

**Risk for C4-B:** once a real `/mcp` HTTP server exists, a deployment that
points a managed MCP client entry (`mcpServers` with
`url: http://127.0.0.1:8788/mcp`, per `_shared/mcp-server-transport-cli.md:34`)
at its **own** Tower would re-enter the nine tools through HTTP, double-charging
the facade and risking recursive turns. C4-B must NOT introduce any
auto-registration of the local `/mcp` into the session's MCP client pool, and
should add a RED test asserting the server composition does not register itself
as a managed MCP server. The existing canary tests cover the symbol level; they
do **not** cover the composition/wiring level.

## 5. Missing black-box behaviors (C4-B RED test targets)

Per the contract in `_shared/mcp-server-transport-cli.md:17-20`:

| Behavior | Spec | Current | Gap |
|---|---|---|---|
| `POST /mcp` | Bearer, JSON content type, Accept incl. JSON and SSE; JSON or SSE response per negotiation | None | No route, no framing, no Accept negotiation, no JSON-vs-SSE response selection |
| `GET /mcp` | Bearer, `Accept: text/event-stream`, optional `Last-Event-ID`; opens/resumes server event stream | None | No SSE stream; `SseResumeTable` (`http.rs:46`) is a bare counter not connected to facade `replay`/`SubscribeParams` (`tower-tools/src/lib.rs:326`) |
| `DELETE /mcp` | Bearer + negotiated MCP session header; terminates that transport session | None | No route, no session header, no termination |
| Session lifecycle | Negotiated MCP session IDs bound to Tower instance + bearer fingerprint; foreign/expired event IDs return safe resumption error | None | No session id negotiation; no binding to Tower instance (`--tower` from CLI matrix) |
| SSE resume | `Last-Event-ID` resumes; never switches Towers or replays another client's events | `SseResumeTable.resume_from` (`http.rs:55`) only advances a monotonic u64 per stream_id | Not backed by any event log; no per-client isolation; no Tower-instance scoping |
| Auth failure equivalence | Indistinguishable 401 for missing/wrong/malformed bearer | `validate_http_bearer` + `reject_token_query` helpers + tests (`http.rs:80,100`) | No HTTP layer emits the 401; helper-only (MCP102-02 PASS is at helper level) |
| Body limits | `--max-message-bytes 1048576` shared inbound limit | `enforce_body_limit` (`http.rs:124`) + test (`http.rs:131`) | No transport applies it; SSE/queue limits not enforced (MCP102-03 PARTIAL, `c0-requirement-matrix.md:135`) |
| Cancellation | HTTP disconnect → turn interrupt | None | No disconnect detection; no wiring to `tower_agent_interrupt`/facade `interrupt_turn` |
| Disconnect | Transport-level cleanup | None | No connection lifecycle |
| Protocol-version gate | Unsupported protocol-version headers fail before tool dispatch | `handle_mcp_jsonrpc` `initialize` returns a fixed `protocolVersion` (`mcp-server/src/lib.rs:71`) | No version negotiation/rejection on HTTP |
| `tools/list` over HTTP | Same nine descriptors | `handle_mcp_jsonrpc` `tools/list` arm (`lib.rs:55`) returns them | Works in-process/stdio; HTTP framing missing |
| `tools/call` error mapping | Stable Tower codes → `isError: true` structured content, preserve operation IDs | `handle_mcp_jsonrpc` `tools/call` error arm (`lib.rs:66`) returns `code:-32000` text error | Does not emit `isError`/structuredContent on error; operation IDs not preserved (parity gap vs in-process `ToolError` at `tower-tools/src/lib.rs:151`) |

## 6. Suggested RED tests + owning crate

Owning crate: **`xai-grok-mcp-server`**. It currently has **no `tests/`
directory** — all tests are inline `#[cfg(test)]` modules. C4-B should create
`crates/codegen/xai-grok-mcp-server/tests/` for black-box integration tests
(inline unit tests cannot bind a real listener). A `streamable-http` test
harness needs `axum`/`hyper`/`tokio` as **dev-dependencies** (currently only
`tokio` macros/rt are dev-deps, `Cargo.toml:21`).

Suggested RED test names (all initially failing because no `/mcp` route exists):

1. `tests/streamable_http_post_tools_list.rs` — `POST /mcp` with Bearer +
   `Content-Type: application/json` + `Accept: application/json, text/event-stream`
   returns JSON-RPC `tools/list` with exactly nine tools. **Fails: no route.**
2. `tests/streamable_http_post_tools_call.rs` — `POST /mcp` `tools/call`
   `tower_agent_start` returns `structuredContent` with `sessionId`; deny path
   (`agent_type=build`) returns `isError: true` with `forbidden` code, identical
   for missing/wrong/malformed bearer (auth-failure equivalence). **Fails: no route.**
3. `tests/streamable_http_get_sse.rs` — `GET /mcp` with
   `Accept: text/event-stream` opens an SSE stream; `Last-Event-ID` resumes from
   that id; a foreign/expired id returns a safe resumption error and does not
   replay another client's events. **Fails: no SSE stream; `SseResumeTable`
   not wired to facade `replay`.**
4. `tests/streamable_http_delete_session.rs` — `DELETE /mcp` with negotiated
   MCP session header terminates that session; subsequent POST with the same
   session header is rejected. **Fails: no route, no session header.**
5. `tests/streamable_http_body_limit.rs` — a 2 MiB POST is rejected with
   `message_too_large` (or the negotiated 413 equivalent) before tool dispatch.
   **Fails: `enforce_body_limit` not applied by any transport.**
6. `tests/streamable_http_disconnect_cancels.rs` — client disconnects mid-SSE;
   server cleans up the stream and does not leak the session; a turn in flight
   is interrupted via `tower_agent_interrupt`. **Fails: no disconnect path.**
7. `tests/streamable_http_session_bound_to_tower.rs` — session id from
   `--tower A` cannot be used against `--tower B`; bearer fingerprint mismatch
   rejects. **Fails: no session negotiation.**
8. `tests/streamable_http_protocol_version_gate.rs` — unsupported
   `protocol-version` header fails before `tools/call` dispatch. **Fails: no
   version check on HTTP.**
9. `tests/streamable_http_no_self_loop.rs` — composition does not register the
   local `/mcp` URL into the session's MCP client pool; calling
   `tower_agent_list` over HTTP does not re-enter via a managed MCP client.
   **Fails/absent: no composition guard.**
10. `tests/streamable_http_stdio_parity.rs` — same fixture script over stdio
    (`process_mcp_stdio_batch`) and over HTTP produces identical `tools/list`,
    `tools/call` success, and error shapes (MCP102-05 conformance). **Fails: no
    HTTP driver.**

## 7. Files for C4-B implementer (non-overlapping with C1-G / C3-A)

C4-B owns the MCP **server** HTTP surface. C1-G owns
`crates/codegen/xai-grok-shell/src/app_server_runtime/**` (turn lifecycle).
C3-A owns `crates/codegen/xai-grok-app-server/src/transport/websocket.rs` (WS
listener). No overlap.

C4-B owned paths:

| Path | Action |
|---|---|
| `crates/codegen/xai-grok-mcp-server/Cargo.toml` | add `axum`/`hyper`/`tokio` (with `net`/`rt-multi-thread`) as real deps under `streamable-http` feature; promote `tokio` dev-dep |
| `crates/codegen/xai-grok-mcp-server/src/transport/http.rs` | replace helper-only module with a real axum router: `POST/GET/DELETE /mcp`, Accept negotiation, SSE stream backed by facade `replay`, session header, body limit, disconnect cleanup |
| `crates/codegen/xai-grok-mcp-server/src/transport/mod.rs` | re-export the HTTP router |
| `crates/codegen/xai-grok-mcp-server/src/lib.rs` | wire `McpTransport::StreamableHttp` to the router; preserve `no_local_self_injection_in_production_source` canary (do NOT add `xai_grok_mcp::`/`McpClient` imports) |
| `crates/codegen/xai-grok-mcp-server/tests/**` | new integration test dir (items §6) |
| `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json` | read-only reference for input/output schemas (already resolved by `tower-tools` tests) |
| `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` | optional: bind the HTTP listener when `--mcp http://ADDR` is set (CLI matrix in `_shared/mcp-server-transport-cli.md:72-78`) — coordinate with C3-B if both share the daemon |

Shared (read-only) boundary:

- `xai-grok-tower/src/lib.rs:45` (`GrokRuntimeFacade`) — the runtime contract
  the HTTP `tools/call` path must reach via `invoke_tower_tool`.
- `xai-grok-tower-tools/src/lib.rs:172` (`invoke_tower_tool`) — the single
  semantic core; C4-B must NOT duplicate tool logic.
- `xai-grok-mcp/src/{servers,wire,acp_transport,mcp_http_client,liveness}.rs`
  — the **client** crate; C4-B must not modify it (it is the outbound MCP
  client, separate concern).

## 8. Risks / blockers (with evidence)

1. **No real server exists** — every "Streamable HTTP" claim is helper-level.
   MCP101-03 OPEN, MCP102-03/05 PARTIAL (`c0-requirement-matrix.md:129,135,137`),
   `C-MCP-HTTP` blocker (`BLOCKERS.md:7`), audit F-03
   (`reviews/c0/architecture-review.md:228`). C4-B is a net-new transport, not a
   patch.
2. **SSE resume is not backed by an event log.** `SseResumeTable`
   (`http.rs:46`) is a `HashMap<stream_id, u64>`; the facade's event stream is
   `replay(SubscribeParams)` (`tower-tools/src/lib.rs:326`). C4-B must connect
   `Last-Event-ID` → `after_event_seq` against the **Tower instance bound to the
   session**, not the in-memory counter. Risk: a naive wiring would replay the
   wrong client's events — the spec explicitly forbids this
   (`_shared/mcp-server-transport-cli.md:25-27`).
3. **Self-loop is only guarded at the symbol level.** Existing canaries
   (`mcp-server/src/lib.rs:147,174`) prevent `tower_agent_hub` and
   `xai_grok_mcp::` imports but do not prevent a deployment from registering
   `http://127.0.0.1:8788/mcp` as a managed MCP server. C4-B should add a
   composition-level RED test (item §6.9).
4. **`tools/call` error shape parity gap.** `handle_mcp_jsonrpc` error arm
   (`lib.rs:66`) returns `{"code":-32000,"message":...}`; the spec requires
   `isError: true` structured content preserving operation IDs
   (`_shared/mcp-server-transport-cli.md:31-32`). C4-B must align the HTTP
   error mapping with in-process `ToolError` (`tower-tools/src/lib.rs:151`).
5. **No `tests/` dir in the crate** — inline `#[cfg(test)]` modules cannot bind
   a real TCP listener. C4-B must add an integration test directory and dev-dep
   `axum`/`hyper` test helpers.
6. **CLI matrix / daemon co-start.** `_shared/mcp-server-transport-cli.md:72-78`
   specifies `--mcp off|stdio|http://ADDR` and a daemon default of
   `http://127.0.0.1:8788`. The `--stdio` conflict rule (stdout framing owner)
   must be honored. C4-B should coordinate with C3-B (WS `--listen`) since the
   daemon default runs both (`_shared/mcp-server-transport-cli.md:100-106`).
7. **HUMAN TLS gate.** `MCP102-HUMAN` (`c0-requirement-matrix.md`) and
   `D-SEC.13` keep TLS termination a HUMAN gate; C4-B must emit the non-loopback
   security warning and not claim TLS PASS.

## 9. Sequencing — GO/NO-GO for C4-B

**NO-GO for C4-B starting now.** Rationale:

- C4-B is independent of C1-G at the **file** level (no overlap, §7), so it
  could theoretically proceed in parallel. However, the C4 HTTP `tools/call`
  path routes through `invoke_tower_tool` → `GrokRuntimeFacade::start_turn` /
  `steer_turn` / `interrupt_turn` — the **exact** methods C1-G is wiring to a
  real `SessionActor` today. Until C1-G lands and the turn lifecycle is real
  (not `unsupported`), C4-B's `tools/call` RED tests for `tower_agent_send` /
  `tower_agent_interrupt` over HTTP would either (a) hit `unsupported` and be
  vacuous, or (b) require a `FakeRuntime`-backed composition that the
  corrective plan explicitly forbids reintroducing.
- The facade contract (`GrokRuntimeFacade`, `xai-grok-tower/src/lib.rs:45`)
  is stable, so C4-B framing/auth/SSE work can be **prepared** in parallel,
  but GREEN for the turn-affecting tools must wait on C1-G.

**Recommended sequence:**

1. **C1-G** (in progress) → land real `start_turn`/`steer_turn`/`interrupt_turn`.
2. **C1-H/C1-I** review of C1-G.
3. **C3-A → C3-B** (WS listener) can proceed in parallel with C4 framing since
   both share the daemon/co-start matrix but touch different crates.
4. **C4-B** after C1-G GREEN: write RED tests §6 against a real
   `GrokRuntimeFacade` composition (or `FakeRuntime` for pure framing tests,
   real facade for `tools/call` semantics), then implement the axum router.

C4-A (this map) is complete and does not block C1-G, C3-A, or C5-A.

## 10. Evidence index

- `crates/codegen/xai-grok-mcp-server/Cargo.toml:18-19` — empty
  `streamable-http` feature.
- `crates/codegen/xai-grok-mcp-server/src/transport/http.rs:12,35,46,73,124` —
  helper functions only.
- `crates/codegen/xai-grok-mcp-server/src/lib.rs:18,28,46,55,62,66,88,147,174` —
  `McpTransport` enum, `call_tool`, `handle_mcp_jsonrpc`, canaries.
- `crates/codegen/xai-grok-mcp-server/src/transport/stdio.rs:11` — real stdio
  loop (not HTTP).
- `crates/codegen/xai-grok-tower-tools/src/lib.rs:5-15,22-77,82,95,124,147,172,
  326,401,487,494` — nine tools, descriptors, ACL, semantic core, canaries.
- `crates/codegen/xai-grok-tower/src/lib.rs:45`, `src/fake.rs:37` — facade
  trait + test impl.
- `crates/codegen/xai-grok-app-server/src/transport/mod.rs:6-8` — no HTTP
  transport kind.
- `crates/codegen/xai-grok-mcp/src/lib.rs:1-30`, `src/servers.rs:28-29` —
  client crate (outbound), not the server.
- `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json`
  — schema source of truth.
- `.llms/grok-build/_shared/mcp-server-transport-cli.md:17-20,25-32,72-78,
  100-106` — exact HTTP surface + CLI matrix + co-start rules.
- `.llms/grok-build/40-mcp-control-plane/v1-01-server-transports/tasks.md:5` —
  MCP101-03 OPEN.
- `.llms/execution/app-server-mcp-tower-corrective/waves/c0-requirement-matrix.md:
  129,135,137` — MCP101-03 OPEN, MCP102-03/05 PARTIAL.
- `.llms/execution/app-server-mcp-tower-corrective/BLOCKERS.md:7` — C-MCP-HTTP.
- `.llms/execution/app-server-mcp-tower-corrective/reviews/c0/architecture-review.md:
  132,228-229` — F-03 audit, Wave C3/C4 ownership.
