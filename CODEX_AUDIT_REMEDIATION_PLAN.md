# Plano de remediação — auditoria Codex provider (2026-07-16)

**Fonte:** `.llms/reviews/code-audit-grok-goblin-codex-provider-2026-07-16.md`  
**Validação:** código atual na branch `goblin-multi-provider-codex` (2026-07-16, pós-implementação parcial)  
**Meta:** “Codex production-ready” (não “100%” informal) com gates mensuráveis.

---

## 1. Veredito da validação

O audit está **essencialmente correto**. O progresso real (phase side-channel, multi-assistant, `prompt_cache_key` no wire, binding na sessão, model cache, thoughts no plain) **não** fecha os invariantes centrais. Vários itens marcados como “PASS offline” na sessão anterior são **parcialmente verdadeiros** ou **estruturalmente falhos** sob concorrência.

### Tabela de validação (AUD-CODEX-*)

| ID | Reivindicação do audit | Validação no código | Status real | Severidade |
|----|------------------------|---------------------|-------------|------------|
| **001** | `patch_assistant_phases` reatribui phase por posição após filtrar `None` | **CONFIRMADO** — `filter_map` só pega phases presentes; loop zera assistants no wire por ordem | **BUG CRÍTICO aberto** | Critical |
| **002** | Materialize por tamanho; deltas de FC args não materializam | **CONFIRMADO** — `materialize_response_output` preferência por `len`; args delta só viram `ToolCallDelta` UI | **BUG CRÍTICO aberto** | Critical |
| **003** | FC anexado à assistant; MCP drop; `_ => {}` | **CONFIRMADO** — `FunctionCall` → `tool_calls` da última assistant; `McpCall` só contador | **Aberto** | High |
| **004** | Chave `anonymous` + IDs crus + log da chave | **CONFIRMADO** — `derive_prompt_cache_key` → `goblin-sess-anonymous`; `format!("goblin-sess-{s}")`; `tracing::info!(prompt_cache_key = %key)` | **Aberto** | High |
| **005** | `ensure_prompt_cache_key` em todo Responses | **CONFIRMADO** — chamado em `conversation_stream_responses` / `conversation_responses` sem gate Codex | **Aberto** | High |
| **006** | Stamp “request-scoped” é last-wins no resolver | **CONFIRMADO** — um `RequestScopedStamp` por `MultiProviderBearerResolver` de sessão; 2ª resolve sobrescreve; teste só isola 2 holders | **BUG CRÍTICO aberto** | Critical |
| **007** | Single-flight só in-process; DashMap race | **CONFIRMADO** — `get` + `insert` em `make_store_and_manager`; refresh sem `acquire_lock` cross-process | **Aberto (prod)** | Critical |
| **008** | Journal recovery no `new`, erro ignorado | **CONFIRMADO** — `let _ = recover_pending_txn(&paths)` no construtor | **Aberto** | High |
| **009** | Binding reconstruído de hints a cada config | **CONFIRMADO** — `reconstruct_full_config` chama `session_auth_for_sampling_hints` e **substitui** `multi_provider_auth` | **Aberto** | High |
| **010** | Model cache frágil | **CONFIRMADO** — `write` não atômico, erros engolidos, ETag sempre `None` no fetch, qualquer Err → stale/bundled | **Aberto** | Med/High |
| **011** | Hoist system lossy + URL gate | **CONFIRMADO** (padrão atual hoist textual + `is_codex_responses_backend` por URL) | **Aberto** | High |
| **012** | Sem prova live PC8 | **CONFIRMADO** — só `live-unavailable` / offline | **Aberto (gate release)** | High |

### O que o audit **não** invalida (progresso real)

| Área | Estado honesto |
|------|----------------|
| Captura SSE de `phase` → map `message_id` | Funciona no path stream |
| Uma `AssistantItem` por message wire (sem colapsar texto) | OK para messages puras |
| C2 canal Reasoning vs Text (fixture stream) | Teste unitário existe e passa |
| Field `prompt_cache_key` no CreateResponse | Existe |
| `cached_tokens` parse/log no stream | Existe |
| Headless plain thoughts (stderr) | Existe |
| Model cache path per-credential + TTL unit tests | Existe (incompleto p/ prod) |
| Shared TokenManager in-process (parcial A2) | Existe |

### Correções conceituais do goal (audit § “Mudanças no GOAL”)

| Correção | Aceitar? |
|----------|----------|
| Separar wire / auth / refresh / cache / discovery / UX | **Sim** — obrigatório |
| R5 cross-process = obrigatório para “production-ready” | **Sim** se o claim for produção; **não** se milestone = “single-process experimental” |
| A1 = stamp por **attempt** de request | **Sim** — audit 006 está certo |
| PC2 chave opaca, sem anonymous global | **Sim** |
| Forge ≠ prova de cache Codex | **Sim** |
| Full replay + `previous_response_id=None` é política válida | **Sim** — só documentar + testar |

