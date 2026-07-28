# Identity, Event Ordering and Replay

**Fonte de verdade.** App Server possui sequencing/replay; Tower Core fornece
Session identity; todos os transportes e tools preservam este contrato.

1. `event_seq` cresce estritamente por Session.
2. `item_revision` nunca diminui; Item terminal nunca volta a in-progress.
3. Rebuild preserva IDs já expostos e não cria um novo lifecycle.
4. Rewind incrementa `history_epoch` e invalida cursors da história removida.
5. Snapshot-then-live captura watermark, bufferiza eventos posteriores, entrega
   snapshot e drena sem gap/duplicação.
6. Cursor é opaco e vinculado a instance, Session, epoch e query.
7. Mutation externa recebe `idempotency_key`; chave+payload idênticos repetem
   resultado; payload diferente retorna conflito.
8. Eventos/payloads aplicam limites e redaction antes de persistência ou wire.

## Conformance

Replay + live deve produzir a mesma visão final em in-process, stdio,
WebSocket, MCP e SDK. MCP pode reduzir a representação a tool result, mas não
alterar ordenação, status, erro ou idempotência.
