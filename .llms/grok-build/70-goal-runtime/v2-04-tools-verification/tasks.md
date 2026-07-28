# Tasks — v2-04-tools-verification

## Tool contracts
- [ ] Gerar strict schemas para `get_goal` e `update_goal` v2 — Follow @implementation-loop
- [ ] Validar origin/action/revisions/idempotency
- [ ] Implementar acknowledgements que não alegam terminal state
- [ ] Implementar legacy adapter com telemetry/deprecation tests

## Prompt protocol
- [ ] Versionar initial/continuation/completion/block prompts
- [ ] Delimitar objective como untrusted content
- [ ] Incluir revisions/budget/requirements/task/verifier gaps
- [ ] Golden tests contra prompt injection e compaction

## Verifier registry
- [ ] Implementar registry/versioning/composite all-any-threshold
- [ ] Implementar command/test/artifact/diff/static verifiers
- [ ] Adaptar skeptic panel sem mutation authority
- [ ] Implementar time/output/permission limits e evidence persistence

## Completion e blocker audit
- [ ] Avaliar cada requirement/deliverable da revisão atual
- [ ] Rejeitar stale/partial/indirect evidence
- [ ] Mapear infra error para InfraPaused
- [ ] Normalizar blocker e aplicar repeated-threshold policy

## Adversarial tests
- [ ] falsa alegação/fake test output/partial scope
- [ ] tentativas de pause/clear/edit/complete via tool
- [ ] wording drift de blocker e repeated completion requests
- [ ] repository/objective prompt injection

## Validação
- [ ] False completion E2E permanece Active/Paused com gaps
- [ ] True completion exige report persistido atual
- [ ] Focused + integration + schema snapshot tests

## Specs e docs
- [ ] Guia de verifier e trust MCP/skill
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar quando MCP pode ser authoritative — type: product-decision — blocking: MCP verifier defaults
- [ ] (HUMAN) Aprovar visual verification fallback — type: product-decision — blocking: visual verifier stable status
