# Epic v1-04 — WebSocket early e bearer remoto

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: `../v1-03-core-in-process-stdio/`, `../../20-tower-core/v1-03-multi-instance-daemon-modes/`
Habilita: `../../40-mcp-control-plane/v1-01-server-transports/`, `../../60-sdk-typescript/v1-01-generated-sdk-client-examples/`
Skills relacionadas: `@implementation-loop`, `@code-audit`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Conecta WebSocket ao processor comum assim que o vertical slice existe e aplica
o bearer/threat model compartilhado. Daemon lifecycle pertence à Tower; este
epic possui WS handshake, connection/auth e conformance.

## Escopo

### ADICIONAR
- WebSocket acceptor, bearer validation, connection lifecycle e limits;
- bounded queues, overload policy e graceful drain/restart.

### REFACTORIZAR
- auth/redaction core é compartilhável com MCP HTTP.

### REMOVER
- qualquer requirement de scopes finos, TLS ou Origin allowlist no MVP.

### MANTÉM
- ACP capability/transport por adapter e rollout flags.

## Business rules

- todos transportes passam a mesma black-box suite;
- loopback bind default; non-loopback explícito; bearer full-control; `ws://` permitido;
- sem scopes finos ou Origin allowlist no MVP, com warning/threat docs;
- lifecycle events não são descartados; delta pode coalescer;
- shutdown resolve/drains truthfully, sem alegar Turn completo.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/control-plane-security.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Mesma black-box conformance passa in-process, stdio e WebSocket.
- Threat/load/fault/cross-platform tests fecham os controls obrigatórios.
- Graceful restart e slow clients não perdem lifecycle nem bloqueiam runtime.

## Riscos e incertezas

- **[HIGH][Confirmed] remote attack surface:** deny-by-default + threat tests.
- **[HIGH][Confirmed] slow client bloqueando runtime:** independent bounded writers.
- **[HIGH][Confirmed] cleartext remote full-control:** accepted MVP tradeoff; warning/redaction/rotation.
- **Human decision required:** release threat-model acceptance, não design — type: manual-verify.
