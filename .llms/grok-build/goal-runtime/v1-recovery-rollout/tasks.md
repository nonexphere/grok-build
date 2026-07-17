# Tasks — v1-recovery-rollout

## Recovery matrix
- [ ] Reconciliar live primary turn e continuation intent
- [ ] Reconciliar child process/session/worktree e apply state
- [ ] Reconciliar verifier report/intent e completion CAS
- [ ] Reconciliar provider usage before/after ledger insert
- [ ] Restaurar unknown/corrupt/foreign state non-driving

## Migration fixtures
- [ ] Importar anonymized sessions: active/paused/blocked/complete/corrupt-tail
- [ ] Comparar SQLite vs legacy projections por event/revision
- [ ] Implementar quarantine/diagnostic para import failure
- [ ] Provar `/goal clear --force` recovery path com confirmação

## Hardening
- [ ] Fuzz JSON/TOML/SQLite state e tool schemas
- [ ] Security tests de path, prompt injection, extensions e redaction
- [ ] Load/performance de 100 records/10 active/10k events
- [ ] Cross-platform restart/process/path suite

## Rollout e rollback
- [ ] Implementar flags por dual-read/write/runtime/UI
- [ ] Definir telemetry thresholds e rollback triggers
- [ ] Testar rollback em cada rollout stage
- [ ] Remover legacy authority somente após sign-off separado

## Validação
- [ ] Executar full Goal Runtime matrix e ordinary-session regression
- [ ] Auditoria independente de completion/security/recovery — Follow @code-audit
- [ ] Produzir release readiness e delivery evidence

## Specs e docs
- [ ] User/migration/troubleshooting/recovery/accounting docs
- [ ] Atualizar todos status somente com evidência

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar rollout thresholds e rollback triggers — type: product-decision — blocking: stable enablement
- [ ] (HUMAN) Executar release sign-off — type: manual-verify — blocking: concluir epic
