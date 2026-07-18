# Prompt para Codex — Planejar árvore de épicos grok-oss (completo)

> **Uso:** copiar o bloco entre `BEGIN_PROMPT` e `END_PROMPT` e colar no Codex (modo com tools + write no repo).  
> **Repo:** `/home/guilherme/github/grok-goblin` (branch tipicamente `goblin-multi-provider-codex` ou a que estiver atual).  
> **Objetivo desta sessão Codex:** **só planejamento / doc tree** — **não** implementar código de produto.

---

```text
BEGIN_PROMPT

# Missão

Você é o planejador principal do produto **grok-oss** (fork Goblin de `xai-org/grok-build`).

Sua missão é **reescrever e alinhar toda a árvore de planejamento** do repositório, seguindo a skill **`@plan-epic-tree`** de ponta a ponta:

- Skill: `~/.agents/skills/plan-epic-tree/SKILL.md` (+ templates em `~/.agents/skills/plan-epic-tree/templates/`)
- Também respeitar: `AGENTS.md` (repo), `GOBLIN.md`, e baseline de qualidade em `~/.grok/AGENTS.md` / `~/.codex/AGENTS.md` se existir.

## O que você DEVE entregar

1. **Árvore de épicos completa e ordenada** sob `.llms/grok-build/` (ou estrutura que a skill exigir, **sem** criar árvore paralela confusa — **reescreva in-place** drafts fracos; use numeração de sequência nas pastas).
2. **Atualizar/reescrever** épicos existentes de:
   - **App Server** (hoje em `.llms/grok-build/app-server/` — desatualizados vs decisões do handoff; ainda falam Thread em vez de Session, etc.)
   - **Goal Runtime** (hoje em `.llms/grok-build/goal-runtime/` — **adiado** no release core, mas o plano Goal deve ser **reescrito** para: goal atual = v1 legado, Goal Runtime novo = v2 + flags; retrocompat só no goal)
3. **Novos grupos/projetos** no mínimo:
   - **Tower** (control plane: promove leader, multi-session, multi-tower)
   - **App Server** (protocolo Session/Turn/Item, WS, SDK TS) — alinhado Codex mas **termo canônico = session**
   - **MCP control plane** (`tower_agent_*`, SSE/HTTP remoto + local)
   - **Providers / multi-auth** (revisar Codex path + **BYOK** Groq/OpenRouter/Cloudflare — já há pacote em `docs/architecture/byok-providers-onboarding/`)
   - **TDD** (método de teste do monorepo)
4. **Arquivo `TDD.md`** canônico descrevendo o método TDD/behavior-test para **este** sistema Grok (onde vivem testes, crates, gates de epic, red-green-refactor, o que mockar, o que é e2e, conformance multi-transport, etc.) — colocar em local óbvio (ex. `.llms/grok-build/TDD.md` e/ou `docs/architecture/TDD.md` com link cruzado).
5. **Grafo de dependências e sequência de implementação** no root README: o que depende do quê; ordem de execução; o que pode rodar em paralelo.
6. **Numeração de pastas de épicos** com prefixo de sequência global (ou por programa + ordem), **kebab-case**, exemplo de forma (você pode ajustar se melhor, mas **deve** ser ordenável lexicograficamente e legível):

   ```text
   .llms/grok-build/
     README.md                 # roadmap + status + dependency graph + principles
     TDD.md                    # método TDD do sistema
     _shared/                  # contratos cross-project
     00-foundation/            # se precisar
     10-providers/             # multi-auth + BYOK
     20-tower-core/            # leader → Tower, multi-session
     30-app-server/            # protocol + facade + transports
     40-mcp-control-plane/
     50-tower-agent-tools/     # tower_agent_* in-process + MCP
     60-sdk-typescript/
     70-goal-runtime/          # futuro / v1 flag + v2
     80-channel-gateways/      # backlog stub só (Telegram) — NÃO implementar no core wave
     90-realtime-voice/        # backlog stub
   ```

   Dentro de cada programa, épicos: `v1-01-<name>/`, `v1-02-<name>/`, … com campo **`Depende de:`** real.

7. **NÃO implementar** Rust/product code nesta sessão. Só docs de plano (e atualizar contratos `_shared/` se necessário).
8. **NÃO** abrir PR; commits só se o humano pedir (e com paths explícitos).

## Skill @plan-epic-tree — regras que você DEVE seguir

Leia e execute a skill completa. Em particular:

- Estrutura: root README + `_shared/` + por-projeto README/SPECS/VISION + `vN-…/README.md` + `contracts/` se >100 linhas + `tasks.md` se >15 tasks.
- Pattern A (Escopo): **ADICIONAR / REFACTORIZAR / REMOVER / MANTÉM**.
- Status: `rascunho | planejado | em progresso | concluído`.
- Riscos: `[SEVERITY][Confidence]`.
- Tasks humanas: `(HUMAN)` + type + blocking.
- Provenance tags em decisões não óbvias.
- Overwrite drafts fracos **in-place** — não criar cópias paralelas “v2 do mesmo epic” confusas.
- Epics self-contained (um @build consegue implementar sem o chat).
- Phase 3 da skill: se precisar, **apresente o desenho da árvore** no final do trabalho no README (dependency graph) — o humano já autorizou a consolidação com base no handoff; **não** fique bloqueado pedindo decisões já travadas no handoff.

Se a skill e o handoff conflitarem em nomenclatura de pastas, **priorizar**: (1) decisões travadas do handoff, (2) skill structure, (3) clareza de sequência numerada.

---

# Fontes canônicas (LER ANTES DE ESCREVER)

Ordem obrigatória de leitura (não pular o handoff):

## A. Decisões de produto travadas (vence sobre specs antigos)

1. `docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md` — **fonte de decisões humanas** (§13, §14, glossário session)
2. `docs/architecture/transcripts/2026-07-18-user-intent-app-server-mcp-tower.md` — transcrições
3. `docs/architecture/CHANNEL_GATEWAYS_AND_REALTIME_VOICE.md` — **FORA do MVP core** (só stubs/backlog na árvore)
4. `GOBLIN.md`, `AGENTS.md` (repo)
5. `docs/architecture/GROK_OSS_IDENTITY_AND_DISTRIBUTION_PLAN.md` — binary `grok-oss`, home `~/.grok-oss`

## B. Specs seed (adaptar, não copiar cegamente)

6. `changes/grok_app_server_spec_bundle/` — plano App Server + schema/TS seed (**renomear Thread → Session** no plano Grok OSS)
7. `changes/grok-build-goal-runtime-technical-spec (1).md` — Goal Runtime
8. `.llms/grok-build/` — árvore **atual** (reescrever/alinhar)

## C. Providers / multi-auth / BYOK

9. `task.md` — multi-provider architecture (Codex-first)
10. `docs/architecture/multi-provider-auth/PROGRESS.md` + reports
11. `docs/architecture/byok-providers-onboarding/` — **inteiro** (PROBLEM, CURRENT_STATE, PROVIDER_MATRIX, GAPS…)
12. `.agents/skills/add-provider/SKILL.md`
13. `TO_RELEASE.md` — honesty Codex offline path

## D. Runtime real a reutilizar (não reinventar)

14. `crates/codegen/xai-grok-shell/src/leader/` — proto-Tower (`connect_or_spawn`)
15. `crates/codegen/xai-grok-shell/src/agent/roster.rs` + handlers session — dashboard fleet
16. `docs/runtime/turn-queue-subagents-and-followups.md` — depth≤1, fila, waits
17. `crates/codegen/xai-grok-mcp/` — MCP **client** (contraste com MCP **server** de control plane)
18. `crates/codegen/xai-grok-voice/` — baseline voz (ditado; full duplex é backlog)

## E. Referências externas na máquina do maintainer (se acessíveis)

19. `~/codex-app-server.md`
20. `~/brainstorm/codex-connector/schemas/codex-app-server/`
21. `~/mcps/codex-bus-mcp/README.md` — padrão agents_*; **inspiração**, não copy de nomes Codex

## F. Issues abertas

22. `.agents/issues/` — docs-001/002, testing-001, operations-001, etc. — planejar hygiene onde couber

---

# Decisões de produto JÁ TRAVADAS (não reabrir; codificar nos épicos)

## Glossário

| Termo Grok OSS | Notas |
|----------------|--------|
| **Session** | Canônico. **Não** usar “thread” na API/docs/SDK/MCP do produto. |
| **Turn / Item** | Mantém (inspirado Codex). |
| **Tower** | Daemon/control plane multi-session (promove leader). |
| **Thread** | Só ao citar Codex ou adapter de compat → mapear para Session. |

## App Server + MCP + Tower (MVP core — IN)

- App Server JSON-RPC inspirado no Codex; **Session**/Turn/Item.
- **WebSocket** early (`ws://` permitido).
- **MCP control plane no mesmo release**; **remoto SSE/HTTP** early — **não** só local.
- Flags: poder subir só app-server, só MCP, ou ambos (default “daemon completo” alinhado ao handoff).
- Auth: **Bearer**; **LAN/internet** permitido; **sem Origin allowlist** no MVP; **sem scopes finos** (full control token); cleartext `http://` + `ws://` OK no MVP — **documentar threat model honestamente**.
- Tools **`tower_agent_*`**: list, start, send, history full|last, interrupt, resume, archive, status, wait (MUST).
- “Hub” no áudio = **a Tower**, não tool separada.
- **In-process tools** para orchestrator (mesmo modelo); **MCP client config** só para Towers **externas** (não auto-MCP loop da Tower local).
- Multi-session **no mesmo processo**; **N Towers** na mesma máquina; 1 Tower pode criar sessions em **qualquer workspace**.
- ACL tower: **customizável**; default só **`orchestrator`**.
- Tools **internas peer-to-peer** agent↔agent sem control plane → **v2** (analisar, não bloquear MVP).
- **TS SDK** MUST + scripts.
- Cap de sessions: **sem enforcement no MVP** (livre); telemetria de uso/picos **desejável**; caps configuráveis depois.
- Dashboard TUI: **não mexer no MVP** (continua ACP/leader/roster) — ver handoff §13.14.
- T4: connect-or-spawn default Tower; nova Tower só com flag.
- Identity: `grok-oss`, `~/.grok-oss`.

