# Epic v1-09 — Capability truth e conformance de produto
Owner: App Server/protocol owners
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
Estimativa: 2–4 semanas
Depende de: ../../20-tower-core/v1-06-canonical-session-actor-runtime/, ../v1-05-history-replay/, ../v1-06-approvals-control/
Habilita: ../v1-07-release-hardening/
Skills relacionadas: @implementation-loop, @code-review, @human-product-test
Proveniência: [provenance: user-input, skill-output, code, doc-tree]

## Objetivo

Alinhar protocol, capabilities, errors, interactions, replay e behavior real do App Server, provando todos os métodos anunciados no runtime product-wired.

## Escopo

### ADICIONAR

- capability registry derivada da composição;
- conformance product-backed para in-process, stdio e WebSocket;
- interaction delivery pelo actor real;
- inventory/removal de placeholder, fake-only e dead paths;
- mapping Codex separado e verificado.

### REFACTORIZAR

- OperationResult, error data e capabilities usam tipos canônicos gerados;
- tests distinguem conformance fake de product integration.

### REMOVER

- capabilities true sem caminho executável;
- hand-authored contract divergence e diagnostics de slice no wire;
- imports/code paths mortos no escopo App Server.

### MANTÉM

- Session nativo e Codex Thread somente no adapter;
- dashboard ACP congelado.

## Contratos

- [capability truth](../../_shared/contract-conformance-capability-truth.md)
- [product readiness](../../_shared/product-runtime-readiness.md)
- [protocol v1](../v1-01-session-protocol/contracts/session-protocol-v1.md)
- [approvals](../../_shared/approvals-controller-history.md)

## Gate de saída

Toda capability anunciada passa black-box no binário real por pelo menos um transport e conformance normalizada nos demais; unavailable é false/omitted, nunca canned success.
