# Epic v1-persistence-leases-accounting — Estado transacional

Status: rascunho
Prioridade: lançamento-bloqueante
Depende de: `../v1-characterization-domain/`
Habilita: `v1-runtime-continuation`
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`, `@code-review`

## Arquitetura

Adiciona SQLite/WAL, migrations, CAS, append-only ledgers, execution leases e
import legacy. O store é process-agnostic e não conhece UI/model loop.

## Escopo

### ADICIONAR
- materialized state, event/usage/evidence/verifier/subagent ledgers;
- lease manager, intents e recovery metadata;
- legacy importer e dual projection.

### REFACTORIZAR
- persistence snapshot vira projection; usage deixa mutable baseline.

### REMOVER
- escrita não-versionada como lifecycle truth após rollout.

### MANTÉM
- session JSONL/GoalUpdated legíveis e rollback feature flag.

## Business rules

- mutation e event commit na mesma transação;
- CAS por record revision; usage idempotente por provider-call ID;
- lease epoch faz fencing; ambiguous recovery não dirige;
- budget arithmetic é overflow-safe e custo incompleto é explícito.

## Contratos

- [leases/idempotency](../../_shared/leases-idempotency.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Duas conexões/processos não criam a mesma continuação/effect intent.
- Crash em cada boundary converge sem perda, duplicação ou driving ambíguo.
- Legacy import, CAS, ledgers e budget accounting passam fixtures/race tests.

## Riscos e incertezas

- **[HIGH][Confirmed] duas fontes durante migração:** divergência — SQLite é truth e JSONL projection com comparison telemetry.
- **[HIGH][Likely] crash boundary incompleta:** duplicação de side effect — intent/resolution e kill tests.
- **UNVERIFIED:** disponibilidade/padrão SQLite já usado no workspace e cross-platform locking.
