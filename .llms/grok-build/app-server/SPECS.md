# App Server — SPECS

Referências normativas: `changes/grok_app_server_spec_bundle/`. O plano explica
arquitetura; schema, TypeScript e JSONL são seeds que devem passar a ser gerados
de uma única fonte Rust.

## 1. Domínio e protocolo

JSON-RPC 2.0 bidirecional com initialize gate e entidades Thread/Turn/Item,
stable IDs, eventSeq, item revision, interactions e capability negotiation.
Core v1 permanece próximo ao Codex; extensions exclusivas usam `grok/*`.

## 2. Runtime facade e projector

Uma `GrokRuntime` facade encapsula sessão e operações. Eventos ACP/xAI/runtime
são normalizados deterministicamente. O mesmo registry impede actor duplicado.

## 3. Server core

Connection registry, scoped serializer, thread registry, turn coordinator,
subscription hub e outbound router. Apenas um foreground Turn por Thread.

## 4. Histórico

Arquivos de sessão continuam autoritativos. SQLite é projeção rebuildable com
offsets, epochs, pagination, replay, fork, rewind e active-turn journal.

## 5. Approvals e controle

Interaction ID é durável e distinto de request ID. Controller lease determina
destino de reverse requests; primeira resposta válida vence; stale answer é
rejeitada e disconnect é reconciliado.

## 6. Transportes e segurança

In-process, stdio, IPC e WebSocket usam o mesmo processor/conformance suite.
Remote é opt-in, autenticado, scoped e origin-checked; filas são limitadas e
slow observers não bloqueiam runtime.

## 7. Clientes e compatibilidade

TUI migra por shadow → adapter → native tracker → daemon default. ACP continua
por adapter compartilhado. Codex compatibility fica em adapter separado.

## 8. Validação

Schema/snapshot/fuzz, golden replay, property tests, fault injection,
transport conformance, security/load/performance e TUI parity.

## Infra

- daemon local promovendo leader existente;
- projection SQLite rebuildable;
- remote TLS/auth somente quando habilitado;
- métricas por connection/thread/turn/queue/replay/projection.
