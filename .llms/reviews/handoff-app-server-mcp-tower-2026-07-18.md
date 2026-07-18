# HANDOFF — Auditoria + Visão de Plataforma (App Server, MCP, Agent Tower)

| Campo | Valor |
|-------|--------|
| **Path canônico** | `docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md` |
| **Tipo** | Handoff para outra IA (planejamento / consolidação de planos) |
| **Data** | 2026-07-18 |
| **Produto** | **grok-oss** (fork Goblin de `xai-org/grok-build`) |
| **Branch auditada** | `goblin-multi-provider-codex` @ `0285ee5` |
| **Remotes** | `origin` = `xai-org/grok-build` (upstream); `fork` = `nonexphere/grok-build` |
| **Status** | Handoff + **respostas humanas parciais (2026-07-18)**. Ainda **não** é plano de implementação aprovado. |
| **Transcrição** | `docs/architecture/transcripts/2026-07-18-user-intent-app-server-mcp-tower.md` |
| **Respostas** | §13 (preenchidas) · Pendências §14 |

**Instrução:** usar este documento como input para consolidar/reescrever planos em `.llms/grok-build/` e/ou `docs/architecture/`. **Não** implementar código a partir deste arquivo até o humano aprovar o plano consolidado. Priorizar §13 + §1 + transcrição sobre defaults antigos da §6 quando houver conflito.

---

## 0. Missão da próxima IA (o que você deve fazer com este doc)

Você (IA seguinte) deve:

1. **Ler este handoff por completo** e as fontes canônicas listadas em §11.
2. **Consolidar / reescrever planos** de App Server + MCP control plane + Agent Tower de forma coesa, maintainable e versionada (árvore `.llms/grok-build/` e/ou `docs/architecture/`).
3. **Concretizar o plano App Server existente** com os requisitos do usuário (§1) que hoje estão ausentes ou só implícitos.
4. **Separar** trabalho de:
   - multi-provider/Codex (já avançado no código),
   - identidade/distribuição grok-oss,
   - Goal Runtime (plano, sem crates novas),
   - **App Server + MCP + Tower** (principal pedido do usuário).
5. Produzir planos **implementation-ready** (épicos, contratos, tasks, gates, decisões humanas).
6. **Não implementar código** até o humano aprovar o plano consolidado.

Skills sugeridas na fase de planejamento (quando autorizado):
- `@architecture-spec-authoring`
- `@plan-epic-tree`
- `@repository-exploration` (para spikes de ownership)
- `@code-audit` (só se revalidar findings)
- **Não** rodar `@execute-plan` / `@implementation-loop` até aprovação.

---

## 1. Visão de produto pedida pelo humano (requisitos normativos da sessão)

O humano quer evoluir o **fork grok-oss** (base: `xai-org/grok-build` closed-contrib) para uma plataforma controlável por API, não só TUI.

### 1.1 App Server (obrigatório)

- API **WebSocket** completa (além de stdio/IPC).
- Interface **próxima do Codex app-server** (JSON-RPC Thread/Turn/Item).
- **TypeScript API/SDK** gerado (como Codex: `generate-ts` / schema).
- Referência local de protocolo Codex:  
  - `~/codex-app-server.md`  
  - `~/brainstorm/codex-connector/schemas/codex-app-server/`  
  - Spec Grok: `changes/grok_app_server_spec_bundle/`

### 1.2 MCP control plane (junto do app-server)

- Ao subir o app-server, **também** deve ser possível expor um **endpoint MCP** que permite:
  - listar/acessar **sessões/threads**,
  - mandar mensagens,
  - orquestrar agents,
  - controlar turns (start/send/interrupt/resume etc.).
- **Flags** para modos:
  - só app-server,
  - só MCP,
  - **ambos** (default desejável quando daemon “completo”).
- **Tokens** (definir, listar, revogar, scopes) — auth do control plane, **não** confudir com tokens de provider/modelo.

### 1.3 Agent Tower (novo orquestrador / peer sessions)

- Nova **tool** (ou família de tools) que permite agentes se conectarem a **outras sessões Grok de verdade**, **não** apenas `spawn_subagent`.
- Hoje: subagents têm **depth limit = 1** (filho não spawna); orquestração multi-hop é forçada no primary (`docs/runtime/turn-queue-subagents-and-followups.md`).
- Visão: agents criam/gerenciam **sessions/threads top-level** (ou first-class peer threads) e **comunicam entre si**.
- **ACL por tipo de agent**: nem todo agent acessa a “tower”.
  - **Com tower:** `orchestrator` (e possivelmente roles explicitamente autorizados).
  - **Sem tower (default):** `build`, `explore`/`repo-explore`, agents “normais”, etc.
- Analogia operacional forte no ambiente do humano: **`~/mcps/codex-bus-mcp`**  
  (`agents_start`, `agents_send`, `agents_resume`, `agents_interrupt`, `agents_archive`, capabilities, inbox, host bridge → app-server privado).

### 1.4 Qualidade de implementação

- Código **fácil de manter**, bem organizado.
- Preferir **uma semantic core** + adapters (já é princípio do plano App Server).
- Não duplicar `SessionActor` / segundo runtime paralelo.

### 1.5 Nota de ASR / ambiguidade

- “**AP-7**” no speech-to-text: interpretado como **App Server** (programa + épicos em `.llms/grok-build/app-server/`).  
  Alternativa: **Phase 7** do spec (TUI migration). A próxima IA deve tratar App Server **como programa inteiro** e marcar Phase 7 como um epic, não o escopo total.
- “**estibordo**”: interpretado como superfície de controle existente (TUI multi-client / leader / ACP / “board” de orquestração). Se o humano quiser outra leitura (produto Starboard, etc.), confirmar.

---

## 2. Estado atual do harness (mapa factual)

### 2.1 Produto e identidade

| Surface | Valor atual | Status |
|--------|-------------|--------|
| Binary | `grok-oss` (+ alias `goblin`) | shipped |
| Home | `~/.grok-oss` (`GROK_OSS_HOME` / fallback `GROK_HOME`) | shipped |
| npm | `@brasalabs/grok-oss` + platform packages | pack OK; publish bloqueado em `NPM_TOKEN` |
| Crates | nomes internos `xai-grok-*` | **sem rename em massa** (política) |
| Upstream | `xai-org/grok-build` | mirror em `main` only |

Docs: `GOBLIN.md`, `docs/architecture/GROK_OSS_IDENTITY_AND_DISTRIBUTION_PLAN.md`, `TO_RELEASE_NPM.md`.

### 2.2 Multi-provider / Codex (código avançado; não é o App Server)

- Crate principal: `xai-grok-multi-auth` + hooks em `xai-grok-shell` / sampler.
- Path offline Codex: catalog `codex/{credential_id}/{slug}`, BearerResolver, 401 attempt-bound, wire OpenResponses, etc. (`TO_RELEASE.md`).
- Login Codex **fail-closed** sem `GROK_CODEX_OAUTH_APPROVED=1` (D10).
- Ledger: `docs/architecture/multi-provider-auth/PROGRESS.md` — **não** full multi-provider 1.0 READY (keyring, xAI adapter multi, subagent multi-account, etc. deferred).

### 2.3 Runtime de sessão / orquestração hoje

| Peça | Onde | Papel |
|------|------|-------|
| `SessionActor` / `SessionHandle` | `xai-grok-shell` | verdade de sessão, turn, tools, permissions |
| `MvpAgent` | `xai-grok-shell` | agent surface, models, ACP, init |
| **Leader** multi-client | `xai-grok-shell/src/leader/` | IPC multi-client, reconnect, capabilities — **base para promover a app-server** |
| ACP | `xai-acp-lib` + shell/pager | protocolo client↔agent atual (TUI/headless) |
| Subagents | `agent/subagent/`, tool `task`/`spawn_subagent` | depth≤1, FG/BG, **não** peer session |
| MCP **client** | `xai-grok-mcp` | Grok **consome** MCP servers externos |
| Tools | `xai-grok-tools` | tool runtime |
| TUI | `xai-grok-pager` | cliente rico; `AcpUpdateTracker` |
| CLI entry | `xai-grok-pager-bin` | `run_leader`, `run_stdio_agent`, auth CLI |

**Inexistente no monorepo (confirmado por busca):**
- crates `xai-grok-app-server*`
- comando `grok-oss app-server`
- **MCP server** de control plane (Grok como servidor MCP de orquestração)
- tool “tower” / peer-session create

### 2.4 Planos já existentes (só docs; status `rascunho`)

Raiz: `.llms/grok-build/README.md`

```text
Goal Runtime (7 épicos)  ── independente de App Server
App Server (8 épicos)    ── consome GoalService; não é dono de goals
```

| Grupo | Épico | Status código |
|-------|-------|---------------|
| App Server | v1-architecture-protocol … v1-ecosystem-ga | **0% implementado** (só seed/spec) |
| Goal Runtime | v1-characterization-domain … v1-recovery-rollout | **0% do redesign**; `/goal` legado no SessionActor |
| Spec fonte App | `changes/grok_app_server_spec_bundle/` | proposta (~1910 linhas + schema/TS seed) |
| Spec fonte Goal | `changes/grok-build-goal-runtime-technical-spec (1).md` | proposta |

