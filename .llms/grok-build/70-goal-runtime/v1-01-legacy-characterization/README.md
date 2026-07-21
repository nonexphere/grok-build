# Epic v1-01 — Caracterização e boundary do Goal legado
Owner: goal runtime owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento core
Estimativa: 1–3 semanas
Depende de: nenhuma (consome o inventário de hot paths de `../../30-app-server/v1-07-release-hardening/` antes do gate final)
Habilita: `../v2-01-domain-foundation/`
Skills relacionadas: `@repository-exploration`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Congela o Goal atual como contrato v1 observável e introduz somente a menor port
versionada/flag necessária para que v2 futuro não exija big-bang. Não implementa
state machine v2, SQLite v2 ou novos verifier semantics.

## Escopo

### ADICIONAR
- characterization/golden/race fixtures; `GoalRuntimeVersion` boundary e kill switch.

### REFACTORIZAR
- encapsular entrypoints v1 sem alterar comportamento.

### REMOVER
- nada; legacy continua selecionável.

### MANTÉM
- slash syntax, tool semantics, persistence/events/TUI/continuation atuais.

## Contratos

- [Goal v1 baseline](./contracts/goal-v1-baseline.md)
- [runtime ownership](../../_shared/runtime-ownership.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

V1 passa goldens por todos entrypoints; flag disabled/v1 não altera ordinary
sessions; App Server hot paths estão mapeados; nenhuma behavior v2 vazou.

## Riscos e incertezas

- **[HIGH][Confirmed] legacy behavior pouco especificado:** characterization real antes de port.
- **[HIGH][Likely] flag muda init/restore:** binary fixtures e rollback.
- **UNVERIFIED:** todos os entrypoints serão confirmados na exploração do epic.
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
