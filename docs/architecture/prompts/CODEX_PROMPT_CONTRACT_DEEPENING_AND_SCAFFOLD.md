# Prompt Codex — Contract deepening + scaffold (P0 vital)

**Uso:** copiar o bloco entre BEGIN_PROMPT e END_PROMPT para o Codex.

**Normative inventory:** `.llms/reviews/codex-epic-tree-review-2026-07-18.md` seções §10–§15.

**Objetivo:** engrossar contratos/schemas e scaffold de crates; NÃO reescrever o roadmap do zero; NÃO implementar processor completo.

---

```text
BEGIN_PROMPT

# Missão (OBRIGATÓRIA)

Você é o planejador/implementador de **contratos e scaffold** do grok-oss.
A árvore de épicos em `.llms/grok-build/` já existe e está **estruturalmente boa**,
mas é **rasa** como especificação de interfaces. Sua missão NÃO é reescrever o
roadmap do zero. Sua missão é:

1. **Detalhar de forma EXTREMAMENTE VERBOSA** todos os contratos vitais.
2. **Criar schemas** (JSON Schema e/ou Rust types serde + exemplos JSONL).
3. **Scaffold crates/módulos** no monorepo para as boundaries vitais.
4. **Reescrever tasks** dos epics core (20–60) para apontarem a seções de contrato.
5. **Corrigir** Wave 3/DAG e status policy conforme o review externo.

Você DEVE seguir o inventário de IDs `D-*` no review:
`.llms/reviews/codex-epic-tree-review-2026-07-18.md` seções **§10–§14**.
Cada ID deve terminar `DONE` com path, ou `PARTIAL` com justificativa e gap.

# NÃO fazer

- NÃO reabrir decisões do handoff (`docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md` §13).
- NÃO implementar processor/runtime completo, nem migrar dashboard/TUI.
- NÃO implementar Goal v2, Telegram, voice.
- NÃO inventar scopes/Origin/TLS obrigatórios (MVP é permissivo + threat model honesto).
- NÃO usar o termo público **thread** (Session é canônico; thread só mapping Codex).
- NÃO criar `tower_agent_hub`.
- NÃO auto-injetar MCP da Tower local em si mesma.
- NÃO `git add -A`; se commitar, paths explícitos e só se o humano pedir (default: deixe staged-ready / só escreva arquivos).

# Ler primeiro (ordem)

1. `.llms/reviews/codex-epic-tree-review-2026-07-18.md` (inteiro, especialmente §10–§15)
2. `docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md` §13–14
3. `.llms/grok-build/README.md` + `_shared/*` + `TDD.md` + `TRACEABILITY.md`
4. `30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md`
5. `_shared/tower-agent-tools.md`, `control-plane-security.md`, `tower-instance-lifecycle.md`
6. Seed: `changes/grok_app_server_spec_bundle/*` (inspiração; renomear Thread→Session)
7. Código real a caracterizar: `crates/codegen/xai-grok-shell/src/leader/`, roster, session storage
8. Skill `@plan-epic-tree` só se precisar ajustar forma de epics — prioridade é **contratos + scaffold**
9. Referências opcionais: `~/codex-app-server.md`, schemas Codex, `~/mcps/codex-bus-mcp` (inspiração)

# Entregáveis concretos

## A. Contratos verbosos (markdown)

Criar/expandir sob `.llms/grok-build/`:

- `contracts/INDEX.md` (ou `_shared/INDEX.md`) com tabela de todos os contratos + status D-*
- Expandir protocol Session para **nível wire-complete** (methods, events, errors, examples)
- Expandir tower lifecycle, security, tools (per-tool schemas)
- Runtime facade trait doc
- MCP server contract
- CLI/flags matrix
- Prefer split files se um único .md > ~300 linhas

## B. Schemas e goldens

- JSON Schema (ou equivalent) para:
  - initialize params/result
  - Session/Turn/Item
  - cada method params/result crítico
  - cada `tower_agent_*` input/output
- `examples/*.jsonl` golden scenarios (≥3): happy coding turn, interrupt, multi-session
- TypeScript types skeleton matching schemas

## C. Scaffold de código (workspace)

Adicionar crates (nomes finais documentados) com:

- `Cargo.toml` members
- modules + public types compiling
- `cargo check -p …` green
- roundtrip tests for serde types / schema where applicable
- `todo!`/`unimplemented!` only behind clear module boundaries; no fake “complete” logic

Proposta default (pode ajustar com ADR):

```text
crates/codegen/xai-grok-app-server-protocol/
crates/codegen/xai-grok-app-server/
crates/codegen/xai-grok-tower/
# mcp server adapter module or crate
# packages/grok-oss-app-server/ (TS)
```

Dependency rules MUST be written and respected (protocol crate: no shell dependency).

## D. Plano/tasks

- Fix root README waves to match real deps (`50/v1-01` before `40/v1-01`)
- Deepen tasks for epics in 20, 30, 40, 50, 60: each task → contract ID + test command
- Status: keep future Goal/gateway/voice as rascunho/backlog clarity
- Update TRACEABILITY paths

## E. TDD

- Expand `TDD.md` with conformance suite layout and named tests for new crates

# Padrão de qualidade (não negociável)

- Verboso: preferir 1 exemplo JSON a mais do que 1 parágrafo vago.
- Determinístico: error codes estáveis.
- Testável: cada regra de contrato deve mapear a um teste nomeado (mesmo que ainda RED).
- Self-contained: outro agent implementa processor sem precisar deste chat.
- Provenance: decisões novas `[provenance: …]`; se conflitar com handoff, **handoff vence**.

# Ordem de trabalho

1. INDEX + crate ADR + scaffold empty crates (cargo check)
2. Session protocol deep + goldens + serde types
3. Tower lifecycle deep
4. tower_agent_* schemas
5. Security matrix
6. Runtime facade trait
7. MCP + CLI contracts
8. TS skeleton
9. Rewrite core epic tasks to point at contracts
10. Fix waves + TRACEABILITY + completion matrix of all D-* IDs
11. Final report: files created, cargo check output summary, remaining PARTIAL

# Definition of Done

Cumprir §14 do review. Se faltar tempo, **nunca** deixe P0-VITAL incompleto para enfeitar Goal/Telegram.

# Output final da sua mensagem

1. Tabela D-* → status → path
2. Tree de arquivos novos
3. `cargo check` commands e resultado
4. Riscos remanescentes / HUMAN ainda abertos
5. Próximo epic de implementação real recomendado após este pass

END_PROMPT
```
