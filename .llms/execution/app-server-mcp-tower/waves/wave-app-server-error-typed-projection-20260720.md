# Wave — typed App Server error projection (2026-07-20)

## Objetivo

Eliminar a duplicação de envelopes de erro entre o protocolo, o processor e o
listener WebSocket, mantendo o contrato JSON-RPC com `data` tipado e
`operationId: null` explícito quando não há operação associada.

## Alterações

- `DomainErrorData` passou a serializar `operationId` nulo explicitamente.
- `ErrorSpec::rpc_error_value()` usa a mesma projeção tipada do catálogo.
- `domain_data_for_numeric()` fornece fallback fail-closed para `internal_error`.
- Processor e WS reutilizam a projeção canônica e não mantêm helpers locais de
  `code`/`retryable`.
- O teste do catálogo valida a presença do campo nulo no wire.

## Evidência

```text
cargo test -p xai-grok-app-server-protocol errors
3 passed

cargo test -p xai-grok-app-server
41 passed
```

O warning existente sobre `src/main.rs` compartilhado por três binários do
`xai-grok-pager-bin` permanece sem relação com esta wave.

## Limite ainda aberto

Esta wave não prova a unificação completa de `OperationResult`/`RpcErrorData`
nem a convergência do catálogo entre App Server e MCP; esses itens continuam
parciais em AS109-02 e dependem de uma fixture de contrato compartilhada.
