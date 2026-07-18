# Tasks — v1-03-core-in-process-stdio

## Processor e connection state
- [ ] Implementar initialize gate/version/capability negotiation — Follow @implementation-loop
- [ ] Parse/validate request/notification/response/server-response
- [ ] Mapear errors e preservar request IDs
- [ ] Rejeitar pre-init/malformed/unsupported calls deterministicamente

## Serialization e registry
- [ ] Definir global/session/turn/read-only scopes por method
- [ ] Implementar scoped serializer sem lock durante runtime await indevido
- [ ] Implementar SessionRegistry com shared pending loads
- [ ] Enforce um foreground Turn e idempotent start

## Subscriptions e outbound
- [ ] Implementar snapshot/live subscription state
- [ ] Implementar priority lanes e bounded queues
- [ ] Garantir lifecycle delivery e delta coalescing
- [ ] Isolar/disconnect slow subscriber com explicit error

## Core methods
- [ ] session start/resume/read/list/subscribe
- [ ] turn start/steer/interrupt
- [ ] item/turn/session lifecycle notifications
- [ ] typed in-process client e shutdown

## Vertical slice
- [ ] Scripted fake-runtime client completes coding Turn
- [ ] Shell-adapter smoke reconstructs final transcript
- [ ] Test reconnect boundary without persistence claims yet
- [ ] Test cancellation, duplicate request e concurrent Sessions

## Validação
- [ ] Processor/property/concurrency tests
- [ ] No unbounded queue/session/task growth
- [ ] Focused crate checks and code review

## Specs e docs
- [ ] Method inventory/serialization table
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
Nenhuma tarefa operacional humana para este epic.
