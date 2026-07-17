# GOAL — Codex multi-provider: Experimental → production-ready

> **Como usar:** cole este documento inteiro como goal da sessão (`/goal` ou mission prompt).
> **Repo:** `grok-goblin` · branch `goblin-multi-provider-codex`
> **Baseline honesta (2026-07-16):** Experimental offline **PASS**; production-ready **NÃO**.
> **Normas:** `TO_RELEASE.md` · `CODEX_AUDIT_REMEDIATION_PLAN.md` · `CODEX_100_PERCENT_GOAL.md`
> **Auditoria:** `.llms/reviews/codex-validation-audit-2026-07-16.md`
> **Evidence:** `SCRATCH/` (live probes redigidos) + testes offline nos crates tocados

---

## Mission

Fechar o caminho Codex multi-provider do Goblin até o gate **production-ready (single-machine multi-process)**, sem regredir os invariantes Experimental já provados e sem claims falsos de cache/prod.

**Definition of done (mensurável):**

| Gate | Critério de PASS |
|------|------------------|
| **A2+R5 Refresh** | 2 processos (ou simulação multi-process): no máximo 1 refresh upstream por rotação; recovery 401 usa stamp da attempt correta |
| **A3 Journal** | Recover sob lock, fail-loud/quarantine; erros de remoção/propagação não engolidos |
| **W1 residual (AUD-003)** | FC como sibling no wire (ou wire-history canônica separada da UI); MCP/unknown opacos ou erro explícito — **sem** `_ => {}` silencioso |
| **A4 residual** | Binding pinado é autoritativo; hints só se não houver pin; rebuild não sobrescreve pin vivo |
| **PC live (AUD-012 / PC8)** | Com `GROK_LIVE_CODEX=1` (ou credencial disponível): turn-2 `cached_tokens > 0` + negative control; artefato SCRATCH redigido **ou** milestone documentado sem claim de cache |
| **M7 (AUD-010)** | Write atômico + modo seguro; ETag/If-None-Match real; 401/403/identity ≠ stale; `source` no contrato público |
| **Hygiene** | Testes dos crates tocados verdes; `cargo check -p xai-grok-shell`; `TO_RELEASE.md` alinhado à evidência |
| **Docs honestas** | Nunca “100%” / production-ready / cache-hit sem gate correspondente PASS |

**“100% Codex path” neste goal** = todos os gates acima PASS (ou PC live explicitamente cortado do milestone com doc).
Não inclui keyring full, adapter xAI completo, nem D10 OAuth approval product — esses ficam 1.0 multi-provider.

---

## Estado de partida (NÃO regredir)

Já **CONFIRMED FIXED** com testes offline (revalidar se mexer nestes paths):

| ID | Invariante | Evidência mínima |
|----|------------|------------------|
| **AUD-001** | Phase 1:1 incl. slots `None` | `patch_assistant_phases_preserves_none_slots_no_sliding` |
| **AUD-002** | Materialize por `output_index` + FC args delta/done | `materialize_*` + `append_function_call_arguments_delta_*` |
| **AUD-004/005** | Key opaca `gpc_<sha256>`, omit sem identidade, **Codex-only** | `prompt_cache_key_opaque_*` + gate `is_codex_responses_backend` |
| **AUD-006 / A1** | Stamp por attempt FIFO; `peek_bearer` **não** grava stamp; `current_bearer` grava | `request_stamp::*` + `auth_info_and_prefix_use_peek_bearer_not_send` |
| **C2** | Commentary → Reasoning; final → Text | `commentary_phase_routes_to_reasoning_then_final_to_text` |
| **A4 partial** | Pin reuse no mesmo credential em `reconstruct_full_config` | static + path `sampler_turn` |

**Mental model de prompt cache (obrigatório):** cache é **provider-managed**. Cliente só: (a) prefixo estável no resend, (b) `prompt_cache_key` opaca quando há identidade, (c) observar `cached_tokens`. Offline **não** prova hit. Não afirmar “cache completo” sem PC8 live.

