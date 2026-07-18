# 30 — App Server

## O que é

Protocolo JSON-RPC multi-client do grok-oss com entidades canônicas
Session/Turn/Item, uma facade sobre a Tower e transports in-process, stdio e
WebSocket. Inspirado no Codex, mas `Thread` existe apenas no adapter de mapping.

## Estado atual

Leader, SessionActor, roster ACP, session files e tracker existem. App Server,
processor, facade estável, WebSocket e geração TS ainda não estão implementados.

## Issues conhecidos

- drafts antigos contradiziam o glossário e threat model humanos;
- protocol seeds em `changes/` são manuais e usam Thread;
- IDs/order/replay/approval controller ainda precisam virar contratos reais;
- dashboard deve permanecer intocado no MVP.

## Epics

- [v1-01-session-protocol](./v1-01-session-protocol/)
- [v1-02-runtime-facade-projection](./v1-02-runtime-facade-projection/)
- [v1-03-core-in-process-stdio](./v1-03-core-in-process-stdio/)
- [v1-04-websocket-remote-auth](./v1-04-websocket-remote-auth/)
- [v1-05-history-replay](./v1-05-history-replay/)
- [v1-06-approvals-control](./v1-06-approvals-control/)
- [v1-07-release-hardening](./v1-07-release-hardening/)
- [v2-01-dashboard-client-migration](./v2-01-dashboard-client-migration/) — futuro

## Contratos

- [Session/Turn/Item](../_shared/session-turn-item-identity.md)
- [runtime ownership](../_shared/runtime-ownership.md)
- [security](../_shared/control-plane-security.md)
- [ordering/replay](../_shared/identity-event-ordering.md)

