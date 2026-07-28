# Review — Codex contract deepening + scaffold — 2026-07-18

| Campo | Valor |
|-------|--------|
| **Passo** | Resposta ao prompt `CODEX_PROMPT_CONTRACT_DEEPENING_AND_SCAFFOLD.md` |
| **Baseline anterior** | `c399006` (checklist D-* + prompt) |
| **Commit deste estado** | ver git após este review (`docs(plan): scaffold app-server/tower crates…`) |
| **Reviewer** | Grok (path-cited + `cargo check` + `cargo test -p xai-grok-app-server-protocol`) |
| **Self-report Codex** | `.llms/grok-build/_shared/INDEX.md` (quase tudo DONE; 3 PARTIAL) |

---

## 1. Resumo executivo

O Codex **cumpriu em grande medida** o pass de deepening + scaffold:

| Área | Veredito |
|------|----------|
| Scaffold de crates Rust no workspace | **SIM** — 6 crates novas + `cargo check` green |
| Package TS `packages/grok-oss-app-server` | **SIM** — types, client, examples, drift script (node_modules local, gitignored) |
| Contratos `_shared` expandidos | **SIM** — crate-map, facade, security, tools, MCP/CLI, etc. |
| Protocol split (methods/events/errors/mapping) | **SIM** |
| JSON Schema + 4 goldens JSONL | **SIM** |
| Testes de wire no protocol crate | **SIM** — **7 passed** |
| Tasks core com D-ID + comando + aceitação | **SIM** (melhoria grande vs antes) |
| Wave 3 corrigida (tools antes de MCP) | **SIM** |
| Processor/runtime real | **NÃO** (correto — só boundary) |
| INDEX auto-DONE | **Útil mas inflado** — DONE = “artefato de scaffold/spec existe”, **não** epic implementado |

**Nota geral deste pass:** **~8/10 para freeze de boundaries**, **~4/10 para runtime pronto**.  
P0-VITAL de **documentação + types + schemas** está em boa forma. P0 de **comportamento do servidor** ainda é o próximo trabalho (epics 20–50).

---

## 2. Inventário do scaffold (código)

### 2.1 Crates Rust (workspace)

| Crate | Path | Papel observado no código | Completude |
|-------|------|---------------------------|------------|
| `xai-grok-app-server-protocol` | `crates/codegen/xai-grok-app-server-protocol/` | Types serde+schemars, methods, events, schemas, goldens, 7 tests | **Alta (scaffold+types)** |
| `xai-grok-app-server` | `…/xai-grok-app-server/` | Trait `AppServerProcessor` + `transport` stub | **Baixa (só interface)** |
| `xai-grok-app-server-client` | `…/xai-grok-app-server-client/` | Client trait boundary | **Baixa** |
| `xai-grok-tower` | `…/xai-grok-tower/` | `TowerHandle` / `TowerInstanceId` + trait `GrokRuntimeFacade` + `RuntimeEvent` | **Média (API sketch boa)** |
| `xai-grok-tower-tools` | `…/xai-grok-tower-tools/` | 9 descriptors + nomes + schema refs + testes mínimos | **Média** |
| `xai-grok-mcp-server` | `…/xai-grok-mcp-server/` | Reexport tools + enum transport | **Mínima** |

Registrados em `Cargo.toml` workspace (linhas de members confirmadas).

### 2.2 Package TypeScript

```text
packages/grok-oss-app-server/
  package.json, package-lock.json, tsconfig.json, .gitignore
  src/{index,client,types}.ts
  src/transports/{stdio,websocket}.ts
  examples/{stdio,websocket}.ts
  test/client.test.ts
  scripts/check-schema-drift.mjs
  node_modules/   # gitignored — NÃO commitar
```

### 2.3 Schemas e goldens

| Artefato | Notas |
|----------|--------|
| `schemas/app-server.schema.json` | $defs ricos: initialize, session, turn, item union, methods params |
| `schemas/tower-tools.schema.json` | pares input/output das 9 tools |
| `schemas/goldens/happy-coding.jsonl` | cenário coding turn |
| `schemas/goldens/interrupt.jsonl` | interrupt |
| `schemas/goldens/multi-session.jsonl` | multi-session |
| `schemas/goldens/reconnect.jsonl` | reconnect |

