# Tasks — v1-persistence-leases-accounting

## Schema e migrations
- [ ] Materializar schema da spec com FKs/indexes/constraints
- [ ] Criar migration runner transacional e schema-version guard
- [ ] Configurar WAL/busy timeout/durability conforme ADR
- [ ] Testar DB novo, upgrade, downgrade recusado e corruption handling

## Store e CAS
- [ ] Implementar trait store do domínio
- [ ] Commit record+event atomicamente
- [ ] Rejeitar stale record/objective/contract revisions
- [ ] Implementar idempotency-key result replay e payload conflict

## Ledgers e budgets
- [ ] Persistir usage por parent/subagent/verifier/planner/compaction scopes
- [ ] Persistir evidence/reports/artifacts com size/redaction metadata
- [ ] Implementar token/time/cost arithmetic e incomplete flags
- [ ] Testar cancellation/restart entre provider result e ledger insert

## Leases e intents
- [ ] Implementar acquire/heartbeat/release/takeover com epoch
- [ ] Persistir continuation/subagent/verifier intents
- [ ] Reconciliar expired/foreign/ambiguous leases em non-driving state
- [ ] Testar duas conexões/processes e CAS conflicts

## Migração
- [ ] Importar fixtures de snapshots legacy com status mapping
- [ ] Implementar dual projection e comparison telemetry
- [ ] Implementar rollback sem perder v1 legacy readability

## Validação
- [ ] Crash matrix em cada transaction boundary
- [ ] SQLite concurrency/load tests
- [ ] `cargo fmt --check`, focused tests e crate checks

## Specs e docs
- [ ] Documentar schema, retention e recovery runbook
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Confirmar storage location ADR — type: product-decision — blocking: schema paths/import
