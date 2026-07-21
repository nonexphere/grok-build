# 10 — Providers

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

Programa de autenticação/inferência multi-provider: estabiliza o caminho Codex
existente e transforma OpenRouter, Groq e Cloudflare Workers AI de configuração
manual em onboarding BYOK nativo.

## Estado atual

- Codex offline path: substancialmente funcional; PC8 live parcial e UX/1.0 pendentes.
- API-key providers: HTTP compatível já funciona via TOML; login/store/catalog nativo não existe.
- `LoginTransport::ApiKey` existe como conceito, mas não está wired end-to-end.

## Issues conhecidos

- status documental do Codex diverge entre fontes e skips podem parecer PASS;
- third-party config pode cair perigosamente em `XAI_API_KEY`;
- catálogo BYOK não é credential-scoped e Cloudflare exige `account_id`;
- backend/capability varia por modelo e provider.

## Epics

- [v1-01-codex-readiness-hygiene](./v1-01-codex-readiness-hygiene/)
- [v1-02-api-key-provider-foundation](./v1-02-api-key-provider-foundation/)
- [v1-03-openrouter-onboarding](./v1-03-openrouter-onboarding/)
- [v1-04-groq-onboarding](./v1-04-groq-onboarding/)
- [v1-05-cloudflare-onboarding](./v1-05-cloudflare-onboarding/)
