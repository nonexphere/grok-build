# App Server — VISION

## Project Role

Ser a superfície programática nativa da Tower para scripts, SDKs e interfaces,
sem deslocar execução, storage ou policy do runtime Grok.

## Evolução

1. v1: Session/Turn/Item, facade, in-process/stdio/WS, replay/approvals e SDK.
2. v2: opcional dashboard como client e peer/internal extensions.
3. v3+: protocol stability e clients adicionais.

## Design Principles

1. Session nativa; Thread somente mapping Codex.
2. Uma semantic core e uma Tower registry.
3. WebSocket/remoto é MVP early.
4. Projection nunca supera session files.
5. Dashboard ACP permanece referência durante v1.

## Out of Scope

MCP tool semantics (programa 40/50), provider auth, Goal v2, Telegram, voice e
migração da TUI durante v1.