**Experimental offline:** MET. Não reabrir Wave 1/3 “do zero”; só regressões e residual AUD-003/A4.

---

## Non-goals (explícitos)

- Não enfraquecer login fail-closed (`GROK_CODEX_OAUTH_APPROVED` / client id explícito) sem decisão D10.
- Não force-push, rewrite history, ou push a `xai-org` sem permissão; PR via `fork` se pedido.
- Não inventar campos upstream; provar com SSE live / OpenAPI / docs forge.
- Não marcar complete com “EXIT=0 uma vez” ou “final answer ok” como se fosse cache/prod.
- Não copiar forge como prova de cache Codex.
- Não silenciar erros de journal/auth para “passar teste”.
- Não expandir escopo para keyring/xAI adapter/D10 product a menos que bloqueie um gate acima.
- Não claim multi-process safe até A2+R5 PASS.

---

## Ordem de execução (obrigatória)

### Wave A — Auth production (P0): AUD-007 + AUD-008

**Prioridade máxima.** Sem isto não há production-ready.

#### A.1 — AUD-007 single-flight cross-process + race fix

**Problema:** `make_store_and_manager` faz get-then-insert (race); refresh só com locks Tokio in-process; sem `acquire_lock` cross-process no refresh/401-recovery.

**Required:**

1. Corrigir criação de store/manager com API atômica (`DashMap::entry` ou equivalente) — sem check-then-act.
2. Envolver refresh token + 401 recovery em lock cross-process existente do credential store (`acquire_lock` / file lock), com CAS ou geração de token.
3. Dois caminhos concorrentes (threads no mínimo; processos se viável) → **um** refresh upstream bem-sucedido; o outro espera e reusa.
4. 401 recovery da attempt N continua usando stamp N (não regredir A1).
5. Testes: multi-thread single-refresh; se possível harness multi-process; regressão peek/current.

**Acceptance:**

- [ ] Race get+insert eliminada com evidência de código + teste.
- [ ] Lock cross-process no path de refresh/401 documentado no PR/TO_RELEASE.
- [ ] Teste prova “N concurrent resolvers → ≤1 refresh” (ou equivalente mensurável).
- [ ] Suites `xai-grok-multi-auth` + `xai-grok-sampler` verdes nos filtros request_stamp / resolve.

#### A.2 — AUD-008 journal fail-loud

**Problema:** `recover_pending_txn` no `FileCredentialStore::new` com erro ignorado; remoção de journal engole erros; recovery fora de lock.

**Required:**

1. Recovery lazy **sob lock**, não silent no construtor (ou construtor só marca dirty e recover no primeiro acesso locked).
2. Falha de recover → fail-loud ou quarantine explícita (não corromper silenciosamente metadata/secret).
3. Remoção de journal propaga erro ou tenta durable; log estruturado.
4. Testes de crash-mid-txn / pending journal (unit ou integration com temp dir).

**Acceptance:**

- [ ] Nenhum `let _ = recover_pending_txn` silencioso no path quente de produção.
- [ ] Teste de recovery sob falha/corrupção controlada.
- [ ] TO_RELEASE: A3 não mais “NOT claimed” se PASS; senão deixar aberto com evidência.

**Wave A done quando:** A2+R5 + A3 PASS offline (e multi-process se harness existir); sem claim de cache.

---

### Wave B — Wire residual (P0 wire): AUD-003 + A4 residual

#### B.1 — AUD-003 FC sibling + MCP/unknown

**Problema:** `FunctionCall` ainda anexa à última assistant (`tool_calls`); `McpCall` drop; `_ => {}` silencioso. Args de FC já materializam (AUD-002) — **não quebrar**.

**Required:**

