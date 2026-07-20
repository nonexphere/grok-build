# MCP server, transports and CLI contract

[provenance: handoff §13.2/§13.9/§13.10b, review D-MCP.*/D-TR.*]

## MCP server boundary

`xai-grok-mcp-server` makes grok-oss a server; existing `xai-grok-mcp` remains
the client for external servers. The server registers the nine `tower_agent_*`
tools only—no resources or prompts in MVP—and dispatches through the shared
facade. Stdio MCP uses Content-Length framing required by the selected MCP
library; Streamable HTTP uses `POST /mcp` plus `GET /mcp` SSE/resumption where
negotiated. HTTP requires the Tower bearer header on every request.

Exact HTTP surface:

| Request | Required headers | Behavior |
|---|---|---|
| `POST /mcp` | Bearer header, JSON content type, Accept including JSON and SSE | JSON-RPC request/notification; JSON or SSE response per negotiation |
| `GET /mcp` | Bearer, `Accept: text/event-stream`, optional `Last-Event-ID` | opens/resumes server event stream |
| `DELETE /mcp` | Bearer + negotiated MCP session header | terminates that MCP transport session |

Negotiated MCP session IDs are bound to Tower instance and bearer fingerprint.
Foreign/expired event IDs return a safe resumption error; they never switch
Towers or replay another client’s events. Unsupported protocol-version headers
fail before tool dispatch.

MCP errors map stable Tower codes to `isError: true` structured content while
preserving operation IDs and retryability. A conformance suite compares tool
list, JSON schemas, success and errors with in-process descriptors.

External client example (never auto-written by grok-oss):

```json
{"mcpServers":{"grok-oss-tower":{"url":"http://127.0.0.1:8788/mcp","headers":{"Authorization":"Bearer ${GROK_OSS_TOWER_TOKEN}"}}}}
```

## App Server transports

Stdio reads/writes one JSON-RPC message per line in MVP; stdout is protocol-only
and diagnostics go to stderr. EOF begins graceful drain. In-process uses typed
processor calls and the same initialize gate. WebSocket requires subprotocol
`grok-oss.app-server.experimental-v2`, bearer during HTTP upgrade, 30s ping,
10s pong timeout and 1 MiB messages. Each text frame contains one JSON object;
binary, fragmented-over-limit and batch frames are rejected.

Unix IPC remains the promoted leader path and is not redesigned in this
scaffold. A new cross-platform custom IPC protocol is deferred; existing local
leader semantics are characterized first.

In-process is a typed handle, not JSON serialized through memory:

```rust
#[async_trait]
pub trait AppServerClient {
    async fn initialize(&self, request: InitializeParams) -> Result<InitializeResult, ClientError>;
    async fn subscribe(&self, cursor: SubscribeParams) -> Result<EventStream, ClientError>;
    async fn close(&self) -> Result<(), ClientError>;
}
```

Typed method calls use the same processor dispatch. A new Unix/custom IPC
protocol is deferred; existing leader IPC remains the TUI/dashboard channel.

## CLI matrix

Canonical command is `grok-oss app-server`.

| Flag | Values/default | Effect |
|---|---|---|
| `--tower` | ID / `default` | isolated Tower instance |
| `--listen` | `off\|stdio://\|ws://ADDR` / `ws://127.0.0.1:8787` in daemon mode | App Server surface |
| `--mcp` | `off\|stdio\|http://ADDR` / `http://127.0.0.1:8788` in daemon mode | MCP surface |
| `--stdio` | alias for `--listen stdio:// --mcp off` | single protocol owns stdout |
| `--token-file` | Tower default path | bearer source; never token literal |
| `--max-message-bytes` | `1048576` | shared inbound limit |
| `--replay-window-events` | `10000` | replay bound |
| `--drain-timeout` | `10s` | shutdown grace |

No args in interactive CLI preserve existing TUI behavior. Explicit daemon
mode defaults to both loopback App Server WS and MCP HTTP. `--stdio` cannot
coexist with MCP stdio because stdout has one framing owner. Any explicit
non-loopback address emits the security warning and requires the release gate.

Token administration is required for remote GA and owned by
`40/v1-05-token-scopes-tls-release`: safe-ID create/list/revoke/rotate,
one-time creation output, scopes and connection revocation. Exact UX remains a
HUMAN freeze before implementation. Token material is never printed by default
or accepted in URLs.

## Health and co-start

`GET /healthz` proves process liveness without auth or state detail.
`GET /readyz` requires bearer and reports instance ID, protocol version,
draining state and enabled surfaces without session/provider secrets.

| App Server | MCP | Valid | Meaning |
|---|---|---:|---|
| off | off | no | no control surface |
| stdio | off | yes | App Server stdio only |
| off | stdio | yes | MCP stdio only |
| ws | off | yes | App Server WS only |
| off | http | yes | MCP HTTP only |
| ws | http | yes/default daemon | both share facade/token |
| stdio | stdio | no | stdout framing conflict |
