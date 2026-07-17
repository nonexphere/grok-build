# Epic v1-approvals-control — Interações e controller lease

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: `../v1-core-in-process/`  
Habilita: `v1-daemon-transports-security`  
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`, `@code-review`

## Arquitetura

Adiciona ApprovalBroker e controller election/recovery para command, file,
plan, user input e MCP elicitation. O broker roteia; runtime ainda decide policy.

## Escopo

### ADICIONAR
- durable Interaction IDs/state, reverse request routing e resolution events;
- controller lease, observer/control access e grant integration.

### REFACTORIZAR
- pending-interaction RAII é preservado abaixo do broker com stable identity.

### REMOVER
- approval tied exclusivamente a uma connection/request ID.

### MANTÉM
- sandbox/hooks/permission authority no runtime.

## Business rules

- primeira resposta válida vence atomicamente;
- stale/wrong-controller/out-of-scope answer é rejeitada;
- disconnect não auto-aprova e segue explicit failover policy;
- persistent grants respeitam runtime scopes e remote restrictions.

## Contratos

- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/security-authority-boundaries.md)
- [runtime ownership](../../_shared/runtime-ownership.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Controller disconnect/takeover/replay não produz double execution.
- Todos interaction types resolvem/cancelam/reemitem deterministicamente.
- Runtime sandbox/hooks/grants permanecem autoridades comprovadas.

## Riscos e incertezas

- **[HIGH][Confirmed] double approval/double execution:** atomic first-answer and idempotency.
- **[HIGH][Likely] controller disconnect ambiguity:** persisted interaction + explicit reissue.
- **Human decision required:** election/reclaim e remote persistent grants.
