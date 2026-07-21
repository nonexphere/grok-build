# 90 — Realtime Voice

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
## O que é

Backlog de conversa full duplex com STT/TTS streaming e barge-in.

## Estado atual

`xai-grok-voice` e pager oferecem captura/STT/ditado; não há loop agent speech
full duplex.

## Issues conhecidos

Provider, privacy, VAD/TTS, latency e hardware target permanecem decisões futuras.

## Epics

- [v1-01-full-duplex-backlog](./v1-01-full-duplex-backlog/)
