# Review — Codex epic tree rewrite vs handoff — 2026-07-18

| Campo | Valor |
|-------|--------|
| **Alvo** | Árvore `.llms/grok-build/**` produzida pelo Codex a partir de `docs/architecture/prompts/CODEX_PROMPT_PLAN_EPIC_TREE_GROK_OSS.md` |
| **Baseline commit (pré-árvore no git)** | `967161f` (prompt only; tree ainda dirty) |
| **Handoff** | `docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md` §13–14 |
| **Self-audit Codex** | `.llms/grok-build/PLAN_AUDIT.md` |
| **Reviewer** | Grok goal pass (path-cited, structural checks) |
| **Código de produto** | Não implementado (correto) |

---

## 1. Resumo executivo

### O Codex fez o que foi pedido?

| Entrega do prompt | Status | Evidência |
|-------------------|--------|-----------|
| Árvore numerada `10..90` | **SIM** | programas no disco |
| Root README + DAG + waves + principles | **SIM** | `.llms/grok-build/README.md` |
| Reescrever App Server (Session, WS early) | **SIM** | `30-app-server/` + contrato Session |
| MCP control plane no mesmo release | **SIM** | `40-mcp-control-plane/` |
| `tower_agent_*` MUST set | **SIM** | `_shared/tower-agent-tools.md` + `50/*` |
| Tower multi-session / multi-instance | **SIM** | `20-tower-core/` + `_shared/tower-instance-lifecycle.md` |
| TDD.md | **SIM** | `.llms/grok-build/TDD.md` |
| Goal v1 legado / v2 flags, fora do core path | **SIM** | `70-goal-runtime/` |
| Providers Codex + BYOK OR/Groq/CF | **SIM** | `10-providers/` |
| Gateways/Telegram + voice como backlog | **SIM** | `80/`, `90/` |
| TRACEABILITY + PLAN_AUDIT | **SIM** (extra útil) | arquivos no root |
| Skill structure (Escopo, riscos, tasks) | **SIM** (estrutural) | 0 broken links; 0 epics sem Escopo/Riscos |
| **Commit da árvore** | **NÃO no momento da análise** | working tree dirty (Codex não commitou) |
| Implementação de código | **NÃO** (correto) | |

**Veredito de planejamento:** a árvore é **largamente alinhada** às decisões travadas do handoff e é **quase implementation-ready** para o wave core, com gaps **humanos** e alguns **desalinhamentos de processo/status** listados abaixo — nenhum deles invalida a estrutura, mas alguns devem ser resolvidos antes de tratar o plano como “congelado para execução”.

**Readiness:**

| Aspecto | Ready? |
|---------|--------|
| Estrutura / sequência / deps acíclicas | **Sim** |
| Glossário Session vs Thread | **Sim** (core) |
| Security MVP permissiva documentada | **Sim** |
| Tools MUST + dual surface | **Sim** |
| Commit no git | **Pendente até commit desta review** |
| Aprovação humana do plano (status `planejado` em massa) | **Parcial / ambíguo** |
| Decisões `[PROPOSED]` / `(HUMAN)` pré-execução | **Aberto** (não bloqueia planejamento; bloqueia freeze de superfície) |

---

## 2. Inventário da árvore

### 2.1 Programas

| # | Path | Epics (count) | Role |
|---|------|---------------|------|
| 10 | `10-providers/` | 5 | Codex hygiene + API-key foundation + OR/Groq/CF |
| 20 | `20-tower-core/` | 4 | leader → Tower, multi-session, multi-instance, ops |
| 30 | `30-app-server/` | 8 (7×v1 + 1×v2 dashboard) | Session protocol → release |
| 40 | `40-mcp-control-plane/` | 2 | MCP server transports + remote security |
| 50 | `50-tower-agent-tools/` | 3 (2×v1 + peer v2 study) | tools contract + ACL/MCP parity |
| 60 | `60-sdk-typescript/` | 1 | generated SDK + examples |
| 70 | `70-goal-runtime/` | 8 (1×v1 + 7×v2) | legado + redesign flags |
| 80 | `80-channel-gateways/` | 1 backlog | Telegram |
| 90 | `90-realtime-voice/` | 1 backlog | full duplex |
| — | `_shared/` | 7 contracts | identity, ownership, security, tools, lifecycle, ordering, leases |
| — | root | `README`, `TDD`, `TRACEABILITY`, `PLAN_AUDIT` | |

**Total:** 9 programas · **33 epics** · **~90 arquivos Markdown** (contado no review).

### 2.2 Substituições estruturais

| Antes | Depois |
|-------|--------|
| `app-server/v1-*` (8 epics, Thread vocabulary) | `30-app-server/v1-01..07` + `v2-01` dashboard |
| `goal-runtime/v1-*` (7 epics como se fossem MVP redesign) | `70-goal-runtime/v1-01` legado + `v2-01..07` |
| `_shared/security-authority-boundaries.md` | **Removido**; conteúdo absorvido/reescrito em `_shared/control-plane-security.md` + ownership |
| (sem Tower/MCP/tools/SDK/providers tree) | programas `10..60` + backlog `80..90` |

### 2.3 Waves (root README)

