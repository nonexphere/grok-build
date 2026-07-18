# Tasks — v1-04-websocket-remote-auth

## Integração Tower
- [ ] Consumir instance endpoints e processor da Tower sem daemon paralelo
- [ ] Compartilhar SessionRegistry e processor com in-process/stdio
- [ ] Preservar ACP routing/dashboard intocados
- [ ] Testar app-only e combined daemon mode

## Transport acceptors
- [ ] WebSocket handshake/subprotocol/frame/compression limits
- [ ] Aceitar Origin sem allowlist conforme contrato e testar o comportamento
- [ ] Common connection lifecycle and max-message enforcement

## AuthN/AuthZ
- [ ] Bearer full-control create/load/rotate/revoke e file permissions
- [ ] Loopback default e non-loopback explicit warning
- [ ] Provar `ws://` support sem alegar confidencialidade
- [ ] Runtime method/path sandbox continua authoritative

## Backpressure e operations
- [ ] Bounded priority queues e delta coalescing
- [ ] Slow-client isolation/disconnect metrics
- [ ] Health/status/connections/sessions/admin commands
- [ ] Graceful drain/restart with interrupted Turn truth

## Conformance e security
- [ ] Uma suite contra in-process/stdio/WebSocket
- [ ] half-close/slowloris/decompression/message bomb tests
- [ ] bearer replay/revoke/redaction/path/symlink tests
- [ ] load 10 clients/100 Sessions without unbounded memory

## Validação
- [ ] Threat-model audit — Follow @code-audit
- [ ] Cross-platform transport CI
- [ ] Performance targets e focused checks

## Specs e docs
- [ ] Daemon/admin/security/pairing runbooks
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aceitar threat model remoto antes do release — type: manual-verify — blocking: remote stable enablement
