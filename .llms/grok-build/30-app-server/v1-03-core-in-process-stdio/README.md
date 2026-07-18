# Epic v1-03 — Core in-process e stdio

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–4 semanas
Depende de: `../v1-02-runtime-facade-projection/`
Habilita: `../v1-04-websocket-remote-auth/`, `../v1-05-history-replay/`, `../v1-06-approvals-control/`, `../../40-mcp-control-plane/v1-01-server-transports/`
Skills relacionadas: `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Entrega processor, connection state, serialization scopes, registries,
subscriptions/outbound, in-process e **stdio NDJSON**. A primeira vertical slice
executa um coding Turn e reconstrói transcript final.

## Escopo

### ADICIONAR
- initialize gate, message processor e structured errors;
- SessionRegistry, TurnCoordinator, SubscriptionHub, OutboundRouter;
- in-process/stdio transports, typed client e core methods.

### REFACTORIZAR
- nenhum runtime core; facade permanece boundary.

### REMOVER
- nenhuma interface existente.

### MANTÉM
- um foreground Turn por Session e background tasks independentes.

## Business rules

- request scopes serializam mutation conflitante sem bloquear outras Sessions;
- idempotency deduplica start/mutation;
- lifecycle não é dropável; slow subscriber é isolado;
- cancellation/interrupt truthfully resolves Turn/Items.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/control-plane-security.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Scripted client executa Turn completo e reconstrói transcript final.
- Concurrency/idempotency/cancellation/slow-subscriber invariants passam.
- Uma sessão carregada nunca cria dois actors.

## Riscos e incertezas

- **[HIGH][Confirmed] duplicate Session load:** actor duplicado — shared pending-load future/registry.
- **[HIGH][Likely] queue/lock coupling:** deadlock/head-of-line — scoped serializer e independent writer.
- **UNVERIFIED:** bounds finais de message/queue para local clients.
