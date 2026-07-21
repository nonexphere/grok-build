# Epic v1-02 — Segurança remota e conformance MCP
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
Prioridade: lançamento-bloqueante
Estimativa: 1–3 semanas
Depende de: `../v1-01-server-transports/`, `../../30-app-server/v1-04-websocket-remote-auth/`
Habilita: release MCP v1 e Tower operations hardening
Skills relacionadas: `@code-audit`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Aplica bearer full-control, redaction, bounds, audit metadata e threat tests ao
HTTP/SSE. Preserva a decisão de cleartext/sem scopes/sem Origin, com warnings
honestos em bind não-loopback.

## Escopo

### ADICIONAR
- token validate/revoke/rotate, abuse tests, connection limits e remote runbook.

### REFACTORIZAR
- auth gate compartilhado com WebSocket App Server.

### REMOVER
- alegações de scopes/TLS/Origin presentes no draft antigo.

### MANTÉM
- full-control token e `http://` support.

## Contratos

- [Tower tools](../../_shared/tower-agent-tools.md)
- [control-plane security](../../_shared/control-plane-security.md)

## TODO checklist

- [ ] RED missing/invalid/revoked bearer em HTTP/SSE
- [ ] Compartilhar auth/redaction core com WS
- [ ] Testar bearer nunca em URL/log/error/history
- [ ] Testar slowloris/oversize/queue/backpressure/reconnect
- [ ] Testar bind public warning e loopback default
- [ ] Audit metadata sem input/output secreto
- [ ] Threat-model audit — Follow @code-audit
- [ ] Runbook reverse proxy TLS recomendado, não obrigatório
- [ ] Human smoke LAN/internet autorizado

## Riscos e incertezas

- **[HIGH][Confirmed] stolen token = full control:** explicit warning/rotation/redaction.
- **[HIGH][Confirmed] cleartext internet:** accepted MVP risk; no false security claim.
- **Human decision required:** aceitar release com este threat model — type: manual-verify — blocking: remote release.
