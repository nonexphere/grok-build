# Tasks — v1-05-history-replay

## Projection schema
- [ ] Criar Session/Turn/Item/event/source-offset/history-epoch tables — Follow @implementation-loop
- [ ] Criar migrations/indexes/constraints e corruption behavior
- [ ] Marcar DB explicitamente rebuildable/non-authoritative
- [ ] Implementar verify/rebuild CLI APIs

## Ingestion
- [ ] Scan existing session tree incrementalmente
- [ ] Persistir byte offsets/file identity e handle rotation/truncation
- [ ] Tolerar corrupt JSONL tail preservando valid prefix
- [ ] Coalescer journal deltas conforme durability ADR

## Replay e pagination
- [ ] Implementar watermark snapshot-then-live
- [ ] Criar opaque scoped cursors e invalidation
- [ ] Implementar session/list/read e item pagination
- [ ] Testar replay+live equivalence e no duplicates

## Fork/rewind/archive
- [ ] Projetar fork parent/relation/worktree sem duplicar actor
- [ ] Incrementar history epoch no rewind
- [ ] Implementar archive metadata e physical policy separadamente
- [ ] Manter hard delete experimental/policy-gated

## Recovery e fault tests
- [ ] crash during delta/projection commit/rebuild
- [ ] locked DB/full disk/corrupt tail/partial fork
- [ ] restore active Turn journal truthfully
- [ ] rebuild 10k-item fixture com stable IDs

## Validação
- [ ] Property/golden/fault/performance tests
- [ ] Projection verify após rebuild real fixture
- [ ] Focused checks and review

## Specs e docs
- [ ] Schema/cursor/rebuild/runbook docs
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar delta durability policy — type: product-decision — blocking: journal stable semantics
- [ ] (HUMAN) Aprovar archive/delete/FTS ownership — type: product-decision — blocking: stable methods
