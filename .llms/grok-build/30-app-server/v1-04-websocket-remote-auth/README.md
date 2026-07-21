# Epic v1-04 — WebSocket early e bearer remoto
Owner: App Server/protocol owners
Escopo: conforme a seção Escopo deste epic

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
## Revisão de implementação

Este epic só pode ser executado quando cada task tiver owner, arquivos ou
contrato afetado, pré-condição, comando de validação e evidência esperada.
Alterações de comportamento exigem Red-Green-Refactor; alterações de contrato
exigem contract test e atualização da matriz de rastreabilidade.

### Gate mínimo

- [ ] dependências e links deste epic foram verificados;
- [ ] interfaces, schemas, estados, erros e compatibilidade estão definidos;
- [ ] caminho fake/conformance está separado do caminho product-backed;
- [ ] testes unitários, integração, black-box e segurança foram classificados;
- [ ] timeout, cancelamento, retry, restart e falhas parciais foram tratados;
- [ ] observabilidade, limites de recurso e redaction foram especificados;
- [ ] comando reproduzível e artefato de evidência foram registrados;
- [ ] bloqueios humanos/externos possuem owner e condição de desbloqueio;
- [ ] status do epic foi reconciliado com `TRACEABILITY.md` e `COMPLETION_COVERAGE.md`.
