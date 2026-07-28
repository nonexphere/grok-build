# Epic v1-01 — Codex readiness e hygiene

Status: rascunho
Prioridade: paralelo ao lançamento core
Estimativa: 1–2 semanas
Depende de: nenhuma
Habilita: `../v1-02-api-key-provider-foundation/`
Skills relacionadas: `@implementation-loop`, `@code-review`, `@add-provider`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Reconcilia o caminho Codex já implementado com `TO_RELEASE.md`, ledger, issues e
testes para estabelecer a base factual que BYOK deve reutilizar.

## Escopo

### ADICIONAR
- matriz única requirement→composition test→status; testes que distinguem PASS de skip.

### REFACTORIZAR
- completar seams genéricos necessários a providers sem reabrir wire Codex.

### REMOVER
- alegações contraditórias/obsoletas após evidência; nenhum runtime path provado.

### MANTÉM
- OAuth fail-closed D10, request-time binding, 401 one-retry, xAI/custom models.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [TDD](../../TDD.md)

## TODO checklist

- [ ] Reproduzir baseline package-scoped seguindo [TDD](../../TDD.md)
- [ ] Fechar/representar docs-001, testing-001/002 e operations-001 com status único
- [ ] Provar production composition de binding, attempt ID e account-scoped cache key
- [ ] Tornar live tests `skipped/blocked`, nunca PASS sem credencial
- [ ] Atualizar release honesty sem ampliar claims
- [ ] Rodar `cargo test -p xai-grok-auth -p xai-grok-multi-auth --no-fail-fast`
- [ ] Rodar checks shell/sampler atingidos e `git diff --check`
- [ ] Atualizar README/SPECS e delivery evidence

## Riscos e incertezas

- **[HIGH][Confirmed] status drift:** fontes discordam — uma matriz canônica e contract tests.
- **[MEDIUM][Likely] live gate externo:** credenciais ausentes — manter PARTIAL explícito.
- **UNVERIFIED:** full PC8 permanece não provado.

