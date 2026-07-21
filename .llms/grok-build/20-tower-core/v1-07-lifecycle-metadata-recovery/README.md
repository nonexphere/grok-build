# Epic v1-07 — Lifecycle, metadata e recovery fiéis
Owner: Tower core/runtime owners
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
Prioridade: P0 lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: ../v1-06-canonical-session-actor-runtime/
Habilita: 50/v1-03, 30/v1-07
Skills relacionadas: @implementation-loop, @code-review
Proveniência: [provenance: user-input, skill-output, code, doc-tree]

## Objetivo

Fazer status, residency, agentType, provider binding, workspace e active Turn refletirem a autoridade canônica em list/status/replay, com archive/resume/restart válidos.

## Escopo

### ADICIONAR

- metadata canônica persistida/rebuildable;
- state machine status × residency × activeTurn;
- recovery de crash-mid-turn e stale actor;
- filters/pagination ordering inputs para consumers.

### REFACTORIZAR

- Session projection e registry rows deixam de hardcode unknown/resident;
- archive/resume são transições atômicas e actor-aware.

### REMOVER

- combinações impossíveis archived+resident e resume de starting/active;
- updatedAt/status derivados de placeholder.

### MANTÉM

- transcript como source of truth;
- projection rebuildable e archive não destrutivo.

## Contratos

- [Tower lifecycle](../../_shared/tower-instance-lifecycle.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [product runtime readiness](../../_shared/product-runtime-readiness.md)

## Gate de saída

Property/concurrency/crash tests provam todas as transições, rows canônicas e restart sem identidade falsa.
