# Codex app-server vs grok-oss App Server — análise de qualidade e estratégia de copiar vs reimplementar

| Campo | Valor |
|-------|--------|
| **Data** | 2026-07-19 |
| **Tipo** | Análise comparativa (read-only) + recomendação de engenharia |
| **Pergunta** | Teria sido melhor copiar o app-server do Codex (fork Goblins) e só reescrever o necessário? |
| **Fontes Codex** | `~/tmp/codex-upstream/codex-rs/app-server*` (snapshot OpenAI Codex / Apache-2.0); worktrees `~/worktree/goblins-*` e contrato em `~/brainstorm/goblins` (fork Goblins de `openai/codex`) |
| **Fontes grok-oss** | `crates/codegen/xai-grok-app-server{,-protocol,-client}`, `xai-grok-tower`, `xai-grok-mcp-server`, `packages/grok-oss-app-server`; specs em `changes/grok_app_server_spec_bundle/`, handoff em `docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md` |
| **Status** | Documento de decisão; **não** autoriza vendor em massa sem revisão de license/NOTICE e ownership do runtime |

---

## 1. Resposta curta

**Não: copiar o app-server do Codex inteiro para o fork grok-oss e “só reescrever o necessário” não teria sido a melhor estratégia global.**

**Sim em camadas seletivas:** copiar / portar *padrões*, *protocol shapes*, *transports*, *codegen de schema/TS*, e *test harness ideas* — isso sim teria acelerado e elevado a qualidade da superfície de API. O que **não** se copia de graça é o *runtime core* (`codex_core::ThreadManager`, login ChatGPT, rollout/state DB, plugins, remote control, analytics, guardian…): são ~48 crates `codex-*` no grafo do `codex-app-server`.

O caminho que o plano Grok já descreve (“promover o leader + facade sobre `SessionActor`”, não um segundo daemon) continua correto. O risco real da implementação atual não é “falta de copiar Codex”, e sim **maturidade de produção ainda baixa** (slice experimental com `FakeRuntime`, protocolo `session/*` em vez de `thread/*`, surface de métodos e integração Shell incompletas) **enquanto** a arquitetura de fronteira (protocol pure, facade, no-import-Shell) é **melhor em pureza** do que o monólito acoplado do Codex.

---

## 2. O que existe em cada lado

### 2.1 Codex / Goblins (`codex-rs`)

Superfície madura e **monorepo-acoplada**:

| Crate | Papel | Ordem de magnitude (LOC `.rs`, snapshot) |
|-------|--------|------------------------------------------|
| `app-server` | Binário + processador JSON-RPC, processors por domínio, event mapping | ~95k linhas / ~152 arquivos |
| `app-server-protocol` | Tipos v1/v2, schemars, `ts_rs`, fixtures, export | ~24k / ~45 arquivos |
| `app-server-transport` | stdio, unix socket, websocket, remote-control, auth | ~10k / ~16 arquivos |
| `app-server-client` | Cliente remoto | ~3k |
| `app-server-daemon` | Daemon / install / update loop | ~3k |
| SDK Python + testes de integração | Harness real contra o servidor | dezenas de testes de app-server |

**Organização interna (boa):**

- `message_processor` despacha para `request_processors/*` (thread, turn, mcp, config, fs, git, marketplace, plugins, remote_control, goals, …).
- Protocolo v2 modular (`protocol/v2/{thread,turn,item,mcp,...}.rs`).
- Transport separado do processador.
- Experimental API via macros (`ExperimentalApi`), codegen TS, schema fixtures.
- Suite de testes pesada: **centenas** de marcadores `#[test]` / `#[tokio::test]` só em `app-server` (~713 no snapshot medido), incluindo suites multi-mil-linhas (`turn_start`, `thread_resume`, plugins, realtime).

**Organização interna (custos de maturidade):**

- Arquivos “god module”: `thread_processor.rs` ~4.2k, `bespoke_event_handling.rs` ~3.8k, testes de integração de 2–4k linhas.
- Processors com `#[allow(clippy::too_many_arguments)]` e dependência direta de `ThreadManager`, `AuthManager`, `ConfigManager`, analytics, skills watcher, etc.
- Protocolo e runtime Codex **co-evoluem**; o app-server não é um adapter genérico — é a face de *um* core.

