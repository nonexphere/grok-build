# Epic E10 — Dead code e caminhos experimentais

Status: planejado  
Escopo: REMOVER + REFACTORIZAR  
Owner: owners dos componentes auditados  
Depende de: [E1 product runtime](../../20-tower-core/v1-09-product-runtime-vertical-completion/), [E3 App Server](../../30-app-server/v1-10-product-contract-capability-ga/), [E4 MCP parity](../../40-mcp-control-plane/v1-06-parity-multisession/), [E6 tools](../../50-tower-agent-tools/v1-04-nine-tools-product-ga/)  
Consumidores: release e capability registry

## Tasks

- [ ] E10-01 consolidar inventário de TODO/FIXME/placeholder/fake/stub.
- [ ] E10-02 mapear APIs não chamadas e features sem consumidores.
- [ ] E10-03 distinguir código morto de conformance, fallback e human gate.
- [ ] E10-04 remover somente itens com prova de obsolescência.
- [ ] E10-05 substituir placeholders product-facing por erro fail-closed.
- [ ] E10-06 atualizar capabilities e documentação após cada remoção.
- [ ] E10-07 executar clippy, tests, diff review e busca residual.

## Acceptance criteria

Todo item termina como removido, substituído, justificado, bloqueado ou falso
positivo; nenhum caminho fake é selecionável pelo binário de produto.
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