### 2.4 Validação executada nesta review

```text
cargo check -p xai-grok-app-server-protocol \
  -p xai-grok-app-server -p xai-grok-app-server-client \
  -p xai-grok-tower -p xai-grok-tower-tools -p xai-grok-mcp-server
→ Finished ok (~0.3s warm)

cargo test -p xai-grok-app-server-protocol --lib
→ 7 passed; 0 failed
  - initialize_roundtrip_preserves_wire_shape
  - item_roundtrip_preserves_tagged_body
  - all_four_jsonl_goldens_are_valid_json_objects
  - generated_protocol_schema_compiles
  - all_nine_tower_tool_input_and_output_examples_validate
  - checked_in_schemas_compile_and_cover_nine_tool_pairs
  - golden_params_and_events_validate_against_named_checked_in_definitions
```

---

## 3. Inventário do plano atualizado (docs)

### 3.1 Novos / expandidos em `_shared/`

| Arquivo | ~linhas | Conteúdo |
|---------|--------:|----------|
| `INDEX.md` | 191 | Matriz D-* DONE/PARTIAL |
| `crate-map.md` | 89 | Ownership, DAG mermaid, forbidden edges, feature flags, file ownership |
| `source-of-truth.md` | 23 | Precedência de fontes |
| `runtime-facade.md` | 66 | Facade boundary |
| `mcp-server-transport-cli.md` | 107 | MCP + CLI + co-start |
| `typescript-sdk.md` | 39 | SDK contract |
| `approvals-controller-history.md` | 50 | leases / history |
| `provider-contract.md` | 55 | BYOK/provider |
| `goal-boundary.md` | 37 | goal flags / no scaffold |
| `ui-freeze.md` | 26 | dashboard freeze |
| `security-authority-boundaries.md` | 5 | redirect stub |
| `control-plane-security.md` | 126 | expandido (era ~menor) |
| `tower-agent-tools.md` | 180 | expandido (era ~43) |
| `tower-instance-lifecycle.md` | 194 | expandido |
| `session-turn-item-identity.md` | 40 | expandido |

### 3.2 Protocol contracts (30-app-server)

| Arquivo | ~linhas |
|---------|--------:|
| `session-protocol-v1.md` | 205 |
| `methods.md` | 176 |
| `events.md` | 125 |
| `errors.md` | 79 |
| `codex-adapter-mapping.md` | 22 |
| **Total contracts local** | **~607** |

### 3.3 Tasks reescritas (core)

Novos ou reescritos com IDs tipo `SP101-01`, `TA101-01`, `[D-…]`, path de crate, comando `cargo test`, critério de aceitação:

- `20-tower-core/v1-01..04/tasks.md`
- `30-app-server/v1-01..07` + `v2-01` tasks
- `40-mcp-control-plane/v1-01..02/tasks.md`
- `50-tower-agent-tools/v1-01..02` + `v2-01/tasks.md`
- `60-sdk-typescript/v1-01/tasks.md`

### 3.4 Root README / TDD / TRACEABILITY

- **Wave 3 corrigida:** `50/v1-01` **depois** tools facade → WS + MCP (alinha DAG real).
- `TDD.md` expandido (~+78 linhas no diff).
- `TRACEABILITY` aponta INDEX e D-IDs.
- Status de Goal/gateway/voice: ainda `planejado` em vários READMEs backlog (política parcialmente aplicada; ver gaps).

---

## 4. Alinhamento com handoff (decisões travadas)

| Decisão | Scaffold/plan | OK? |
|---------|---------------|-----|
| Session canônico | types/schemas/session methods | **SIM** |
| Sem `tower_agent_hub` | 9 tools only | **SIM** |
| Crate protocol sem IO | protocol deps = serde/schemars only | **SIM** |
| Facade sem 2º SessionActor | docs + trait only | **SIM** |
| MCP server ≠ client | crate separado reexport tools | **SIM** (mínimo) |
| Bearer / no scopes / cleartext | security contract expandido | **SIM** |
| Multi-tower / multi-session | lifecycle expandido | **SIM** |
| Dashboard freeze | `ui-freeze.md` | **SIM** |
| Goal fora + flags | `goal-boundary.md` | **SIM** |
| TS package path proposed | `packages/grok-oss-app-server` | **SIM** (publish HUMAN) |

