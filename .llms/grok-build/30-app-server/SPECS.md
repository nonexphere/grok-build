# App Server — SPECS

## 1. Protocolo

JSON-RPC 2.0 bidirecional, initialize/capability gate e entidades
Session/Turn/Item. Native methods: `session/*`, `turn/*`, `item/*`. Adapter
Codex traduz `thread/*`; não há alias nativo automático.

## 2. Runtime facade

Uma facade narrow sobre Tower/SessionActor define operations e event stream.
Processor, MCP, tools e SDK não alcançam actor internals.

## 3. Core e transports

Connection state, scoped serializer, Session registry, Turn coordinator,
subscription hub e outbound router. In-process, stdio e WS passam a mesma
conformance suite. WS chega early, antes de history/GA completos.

## 4. History e approvals

Session files autoritativos; SQLite opcional rebuildable. Replay usa
event_seq/watermark/cursor. Interaction ID é durável e separado de request ID;
runtime continua dono da policy de aprovação.

## 5. Auth/rede

Bearer full-control em WS; loopback default, bind remoto explícito; cleartext e
sem Origin/scopes conforme threat model compartilhado. Limites/redaction são
obrigatórios.

## 6. Dashboard/ACP

Dashboard continua ACP/leader/roster no v1. Migração é v2 separada e não bloqueia
App Server/Tower release.

## 7. Validação

Seguir [TDD](../TDD.md), incluindo transport conformance, replay, concurrency,
security, fault/load e generated-schema drift.

