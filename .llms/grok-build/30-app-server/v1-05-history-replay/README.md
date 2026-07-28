# Epic v1-05 — History, projection e replay

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–4 semanas
Depende de: `../v1-03-core-in-process-stdio/`
Habilita: `../v1-07-release-hardening/`, `../../60-sdk-typescript/v1-01-generated-sdk-client-examples/`
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Cria SQLite rebuildable projection, incremental ingestion, event journal,
snapshot-then-live, pagination, fork/rewind e supported rebuild/verify tools.
Session files permanecem autoritativos.

## Escopo

### ADICIONAR
- projection schema/migrations/offsets/epochs/cursors;
- active-turn journal e recovery; archive/delete/fork/rewind projections.

### REFACTORIZAR
- session list/read usam projection sem mudar session persistence.

### REMOVER
- nenhuma session artifact source.

### MANTÉM
- existing sessions/forks/subagents readable.

## Business rules

- rebuild é idempotente e preserva exposed IDs;
- cursor vincula session/history epoch/query;
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
- **[MEDIUM][Likely] scope creep de FTS/delete:** manter fora do MUST se não requerido por tools.
- **UNVERIFIED:** delta durability final até medir session artifacts reais.
