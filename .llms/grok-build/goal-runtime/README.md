# Goal Runtime

## O que é

Runtime Rust transacional que transforma o `/goal` existente em objetivo
persistente, verificável, recuperável e controlado pelo harness.

## Papel

Possui lifecycle, contrato, budgets, evidence, continuation policy,
verification e task graph. Delega inferência/ferramentas ao runtime Grok e
publica projeções para TUI, ACP, headless e App Server.

## Stack

- Rust/Tokio nos crates `xai-grok-shell`, `xai-grok-tools*`,
  `xai-grok-workspace`, `xai-grok-pager*`
- SQLite/WAL para estado, eventos e ledgers
- JSON/JSONL e ACP como projeções compatíveis

## Estado atual

O baseline já tem `GoalTracker`, comandos slash, `update_goal`, continuação,
planner, skeptic panel, subagents/worktrees, persistência de snapshot e UI.
Essas capacidades estão acopladas ao `SessionActor`, usam semantics antigas e
não satisfazem os invariantes transacionais/fail-closed da nova especificação.

## Issues conhecidos

- modelo e runtime ainda não têm fronteira de autoridade v2 completa;
- completion não possui requirement/evidence audit rigoroso;
- persistence não é o ledger SQLite/CAS especificado;
- continuation/recovery precisam de leases e deduplicação duráveis;
- accounting de parent/child/verifier é incompleto;
- goal/task/subagent state ainda não tem projeção estável para app-server;
- decisões OQ1–OQ11 da especificação permanecem abertas.

## Epics

- [v1-characterization-domain](./v1-characterization-domain/)
- [v1-persistence-leases-accounting](./v1-persistence-leases-accounting/)
- [v1-runtime-continuation](./v1-runtime-continuation/)
- [v1-tools-verification](./v1-tools-verification/)
- [v1-task-graph-subagents](./v1-task-graph-subagents/)
- [v1-clients-projections](./v1-clients-projections/)
- [v1-recovery-rollout](./v1-recovery-rollout/)

## Contratos

- [runtime ownership](../_shared/runtime-ownership.md)
- [identity/event ordering](../_shared/identity-event-ordering.md)
- [leases/idempotency](../_shared/leases-idempotency.md)
- [security/authority](../_shared/security-authority-boundaries.md)
