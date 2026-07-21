# 50 — Tower Agent Tools

## O que é

Contrato e implementação compartilhada da família `tower_agent_*`, exposta ao
orchestrator in-process e a clients MCP.

## Estado atual

Subagents depth=1 e operations de leader/roster existem em superfícies
fragmentadas; não há tools first-class de peer top-level Session.

## Issues conhecidos

- agent role ACL ainda não protege Tower operations;
- history/wait/redaction não têm contrato único;
- tools MCP e internas podem divergir se implementadas separadamente.

## Epics

- [v1-01-tool-contract-and-facade](./v1-01-tool-contract-and-facade/)
- [v1-02-in-process-acl-mcp-parity](./v1-02-in-process-acl-mcp-parity/)
- [v1-03-nine-tool-semantic-completion](./v1-03-nine-tool-semantic-completion/) — fecha gaps live das nove tools
- [v2-01-peer-messaging-study](./v2-01-peer-messaging-study/)
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
