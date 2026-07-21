# Epic E6 — Nove Tower tools product-backed

Status: planejado/P0  
Escopo: REFACTORIZAR + ADICIONAR  
Owner: `xai-grok-tower-tools`  
Depende de: [E1 product runtime](../../20-tower-core/v1-09-product-runtime-vertical-completion/), [E3 App Server contract](../../30-app-server/v1-10-product-contract-capability-ga/)  
Consumidores: MCP e App Server

## Tasks

- [ ] E6-01 mapear cada tool para comando real do actor.
- [ ] E6-02 completar input/output schemas e validação pré-efeito.
- [ ] E6-03 preservar ACL, agent type, identity e operationId.
- [ ] E6-04 cobrir start/list/read/resume/fork/archive/subscribe/turn/interaction.
- [ ] E6-05 executar matriz product-backed em HTTP e stdio.
- [ ] E6-06 testar erro, timeout, cancel, retry e partial operation.
- [ ] E6-07 remover placeholders que possam anunciar sucesso.

## Gate

Os nove tools passam schema, semântica, ACL e lifecycle usando o mesmo runtime
product-backed, sem duplicar estado.
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
