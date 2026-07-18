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

## 7. Recomendações fix-forward (sem reescrever a árvore nesta sessão)

1. **Commit** da árvore + este review.  
2. Patch pequeno no root README: Wave 3/4 alinhadas ao DAG real (`50/v1-01` → `40/v1-01`).  
3. Normalizar status: core `rascunho`→`planejado` **só após** H-PLAN-1; backlog futuro `rascunho`.  
4. Stub/redirect para `security-authority-boundaries` se ainda houver refs externas.  
5. Task de session resource telemetry em `20/v1-04`.  
6. Não iniciar implementação até H-PLAN-1 (+ H-SEC-1 antes de bind público).

---

## 8. Veredito final

| Pergunta | Resposta |
|----------|----------|
| Codex cumpriu o prompt de planejamento? | **Sim, em substância** (estrutura, Session, Tower, MCP, tools, TDD, Goal split, providers, backlog). |
| Estava commitado? | **Não** (no momento da análise). |
| Plano implementation-ready? | **Quase** — estrutura e deps OK; falta commit + limpeza wave/status + respostas humanas P0/P1 para freeze. |
| Bloqueia execução de wave 0? | **Processo:** commit + ok humano no plano. **Produto:** decisions H-AS-* não bloqueiam spike de characterization. |

---

## 9. Paths de referência

- `.llms/grok-build/README.md`
- `.llms/grok-build/TDD.md`
- `.llms/grok-build/TRACEABILITY.md`
- `.llms/grok-build/PLAN_AUDIT.md`
- `.llms/grok-build/_shared/*.md`
- `docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md`
- `docs/architecture/prompts/CODEX_PROMPT_PLAN_EPIC_TREE_GROK_OSS.md`
- Este review: `.llms/reviews/codex-epic-tree-review-2026-07-18.md`
