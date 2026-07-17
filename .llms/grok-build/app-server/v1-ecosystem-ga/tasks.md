# Tasks — v1-ecosystem-ga

## ACP e Codex adapters
- [ ] Rerotear ACP pelo shared registry/runtime facade
- [ ] Mapear session/prompt/update/reverse permission sem semantic fork
- [ ] Isolar missing-jsonrpc/ID/capability/Codex differences em adapter crate
- [ ] Executar native-vs-adapter conformance fixtures

## Goal integration
- [ ] Projetar goal lifecycle/requirements/tasks/subagents/evidence/usage/verifier
- [ ] Mapear user goal commands pela facade com revisions/idempotency
- [ ] Correlacionar Goal/Task child Threads sem lifecycle authority
- [ ] Test App Server disconnect sem interromper/corromper Goal Runtime

## SDK e clients
- [ ] Gerar TypeScript SDK da mesma source
- [ ] Criar Electron IPC e VS Code examples
- [ ] Criar remote/mobile summary/scoped approval reference
- [ ] Validar capability downgrade e version compatibility

## Operations e observability
- [ ] Implementar metrics/traces/admin projection verify/rebuild/token revoke
- [ ] Alertas para queue/projection/replay/auth/runtime adapter failures
- [ ] Graceful restart/recovery runbook and tooling
- [ ] Stability/deprecation/versioning policy

## GA hardening
- [ ] Full transport/golden/property/fault/fuzz/load suites
- [ ] Threat review de remote, approvals, paths, secrets e plugins — Follow @code-audit
- [ ] TUI parity report + ACP compatibility + SDK drift green
- [ ] Verificar Definition of Done item por item com evidence

## Validação
- [ ] Multi-client E2E TUI+automation+observer+ACP
- [ ] Reconnect active Turn/approval/goal with no gap/double effect
- [ ] Production readiness audit and delivery report

## Specs e docs
- [ ] Protocol/SDK/client/remote/admin/migration/runbook docs
- [ ] Atualizar todos status somente após gates

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar stable Grok extension inventory — type: product-decision — blocking: v1 compatibility freeze
- [ ] (HUMAN) Aprovar remote GA scope/security exceptions — type: product-decision — blocking: remote GA
- [ ] (HUMAN) Executar production sign-off — type: manual-verify — blocking: concluir epic
