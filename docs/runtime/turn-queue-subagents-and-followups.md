# Runtime: turns, fila, subagents, waits e follow-ups

| Campo | Valor |
|---|---|
| Repo | Goblin fork (`nonexphere/grok-build` / `~/github/grok-goblin`) |
| Audience | Operadores da TUI + quem mexe no harness (pager/shell/tools) |
| Status | Análise baseada em user-guide + source code (2026-07-17) |
| Upstream docs | `~/.grok/docs/user-guide/` (instalação local) e equivalentes no monorepo |

Este documento explica **por que o Grok parece “travado”**, **quando follow-up funciona**, e **o que o código faz** com subagents, shell e fila de prompts. Consolida comportamento observável na TUI e implementação em crates.

---

## 1. Resumo executivo

1. **“Travado” quase nunca é deadlock de UI.** Em geral o **turn do agente principal ainda está aberto**, esperando:
   - shell em **foreground**, ou
   - subagent em **foreground** (`run_in_background: false`), ou
   - `get_task_output` / wait com `timeout_ms > 0`.
2. **Follow-up com `Enter` mid-turn só enfileira.** A fila **só drena** quando `session.state` está **idle** (e não há holds como troca de modelo).
3. **Send-now / interject** é o caminho para “falar agora”. Com o parent em **wait interruptível** (`blocking_wait_depth > 0`), o input tende a ir para o **send-now path** automaticamente.
4. **Subagent background** não bloqueia o parent no spawn; **foreground sim**, via `BlockingWaitGuard` no parent.
5. **Tasks pane (`Ctrl+B`)** é a fonte de verdade para subagents e comandos em background.

---

## 2. Mapa de crates (source of truth)

| Responsabilidade | Crate / path (relativo ao repo) |
|---|---|
| TUI, fila, teclas, drain da queue | `crates/codegen/xai-grok-pager/` |
| Drain da fila de prompts | `.../pager/src/app/dispatch/queue.rs` → `maybe_drain_queue` |
| Ações (InterjectPrompt / send-now) | `.../pager/src/actions/` |
| Estado do agent na TUI | `.../pager/src/app/agent.rs` |
| Turn / sessão / subagents | `crates/codegen/xai-grok-shell/` |
| Wait interruptível no parent | `.../shell/src/tools/tool_context.rs` → `BlockingWaitGuard` |
| Spawn FG/BG + guard | `.../shell/src/agent/subagent/handle_request.rs` |
| Coordenador de subagents | `.../shell/src/agent/mvp_agent/subagent_coordinator.rs` |
| Tool `task` (spawn) | `.../tools/src/implementations/grok_build/task/` |
| Tool `get_task_output` / waits | `.../tools/src/implementations/grok_build/task_output/` |
| Shell / background / timeout | `.../tools/src/implementations/grok_build/bash/mod.rs` |
| Eventos de redirect (interject / cancel) | `crates/codegen/xai-file-utils/src/events/types.rs` |
| Buffer de interjeições | `crates/common/xai-interjection-core/` |

User-guide instalado (espelho conceitual):

- `docs` locais em `~/.grok/docs/user-guide/03-keyboard-shortcuts.md`
- `16-subagents.md`
- `20-background-tasks.md`

---

## 3. Modelo mental: o que é um turn

Um **turn** é a unidade de trabalho do agente principal (uma ida ao modelo + loop de tools até o turno terminar).

| Estado | Significado | Fila (`Enter`) | Send-now |
|---|---|---|---|
| **Idle** | Sem turn ativo | Drena / inicia novo turn | N/A ou no-op se nada na fila |
| **Turn running** | Modelo + tools + awaits | **Enqueue only** | Interject / cancel-and-send (conforme chord) |
| **Blocked waiting** | Turn em wait interruptível (`blocking_wait_depth > 0`) | Hold / path especial | Preferido; input costuma ir para send-now |

“Parece travado” = na maioria dos casos **turn running ou blocked waiting**, sem o operador olhar o Tasks pane.

