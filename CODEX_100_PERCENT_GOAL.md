# GOAL PROMPT — Codex 100% (wire + UX + prompt cache + control plane)

> **How to use:** paste this entire document as the agent/session goal.
> **Repo:** `grok-goblin` · branch `goblin-multi-provider-codex`
> **Normative inventory:** `TO_RELEASE.md`
> **Protocol baseline:** `docs/architecture/multi-provider-auth/protocol-baseline.md`
> **Plan source:** `task.md` · multi-provider auth docs under `docs/architecture/multi-provider-auth/`
> **Reference implementation (behavior, not copy):** `~/forge/forge-responses-api`
> **Evidence dir:** `SCRATCH/` (live probes, SSE dumps, cache proof)

---

## Mission

Deliver a **complete, production-honest Codex integration** in Goblin so that:

1. **OpenResponses / ChatGPT Codex wire fidelity** matches what the live API actually emits and expects (stream, phases, multi-message, reasoning, tools, resend).
2. **Prompt cache is first-class and complete** — stable keys, stable prefixes, correct history resend, observable `cached_tokens`, and empirical proof of cache hits on multi-turn.
3. **Auth / credential control plane** is correct under concurrency (no global stamp races, real single-flight, journal safety).
4. **User-facing surfaces** (TUI + headless plain/JSON) show commentary/thoughts and final answers correctly.
5. **Docs and readiness claims** match code evidence (no false BLOCKERS_PASS).

**Definition of “100% Codex” for this goal** = all **P0 wire (C\*)** + **P0 prompt-cache (PC\*)** + **P0 auth (A1–A4)** + **model catalog cache (M7/D9)** + **honest docs (A5/P6/O2)** + **gated live proof**.
R-items (keyring, full xAI adapter, D10 OAuth approval) remain **1.0 multi-provider** unless unblocked; do not claim 100% product multi-provider without them, but **do** claim 100% Codex path when C+PC+A+M7 pass.

---

## Non-goals (explicit)

- Do **not** weaken fail-closed login (`GROK_CODEX_OAUTH_APPROVED` / explicit client id) without D10 decision.
- Do **not** force-push, rewrite history, or push to `xai-org` without permissions; use `fork` PR.
- Do **not** invent upstream fields; prove from live SSE / OpenAPI / forge docs.
- Do **not** mark complete without tests + (where possible) live SCRATCH evidence.
- Do **not** treat “final answer EXIT=0 once” as prompt-cache complete.

---

## Already shipped (do not regress)

| Area | Evidence |
|------|----------|
| Catalog keys `codex/{credential_id}/{slug}` | multi-auth catalog merge |
| Request-time Bearer + TokenManager resolve | `BearerResolver` |
| Current-thread safe resolve (`block_on_safe`) | no LocalSet panic |
| Short slug + full catalog key; multi-account ambiguity error | shell/model resolve |
| System/developer → `instructions` hoist | Codex 400 fix |
| Empty `response.completed.output` recovery from `output_item.done` + text fallback | live `pong` |
| Effort menu + CLI `--reasoning-effort` / `--effort` | stamp **after** merge |
| Login fail-closed without approval env | D10 |
| Usage parse includes `input_tokens_details.cached_tokens` → `cached_prompt_tokens` | sampler stream |
| Partial types: `AssistantPhase` + `AssistantItem.phase` / `message_id` | **struct only — not end-to-end** |

---

## Workstreams (implement all)

### Wave 0 — Baseline honesty

- [ ] **H1** Align `TO_RELEASE.md`, `PROGRESS.md`, readiness matrix with reality (withdraw false PASS).
- [ ] **H2** Keep a running evidence log under `SCRATCH/codex-100/` (commands, EXIT, key log lines, JSON snippets).

---

### Wave 1 — OpenResponses / Codex wire fidelity (C1–C5)

#### C1 — Capture & preserve `phase`

**Problem:** Live Codex emits assistant messages with `phase: "commentary" | "final_answer"`.
`async-openai` `OutputMessage` has **no** `phase` field → serde drops it. Canary test documents this.

**Required:**

1. Capture phase from **raw SSE JSON** (or side-channel map `item_id`/`msg_*` → phase) on:
   - `response.output_item.added` / `response.output_item.done`
   - any message payload that carries `phase`
2. Store on `AssistantItem.phase` + `message_id`.
3. On resend (`ConversationItem` → Responses `input`), serialize `phase` on assistant messages when present.
4. Unit tests: parse fixture SSE with commentary + final_answer; assert two phases survive round-trip.
5. Do **not** depend on async-openai adding the field; if upstream adds it later, keep dual path.

#### C2 — Surface commentary / reasoning to the user

**Problem:** Headless plain no-ops thought chunks; TUI depends on thinking settings; commentary never reaches thought channel.

**Required:**

