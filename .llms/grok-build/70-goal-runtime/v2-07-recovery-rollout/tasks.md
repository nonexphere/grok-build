# Tasks — v2-07-recovery-rollout

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
- [ ] Implementar seleção `disabled|v1|v2` e validação de config
- [ ] Implementar dual-read/projection sem duas authorities concorrentes
- [ ] Definir telemetry thresholds e rollback triggers
- [ ] Testar rollback v2→v1 em cada rollout stage
- [ ] Preservar v1 selecionável após sign-off conforme contrato

## Validação
- [ ] Executar full Goal Runtime matrix e ordinary-session regression
- [ ] Auditoria independente de completion/security/recovery — Follow @code-audit
- [ ] Produzir release readiness e delivery evidence

## Specs e docs
- [ ] User/migration/troubleshooting/recovery/accounting docs
- [ ] Atualizar todos status somente com evidência

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar rollout thresholds e rollback triggers — type: product-decision — blocking: stable enablement
- [ ] (HUMAN) Escolher versão default após opt-in evidence — type: product-decision — blocking: default switch
- [ ] (HUMAN) Executar release sign-off — type: manual-verify — blocking: concluir epic
