# Epic v1-01 — Caracterização e boundary do Goal legado

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