| Wave | Conteúdo resumido |
|------|-------------------|
| 0 | providers hygiene + tower leader char + session protocol |
| 1 | API-key foundation + multi-session registry + runtime facade |
| 2 | app-server core in-process/stdio + multi-instance |
| 3 | WS auth + MCP (deps reais exigem tools facade antes do MCP — ver misalignment §4.2) |
| 4 | history/approvals + tool contract |
| 5 | ACL/tools parity + SDK + BYOK providers |
| 6 | release hardening conjunto |
| 7+ | Goal v2, dashboard client, gateways, voice |

---

## 3. Matriz de alinhamento handoff §13 → plano

| Decisão handoff | Plano Codex | Alinhado? |
|-----------------|-------------|-----------|
| Termo **session** (não thread) | `_shared/session-turn-item-identity.md`, methods `session/*` | **SIM** |
| Tower = promove leader | `20/*`, DD-02 | **SIM** |
| Multi-session mesmo processo | `20/v1-02` | **SIM** |
| N Towers / machine | `20/v1-03` | **SIM** |
| Session any workspace | lifecycle contract | **SIM** |
| T4 connect-or-spawn | `20/v1-03` + TRACEABILITY | **SIM** |
| WS early + MCP same release | waves 3–4, `30/v1-04`, `40/*` | **SIM** |
| MCP remoto SSE/HTTP | `40/*` | **SIM** |
| Bearer, no fine scopes, Origin free, cleartext OK | `_shared/control-plane-security.md` | **SIM** |
| Threat model honesto | mesmo + HUMAN remote accept | **SIM** |
| `tower_agent_*` MUST set | `_shared/tower-agent-tools.md` | **SIM** |
| hub = Tower, sem tool hub | tools contract | **SIM** |
| In-process tools; MCP config só externas | DD-04, `50/v1-02` | **SIM** |
| ACL orchestrator default | security + tools | **SIM** |
| Peer messaging = v2 study | `50/v2-01` | **SIM** |
| Dashboard intocado MVP | ownership + `30/v2-01` futuro | **SIM** |
| TS SDK MUST | `60/*` | **SIM** |
| Caps sessions free + telemetry later | lifecycle (sem hard cap) | **SIM** (telemetria pode estar fraca — §4) |
| Goal fora core; v1/v2 flags | `70/*` + DAG | **SIM** |
| Gateways/voice out | `80`, `90` backlog | **SIM** |
| BYOK Groq/OR/CF | `10/v1-02..05` | **SIM** |
| TDD method | `TDD.md` | **SIM** |
| Identity grok-oss / ~/.grok-oss | root README | **SIM** |

---

## 4. Desalinhamentos e gaps (materiais)

### 4.1 [MEDIUM][Confirmed] Árvore **não commitada** pelo Codex

- **Evidence:** `git status` dirty com delete de `app-server/`/`goal-runtime/` antigos + untracked `10..90`, `TDD.md`, etc., com HEAD ainda em `967161f`.
- **Impact:** risco de perda; CI/outros agents não veem o plano.
- **Fix:** commit explícito de `.llms/grok-build/**` + este review (esta sessão).

### 4.2 [MEDIUM][Confirmed] Wave 3 no root README **subespecifica** deps do MCP

- **Evidence:** Wave 3 lista `30/v1-04` + `40/v1-01`, mas `40/v1-01-server-transports` **Depende de** `50/v1-01-tool-contract-and-facade` (além de tower multi-instance + app-server core).
- **Impact:** executor que seguir só a tabela de waves pode começar MCP antes do contrato de tools.
- **Fix:** atualizar wave 3/4 no root para: `50/v1-01` antes ou junto de `40/v1-01`; preferir DAG dos READMEs de epic.

### 4.3 [MEDIUM][Likely] Status `planejado` em epics **futuros/backlog** sem review humana

- **Evidence:** 12 epics `planejado` incluem Goal v2 inteiro, Telegram, voice, dashboard migration, peer study; root diz “novos começam rascunho; passam a planejado após review”.
- **Skill:** `planejado` ≈ ready to start after prior ship/review.
- **Impact:** confunde “backlog documentado” com “aprovado para execução”.
- **Fix recomendado:** rebaixar Goal v2 / 80 / 90 / peer / dashboard para `rascunho` **ou** marcar explicitamente `planejado (backlog, não na critical path)` no root.

### 4.4 [LOW][Confirmed] Remoção de `_shared/security-authority-boundaries.md`

- **Evidence:** arquivo deleted no git status; substituído por `control-plane-security.md` + tabela em `runtime-ownership.md`.
- **Impact:** links externos antigos (docs/handoffs) podem apontar path morto; conteúdo de “modelo primário vs verifier vs observer” do doc antigo pode ter encolhido.
- **Fix:** nota de redirect no root README ou restaurar thin stub pointing to new files.

### 4.5 [LOW][Likely] Telemetria de uso/picos por session (T1 handoff) fraca nos epics

- **Evidence:** handoff T1 pede logs de uso/picos; `20-tower-core` ops/hardening menciona operations mas não um epic dedicado a resource telemetry.
- **Impact:** caps futuros sem base de medição.
- **Fix:** task explícita em `20/v1-04` ou epic opcional `v1-05-session-resource-telemetry`.

### 4.6 [LOW][Possible] Soft dep Goal v1 ↔ App Server release

