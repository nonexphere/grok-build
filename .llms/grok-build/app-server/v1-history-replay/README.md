# Epic v1-history-replay — Projection store e continuidade

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: `../v1-core-in-process/`  
Habilita: `v1-daemon-transports-security`  
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`, `@code-review`

## Arquitetura

Cria SQLite rebuildable projection, incremental ingestion, event journal,
snapshot-then-live, pagination, fork/rewind e supported rebuild/verify tools.
Session files permanecem autoritativos.

## Escopo

### ADICIONAR
- projection schema/migrations/offsets/epochs/cursors;
- active-turn journal e recovery; archive/delete/fork/rewind projections.

### REFACTORIZAR
- thread list/read usam projection sem mudar session persistence.

### REMOVER
- nenhuma session artifact source.

### MANTÉM
- existing sessions/forks/subagents readable.

## Business rules

- rebuild é idempotente e preserva exposed IDs;
- cursor vincula thread/history epoch/query;
- corrupt tail não apaga prefixo válido;
- replay watermark elimina gap/duplicação com live;
- hard delete permanece experimental até decisão.

## Contratos

- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Replay+live equivale a live contínuo sem gap/duplicate.
- Rebuild preserva IDs e recupera DB corrupta a partir das fontes.
- Pagination/fork/rewind/crash fixtures e performance target passam.

## Riscos e incertezas

- **[HIGH][Confirmed] projection drift:** rebuild + offsets + comparison checks.
- **[HIGH][Likely] replay/live race:** watermark buffering/property tests.
- **Human decision required:** delta durability, archive/delete semantics e FTS ownership.