## Goal (rewritten plan, NOT in core ship wave)

- Goal redesign **não** no mesmo release do Tower MVP.
- Plano deve: tratar goal atual como **v1**; Goal Runtime especificado como **v2** + **flags** enable/disable; inventariar hot-paths tocados pelo App Server para dual-version **futuro**.
- Retrocompat **somente** no goal (quando chegar a hora).

## FORA do core (só stubs/backlog na árvore)

- Channel gateways / **Telegram bridge** — `CHANNEL_GATEWAYS_AND_REALTIME_VOICE.md`
- Realtime voice full duplex — mesmo doc; baseline `xai-grok-voice` = dictation

## Providers

- Revisar e **alinhar** multi-provider Codex (já avançado no código) + plano **BYOK** (Groq, OpenRouter, Cloudflare Workers AI).
- Transformar `byok-providers-onboarding` de “problem pack” em **épicos versionados** com deps (após ou em paralelo controlado com Tower — **você define sequência** com justificativa).
- Usar skill mental de `add-provider`; API-key path ≠ OAuth Codex.

---

# Requisitos especiais do maintainer (esta sessão)

1. **Sequência é o mais importante:** pastas numeradas + root dependency graph + “o que depende do quê” + ordem de PRs/waves.
2. **Reescrever** épicos App Server e Goal legados para refletir handoff (session, remoto, tower_agent_*, goal v1/v2 flags).
3. **`TDD.md`** completo e acionável para este monorepo Rust:
   - crates e `cargo test -p …` típicos
   - red → green → refactor
   - behavior tests vs unit
   - contract/conformance (transports: in-process, stdio, WS, MCP)
   - o que mockar (rede externa, OAuth) vs o que não mockar
   - gates por epic (quando marcar concluído)
   - relação com `xai-grok-pager` PTY tests se relevante
