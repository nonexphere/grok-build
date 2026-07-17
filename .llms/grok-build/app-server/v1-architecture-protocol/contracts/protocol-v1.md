# Grok App Server Protocol v1

**Fonte de verdade.** Este contrato pertence ao App Server. O Rust protocol
crate deverá gerar schema, TypeScript, fixtures e SDKs. O bundle em `changes/`
é seed de design, não uma segunda fonte após implementação.

## Envelope

Native wire usa JSON-RPC 2.0 requests, notifications, success e failure. IDs de
request são string ou number; server-initiated requests usam o mesmo envelope.
Mensagens anteriores ao `initialize` são recusadas salvo initialize/health
explicitamente permitidos.

## Initialize

Cliente envia protocol min/max, client info, capabilities, notification opt-out
e metadata bounded. Servidor escolhe versão intersectada e retorna server info,
capabilities e transport security facts. `initialized` finaliza handshake.

Ausência de versão compatível é erro terminal da conexão. Capability ausente
não pode ser invocada nem implicitamente simulada.

## Core entities

### Thread

Representa sessão Grok persistente: stable ID, status, cwd/display cwd, model,
parent/relation, active/latest Turn, capabilities e safe metadata.

### Turn

Representa um prompt/runtime round: ID, ordinal, kind/status, input, model,
origin, timing, Items view, usage/error e metadata. Há no máximo um foreground
Turn por Thread; background Items podem sobreviver ao Turn.

### Item

Unidade observável: message, safe reasoning summary, command, file change,
plan, MCP, skill, hook, approval/input, subagent, worktree, background task,
compaction, goal ou typed Grok extension. Contém ID, Thread/Turn, status,
revision, timing e bounded metadata.

## Stable methods

- `thread/start`, `thread/resume`, `thread/fork`, `thread/read`, `thread/list`,
  `thread/subscribe`;
- `turn/start`, `turn/steer`, `turn/interrupt`;
- lifecycle notifications `thread/*`, `turn/*`, `item/started`,
  `item/completed`, typed deltas;
- server requests para command/file/plan approval, user input e MCP
  elicitation; `serverRequest/resolved` fecha a interação.

Grok-specific methods usam namespace `grok/*`. Experimental methods são
capability-gated e não recebem estabilidade implícita.

## Ordering and replay

- `eventSeq` cresce estritamente por Thread;
- Item revision cresce monotonicamente;
- delta referencia Thread/Turn/Item/eventSeq/revision e optional stream/sequence;
- snapshot-then-live usa watermark/buffer/drain;
- cursor é opaque e vinculado a query/history epoch;
- rewind incrementa epoch e invalida history removed cursors.

## Interactions

Interaction ID é durável e distinto do request ID usado numa conexão. Uma
interaction carrega Thread/Turn/Item, reason, expiry, available decisions e
controller epoch. Primeira resposta válida vence; stale/replayed/wrong-scope é
erro sem efeito. Disconnect nunca implica accept.

## Idempotency

Mutating client methods exigem/recomendam `idempotencyKey` conforme inventory.
Mesma chave e payload retorna resultado original; payload divergente retorna
conflict. Request retry não duplica Thread, Turn, fork, approval ou effect.

## Errors

| Categoria | Exemplos |
|---|---|
| protocol | parse, invalid request/params, method/capability/version |
| state | not initialized, thread busy/not found, stale cursor/revision |
| authority | unauthorized, forbidden, controller required/stale |
| runtime | cancelled, provider/tool/session failure |
| resource | overloaded, message too large, timeout, projection unavailable |

Erro contém code, safe message e typed/bounded data. Secrets, hidden reasoning e
raw provider credentials nunca aparecem.

## Transport invariance

In-process, stdio, IPC e WebSocket usam o mesmo processor e conformance suite.
Transport pode acrescentar authenticated/remote/max-size facts, mas não mudar
method semantics. Compatibility listener Codex pode aceitar missing `jsonrpc`
somente conforme ADR e adapter isolado.

## Backpressure

Cada conexão tem filas bounded e writer independente. Lifecycle não é
descartado; deltas podem ser coalescidos preservando final revision. Observer
lento não afeta runtime/TUI controller. Overload produz disconnect/error
explícito e replay posterior.

## Security

Remote é disabled default. Non-loopback requer auth scopes, Origin allowlist e
TLS policy. Path/command/file operations continuam sob runtime sandbox/hooks.
Protocol transmite apenas provider descriptors seguros e safe reasoning
summaries.

## Compatibility invariants

- session UUID/known prompt ID permanecem stable Thread/Turn IDs;
- generated schema/TS/examples derivam da mesma Rust source;
- ACP e Codex adapters compartilham processor/runtime registry;
- additive v1 changes preservam clients que ignoram campos novos;
- projection rebuild não muda IDs já publicados.
