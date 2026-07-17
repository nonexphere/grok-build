# Epic v1-daemon-transports-security — Leader promovido e transportes

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: `../v1-history-replay/`, `../v1-approvals-control/`  
Habilita: `v1-tui-migration`  
Skills relacionadas: `@implementation-loop`, `@code-audit`, `@code-review`

## Arquitetura

Promove o leader existente a daemon App Server e conecta in-process, stdio,
IPC e WebSocket ao mesmo processor. Adiciona auth, backpressure, health,
graceful restart e common conformance.

## Escopo

### ADICIONAR
- transport acceptors, authn/authz, connection lifecycle, daemon/admin APIs;
- bounded queues, overload policy e graceful drain/restart.

### REFACTORIZAR
- leader framing/routing/reconnect vira substrate do App Server.

### REMOVER
- segundo daemon/control plane alternativo; nunca criar paralelo.

### MANTÉM
- ACP capability/transport por adapter e rollout flags.

## Business rules

- todos transportes passam a mesma black-box suite;
- remote off por default; non-loopback exige TLS/auth/scopes/Origin;
- lifecycle events não são descartados; delta pode coalescer;
- shutdown resolve/drains truthfully, sem alegar Turn completo.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/security-authority-boundaries.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Mesma black-box conformance passa in-process, stdio, IPC e WebSocket.
- Threat/load/fault/cross-platform tests fecham os controls obrigatórios.
- Graceful restart e slow clients não perdem lifecycle nem bloqueiam runtime.

## Riscos e incertezas

- **[HIGH][Confirmed] remote attack surface:** deny-by-default + threat tests.
- **[HIGH][Confirmed] slow client bloqueando runtime:** independent bounded writers.
- **Human decision required:** enterprise identity/pairing e remote modes.
