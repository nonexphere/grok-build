# 70 — Goal Runtime

## O que é

Programa futuro que preserva o `/goal` atual como **v1 legado**, caracteriza
seu comportamento e implementa um Goal Runtime transacional **v2** selecionável
por flags. Não pertence à critical path Tower/App Server/MCP.

## Estado atual

Goal v1 já existe em `GoalTracker`, slash/tools, continuation, persistence e
TUI. A especificação em `changes/grok-build-goal-runtime-technical-spec (1).md`
descreve o target v2, ainda não implementado.

## Issues conhecidos

- draft anterior chamava o redesign de v1, contradizendo a intenção humana;
- hot paths do App Server ainda precisam de inventário para port versionada;
- storage, rollout e verifier decisions permanecem abertas para a execução futura.

## Epics

- [v1-01-legacy-characterization](./v1-01-legacy-characterization/) — congelar v1
- [v2-01-domain-foundation](./v2-01-domain-foundation/)
- [v2-02-persistence-leases-accounting](./v2-02-persistence-leases-accounting/)
- [v2-03-runtime-continuation](./v2-03-runtime-continuation/)
- [v2-04-tools-verification](./v2-04-tools-verification/)
- [v2-05-task-graph-subagents](./v2-05-task-graph-subagents/)
- [v2-06-clients-projections](./v2-06-clients-projections/)
- [v2-07-recovery-rollout](./v2-07-recovery-rollout/)

## Relação com o core

App Server v1 somente inventaria hot paths e define port que futuramente aceita
`disabled|v1|v2`. Nenhum epic core depende de Goal v2. Retrocompat/dual-version
é obrigatória **somente** neste programa. [provenance: user-input]

