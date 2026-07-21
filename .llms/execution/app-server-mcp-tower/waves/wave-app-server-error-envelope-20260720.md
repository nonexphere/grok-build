# Wave: App Server error envelope convergence — 2026-07-20

## Change

The App Server processor and WebSocket framing layer now emit an explicit
`operationId: null` in every JSON-RPC error `data` object. This aligns the
in-process, stdio, and WebSocket wire shapes with the MCP structured error
projection and makes absence of failed-operation identity explicit rather than
transport-dependent.

## TDD evidence

The new regression assertion first failed because `operationId` was absent,
then passed after both error envelope producers were updated.

```text
cargo test -p xai-grok-app-server
41 passed; 0 failed
```

## Remaining scope

AS109-02 remains partial until `OperationResult` and `RpcErrorData` share a
canonical typed projection and the App Server/MCP error catalogs are proven
identical for the full method/error matrix.
