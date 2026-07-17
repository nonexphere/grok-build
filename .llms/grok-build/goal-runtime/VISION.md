# Goal Runtime — VISION

## Project Role

Ser a autoridade process-agnostic para objetivos longos no Grok Build, capaz de
continuar, provar, pausar e recuperar execução sem delegar lifecycle ao modelo.

## Architecture Fit

O runtime fica entre comandos/model intents e o `SessionActor`. Ele usa os
subsystems existentes, preserva sessões comuns e fornece uma API lease-based
para futuros app-server e multi-goal schedulers.

## Evolução

1. **v1:** um goal ativo por sessão, runtime transacional, verificação,
   subagents/worktrees, TUI/ACP/headless e recovery.
2. **v2:** múltiplos goals selecionáveis por sessão e goal groups.
3. **v3+:** scheduling global, políticas organizacionais e verifier marketplace
   sem redesenhar domínio/storage.

## Design Principles

1. Runtime é autoridade; modelo expressa intents.
2. Completion é evidência, não texto.
3. Recovery é estado explícito, nunca adivinhação.
4. Accounting inclui toda computação atribuível.
5. Extensions não recebem trust implícito.

## Out of Scope

- daemon obrigatório;
- múltiplos goals ativos na mesma sessão v1;
- nested subagent trees arbitrárias;
- merge cego ou resolução automática de conflito;
- prova formal ou dependência do runtime Codex.
