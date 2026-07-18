# Tasks — v2-01-dashboard-client-migration

## Shadow projection
- [ ] Alimentar App projector ao lado do tracker atual sem UI mutation — Follow @implementation-loop
- [ ] Comparar Items/status/order/content/final blocks por stable correlation
- [ ] Registrar bounded anonymized mismatch reports
- [ ] Medir CPU/memory/latency overhead

## Adapter client
- [ ] Conectar TUI por typed in-process client
- [ ] Mapear session/turn lifecycle, steering e interruption
- [ ] Integrar subscriptions/controller/approvals/input
- [ ] Preservar existing shortcuts/actions/error UX

## Native tracker
- [ ] Renderizar message/reasoning/tool/file/MCP/skill/hook/plan Items
- [ ] Renderizar goal/subagent/worktree/background/compaction/rewind
- [ ] Handle out-of-order delta/revision/reconnect/replay
- [ ] Implementar controller loss/reclaim and observer indicators

## Rollout
- [ ] Flags shadow/opt-in/default e instant ACP rollback
- [ ] Persistir no UI apenas protocol state necessário
- [ ] Test restart/reconnect during active Turn/approval
- [ ] Definir telemetry and default-switch gate

## Parity/performance
- [ ] Golden visual/model tests de todos reference flows
- [ ] Terminal-size/accessibility/input stress
- [ ] p95 local lifecycle and render benchmarks
- [ ] No dropped lifecycle/duplicate block/secret leak assertions

## Validação
- [ ] Produzir machine-readable parity report
- [ ] Manual smoke dos richest flows
- [ ] Full pager/app-client regression

## Specs e docs
- [ ] TUI migration/rollback/troubleshooting docs
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar parity thresholds — type: product-decision — blocking: daemon default
- [ ] (HUMAN) Validar TUI smoke/parity report — type: manual-verify — blocking: concluir epic
