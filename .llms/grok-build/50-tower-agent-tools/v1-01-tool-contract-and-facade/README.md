# Epic v1-01 — Contrato e facade `tower_agent_*`
Owner: Tower tools owners
Escopo: conforme a seção Escopo deste epic

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
Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: `../../20-tower-core/v1-02-multi-session-workspace-registry/`, `../../30-app-server/v1-03-core-in-process-stdio/`
Habilita: MCP server e `v1-02`
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Cria Rust tool facade e schemas a partir do contrato compartilhado, com
Session/Turn semantics, errors, pagination e idempotência únicos.

## Escopo

### ADICIONAR
- list/start/send/history/interrupt/resume/archive/status/wait typed operations.

### REFACTORIZAR
- roster/session handlers existentes como primitives atrás da facade.

### REMOVER
- conceito/tool `hub` separado.

### MANTÉM
- subagent tools atuais e runtime permissions.

## Contratos

- [Tower tools](../../_shared/tower-agent-tools.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [control-plane security](../../_shared/control-plane-security.md)

## TODO checklist

- [ ] RED contract test para cada MUST operation
- [ ] Definir params/results/errors/idempotency/cursors
- [ ] Implementar facade sobre registry/App Server primitives
- [ ] History full/last + cursor/max bytes/redaction
- [ ] Wait subscription/timeout/cancel sem lock leak
- [ ] Start workspace/agent type e resume dormant
- [ ] Interrupt/archive truth/idempotency
- [ ] Schema snapshots e malformed/oversize tests
- [ ] Concurrency duplicate send/start/wait tests
- [ ] Docs/examples e drift gate

## Riscos e incertezas

- **[HIGH][Confirmed] duplicate effects:** idempotency + actor state gates.
- **[HIGH][Likely] history secret/size leak:** projection redaction + byte limits.
- **[MEDIUM][Possible] wait task leak:** cancellation-safe subscriptions.