---

## 5. O que está **muito bom**

1. **Boundary de crates correta** — protocol puro; processor/tower/tools/mcp separados; DAG documentado.  
2. **Types Rust reais + JsonSchema + goldens validados** — não só markdown.  
3. **7 testes protocol** que falhariam se schema/golden divergissem.  
4. **`GrokRuntimeFacade` trait** legível e alinhado a Session/Turn/Item.  
5. **Tasks acionáveis** com D-ID + comando — salto de qualidade vs checklist genérico.  
6. **Honestidade de PARTIAL** em INDEX (token CLI, npm publish, etc.).  
7. **Wave 3 fix** (tools antes de MCP).  
8. **Não** implementou processor falso “funcionando”.

---

## 6. Gaps, riscos e desalinhamentos (materiais)

### 6.1 [HIGH][Confirmed] INDEX “DONE” ≠ implementação de epic

- **Evidence:** INDEX define DONE como existência de contrato/scaffold; tasks dos epics ainda pedem “implement complete unions / projection / shell adapter”.  
- **Impact:** leitor confunde “P0 scaffold done” com “epics 30/v1-01 concluídos”.  
- **Fix:** renomear mentalmente / doc: *scaffold-complete* vs *epic-complete*; manter tasks unchecked até runtime behavior.

### 6.2 [MEDIUM][Confirmed] Tasks citam filtros de teste ainda não existentes como módulos nomeados

- **Evidence:** e.g. `cargo test -p xai-grok-app-server-protocol initialize` — existe teste com “initialize” no nome (ok com filter substring); mas `cargo test -p xai-grok-tower facade_conformance` / `projection` / `xai-grok-shell facade_conformance` **ainda não existem**.  
- **Impact:** tasks são o mapa do trabalho restante — bom — mas não devem ser lidas como “já green”.  
- **Fix:** manter assim; próximo execute-plan cria esses testes em RED primeiro.

### 6.3 [MEDIUM][Likely] Schema JSON monólito vs completeness

- **Evidence:** `app-server.schema.json` é denso e cobre muito; `tower-tools.schema.json` cobre 9 tools. Goldens são curtos (8–14 linhas).  
- **Impact:** goldens validam “JSON object + named defs”, não um transcript de produto completo com todas as notificações em ordem real de processor.  
- **Fix:** epics de core in-process devem expandir goldens para stream completo pós-processor.

### 6.4 [MEDIUM][Confirmed] `xai-grok-app-server` / `mcp-server` quase vazios

- **Evidence:** processor = 1 trait method; mcp-server = enum + reexport.  
- **Impact:** esperado para scaffold; risco de alguém “preencher” fora do epic owner.  
- **Fix:** crate-map já diz ownership — enforce em review de PR.

### 6.5 [MEDIUM][Likely] Integração com `xai-grok-shell` ainda zero

- **Evidence:** nenhum `app_server_runtime` em shell; facade sem impl real.  
- **Impact:** vertical slice real ainda não existe.  
- **Fix:** epic `30/v1-02` + `20/v1-01` characterization — próximo código de valor.

### 6.6 [LOW][Confirmed] `node_modules` no worktree

- **Evidence:** `packages/grok-oss-app-server/node_modules/` presente; `.gitignore` cobre.  
- **Impact:** commit errado se `git add` descuidado.  
- **Fix:** commit só source + lockfile; nunca node_modules.

### 6.7 [LOW][Possible] Geração schema “from Rust” vs checked-in dual source

- **Evidence:** schemas checked-in + schemars no crate; TS types hand-authored + drift script.  
- **Impact:** dual source até pipeline de generate único.  
- **Fix:** epic 60 deve tornar generate→commit o único path.

### 6.8 [LOW][Confirmed] Status `planejado` em Goal v2 / gateways / voice

