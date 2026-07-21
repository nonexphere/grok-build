# Epic v1-05 — Cloudflare Workers AI onboarding
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
Prioridade: pós-foundation, paralelo ao core
Estimativa: 2–3 semanas
Depende de: `../v1-02-api-key-provider-foundation/`
Habilita: Cloudflare L1–L3
Skills relacionadas: `@add-provider`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Credential combina API token secreto e `account_id` não secreto; base URL é
derivada por credential. V1 cobre Workers AI OpenAI-compatible, não AI Gateway.

## Escopo

### ADICIONAR
- structured account metadata, URL template, model discovery/filter e validation.

### REFACTORIZAR
- binding/catalog invalidam quando account metadata muda.

### REMOVER
- nenhuma native `ai/run` path.

### MANTÉM
- Chat Completions-only product claim v1 e custom config.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [TDD](../../TDD.md)

## TODO checklist

- [ ] Fixtures token/account 401/403/not-found e catalog
- [ ] Validar account ID sem logar token
- [ ] Persistir token e metadata atomicamente
- [ ] Construir base URL segura sem path injection
- [ ] Filtrar text/chat models e preservar `@cf/...` slug
- [ ] Invalidar binding/catalog ao substituir account
- [ ] E2E onboarding→catalog→turn→logout
- [ ] Distinguir Workers AI de AI Gateway nas docs/errors
- [ ] Live smoke opt-in com credential humana

## Riscos e incertezas

- **[HIGH][Confirmed] tenancy na URL:** binding stale pode rotear conta errada — generation binding.
- **[MEDIUM][Likely] `/models` shape/filter:** revalidar em implementação.
- **UNVERIFIED:** endpoint exato permanece sujeito a docs/live contract no epic.
