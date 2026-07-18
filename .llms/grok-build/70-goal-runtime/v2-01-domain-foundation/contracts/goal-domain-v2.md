# Goal Domain v2

**Fonte de verdade.** Este contrato target pertence ao Goal Runtime v2. Deriva da
especificação normativa em `changes/grok-build-goal-runtime-technical-spec (1).md`.

## Aggregate

`GoalRecord` contém identidade, session binding, status, phase, objective e
contract revisions, record revision, budget, execution/verification summaries,
timestamps e pause/terminal reason. Históricos detalhados vivem em ledgers.

## Lifecycle status

```text
Absent → Active
Active → UserPaused | BackoffPaused | NoProgressPaused | InfraPaused
Active → Blocked | BudgetLimited | UsageLimited | Complete
paused/blocked/limited → Active por comando autorizado
non-absent → Absent por clear autorizado
Complete → Active somente por replacement goal
```

`Planning`, `Executing`, `Verifying` e `Recovering` são phases. Phase diferente
de `Idle` exige status `Active`. Unknown restore vira `UserPaused/Recovering`.

## Authority

| Origin | Commands aceitos |
|---|---|
| user/admin facade | set, pause, resume, clear, edit, budget, audit |
| runtime | round/lease/recovery/budget/verifier transitions |
| model tool | progress report, completion request, repeated blocker report |
| subagent/verifier | scoped result/evidence; nunca parent lifecycle admin |

## Revisions

- `record_revision` incrementa em toda mutation;
- `objective_revision` incrementa em edit de objetivo;
- `contract_revision` incrementa em alteração do contrato;
- plan, task result, evidence e verifier result carregam revisions alvo;
- stale result é armazenável para audit/usage, mas não muta materialized state.

## Goal contract

O contrato possui title/objective, assumptions, constraints, requirements,
deliverables, verifier plans e completion rule. Requirement tem ID estável,
source, criticality, scope, dependencies e verifier plan. `all_required` exige
todos os required requirements e deliverables conclusivamente provados.

## Completion

1. model envia completion request;
2. runtime captura revisions e cria verifier attempt;
3. verifiers registram bounded evidence e requirement outcomes;
4. infra error transiciona para `InfraPaused`;
5. stale report é audit-only;
6. apenas report atual com todos required `proven` autoriza CAS para Complete.

Outcomes mínimos: `proven`, `contradicted`, `incomplete`, `inconclusive`,
`missing_evidence`, `blocked`. Prosa do modelo/conversa nunca é evidence
autoritativa por si só.

## Blocked

O modelo reporta descrição/categoria/requirements, mas o runtime normaliza o
fingerprint. Só a mesma condição externa/contraditória repetida pelo threshold
configurado pode levar a `Blocked`. Dificuldade, incerteza, lentidão, orçamento
baixo ou trabalho incompleto não são blockers.

## Budgets

Tokens, active time, wall time, cost, turns, verifier attempts, no-progress e
concurrency podem ser limitados. Ausência de default preserva comportamento
ilimitado compatível. Hard budget usa accounting durável e reserva de
verificação; accounting incompleto nunca é tratado como zero certo.

## Events

Todo transition produz evento com IDs, origin, command/decision, revisions,
lease epoch quando aplicável e timestamp. Objective/evidence payload não entra
em logs por default. Projeções podem omitir detalhes, mas não mudar semantics.

## Error classes

| Classe | Efeito |
|---|---|
| stale revision/lease | rejeitar sem mutation |
| invalid transition/input | rejeitar com erro tipado |
| required infra unavailable | pause fail-closed |
| budget/usage reached | estado limited correspondente |
| repeated runtime failure | backoff pause |
| corrupt/unknown restore | paused/recovering + diagnóstico |

## Compatibility

Legacy snapshot/tool/event/slash são adaptados nas bordas. Legacy complete não
é reaberto automaticamente e recebe `legacy_completion`; audit explícito pode
reverificá-lo. Ordinary non-goal sessions não inicializam runtime pesado.

## Acceptance invariants

- model-origin nunca produz administrative transition;
- `Complete` sempre referencia report conclusivo da revisão atual;
- record revision cresce estritamente;
- stale results não mudam state;
- user command preempts synthetic continuation;
- somente um non-terminal goal existe por session no v2 inicial.
