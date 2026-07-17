# Epic v1-runtime-continuation — Runtime e loop determinístico

Status: rascunho
Prioridade: lançamento-bloqueante
Depende de: `../v1-persistence-leases-accounting/`
Habilita: `v1-tools-verification`, `v1-task-graph-subagents`
Skills relacionadas: `@implementation-loop`, `@code-review`

## Arquitetura

Extrai `GoalService`/`GoalRuntime`, cria `GoalSessionPort` sobre `SessionActor`
e concentra continuation numa policy pura seguida por protocolo race-safe.

## Escopo

### ADICIONAR
- service registry/handles, callbacks de turn e continuation decisions;
- global resource governor e recovery entrypoint.

### REFACTORIZAR
- lógica distribuída em `acp_session_impl/goal.rs` migra para runtime.

### REMOVER
- starts sintéticos fora da policy/lease/intent gate após compat rollout.

### MANTÉM
- prompt queue, inference, cancellation e compaction no SessionActor.

## Business rules

- user command vence synthetic continuation;
- decisão pura não produz side effects;
- start revalida revision, lease, session idle e queued input;
- budgets/no-progress/run caps são hard gates;
- um runtime não inicia trabalho enquanto recovery estiver inconclusivo.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- E2E determinístico continua um goal sob limits sem verifier.
- Pause/edit/clear/user-input races nunca iniciam Turn indevido.
- Ordinary sessions e compaction permanecem compatíveis e lazy.

## Riscos e incertezas

- **[HIGH][Confirmed] races com user input/cancel:** double turn — intent+CAS+final idle check.
- **[HIGH][Likely] blocking entre actor/runtime:** deadlock — portas unidirecionais e testes determinísticos.
- **UNVERIFIED:** hook exato para compaction usage e scheduler governor.
