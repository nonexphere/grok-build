# Epic v1-03 — OpenRouter onboarding

Status: rascunho
Prioridade: pós-foundation, paralelo ao core
Estimativa: 1–2 semanas
Depende de: `../v1-02-api-key-provider-foundation/`
Habilita: BYOK vertical slice de referência
Skills relacionadas: `@add-provider`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Primeiro provider descriptor API-key: Bearer, base
`https://openrouter.ai/api/v1`, catálogo grande e Chat Completions default.

## Escopo

### ADICIONAR
- provider/validation/catalog filters; optional `HTTP-Referer`/`X-Title` policy.

### REFACTORIZAR
- nenhum sampler protocol; reuse OpenAI-compatible chat path.

### REMOVER
- nenhuma config manual existente.

### MANTÉM
- Responses beta fora do default; model slug com `/` intacto.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [TDD](../../TDD.md)

## TODO checklist

- [ ] Frozen `/models` + 401/429/malformed fixtures primeiro
- [ ] Registrar descriptor/kill switch/backend default
- [ ] Validar key sem persistir em falha
- [ ] Filtrar/paginar/search catálogo sem truncar slug
- [ ] Construir binding/base/headers reservados
- [ ] Testar tool-streaming Chat Completions representativo
- [ ] E2E onboarding→catalog→headless turn→logout
- [ ] Live smoke opt-in com evidence redigida
- [ ] Documentar feature matrix e Responses beta como não alegada

## Riscos e incertezas

- **[MEDIUM][Confirmed] catálogo enorme:** filtro/search/curated defaults.
- **[MEDIUM][Likely] capability metadata muda:** fixtures + honest unknown.
- **[LOW][Possible] headers de marketing:** `[PROPOSED]` opt-in até decisão.