---

## 2. Definição de “100%” (mensurável)

**Nome:** Codex production-ready (single-machine multi-process)  

**Não é 100%** se qualquer P0 abaixo falhar.

### Gates (todos REQUIRED)

| Gate | Critério de PASS |
|------|------------------|
| **W1 Wire fidelity** | Round-trip fixture: ordem, phase por item (incl. `None` intercalado), reasoning, FC com args, custom/web/code, refusal; unknown item preservado ou erro explícito |
| **W2 Stream materialize** | Merge por `output_index`/id; FC args de delta/done; completed parcial vs stream; nunca drop silencioso |
| **A1 Attempt stamp** | 2 resolves concorrentes **no mesmo resolver/sessão** → 2 stamps; 401 da attempt N usa stamp N |
| **A2+R5 Refresh** | 2 processos: 1 refresh upstream; journal recovery sob lock, fail-loud |
| **A4 Binding** | `ModelBinding` persistido na sessão; rebuild de config **não** re-deriva de header/URL se binding já pinado |
| **PC key** | Opaca (hash), capability-gated Codex-only, sem `anonymous` global; title/subagent com key distinta ou omitida |
| **PC live** | Turn2 `cached_tokens > 0` + negative control (SCRATCH redigido) **ou** milestone sem claim de cache |
| **M7 catalog** | Write atômico, policy 401/403 ≠ stale, ETag/If-None-Match, source no contrato |
| **Hygiene** | `git diff --check`, testes dos crates tocados, clippy nos packages |

**Milestone intermediário (Experimental):** W1+W2+A4+PC key opaca+capability + A1 attempt stamp; **sem** cross-process e **sem** live cache claim.

---

## 3. Plano de ataque (ordem obrigatória)

### Wave 0 — Contrato e hygiene (½ dia)

1. Atualizar `CODEX_100_PERCENT_GOAL.md` / `TO_RELEASE.md` com gates da §2 (retirar “100%” ambíguo).
2. Corrigir trailing whitespace do worktree (`git diff --check`).
3. Matriz de variants Responses (checklist de fixtures).

**Done:** docs alinhados; diff --check limpo nos paths Codex.

---

### Wave 1 — Identidade e auth attempt (P0) — fecha 006, 009, parte 004/005

| Task | O quê | Acceptance |
|------|--------|------------|
| **1.1** | `BearerResolveResult { token, stamp }` (ou lease id) no path multi-provider; amostrador/request guarda stamp **por attempt** | Tipo no multi-auth + shell; 401 recovery lê stamp do attempt, não `resolver.last()` |
| **1.2** | Teste: 2 `current_bearer` sequenciais no **mesmo** resolver; 401 simulado com stamp da 1ª resolve | Stamp da 2ª não é usado na recovery da 1ª |
| **1.3** | `ModelBinding` autoritativo na sessão: set na seleção de modelo; `reconstruct_full_config` **reusa** se presente e compatível | Não sobrescreve com hints se binding pinado |
| **1.4** | Headers `ChatGPT-Account-ID` / `x-goblin-credential-id` derivados do binding | Headers = efeito, não fonte |
| **1.5** | `prompt_cache_key`: HMAC/sha256 opaco `(ns, provider, credential_id, session_id)`; **omitir** se sem identidade; log só prefixo 8 hex | Sem `goblin-sess-anonymous`; sem log full key |
| **1.6** | `ensure_prompt_cache_key` **somente** se `is_codex_responses_backend` (ou capability no client) | Teste: backend não-Codex → key None |

**Done:** AUD-006, 009, 004, 005 fechados com testes unitários.

---

### Wave 2 — Refresh + journal (P0 produção) — fecha 007, 008

| Task | O quê | Acceptance |
|------|--------|------------|
| **2.1** | `SHARED_MANAGERS.entry(home).or_insert_with(...)` | Sem race check-then-insert |
| **2.2** | Refresh / recover_unauthorized: `store.acquire_lock(Refresh)` → reload → CAS → release | Integração com TokenManager |
| **2.3** | Teste multi-thread (e se possível multi-process) single refresh | Contador mock de refresh = 1 |
| **2.4** | Journal: recovery **lazy** sob lock no primeiro acesso; `recover_pending_txn` fail-loud/quarentena | Sem `let _ =` silencioso no `new` |
| **2.5** | Remoção de journal com fsync/erro propagado | Teste crash point documentado |

**Done:** AUD-007, 008; gate A2+R5.

*Se o milestone for Experimental single-process: 2.2–2.5 ficam P1 documentados; **não** claim production.*

---

### Wave 3 — Wire fidelity (P0) — fecha 001, 002, 003, 011

