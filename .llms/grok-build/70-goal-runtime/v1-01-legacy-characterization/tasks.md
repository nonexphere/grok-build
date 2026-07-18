# Tasks — v1-01-legacy-characterization

## Inventário v1
- [ ] Mapear tracker, slash, tools, continuation, persistence, pager, ACP/headless — Follow @repository-exploration
- [ ] Consumir inventário de hot paths produzido pelo App Server
- [ ] Registrar exact package tests e feature/config gates

## Characterization RED/GREEN
- [ ] Golden de create/status/pause/resume/clear/complete/block
- [ ] Snapshot/event/tool schema fixtures
- [ ] Race user input/pause/continuation e restart fixtures
- [ ] Ordinary-session lazy/no-goal regression

## Boundary de versão
- [ ] Definir `disabled|v1|v2` config schema com v2 indisponível fail-loud
- [ ] Encapsular v1 atrás de port preservando behavior
- [ ] Testar restore e rollback v1/disabled
- [ ] Proibir v2 selection antes do epic v2 release gate

## Validação
- [ ] Suites focadas + `cargo fmt --check` + checks dos crates tocados
- [ ] Diff review contra baseline — Follow @code-review

## Specs e docs
- [ ] Atualizar contrato v1 e matriz v1→v2
- [ ] Não marcar v2 planejado como implementado

## Tarefas operacionais (humanas)
Nenhuma tarefa operacional humana para este epic.
