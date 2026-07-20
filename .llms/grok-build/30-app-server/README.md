# 30 — App Server

## O que é

Protocolo JSON-RPC multi-client do grok-oss com entidades canônicas
Session/Turn/Item, uma facade sobre a Tower e transports in-process, stdio e
WebSocket. Inspirado no Codex, mas `Thread` existe apenas no adapter de mapping.

## Estado atual

Leader, SessionActor, roster ACP, session files e tracker existem. O protocolo,
processor, facade tipada, transports in-process/stdio/WebSocket, autenticação
base e gates de conformance já existem e têm cobertura de testes em nível de
componente. A integração produtiva ainda é parcial: o composition root não
está conectado a um actor canônico residente com semântica completa de Turn,
steer, Interaction e replay de eventos do runtime real; a geração/consumer TS
também não é um gate de lançamento completo.

Portanto, este diretório não está concluído: os epics de runtime produtivo,
paridade cross-transport e release hardening continuam sendo necessários.

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
- [v1-08-tower-product-runtime](./v1-08-tower-product-runtime/) — supervisor combinado
- [v1-09-capability-contract-product-conformance](./v1-09-capability-contract-product-conformance/) — capability truth e runtime real
- [v2-01-dashboard-client-migration](./v2-01-dashboard-client-migration/) — futuro

## Contratos

- [Session/Turn/Item](../_shared/session-turn-item-identity.md)
- [runtime ownership](../_shared/runtime-ownership.md)
- [security](../_shared/control-plane-security.md)
- [ordering/replay](../_shared/identity-event-ordering.md)
