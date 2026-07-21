# Epic v2-05 — Task DAG, subagents e worktrees v2
Owner: goal runtime owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento core
Estimativa: 2–4 semanas
Depende de: `../v2-03-runtime-continuation/`, `../v2-04-tools-verification/`
Habilita: `../v2-06-clients-projections/`
Skills relacionadas: `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Transforma planner output em task graph durável e integra scheduler,
subagents e worktrees existentes com ownership, budgets, acceptance e recovery.

## Escopo

### ADICIONAR
- planner JSON contract, DAG validation, task/subagent/worktree records;
- scheduler/resource governor, acceptance e integration state.

### REFACTORIZAR
- subagent coordination passa a reportar lifecycle/usage/artifacts ao goal.

### REMOVER
- writer concorrente no parent workspace por default e cleanup cego.

### MANTÉM
- one-level child depth e parent como integrador/autoridade.

## Business rules

- writer usa worktree próprio; research/verifier é read-only;
- task só conclui após acceptance atual e integração quando requerida;
- conflito preserva worktree e vira estado acionável;
- child não completa parent goal;
- revision/baseline stale invalida mutation, não accounting/audit.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/control-plane-security.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Dois writers disjuntos integram e são aceitos com accounting completo.
- Conflito é preservado e surfaced sem data loss.
- Crash/cancel/edit/restart não deixa child/worktree sem estado reconciliável.

## Riscos e incertezas

- **[HIGH][Confirmed] conflito/data loss:** clean-only apply e preservar árvores conflitantes.
- **[HIGH][Likely] budget explosion:** slices + governor + parent reserve.
- **Human decision required:** auto-apply apenas com paths disjuntos/acceptance configurada.
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
