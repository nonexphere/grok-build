# Epic v1-05 — Tokens, scopes e TLS para release remoto
Owner: MCP/control-plane owners
Escopo: conforme a seção Escopo deste epic

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
Status: rascunho
Prioridade: P1 release-remoto-bloqueante
Estimativa: 2–4 semanas
Depende de: ../v1-04-mcp-contract-transport-completion/, ../../30-app-server/v1-09-capability-contract-product-conformance/
Habilita: 30/v1-07 remote release
Skills relacionadas: @architecture-spec-authoring, @implementation-loop, @code-audit, @human-product-test
Proveniência: [provenance: user-input, conversation, skill-output, doc-tree]

## Objetivo

Entregar lifecycle administrativo de credenciais do control plane, autorização por scopes e um caminho TLS/WSS release-ready sem remover o modo local simples.

## Escopo

### ADICIONAR

- token IDs, create/list/revoke/rotate e durable revocation;
- scopes por surface/operação e least-privilege presets;
- process TLS ou reverse-proxy contract testável;
- readiness/audit/fingerprint e migration do bearer legado;
- CLI/admin API sem secret em argv/log.

### REFACTORIZAR

- bearer full-control atual torna-se legacy/admin scope explícito;
- App Server e MCP compartilham authn/authz core.

### REMOVER

- query-string bearer no modo seguro;
- claim de remote-ready sobre cleartext;
- revocation somente por restart.

### MANTÉM

- loopback development com token simples;
- insecure-no-auth apenas opt-in local e warning forte.

## Human gates

- (HUMAN, product-decision, blocking: implementation) aprovar scope taxonomy e one-time token display.
- (HUMAN, manual-verify, blocking: remote release) aprovar TLS termination/runbook e threat model.

## Gate de saída

Token revogado perde acesso imediatamente, scopes são iguais em App Server/MCP, URL nunca carrega token e um smoke TLS/WSS remoto autorizado passa.