**Estimativa do spec App Server:** 31–44 person-weeks; MVP in-process 8–12 person-weeks.

### 2.5 O que o plano App Server **já cobre bem**

- Promover **leader**, não segundo daemon runtime.
- Protocolo Thread/Turn/Item JSON-RPC 2.0 próximo do Codex.
- Transportes: in-process, stdio, IPC, WebSocket.
- Runtime facade + item projector + projection SQLite rebuildable.
- Approvals / controller leases / reverse requests.
- Tokens remotos, pairing, scopes (`threads:read`, `turns:start`, …).
- CLI: `grok app-server --stdio|--socket|--listen ws://…`, `tokens list|revoke`.
- Feature flags sugeridos: `app_server`, `app_server_websocket`, `app_server_remote_control`, …
- TypeScript gerado a partir de tipos Rust.
- ACP permanece via adapter; Codex-compat adapter separado.

### 2.6 O que o plano App Server **NÃO cobre** (gaps críticos vs visão do humano)

| Requisito do humano | Estado no plano atual |
|--------------------|------------------------|
| MCP **server** control plane (`/mcp` ou stdio MCP) | **Ausente** — “MCP” no plano = client/elicitation/tools do runtime, não control plane |
| Co-start app-server + MCP com **flags** (only-one / both) | **Ausente** |
| Tools de agent para criar **sessões peer** (não subagent) | **Ausente** (só Items de subagent tree) |
| ACL por **agent type** (tower allowlist) | **Ausente** |
| Inbox / messaging agent↔agent | **Ausente** (codex-bus tem; Grok plan não) |
| Host-bridge pattern (processo por agent + MCP inject) | **Ausente** (existe no `codex-bus-mcp`) |
| Integração product name `grok-oss` / home `~/.grok-oss` | Spec ainda fala `grok` / `~/.grok` em vários trechos |

---

## 3. Arquitetura alvo proposta (para a próxima IA consolidar — **não implementada**)

```text
                    ┌──────────────────────────────────────────┐
   External clients │  TUI / VS Code / SDK TS / automation     │
                    └───────────────┬──────────────────────────┘
                                    │ JSON-RPC (stdio|IPC|WS)
                                    v
                    ┌──────────────────────────────────────────┐
                    │  App Server (processor único)            │
                    │  Thread/Turn/Item · auth scopes · tokens │
                    └───────────────┬──────────────────────────┘
                                    │ Runtime facade
                                    v
                    ┌──────────────────────────────────────────┐
                    │  Grok Runtime (SessionActor / leader)    │
                    │  tools · providers · sandbox · subagents │
                    └───────────────┬──────────────────────────┘
                                    │
          ┌─────────────────────────┼─────────────────────────┐
          v                         v                         v
   MCP Client (existente)   Agent Tower tools          MCP Server (NOVO)
   (servers externos)       (orchestrator-only)        agents_* / sessions_*
                                                          │
                                                          │ same facade
                                                          v
                                               Thread registry / peer sessions
```

### 3.1 Princípios de ownership (já nos contratos `_shared/`)

- **App Server** = protocolo, conexões, subscriptions, leases, replay projection.
- **Runtime Grok** = execução, tools, permissions, sandbox, session files.
- **Goal Runtime** (se/quando) = lifecycle de goals; App Server só projeta.
- **MCP control plane** deve ser **adapter fino** sobre a **mesma** facade do App Server — **não** segundo semantic core.
- **Tower tools** = clients privilegiados da facade (como um “SDK interno”), gated por agent capability.

### 3.2 Relação com `codex-bus-mcp` (referência de produto, não copy cego)

Padrões a **estudar e adaptar**:
- tools `agents_start|send|resume|interrupt|archive`
- capabilities por agent
- registry SQLite de instances/messages/hosts
- HTTP Streamable MCP + WS `/connect` para hosts
- injeção de MCP de volta na thread gerenciada

O que **não** copiar sem decisão:
- depender de processo `codex app-server` externo
- nomes `codex-*` / home `~/.codex-bus`
- host multi-machine enterprise sem threat model

### 3.3 Subagent vs Peer Session (contrato que o plano deve travar)

| Dimensão | Subagent (hoje) | Peer / Tower session (novo) |
|----------|-----------------|-----------------------------|
| Identity | child sob parent session | thread/session first-class |
| Depth | max 1 | N peers; policy limita quem spawna |
| Lifecycle | cancel do parent pode matar FG | lifecycle independente (policy) |
| Tools | subset / capability mode | profile do agent type |
| Visibility | Tasks pane do parent | listável via app-server + MCP |
| Auth model/provider | pin/herança | binding próprio (multi-provider) |
| Comunicação | return result ao parent | inbox/send + shared events |

**Decisão humana necessária:** peer sessions são:
- (A) threads no **mesmo** process/daemon, ou  
- (B) processos/leaders isolados (padrão codex-bus host), ou  
- (C) híbrido (local A, remote B).

Recomendação de handoff: **(A) no v1** (mesmo app-server/runtime registry); (B) só se isolamento forte for P0.

---

## 4. Inventário de issues / riscos (audit)

### 4.1 Issues persistidas em `.agents/issues/`

| ID | Status | Severity | Resumo |
|----|--------|----------|--------|
| data-001 | **DONE** | HIGH | 401 recovery FIFO — fechado |
| data-002 | **DONE** | HIGH | compaction cache key account scope — fechado |
| data-003 | **DONE** | HIGH | Codex-only TUI login gate — fechado |
| testing-002 | **DONE** | LOW | `git diff --check` whitespace — fechado |
| **docs-001** | **OPEN** | MEDIUM | docs de release contradizem evidência (TO_RELEASE vs remediation) |
| **docs-002** | **OPEN** | MEDIUM | artefato `grok-luna-harness-audit-2026-07-17.md` ausente |
| **testing-001** | **OPEN** | MEDIUM | live test Codex “passa” quando skipped |
| **operations-001** | **OPEN** | MEDIUM | tip rebased sem revalidação completa |
| UI-MODEL-IDENTITY-001 | **DONE** | MEDIUM | system_prompt_label sticky — fechado |

### 4.2 Achados de auditorias recentes (`.llms/reviews/`)

- `code-audit-grok-goblin-upstream-regression-fix-2026-07-17.md` — Highs de auth Codex-only / FIFO / compaction (muitos **corrigidos** depois em `cd26c75` + data-00x DONE).
- Várias audits Codex provider 2026-07-16/17 — usar como histórico; reconciliar status em `TO_RELEASE.md` (docs-001).
- App Server README lista riscos estruturais **ainda válidos** (sem implementação):
  - risco de control plane / actor duplicado;
  - IDs/event ordering ainda não contrato estável no código;
  - schema/TS em `changes/` é seed, não codegen;
  - decisões humanas abertas (controller, strictness, remote auth);
  - projection DB não existe;
  - TUI parity/reconnect não demonstrados.

### 4.3 Gaps arquiteturais / produto (novos — **ainda sem issue file**)

Estes **não** estão materializados como `.agents/issues/*`. A próxima IA deve, após plano aprovado, materializar via `@issue-lifecycle` ou tasks de épico.

| ID sugerido | Severity | Type | Finding |
|-------------|----------|------|---------|
| ARCH-AS-001 | Critical (product) | architecture-gap | **Zero** implementação App Server; só specs |
| ARCH-MCP-001 | High | architecture-gap | Sem MCP **server** de orquestração de sessões |
| ARCH-TOWER-001 | High | architecture-gap | Sem peer sessions; orquestração limitada a subagent depth=1 |
| ARCH-ACL-001 | High | security / product | Sem modelo de capability “tower” por agent type |
| ARCH-FLAG-001 | Medium | config-gap | Sem flags CLI de co-start app-server/MCP/tokens |
| ARCH-ID-001 | Medium | contract-drift | Specs ainda falam `grok`/`~/.grok`; produto é `grok-oss`/`~/.grok-oss` |
| ARCH-GOAL-001 | Medium | plan-debt | Goal Runtime planejado mas acoplado legado; integração App Server só no epic final |
| RISK-DUP-001 | High | maintainability | Risco clássico: segundo SessionActor / segundo processor se MCP e App Server divergirem |
| RISK-AUTH-001 | High | security | Tokens de control plane vs credentials de provider precisam planes separados (spec §9.1 já avisa) |
| RISK-REMOTE-001 | High | security | WS remote deny-by-default; Codex WS ainda experimental/unauth em defaults legados — **não copiar** |
| MP-1.0-xxx | Medium | deferred product | Multi-provider 1.0 (keyring, D10 product, multi-account subagent) — paralelo, não bloquear MVP App Server se escopo for local |
| BYOK-Q* | — | open questions | `docs/architecture/byok-providers-onboarding/GAPS_AND_QUESTIONS.md` — fora do núcleo Tower, mas competem por atenção |

### 4.4 Runtime pain points já documentados (não bugs “novos”, mas design constraints)

Fonte: `docs/runtime/turn-queue-subagents-and-followups.md`

- Turn “travado” = parent waiting FG shell/subagent/wait.
- Fila de prompts só drena em idle.
- Depth 1 força orquestrador primary — **motivação direta da Tower**.
- Tasks pane (`Ctrl+B`) é UI de observação de subagents — Tower precisará superfície equivalente em TUI + app-server Items.

---

