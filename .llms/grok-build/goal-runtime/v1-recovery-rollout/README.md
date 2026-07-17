# Epic v1-recovery-rollout — Migração e production gate

Status: rascunho
Prioridade: lançamento-bloqueante
Depende de: todos os épicos Goal Runtime v1 anteriores
Habilita: release estável do Goal Runtime
Skills relacionadas: `@code-audit`, `@code-review`, `@release-checklist`, `@delivery-report`

## Arquitetura

Fecha recovery de todos os in-flight resources, executa dual-write rollout,
security/load/fuzz e fornece rollback/runbooks. Não adiciona feature lateral.

## Escopo

### ADICIONAR
- reconciler de turn/child/verifier/worktree/usage/intents;
- migration telemetry, recovery tools e release evidence.

### REFACTORIZAR
- startup passa por restore paused/recovering antes de driving.

### REMOVER
- adapters/flags legacy somente após janela e evidência aprovadas.

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
- [security/authority](../../_shared/security-authority-boundaries.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Toda recovery matrix e rollback stage tem resultado determinístico provado.
- Security/load/fuzz/cross-platform e ordinary-session regression passam.
- Readiness review e sign-off aprovam telemetry, accounting e rollback.

## Riscos e incertezas

- **[HIGH][Confirmed] restart ambíguo:** duplicate work/data loss — exhaustive reconciliation/fault matrix.
- **[HIGH][Likely] legacy fixture diversity:** import surprises — anonymized real fixtures + quarantine.
- **Human decision required:** rollout thresholds, compatibility window e sign-off.
