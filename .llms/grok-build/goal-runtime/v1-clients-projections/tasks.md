# Tasks — v1-clients-projections

## Command facade
- [ ] Implementar set/status/pause/resume/clear/edit/budget/audit/events/report
- [ ] Aplicar revision/idempotency/user-priority em todas mutations
- [ ] Preservar slash legacy parsing e reserved-token escape
- [ ] Testar TTY/non-TTY, confirmation e stale command

## Event projections
- [ ] Definir additive `GoalUpdatedV2` e granular events
- [ ] Projetar lifecycle, requirements, tasks, children, usage e reports
- [ ] Redigir/redact/truncar payloads sensíveis
- [ ] Snapshot/replay e old-client golden tests

## Pager
- [ ] Implementar compact chip e expandable dashboard
- [ ] Renderizar requirement/evidence/verifier status sem inference
- [ ] Integrar tasks/child transcript/worktree state
- [ ] Testar terminal sizes, accessibility e live updates

## Headless e ACP
- [ ] Implementar flags, streaming JSON events e exit code matrix
- [ ] Garantir normal session pipeline/tools/permissions reuse
- [ ] Mapear ACP commands/events sem segundo runtime
- [ ] Persistir final runtime-generated completion report

## App Server seam
- [ ] Expor read/command/event interface provider-neutral
- [ ] Correlacionar Goal/Task/Report com Thread/Turn sem delegar authority
- [ ] Criar contract fixtures para projector futuro

## Validação
- [ ] E2E create→pause→edit→resume→verify→complete em TUI/headless
- [ ] Old pager/ACP compatibility tests
- [ ] Render/performance e secret-redaction tests

## Specs e docs
- [ ] Guia de lifecycle e headless automation
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
Nenhuma tarefa operacional humana para este epic.