| Task | O quê | Acceptance |
|------|--------|------------|
| **3.1** | `patch_assistant_phases`: zip **1:1** com assistants de origem (preservar `None`); preferir `message_id` se presente no wire | Teste AUD-001 repro → fail → pass |
| **3.2** | Materializer: merge por index/id; acumular `function_call_arguments` delta/done; completed vs stream com diagnóstico | Fixture multi-item + FC streaming |
| **3.3** | Conversation model: FC como sibling (ou wire-history canônico separado da projeção UI) | Ordem commentary→FC→final preservada no resend |
| **3.4** | MCP / unknown: store opaco `WireItem` ou reject explícito | Sem drop silencioso |
| **3.5** | Hoist system: capability tipada; non-text → erro ou path documentado | Teste + sem gate só por substring se possível |

**Done:** AUD-001–003, 011; gate W1+W2.

---

### Wave 4 — Prompt cache policy + paths (P0/P1) — fecha 004 residual, 010 parcial, 012 path

| Task | O quê | Acceptance |
|------|--------|------------|
| **4.1** | Documentar política: full replay, `store=false`, `previous_response_id=None` | Doc + teste de serialização estável |
| **4.2** | Title gen / subagent / compact: key distinta ou omitida; nunca anonymous compartilhado | Code review + unit |
| **4.3** | Prefix diagnostic hash (instructions+tools+history ids) em log debug | Snapshot test |
| **4.4** | Compaction: bump generation na key ou omit | Teste |
| **4.5** | Usage: surfaces headless JSON + ausência de usage | Teste |
| **4.6** | **PC8 live** (gated): turn1/turn2, negative control, SCRATCH redigido | `cached_tokens > 0` ou fail gate |

**Done:** PC production claim possível só com 4.6.

---

### Wave 5 — Model catalog (P1) — fecha 010

| Task | O quê |
|------|--------|
| **5.1** | Atomic write (`tmp`+rename), mode 0600, erros logados |
| **5.2** | Fetch com `If-None-Match`; persist ETag real |
| **5.3** | Policy: 401/403/identity → **não** stale; 5xx/timeout → stale ok |
| **5.4** | `ModelCatalog` (ou wrapper) com `source: Network\|Stale\|Bundled` |
| **5.5** | Bundled versionado / marcado UI |

---

### Wave 6 — Release gates

1. `git diff --check`  
2. `cargo test -p xai-grok-sampling-types -p xai-grok-sampler -p xai-grok-multi-auth --lib`  
3. `cargo check -p xai-grok-shell`  
4. Dual run dos testes críticos  
5. SCRATCH pack: unit logs + live (se houver) + diff-stat  
6. Atualizar `TO_RELEASE.md` só com PASS comprovados  

---

## 4. Mapa finding → task

| Finding | Wave / Task |
|---------|-------------|
| AUD-001 | 3.1 |
| AUD-002 | 3.2 |
| AUD-003 | 3.3–3.4 |
| AUD-004 | 1.5, 4.2 |
| AUD-005 | 1.6 |
| AUD-006 | 1.1–1.2 |
| AUD-007 | 2.1–2.3 |
| AUD-008 | 2.4–2.5 |
| AUD-009 | 1.3–1.4 |
| AUD-010 | Wave 5 |
| AUD-011 | 3.5 |
| AUD-012 | 4.6 |

---

## 5. O que **não** fazer

- Declarar “Codex 100%” com só fixtures offline e stamp por sessão.
- Usar Forge / Groq `cached_tokens` como prova de cache Codex.
- “Corrigir” A1 com mais um Mutex no resolver sem attempt id.
- Mascarar 401 de `/models` com catálogo stale.
- Enviar `prompt_cache_key` a todos os backends Responses.

---

## 6. Estimativa grossa

| Wave | Esforço (ordem de grandeza) |
|------|----------------------------|
| 0 | 0.5 d |
| 1 | 2–3 d |
| 2 | 2–4 d (multi-process mais caro) |
| 3 | 3–5 d |
| 4 | 2–3 d + live env |
| 5 | 1–2 d |
| 6 | 0.5–1 d |

**Experimental shipável:** Waves 0+1+3 (+ 1.5/1.6) sem live e sem cross-process.  
**Production-ready:** todas as waves P0 + 4.6 + 2.x.

---

## 7. Kickoff executor (copiar)

```text
Execute CODEX_AUDIT_REMEDIATION_PLAN.md Wave 1 then Wave 3 (critical wire+auth).
Do not claim production-ready until gates W1/W2/A1-attempt/A4/PC-key/M7-policy pass
with real tests. Validate AUD-001 first with a failing test, then fix correlation.
```

---

## 8. Resumo em uma frase

O audit acertou: **replay pode mentir (phase/materialize), 401 pode usar stamp errado sob concorrência, e cache “completo” ainda não é prova nem chave segura** — o plano acima ataca nessa ordem (identidade → lock/journal → wire → cache policy → live → catalog).