- **Evidence:** `70/v1-01` “Depende de: nenhuma” mas “consome inventário de hot paths de `30/v1-07`”.
- **Impact:** ordem de characterization vs inventário pouco clara.
- **Fix:** declarar `Depende de: soft/optional 30/v1-07` ou “pode iniciar em paralelo; gate final requer inventário”.

### 4.7 [INFO] Thread mentions residuais

- **Evidence:** ~1–2 menções por arquivo de App Server/SDK no sentido **mapping Codex only** (esperado).
- **Impact:** nenhum se contract tests proíbem `thread` nativo (já planejado em riscos).

### 4.8 [INFO] PLAN_AUDIT autoavaliado “provado”

- **Evidence:** `.llms/grok-build/PLAN_AUDIT.md` — útil, mas é self-report do autor do plano.
- **Impact:** não substitui review externa (este arquivo).

### 4.9 [LOW][Confirmed] Estrutura skill: sem `00-foundation` separado

- **Evidence:** prompt sugeriu `00-foundation` opcional; Codex embutiu foundation em `20/v1-01` + `30/v1-01`.
- **Impact:** nenhum material; OK.

### 4.10 Fora de escopo / não desalinhamento

- Telegram/voice não no core — **correto**.
- Sem código produto — **correto**.
- Broken relative links no tree — **0** (check automatizado 2026-07-18).

---

## 5. O que o maintainer ainda precisa responder / decidir

Agrupado para resposta item a item.  
**Prioridade:** P0 = antes de executar wave 0–3 · P1 = antes de freeze público/remoto · P2 = só programas futuros.

### 5.1 P0 — Process / plano

| ID | Pergunta | Contexto |
|----|----------|----------|
| **H-PLAN-1** | Aprova esta árvore como plano canônico (status → `planejado` nos epics **core** 10–60)? | Hoje mix rascunho/planejado |
| **H-PLAN-2** | Confirma rebaixar Goal v2 / 80 / 90 / dashboard / peer para `rascunho` até autorização explícita? | §4.3 |
| **H-PLAN-3** | Aceita correção da tabela Wave 3 (incluir `50/v1-01` antes de MCP)? | §4.2 |

### 5.2 P0/P1 — Surface protocol & product freeze

| ID | Origem | Pergunta |
|----|--------|----------|
| **H-AS-1** | `30/v1-01` tasks | Aceitar missing-`jsonrpc` compatibility listener? |
| **H-AS-2** | `30/v1-01` | Stable vs experimental method inventory para v1 public freeze |
| **H-AS-3** | `30/v1-05` | Delta durability policy (journal) |
| **H-AS-4** | `30/v1-05` | Archive/delete/FTS ownership |
| **H-AS-5** | `30/v1-06` | Controller election/reclaim policy |
| **H-AS-6** | `30/v1-06` | Remote `always` grants? |
| **H-AS-7** | `30/v1-07` | Stable Grok extension inventory |

### 5.3 P1 — Security remoto (já decidido no handoff; falta **aceitação de release**)

| ID | Origem | Pergunta |
|----|--------|----------|
| **H-SEC-1** | handoff R1–R5 + `30/v1-04` + `30/v1-07` | Confirma por escrito o threat model (bearer + cleartext + internet + full control) **antes** de qualquer release com bind público? |
| **H-SEC-2** | root README | Default listen loopback + flag explícita para `0.0.0.0` — OK? (já no security contract; confirmar) |

### 5.4 P1 — SDK / MCP naming `[PROPOSED]`

| ID | Proposta atual | Pergunta |
|----|----------------|----------|
| **H-SDK-1** | `packages/grok-oss-app-server` | Confirma path monorepo? |
| **H-SDK-2** | Node + browser WS no mesmo package | Browser no MVP SDK? |
| **H-SDK-3** | Sem npm publish até freeze | Confirma? |
| **H-MCP-1** | key `grok-oss-tower` ou `tower-<id>` | Nome canônico na config de **clientes externos**? |

### 5.5 P1 — Tower CLI `[PROPOSED]`

| ID | Origem | Pergunta |
|----|--------|----------|
| **H-TOWER-1** | `20/v1-03` | Nomes CLI finais (`grok-oss app-server`, `--tower new`, env URL)? Documentar e congelar |

### 5.6 P2 — Providers (execução)

| ID | Origem | Pergunta |
|----|--------|----------|
| **H-PROV-1** | root / gates live | Credenciais live OR/Groq/CF/Codex quando for smoke |
| **H-PROV-2** | OpenRouter epic | Headers de marketing opt-in? `[PROPOSED]` |
| **H-PROV-3** | byok GAPS (históricos) | MVP providers: só L1 chat-only? multi-key? TUI login? — se não respondidos no BYOK pack, reaparecem no foundation epic |

### 5.7 P2 — Goal (só quando entrar no programa 70)

| ID | Origem |
|----|--------|
| **H-GOAL-1** | session-local vs global SQLite |
| **H-GOAL-2** | auto-resume interativo |
| **H-GOAL-3** | storage location ADR |
| **H-GOAL-4** | MCP authoritative em verification? |
| **H-GOAL-5** | visual verification fallback |
| **H-GOAL-6** | clean auto-apply policy |
| **H-GOAL-7** | rollout thresholds / default version pós opt-in |

### 5.8 P2 — Gateways / voice

