# Tasks — v1-daemon-transports-security

## Leader promotion
- [ ] Mapear/migrar registration, request multiplexing e lifecycle sem fork daemon
- [ ] Compartilhar ThreadRegistry e processor com in-process path
- [ ] Preservar ACP routing/capabilities/reconnect
- [ ] Implementar single-instance/readiness/relaunch semantics

## Transport acceptors
- [ ] stdio NDJSON framing/EOF/error/shutdown
- [ ] Unix socket/Windows named pipe permissions/peer identity
- [ ] WebSocket handshake/Origin/subprotocol/frame/compression limits
- [ ] Common connection lifecycle and max-message enforcement

## AuthN/AuthZ
- [ ] Local peer/socket/pipe trust policy
- [ ] Remote tokens/scopes/expiry/revocation/pairing hooks
- [ ] Loopback/non-loopback TLS and Origin requirements
- [ ] Method/Thread/path authorization matrix

## Backpressure e operations
- [ ] Bounded priority queues e delta coalescing
- [ ] Slow-client isolation/disconnect metrics
- [ ] Health/status/connections/threads/admin commands
- [ ] Graceful drain/restart with interrupted Turn truth

## Conformance e security
- [ ] Uma suite contra in-process/stdio/IPC/WebSocket
- [ ] half-close/slowloris/decompression/message bomb tests
- [ ] peer ACL/token replay/scope/Origin/path/symlink tests
- [ ] load 10 clients/100 Threads without unbounded memory

## Validação
- [ ] Threat-model audit — Follow @code-audit
- [ ] Cross-platform transport CI
- [ ] Performance targets e focused checks

## Specs e docs
- [ ] Daemon/admin/security/pairing runbooks
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Escolher enterprise identity/pairing backend — type: product-decision — blocking: enterprise remote auth
- [ ] (HUMAN) Aprovar remote control modes — type: product-decision — blocking: remote stable enablement