## 5. Concretização recomendada do plano (rascunho para a próxima IA)

> Isto é **orientação de planejamento**, não plano aprovado. A próxima IA deve expandir em épicos/tasks reais.

### 5.1 Programas de entrega (sugeridos)

| Programa | Prioridade | Depende de |
|----------|------------|------------|
| **P0 — App Server Core** | P0 | nenhum código novo; specs existentes |
| **P1 — MCP Control Plane** | P0/P1 | P0 facade mínima (mesmo processor) |
| **P2 — Agent Tower** | P1 | P0 Thread registry + ACL + (ideal) P1 |
| **P3 — TS SDK + generate** | P1 | protocol crate |
| **P4 — Goal Runtime** | P2 (paralelo fraco) | não bloqueia MVP control plane |
| **P5 — Multi-provider 1.0 / BYOK** | paralelo | não misturar no epic de protocol |

### 5.2 Extensões aos épicos App Server existentes

Manter a árvore `.llms/grok-build/app-server/` e **adicionar** (ou expandir):

1. **`v1-architecture-protocol`**  
   - Incluir ADRs: MCP control plane ownership; Tower vs subagent; agent capability matrix; flags de boot; product identity grok-oss.

2. **`v1-runtime-facade-projection`**  
   - Facade methods para: list/create/resume/fork threads; post message; interrupt; agent roster; capability check.

3. **`v1-core-in-process`**  
   - Vertical slice in-process: initialize → thread/start → turn/start → stream items.

4. **`v1-daemon-transports-security`** (hoje Phase 6)  
   - WebSocket + tokens + scopes.  
   - **Novo:** acceptors MCP (stdio MCP e/ou Streamable HTTP) compartilhando o **mesmo** method layer (ou thin adapter MCP→facade).

5. **Novo epic sugerido: `v1-mcp-control-plane`**  
   - Tools MCP espelhando subset de app-server methods.  
   - Flags: `--mcp off|stdio|http`, `--app-server …`, default both.  
   - Auth token shared/scoped com app-server.

6. **Novo epic sugerido: `v1-agent-tower`**  
   - Tool(s) `tower_*` ou `session_spawn` / `agents_*` no runtime Grok.  
   - Allowlist: default só `orchestrator` (config + agent definition field).  
   - Peer session create ≠ subagent; persistence first-class.  
   - Policy: budget, sandbox inheritance, worktree, model binding.  
   - Messaging: send/inbox/broadcast mínimo.

7. **`v1-ecosystem-ga`**  
   - SDK TS, exemplos, runbooks, security review including tower privilege escalation.

### 5.3 CLI / flags (alvo de produto)

Espelhar Codex + extensão Goblin (nomes a decidir; product binary `grok-oss`):

```text
grok-oss app-server \
  --listen stdio:// | unix:// | ws://127.0.0.1:PORT | off \
  --mcp off | stdio | http://127.0.0.1:PORT/mcp \
  --ws-auth ... --token-file ... \
  --remote-mode disabled|observeOnly|interactive|fullControl

grok-oss app-server tokens list|create|revoke
grok-oss app-server status|stop|pair
```

**Requisito humano:** default “daemon completo” = app-server transport local + MCP; flags para desligar cada um.

### 5.4 Agent capability matrix (proposta inicial)

| Agent type | spawn_subagent | tower create peer | tower send | tower interrupt | admin tokens |
|------------|----------------|-------------------|------------|-----------------|--------------|
| `orchestrator` | yes | **yes** | **yes** | **yes** | no |
| `build` | yes (se depth allow) | no | no | no | no |
| `review` / `repo-explore` / `explore` | limited | no | no | no | no |
| `architect` | limited | no | no | no | no |
| `general` | yes | no | no | no | no |
| custom | via config `tower_access = true` | opt-in | opt-in | opt-in | no |

Fonte de profiles: `~/.grok/agents/*.md` (instalação local do harness) — no produto, espelhar em skills/agents do shell.

### 5.5 Manutenibilidade (constraints de código)

- **Uma** crate de protocol: `xai-grok-app-server-protocol` (Rust = source of truth).
- **Uma** crate server: `xai-grok-app-server` com modules `transport/{stdio,ipc,ws}`, `mcp/`, `auth/`, `processor/`.
- **Facade** em `xai-grok-shell` (`app_server_runtime`) — thin; sem business logic de protocol.
- **MCP adapter** traduz tools MCP → facade calls; zero fork de Thread state machine.
- **Tower tools** em `xai-grok-tools` chamam facade/session registry — não falam wire JSON-RPC direto.
- Preferir novos arquivos a editar monólitos upstream (política GOBLIN).
- Testes: transport conformance suite **uma** vez, rodar em todos os transports + MCP tool path.

---

## 6. Questionário de intenção e escopo (RESPONDER ANTES DO PLANEJAMENTO CODEX)

> **Para o humano:** preencher os campos `Resposta:` (e `Notas:` se quiser).  
> Itens marcados **[BLOQUEANTE]** mudam arquitetura/épicos. Itens **[MVP]** definem o primeiro ship.  
> Itens **[DEPOIS]** podem ficar “adiar” sem travar o plano.  
> Defaults entre parênteses são **sugestão do handoff**, não decisão.

**Como usar com o Codex:** copiar este documento + suas respostas para a sessão de planejamento e pedir:  
*“Consolide o plano App Server + MCP + Agent Tower usando minhas respostas da §6; não invente onde eu não respondi.”*

---

### 6.0 Meta do projeto (produto e sucesso)

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q0.1 | **[BLOQUEANTE][MVP]** | Em **uma frase**, o que o grok-oss deve ser capaz de fazer quando o programa App Server+MCP+Tower “funciona”? | “Subir um daemon local, controlar threads via WS/TS SDK e MCP, e o orchestrator criar peers que conversam entre si.” |
| Q0.2 | **[BLOQUEANTE][MVP]** | Quem é o **usuário primário** do v1? (você / devs internos / open-source público / automação CI) | você + automação local |
| Q0.3 | **[MVP]** | Qual o **primeiro cliente real** que deve consumir a API? (script TS, outro agent, TUI, VS Code, n8n, codex-bus-like, etc.) | script TS + MCP de agents |
| Q0.4 | **[MVP]** | Qual é a **demo de aceite** do MVP? (lista 3–7 passos observáveis) | ver §6.0 template abaixo |
| Q0.5 | **[MVP]** | O que **explicitamente NÃO** entra no MVP? (lista de non-goals) | remote multi-user, Goal Runtime redesign, BYOK completo, TUI 100% migrada |
| Q0.6 | — | Prazo ou pressão de calendário? (semana / mês / sem deadline) | sem deadline rígido |
| Q0.7 | — | Preferência de staffing? (só você+agents / 1 wave por vez / paralelizar) | 1 wave por vez, maintainable |

**Respostas Q0:**

```text
Q0.1 (visão 1 frase):
Resposta:

Q0.2 (usuário primário):
Resposta:

Q0.3 (primeiro cliente):
Resposta:

Q0.4 (demo de aceite MVP — passos):
1.
2.
3.
4.
5.

Q0.5 (non-goals MVP):
-

Q0.6 (prazo):
Resposta:

Q0.7 (modo de execução):
Resposta:
```

---

