# Epic v2-06 — Clients e projections v2
Owner: goal runtime owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento core
Estimativa: 2–4 semanas
Depende de: `../v2-04-tools-verification/`, `../v2-05-task-graph-subagents/`
Habilita: `../v2-07-recovery-rollout/`; integração App Server pós-core
Skills relacionadas: `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Publica `GoalUpdatedV2`/events e comandos completos para slash, pager, ACP e
headless. A UI é uma projeção do runtime e não muta state localmente.

## Escopo

### ADICIONAR
- edit/budget/audit/events/report commands;
- dashboard requirement/task/subagent/evidence/budget/verifier;
- headless lifecycle events e deterministic exit codes;
- `GoalService` projection consumível pelo App Server.

### REFACTORIZAR
- goal modal e tasks pane passam a consumir state/event versionados.

### REMOVER
- inferência de lifecycle a partir de texto/transcript.

### MANTÉM
- old pager/ACP fields additive durante compat window.

## Business rules

- user lifecycle commands têm optimistic concurrency e feedback explícito;
- pause impede novo start imediatamente;
- dashboard distingue proven/incomplete/missing/blocked/infra;
- headless exit só é success após runtime completion report.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Create→pause→edit→resume→verify→complete funciona em TUI e headless.
- Old pager/ACP snapshots passam; eventos não vazam secrets.
- Goal facade fixtures permitem projeção App Server sem lifecycle coupling.

## Riscos e incertezas

- **[HIGH][Confirmed] compat UI/wire:** old clients — additive fields, snapshots e rollback.
- **[MEDIUM][Likely] dashboard overload:** progressive disclosure e render benchmarks.
- **UNVERIFIED:** forma final de Goal Item no protocolo App Server.
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
