# Epic v1-02 — Foundation para providers de API key
Owner: provider/auth owners
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
Prioridade: paralelo ao lançamento core
Estimativa: 2–4 semanas
Depende de: `../v1-01-codex-readiness-hygiene/`
Habilita: `v1-03`, `v1-04`, `v1-05`
Skills relacionadas: `@add-provider`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Estende o control plane multi-auth com `ApiKey` real, provider descriptors,
credential-scoped catalogs e request-time static bearer. É uma extensão do
sistema Codex, não um wizard TOML paralelo. [provenance: inferred]

## Escopo

### ADICIONAR
- API-key login/import TTY + env/file non-TTY; metadata estruturada; static secret resolver;
- provider-driven CLI/catalog/logout/status e per-provider backend policy.

### REFACTORIZAR
- registry/composition/catalog merge hoje Codex-specific para descriptor-driven.

### REMOVER
- fallback `XAI_API_KEY` para bindings third-party nativos.

### MANTÉM
- custom `[model.*]`, Codex OAuth, legacy xAI e binding imutável.

## Business rules

Secret nunca entra em argv, TOML, model entry, status ou log. API key não tem
refresh; 401 invalida/reauth e no máximo um retry somente se geração mudou.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [TDD](../../TDD.md)

## TODO checklist

- [ ] Escrever RED de `LoginTransport::ApiKey` via composition root — Follow @add-provider
- [ ] Definir descriptor/capabilities/config metadata genéricos
- [ ] Implementar secure TTY paste e env/file non-TTY sem argv secret
- [ ] Persistir secret/metadata crash-consistent e owner-only
- [ ] Criar credential-scoped catalog merge e ambiguity rules
- [ ] Resolver static bearer imediatamente antes do request
- [ ] Proibir `XAI_API_KEY` fallback para native provider binding
- [ ] Implementar status/logout/replace-key provider-driven
- [ ] Testar duas credentials com mesmo model slug
- [ ] Testar canary secret full/prefix/suffix em todos sinks
- [ ] Preservar Codex/xAI/custom regressions
- [ ] Atualizar docs, config schema e provider checklist

## Riscos e incertezas

- **[HIGH][Confirmed] terceiro auth system:** evitar reutilizando multi-auth/store/binding.
- **[HIGH][Confirmed] secret leak:** TTY/file/store/redaction tests.
- **[MEDIUM][Likely] generic seam Codex-shaped:** provider fixtures antes de abstraction freeze.