---

## 4. `BlockingWaitGuard` — o contador que muda o comportamento da fila

### 4.1 Definição

Em `xai-grok-shell/src/tools/tool_context.rs`:

- `BlockingWaitGuard` é um RAII: no `enter` incrementa `blocking_wait_depth`; no `Drop` decrementa.
- Comentário no source: `queue_input` lê esse contador; se **non-zero**, um prompt que chega durante o wait toma o **send-now path**.

```text
blocking_wait_depth == 0  →  mid-turn Enter = enqueue “normal”
blocking_wait_depth > 0   →  input tende a send-now (wait interruptível)
```

### 4.2 Quem sobe o contador

| Operação | Sobe `blocking_wait_depth`? |
|---|---|
| Subagent **foreground** (`run_in_background: false`) | **Sim** — no spawn, enquanto o parent awaita o filho |
| Subagent **background** (`run_in_background: true`) | **Não** no spawn |
| `get_task_output` / wait com `timeout_ms > 0` | **Sim** (wait bloqueante do turn) |
| Shell foreground | Turn bloqueado no await do processo (equivalente prático de “running/waiting”) |

Código do spawn (simplificado):

```rust
// handle_request.rs
let mut parent_wait_guard = (!request.run_in_background)
    .then(|| BlockingWaitGuard::enter(ctx.parent_blocking_wait_depth.clone()));
```

Comentário em `subagent/mod.rs`:

> A foreground spawn holds a `BlockingWaitGuard` on the parent for the blocking await so `queue_input` routes a prompt sent during the wait onto send-now; **never for background spawns**.

---

## 5. Subagents em detalhe

### 5.1 Parâmetros relevantes

API do spawn (tool `task` / `spawn_subagent` no harness Grok Build):

| Campo | Default conceitual | Efeito |
|---|---|---|
| `run_in_background` / `background` | **false** se omitido na API documentada | FG: parent espera; BG: retorna id |
| `prompt` | obrigatório | Task do filho |
| `subagent_type` | ex. `general-purpose` | Profile/role |
| `model` | opcional | Override explícito (ganha de pins) |

Além disso, a definição do agent pode forçar `background` default no profile (resolvido em `handle_request` com `definition.background.unwrap_or(false)`).

### 5.2 Fluxo foreground (default)

```text
Parent turn
  → spawn(run_in_background=false)
  → BlockingWaitGuard++ no parent
  → UI: "Subagent running: …"
  → Parent NÃO avança o tool-loop até o filho terminar
  → Filho completa → resultado no parent → Guard Drop → parent continua o MESMO turn
```

**UX:** “lançou subagent e ficou parado.” Correto: está **blocked waiting** no filho.

### 5.3 Fluxo background

```text
Parent turn
  → spawn(run_in_background=true)
  → retorna subagent_id imediatamente (sem Guard no spawn)
  → Parent pode chamar mais tools / mais spawns
  → get_task_output(ids, timeout_ms=N)  ← aqui Guard++ de novo
  → wait até complete ou timeout (cap global ~10 min)
```

**UX:** “lançou e continuou; depois travou no wait.”

### 5.4 Cancel do turn vs sobrevida do filho

Em `task/types.rs` (comentários de design):

- Subagents com **`run_in_background: true`** são **excluídos** de cancel por `parent_prompt_id` — sobrevivem a cancel do turn do parent; o usuário (ou o modelo no próximo turn) polla via `get_task_output`.
- Subagents **foreground** do turn cancelado tendem a ser cancelados junto com o turn.

### 5.5 Depth

Só a sessão top-level spawna. Filho que chama `spawn_subagent` falha (depth limit = 1). Orquestração multi-hop **tem** que ser no primary (ex.: agent `orchestrator`).

### 5.6 Onde ver subagents na TUI