| ID | Origem |
|----|--------|
| **H-GW-1** | Telegram Bot API vs MTProto + hosting |
| **H-VOICE-1** | STT/TTS local vs cloud + privacy |

### 5.9 Já travado — **não** reperguntar

Session canônico · Tower/leader · multi-session · multi-tower · WS+MCP early · bearer/R* security · `tower_agent_*` MUST · hub=Tower · in-process vs external MCP · ACL orchestrator · dashboard freeze · T4 connect · Goal fora core · Telegram/voice backlog · TDD method.

---

## 6. Checks estruturais executados (evidência)

| Check | Resultado |
|-------|-----------|
| `git status` / log | dirty tree; HEAD `967161f` — capturado em scratch |
| Contagem `.md` em `.llms/grok-build` | 90 |
| Contagem epics `v*` | 33 |
| Broken relative links | **0** |
| Epics sem Escopo / Riscos | **0** |
| Status counts | 21 `rascunho`, 12 `planejado` |
| Dep cycles (manual sample + PLAN_AUDIT) | nenhum encontrado |
| `thread` nativo vs mapping | só mapping (amostra rg) |
| Handoff MUST tools list | completo em `_shared/tower-agent-tools.md` |

Scratch (goal): `/tmp/grok-goal-f169b8f2b089/implementer/{git-status,tree-files,alignment-notes,human-proposed}.txt`

---

## 7. Recomendações fix-forward (processo)

1. ~~Commit da árvore + review~~ — feito em `bfbebbd` (+ commits posteriores se houver).  
2. Patch Wave 3/4 no root README (DAG real: `50/v1-01` antes de `40/v1-01`).  
3. Normalizar status `planejado` vs backlog.  
4. Stub/redirect para `security-authority-boundaries` se refs externas.  
5. Task telemetria session resource em `20/v1-04`.  
6. **Próximo passo principal:** pass **contract deepening + crate scaffold** (seções §10–§14 + prompt §15). Não reescrever o roadmap do zero.

---

## 8. Veredito final (estrutura vs profundidade)

| Pergunta | Resposta |
|----------|----------|
| Codex cumpriu o prompt de planejamento? | **Sim, em substância** (estrutura, Session, Tower, MCP, tools, TDD, Goal split, providers, backlog). |
| Estava commitado na 1ª análise? | **Não** — commitado depois. |
| Roadmap/ordem/deps prontos? | **Sim (~8.5/10)** |
| Contratos implementation-grade? | **Não (~6/10)** — invariantes bons; wire/schemas/APIs rasos |
| Modularidade de crates definida? | **Não (~5/10)** — “promover leader / facade” sem mapa de crates |
| Tasks acionáveis por @build sem inventar API? | **Não (~5.5–6/10)** — checklists de alto nível |
| Pronto para ship público sem rework? | **Não** |
| Pronto para wave 0–1 characterization + protocol crate? | **Sim, com deepening prévio preferível** |

**Conclusão:** não está perfeito. Está **excelente como ordenação de produto** e **insuficiente como especificação de interface** que o resto do sistema depende. O gap crítico **não** é “falta de epics” — é **falta de contratos verbosos + schemas + scaffold** no path que congela a API pública (Session protocol, Tower lifecycle, `tower_agent_*`, security, MCP surface).

---

## 9. Paths de referência

- `.llms/grok-build/README.md`
- `.llms/grok-build/TDD.md`
- `.llms/grok-build/TRACEABILITY.md`
- `.llms/grok-build/PLAN_AUDIT.md`
- `.llms/grok-build/_shared/*.md`
- `docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md`
- `docs/architecture/prompts/CODEX_PROMPT_PLAN_EPIC_TREE_GROK_OSS.md`
- Prompt deepening: `docs/architecture/prompts/CODEX_PROMPT_CONTRACT_DEEPENING_AND_SCAFFOLD.md`
- Este review: `.llms/reviews/codex-epic-tree-review-2026-07-18.md`

---

## 10. Diagnóstico de profundidade (por que “melhorar tudo”)

### 10.1 O que a árvore atual é

- Árvore de **épicos versionados** com Escopo ADICIONAR/REFACTORIZAR/REMOVER.
- Contratos `_shared/` com **invariantes e ownership**.
- Um contrato App Server Session (~120 linhas) com methods listados em prosa.
- Tools contract com tabela de nomes + error codes (~40 linhas).
- Tasks em muitos epics como **bullet de alto nível** (“Implementar X”).

### 10.2 O que a árvore atual **não** é

