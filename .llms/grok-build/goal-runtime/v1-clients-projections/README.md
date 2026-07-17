# Epic v1-clients-projections — UX, ACP e headless

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: `../v1-tools-verification/`, `../v1-task-graph-subagents/`  
Habilita: `v1-recovery-rollout`, `../../app-server/v1-ecosystem-ga/`  
Skills relacionadas: `@implementation-loop`, `@code-review`

## Arquitetura

Publica `GoalUpdatedV2`/events e comandos completos para slash, pager, ACP e
headless. A UI é uma projeção do runtime e não muta state localmente.

## Escopo

### ADICIONAR
- edit/budget/audit/events/report commands;
- dashboard requirement/task/subagent/evidence/budget/verifier;
- headless lifecycle events e deterministic exit codes;
- `GoalService` projection consumível pelo App Server.

### REFACTORIZAR
- goal modal e tasks pane passam a consumir state/event versionados.

### REMOVER
- inferência de lifecycle a partir de texto/transcript.

### MANTÉM
- old pager/ACP fields additive durante compat window.

## Business rules

- user lifecycle commands têm optimistic concurrency e feedback explícito;
- pause impede novo start imediatamente;
- dashboard distingue proven/incomplete/missing/blocked/infra;
- headless exit só é success após runtime completion report.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Create→pause→edit→resume→verify→complete funciona em TUI e headless.
- Old pager/ACP snapshots passam; eventos não vazam secrets.
- Goal facade fixtures permitem projeção App Server sem lifecycle coupling.

## Riscos e incertezas

- **[HIGH][Confirmed] compat UI/wire:** old clients — additive fields, snapshots e rollback.
- **[MEDIUM][Likely] dashboard overload:** progressive disclosure e render benchmarks.
- **UNVERIFIED:** forma final de Goal Item no protocolo App Server.
