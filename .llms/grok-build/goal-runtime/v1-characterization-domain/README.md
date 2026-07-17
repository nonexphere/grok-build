# Epic v1-characterization-domain — Baseline e domínio puro

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: nenhuma  
Habilita: `v1-persistence-leases-accounting`, `v1-architecture-protocol` (vocabulário compartilhado)  
Skills relacionadas: `@repository-exploration`, `@architecture-spec-authoring`, `@implementation-loop`

## Arquitetura

Caracteriza o `/goal` existente antes do refactor e extrai IDs, records,
status/phase, commands e transition engine puros. Define a fronteira
model/runtime e trava ADRs que alteram storage ou UX.

## Escopo

### ADICIONAR
- domain types v2, transition table, revisions e property tests;
- golden snapshots do comportamento atual e decision log.

### REFACTORIZAR
- separar state machine persistível do estado transitório em `GoalTracker`.

### REMOVER
- nenhuma behavior nesta fase; incompatibilidades são adapters explícitos.

### MANTÉM
- slash syntax, snapshots, ordinary sessions e eventos legados caracterizados.

## Business rules

- lifecycle status e execution phase são campos distintos;
- model-origin nunca pausa, resume, edita, limpa ou completa diretamente;
- estado desconhecido restaura não-driving;
- edit incrementa revision e invalida resultados stale.

## Contratos

- [goal domain v1](./contracts/goal-domain-v1.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [security/authority](../../_shared/security-authority-boundaries.md)

## Plano de execução

1. Mapear entrypoints/state/tests atuais.
2. Aprovar ADRs e matrizes de compatibilidade.
3. Escrever characterization/golden tests.
4. Extrair domínio puro atrás de adapter.
5. Executar property/serde/transition tests.

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Characterization cobre todos os entrypoints atuais e está verde.
- Transition properties provam authority, completion e stale-result invariants.
- ADRs de storage/restart estão aceitos e domínio v2 não exige I/O.

## Riscos e incertezas

- **[HIGH][Confirmed] refactor de comportamento maduro:** regressões silenciosas — mitigar com characterization antes de mudança.
- **[HIGH][Likely] status legacy ambíguo:** import pode dirigir execução indevida — unknown/ambiguous restaura paused.
- **UNVERIFIED:** comandos exatos e feature flags de teste do baseline.
- **Human decision required:** localização do SQLite e política de auto-resume interativo. [provenance: doc-tree]