1. Stream path: while `phase=commentary` (or reasoning summary deltas), emit `AgentThoughtChunk` (or equivalent shell event).
2. Final path: only `phase=final_answer` (or last assistant without commentary-only) is primary assistant text.
3. Headless **plain**: show thoughts when commentary/reasoning present (or gated by flag with default **on** for Codex).
4. Headless **JSON**: stable event types for thought vs message.
5. TUI: commentary routes like thinking; final as assistant bubble.
6. Tests for event routing; optional live probe logs both channels.

#### C3 — One `AssistantItem` per wire message (no collapse)

**Problem:** `response_to_conversation_items` concatenates all `OutputItem::Message` into **one** assistant blob + tools.

**Required:**

1. Emit **one** `ConversationItem::Assistant` per message item (preserve order with reasoning / function_call siblings).
2. Tool calls stay attached to the correct assistant turn (or documented Responses ordering rules).
3. Update all call sites / session persistence / UI that assumed single trailing assistant.
4. Fixture: commentary message + final_answer message + tools → N items, not 1.

#### C4 — Stream materialize = forge-strength

**Problem:** Recovery only when `completed.output` is empty; weaker than full materialize from stream.

**Required:**

1. Materialize `output[]` from **all** stream item events (`output_item.added`/`done`), by `output_index` / id, not only empty-completed recovery.
2. Prefer completed snapshot when non-empty **and** consistent; else stream map wins with diagnostics.
3. Preserve reasoning, function_call, custom_tool, message order.
4. Tests with multi-item SSE fixtures (empty completed, partial completed, reordered done).

#### C5 — History resend fidelity (feeds prompt cache)

**Required:**

1. Resend preserves: `phase`, reasoning items (incl. encrypted/`tco_*` blobs if API requires), backend tool calls, function_call + outputs, message ids where required.
2. System/developer stay on `instructions` (not re-injected as `input` roles).
3. Stable ordering identical to original emit order (prefix KV-cache friendliness — see PC wave).
4. Multi-turn + tool-loop integration tests.

---

### Wave 2 — Prompt cache **complete** (PC1–PC12) — P0

This is a first-class product requirement, not a nice-to-have. Codex/OpenAI-style caching is **provider-managed** prefix cache. Goblin must **enable, stabilize, observe, and prove** it.

#### PC1 — Request field: `prompt_cache_key`

**Current bug:** `impl From<&ConversationRequest> for CreateResponse` hardcodes:

```text
prompt_cache_key: None
previous_response_id: None
prompt_cache_retention: None
```

**Required:**

1. Add `ConversationRequest.prompt_cache_key: Option<String>` (and builder).
2. Map into `CreateResponse.prompt_cache_key`.
3. Accept legacy alias only if needed for interop; official field is snake_case `prompt_cache_key`.
4. Unit test: non-None key appears in serialized request body.

#### PC2 — Stable key derivation (session affinity)

**Required:** derive a **stable** key per conversation/thread, not per turn:

| Priority | Source |
|----------|--------|
| 1 | Explicit session / thread id (Goblin session id) |
| 2 | `x_grok_session_id` / conversation id already on request |
| 3 | Durable fallback: hash(credential_id + session_path) — never random per request |

Rules:

- Same session → same `prompt_cache_key` across turns.
- New session → new key.
- Subagent / worktree: **own** key (do not share parent key unless intentional continuation).
- Document key format (e.g. `goblin-sess-<uuid>`).

#### PC3 — Prefix stability (the real cache enabler)

Cache hits require a **byte-stable prefix**. Implement and test:

1. **`instructions`** stable for the session (system/developer hoist deterministic; no timestamp/noise in system unless product requires it).
2. **History order** identical to prior turns (C3/C5).
3. **Tools schema** stable ordering (sort only if already sorted historically — do not reshuffle tool defs mid-session).
4. **Model + reasoning.effort** consistent for the continuation (or document that effort change breaks prefix).
5. Do not strip/rewrite earlier assistant/reasoning/tool items between turns except explicit compaction.
6. Compaction: when history is compacted, **new** cache prefix starts; document + optional new `prompt_cache_key` suffix generation.

#### PC4 — `previous_response_id` (optional chain mode)

**Required design (implement or explicitly document + stub with tests):**

1. Option A (default for Goblin agent): full `input` replay + `prompt_cache_key` (no server store dependency).
2. Option B: server-side chain via `previous_response_id` + `store` when API supports it for Codex backend.
3. If Option B: handle `previous_response_not_found`; fall back to full replay without hanging.
4. Never send both incompatible shapes without a tested policy.

#### PC5 — `prompt_cache_retention`

1. Discover live Codex acceptance (send / omit / reject).
2. Policy: omit if rejected; never silently send unsupported retention that causes 4xx.
3. Document in protocol baseline.

#### PC6 — Account / routing affinity