### 2.2 grok-oss (implementação em andamento)

Superfície **fina**, experimental, orientada a contratos:

| Crate / package | Papel | Ordem de magnitude |
|-----------------|--------|--------------------|
| `xai-grok-app-server-protocol` | Wire types puros, errors, transitions, envelope | ~2k LOC / 7 arquivos |
| `xai-grok-app-server` | `FacadeProcessor`, stdio / in-process / WS (feature), security, replay | ~3.1k / 11 arquivos |
| `xai-grok-app-server-client` | stub mínimo | ~50 LOC |
| `xai-grok-tower` | `GrokRuntimeFacade`, registry, lease, lock multi-instance, budgets, projection, **FakeRuntime** | ~2.6k / 13 arquivos |
| `xai-grok-mcp-server` | control plane MCP (HTTP streamable + stdio) | ~2.9k |
| `xai-grok-tower-tools` | tools de orquestração peer | ~1.3k |
| `packages/grok-oss-app-server` | client TS (stdio/WS) | package pequeno |

**Invariantes explícitos no código (qualidade de design alta):**

```text
// xai-grok-app-server: "must never construct SessionActor or depend on Shell"
// xai-grok-tower: "never imports Shell"; composition root (pager-bin) injects adapter
```

- Protocol crate **não** puxa transport/runtime.
- App-server depende só de `protocol` + `tower` (2 path deps), não de shell/agent.
- `WireCounter` como string decimal canônica (interop JS).
- `ProviderBinding` documentado como “sem secrets”.
- Initialize gate (`classify_pre_init`), catalog de erros, leases de controller, multi-instance lock, redaction canaries.

**Maturidade de produto baixa:**

- Protocol version: `2026-07-18.experimental-v2`.
- Métodos wire usam **`session/*`** (não `thread/*` do Codex) — desvio consciente ou drift do spec “ficar próximo do Codex”.
- Runtime de produção: adapter Shell **planejado**; testes e slice vertical giram em **`FakeRuntime`**.
- ~170 marcadores de teste em app-server + tower + mcp-server (muito menos surface e menos “real server” e2e).
- Client Rust quase vazio; TS SDK próprio, não gerado no nível `ts_rs` do Codex.
- Ainda não há o equivalente a remote-control, daemon package, plugins marketplace, models refresh worker, etc. do Codex (e o handoff **não** pede 1:1 isso no v1).

---

## 3. Qualidade de código — comparativo

Escala relativa (1–5), **no que cada um está tentando ser**:

| Dimensão | Codex app-server | grok-oss App Server / Tower | Comentário |
|----------|------------------|-----------------------------|------------|
| Completude de produto | **5** | **2** | Codex é shipado com clients reais; Grok é scaffold + vertical slice. |
| Cobertura de testes de integração | **5** | **2–3** | Codex: suites enormes; Grok: contract/conformance + FakeRuntime + alguns canaries. |
| Separação de camadas | **3** | **5** | Grok ganha: protocol pure + facade + composition root. Codex: app-server *é* o orchestration hub do core. |
| Acoplamento | **2** | **5** | Codex: 48 deps `codex-*`. Grok app-server: 2 path deps. |
| Modularidade de processors | **4** | **3** | Codex tem muitos processors; Grok concentra dispatch em `processor.rs` (~500 LOC) — saudável *para o tamanho atual*, precisa fatiar ao crescer. |
| Tamanho de módulo / “god files” | **2** | **4** | Codex tem monólitos de 3–4k LOC; Grok ainda cabe na cabeça. |
| Geração de schema/TS | **5** | **3** | Codex: `ts_rs` + export + fixtures. Grok: schemars + `generate-schema` example + package TS manual. |
| Segurança control-plane | **4** | **3–4** | Ambos pensam auth WS / local; Grok investiu canaries cedo; Codex tem remote-control + enroll maduros. |
| Manutenibilidade *incremental* | **3** | **4** | Adicionar feature no Codex toca core+protocol+processor+suite. No Grok, o custo ainda é baixo *porque* a surface é pequena. |
| Alinhamento com runtime dono | **5** (Codex core) | **2** (Shell real) / **5** (facade design) | Copiar Codex não “liga” no `SessionActor` sem reescrever o meio. |
| Risco de segundo runtime | **Baixo** (é o runtime) | **Controlado por design** | FakeRuntime é *só* teste se o adapter Shell existir; risco é fake virar produção por omissão. |

