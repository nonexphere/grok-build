# Tasks — v2-03-runtime-continuation

## Service e session port
- [ ] Definir `GoalService`, runtime handle e registry por GoalId/session — Follow @implementation-loop
- [ ] Implementar `GoalSessionPort` sem expor actor internals
- [ ] Ligar turn start/end/cancel e provider usage callbacks
- [ ] Garantir lazy init para ordinary sessions

## Policy pura
- [ ] Implementar decision input/output completo
- [ ] Codificar ordem normativa Stop/Pause/Wait/Verify/Continue
- [ ] Implementar budgets, no-progress, queued-input e permission gates
- [ ] Table/property tests para cada razão e precedence

## Start protocol
- [ ] Persistir continuation intent sob lease
- [ ] Revalidar revision/lease/idle/queue antes de start
- [ ] Resolver intent em started/deferred/cancelled
- [ ] Deduplicar callbacks e repeated actor notifications

## Recovery e governor
- [ ] Entrar em Recovering antes de reconciliar
- [ ] Implementar process-level concurrency limits e fairness
- [ ] Contabilizar active/wall time em pause/cancel/restart
- [ ] Integrar compaction checkpoint sem perder goal state

## Race tests
- [ ] pause vs continuation
- [ ] user prompt between decision/start
- [ ] edit/clear vs active turn completion
- [ ] lease expiry/takeover e duplicate callback

## Validação
- [ ] E2E determinístico com verifier disabled e hard limits
- [ ] Ordinary session regression suite
- [ ] `cargo fmt --check`, checks e focused tests

## Specs e docs
- [ ] Documentar policy decision table e sequence diagrams
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
Nenhuma tarefa operacional humana para este epic.