1. Cache is per-account/token affinity on ChatGPT backend — **same credential** must back consecutive turns of a session.
2. Multi-account: binding must not flip mid-session (ties to A4 `ModelBinding`).
3. Log credential id (not secret) + cache key on each request for diagnosis.

#### PC7 — Observe `cached_tokens` end-to-end

Already partially parsed. Complete:

1. Stream + non-stream: `usage.input_tokens_details.cached_tokens` (and alternate shapes if seen live).
2. Surface in: sampling metrics, session usage, headless JSON usage event, optional TUI.
3. Structured log each turn: `input_tokens`, `cached_tokens`, `cache_hit_ratio`, `prompt_cache_key`.
4. Regression tests with fixture usage JSON (existing client tests extend).

#### PC8 — Live **proof** probe (gate for “cache complete”)

Implement `scripts/` or `cargo` bin / test gated by env:

```text
GROK_LIVE_CODEX=1  # or existing live gate
```

Protocol (mirror forge `verify:prompt-cache` intent):

1. Turn 1: large stable system/instructions + long user prefix + unique suffix A → record `cached_tokens` (often 0).
2. Turn 2: **same** instructions + same history prefix + new user suffix B → **require** `cached_tokens > 0` (or documented minimum ratio).
3. Turn 3: tool loop if tools enabled — still non-decreasing cache on shared prefix where API allows.
4. Negative control: change `prompt_cache_key` or mutate early history → cache drop expected.
5. Save full SSE + usage JSON under `SCRATCH/codex-100/prompt-cache/`.
6. Fail CI job only when gate env set; unit tests always run offline.

#### PC9 — Stream path does not break cache accounting

1. Empty `completed.output` recovery must not drop usage.
2. Materialize must not invent tokens; usage comes from completed/usage events.
3. Multi-message commentary must not double-count usage.

#### PC10 — Compaction / context management interaction

1. When Goblin compacts history, define: new key vs same key + shorter prefix.
2. After compact, turn N+1 may show lower `cached_tokens`; must not crash.
3. Tests with synthetic compaction.

#### PC11 — Headless / API / subagent

1. Every code path that builds Codex Responses requests sets PC key (shell, headless, subagent, title-gen if Codex — see P4).
2. Title generation must **not** pollute main session cache key (separate key or non-Codex path).

#### PC12 — Docs for operators

1. Document how cache works, how to read `cached_tokens`, how to debug 0 hits (prefix mutation checklist).
2. Update `TO_RELEASE.md` with PC checklist status.
3. README Experimental blurb mentions prompt-cache status honestly.

**Prompt-cache acceptance (all required):**

- [ ] Non-null stable `prompt_cache_key` on live multi-turn
- [ ] Resend preserves phase + reasoning + tools order
- [ ] Live turn-2 `cached_tokens > 0` under SCRATCH proof (with real account)
- [ ] Metrics/logs expose cache hit
- [ ] Unit + fixture tests green without network
- [ ] Negative control documented

---

### Wave 3 — Auth / store / concurrency (A1–A4)

| ID | Work |
|----|------|
| **A1** | Request-scoped `SentCredentialStamp` (not process-global last-wins map) |
| **A2** | Shared long-lived TokenManager + real single-flight; design cross-process lock |
| **A3** | Journal recovery under lock, fail-loud, race/fault tests |
| **A4** | Typed `ModelBinding` session state (not wire header as sole binding) |
| **A5** | Honest readiness docs |

Do not claim Beta without A1–A4 proofs.

---

### Wave 4 — Model catalog cache (M7 / D9)

| Item | Requirement |
|------|-------------|
| Path | `~/.grok/cache/models/codex/<credential-id>.json` |
| TTL | 5 minutes fresh |
| ETag | revalidate when supported |
| Isolation | per credential; never cross-account |
| Offline | serve stale; mark stale in UI/logs |
| Empty | bundled fallback models |
| Errors | do not delete cache on transient 5xx |
| Invalidate | on account identity change |
| Catalog | no blocking live fetch on every UI tick without cache |

---

### Wave 5 — Product / OSS (P\* needed for Beta)

| ID | Work |
|----|------|
| **P1** | TUI login modal / polished Codex login |
| **P2** | Multi-provider CLI (accounts, scoped logout, revoke) |
| **P3** | Interactive `/model` + effort validation |
| **P4** | Title gen must not hit xAI proxy on Codex sessions |
| **P5** | Optional MCP for headless smokes |
| **P6** | User docs: env gates, install, limits, cache |
| **P7** | CI: multi-auth + sampling-types + effort + cache unit tests; optional gated live |

---

### Wave 6 — Architecture 1.0 (R\*) — after Beta

