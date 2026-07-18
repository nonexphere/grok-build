# Control plane security and threat model

[provenance: handoff R1..R5/§13.4, review D-SEC.*]

## Locked MVP authority model

One Tower bearer grants full control of sessions visible to that Tower. There
are no scopes, Origin allowlist or mandatory TLS in MVP. That is an explicit
product decision, not a security claim. Loopback is the default; non-loopback
bind requires an explicit CLI value, emits a high-severity warning and remains
blocked from release until the human gate below is accepted.

## Token contract

- 32 random bytes from the OS CSPRNG, base64url without padding; minimum 256 bits.
- Stored only in `~/.grok-oss/towers/<id>/control.token`, regular file mode 0600.
- Parent directory must be owned by current user and not group/world writable.
- Creation uses exclusive open; rotation uses write+fsync+atomic rename.
- Tokens never appear in argv, query strings, JSON-RPC params, MCP tool args,
  structured fields, telemetry, panic messages or error bodies.
- HTTP/WS accepts only `Authorization: Bearer <token>`. Cookies, query params,
  Basic auth and WebSocket URL credentials are rejected.

## Authentication failure matrix

| Condition | HTTP/WS | stdio/local in-process | Audit outcome |
|---|---|---|---|
| missing header | 401 generic | n/a | `auth_missing` |
| malformed scheme/token | 401 generic | n/a | `auth_malformed` |
| wrong/revoked token | 401 generic | n/a | `auth_rejected` |
| expired token (future expiring format) | 401 generic | n/a | `auth_rejected` |
| unsafe token file perms | server refuses startup | server refuses token mode | `token_permissions_unsafe` |
| non-loopback without explicit bind | config error | n/a | `remote_bind_not_explicit` |
| authenticated but invalid method | JSON-RPC method error | same | `method_not_found` |
| authorized tool, ACL-denied agent | MCP tool error | in-process tool error | `tower_acl_denied` |

Comparison is constant-time over decoded bytes. Responses never distinguish
unknown, expired or revoked tokens. Connection logs record a keyed token
fingerprint only when needed for rotation debugging.

## Full-control operation set

Authenticated clients can list/read sessions and items; create/archive
sessions; start/steer/interrupt turns; answer interactions; subscribe/replay;
and invoke all nine Tower tools. Provider credential values, environment
secrets and raw tool-private state are never readable through this authority.

## Transport posture

Cleartext `http://` and `ws://` are allowed because TLS is optional in MVP.
Loopback cleartext is the recommended default. LAN/internet cleartext exposes
the bearer and full session control to passive or active network attackers.
Documentation and startup warnings MUST say this verbatim in substance. A
reverse proxy may terminate TLS, but forwarded identity does not replace bearer.

Required startup warning for explicit non-loopback cleartext bind:

```text
HIGH RISK: grok-oss Tower is listening on a non-loopback cleartext address.
Anyone who obtains this bearer token has full control of this Tower. HTTP/WS
traffic can expose the token and session contents to network attackers. Use a
trusted network or TLS-terminating reverse proxy. No Origin or token scopes are
enforced in this MVP.
```

The warning goes to stderr/logs without address credentials and is emitted once
per enabled remote surface. It is not a confirmation prompt that automation can
accidentally bypass.

## Explicitly deferred defenses

Fine-grained scopes, Origin checks, TLS/WSS termination, pairing/device approval,
multi-user identity and expiring delegated tokens are future features. They must
be additive and migration-tested; none is simulated by undocumented header
conventions in MVP.

## Limits and backpressure defaults

- inbound JSON-RPC/MCP body: 1 MiB; reject before full allocation where possible;
- WebSocket frame/message: 1 MiB; ping 30s, pong timeout 10s;
- per-connection outbound queue: 1024 events;
- replay request: 10,000 events or 16 MiB, whichever occurs first;
- initialize timeout: 10s; idle unauthenticated HTTP stream timeout: 15s;
- Tower tool wait timeout: max 300s;
- no request-rate quota in MVP, but concurrency and queue bounds remain enforced.

Overflow returns a stable `resync_required` cursor error and closes the affected
subscription; it never drops a middle event silently.

## Audit fields and redaction

Structured audit event fields: timestamp, instance ID, transport, connection
ID, token fingerprint, method/tool, session ID, turn ID, operation ID,
idempotency-key hash, decision, stable error code, latency and response bytes.
No prompts, model output, raw params, token, provider key or auth header.
Canary tests inject values named `GROK_TEST_SECRET_CANARY` through headers,
params, tool output and panics, then assert absence across logs/errors/metrics.
Canaries cover lengths 0, 1, 8, 32, 43 (token-shaped), 256 and 4096 bytes plus
prefix/suffix embeddings, JSON escaping and Unicode. Redaction replaces the
entire secret with a constant marker; it never preserves a revealing prefix.

## Threat scenarios

1. Stolen bearer gives full Tower control: minimize file/network exposure and rotate.
2. Cleartext remote sniffing steals bearer: warn; recommend loopback or TLS proxy.
3. Malicious website opens WS: no Origin defense exists; bearer secrecy is sole gate.
4. Symlink token swap reads/writes another file: reject symlink/non-regular paths.
5. Oversized frames exhaust memory: enforce byte limit before deserialization.
6. Slow subscriber exhausts queue: bounded queue then explicit resync.
7. Prompt/tool output leaks credentials: projector and audit redaction with canaries.
8. Local unauthorized agent invokes Tower: fail-closed agent-type ACL.
9. Replay cursor crosses restart epoch: reject and demand snapshot/reconnect.
10. MCP self-loop recursively invokes Tower: composition forbids local auto-injection.
11. Confused Tower instance uses wrong token: instance-bound metadata and endpoints.
12. Timing probe enumerates valid tokens: constant-time compare and generic 401.

## Required tests and release gate

Named suites: `bearer_header_only`, `token_file_permissions`,
`remote_bind_requires_explicit_address`, `redaction_canary_all_sinks`,
`oversized_message_rejected`, `slow_subscriber_gets_resync`,
`tower_acl_fail_closed`, and `epoch_mismatch_rejected`.

`(HUMAN, manual-verify, blocking: remote release)`: review this threat model and
explicitly accept full-control bearer plus optional TLS before any non-loopback
mode is advertised as release-ready. Local/scaffold work is not blocked.
