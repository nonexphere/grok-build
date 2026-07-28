# Tasks — v2-01-domain-foundation

## Baseline observável
- [ ] Inventariar `goal_tracker`, `acp_session_impl/goal.rs`, tools, persistence, pager e testes — Follow @repository-exploration
- [ ] Registrar comandos e features reais para unit/integration tests
- [ ] Criar golden tests de slash commands, snapshots e `GoalUpdated`
- [ ] Criar race characterization de pause/continuation e deferred completion

## ADRs e vocabulário
- [ ] Decidir storage location e ownership
- [ ] Decidir interactive restart policy
- [ ] Fixar status, phase, origin, revision e error enums
- [ ] Documentar legacy mappings e rollback boundary

## Domínio puro
- [ ] Criar IDs/records/commands/results v2
- [ ] Implementar transition table sem I/O
- [ ] Implementar edit invalidation e stale-result rejection
- [ ] Implementar blocker/no-progress fingerprints
- [ ] Adaptar testes existentes sem remover assertions

## Provas
- [ ] Property tests de sequências legais/ilegais
- [ ] Serde round-trip e unknown enum recovery
- [ ] Provar `Complete` inalcançável sem verifier report atual
- [ ] Provar model-origin sem administrative transitions

## Validação
- [ ] Rodar suites focadas determinadas no baseline
- [ ] `cargo fmt --check` e `cargo check` nos crates tocados
- [ ] Revisar diff contra golden baseline — Follow @code-review

## Specs e docs
- [ ] Atualizar `goal-runtime/SPECS.md` com ADRs aceitos
- [ ] Atualizar índice/status root sem marcar concluído antes dos gates

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar session-local vs global SQLite — type: product-decision — blocking: persistence epic
- [ ] (HUMAN) Aprovar auto-resume interativo — type: product-decision — blocking: recovery epic
