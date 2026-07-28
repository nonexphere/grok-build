# Epic v1-01 — SDK TS gerado, client e exemplos

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: `../../30-app-server/v1-04-websocket-remote-auth/`, `../../30-app-server/v1-05-history-replay/`
Habilita: scripts oficiais e App Server release hardening
Skills relacionadas: `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Rust gera schema/declarations; package TS adiciona client WS handwritten pequeno
e transport-agnostic types. Bundle `changes/` vira fixture/migration seed.

## Escopo

### ADICIONAR
- reproducible generate-ts, typed client, async notifications, reconnect e examples.

### REFACTORIZAR
- Thread seeds para Session nativa; errors/capabilities conforme protocol source.

### REMOVER
- generated artifacts manuais como source of truth.

### MANTÉM
- Codex mapping em adapter/doc específico.

## Contratos

- [Session identity](../../_shared/session-turn-item-identity.md)
- [App Server protocol](../../30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md)

## TODO checklist

- [ ] RED generation drift contra Rust schema
- [ ] Gerar types Session/Turn/Item/errors/capabilities
- [ ] Implementar initialize/request/notification correlation
- [ ] Implementar Item async stream/reconnect/replay cursor
- [ ] Implementar bearer handshake sem log/storage inseguro
- [ ] Node script start→send→stream→history→interrupt
- [ ] Browser WS example com explicit token input
- [ ] Conformance SDK vs direct WS fixtures
- [ ] Package build/typecheck/test e monorepo integration
- [ ] Documentar versioning e no-publish MVP
- [ ] (HUMAN) Confirmar path/package/browser scope — type: product-decision — blocking: package freeze

## Riscos e incertezas

- **[HIGH][Confirmed] schema drift:** generated-only + CI.
- **[MEDIUM][Likely] reconnect duplicate events:** cursor/event_seq conformance.
- **[LOW][Possible] npm premature stability:** local package até freeze.