| Superfície | Conteúdo |
|---|---|
| Scrollback do parent | Bloco lifecycle: `Subagent running` / `Subagent started` / completed |
| **`Ctrl+B`** Tasks pane | Lista subagents + bg commands + monitors, spinner, elapsed, kill |
| Enter no bloco do subagent | Transcript fullscreen do filho (observacional) |

---

## 6. Shell / comandos longos

### 6.1 Foreground vs background

| Chamada | Parent | Observação |
|---|---|---|
| `run_terminal_command` **sem** background | Turn preso no processo | Principal causa de “freeze sem subagent” |
| `background: true` | Devolve `task_id`; parent segue | Notificação / auto-wake na completion |
| Timeout + **auto_background_on_timeout** | Comando pode ir pro bg em vez de ser morto | Parent “solta”; task continua |
| Usuário **`Ctrl+G`** | Manda o **comando FG atual** pro background | Parent desbloqueia; comando segue |

Descrição do bash tool (`bash/mod.rs`) documenta timeout, auto-bg e `background: true` para dev servers / builds longos.

### 6.2 Cap de wait em task output

`task_output/mod.rs`:

- `get_task_output` **sem** `timeout_ms` → snapshot **não-bloqueante**.
- Com `timeout_ms > 0` → wait bloqueante, limitado por `MAX_WAIT_BLOCK` (**600s** default).
- Override: env `GROK_MAX_WAIT_BLOCK_MS`.

Isso evita um único wait eterno; completions ainda podem acordar o modelo via auto-wake.

---

## 7. Fila de prompts (`pending_prompts`) e `maybe_drain_queue`

### 7.1 Quando a fila **não** drena

`maybe_drain_queue` (`pager/.../dispatch/queue.rs`) retorna cedo (e loga `prompt.drain_blocked`) se:

| Razão (`reason`) | Significado |
|---|---|
| `turn_running` | Turn ainda ativo — **caso mais comum** |
| `model_switch_pending` | `/model` em voo; fila segura até complete/reconnect clear |
| `loading_replay` | Replay de sessão |
| `server_queue_owns_next_turn` | Fila server-side (leader) manda no próximo turn |
| `no_session_id` | Sessão ainda não pronta |
| `user_editing_front` | Usuário editando o item da frente da fila |

Só quando **`session.state.is_idle()`** (e sem os holds acima) a fila promove o próximo `QueueEntry` e dispara `Effect::SendPrompt` / bash / cron / etc.

### 7.2 Tipos de entrada na fila

Conforme comentários em `maybe_drain_queue`:

- **Prompt** — follow-up de usuário
- **Command** — slash/comando interno (ex. compact)
- **BashCommand** — shell mode `!`
- **Cron** — scheduled task formatada

### 7.3 UX de follow-up (user-guide + código)

| Ação do usuário mid-turn | Efeito |
|---|---|
| Texto + **`Enter`** | **Enqueue** na fila local |
| Fila + agent em **blocked waiting** | Fila em **hold** até idle (ou send-now) |
| **`Enter` com texto** em blocked waiting (doc TUI) | Pode **entregar na hora** (cancela wait / send-now path) |
| Double-Enter (composer vazio + fila) | Envia o **topo** da fila agora |
| **InterjectPrompt** (send-now chord) | Interrompe o fluxo atual de conversa (ver §8) |

Painel da fila: **`Ctrl+;`** / `Ctrl+'` (varia por host; VS Code family macOS pode usar `Ctrl+4`).

---

## 8. Send-now, interject e redirects

### 8.1 Eventos de telemetria / redirect

Em `xai-file-utils/src/events/types.rs`:

| Tipo | Significado |
|---|---|
| `InterjectionSource::Direct` | Interject direto mid-turn (`x.ai/interject`) |
| `InterjectionSource::Queue` | “Send now” em linha da fila |
| `RedirectKind::Interjection` | Steer mid-turn |
| `RedirectKind::CancelThenSend` | Turn abortado; prompt novo como próximo |
| `RedirectKind::QueuedAfterCancel` | Abort + promoção do que estava na fila |

