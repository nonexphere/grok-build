# Epic v1-02 — Tools in-process, ACL e parity MCP
Owner: Tower tools owners
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
Estimativa: 2–3 semanas
Depende de: `../v1-01-tool-contract-and-facade/`, `../../40-mcp-control-plane/v1-01-server-transports/`
Habilita: orchestrator swarm no MVP e SDK/release hardening
Skills relacionadas: `@implementation-loop`, `@code-review`, `@code-audit`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Registra as mesmas operations como model tools locais. Agent type ACL default
allow somente `orchestrator`; config pode ampliar. MCP local não é auto-
injetado, eliminando loop/tool duplication.

## Escopo

### ADICIONAR
- tool registry integration, role ACL config/validation e differential conformance.

### REFACTORIZAR
- orchestrator tool surface para descriptor-driven Tower capability.

### REMOVER
- qualquer auto-config da própria Tower como MCP client.

### MANTÉM
- external Tower via configured MCP e subagent tools.

## Contratos

- [Tower tools](../../_shared/tower-agent-tools.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [control-plane security](../../_shared/control-plane-security.md)

## TODO checklist

- [ ] RED orchestrator allow e build/explore deny
- [ ] Implementar immutable effective agent type/ACL lookup
- [ ] Registrar tools in-process da mesma schema source
- [ ] Testar config allowlist e invalid role fail-closed
- [ ] Proibir session/model de auto-conceder ACL
- [ ] Assert nenhuma config MCP da Tower local é injetada
- [ ] Configurar Tower externa por MCP sem name collision
- [ ] Differential MCP vs in-process success/error/state tests
- [ ] E2E orchestrator start→send→wait→history→archive
- [ ] Security audit de privilege escalation — Follow @code-audit

## Riscos e incertezas

- **[HIGH][Confirmed] privilege escalation:** runtime-owned role + deny default.
- **[HIGH][Likely] tool duplication/loop:** composition assertions.
- **[MEDIUM][Possible] external/local identity confusion:** instance IDs in results.
