# Epic v2-04 — Tools v2 e completion audit
Owner: goal runtime owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento core
Estimativa: 2–4 semanas
Depende de: `../v2-03-runtime-continuation/`
Habilita: `../v2-05-task-graph-subagents/`, `../v2-06-clients-projections/`
Skills relacionadas: `@implementation-loop`, `@session-evidence-gate`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Entrega tools v2, prompt protocol e registry extensível de verifiers. Completion
request inicia audit; somente report conclusivo atual autoriza `Complete`.

## Escopo

### ADICIONAR
- `get_goal`, action-based `update_goal`, schemas e acknowledgements;
- verifier registry/composites/evidence providers e completion/block audit.

### REFACTORIZAR
- skeptic panel vira verifier plugin fail-closed.

### REMOVER
- model-driven terminal mutation e fail-open infra behavior.

### MANTÉM
- legacy tool schema via telemetry-backed compatibility adapter.

## Business rules

- cada required requirement/deliverable precisa evidence conclusiva atual;
- verifier é read-only e stale report não completa;
- infrastructure error causa `InfraPaused`;
- MCP/skill output é advisory salvo configuração authoritative explícita;
- blocker exige mesma condição repetida e fingerprint runtime-owned.

## Contratos

- [security/authority](../../_shared/control-plane-security.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- False completion é rejeitada em unit, integration e adversarial suites.
- True completion exige requirement matrix e report conclusivo current-revision.
- Infra failure pausa; tool/model nunca administra lifecycle.

## Riscos e incertezas

- **[HIGH][Confirmed] false completion:** dano central — deterministic verifiers + audit matrix + fail-closed.
- **[MEDIUM][Likely] verifier caro/falso negativo:** loops — bounded retries e actionable gaps.
- **Human decision required:** trust de MCP e verificação visual defaults.
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
