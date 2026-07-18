# Tower Agent Tools — VISION

## Project Role

Permitir ao orchestrator operar sessions top-level reais, superando o limite
de profundidade dos subagents sem remover o mecanismo atual.

## Design Principles

1. Contract-first e transport invariant.
2. Least privilege por agent type.
3. Session top-level não se disfarça de subagent.
4. History é bounded e redigida.

## Out of Scope

Peer messaging ad hoc no v1 e bypass de approvals/sandbox.