### 3.1 O que o Codex faz *melhor* (qualidade real, não marketing)

1. **Protocolo operacional completo** — thread lifecycle, resume com redaction, turn steer/interrupt, items ricos, approvals, config layers, MCP methods no mesmo bus, goals, plugins, search, fs, git, command exec, environments.
2. **Event pipeline de produção** — `bespoke_event_handling` e mappers core→item são o trabalho sujo que clientes precisam; Grok ainda tem `projection.rs` pequeno e FakeRuntime.
3. **Transport e multi-connection** — control socket, WS acceptor, remote control, connection IDs, serialization queues, connection RPC gate.
4. **Tooling de contrato** — schema fixtures, export TS, experimental flags versionados por campo.
5. **Densidade de regressão** — testes que exercitam edge cases de resume, list, plugins, realtime.

### 3.2 O que o grok-oss faz *melhor* (ou mais limpo)

1. **Boundary architecture** — app-server não importa Shell; Tower define `GrokRuntimeFacade`; pager-bin injeta. Isso evita o destino do Codex (app-server como “god process”).
2. **Protocol crate minimalista** — fácil de raciocinar, versionar e gerar schema sem puxar half monorepo.
3. **Invariantes de segurança no desenho** — `ProviderBinding` sem credenciais, canaries de secret em projection, fail-closed em disconnect (sem auto-allow).
4. **Features de produto Grok** — multi-provider binding, tower multi-instance, MCP control plane *junto* do app-server (visão handoff), agent tower tools — o Codex não é o template para isso.
5. **Tamanho cognitivo** — um dev novo entende o app-server Grok em horas; o Codex leva semanas.

### 3.3 Onde a implementação Grok ainda é fraca (honestidade)

| Gap | Evidência | Impacto |
|-----|-----------|---------|
| Runtime real não é o centro dos testes | `FakeRuntime` dominante | Contrato pode passar e Shell adapter quebrar |
| Naming `session/*` vs Codex `thread/*` | protocol methods | Dificulta drop-in clients Codex e “mechanical adapters” do spec |
| Falta processors por domínio | um `dispatch` match | Vai virar god-file se crescer sem fatiar |
| Client/SDK | client Rust ~51 LOC; TS package fino | Integração externa e IDE/clients atrasam |
| Daemon / lifecycle de processo | parcialmente em tower lock + handoff | Sem paridade com app-server-daemon Codex |
| Event fidelity | projection pequena | TUI/clients ricos precisam de Item model mais denso |
| Testes “de mentira” em controller | flags tipo `auto_allow_on_disconnect = false` assertados localmente | Preferir testes de comportamento no processor/runtime |

---

## 4. “Só copiar o código do Codex” — por que não fecha

### 4.1 Acoplamento fatal ao monorepo Codex

`codex-app-server` **não é um servidor pluggable**. Ele instancia e opera:

- `codex_core::ThreadManager` / config / rollout / state DB  
- `codex_login::AuthManager` (incl. refresh ChatGPT via server→client request)  
- plugins, marketplace, guardian, analytics, model providers Codex, exec server, etc.

Portar o crate **sem** o core é reescrever 80%+ do que “parece copiado”. Portar **com** o core é abandonar `SessionActor` / multi-provider Grok e virar fork de Codex — o que **já é o projeto Goblins** (`~/brainstorm/goblins`), não o grok-oss.

### 4.2 Domínio de produto diferente

| Tema | Codex | grok-oss |
|------|-------|----------|
| Runtime de sessão | ThreadManager + rollout | SessionActor + leader + storage Grok |
| Auth modelo | ChatGPT / OpenAI-centric + providers | multi-provider / multi-auth / Codex *como provider* |
| Identidade CLI | `codex` | `grok-oss` / `@brasalabs/grok-oss` |
| Control plane extra | remoto OpenAI-ish | MCP control plane + Agent Tower peers |
| TUI | Codex TUI | Grok pager/TUI madura — *gate de paridade* no spec |