- **Evidence:** READMEs de 70/80/90 ainda `planejado` em vários casos.  
- **Impact:** confusão de “aprovado para execução”.  
- **Fix:** cosmetic no root policy (já parcialmente documentado).

### 6.9 [INFO] Self-audit INDEX é generoso

- Provas genéricas repetem a mesma frase para dezenas de D-IDs.  
- **Review externa** (este arquivo) deve ser a fonte de veredito de qualidade.

### 6.10 [INFO] Não há processor conformance suite ainda

- TDD.md descreve layout futuro; pasta `tests/conformance` ainda não é o suite multi-transport real.

---

## 7. Matriz P0-VITAL (review §12) — reavaliação

| Bloco | Status real | Notas |
|-------|-------------|-------|
| Crate map + workspace scaffold | **DONE** | 6 crates + DAG |
| Session protocol wire + goldens | **DONE (scaffold-grade)** | types+schema+4 goldens+7 tests |
| Tower lifecycle docs + instance id types | **DONE (docs+ids)** | Handle stub; sem connect-or-spawn real |
| tower_agent_* schemas + ACL docs | **DONE (schemas+descriptors)** | sem execução |
| Security matrix docs | **DONE** | sem auth implementation |
| Runtime facade trait | **DONE (trait)** | sem impl SessionActor |
| MCP surface | **PARTIAL scaffold** | docs fortes; código mínimo |
| CLI flags | **DONE (docs)** | sem wiring binário |
| TS skeleton | **DONE** | private package |
| Core tasks rewrite | **DONE** | |

---

## 8. O que ainda precisa de resposta humana (atualizado)

Do INDEX + handoff (ainda aberto):

| ID | Decisão | Blocking |
|----|---------|----------|
| H-SEC-1 | Aceitar threat model remoto full-control cleartext | release remoto |
| H-AS-jsonrpc | Adapter Codex missing-jsonrpc? native stays strict | só adapter |
| H-SDK-publish | Nome/publicação npm | publish |
| H-TR-tokens | CLI tokens create/list/revoke no MVP? | token CLI |
| H-AP-default | Interaction deadline / wait→deny default headless | headless policy |
| H-PLAN-1 | Aprovar plano+scaffold como base de execução | process |

Não reabrir: Session, 9 tools, multi-tower, dashboard freeze, Goal fora core.

---

## 9. Próximos passos recomendados (execução)

1. **Wave 0 código real:** `20/v1-01` characterization leader + tests; `30/v1-01` completar gaps de types/tests nomeados nas tasks (muitos already partial green).  
2. **Wave 1:** facade thin adapter em shell (`app_server_runtime`) + multi-session registry.  
3. **Wave 2–3:** processor in-process/stdio → WS/bearer → MCP server real.  
4. **Não** expandir Goal/Telegram até core vertical slice “coding turn” via App Server.

Primeiro PR de implementação sugerido:  
`goblin-app-server-protocol-hardening` (fechar SP101-* que ainda falham por filtro/nome) **ou**  
`goblin-tower-leader-characterization` (TW101-*).

---

## 10. Veredito final

| Pergunta | Resposta |
|----------|----------|
| Codex fez o deepening pedido? | **Sim, com qualidade alta no protocol/types/docs** |
| Scaffold modular correto? | **Sim** (boundaries certas) |
| Pronto para implementação de processor? | **Sim, como base** — contratos/types existem |
| Runtime/product ready? | **Não** |
| INDEX confiável como “tudo DONE”? | **Só no sentido scaffold/spec** |
| Commit necessário? | **Sim** — estado dirty antes desta review |

---

## 11. Paths-chave

```text
.llms/grok-build/_shared/INDEX.md
.llms/grok-build/_shared/crate-map.md
.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/
crates/codegen/xai-grok-app-server-protocol/
crates/codegen/xai-grok-tower/
crates/codegen/xai-grok-tower-tools/
crates/codegen/xai-grok-app-server/
crates/codegen/xai-grok-mcp-server/
packages/grok-oss-app-server/
docs/architecture/prompts/CODEX_PROMPT_CONTRACT_DEEPENING_AND_SCAFFOLD.md
```
