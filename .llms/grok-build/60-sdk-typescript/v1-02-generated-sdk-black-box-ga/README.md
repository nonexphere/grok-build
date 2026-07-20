# Epic v1-02 — SDK gerado, black-box e GA

Status: rascunho
Prioridade: P1 lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: ../v1-01-generated-sdk-client-examples/, ../../30-app-server/v1-09-capability-contract-product-conformance/, ../../40-mcp-control-plane/v1-04-mcp-contract-transport-completion/
Habilita: 30/v1-07
Skills relacionadas: @implementation-loop, @code-review, @human-product-test
Proveniência: [provenance: user-input, skill-output, code, doc-tree]

## Objetivo

Trocar o mirror TS manual por geração reproduzível e provar o client contra listeners reais, incluindo reconnect, abort, errors e packaging.

## Escopo

### ADICIONAR

- generator completo de declarations e method bindings;
- clean regeneration gate;
- Node stdio/WS black-box e MCP example separado;
- reconnect/AbortSignal/backpressure/close tests;
- package/export/version compatibility matrix.

### REFACTORIZAR

- client handwritten pequeno consome somente tipos gerados;
- examples executam runtime product-backed.

### REMOVER

- Interim hand-authored mirror como source of truth;
- testes apenas FakeTransport como prova de release;
- browser bearer example inseguro.

### MANTÉM

- browser WS indisponível até handshake seguro;
- package private até human publish approval.

## Contratos

- [TypeScript SDK](../../_shared/typescript-sdk.md)
- [conformance](../../_shared/contract-conformance-capability-truth.md)
- [product readiness](../../_shared/product-runtime-readiness.md)

## Gate de saída

Apagar artifacts e regenerar produz diff limpo; Node stdio/WS controla Session real; SDK errors/cursors/capabilities coincidem com Rust.

