# Epic v1-06 — Interações e controller lease
Owner: App Server/protocol owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: `../v1-03-core-in-process-stdio/`
Habilita: `../v1-07-release-hardening/`
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

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
- [security/authority](../../_shared/control-plane-security.md)
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
- **[HIGH][Likely] headless approval sem controller:** fail-closed com policy explícita.
- **UNVERIFIED:** persistent `always` grant remoto; deixar desabilitado até decisão.
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
