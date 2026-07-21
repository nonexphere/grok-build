# 40 — MCP Control Plane

## O que é

MCP server da Tower para clients locais e remotos. O crate `xai-grok-mcp`
atual é client; este programa adiciona server sem confundir os papéis.

## Estado atual

MCP client possui stdio/HTTP/OAuth/liveness; não há servidor de controle de
sessions nem Streamable HTTP/SSE da Tower.

## Issues conhecidos

- risco de duplicar semantic core de App Server;
- remote bearer permissivo exige threat model e limits reais;
- SSE legado vs Streamable HTTP precisa compat explícita.

## Epics

- [v1-01-server-transports](./v1-01-server-transports/)
- [v1-02-remote-security-conformance](./v1-02-remote-security-conformance/)
- [v1-03-tower-product-runtime](./v1-03-tower-product-runtime/) — supervisor combinado
- [v1-04-mcp-contract-transport-completion](./v1-04-mcp-contract-transport-completion/) — schema/stdio/HTTP completos
- [v1-05-token-scopes-tls-release](./v1-05-token-scopes-tls-release/) — auth lifecycle e remote release
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