4. **Não** misturar implementação Telegram/voice no wave core.
5. Prosa dos planos: **PT-BR** ok; identifiers em inglês.
6. Registrar git SHA + data no root README no momento do snapshot do plano.

---

# Workflow sugerido (execute nesta ordem)

1. `git rev-parse HEAD` + `git status` (só leitura de contexto).
2. Ler handoff + skill plan-epic-tree completa.
3. Inventariar `.llms/grok-build/**` atual.
4. Desenhar a **nova taxonomia de pastas numeradas** + grafo de deps (escrever no root README).
5. Atualizar `_shared/` (ownership, identity/session glossary, security/bearer, tower ACL, TDD principles se cross-cutting).
6. Reescrever projetos App Server, Goal, + criar Tower/MCP/Providers/SDK conforme desenho.
7. Escrever `TDD.md`.
8. Preencher cada epic: Escopo, contratos, tasks com `Follow @…`, riscos, Depende de.
9. Quality gate da skill (minimum + full se possível).
10. Entregar resumo final: tabela de épicos, ordem de execução wave 0…N, blockers `(HUMAN)`, o que ficou `[PROPOSED]`.

---

# Definition of Done desta sessão de planejamento

- [ ] Root `.llms/grok-build/README.md` com roadmap numerado + status + **dependency graph** + principles alinhados ao handoff
- [ ] Pastas de épicos **ordenáveis** por nome (prefixo numérico)
- [ ] App Server épicos reescritos (Session, WS early, security permissiva documentada)
- [ ] MCP control plane + `tower_agent_*` como épicos explícitos
- [ ] Tower multi-instance + multi-workspace model nos contratos
- [ ] Goal reescrito como v1 legado / v2 futuro + flags; **não** na critical path do core MVP
- [ ] Providers: Codex + BYOK (Groq/OR/CF) como épicos alinhados a `byok-providers-onboarding` + `task.md`
- [ ] `TDD.md` escrito e referenciado pelos épicos (tasks de teste apontam para ele)
- [ ] Gateways/Telegram e realtime voice só como **backlog stubs** com deps no core
- [ ] Nenhum código de produto implementado
- [ ] Resumo executivo no final da resposta ao humano

## Saída esperada na mensagem final

1. Árvore de diretórios criada/atualizada (paths)
2. Ordem de implementação (wave 0, 1, 2…) com deps
3. Lista de épicos com status `planejado` ou `rascunho` e tempo estimado 1–4 semanas cada
4. Decisões `[PROPOSED]` que ainda precisam do humano (mínimas)
5. Primeiro epic a executar quando formos implementar

END_PROMPT
```

---

## Notas para você (Guilherme)

1. Cole o bloco `BEGIN_PROMPT`…`END_PROMPT` no Codex com o repo `grok-goblin` aberto e permissão de escrita em docs.
2. Se o Codex não achar `~/.agents/skills/plan-epic-tree/`, aponte o path absoluto na máquina.
3. Depois que o Codex terminar o plano, a gente revisa o root README e só então entra em `@execute-plan` / implementação.
4. Commits deste prompt no repo: opcional — o arquivo já está em  
   `docs/architecture/prompts/CODEX_PROMPT_PLAN_EPIC_TREE_GROK_OSS.md`.
