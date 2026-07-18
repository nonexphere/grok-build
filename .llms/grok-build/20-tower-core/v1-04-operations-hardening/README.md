# Epic v1-04 — Operations e hardening da Tower

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 1–3 semanas
Depende de: `../v1-03-multi-instance-daemon-modes/`, `../../30-app-server/v1-07-release-hardening/`, `../../40-mcp-control-plane/v1-02-remote-security-conformance/`
Habilita: release Tower v1
Skills relacionadas: `@code-audit`, `@code-review`, `@release-checklist`, `@delivery-report`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Fecha health/readiness, graceful drain, recovery, telemetry bounded e runbooks
sem adicionar quota enforcement ou migrar dashboard.

## Escopo

### ADICIONAR
- health/status/admin-safe views, fault/load/cross-platform evidence e recovery runbook.

### REFACTORIZAR
- startup/shutdown para explicit instance lifecycle.

### REMOVER
- flags experimentais somente após gates individuais.

### MANTÉM
- unlimited sessions policy e dashboard ACP.

## Contratos

- [Tower lifecycle](../../_shared/tower-instance-lifecycle.md)
- [runtime ownership](../../_shared/runtime-ownership.md)

## TODO checklist

- [ ] Fault matrix spawn/readiness/crash/drain/restart
- [ ] Health sem secret/session payload leak
- [ ] Load N clients/Sessions/Towers com bounded queues/tasks/FDs
- [ ] Resource current/peak telemetry accuracy classification
- [ ] Cross-platform socket/pipe/state permissions
- [ ] Security/operations audit — Follow @code-audit
- [ ] Runbooks e rollback testados
- [ ] Release checklist/delivery evidence e status reconciliation

## Riscos e incertezas

- **[HIGH][Confirmed] partial shutdown lies:** terminal states/recovery truth.
- **[MEDIUM][Likely] telemetry não-portável:** label best-effort e test per platform.
- **Human decision required:** release sign-off — type: manual-verify — blocking: concluir epic.