### 8.2 Chords típicos (TUI)

| Host | Send-now / InterjectPrompt |
|---|---|
| Default | `Ctrl+Enter`, alt `Ctrl+I` |
| Apple Terminal | `Ctrl+O` (primary em alguns binds) |
| VS Code / Cursor / Windsurf / Zed | **`Ctrl+L`** (Enter/I frequentemente não chegam no PTY) |

**Intenção de produto (user-guide):**

- **Send-now** = “para o que estás a fazer e trata disto” (interruptivo).
- **Enqueue (`Enter`)** = bilhete para o **próximo** boundary de turn (não interrompe).

**Código:** com `blocking_wait_depth > 0`, o path de input favorece send-now mesmo sem o usuário memorizar o chord.

### 8.3 O que **não** morre no cancel do turn

- Subagents **background**
- Comandos já enviados ao **background** (`Ctrl+G` ou `background: true`)
- Resto da fila (exceto promoção/cancel paths)

---

## 9. Auto-wake e “acordou sozinho”

No `ToolContext` (shell):

- `auto_wake_enabled` — completions de bg task / subagent podem gerar **prompt sintético** e um novo turn sem input humano.
- `goal_loop_active_gate` — pode **suprimir** auto-wake de bash/subagent durante loops de goal para não desviar o parent.
- `monitor_event_buffer` — eventos de `monitor` drenados entre steps de sampling como mensagem sintética.

**UX:** filho BG termina → notificação / auto-wake → parent “volta a trabalhar” sem Enter. Parece mágico; é o harness.

---

## 10. Diagrama unificado (código + UX)

```text
                         ┌──────────────────────┐
                         │  session.state: idle  │
                         │  maybe_drain_queue ✓ │
                         └──────────┬───────────┘
                                    │ SendPrompt / drain
                         ┌──────────▼───────────┐
                         │   turn_running       │
                         │   Enter → enqueue    │
                         └──────────┬───────────┘
                ┌───────────────────┼────────────────────┐
                │                   │                    │
         bash FG await      spawn FG + Guard++    spawn BG (sem Guard)
                │                   │                    │
                │                   │              mais tools / spawns
                │                   │                    │
                │                   │         get_task_output(timeout>0)
                │                   │              Guard++
                └───────────────────┴────────────────────┘
                                    │
                     blocking_wait_depth > 0
                     queue_input → send-now path
                                    │
                          filho / cmd termina
                          Guard Drop (depth--)
                                    │
                         turn completa → idle
                         drain fila / auto-wake
```

---

## 11. Padrões que o **modelo** escolhe (por isso “às vezes X às vezes Y”)

O harness é determinístico; a **política** de tools é do modelo:

| Padrão do agent | O que o usuário vê |
|---|---|
| `spawn(bg=false)` | Trava **no spawn** |
| `spawn(bg=true)` × N, depois tools | Continua um pouco |
| depois `get_task_output(..., timeout)` | Trava no wait (até 10 min cap) |
| bash longo FG | Trava sem subagent |
| bash `background: true` | Não trava no shell |
| wait_all em vários ids | Parece idle, Tasks pane cheio |

Agents tipo **orchestrator** (só handoff + wait de evidência) maximizam tempo em **blocked waiting** — é o design, não bug.

---

## 12. Checklist operacional

| Sintoma | Causa provável (código) | Ação |
|---|---|---|
| Spinner, pouco texto, filhos no Tasks | Parent em spawn FG ou `get_task_output` wait | `Ctrl+B`; esperar ou send-now |
| Travado sem subagent | Bash FG | `Ctrl+G` ou send-now |
| Digitei Enter e “não ouviu” | `turn_running` → fila não drena | Esperar idle ou send-now |
| Fila presa após turn | `model_switch_pending` / edit front / server queue | Completar `/model`; sair do edit da fila |
| Cancelei e filho morreu | Spawn era FG | Preferir bg se precisar sobreviver |
| Filho sobreviveu ao cancel | Spawn BG | Esperado |
| Voltou sozinho | Auto-wake de completion | Esperado se auto-wake ON |

