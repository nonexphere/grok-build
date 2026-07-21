# Epic E2 — Lifecycle, persistence, recovery e identity

Status: planejado/P0  
Escopo: REFACTORIZAR + ADICIONAR  
Owner: `xai-grok-tower`, `xai-grok-shell`  
Depende de: [E1 product runtime](../v1-09-product-runtime-vertical-completion/)  
Consumidores: App Server, MCP, SDK

## Tasks

- [ ] E2-01 congelar identity, epoch, cursor e idempotency.
- [ ] E2-02 completar archive, dormant resume e restart.
- [ ] E2-03 reconstruir history dos arquivos canônicos.
- [ ] E2-04 garantir rollback de spawn/persistência parcial.
- [ ] E2-05 testar crash, duplicate start, stale cursor e rebind.
- [ ] E2-06 testar interrupt-versus-complete com um terminal state.
- [ ] E2-07 adicionar limites de fila, payload, tempo e memória.

## Gate

Property/conformance/fault tests preservam identidade e ordenação após restart,
falha e reconexão.
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
