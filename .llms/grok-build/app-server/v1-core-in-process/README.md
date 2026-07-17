# Epic v1-core-in-process — Vertical slice Thread/Turn/Item

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: `../v1-runtime-facade-projection/`  
Habilita: `v1-history-replay`, `v1-approvals-control`  
Skills relacionadas: `@implementation-loop`, `@code-review`

## Arquitetura

Entrega processor, connection state, serialization scopes, registries,
subscriptions/outbound e typed in-process client. A primeira vertical slice
executa um coding Turn e reconstrói transcript final.

## Escopo

### ADICIONAR
- initialize gate, message processor e structured errors;
- ThreadRegistry, TurnCoordinator, SubscriptionHub, OutboundRouter;
- in-process transport/client e core methods.

### REFACTORIZAR
- nenhum runtime core; facade permanece boundary.

### REMOVER
- nenhuma interface existente.

### MANTÉM
- um foreground Turn por Thread e background tasks independentes.

## Business rules

- request scopes serializam mutation conflitante sem bloquear outras Threads;
- idempotency deduplica start/mutation;
- lifecycle não é dropável; slow subscriber é isolado;
- cancellation/interrupt truthfully resolves Turn/Items.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/security-authority-boundaries.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Scripted client executa Turn completo e reconstrói transcript final.
- Concurrency/idempotency/cancellation/slow-subscriber invariants passam.
- Uma sessão carregada nunca cria dois actors.

## Riscos e incertezas

- **[HIGH][Confirmed] duplicate Thread load:** actor duplicado — shared pending-load future/registry.
- **[HIGH][Likely] queue/lock coupling:** deadlock/head-of-line — scoped serializer e independent writer.
- **UNVERIFIED:** bounds finais de message/queue para local clients.
