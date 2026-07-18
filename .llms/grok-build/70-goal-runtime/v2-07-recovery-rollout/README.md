# Epic v2-07 — Migração, flags e rollout v2

Status: planejado
Prioridade: pós-lançamento core
Estimativa: 2–4 semanas
Depende de: `../v2-01-domain-foundation/`, `../v2-02-persistence-leases-accounting/`, `../v2-03-runtime-continuation/`, `../v2-04-tools-verification/`, `../v2-05-task-graph-subagents/`, `../v2-06-clients-projections/`
Habilita: opt-in e futuro default estável do Goal Runtime v2
Skills relacionadas: `@code-audit`, `@code-review`, `@release-checklist`, `@delivery-report`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Fecha recovery de todos os in-flight resources, executa dual-version/dual-projection rollout,
security/load/fuzz e fornece rollback/runbooks. Não adiciona feature lateral.

## Escopo

### ADICIONAR
- reconciler de turn/child/verifier/worktree/usage/intents;
- migration telemetry, recovery tools e release evidence.

### REFACTORIZAR
- startup passa por restore paused/recovering antes de driving.

### REMOVER
- nenhum adapter/flag v1 é removido; v1 permanece selecionável conforme decisão humana.

### MANTÉM
- legacy completed goals rotulados, auditáveis e não reabertos automaticamente.

## Business rules

- recovery nunca presume side-effect success;
- interactive auto-resume segue ADR; headless é opt-in `auto-if-clean`;
- rollback preserva dados legíveis e não reduz segurança;
- release requer todos os explicit gates, não apenas build green.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/control-plane-security.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Toda recovery matrix e rollback stage tem resultado determinístico provado.
- Security/load/fuzz/cross-platform e ordinary-session regression passam.
- Readiness review e sign-off aprovam telemetry, accounting e rollback.

## Riscos e incertezas

- **[HIGH][Confirmed] restart ambíguo:** duplicate work/data loss — exhaustive reconciliation/fault matrix.
- **[HIGH][Likely] legacy fixture diversity:** import surprises — anonymized real fixtures + quarantine.
- **Human decision required:** default version, rollout thresholds, compatibility window e sign-off.
