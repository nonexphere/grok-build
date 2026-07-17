# Goal Runtime — SPECS

Referência normativa detalhada: `changes/grok-build-goal-runtime-technical-spec (1).md`.
Os épicos preservam os MUST/MUST NOT dessa fonte e registram qualquer mudança
como decisão humana, nunca como simplificação silenciosa.

## 1. Domínio

- um goal não terminal por sessão no v1;
- lifecycle status separado de execution phase;
- revisions de record/objective/contract;
- GoalContract com requirements, deliverables e verifier plans;
- `Complete` requer report conclusivo da revisão atual.

## 2. Persistência

SQLite materializa state e mantém ledgers append-only para eventos, usage,
evidence, verifier e subagent. Mutations usam CAS e idempotency. JSONL,
`GoalUpdatedV2` e UI são projeções.

## 3. Continuação

Uma função pura decide Stop/Pause/Wait/Verify/Continue com base em lifecycle,
idle state, user input, lease, budgets, progress, verifiers, subagents,
permissions e recovery. Start usa intent durável e revalidação de idle/lease.

## 4. Tools e verificação

`get_goal` lê estado. `update_goal` aceita somente intents de progresso,
completion request e blocker repetido. Registry de verifiers suporta Rust,
command/test/artifact/skill/MCP/composite; erro de infraestrutura pausa.

## 5. Tasks, subagents e worktrees

Planner gera DAG durável. Writer task usa worktree isolado por padrão;
research/verifier são read-only. Parent integra e aceita resultados. Usage,
artifacts, baseline e apply status são rastreados.

## 6. Interfaces

Slash, TUI, ACP e headless controlam set/status/pause/resume/clear/edit/budget/
audit/events/report. O App Server consome serviço/eventos sem assumir lifecycle.

## 7. Validação

Unit/property, deterministic concurrency, SQLite races, crash injection,
adversarial prompts, migration fixtures, performance e security tests. Comandos
exatos devem ser confirmados no epic de caracterização.

## Infra

- DB: session-local por default recomendado; decisão humana no primeiro epic.
- Secrets: nenhum.
- Observability: métricas/traces com IDs e revisions, sem objective/evidence payload.