O spec Grok já diz: *adaptar o runtime Grok; não duplicar*. Copiar app-server Codex e “apontar” para Shell não é um patch — é um rewrite de adapters em cada method.

### 4.3 License / provenance (factível, mas não grátis)

- Codex e Goblins: **Apache-2.0** (compatível com grok-oss Apache-2.0).
- Copiar trechos grandes exige **NOTICE**, copyright headers, e política de *vendor* (e não misturar código sem atribuição).
- Não é bloqueio legal óbvio; é processo + dívida de sync com upstream Codex.

### 4.4 Custo de manutenção dual

Se você copia app-server e tenta:

- acompanhar `openai/codex` (ou Goblins), **e**
- manter adaptações Shell/Grok,

você herda merges dolorosos em monólitos de 4k linhas **sem** ter o core idêntico. Goblins existe precisamente para “fork Codex + identidade”; grok-oss existe para “fork Grok Build + multi-provider + plataforma”.

### 4.5 O que *teria* sido melhor copiar cedo (counterfactual)

| Artefato Codex | Valor se portado seletivamente | Esforço vs reimplementar do zero |
|----------------|--------------------------------|-----------------------------------|
| `app-server-protocol` shapes (thread/turn/item) + naming | Alto — alinhamento de clients e mental model | Médio (rename + strip ChatGPT-only) |
| Pipeline schemars → TS (`ts_rs` / export fixtures) | Alto | Médio |
| Padrão `request_processors/*` + message_processor | Alto (estrutura) | Baixo copiar *esqueleto* |
| `app-server-transport` (stdio framing, WS auth, queues) | Alto | Médio-alto (deps) |
| Connection ID / outgoing message sender / serialization queues | Alto para multi-client | Médio |
| Suites de integração *como checklist* (não o código) | Alto | Baixo |
| `bespoke_event_handling` + thread_processor | Baixo sem core | **Muito alto** — reimplementar via projector no facade |
| app-server-daemon / remote-control / plugins | Só se produto pedir | Alto; adiar |

**Conclusão counterfactual:** o melhor ROI seria “**fork mental do protocolo + transport**”, não “**git subtree do crate app-server**”.

---

## 5. O caminho que estamos seguindo — avaliação

### 5.1 Alinhado com o plano

O plano em `changes/grok_app_server_spec_bundle/grok_app_server_plan_and_spec.md` pede:

1. protocol estável Thread/Turn/Item  
2. runtime facade sobre assets Grok  
3. item projector  
4. subscription / approvals  
5. projection store rebuildable  

O código atual materializa **(1) parcial**, **(2) como trait + FakeRuntime**, **(3–5) em esqueleto**. A decisão de **não** importar Shell no app-server é exatamente o princípio “one semantic core / runtime-first”.

### 5.2 Drift a corrigir (qualidade + Codex-proximity)

1. **Naming:** spec fala `thread/*`; implementação experimental usa `session/*`. Decidir e documentar (alias de compat ou rename antes de clients externos).  
2. **Geração TS:** aproximar do pipeline Codex (uma source of truth Rust → schema + TS), em vez de package TS paralelo que drift.  
3. **Shell adapter como P0 de verdade:** sem ele, a qualidade “de produção” do Codex é inatingível por definição.  
4. **Fatiar processors** antes de `processor.rs` passar de ~800–1000 LOC.  
5. **Harness e2e** (processo real stdio/WS + SessionActor) no estilo `sdk/python/tests/test_app_server_*.py` do Codex — mesmo que em Rust.

### 5.3 Onde *ainda* faz sentido “copiar” agora (recomendação prática)

**Fazer (seletivo):**

- Extrair *checklists* de comportamento das suites Codex (resume redaction, list pagination, initialize gate, multi-connection fan-out, slow client).  
- Portar ideias de `OutgoingMessageSender` / connection-scoped request IDs / serialization queues.  
- Alinhar entity model (`Thread` vs `Session`) com o doc de handoff e com clients desejados.  
- Opcional: vendor **somente** `app-server-protocol` *como referência* em `third_party/` ou doc de mapping method-by-method (sem compilar o core).

