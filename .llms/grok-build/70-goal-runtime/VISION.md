# Goal Runtime — VISION

## Project Role

Evoluir goals long-running sem quebrar o comportamento atual: v1 permanece
selecionável enquanto v2 aprende a continuar, verificar e recuperar com estado
transacional e autoridade fora do modelo.

## Evolução

1. v1: baseline legado caracterizado e isolado atrás de port.
2. v2: runtime transacional completo, opt-in por flag.
3. rollout: dual-read/projection, migration e rollback antes do default switch.
4. v3+: multi-goal/scheduling somente após v2 estável.

## Design Principles

1. Compatibilidade explícita e limitada ao Goal.
2. Completion exige evidence atual.
3. Recovery ambígua é non-driving.
4. Modelo expressa intent; runtime administra lifecycle.
5. Goal v2 não bloqueia Tower MVP.

## Out of Scope

Dual-version de App Server/MCP/Tower, daemon obrigatório e nested subagent trees.