- Spec de **wire JSON** por method (request/response/notification examples).
- **JSON Schema** / OpenAPI-like / TypeScript types versionados como artefato.
- **Trait/API Rust** pública por crate (`SessionRegistry`, `RuntimeFacade`, `TowerAgentOps`).
- **State machines** completas com transições ilegais e error codes.
- **Golden vertical slice** event-by-event (session start → turn → tools → complete).
- **Crate layout** com boundaries de compile-time (o que não pode depender de o quê).
- **Compatibility matrix** (native session/* vs Codex thread/* adapter).
- **MCP tool inputSchema/outputSchema** por `tower_agent_*`.
- **Idempotency / cursor / eventSeq** com exemplos de reordering e reconnect.
- **Threat model test cases** enumerados (auth fail matrix).

### 10.3 Contagens (snapshot review)

| Métrica | Valor (aprox.) | Interpretação |
|---------|----------------|---------------|
| Epics | 33 | Bom cobertura de programa |
| Arquivos `tasks.md` | ~16 | Muitos epics só com TODO no README |
| Contratos locais `contracts/` | ~3 | Subespecificado |
| `session-protocol-v1.md` | ~120 linhas | Bom ADR; curto para wire freeze |
| `tower-agent-tools.md` | ~43 linhas | Nomes OK; schemas ausentes |

### 10.4 Risco se implementar sem deepening

1. Três agents implementam três shapes de `session/start`.  
2. MCP e in-process tools divergem em error codes/params.  
3. Shell monólito absorve app-server (merge hell com upstream).  
4. SDK TS vira hand-written e diverge do Rust.  
5. Conformance multi-transport fica impossível (não há golden events).  
6. Segurança remota “funciona no happy path” e falha em edge cases não listados.

---

## 11. Inventário EXAUSTIVO do que falta detalhar

> Codex deve preencher **cada item** abaixo em contratos/schemas/scaffold.  
> Marcar cada item com: `DONE` | `PARTIAL` | `N/A` + path do artefato.  
> Preferir **verbosidade** a elegância. Exemplos concretos > abstração.

### 11.0 Meta / organização da doc

| ID | Entrega |
|----|---------|
| D-00.1 | Índice mestre `contracts/INDEX.md` listando todos os contratos e status |
| D-00.2 | Glossário expandido (Session, Turn, Item, Tower, Instance, Resident, Dormant, Controller, Interaction, IdempotencyKey, eventSeq, historyEpoch, AgentType, CredentialId, …) com **anti-definições** (o que NÃO é) |
| D-00.3 | Mapa “fonte de verdade” por domínio (qual arquivo vence em conflito) |
| D-00.4 | Versionamento de protocolo (`session-protocol` version string, compatibility rules) |
| D-00.5 | Política de breaking change e experimental capabilities |
| D-00.6 | Correção Wave 3/4 no root README (DAG real) |
| D-00.7 | Status policy: o que é `rascunho` vs `planejado` para backlog 70/80/90 |
| D-00.8 | Redirect note para `security-authority-boundaries.md` removido |

---

### 11.1 Crate & module architecture (modularidade)

| ID | Entrega |
|----|---------|
| D-CR.1 | **Crate map canônico** (nomes, paths sob `crates/codegen/`, responsabilidades) |
| D-CR.2 | Dependency DAG de crates (o que **proibido** depender de o quê) |
| D-CR.3 | `xai-grok-app-server-protocol`: só types/serde/schema — **zero** Tokio/IO |
| D-CR.4 | `xai-grok-app-server`: processor, connection, transports (stdio/ws/in-process) |
| D-CR.5 | `xai-grok-tower` (ou nome final): instance registry, multi-session, promote leader — boundary vs `shell::leader` |
| D-CR.6 | `xai-grok-tower-tools` ou módulo em `xai-grok-tools`: `tower_agent_*` |
| D-CR.7 | `xai-grok-mcp-server` (ou feature): MCP server adapter → facade (separado de `xai-grok-mcp` **client**) |
| D-CR.8 | `xai-grok-app-server-client` (Rust) + path TS `packages/…` |
| D-CR.9 | Thin adapters em `xai-grok-shell` / `xai-grok-pager-bin` (CLI) apenas |
| D-CR.10 | Feature flags Cargo por transport/MCP/remote |
| D-CR.11 | Scaffold real: `Cargo.toml` workspace members + `lib.rs` stubs + `mod` tree + `// TODO epic` links |
| D-CR.12 | Tabela file-level ownership: path → epic owner |
| D-CR.13 | Strategy de merge com upstream (minimize touch de monólitos) |
| D-CR.14 | Diagrama mermaid crates + runtime processes |

**Scaffold mínimo exigido (código skeleton, sem business logic completa):**

```text
crates/codegen/xai-grok-app-server-protocol/  # types, errors, schema export hooks
crates/codegen/xai-grok-app-server/           # processor stub, transport traits
crates/codegen/xai-grok-tower/                # TowerHandle / InstanceId stubs
# + packages/grok-oss-app-server/             # TS package skeleton [PROPOSED path]
```

(Ajustar nomes se justificado no doc, mas **fixar um** e não deixar “TBD”.)

---

### 11.2 Session / Turn / Item protocol (CORE VITAL)

Arquivo alvo principal: expandir  
`30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md`  
+ opcionalmente split:

- `contracts/methods.md`
- `contracts/events.md`
- `contracts/errors.md`
- `contracts/examples.jsonl`
- `schemas/*.json` (ou gerados de Rust stubs)

| ID | Entrega |
|----|---------|
| D-SP.1 | Envelope JSON-RPC exact fields + examples (request, response, error, notification, server-request) |
| D-SP.2 | `initialize` params/result **completos** (capabilities object tree) |
| D-SP.3 | `initialized` notification |
| D-SP.4 | Reject-pre-init matrix (cada method class → error code) |
| D-SP.5 | **Session** type: todos os fields, optionality, max sizes, redaction |
| D-SP.6 | **Turn** type: fields, kinds, statuses, transitions |
| D-SP.7 | **Item** type: discriminated union de **todos** kinds planejados no MVP + extension point |
| D-SP.8 | Method inventory table: method \| side \| params schema \| result \| errors \| idempotent? \| scope lock |
| D-SP.9 | `session/start` full contract + 3 examples (happy, invalid cwd, unauthorized) |
| D-SP.10 | `session/resume`, `fork`, `read`, `list`, `subscribe` — cada um com examples |
| D-SP.11 | `turn/start`, `steer`, `interrupt` — examples + concurrent rules |
| D-SP.12 | Notifications: `session/*`, `turn/*`, `item/started`, `item/*delta`, `item/completed` — payload schemas |
| D-SP.13 | Server requests: approvals/questions/MCP elicitation — Interaction model |
| D-SP.14 | `eventSeq` / item revision / history epoch — formal rules + examples of invalid client cursor |
| D-SP.15 | Snapshot-then-live subscribe algorithm (pseudocode + sequence diagram) |
| D-SP.16 | Idempotency key semantics + conflict examples |
| D-SP.17 | Error catalog: **numeric/string codes**, retryable?, safe message rules |
| D-SP.18 | Backpressure: queue limits, drop/coalesce policy, client-visible errors |
| D-SP.19 | Transport invariance statement + per-transport framing (stdio NDJSON, WS frames, in-process) |
| D-SP.20 | Codex adapter mapping table `thread/* ↔ session/*` (isolated crate/module) |
| D-SP.21 | Golden scenario JSONL: full coding turn (user text → agent message → tool → result → completed) |
| D-SP.22 | Golden scenario: interrupt mid-turn |
| D-SP.23 | Golden scenario: multi-session concurrent turns |
| D-SP.24 | Golden scenario: reconnect after disconnect during turn |
| D-SP.25 | Compatibility decisions: missing `jsonrpc` (default deny vs adapter) — document HUMAN gate |
| D-SP.26 | Rust type stubs mirroring schemas (serde) + roundtrip tests that compile |
| D-SP.27 | JSON Schema files checked in (even if later generated) **or** generate script stub + first golden schema |
| D-SP.28 | TypeScript types (hand-authored first OK if mark “will be generated”) matching schema |
| D-SP.29 | Max message size, max list page size, timeouts defaults |
| D-SP.30 | Non-goals list (what protocol will NOT do in v1) |

---

### 11.3 Tower instance & multi-session lifecycle (CORE VITAL)

Expandir `_shared/tower-instance-lifecycle.md` + contracts em `20-tower-core/`.

| ID | Entrega |
|----|---------|
| D-TW.1 | `TowerInstanceId` format |
| D-TW.2 | State dir layout `~/.grok-oss/towers/<id>/` (files, permissions, secrets split) |
| D-TW.3 | Default instance selection algorithm (env, socket path, lock file) |
| D-TW.4 | connect-or-spawn exact state machine (flowchart + failure modes) |
| D-TW.5 | Multi-instance isolation: ports, tokens, session steals forbidden |
| D-TW.6 | Session residency: resident vs dormant vs archived vs dead |
| D-TW.7 | Mapping SessionId ↔ disk path `~/.grok-oss/sessions/...` |
| D-TW.8 | “any workspace” rules: path canonicalization, symlink policy, sandbox interaction |
| D-TW.9 | Soft telemetry: metrics to log (rss, open FDs, active turns, queue depth) — peaks |
| D-TW.10 | No hard cap formalization + future config knobs |
| D-TW.11 | Relationship to existing leader protocol (byte-level preserve list for ACP) |
| D-TW.12 | Characterization test list (file + assertion names) for leader behavior |
| D-TW.13 | Public Rust API sketch: `TowerHandle`, `open_session`, `list_sessions` |
| D-TW.14 | Thread-safety / actor model notes (who runs on which runtime) |
| D-TW.15 | Shutdown/drain/restart semantics |

---

### 11.4 Runtime facade (App Server ↔ SessionActor)

| ID | Entrega |
|----|---------|
| D-RF.1 | Trait `GrokRuntimeFacade` (or final name) full method list |
| D-RF.2 | Mapping facade method → SessionActor/command channel ops |
| D-RF.3 | Event model: enum of runtime events → protocol Items (table) |
| D-RF.4 | Projection rules: what is dropped/redacted |
| D-RF.5 | One-actor invariant tests |
| D-RF.6 | Fake runtime for conformance (behavior requirements) |
| D-RF.7 | Explicit non-duplication: no second SessionActor path |

---

### 11.5 `tower_agent_*` tools (CORE VITAL)

Expandir `_shared/tower-agent-tools.md` + per-tool files se necessário  
`50-tower-agent-tools/v1-01-…/contracts/tools/*.md`

Para **cada** tool MUST:

| Tool | Specs required |
|------|----------------|
| `tower_agent_list` | params, filters, pagination, result rows, errors |
| `tower_agent_start` | workspace, agent_type, model?, idempotency, result session |
| `tower_agent_send` | session_id, input blocks, steer vs new turn rules |
| `tower_agent_history` | mode full\|last, cursor, max_bytes, redaction |
| `tower_agent_interrupt` | idempotent interrupt semantics |
| `tower_agent_resume` | dormant rules |
| `tower_agent_archive` | vs delete |
| `tower_agent_status` | fields safe summary |
| `tower_agent_wait` | timeout, cursor, wakeup conditions, no lock hold |

| ID | Entrega |
|----|---------|
| D-TA.1 | Per-tool JSON Schema input |
| D-TA.2 | Per-tool JSON Schema output |
| D-TA.3 | Shared error codes with examples |
| D-TA.4 | Idempotency keys per mutating tool |
| D-TA.5 | ACL matrix agent_type × tool |
| D-TA.6 | In-process tool registration (how appears to model) |
| D-TA.7 | MCP tool descriptors (name, description, inputSchema) |
| D-TA.8 | Semantic parity suite definition (same cases MCP vs in-process) |
| D-TA.9 | Forbidden: `tower_agent_hub` |
| D-TA.10 | v2 peer messaging boundary (what is explicitly out) |
| D-TA.11 | Examples for multi-session swarm orchestration |
| D-TA.12 | Size limits & timeout defaults |

---

### 11.6 Security & auth (CORE VITAL)

Expandir `_shared/control-plane-security.md`.

| ID | Entrega |
|----|---------|
| D-SEC.1 | Token format (entropy, encoding, storage paths, permissions 0600) |
| D-SEC.2 | Bearer extraction rules (header only; reject query/argv) |
| D-SEC.3 | Authn failure matrix (missing/invalid/revoked/expired) |
| D-SEC.4 | Full-control meaning: exact method/tool allow set (everything) |
| D-SEC.5 | Explicit **no scopes** / **no Origin check** in MVP + future section |
| D-SEC.6 | Bind defaults: loopback; non-loopback requires flag; warning text |
| D-SEC.7 | Cleartext `http`/`ws` allowed; TLS optional path documented |
| D-SEC.8 | Redaction rules (bearer, provider keys, canaries lengths) |
| D-SEC.9 | Audit log fields (no secrets) |
| D-SEC.10 | Rate/size limits defaults |
| D-SEC.11 | Threat model scenarios (MITM, stolen token, malicious webpage, LAN attacker) |
| D-SEC.12 | Test plan security (list of tests) |
| D-SEC.13 | HUMAN gate checklist before public bind release |

---

### 11.7 MCP control plane

| ID | Entrega |
|----|---------|
| D-MCP.1 | Transports: stdio + Streamable HTTP/SSE exact endpoints |
| D-MCP.2 | Auth for HTTP MCP (same bearer) |
| D-MCP.3 | Tool list = `tower_agent_*` only (or enumerated extras) |
| D-MCP.4 | Mapping MCP call → facade → protocol effects |
| D-MCP.5 | No auto-register local Tower as MCP client of itself |
| D-MCP.6 | External MCP client config example (`mcp_servers.grok-oss-tower`) |
| D-MCP.7 | Conformance cases shared with App Server suite |
| D-MCP.8 | Relationship to existing `xai-grok-mcp` **client** crate (no merge confusion) |
| D-MCP.9 | Scaffold crate/module for server |
| D-MCP.10 | Flags CLI: `--mcp off|stdio|http://…` co-start with app-server |

---

### 11.8 Transports & CLI

| ID | Entrega |
|----|---------|
| D-TR.1 | stdio framing, EOF, logging on stderr only |
| D-TR.2 | WebSocket: subprotocol name, ping/pong, max frame |
| D-TR.3 | in-process client API |
| D-TR.4 | Optional unix socket / IPC (if in MVP or defer) |
| D-TR.5 | CLI surface `grok-oss app-server …` full flag matrix |
| D-TR.6 | CLI `tokens create/list/revoke` if MVP |
| D-TR.7 | Health endpoints (`/healthz`/`/readyz`) if any — no session data |
| D-TR.8 | Co-start flags app-server + MCP combinations matrix |

---

### 11.9 TypeScript SDK

| ID | Entrega |
|----|---------|
| D-TS.1 | Package path + name freeze `[PROPOSED]` or HUMAN |
| D-TS.2 | Public client class API (connect, initialize, session*, turn*, subscribe) |
| D-TS.3 | Event iterator / async stream design |
| D-TS.4 | Example scripts (stdio + ws) checked into monorepo |
| D-TS.5 | Generation pipeline from Rust (or interim hand types with drift test) |
| D-TS.6 | Browser vs Node differences |
| D-TS.7 | Error type mapping |

---

### 11.10 Approvals / controller / history

| ID | Entrega |
|----|---------|
| D-AP.1 | Controller lease state machine |
| D-AP.2 | Interaction ID vs request ID |
| D-AP.3 | Disconnect during approval |
| D-AP.4 | History store vs projection SQLite (what is MVP) |
| D-AP.5 | Replay cursors and epoch invalidation examples |
| D-AP.6 | HUMAN decisions listed with defaults if safe |

---

### 11.11 Providers (Codex + BYOK)

| ID | Entrega |
|----|---------|
| D-PR.1 | Provider descriptor schema (API-key providers) |
| D-PR.2 | Binding immutability rules reused from multi-auth |
| D-PR.3 | Catalog key format per provider |
| D-PR.4 | Onboarding flow CLI steps per OR/Groq/CF |
| D-PR.5 | Test fixtures requirements (no live required) |
| D-PR.6 | Link to `docs/architecture/byok-providers-onboarding/*` as normative inputs |
| D-PR.7 | Hygiene epic mapping to open issues (docs-001, testing-001, …) |

---

### 11.12 Goal (plan only deepening; not core scaffold)

| ID | Entrega |
|----|---------|
| D-GO.1 | Flag `goal_runtime = v1|v2|disabled` contract |
| D-GO.2 | Hot-path inventory template App Server must fill |
| D-GO.3 | Dual-version test strategy |
| D-GO.4 | Explicit: no Goal v2 crate scaffold in this pass unless trivial stubs |

---

### 11.13 TDD deepening

| ID | Entrega |
|----|---------|
| D-TD.1 | Expand `TDD.md` with **named test modules** per core crate |
| D-TD.2 | Conformance suite layout (one tests crate or `tests/conformance`) |
| D-TD.3 | RED/GREEN evidence format for epic completion |
| D-TD.4 | List of golden JSONL files to create |
| D-TD.5 | Security tests list |
| D-TD.6 | Commands matrix updated for new crates |

---

### 11.14 Dashboard / ACP freeze

| ID | Entrega |
|----|---------|
| D-UI.1 | Explicit “do not modify” file list / surfaces for MVP |
| D-UI.2 | Roster ACP methods remain source for dashboard |
| D-UI.3 | Future migration epic boundary only |

---

### 11.15 Gateways / voice (docs only)

| ID | Entrega |
|----|---------|
| D-BK.1 | Keep backlog; add “consumes contracts X/Y/Z” only |
| D-BK.2 | No Telegram/voice implementation or deep schema in this pass |

---

### 11.16 Epic tasks rewrite quality bar

For **each** epic in programs **20, 30, 40, 50, 60** (core):

| ID | Entrega |
|----|---------|
| D-TK.1 | Prefer separate `tasks.md` if >10 items |
| D-TK.2 | Every implementation task names: crate path, test command, acceptance observation |
| D-TK.3 | Every task links to a contract section ID (D-SP.x / D-TA.x …) |
| D-TK.4 | Layers named by real work phases, not “Foundation/Core” genérico |
| D-TK.5 | HUMAN tasks keep type + blocking |

---

## 12. Prioridade do que é “vital” (não pode errar)

Ordem de freeze (o resto do sistema depende disto):

```text
P0-VITAL (scaffold + contracts first)
  1. Crate/module map + workspace scaffold (D-CR.*)
  2. Session protocol wire + golden JSONL (D-SP.*)
  3. Tower lifecycle + connect-or-spawn (D-TW.*)
  4. tower_agent_* schemas + ACL (D-TA.*)
  5. Security bearer/threat tests list (D-SEC.*)
  6. Runtime facade trait boundary (D-RF.*)

P1 (same release, after P0 docs/scaffold)
  7. MCP server surface (D-MCP.*)
  8. Transport/CLI flags (D-TR.*)
  9. TS types + examples skeleton (D-TS.*)
 10. TDD conformance layout (D-TD.*)

P2 (can stay thinner)
  Providers flows, Goal flags, gateways/voice backlog
```

**Regra:** nenhum epic de implementação de processor/MCP/tools avança a “código de negócio” sem P0-VITAL marcado DONE nos contratos.

---

## 13. Scaffold vs implementação

| Permitido neste pass do Codex | Proibido |
|-------------------------------|----------|
| Criar crates skeleton, `lib.rs`, modules vazios, traits, types serde | Implementar processor completo / leader rewrite real |
| JSON Schema + examples JSONL | Ship production auth without tests |
| Compile-time empty impls `todo!()` / `unimplemented!()` behind cfg | Silent behavior change in TUI/dashboard |
| Unit tests that lock **type shapes** and schema roundtrip | Live network provider calls |
| Expand markdown contracts extensively | Reopen handoff locked product decisions |
| Fix Wave 3 README DAG | Rewrite Goal v2 as core dependency |

---

## 14. Definition of Done deste próximo pass Codex

- [ ] Todos os itens **P0-VITAL** (seção 12) têm artefato path-cited  
- [ ] Scaffold crates adicionados ao workspace e `cargo check -p <new>` passa (stubs)  
- [ ] Golden JSONL ≥ 3 cenários  
- [ ] `tower_agent_*` input/output schemas para as 9 tools  
- [ ] Security matrix + threat scenarios documentados  
- [ ] Crate dependency DAG documentado e enforced (no shell→protocol inversion se possível)  
- [ ] Root README waves corrigidas  
- [ ] `contracts/INDEX.md` + checklist D-* com status  
- [ ] Update TRACEABILITY.md se paths mudarem  
- [ ] Resumo executivo no final da sessão Codex  

---

## 15. PROMPT CABULOSO PARA O CODEX (copiar)

> Arquivo duplicado para copiar fácil:  
> `docs/architecture/prompts/CODEX_PROMPT_CONTRACT_DEEPENING_AND_SCAFFOLD.md`

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

---

## 16. Como o maintainer usa isto

1. Abra o Codex no repo `grok-goblin`.  
2. Cole o bloco `BEGIN_PROMPT`…`END_PROMPT` (ou o arquivo em `docs/architecture/prompts/CODEX_PROMPT_CONTRACT_DEEPENING_AND_SCAFFOLD.md`).  
3. Exija na resposta a **tabela D-*** completa.  
4. Depois do pass, rode outra review (ou peça review) só de P0-VITAL antes de `@execute-plan` em processor real.

**Não** comece implementação larga do App Server sem o P0-VITAL fechado.