**Não fazer:**

- Substituir `xai-grok-app-server` por `codex-app-server` no workspace.  
- Trazer `ThreadManager` / rollout / login ChatGPT stack.  
- “Compat 100% Codex wire” como gate de v1 sem cliente real que exija isso (flag `app_server_codex_compat` no plano já prevê opcionalidade).

---

## 6. Veredito por cenário

| Cenário | Melhor estratégia |
|---------|-------------------|
| Quer um **Codex com branding Goblin** | Já existe: fork Goblins — *não* reimplementar app-server no Grok |
| Quer **Grok runtime + API multi-client** (produto atual) | Facade + protocol + transports; **inspirar** Codex, **não** copiar o crate |
| Quer **clientes Codex drop-in** falando com Grok | Camada de **compat adapter** (rename methods, map items) — copiar *protocol shapes*, reimplementar handlers no facade |
| Quer **velocidade máxima no MVP** | Manter slice FakeRuntime **só** se Shell adapter entrar na mesma wave; senão, qualidade falsa |
| Quer **qualidade Codex em 6 meses** | Investir em projector + e2e + codegen, não em LOC copiada |

### Nota sobre `~/brainstorm/goblins`

O diretório de contrato Goblins (branches `main`/`goblins`, npm `@brasalabs/goblins`) é o **lugar certo** para evolução *do* app-server Codex. O checkout local em `brainstorm/goblins` pode estar **incompleto** (sem árvore `codex-rs` full); a análise de código usou snapshots completos em `~/tmp/codex-upstream` e worktrees `goblins-*`. Para comparar de novo no futuro, preferir um worktree com `codex-rs/app-server` presente.

---

## 7. Scorecard final (síntese)

```text
                    Completude   Pureza/Fronteiras   Acoplamento   Testes realistas   Fit no Grok runtime
Codex app-server       █████         ███                █               █████              █ (não é Grok)
Grok implementação     ██            █████              █████           ██                 ████ (design) / ██ (Shell)
```

- **Qualidade de *arquitetura* Grok (hoje):** superior em boundaries.  
- **Qualidade de *produto/engenharia de produção* Codex (hoje):** superior por ordem de magnitude.  
- **Copiar o monólito Codex:** troca o gap de produção por um gap de integração e por um monorepo errado.  
- **Copiar seletivamente protocolo + transport patterns + checklists de teste:** melhor dos dois mundos e alinhado ao plano existente.

---

## 8. Ações recomendadas (não implementadas neste doc)

1. Documentar **tabela de mapping** `Codex method/type → Grok method/type` (uma página viva).  
2. Decidir **rename `session` → `thread`** ou aliases de compat **antes** de SDK público.  
3. Priorizar **Shell-backed `GrokRuntimeFacade`** + 5–10 testes e2e estilo Codex lifecycle.  
4. Extrair processors por domínio quando o match de methods crescer.  
5. Unificar codegen TS no pipeline Rust (evitar drift do `packages/grok-oss-app-server`).  
6. Tratar Codex como **oráculo de comportamento**, Goblins como **produto irmão**, Grok como **runtime dono**.

---

## 9. Evidência de medição (snapshot 2026-07-19)

| Métrica | Codex (`tmp/codex-upstream`) | grok-oss |
|---------|------------------------------|----------|
| LOC app-server (+protocol/transport/client/daemon) | ~135k+ `.rs` | ~5k protocol+server+client; ~12k com tower+mcp+tools |
| Arquivos app-server | ~152 | ~11 |
| Path deps no app-server | ~48 `codex-*` | 2 (`protocol`, `tower`) |
| Marcadores de teste (aprox.) | ~713 (app-server) | ~170 (app-server+tower+mcp) |
| Maior arquivo produção | `thread_processor.rs` ~4.2k | `lib.rs` app-server ~1.4k (muitos testes embutidos) / mcp http_server ~1.1k |
| License | Apache-2.0 | Apache-2.0 |

---

*Documento gerado a partir de inspeção local de código e specs; números de LOC são aproximados e variam com o commit do snapshot Codex.*