1. Modelo canônico de wire history: FC (e itens de tool backend) como siblings ordenados no replay, **ou** store de wire separado da projeção UI — documentar a escolha.
2. MCP e unknown: preservar opaco para resend **ou** erro explícito (nunca drop silencioso que mude prefixo de cache).
3. Round-trip fixture: `commentary → FC → final` (e MCP se fixture existir) sobrevive convert→resend com ordem estável.
4. Testes unitários no `xai-grok-sampling-types` (+ stream se necessário).

**Acceptance:**

- [ ] Fixture multi-item com FC (e se possível MCP) round-trip sem colapso de ordem.
- [ ] Zero `_ => {}` que descarte variant Responses sem log/erro/opaco.
- [ ] Não regredir materialize/phase tests.

#### B.2 — A4 binding residual (AUD-009)

**Required:**

1. Se `MultiProviderSessionAuth` pinado e credential+provider compatíveis → **nunca** substituir por hint-derived binding.
2. Hints só quando não há pin (primeira seleção / migrate).
3. Teste: reconstruct com pin presente + hints conflitantes → pin vence.

**Acceptance:**

- [ ] Teste “pin wins over hints”.
- [ ] Headers continuam derivados do pin (efeito, não fonte).

**Wave B done quando:** W1 residual PASS; A4 residual PASS.

---

### Wave C — Prompt cache live proof (P0 release se claim cache): AUD-012 / PC8

**Só rodar com credencial real / `GROK_LIVE_CODEX=1`. Se indisponível: documentar bloqueio e NÃO claim cache.**

**Required:**

1. Probe automatizável (bin/test ignored by default):
   - Turn 1: prompt estável + mesma `prompt_cache_key` + mesma credential.
   - Turn 2: mesmo prefixo + continuação → assert `cached_tokens > 0` (ou campo equivalente no usage parse).
   - Negative control: mutar early history → cache miss ou `cached_tokens` não aumenta de forma espúria.
2. Artefato SCRATCH redigido (sem tokens, sem PII, key só label 8 hex).
3. Se live impossível: `TO_RELEASE` / README com “cache: provider-managed; live proof pending” — **milestone production sem claim de cache** ainda possível se Wave A+B+D (parcial) ok e doc honesta.

**Acceptance:**

- [ ] PASS live com evidência SCRATCH **ou** explicit “cache not claimed” no TO_RELEASE.
- [ ] Nunca logar bearer / refresh / full cache key.

---

### Wave D — Model catalog production (P1, mas no DoD se “100% path”): AUD-010

**Required:**

1. Write atômico (`tmp` + rename); preferir mode 0600 no secret-adjacent cache se path for user-private.
2. Erros de save logados (não `let _ =`).
3. Fetch com `If-None-Match` quando ETag conhecido; parse ETag real (não `None` permanente).
4. Policy: 401/403/identity mismatch → **não** servir stale como se fosse OK; 5xx/timeout → stale/bundled ok com `source` explícito.
5. `ModelCatalog` (ou tipo público) expõe `source` / stale / bundled flags — não dropar no `into_model_catalog`.
6. Testes unitários de write atômico, policy de erro, ETag round-trip se mockável.

**Acceptance:**

- [ ] Critérios M7 da matriz de remediação PASS.
- [ ] Testes `model_cache` / catalog verdes.

---

### Wave E — P1 polish (após A–D ou em paralelo se não bloquear)

| ID | Task |
|----|------|
| **AUD-011** | Gate Codex por capability/tipo de backend, não só substring URL; política explícita p/ system/developer non-text |
| **PC7 e2e** | `cached_tokens` chega a headless JSON usage + session usage (teste ou probe) |
| **P4** | Title gen em sessão Codex não bate proxy xAI indevido |
| **P6/P7** | Docs usuário + job CI opcional para live gate |
| **PC10** | Policy de key em compaction (omit vs re-derive) documentada + teste se aplicável |
| **Hygiene** | `git diff --check` nos paths tocados; clippy packages alterados se o repo exigir |

---

## Protocolo de execução (a cada wave)

