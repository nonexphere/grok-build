# 80 — Channel Gateways

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

Backlog pós-core de bridges, começando por Telegram, como clients da Tower.

## Estado atual

Somente visão em `docs/architecture/CHANNEL_GATEWAYS_AND_REALTIME_VOICE.md`.

## Issues conhecidos

Auth de bot, chat→Session, approvals e privacy exigem decisões humanas futuras.

## Epics

- [v1-01-telegram-bridge-backlog](./v1-01-telegram-bridge-backlog/)