### Teclas essenciais

| Tecla | Função |
|---|---|
| `Ctrl+B` | Tasks pane (subagents + bg) |
| `Ctrl+T` | Todos pane |
| `Ctrl+;` / `Ctrl+'` | Prompt queue |
| `Ctrl+G` | FG shell → background |
| `Ctrl+Enter` / `Ctrl+I` / **`Ctrl+L`** (VS Code family) | Send-now / interject |
| Enter no bloco do subagent | Transcript do filho |

---

## 13. Implicações para o agent `orchestrator` (home config)

No home do usuário (`~/.grok/agents/orchestrator.md`):

- Primary **depth=0** spawna leaves depth=1.
- Loop típico: recover → spawn `build` / `review` / `repo-explore` → **wait de evidência**.
- UX esperada: **muito tempo em blocked waiting**, mesmo com vários filhos em paralelo no Tasks pane.
- Melhorar “vivacidade” do parent = mais spawns `background: true` **antes** de um único wait agregado (trade-off de controle vs latência de feedback no scrollback do parent).

---

## 14. Como depurar com evidência de sessão

Logs/sessões locais (instalação):

```text
~/.grok/sessions/<encoded-cwd>/<session-id>/
  updates.jsonl     # ACP + subagent_spawned / tool calls
  events.jsonl      # tool_started, turn_started, …
  signals.json      # toolsUsed, modelsUsed
```

Úteis:

- `subagent_spawned` → `model`, `subagent_type`, description
- `prompt.drain_blocked` (ulog) → reason + queue_depth
- `tool_started` / `tool_completed` no parent durante “freeze”
- `RedirectKind` / interjection events se o user usou send-now

No monorepo, testes e2e de fila/interject vivem sob:

- `crates/codegen/xai-grok-pager/tests/pty_e2e/`
  (ex.: `queued_message_renders_once_not_twice.rs`, `edit_interject_lone_queued_row_keeps_tui_alive.rs`)

---

## 15. Relação com a documentação de produto

| Doc instalado | O que cobre |
|---|---|
| `03-keyboard-shortcuts.md` | Queue, hold, send-now chords, double-Enter |
| `16-subagents.md` | `background` default false, tasks pane, depth 1, UI blocks |
| `20-background-tasks.md` | `background: true`, `get_task_output`, `Ctrl+G`, `Ctrl+B` |

Este arquivo **não substitui** esses guides; **liga** o comportamento da TUI às structs/funções do harness no fork Goblin.

---

## 16. Change log deste documento

| Data | Nota |
|---|---|
| 2026-07-17 | Versão inicial: consolidação user-guide + source `grok-goblin` (BlockingWaitGuard, maybe_drain_queue, task_output cap, redirect kinds) |

---

## 17. Referências rápidas de código

| Conceito | Arquivo |
|---|---|
| `BlockingWaitGuard` / `blocking_wait_depth` | `crates/codegen/xai-grok-shell/src/tools/tool_context.rs` |
| Guard no spawn FG | `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` |
| Comentário parent wait depth | `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` |
| `maybe_drain_queue` | `crates/codegen/xai-grok-pager/src/app/dispatch/queue.rs` |
| `model_switch_pending` hold | `crates/codegen/xai-grok-pager/src/app/agent.rs` |
| `InterjectPrompt` action | `crates/codegen/xai-grok-pager/src/actions/` |
| `run_in_background` sobrevive cancel | `crates/codegen/xai-grok-tools/.../task/types.rs` |
| Cap 10m wait | `crates/codegen/xai-grok-tools/.../task_output/mod.rs` |
| Bash bg / auto-bg description | `crates/codegen/xai-grok-tools/.../bash/mod.rs` |
| `RedirectKind` / `InterjectionSource` | `crates/codegen/xai-file-utils/src/events/types.rs` |
