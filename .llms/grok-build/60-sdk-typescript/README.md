# 60 — SDK TypeScript

## O que é

Schema e client TypeScript reais para App Server WebSocket, gerados da fonte
Rust e acompanhados por scripts executáveis.

## Estado atual

Bundle `changes/grok_app_server_spec_bundle` contém schema/TS/examples seed;
não há package, geração reproduzível ou client comprovado.

## Issues conhecidos

- path Node/browser e publicação estão abertos;
- schema pode driftar do processor;
- examples antigos usam Thread.

## Epics

- [v1-01-generated-sdk-client-examples](./v1-01-generated-sdk-client-examples/)
- [v1-02-generated-sdk-black-box-ga](./v1-02-generated-sdk-black-box-ga/) — geração real e listener black-box
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
