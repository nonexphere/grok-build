# Tasks — v1-07-release-hardening

## Compatibilidade e Codex adapter
- [ ] Manter ACP/dashboard current path e executar regression suite
- [ ] Mapear Codex thread↔Session/prompt/update/reverse permission sem semantic fork
- [ ] Isolar missing-jsonrpc/ID/capability/Codex differences em adapter crate
- [ ] Executar native-vs-adapter conformance fixtures

## Goal hot-path inventory
- [ ] Inventariar reads/events/commands de goal v1 tocados pela facade
- [ ] Definir port versionada `disabled|v1|v2` sem implementar v2
- [ ] Test App Server disconnect sem interromper/corromper goal v1
- [ ] Registrar gaps para o futuro programa Goal v2

## Conformance cross-program
- [ ] Consumir evidence do SDK, MCP e Tower tools, sem reimplementar
- [ ] Validar capability downgrade e version compatibility
- [ ] Differential native/Codex adapter fixtures
- [ ] Testar daemon modes e remote clients combinados

## Operations e observability
- [ ] Implementar metrics/traces/admin projection verify/rebuild/token revoke
- [ ] Alertas para queue/projection/replay/auth/runtime adapter failures
- [ ] Graceful restart/recovery runbook and tooling
- [ ] Stability/deprecation/versioning policy

## GA hardening
- [ ] Full transport/golden/property/fault/fuzz/load suites
- [ ] Threat review de remote, approvals, paths, secrets e plugins — Follow @code-audit
- [ ] Dashboard/ACP regression + SDK/MCP drift green
- [ ] Verificar Definition of Done item por item com evidence

## Validação
- [ ] Multi-client E2E TUI existente + automation + observer + MCP
- [ ] Reconnect active Turn/approval/goal with no gap/double effect
- [ ] Production readiness audit and delivery report

## Specs e docs
- [ ] Protocol/SDK/client/remote/admin/migration/runbook docs
- [ ] Atualizar todos status somente após gates

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar stable Grok extension inventory — type: product-decision — blocking: v1 compatibility freeze
- [ ] (HUMAN) Aceitar remote MVP threat model — type: manual-verify — blocking: remote GA
- [ ] (HUMAN) Executar production sign-off — type: manual-verify — blocking: concluir epic