1. **Inspect** — ler código atual + testes existentes nos paths; não reimplementar o que já PASS.
2. **Test-first quando bug** — para AUD-007/008/003, preferir teste que falha → fix → verde (AUD-001 style).
3. **Change** — menor diff correto; sem refactor cosmético.
4. **Validate** — rodar filtros + suite do crate + `cargo check -p xai-grok-shell` se shell/auth tocados.
5. **Update honesty** — `TO_RELEASE.md` e review notes: PASS/PARTIAL/OPEN com evidência; retirar claims se algo regredir.
6. **Não commit/push** a menos que o usuário peça explicitamente.

### Comandos de regressão mínima (sempre no fim de wave)

```bash
cargo test -p xai-grok-sampling-types --lib preserves_none_slots
cargo test -p xai-grok-sampling-types --lib materialize
cargo test -p xai-grok-sampling-types --lib prompt_cache_key_opaque
cargo test -p xai-grok-sampling-types --lib append_function
cargo test -p xai-grok-multi-auth --lib request_stamp
cargo test -p xai-grok-sampler --lib auth_info_and_prefix_use_peek
cargo test -p xai-grok-sampler --lib commentary_phase
cargo test -p xai-grok-sampling-types --lib --quiet
cargo test -p xai-grok-sampler --lib --quiet
cargo test -p xai-grok-multi-auth --lib --quiet
cargo check -p xai-grok-shell
```

Acrescentar filtros novos da wave (refresh lock, journal, FC sibling, model_cache, etc.).

---

## Matriz de status alvo (atualizar ao fechar)

| ID | Alvo deste goal | Status de partida |
|----|-----------------|-------------------|
| 001 phase 1:1 | **manter PASS** | PASS |
| 002 materialize+FC args | **manter PASS** | PASS |
| 003 FC sibling/MCP/unknown | **fechar** | PARTIAL |
| 004/005 opaque Codex key | **manter PASS** | PASS |
| 006 A1 attempt stamp | **manter PASS** | PASS |
| 007 cross-process refresh | **fechar** | OPEN |
| 008 journal fail-loud | **fechar** | OPEN |
| 009 A4 pin residual | **fechar** | PARTIAL |
| 010 model cache M7 | **fechar** | PARTIAL |
| 011 capability gate | fechar se tempo (P1) | PARTIAL |
| 012 PC8 live | **fechar ou cut honesto** | OPEN |

---

## Claims permitidos vs proibidos

| Claim | Quando permitido |
|-------|------------------|
| “Experimental offline gates PASS” | Já verdade; manter se regressões verdes |
| “Production-ready (single-machine multi-process)” | Só após Wave A (+ B se wire for P0 do claim) PASS |
| “Prompt cache hits proven” | Só após Wave C live PASS |
| “100% Codex path” | Todos os gates DoD PASS (PC live ou cut documentado) |
| “Multi-provider product complete” | **Proibido** neste goal (falta keyring/xAI/D10 etc.) |

---

## Entregáveis finais da sessão goal

1. Código + testes para waves executadas (A→… conforme tempo; priorizar A depois B).
2. `TO_RELEASE.md` atualizado com matriz PASS/OPEN real.
3. Relatório final: o que fechou, o que falta, comandos rodados + resultados, claims honestos.
4. Se live rodou: path SCRATCH redigido.
5. **Nunca** declarar production-ready ou 100% sem a matriz DoD fechada.

---

## Ordem se tempo for limitado

1. **Wave A** (007/008) — inegociável para prod
2. **Wave B** (003 + A4 residual) — wire honesty + cache prefix
3. **Wave C** se credencial existir; senão documentar cut
4. **Wave D** catalog
5. **Wave E** polish

Se bloquear em credencial live, **não parar o goal**: feche A+B+D offline e deixe C como EXTERNAL_SETUP / NOT claimed.

---

## Uma frase de missão

> Feche auth multi-process + journal, wire residual FC/MCP, binding pin residual e (se possível) prova live de cache + catalog M7 — sem regredir phase/materialize/stamp/opaque-key — e só então permita o claim production-ready com docs honestas.
