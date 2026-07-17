# App Server — VISION

## Project Role

Ser a única superfície programática multi-client do Grok Build, preservando o
runtime existente e permitindo TUI, desktop, IDE, automação e remote/mobile.

## Architecture Fit

Transportes convergem no mesmo processor, que opera uma facade estreita sobre
um único registry de sessões. Projeções são reconstruíveis; permissões e
execução continuam no runtime.

## Evolução

1. **v1:** in-process/IPC/stdio/WebSocket, Thread/Turn/Item, replay, approvals,
   TUI parity, ACP e SDK inicial.
2. **v2:** clientes desktop/IDE maduros e remote control administrável.
3. **v3+:** políticas enterprise, compat adapters adicionais e ecossistema de
   clientes sem expandir a autoridade do protocolo.

## Design Principles

1. Uma semantic core para todos os transportes.
2. Identity e ordering são contratos persistentes.
3. Projection nunca supera a source of truth.
4. Backpressure é explícita e isolada por conexão.
5. Segurança remota é deny-by-default.
6. TUI continua sendo o cliente de referência.

## Out of Scope

- reescrever SessionActor, tools ou sampling;
- substituir session files como verdade no v1;
- expor hidden chain-of-thought;
- compatibilidade com todo endpoint experimental Codex;
- permitir que cliente remoto burle trust/sandbox/policy.
