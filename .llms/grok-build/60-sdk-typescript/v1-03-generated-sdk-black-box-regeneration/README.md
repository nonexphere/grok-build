# Epic E7 — SDK gerado e black-box compatibility

Status: planejado/P1  
Escopo: REFACTORIZAR + ADICIONAR  
Owner: SDK TypeScript + owners de protocolo  
Depende de: [E3 App Server contract](../../30-app-server/v1-10-product-contract-capability-ga/), [E4 MCP parity](../../40-mcp-control-plane/v1-06-parity-multisession/)  
Consumidores: clientes TypeScript

## Tasks

- [ ] E7-01 limpar outputs e regenerar schemas/tipos.
- [ ] E7-02 falhar CI em drift manual ou geração não determinística.
- [ ] E7-03 testar initialize/list/call/error contra listeners reais.
- [ ] E7-04 testar abort, reconnect, replay e capability negotiation.
- [ ] E7-05 documentar versionamento e compatibilidade do wire.
- [ ] E7-06 publicar exemplos executáveis sem mocks de transporte.

## Gate

Regeneração limpa produz diff vazio e o SDK passa black-box stdio/HTTP/WS com
erros e reconexões reais.
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