| ID | Work |
|----|------|
| **R1** | Real xAI multi-provider adapter |
| **R2** | Keyring-first credential store |
| **R3** | Generic `RequestAuthResolver` composition root |
| **R4** | Parent/subagent concurrent multi-account isolation |
| **R5** | Cross-process refresh lock with proof tests |
| **R6** | D10 production OAuth client approval |

---

### Wave 7 — Release hygiene (O\*)

| ID | Work |
|----|------|
| **O1** | PR into intended public repo with permissions |
| **O2** | README Works / Doesn’t = this goal |
| **O3** | LICENSE/NOTICE for fork divergence |
| **O4** | No secrets in docs; clean-machine install smoke |

---

## Implementation order (strict)

1. **C1 + C3 + C4** (types + materialize + no collapse) — foundation for UI and cache
2. **C5 + PC1 + PC2 + PC3** (resend + key + prefix) — enable cache
3. **C2** (thought/commentary UX)
4. **PC6–PC9 + PC8 live probe** — prove cache
5. **PC10–PC12** compaction + paths + docs
6. **A1–A4** auth hardening
7. **M7/D9** model cache
8. **P4, P7, A5, P6, O2** product honesty
9. R\* / O\* as capacity allows

---

## Key code surfaces (start here)

| Concern | Paths |
|---------|--------|
| Phase types | `crates/codegen/xai-grok-sampling-types/src/conversation.rs` (`AssistantPhase`, `AssistantItem`) |
| Collapse bug | `response_to_conversation_items` (same file) |
| CreateResponse / cache None | `impl From<&ConversationRequest> for rs::CreateResponse` |
| System hoist | `hoist_system_messages_to_instructions` |
| Stream materialize | `crates/codegen/xai-grok-sampler/src/stream/responses.rs` |
| Client send path | `crates/codegen/xai-grok-sampler/src/client.rs` |
| Thoughts / plain | shell headless event mapping; `AgentThoughtChunk` |
| Auth / stamp | `crates/codegen/xai-grok-multi-auth/` |
| Model catalog | multi-auth Codex models client |
| Effort stamp | shell merge order tests `codex_effort_after_merge` |
| Reference behavior | `~/forge/forge-responses-api` (prompt_cache_key, phase preserve, materialize, usage) |

---

## Validation matrix

| Layer | Command / action | Pass criteria |
|-------|------------------|---------------|
| Unit | `cargo test -p xai-grok-sampling-types …` | phase round-trip, no collapse, hoist, cache key serialize |
| Unit | `cargo test -p xai-grok-sampler …` | stream materialize, usage cached_tokens |
| Unit | `cargo test -p xai-grok-multi-auth …` | token/stamp/journal |
| Integration | effort-after-merge, model resolve | existing + new |
| Live wire | `goblin -p … --model <codex>` | final text + EXIT=0 |
| Live phase | log/SCRATCH SSE | commentary + final both present internally |
| Live cache | gated probe PC8 | turn-2 `cached_tokens > 0` |
| Docs | TO_RELEASE / README | match evidence |

---

## Done checklist (100% Codex path)

### Wire

- [ ] C1 phase capture + resend
- [ ] C2 thought/commentary UX (TUI + headless)
- [ ] C3 multi-assistant no collapse
- [ ] C4 full stream materialize
- [ ] C5 history fidelity

### Prompt cache

- [ ] PC1–PC3 key + prefix
- [ ] PC4 policy for previous_response_id
- [ ] PC5 retention policy
- [ ] PC6 account affinity
- [ ] PC7 observability
- [ ] PC8 live proof SCRATCH
- [ ] PC9–PC12 stream/compaction/paths/docs

### Control plane

- [ ] A1–A4 + A5 honesty
- [ ] M7/D9 model cache

### Beta polish

- [ ] P4 title noise
- [ ] P6/P7/O2 docs + CI

### Explicitly still not “1.0 multi-provider”

- [ ] R1–R6 (track separately)

---

## Agent execution rules

1. Work in inspect → change → test → evidence loops; update `TO_RELEASE.md` checkboxes as items close.
2. Prefer smallest complete fixes; no drive-by refactors.
3. When blocked on live API behavior, dump SSE to SCRATCH and adjust from evidence.
4. Never commit secrets or raw OAuth tokens.
5. If the same failure survives two fix attempts, stop, re-hypothesize from logs, then continue.
6. Final report: files changed, tests run, SCRATCH paths, remaining R\*/deferred items.

---

## One-line kickoff for an executor

```text
Implement Codex 100% per CODEX_100_PERCENT_GOAL.md: finish C1–C5 OpenResponses fidelity,
full prompt-cache (PC1–PC12 including stable prompt_cache_key, prefix stability, cached_tokens
observability, and gated live proof), then A1–A4 + M7 model cache; do not claim complete
without SCRATCH evidence; follow TO_RELEASE.md tiers; do not regress shipped multi-auth path.
```
)
