# Tasks — v1-task-graph-subagents

## Planner contract e DAG
- [ ] Definir schema versionado para tasks/deps/paths/role/output/acceptance
- [ ] Validar IDs, ciclos, órfãos, readiness e revision binding
- [ ] Persistir task graph e plan projection atomicamente
- [ ] Testar malformed/adversarial planner output

## Scheduler e budgets
- [ ] Selecionar ready tasks com global/per-goal concurrency limits
- [ ] Alocar token/time/cost slices e verifier reserve
- [ ] Implementar cancellation/dependency failure/retry limits
- [ ] Persistir spawn intent antes de criar child

## Worktree lifecycle
- [ ] Criar branch/path determinísticos e registrar baseline
- [ ] Bloquear writer shared-workspace por default
- [ ] Implementar clean apply com hunk/baseline validation
- [ ] Preservar conflict/failed cleanup e expor recovery action

## Acceptance e integração
- [ ] Executar task verifier plan antes de accepted
- [ ] Distinguir produced/accepted/applied/integrated
- [ ] Rejeitar late stale child mutation mantendo usage/report
- [ ] Atualizar parent requirements somente após integração comprovada

## Recovery/races
- [ ] crash record-before-spawn e spawn-before-ack
- [ ] pause/edit/clear com children ativos
- [ ] two writers disjoint success e conflicting pair preservation
- [ ] orphan process/worktree reconciliation

## Validação
- [ ] E2E de dois writers e research child com accounting completo
- [ ] Path/symlink/security tests
- [ ] Focused + integration + workspace tests

## Specs e docs
- [ ] Documentar planner envelope e worktree runbook
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar política de clean auto-apply — type: product-decision — blocking: stable auto-apply
