# Epic v1-04 — Groq onboarding

Status: rascunho
Prioridade: pós-foundation, paralelo ao core
Estimativa: 1–2 semanas
Depende de: `../v1-02-api-key-provider-foundation/`
Habilita: Groq L1–L3
Skills relacionadas: `@add-provider`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Provider Bearer em `https://api.groq.com/openai/v1`, Chat Completions default;
Responses somente por metadata/override futuro comprovado.

## Escopo

### ADICIONAR
- descriptor, model discovery/filter e guided key lifecycle.

### REFACTORIZAR
- nenhuma semantic de cache/Codex compartilhada por nome de campo.

### REMOVER
- nenhum custom model.

### MANTÉM
- wire slugs e provider-specific usage sem alegação de Codex cache.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [TDD](../../TDD.md)

## TODO checklist

- [ ] Fixtures `/models`, auth errors e chat stream
- [ ] RED para base URL sem `/v1`/backend incorreto
- [ ] Registrar/validar/persistir provider credential
- [ ] Merge credential-scoped catalog com capability metadata
- [ ] E2E onboarding→turn com tool→logout
- [ ] Two-key isolation e 401 reauth
- [ ] Codex cached-token evidence continua isolada
- [ ] Live smoke opt-in e docs honestas

## Riscos e incertezas

- **[MEDIUM][Confirmed] models/capabilities voláteis:** discovery + fixtures.
- **[MEDIUM][Possible] Responses parcial:** não habilitar sem per-model proof.

