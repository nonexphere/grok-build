# App Server

## O que é

Control plane e protocolo multi-client do Grok Build, criado pela promoção e
generalização do leader existente, não por um daemon/runtime paralelo.

## Papel

Possui JSON-RPC, Thread/Turn/Item, conexões, subscriptions, controller leases,
reverse requests, replay, projeções, transportes e daemon lifecycle. Delega
agent behavior, providers, tools, permissions, MCP, skills, hooks, subagents,
worktrees e sessão ao runtime Grok.

## Stack

- Rust/Tokio; novo protocol crate e server/client crates
- JSON-RPC 2.0; in-process, stdio NDJSON, IPC e WebSocket
- SQLite rebuildable projection sobre arquivos de sessão existentes
- Rust como fonte para JSON Schema/TypeScript/SDKs

## Estado atual

Leader, `SessionActor`, session files, ACP fan-out e `AcpUpdateTracker` já
fornecem grande parte dos primitives. Falta uma facade estável, entidades e IDs
canônicos, projector, processor JSON-RPC, replay consistente, approval broker,
transport conformance e migration da TUI.

## Issues conhecidos

- risco de criar control planes/actors duplicados;
- IDs de Item/event ordering ainda não são contrato estável;
- schema/TS/examples em `changes/` são proposta, não geração comprovada;
- decisões de controller, protocol strictness, remote auth e delta durability abertas;
- projection DB não existe e não pode virar segunda source of truth;
- TUI parity e reconnect não estão demonstrados.

## Epics

- [v1-architecture-protocol](./v1-architecture-protocol/)
- [v1-runtime-facade-projection](./v1-runtime-facade-projection/)
- [v1-core-in-process](./v1-core-in-process/)
- [v1-history-replay](./v1-history-replay/)
- [v1-approvals-control](./v1-approvals-control/)
- [v1-daemon-transports-security](./v1-daemon-transports-security/)
- [v1-tui-migration](./v1-tui-migration/)
- [v1-ecosystem-ga](./v1-ecosystem-ga/)

## Contratos

- [runtime ownership](../_shared/runtime-ownership.md)
- [identity/event ordering](../_shared/identity-event-ordering.md)
- [leases/idempotency](../_shared/leases-idempotency.md)
- [security/authority](../_shared/security-authority-boundaries.md)
