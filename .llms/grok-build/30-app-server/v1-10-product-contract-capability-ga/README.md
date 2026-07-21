# Epic E3 — App Server product contract e capability truth

Status: planejado/P0  
Escopo: REFACTORIZAR + ADICIONAR  
Owner: `xai-grok-app-server`, `xai-grok-app-server-protocol`  
Depende de: [E1 product runtime](../../20-tower-core/v1-09-product-runtime-vertical-completion/), [E2 lifecycle](../../20-tower-core/v1-10-lifecycle-recovery-hardening/)  
Consumidores: MCP, SDK e produto

## Tasks

- [ ] E3-01 fechar catálogo de métodos e schemas.
- [ ] E3-02 derivar capability registry de caminhos executáveis.
- [ ] E3-03 completar projeções session/turn/item/event/interaction.
- [ ] E3-04 convergir erro, retryability e operation identity.
- [ ] E3-05 validar in-process, stdio e WebSocket com a mesma semântica.
- [ ] E3-06 testar timeout, cancel, disconnect, reconnect e resync.
- [ ] E3-07 adicionar schema drift gate e geração limpa.

## Gate

Capability matrix, wire schemas e errors são idênticos entre transportes e
nenhum método anuncia caminho somente fake.
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
