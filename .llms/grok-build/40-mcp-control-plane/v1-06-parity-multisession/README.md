# Epic E4 — MCP parity e multi-session

Status: planejado/P0  
Escopo: REFACTORIZAR + ADICIONAR  
Owner: `xai-grok-mcp-server`  
Depende de: [E3 App Server contract](../../30-app-server/v1-10-product-contract-capability-ga/)  
Consumidores: `rmcp`, SDK e clientes externos

## Tasks

- [ ] E4-01 fechar parity stdio/HTTP nos nove tools.
- [ ] E4-02 validar schemas antes do efeito runtime.
- [ ] E4-03 completar POST/GET/DELETE, TTL, rebind e reconexão.
- [ ] E4-04 suportar múltiplas sessões isoladas.
- [ ] E4-05 testar cancelamento, disconnect, stream limits e resync.
- [ ] E4-06 executar matriz independente `rmcp` HTTP/stdio.
- [ ] E4-07 executar stdio product-backed com turn e interrupção reais.

## Gate

Clientes independentes observam os mesmos resultados, erros, ids e estados em
stdio e Streamable HTTP.
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
