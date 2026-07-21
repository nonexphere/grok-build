# Epic E1 — Runtime product-backed e actor canônico

Status: planejado/P0  
Escopo: ADICIONAR + REFACTORIZAR  
Owner: `xai-grok-shell`, `xai-grok-tower`, `xai-grok-pager-bin`  
Depende de: [v1-06 canonical session actor](../v1-06-canonical-session-actor-runtime/), [v1-08 product session host](../v1-08-product-session-host/)  
Consumidores: App Server, MCP e Tower Tools

## Tasks

- [ ] E1-01 definir `ProductSessionDependencies` com ownership explícito.
- [ ] E1-02 implementar factory única de `SessionActor` real.
- [ ] E1-03 injetar a mesma autoridade no composition root.
- [ ] E1-04 proibir FakeRuntime/echo no binário de produto.
- [ ] E1-05 ligar provider, modelo, sandbox, workspace e persistence reais.
- [ ] E1-06 implementar start/turn/event/wait/history vertical.
- [ ] E1-07 testar readiness e rollback de bootstrap.
- [ ] E1-08 testar interrupt, shutdown, concorrência e cleanup.
- [ ] E1-09 executar black-box pelo `grok-oss`.

## Gate

O processo real executa initialize → session → turn → history → interrupt →
archive/resume sem segundo actor, capability falsa ou transcript divergente.
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
