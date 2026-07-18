# TypeScript SDK contract

[provenance: handoff §13.7/K1..K3, review D-TS.*]

Proposed package path/name is `packages/grok-oss-app-server` /
`@brasalabs/grok-oss-app-server`. It remains `private: true` and experimental
until a human approves public naming and protocol stability.

The package exports exact wire types, `RpcError`, a transport-neutral
`AppServerClient`, stdio/Node and WebSocket transports, and an
`AsyncIterable<Item>` event stream. The final
`subscribe({sessionId, historyEpoch, afterEventSeq})` behavior performs ordered
replay, live continuation and reconnect with epoch validation. The present
scaffold implements the typed iterator and one subscription; automatic
reconnect/AbortSignal are explicitly implementation work in `60/v1-01`, not
fake scaffold behavior. `close` rejects pending requests on transport closure.

Node supports stdio and WebSocket; browsers support WebSocket only. Browser
bearer configuration must account for the WebSocket API’s header limitation and
therefore remains unsupported until a safe handshake mechanism is specified;
tokens MUST NOT be placed in URLs. This is an explicit current limitation.

Rust serde types + checked-in JSON Schema are the source of truth. The current
TS file is an interim matching skeleton. `npm run check:drift` verifies critical
definitions/fields/status unions now. The future generator writes to a temp tree
and CI diffs it against checked-in output; then generated files become read-only.

Error mapping preserves JSON-RPC code, stable `data.code`, retryability and
request ID. Transport close, initialize failure, epoch mismatch and resync are
distinct typed errors. Examples required before release: Node stdio and Node
WebSocket; browser WebSocket is added only after safe bearer transport exists.

Public class methods are `initialize`, `sessionStart`, `sessionResume`,
`sessionFork`, `sessionRead`, `sessionList`, `sessionArchive`, `turnStart`,
`turnSteer`, `turnInterrupt`, `subscribe`, generic `request`, `notify` and
`close`. Construction receives a transport; helper transports perform Node
stdio spawn or WS bearer upgrade. Native browser WebSocket cannot set the bearer
header, so browser support is explicitly unavailable rather than putting the
token in a URL.
