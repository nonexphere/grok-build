# Goal Runtime — SPECS

## 1. Version selection

Target futuro: `goal_runtime = "disabled" | "v1" | "v2"`; default/migration
somente no rollout epic. V1 e v2 não escrevem o mesmo authoritative state sem
dual-write contract e rollback testados.

## 2. V1 baseline

Characterization cobre slash, tool schema, snapshots/events, continuation,
completion/blocker, TUI/ACP/headless e recovery atuais. É a referência de
compatibilidade, não o desenho ideal.

## 3. V2 target

GoalRecord revisionado, state machine pura, SQLite/ledgers/CAS/leases,
completion audit por evidence, task DAG/subagents/worktrees e projections.

## 4. Integration port

Shell/App Server usa interface versionada; SessionActor permanece owner de
prompt/inference/cancel. Goal v2 não exige Tower/App Server daemon.

## 5. Validação

Seguir [TDD](../TDD.md): characterization antes do refactor, dual-version
fixtures, concurrency/crash/adversarial completion, rollback e ordinary-session regression.

