# Session protocol v1 — stable errors and backpressure

[provenance: review D-SP.17..19, handoff R1..R5]

## JSON-RPC and domain catalog

| Numeric | `data.code` | Retryable | Safe message rule |
|---:|---|---:|---|
| -32700 | `parse_error` | no | no input echo |
| -32600 | `invalid_request` | no | identify field class, not secret value |
| -32601 | `method_not_found` | no | method may be echoed after length/control filtering |
| -32602 | `invalid_params` | no | field path allowed; rejected value omitted |
| -32603 | `internal_error` | maybe | correlation/operation ID only |
| -32001 | `unauthorized` | no | same for missing/invalid/revoked token |
| -32002 | `not_initialized` | yes | initialize first |
| -32003 | `already_initialized` | no | one initialize per connection |
| -32004 | `protocol_version_unsupported` | no | safe supported-version list |
| -32010 | `session_not_found` | no | only after authority check |
| -32011 | `turn_not_found` | no | only after session authority check |
| -32012 | `epoch_mismatch` | yes | current epoch may be returned |
| -32013 | `cursor_too_old` | yes | request snapshot/resubscribe |
| -32014 | `resync_required` | yes | last safe cursor only |
| -32015 | `idempotency_conflict` | no | key hash/operation ID, never original payload |
| -32016 | `invalid_state` | state-dependent | safe current/allowed states |
| -32017 | `interaction_pending` | yes | interaction ID allowed |
| -32018 | `controller_lease_required` | yes | current lease revision, no client identity |
| -32019 | `interaction_already_resolved` | no | terminal decision category allowed |
| -32020 | `invalid_workspace` | no | normalized safe reason; avoid existence leak |
| -32021 | `message_too_large` | no | observed/allowed byte counts |
| -32022 | `backpressure` | yes | queue cap and resync instruction |
| -32023 | `tower_draining` | yes | drain deadline |
| -32024 | `runtime_unavailable` | yes | operation ID only |

Every domain error data object includes `code` and `retryable`; mutation errors
also include `operationId` when allocated. Messages are stable enough for humans
but clients branch only on numeric + string code.

## Idempotency conflict

```json
{"jsonrpc":"2.0","id":45,"error":{"code":-32015,"message":"The idempotency key was already used with different input.","data":{"code":"idempotency_conflict","retryable":false,"operationId":"op_original"}}}
```

The server stores a canonical request digest and original terminal/nonterminal
result. It never returns the original prompt/tool arguments in conflict data.

## Backpressure policy

Default outbound capacity is 1024 event envelopes per connection. Lifecycle and
terminal events are non-droppable. Adjacent deltas for the same
Session/Turn/Item/stream may coalesce while retaining final content/revision.
If coalescing cannot restore capacity, the processor:

1. marks only affected subscriptions resync-required;
2. enqueues `subscription/resyncRequired` if possible;
3. closes those subscriptions with last safe eventSeq;
4. keeps runtime and unrelated subscriptions alive;
5. records a redacted metric/audit event.

Inbound maximum is 1 MiB before JSON allocation. Page size is 100. Replay is
10,000 events or 16 MiB. Initialize is 10s. WebSocket ping is 30s and pong
timeout 10s. Tool wait maximum is 300s.

Named tests: `oversized_input_rejected_before_deserialize`,
`delta_coalescing_preserves_final_revision`,
`terminal_event_is_never_dropped`, and
`one_slow_subscription_does_not_block_runtime_or_other_sessions`.

## Transport framing invariance

| Transport | One message | EOF/close | Diagnostics |
|---|---|---|---|
| in-process | typed processor call | client handle drop | tracing only |
| stdio | one UTF-8 JSON object per line (NDJSON) | stdin EOF begins drain | stderr only |
| WebSocket | one JSON object per text message | close begins connection cleanup | server logs only |
| existing leader IPC | existing byte protocol preserved | existing semantics | existing channel |

Batch arrays and binary WebSocket messages are invalid. Transport adapters may
add authenticated peer metadata but cannot change method result or domain error.
