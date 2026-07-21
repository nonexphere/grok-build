# Epic v1-04 — Operations e hardening da Tower
Owner: Tower core/runtime owners
Escopo: conforme a seção Escopo deste epic

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
## Revisão de implementação

Este epic só pode ser executado quando cada task tiver owner, arquivos ou
contrato afetado, pré-condição, comando de validação e evidência esperada.
Alterações de comportamento exigem Red-Green-Refactor; alterações de contrato
exigem contract test e atualização da matriz de rastreabilidade.

### Gate mínimo

- [ ] dependências e links deste epic foram verificados;
- [ ] interfaces, schemas, estados, erros e compatibilidade estão definidos;
- [ ] caminho fake/conformance está separado do caminho product-backed;
- [ ] testes unitários, integração, black-box e segurança foram classificados;
- [ ] timeout, cancelamento, retry, restart e falhas parciais foram tratados;
- [ ] observabilidade, limites de recurso e redaction foram especificados;
- [ ] comando reproduzível e artefato de evidência foram registrados;
- [ ] bloqueios humanos/externos possuem owner e condição de desbloqueio;
- [ ] status do epic foi reconciliado com `TRACEABILITY.md` e `COMPLETION_COVERAGE.md`.