### 6.1 App Server (protocolo, transportes, clientes)

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q1.1 | **[BLOQUEANTE][MVP]** | Protocolo deve ficar **perto do Codex** (Thread/Turn/Item) ou pode divergir cedo com `grok/*`? | perto do Codex; Grok-only em `grok/*` |
| Q1.2 | **[BLOQUEANTE][MVP]** | Transportes no **MVP**: quais? `stdio` / `unix socket` / `ws://127.0.0.1` / in-process / todos | stdio + ws loopback + in-process |
| Q1.3 | **[MVP]** | Wire JSON-RPC: exigir `"jsonrpc":"2.0"` nativo? Aceitar omissão estilo Codex em adapter? | nativo estrito; adapter opcional depois |
| Q1.4 | **[MVP]** | CLI canônica: `grok-oss app-server …` ou subcomando outro? | `grok-oss app-server` |
| Q1.5 | **[MVP]** | Default ao rodar sem flags: o que sobe? | listen local (stdio ou unix) + MCP on |
| Q1.6 | **[DEPOIS]** | TUI deve migrar para app-server no **mesmo** programa MVP? | **não** — TUI depois (Phase 7) |
| Q1.7 | **[DEPOIS]** | Compat adapter Codex (clientes Codex falando com Grok) no v1? | **não** no MVP |
| Q1.8 | **[MVP]** | ACP atual: manter paralelo, deprecar, ou bridge imediato? | manter paralelo; bridge depois |
| Q1.9 | **[MVP]** | Projection SQLite no MVP ou só session files + replay simples? | session files primeiro; SQLite no epic history |
| Q1.10 | — | Métodos **obrigatórios** no MVP? (initialize, thread/*, turn/*, list, interrupt, …) | core Codex + list/read/interrupt |
| Q1.11 | — | Métodos **fora** do MVP? (rewind, fork, remote pair, admin rebuild, …) | listar o que pode esperar |
| Q1.12 | **[DEPOIS]** | Electron / VS Code extension no escopo deste programa? | **não** no MVP |

**Respostas Q1:**

```text
Q1.1 (fidelidade Codex):
Resposta:

Q1.2 (transportes MVP):
Resposta:

Q1.3 (jsonrpc strictness):
Resposta:

Q1.4 (CLI):
Resposta:

Q1.5 (default boot):
Resposta:

Q1.6 (TUI no MVP?):
Resposta:

Q1.7 (Codex-compat client adapter?):
Resposta:

Q1.8 (ACP):
Resposta:

Q1.9 (projection SQLite no MVP?):
Resposta:

Q1.10 (methods MUST MVP):
Resposta:

Q1.11 (methods NOT MVP):
Resposta:

Q1.12 (desktop/IDE clients):
Resposta:
```

---

### 6.2 MCP control plane (servidor de orquestração)

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q2.1 | **[BLOQUEANTE][MVP]** | MCP control plane é **P0 no mesmo release** do app-server ou fase 2? | **mesmo release** (flags allow only-one) |
| Q2.2 | **[BLOQUEANTE][MVP]** | Transportes MCP no MVP: `stdio` / Streamable HTTP `/mcp` / ambos | stdio + HTTP loopback |
| Q2.3 | **[BLOQUEANTE][MVP]** | Tools MCP mínimas? (ex.: list/start/send/resume/interrupt/archive/status) | espelhar codex-bus `agents_*` subset |
| Q2.4 | **[MVP]** | MCP e app-server compartilham **mesmo token/scopes** ou auth separada? | mesmo plane de tokens; scopes distintos |
| Q2.5 | **[MVP]** | Flags desejadas (forma exata)? | `--listen …` + `--mcp off\|stdio\|http://…` |
| Q2.6 | **[MVP]** | Default “daemon completo” = app-server **e** MCP on? | **sim** |
| Q2.7 | — | Recursos MCP (resources/prompts) além de tools no v1? | tools only no MVP |
| Q2.8 | — | MCP deve expor **todas** as threads do daemon ou só “managed”? | todas as threads locais do daemon (scoped by token) |
| Q2.9 | **[DEPOIS]** | Multi-host `/connect` estilo codex-bus no v1? | **não** |
| Q2.10 | — | Nome do server MCP na config do client (`grok-agents`, `grok-oss`, …)? | `grok-oss` / `grok-agents` |

**Respostas Q2:**

```text
Q2.1 (MCP no mesmo release?):
Resposta:

Q2.2 (MCP transports):
Resposta:

Q2.3 (tools MCP MVP — lista):
Resposta:

Q2.4 (auth compartilhada?):
Resposta:

Q2.5 (flags CLI exatas):
Resposta:

Q2.6 (default both on?):
Resposta:

Q2.7 (resources/prompts?):
Resposta:

Q2.8 (quais threads visíveis):
Resposta:

Q2.9 (multi-host bridge):
Resposta:

Q2.10 (nome MCP server):
Resposta:
```

---

### 6.3 Agent Tower (peer sessions + comunicação)

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q3.1 | **[BLOQUEANTE][MVP]** | Tower é **P0** junto com app-server ou vem depois do core API? | depois do vertical slice app-server, **antes** do GA |
| Q3.2 | **[BLOQUEANTE][MVP]** | Peer session = (A) thread no **mesmo** daemon, (B) processo/leader isolado, (C) híbrido? | **(A)** no v1 |
| Q3.3 | **[BLOQUEANTE][MVP]** | Tower **complementa** subagents ou **substitui**? | **complementa** |
| Q3.4 | **[BLOQUEANTE][MVP]** | Quais agent types têm tower por default? | só `orchestrator` |
| Q3.5 | **[MVP]** | Custom agents podem optar-in (`tower_access=true`)? | **sim**, config explícita |
| Q3.6 | **[BLOQUEANTE][MVP]** | Nome da família de tools: `agents_*` / `tower_*` / `session_*` / outro? | decidir brand vs parity |
| Q3.7 | **[MVP]** | Operações mínimas da tower? | create peer, send, wait/poll, interrupt, list, archive |
| Q3.8 | **[MVP]** | Comunicação: só send+result, ou inbox assíncrona + broadcast? | send+result no MVP; inbox v1.1 |
| Q3.9 | **[MVP]** | Peer herda cwd/sandbox/model/provider do parent ou escolhe os seus? | defaults do parent; overrides opcionais |
| Q3.10 | **[MVP]** | Peer pode ter **agent type** diferente (build/review/explore)? | **sim** (orchestrator escolhe) |
| Q3.11 | **[MVP]** | Depth/limits: quantos peers simultâneos? max por orchestrator? | ex.: 8 concurrent, config |
| Q3.12 | **[MVP]** | Cancel do parent: peers sobrevivem ou morrem? | **sobrevivem** (como bg subagent) por default |
| Q3.13 | — | Worktree isolation para peers no MVP? | opcional flag, default off |
| Q3.14 | — | Tower tools devem aparecer só em capability mode `all` / profile allowlist? | allowlist por agent definition |
| Q3.15 | **[DEPOIS]** | UI TUI para peers (além de Tasks pane)? | depois |
| Q3.16 | — | Relação tower ↔ MCP: peers criados via tool devem aparecer no MCP list? | **sim**, same registry |

**Respostas Q3:**

```text
Q3.1 (quando a tower entra):
Resposta:

Q3.2 (A mesmo process / B multi-process / C híbrido):
Resposta:

Q3.3 (complementa ou substitui subagents):
Resposta:

Q3.4 (agent types com tower default):
Resposta:

Q3.5 (opt-in custom):
Resposta:

Q3.6 (nome das tools):
Resposta:

Q3.7 (ops mínimas):
Resposta:

Q3.8 (messaging model):
Resposta:

Q3.9 (herança cwd/sandbox/model):
Resposta:

Q3.10 (peer agent type diferente):
Resposta:

Q3.11 (limites concurrency):
Resposta:

Q3.12 (lifecycle vs cancel parent):
Resposta:

Q3.13 (worktree no MVP):
Resposta:

Q3.14 (capability gating):
Resposta:

Q3.15 (UI TUI):
Resposta:

Q3.16 (peers visíveis no MCP):
Resposta:
```

---

### 6.4 Tokens, auth, segurança, remote

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q4.1 | **[BLOQUEANTE][MVP]** | Auth local no MVP: peer UID / socket 0600 / file token / todos? | socket ACL + optional file token |
| Q4.2 | **[BLOQUEANTE][MVP]** | Remote (não-loopback) no MVP? | **não** — deny by default |
| Q4.3 | **[MVP]** | Scopes de token necessários no MVP? | `threads:read/write`, `turns:start/steer/interrupt`, `approvals:respond` |
| Q4.4 | **[MVP]** | CLI de tokens: create/list/revoke no MVP? | list/revoke mínimo; create se remote/file token |
| Q4.5 | **[DEPOIS]** | Pairing QR / device approval? | depois |
| Q4.6 | **[MVP]** | Provider credentials (xAI/Codex) **nunca** vazam no protocol? | **MUST** (já no spec) |
| Q4.7 | **[MVP]** | Approvals: quem responde no headless/MCP-only? (auto-deny / auto-allow policy / controller client) | fail-closed ou policy config explícita |
| Q4.8 | — | Rate limits / max message size no MVP? | defaults conservadores do spec |
| Q4.9 | — | Audit log de control-plane actions? | structured logs no MVP; audit file depois |

**Respostas Q4:**

```text
Q4.1 (auth local):
Resposta:

Q4.2 (remote no MVP):
Resposta:

Q4.3 (scopes MVP):
Resposta:

Q4.4 (CLI tokens):
Resposta:

Q4.5 (pairing):
Resposta:

Q4.6 (redação de secrets — confirmar):
Resposta:

Q4.7 (approvals headless):
Resposta:

Q4.8 (limits):
Resposta:

Q4.9 (audit):
Resposta:
```

---

### 6.5 TypeScript SDK e DX

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q5.1 | **[BLOQUEANTE][MVP]** | SDK TS é **obrigatório no MVP** ou “generate-ts existe” basta? | generate-ts + client mínimo no monorepo |
| Q5.2 | **[MVP]** | Onde mora o SDK? (`packages/`, `crates/.../generated/`, npm package separado) | monorepo generated; publish depois |
| Q5.3 | **[MVP]** | Exemplos obrigatórios? (stdio, ws, MCP tool call) | 1 exemplo stdio + 1 ws |
| Q5.4 | **[DEPOIS]** | Publicar npm `@brasalabs/grok-oss-app-server` no v1? | **não** até protocol estabilizar |
| Q5.5 | — | Linguagens além de TS no MVP? (Python/Rust client) | **não** |

**Respostas Q5:**

```text
Q5.1 (SDK depth no MVP):
Resposta:

Q5.2 (local do SDK):
Resposta:

Q5.3 (exemplos):
Resposta:

Q5.4 (npm publish SDK):
Resposta:

Q5.5 (outras langs):
Resposta:
```

---

### 6.6 Escopo de programas paralelos (o que entra / não entra)

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q6.1 | **[BLOQUEANTE]** | Goal Runtime redesign neste programa? | **não** — plano separado |
| Q6.2 | **[BLOQUEANTE]** | Multi-provider 1.0 (keyring, D10 product, multi-account subagent) neste programa? | **não** — paralelo |
| Q6.3 | — | BYOK (OpenRouter/Groq/CF) neste programa? | **não** |
| Q6.4 | — | npm publish `@brasalabs/grok-oss` (binary) bloqueia App Server? | **não** |
| Q6.5 | — | Issues abertas docs/ops (docs-001/002, testing-001, ops-001): fechar antes de planejar? | **não** bloqueia plano; opcional hygiene |
| Q6.6 | **[MVP]** | Identity paths: specs devem usar `grok-oss` / `~/.grok-oss` desde o dia 1? | **sim** |
| Q6.7 | — | Upstream merge policy durante o programa? (pausar sync / sync contínuo) | sync com cuidado; feature em `goblin-*` |

**Respostas Q6:**

```text
Q6.1 (Goal Runtime no programa):
Resposta:

Q6.2 (multi-provider 1.0 no programa):
Resposta:

Q6.3 (BYOK no programa):
Resposta:

Q6.4 (npm binary publish):
Resposta:

Q6.5 (hygiene issues):
Resposta:

Q6.6 (identity grok-oss nos planos):
Resposta:

Q6.7 (upstream sync):
Resposta:
```

---

### 6.7 Qualidade, testes, organização de código

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q7.1 | **[BLOQUEANTE]** | Preferência de crates: novas `xai-grok-app-server*` vs tudo em shell? | **crates novas** + facade thin no shell |
| Q7.2 | **[MVP]** | Gate de qualidade mínimo por epic? | unit + conformance black-box transport |
| Q7.3 | **[MVP]** | Fuzz/load no MVP ou só no GA? | GA |
| Q7.4 | — | “Código fácil de manter” — constraints extras? (sem macro hell, sem second actor, file size, …) | one processor; adapters thin; novos arquivos |
| Q7.5 | — | Documentação de usuário no MVP? (README app-server) | runbook mínimo + README |
| Q7.6 | — | Idioma da documentação de plano: PT / EN / misto? | PT ok nos planos; comments/code EN se repo for EN |

**Respostas Q7:**

```text
Q7.1 (layout de crates):
Resposta:

Q7.2 (gates de teste):
Resposta:

Q7.3 (fuzz/load):
Resposta:

Q7.4 (constraints de maintainability):
Resposta:

Q7.5 (docs de usuário MVP):
Resposta:

Q7.6 (idioma docs):
Resposta:
```

---

### 6.8 Priorização e fatias de entrega (como o Codex deve fatiar)

| ID | Prioridade | Pergunta | Default sugerido |
|----|------------|----------|------------------|
| Q8.1 | **[BLOQUEANTE][MVP]** | Ordem preferida das fatias? | 1 protocol+facade → 2 in-process core → 3 stdio/ws+tokens → 4 MCP → 5 tower → 6 SDK polish |
| Q8.2 | **[MVP]** | Vertical slice “hello world” aceitável? | in-process thread/start + turn/start + message stream |
| Q8.3 | — | Quer PRs pequenos e frequentes ou monólito de plano primeiro? | plano completo → PRs por epic |
| Q8.4 | — | Branch base dos PRs? | `goblin` (policy AGENTS.md) |
| Q8.5 | — | Estimativa: aceita ~MVP 8–12 person-weeks do spec ou quer MVP mais fino? | MVP mais fino se possível (core+MCP+tower mínima) |

**Respostas Q8:**

```text
Q8.1 (ordem das fatias):
Resposta:

Q8.2 (vertical slice):
Resposta:

Q8.3 (PR strategy):
Resposta:

Q8.4 (base branch):
Resposta:

Q8.5 (tamanho do MVP):
Resposta:
```

---

### 6.9 Cenários de uso (stories) — confirme ou edite

Marque **SIM / NÃO / DEPOIS** em cada story:

| ID | Story | MVP? |
|----|-------|------|
| S1 | Script TypeScript sobe/conecta app-server, cria thread, manda prompt, recebe stream | |
| S2 | `grok-oss app-server --mcp http://127.0.0.1:…` e um client MCP lista/sessões e manda mensagem | |
| S3 | Orchestrator na TUI usa tool tower e cria peer `build` que implementa um fix em sessão própria | |
| S4 | Orchestrator manda follow-up ao peer e lê resultado sem ser subagent depth-1 | |
| S5 | Dois peers trocam mensagens (A→B) via tower/MCP | |
| S6 | Token sem scope `turns:start` não consegue iniciar turn | |
| S7 | Flags: só WS sem MCP; só MCP sem WS | |
| S8 | Interrupt de turn ativo via MCP e via WS | |
| S9 | Resume de thread persistida após restart do daemon | |
| S10 | TUI full parity via app-server client | |
| S11 | Remote phone/browser control | |
| S12 | Goal Runtime `/goal` transacional completo | |

**Respostas S\*:**

```text
S1:
S2:
S3:
S4:
S5:
S6:
S7:
S8:
S9:
S10:
S11:
S12:
Notas extras de stories:
```

---

### 6.10 Intenção livre (escreva à vontade)

Use este bloco para o que as tabelas não capturam: analogias, “quero parecer o codex-bus”, limites éticos, obsessões de UX, medo de complexidade, etc.

```text
INTENTION_FREEFORM:
(coloque aqui)
```

---

### 6.11 Checklist mínimo para destravar o Codex (preencher o que for **[BLOQUEANTE]**)

- [ ] Q0.1 visão + Q0.4 demo de aceite  
- [ ] Q0.5 non-goals  
- [ ] Q1.2 transportes MVP  
- [ ] Q2.1–Q2.3 MCP no release + tools  
- [ ] Q3.2–Q3.4 + Q3.6 tower model/ACL/nomes  
- [ ] Q4.1–Q4.2 auth/remote  
- [ ] Q5.1 SDK depth  
- [ ] Q6.1–Q6.2 o que **não** entra  
- [ ] Q8.1 ordem das fatias  
- [ ] Stories S1–S9 marcadas  

Quando o checklist mínimo estiver respondido, o Codex pode planejar sem inventar o núcleo do produto.

---

### 6.12 Mapa compacto H1–H10 (legado do handoff inicial)

| ID antigo | Equivale a | Default |
|-----------|------------|---------|
| H1 | Q3.2 | (A) mesmo process |
| H2 | Q2.2 | stdio + HTTP loopback |
| H3 | Q3.6 | decidir |
| H4 | Q3.4 | só orchestrator |
| H5 | Q3.3 | complementa |
| H6 | Q4.2 | remote não |
| H7 | Q4.4–Q4.5 | file token; pairing depois |
| H8 | Q6.1 | Goal fora |
| H9 | Q6.2 | multi-provider paralelo |
| H10 | Q5.2/Q5.4 | monorepo; publish depois |

---


## 7. Ordem de leitura obrigatória (próxima IA)

### 7.1 Governança e produto
1. `AGENTS.md` (repo) + `GOBLIN.md`
2. `docs/architecture/GROK_OSS_IDENTITY_AND_DISTRIBUTION_PLAN.md`
3. `task.md` (multi-provider D1–D10) — contexto, não núcleo Tower

### 7.2 App Server / Goal (planos existentes)
4. `.llms/grok-build/README.md`
5. `.llms/grok-build/_shared/*.md` (ownership, identity, leases, security)
6. `.llms/grok-build/app-server/**` (VISION, SPECS, epics, tasks)
7. `changes/grok_app_server_spec_bundle/grok_app_server_plan_and_spec.md` (**fonte principal**)
8. `changes/grok_app_server_spec_bundle/grok_app_server_protocol_v1.ts` + schema + examples
9. `changes/grok-build-goal-runtime-technical-spec (1).md` (só se planejar Goal)

### 7.3 Runtime real (código a reutilizar)
10. `crates/codegen/xai-grok-shell/src/leader/`
11. `crates/codegen/xai-grok-shell/src/agent/mvp_agent/` + `subagent/`
12. `crates/codegen/xai-acp-lib/`
13. `crates/codegen/xai-grok-mcp/` (client — contraste com server novo)
14. `docs/runtime/turn-queue-subagents-and-followups.md`

### 7.4 Referências externas no filesystem do humano
15. `~/codex-app-server.md`
16. `~/brainstorm/codex-connector/schemas/codex-app-server/`
17. `~/mcps/codex-bus-mcp/README.md` (+ tools `agents_*`)
18. `~/.grok/agents/orchestrator.md` + `~/.grok/docs/agent-specs/orchestrator.md`

### 7.5 Issues / release honesty
19. `.agents/issues/**`
20. `TO_RELEASE.md`, `TO_RELEASE_NPM.md`
21. `.llms/reviews/code-audit-*.md` (recentes)

---

## 8. Escopo desta auditoria / limites de evidência

**Auditado (read-only):**
- árvore de planos `.llms/grok-build` e `changes/*app*server*`
- inventário de issues `.agents/issues`
- reviews recentes multi-provider
- presença/ausência de crates app-server
- leader/subagent/MCP client surfaces
- docs de runtime de turns/subagents
- referência codex-bus-mcp e codex-app-server docs no home do usuário

**Não feito (deliberadamente):**
- re-execução completa de `cargo test` do monorepo
- threat model formal de WS/MCP novo
- implementação ou reescrita de épicos
- materialização de issues novas em disco
- validação live de Codex PC8

**Evidence level:**  
- **Alto** para “App Server não implementado + planos rascunho existem”.  
- **Alto** para “MCP atual é client; não control plane”.  
- **Alto** para “subagent depth=1 limita orquestração”.  
- **Médio** para status multi-provider (docs-001 ainda OPEN; reconciliar com código).  
- **Inferido** para “estibordo” e “AP-7” (ASR).

---

## 9. Deliverables esperados da próxima IA (quando o humano autorizar planejamento)

1. **Documento consolidado** (ex.: `docs/architecture/APP_SERVER_MCP_TOWER_PLAN.md` ou árvore `.llms/grok-build/` atualizada) unificando:
   - App Server
   - MCP control plane
   - Agent Tower
   - flags/tokens
   - ACL de agents
2. **ADRs** para H1–H10 (ou subset).
3. **Épicos reescritos** com tasks checkáveis e gates; status `planejado` só após decisões humanas.
4. **Matriz de rastreabilidade**: requisito do usuário → seção do plano → epic → task → teste.
5. **Lista de issues** a materializar (`@issue-lifecycle`) sem implementá-las.
6. **NÃO** começar código sem “go” explícito.

---

## 10. Resumo executivo (1 página)

O **grok-oss** já tem:
- fork operacional com multi-provider/Codex **substancialmente implementado** (offline path),
- identidade de produto (`grok-oss`, `~/.grok-oss`, npm layout),
- runtime maduro (SessionActor, leader multi-client, ACP, subagents, MCP **client**),
- **planos detalhados mas não implementados** de App Server e Goal Runtime.

O humano pede agora a **camada de plataforma**:
1. App Server WebSocket + API TypeScript (estilo Codex),
2. MCP de controle de sessões/agentes co-hospedado com flags,
3. Agent Tower: sessões peer + comunicação + ACL por tipo de agent.

O plano App Server em `changes/` + `.llms/grok-build/app-server/` é a **melhor base**, mas precisa ser **estendido** (MCP server + Tower + flags + identity grok-oss). Implementação de código ainda **não começou**. O maior risco de engenharia é **duplicar runtime/control planes**; o maior risco de produto é **privilege escalation** se a tower for liberada a todos os agents.

Issues abertas atuais são sobretudo **docs/ops/test honesty** (docs-001/002, testing-001, operations-001), não blockers de design da tower. Multi-provider 1.0 e BYOK são programas paralelos.

**Próximo passo humano:** autorizar a IA de planejamento a reescrever/consolidar os planos (sem código).  
**Próximo passo IA de planejamento:** ler §7, responder H1–H10 com recomendações, produzir plano unificado e matriz de rastreabilidade.

---

## 11. Índice de paths canônicos (cópia rápida)

```text
# Produto / fork
GOBLIN.md
AGENTS.md
task.md
TO_RELEASE.md
TO_RELEASE_NPM.md
docs/architecture/GROK_OSS_IDENTITY_AND_DISTRIBUTION_PLAN.md
docs/architecture/multi-provider-auth/PROGRESS.md
docs/runtime/turn-queue-subagents-and-followups.md

# Planos App Server + Goal
.llms/grok-build/README.md
.llms/grok-build/_shared/
.llms/grok-build/app-server/
.llms/grok-build/goal-runtime/
changes/grok_app_server_spec_bundle/
changes/grok-build-goal-runtime-technical-spec (1).md

# Issues
.agents/issues/

# Código-chave
crates/codegen/xai-grok-shell/src/leader/
crates/codegen/xai-grok-shell/src/agent/subagent/
crates/codegen/xai-grok-mcp/
crates/codegen/xai-acp-lib/
crates/codegen/xai-grok-pager-bin/src/main.rs

# Referências externas (máquina do humano)
~/codex-app-server.md
~/brainstorm/codex-connector/schemas/codex-app-server/
~/mcps/codex-bus-mcp/
~/.grok/agents/
```

---

## 12. Instrução final desta sessão

**Não executar implementação a partir deste handoff.**  
Quando o humano autorizar planejamento: consolidar épicos/contratos a partir deste documento e revalidar paths se a branch tiver avançado.

---

## 13. Respostas humanas (2026-07-18) — decisões travadas

**Fonte primária:**  
`docs/architecture/transcripts/2026-07-18-user-intent-app-server-mcp-tower.md`  
(mensagem/áudio do maintainer; ASR normalizado).

**Como ler:** onde §13 conflita com defaults da §5–§6, **§13 vence**.

### 13.1 Brand e conceito Tower

| Decisão | Valor |
|---------|--------|
| Nome do conceito | **Tower** (mantido; gosto do humano confirmado) |
| Tower é | **Novo produto-nome** para control plane multi-sessão + swarm; não existia no vocabulario do repo |
| Ancestral no código | **Leader** multi-client (`connect_or_spawn` / `run_leader`): primeira UI sobe o processo; próximas UIs `goblin`/`grok-oss` **conectam no leader já em execução** + multi-sessão via **dashboard** |
| Ação para o Codex | **Analisar** leader + dashboard + session registry como **proto-Tower** a promover/generalizar — **não** reinventar do zero; mapear o que já é multi-client/multi-session vs o que falta (MCP SSE remoto, WS app-server, tools `tower_agent_*`, ACL agent type) |

### 13.2 Escopo MVP vs v2 (programa App Server + MCP + Tower)

| Camada | No MVP? | Notas |
|--------|---------|--------|
| App Server (protocolo Codex-like) | **SIM** | Construir de forma **ordenada, em sequência** (não tudo em um PR monólito) |
| WebSocket app-server | **SIM — early** | Junto da “primeira fatia funcional” de rede, não só no fim |
| MCP control plane | **SIM — mesmo release** do App Server | |
| MCP **remoto** (SSE / Streamable HTTP), não só local | **SIM — early** | Humano: *“não quero só MCP local”* |
| MCP tools de orquestração de swarm/sessões | **SIM** | Ver §13.3 |
| Multi-sessão **no mesmo processo** | **SIM** | Já existe UX (dashboard); formalizar no protocol/MCP |
| ACL tower por agent type (customizável; default orchestrator) | **SIM** | |
| Nome de tools | **`tower_agent_*`** | Ver §13.3 |
| TypeScript SDK + scripts | **SIM** | WS + SDK (inspiração Codex) |
| Tools **internas** agent↔agent (peer messaging via runtime tools) | **NÃO no MVP → v2** | Codex deve **analisar** desenho; não implementar no v1 se puder adiar |
| Goal Runtime redesign v1/v2 + flags | **FORA deste programa** (futuro) | Ver §13.5 |
| Retrocompat ampla em tudo | **NÃO** | Só planejada para **goal** no futuro |

**Tensão resolvida pelo humano:**  
“MVP com todas as funcionalidades **deste programa**” = App Server + MCP remoto + WS + swarm tools + multi-session + ACL + SDK.  
**Não** inclui tools internas de peer messaging (v2) nem Goal v2.

### 13.3 MCP tools MVP (swarm / sessões)

Família: **`tower_agent_*`** (nome confirmado).

Capacidades **MUST** no MVP (lista mínima a expandir no plano):

| Capability | Intent |
|------------|--------|
| **list** | Listar agentes/sessões/threads gerenciáveis |
| **start** | Criar/iniciar sessão/agent/thread |
| **send** | Enviar mensagem / iniciar turn de input |
| **hub** | **= a própria Tower** (M1 travado round 3). Não é tool/conceito separado de “inbox”; “hub” no áudio = o control plane Tower |
| **history / messages** | **full** **ou** **last**; **paginação + limites de bytes + redaction de secrets = SIM (M4)** |
| **interrupt / resume / archive / status / wait** | **MUST no MVP (M2)** |
| (swarm) | Gerenciar e orquestrar **várias** sessões/agentes |

**Superfície dupla (M3):** as mesmas operações `tower_agent_*` existem:
1. como **tools MCP** (clients externos / automação), e  
2. como **tools do modelo** no runtime para o **orchestrator** (mesmo contrato/semântica).  

Peer messaging *ad hoc* entre agents sem passar pelo control plane permanece **v2**.

### 13.4 Auth / rede (decisões 2026-07-18 round 2)

| ID | Decisão do humano | Valor canônico |
|----|-------------------|----------------|
| **R1** | LAN / **internet** | Bind remoto permitido no MVP (não só loopback) |
| **R2** | **Bearer** | `Authorization: Bearer <token>` (WS handshake + HTTP/MCP) |
| **R3** | Suporte **`ws://` e `http://`** | Cleartext **permitido** no MVP; **não** exigir TLS-only (HTTPS/WSS podem existir depois/opt-in) |
| **R4** | **Libera tudo** (Origin) | **Sem** Origin allowlist no MVP (browser clients aceitos sem check de Origin) |
| **R5** | **Sem escopo fino** | Token **full control** (sem matrix `threads:read` / `turns:start` no MVP) |

| Outros | Valor |
|--------|--------|
| MCP local only | **Rejeitado** |
| MCP remoto SSE / Streamable HTTP | **Required early** |
| App Server WebSocket | **Required early** (`ws://` ok) |
| Multi-sessão | Várias sessões **por Tower process** |
| Multi-Tower na mesma máquina | **SIM (M6)** — ver §13.11 |

**Aviso de segurança (não negociável no doc, mesmo com MVP permissivo):**  
Com R1+R3+R4+R5, a superfície é **high risk**. O plano **MUST**:
- documentar threat model honestamente (“bearer over cleartext on internet = stolen token = full control”);
- default de **bind** ainda pode ser configurável (ex. flag explícita `--listen 0.0.0.0:…`);
- **nunca** logar/retornar o bearer em claro em events;
- redaction de secrets em history (M4).  
Codex **não** deve “suavizar” isso sem decisão humana; o humano **aceitou** o tradeoff de MVP.

### 13.5 Goal e dual-versioning

| Decisão | Valor |
|---------|--------|
| Goal redesign no programa atual | **NÃO** — versão futura |
| Quando goal for feito | Refatorar goal atual → **v1**; implementar **v2**; **flag** ativa/desativa v1 vs v2 |
| Durante App Server **agora** | **Identificar** componentes/mechanisms muito tocados; opcionalmente “reforçar” fronteiras para facilitar dual-version depois |
| Retrocompat / dual flags | **Obrigatório só no goal (futuro)**; **não** exigir em app-server/MCP/tower |

### 13.6 ACL Tower (agent types)

| Decisão | Valor |
|---------|--------|
| Customizável | **SIM** |
| Default allow | **`orchestrator`** |
| Default deny | todos os outros agent types |
| Tools internas peer (runtime) | **v2** — ACL da v2 reutilizar mesma matriz se possível |

### 13.7 Entrega e DX

| Decisão | Valor |
|---------|--------|
| Ordem de build | **Sequencial / ordenada** (fatiar app-server → rede WS+MCP → tools swarm → SDK) |
| Acesso app-server | **WebSocket** + **TypeScript SDK/scripts** (MUST) |
| Inspiração protocolo | Codex app-server |
| SDK | Interface TypeScript real (não só schema solto) |

### 13.8 Mapa rápido §6 → resposta

| ID | Status | Resposta (resumo) |
|----|--------|-------------------|
| Q0.1 | **OK** | Daemon/Tower multi-sessão; WS + MCP remoto; orquestrar swarm; SDK TS |
| Q0.3 | **OK** | MCP clients + scripts TS; UIs conectando no Tower/leader |
| Q0.5 | **OK** | Tools internas peer messaging; Goal redesign; retrocompat geral |
| Q1.1 | **OK** | Perto do Codex |
| Q1.2 | **OK** | WS early + (stdio/IPC conforme ordem); multi-client local herdado do leader |
| Q1.6 | **parcial** | Multi-UI já via leader; “TUI nativo app-server” não detalhado |
| Q2.1 | **OK** | MCP **mesmo release** App Server |
| Q2.2 | **OK** | **Remoto SSE** + (local também ok); **não** só local |
| Q2.3 | **OK** | list, start, send, history full\|last, interrupt/resume/archive/status/wait; “hub”=Tower |
| Q2.6 | **OK** | Ambos no release; boot “Tower up” como leader |
| Q3.1 | **OK** | Conceito Tower no MVP control plane; tools internas agent v2 |
| Q3.2 | **OK** | **(A) mesmo processo**, multi-sessão |
| Q3.3 | **OK** | Tower control plane **complementa** (subagents não removidos); peer tools internas depois |
| Q3.4 | **OK** | Default só orchestrator; customizável |
| Q3.6 | **OK** | `tower_agent_*` |
| Q4.2 | **OK** | Remoto LAN/internet; bearer; ws/http cleartext; sem Origin; sem scopes |
| Q5.1 | **OK** | SDK TS **obrigatório** |
| Q6.1 | **OK** | Goal **fora** (futuro v1/v2+flag) |
| Q6.2 | **implícito** | Não mencionado → tratar multi-provider 1.0 como **paralelo**, não núcleo |
| Q8.1 | **OK direção** | Sequencial; early WS+MCP remoto |
| S1–S2,S6–S9 | **OK** | SIM (implícito) |
| S3–S5 | **parcial** | Swarm/sessões via MCP SIM; peer messaging interno agent **v2** |
| S10 | **aberto** | |
| S11 | **SIM** (remoto MCP/WS) | |
| S12 | **NÃO** (goal futuro) | |

### 13.9 Implicações de arquitetura (para o Codex planejar)

1. **Tower daemon** multi-sessão; **várias Towers** por máquina com identidade distinta (porta/socket/token).  
2. Uma Tower: **qualquer workspace** em `start` (cwd por sessão).  
3. App Server **WS** (`ws://`) + MCP **HTTP/SSE** (`http://`) early.  
4. Auth: **bearer full-control**; sem scopes finos; sem Origin gate.  
5. **`tower_agent_*`** no MCP **e** no tool surface do orchestrator (mesmo modelo).  
6. History com full|last + pagination + size limits + redaction.  
7. interrupt/resume/archive/status/wait = MUST.  
8. Goal fora; inventário de hot-paths opcional.  
9. Threat model remoto documentado (MVP permissivo).

### 13.10 “Hub” = Tower (M1 — travado round 3)

| Decisão | Valor |
|---------|--------|
| **M1** | Com “hub” o humano **quis dizer o Tower** (o control plane), **não** uma tool/entidade separada |

**Consequência no plano:**
- **Não** criar tool `tower_agent_hub` só por causa do áudio.
- O “hub” é o **daemon Tower** (app-server + MCP + registry multi-sessão).
- Superfície de tools: `tower_agent_list|start|send|…` sobre **esse** hub/Tower.
- Se no futuro precisar de overview explícito, preferir `tower_agent_status` / `list` rico, não o nome “hub”.

### 13.10b MCP config vs tool interna (M5 — intenção + recomendação)

**Pergunta do humano (round 3):**  
Se já existe **tool interna** de acesso ao Tower (orchestrator → `tower_agent_*` no mesmo processo), o agent **não** precisa de config MCP para a Tower **local**. Config MCP (`mcp_servers.*`) serviria **só** para conectar em **Towers externas** (outra máquina / outra instância). O que achamos?

| Superfície | Quem usa | Como conecta |
|------------|----------|--------------|
| **In-process tools** `tower_agent_*` | Orchestrator (e tipos com ACL) na **mesma** Tower | Chamada de tool local → facade da Tower atual (**sem** `config.toml` MCP) |
| **MCP server** desta Tower | Clientes externos (Cursor, scripts, outras apps, **outras** Towers) | `http://…/mcp` + bearer |
| **MCP client** no Grok | Sessão que quer falar com **outra** Tower | Sim: entrada `mcp_servers.<name>` apontando para Tower remota |

**Recomendação de arquitetura (aceitar como default de plano salvo veto):**

1. **SIM à intenção:** tool interna ≠ MCP client config. Orchestrator na Tower A usa tools in-process para A.  
2. **MCP server** da Tower A continua **MUST** (você quer SSE remoto + automação + SDK/clients).  
3. **MCP client config** para “a própria Tower” é **redundante e confusa** — evitar auto-injetar a Tower local como MCP server da própria sessão (loop/tool-dup).  
4. **MCP client config** para **Towers externas** = path suportado (multi-tower federation light).  
5. **Nome M5** só importa para (4) e para docs de clientes externos; proposed id: `grok-oss-tower` (remoto) ou por-instance `tower-<id>`.

**Status M5:** intenção humana **alinhada**; detalhe de naming de server key para externos ainda `[PROPOSED]` `grok-oss-tower`.

### 13.11 Multi-Tower + multi-workspace (M6 — travado)

| Requisito | Valor |
|-----------|--------|
| Um processo Tower | Pode **criar/gerir sessões em qualquer workspace** (cwd/root por sessão, não preso a um único project dir do daemon) |
| Várias Towers na **mesma máquina** | **SIM**, se o usuário quiser (instâncias isoladas) |
| Isolamento entre Towers | Por **listen endpoint** (porta/`--listen`) + **bearer token** + estado/home instance id — Codex deve desenhar discovery opcional (ex. listar towers locais) sem forçar singleton global |
| Default UX | Pode continuar “connect or spawn **default** tower”; flags para **nova** tower / tower explícita |

Isto **substitui** a pergunta “1 tower por machine vs por cwd”:  
→ **N towers por machine**; **1 session → 1 workspace** (qualquer path); **1 tower → N sessions → N workspaces**.

### 13.12 Nome do MCP server (M5)

- **In-process:** sem key MCP (tools internas).  
- **Clientes externos / outras Towers:** key de config; **`[PROPOSED]`** `grok-oss-tower` ou `tower-<instance-id>`.  
- Ver §13.10b.

### 13.13 Map R*/M* (atualizado round 3)

| ID | Status | Resposta |
|----|--------|----------|
| R1 | **OK** | LAN/internet |
| R2 | **OK** | Bearer |
| R3 | **OK** | `ws://` + `http://` (cleartext OK) |
| R4 | **OK** | Origin liberado (sem allowlist) |
| R5 | **OK** | Sem escopos finos (full control) |
| M1 | **OK** | “hub” = **Tower** (não tool separada) |
| M2 | **OK** | interrupt/resume/archive/status/wait **MUST** |
| M3 | **OK** | Orchestrator **também** usa o mesmo modelo de tools |
| M4 | **OK** | pagination + limits + redaction **SIM** |
| M5 | **OK intenção** | Tool interna local; MCP config **só** towers **externas**; nome key `[PROPOSED]` |
| M6 | **OK** | N Towers/machine; 1 Tower → any workspace sessions |
| T1 | **OK** | Sem cap enforced no MVP; livre; telemetria de uso/picos desejável; caps configuráveis depois |
| T2 | **OK direção** | Glossário Thread↔Session MUST; fork + dormant; unificar lifecycle; estudo Codex |
| T3 | **OK** | Dashboard intocado no MVP; ver §13.14 |
| T4 | **OK** | **A** — connect default tower / spawn / nova só com flag |

---

## 14. Perguntas que ainda faltam (atualizado)

**Resolvido:** R1–R5, M1–M6, **T1–T4**.  
**Em aberto (podem ser `[PROPOSED]` no plano):** K1–K3, O1–O2, Q0.2/Q0.4/Q0.6, Q7.6.

### 14.3 Sessões / swarm / dashboard — **T1–T4 em detalhe**

Responda em áudio/chat com o **ID** (ex. “T1: 32” / “T3: paralelo no MVP”).

---

#### **T1 — Cap de sessões** — **RESPONDIDO**

| Decisão | Valor |
|---------|--------|
| Cap rígido no MVP | **Não implementar** por enquanto — deixar **livre** |
| Configurável por máquina | **SIM** (quando houver estudo de custo); default “unlimited” / sem enforcement |
| Observabilidade | **Desejável:** logs de uso de recurso **por sessão** (atual + **picos**) — CPU/mem/FDs ou proxy metrics — para **depois** calibrar caps |
| Ação Codex | Não gastar épico em quota system; opcional spike “session resource telemetry” como task não-bloqueante |

```text
T1: free no MVP; caps configuráveis depois com base em medição; log de uso/picos desejável
```

---

#### **T2 — Thread vs Session / start** — **RESPONDIDO (direção + estudo)**

| Decisão | Valor |
|---------|--------|
| Glossário | **MUST no plano:** definir **Thread** (Codex app-server) vs **Session** (Grok) vs Turn/Item — e o mapping 1:1 ou não |
| Inspiração | Codex app-server, mas **não copy cego** |
| Capacidades desejadas | **Fork** de conversa; **iniciar/reativar sessão inativa (dormant)**; start relacionado a trade/sessão de trabalho |
| Unificação | Preferência de direção: **relacionar** thread↔sessão (não dois lifecycles órfãos); **Codex decide** mapping exato após estudo do código |
| cwd | Implícito M6: start com workspace path |

**Estudo obrigatório (Codex / planejador) — glossário:**

| Termo | Onde vive hoje | Notas |
|-------|----------------|-------|
| **Session (Grok)** | `~/.grok-oss/sessions/<cwd>/<id>/`, `SessionActor`, dashboard roster | Persistência real, history, tools |
| **Thread (Codex app-server)** | Spec em `changes/grok_app_server_spec_bundle/` | Unidade de protocolo multi-client |
| **Turn / Item** | Spec App Server | Unidade de trabalho / fragmentos stream |
| **Subagent session** | depth≤1 sob parent | Não é peer Tower |
| **Dormant** | Roster: on-disk, not resident | Candidato a “start inactive” / resume |

```text
T2: estudar Thread vs Session; fork + resume inactive; unificar conceitualmente; glossário no plano
```

---

#### **T3 — Dashboard no MVP** — **RESPONDIDO**

| Decisão | Valor |
|---------|--------|
| MVP | **Não mexer** no dashboard — deixar como está |
| Depois | Reavaliar migração para App Server client |
| Explicação | Ver **§13.14** (dashboard ↔ ACP) |

```text
T3: leave dashboard as-is in MVP; explain relation to ACP in §13.14
```

---

#### **T4 — Connect default** — **RESPONDIDO = A**

| Decisão | Valor |
|---------|--------|
| Default | **A:** connect na Tower default se existir; senão spawn; **nova** Tower só com flag explícita |
| Multi-tower | Continua M6 (flags/`--listen`/URL para instância não-default) |

```text
T4: A
```

---

### 13.14 Dashboard TUI ↔ ACP (explicação factual — T3)

> Para o humano e para o Codex: **o que é o dashboard hoje** e como se liga ao ACP.  
> Evidência: `xai-grok-pager` views/dashboard, `xai-grok-shell` roster + session handlers, user-guide sessions.

#### O que é o dashboard

- UI fullscreen do **pager** (`ActiveView::AgentDashboard`): lista **agents/sessões top-level** (e subagents), com peek, attach, dispatch de novo agent, filtros.
- Abre via `/dashboard` (alias `/sessions`) — ver `~/.grok/docs/user-guide/17-sessions.md`.
- Feature flag: `GROK_AGENT_DASHBOARD=0` ou `[dashboard].enabled`.
- **Não** é o App Server; **não** é MCP. É view da TUI.

#### Onde entram os dados

```text
┌─────────────┐     ACP / leader IPC      ┌──────────────────┐
│  TUI pager  │ ◄──────────────────────► │  Leader / MvpAgent│
│  dashboard  │  ext methods + notify    │  SessionActors    │
└─────────────┘                          └──────────────────┘
        │                                          │
        │ rows from app.agents + roster            │ disk sessions
        ▼                                          ▼
  render list                          ~/.grok-oss/sessions/...
```

1. **ACP (Agent Client Protocol)**  
   Canal entre **cliente** (TUI/pager, IDE, stdio) e **agent runtime** (shell/`MvpAgent`).  
   Métodos “normais” de sessão (prompt, updates de stream, permissões) + **ext methods** `x.ai/…`.

2. **Roster multi-sessão (FleetView)** — pensado para dashboard multi-client no **leader**:
   - request `x.ai/sessions/list` → lista `RosterEntry` (resident + dormant recentes)
   - notify `x.ai/sessions/changed` → upserts/removes para dashboards abertos ficarem em sync  
   Código: `shell/src/agent/roster.rs`, `handlers/session.rs`.

3. **Outros ext methods de sessão**  
   `x.ai/session/list`, `session/info`, `session/close`, `session_summaries/*` — metadados e load.

4. **Leader**  
   Um processo “dono” das sessões resident; vários clients TUI conectam (`connect_or_spawn`).  
   Dashboard em cada client enxerga o **fleet** via roster ACP, não via App Server JSON-RPC.

5. **Session files**  
   Verdade em disco: `updates.jsonl`, `chat_history.jsonl`, etc. (home produto `~/.grok-oss`).

#### Relação com App Server / Tower (MVP)

| Peça | MVP (decisão T3) |
|------|------------------|
| Dashboard | **Intocado** — continua ACP + leader + pager |
| App Server WS + MCP | **Novo** path para scripts/remoto/orchestrator tools |
| Futuro | Opcional: dashboard vira client App Server (TUI migration epic — **não** MVP) |

**Não confundir:**
- **Dashboard** = UI multi-sessão na TUI  
- **Tower** = daemon/control plane (produto novo nome; promove leader)  
- **ACP** = protocolo client↔agent **atual**  
- **App Server** = protocolo Thread/Turn/Item **novo** (Codex-like)

### 14.4 SDK TypeScript

| ID | Pergunta |
|----|----------|
| **K1** | SDK no monorepo path preferido? (ex. `packages/grok-oss-app-server`, `npm/…`) |
| **K2** | Publicar npm no MVP ou só path local / generate? |
| **K3** | SDK só Node, ou também browser? |

### 14.5 Sequência de entrega (confirmar fatias)

Proposta para você confirmar (SIM/ajustar):

1. Proto-Tower: formalizar leader lifecycle + multi-session registry (sem renomear tudo)  
2. Protocol crate + facade  
3. In-process / stdio app-server vertical slice  
4. **WebSocket + auth token**  
5. **MCP SSE remoto + `tower_agent_*`**  
6. TS SDK mínimo + exemplos  
7. ACL agent-type + docs  
8. (v2) tools internas peer messaging  

| ID | Pergunta |
|----|----------|
| **O1** | Essa ordem serve? O que puxar/empurrar? |
| **O2** | “Primeira funcionalidade” = fatia 4+5 juntas (WS+MCP remoto) assim que o core existir? |

### 14.6 Meta residual

| ID | Pergunta |
|----|----------|
| **Q0.2** | Usuário primário além de você? (open-source público no dia 1?) |
| **Q0.4** | Demo de aceite em 5 passos **literais** (comando + resultado esperado) — se puder gravar |
| **Q0.6** | Prazo ou “quando ficar pronto”? |
| **Q7.6** | Planos em PT ou EN para o Codex? |

### 14.7 Já decidido — não perguntar de novo

- Nome **Tower**  
- MCP **mesmo release** App Server  
- MCP **remoto SSE** + **WebSocket** early  
- Tools MCP swarm: list/start/send/hub/history full\|last  
- Multi-sessão **mesmo processo**  
- ACL default **orchestrator only**, customizável  
- Tools names **`tower_agent_*`**  
- Peer tools **internas** → **v2**  
- Goal redesign → **futuro** (v1/v2 + flag); retrocompat **só goal**  
- Durante App Server: **inventariar** hot-paths (não dual-flag obrigatório)  
- **TS SDK** MUST  
- Inspiração **Codex** app-server  
- Analisar **leader/dashboard** como proto-Tower  

---

**Fim do handoff.**

