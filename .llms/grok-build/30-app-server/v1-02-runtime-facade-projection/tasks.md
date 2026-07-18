# Tasks — v1-02-runtime-facade-projection

## Facade contract
- [ ] Definir operations start/resume/read/list/fork/turn/control — Follow @implementation-loop
- [ ] Definir runtime event enum e error mapping
- [ ] Implementar fake runtime deterministic para server tests
- [ ] Implementar shell adapter sem second actor

## Identity allocator
- [ ] Mapear session UUID→Session e prompt ID→Turn
- [ ] Definir Item IDs por stable source/event/history epoch
- [ ] Persistir/derivar source offsets e revision
- [ ] Testar rebuild stability e collision handling

## Event normalization
- [ ] Projetar user/agent/reasoning-safe-summary Items
- [ ] Projetar command/file/MCP/skill/hook/permission/plan Items
- [ ] Projetar subagent/worktree/background/compaction/rewind/provider errors
- [ ] Projetar Goal state via interface quando disponível sem hard dependency

## Golden fixtures
- [ ] Extrair anonymized fixtures de todos major flows
- [ ] Assert IDs/order/status/revision/final state/no duplicates
- [ ] Comparar contra `AcpUpdateTracker` behavior oracle
- [ ] Adicionar malformed/out-of-order/source-retry cases

## Segurança e performance
- [ ] Redact secrets e omit hidden reasoning
- [ ] Limitar payload/diff/output projection
- [ ] Medir projector overhead e allocation
- [ ] Garantir no blocking I/O no actor event path

## Validação
- [ ] Facade fake/shell contract suite
- [ ] Golden normalization suite
- [ ] `cargo fmt --check`, checks e focused tests

## Specs e docs
- [ ] Documentar event mapping e instrumentation decisions
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
Nenhuma tarefa operacional humana para este epic.
