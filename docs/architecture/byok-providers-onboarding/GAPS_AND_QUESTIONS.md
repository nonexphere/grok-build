# Gaps and Questions — Decisions for the Planner / Maintainer

Items marked **[HUMAN]** need product choice. Items marked **[PLAN]** are for the architect to resolve with evidence. Items marked **[IMPL]** are execution detail after architecture is fixed.

---

## 1. Product scope

| ID | Question | Options | Suggested default for discussion |
|----|----------|---------|----------------------------------|
| Q1 | **[HUMAN]** MVP providers? | All three vs phased OR → Groq → CF | All three L1–L2 if same pattern |
| Q2 | **[HUMAN]** Multi-key per provider in v1? | Single key only vs multi | Single key UX, multi-key data model |
| Q3 | **[HUMAN]** TUI login required in v1? | CLI only vs CLI+TUI | CLI first |
| Q4 | **[HUMAN]** Auto-import entire OpenRouter catalog? | All / curated / search-on-demand | Curated + search |
| Q5 | **[HUMAN]** Must web search work on BYOK models? | Yes / no / later | No for chat-only MVP |
| Q6 | **[HUMAN]** Persist default model to config.toml? | Yes / session-only | Yes (non-secret) |

---

## 2. Architecture choice

| ID | Question | Notes |
|----|----------|-------|
| Q7 | **[HUMAN]** Candidate A vs B vs C vs D? | See `ARCHITECTURE_TENSIONS.md` |
| Q8 | **[PLAN]** If A/D: finish `LoginTransport::ApiKey` generically or per provider? | Generic coordinator preferred |
| Q9 | **[PLAN]** How are synthetic models marked vs user TOML models? | Avoid overwrite conflicts |
| Q10 | **[PLAN]** Interaction with `GROK_MODELS_BASE_URL` when both set? | Precedence table required |
| Q11 | **[PLAN]** Feature flag for BYOK providers? | Align with `native-multi-provider-auth` or new flag |

---

## 3. API backend policy

| ID | Question | Notes |
|----|----------|-------|
| Q12 | **[HUMAN]** First slice chat-only for all three? | Strongly recommended |
| Q13 | **[PLAN]** When to enable OpenRouter/Groq Responses? | Per-model allowlist vs user toggle |
| Q14 | **[PLAN]** How to show backend in `/model` UI? | Display suffix? meta field? |
| Q15 | **[PLAN]** History conversion on backend switch mid-session | Strip unsupported items? refuse switch? |
| Q16 | **[PLAN]** Tool-capable model filter for agent mode | Hide non-tool models? |

---

## 4. Cloudflare-specific

| ID | Question | Notes |
|----|----------|-------|
| Q17 | **[HUMAN]** Workers AI only vs AI Gateway too? | Workers AI first |
| Q18 | **[PLAN]** Account ID validation UX | Paste vs extract from URL |
| Q19 | **[PLAN]** Token scope check | Soft warning vs hard fail |
| Q20 | **[PLAN]** Model list source if `/models` incomplete | Bundled list of popular `@cf/…` |

---

## 5. Security and storage

| ID | Question | Notes |
|----|----------|-------|
| Q21 | **[HUMAN]** Allow `api_key` in config.toml still? | Keep for power users; wizard never writes it |
| Q22 | **[PLAN]** Secret backend for API keys | File multi-auth now; keyring later |
| Q23 | **[PLAN]** Redaction tests for new paths | Reuse Codex canary lengths 4/8/12/20 |
| Q24 | **[PLAN]** Status command fields for BYOK | alias, provider, created_at, **no** key |

---

## 6. CLI / UX grammar

| ID | Question | Suggested shape (debate) |
|----|----------|--------------------------|
| Q25 | **[PLAN]** Login grammar | `goblin login --provider openrouter` + prompt key |
| Q26 | **[PLAN]** Non-TTY | `--api-key-env OPENROUTER_API_KEY` or `--api-key-file` (avoid argv secrets) |
| Q27 | **[PLAN]** Logout | `logout --provider groq` vs global logout only |
| Q28 | **[PLAN]** Models | `goblin models` includes BYOK merged entries |
| Q29 | **[PLAN]** Alias | user-facing `openrouter/default` vs only UUID |

**Security note:** Prefer env/file for keys in scripts; interactive paste for TTY. Avoid `history`-visible `--api-key sk-…` as the primary path.

---

## 7. Catalog identity

| ID | Question | Notes |
|----|----------|-------|
| Q30 | **[PLAN]** Key format | Reuse `provider/{uuid}/{slug}` |
| Q31 | **[PLAN]** Short slug resolution | Same ambiguity rules as Codex multi-account |
| Q32 | **[PLAN]** Display names | `Llama 3.3 (Groq)` vs raw id |
| Q33 | **[PLAN]** Conflict with user `[model.groq-llama]` | User TOML wins? fail? namespace? |

---

## 8. Request path

| ID | Question | Notes |
|----|----------|-------|
| Q34 | **[PLAN]** Bearer resolve | Static from store each request vs cache in memory with generation |
| Q35 | **[PLAN]** 401 behavior | No refresh → clear reauth message; no infinite retry |
| Q36 | **[PLAN]** Extra headers for OpenRouter | Defaults for Goblin app referer/title? |
| Q37 | **[PLAN]** Cloudflare base URL rebuild on account change | Must rebuild ModelBinding / catalog |

---

## 9. Testing and release

| ID | Question | Notes |
|----|----------|-------|
| Q38 | **[PLAN]** Offline unit tests with fixtures | Frozen `/models` JSON per provider |
| Q39 | **[PLAN]** Live smoke | Opt-in env + durable redacted evidence under `.llms/evidence/` |
| Q40 | **[PLAN]** README honesty | Document chat-only MVP limitations |
| Q41 | **[PLAN]** Relation to `TO_RELEASE.md` | Separate tier from Codex production claims |

---

## 10. Gaps confirmed in code (not optional)

These are **facts**, not questions:

1. No AuthProvider modules for openrouter/groq/cloudflare.
2. `LoginTransport::ApiKey` not implemented end-to-end.
3. Login CLI parser only accepts xai/grok/codex.
4. Catalog merge for multi-auth exists for Codex only.
5. Default `api_backend` is chat_completions — good for these three, bad if someone copies Codex merge blindly (`Responses`).
6. Global models_base_url cannot express multi-provider multi-key.
7. Skill text currently steers pure custom-model work away from `@add-provider` — goal as stated **is** lifecycle-owned, so skill boundary needs update after architecture choice.

---

## 11. Out-of-scope risks to name, not silently absorb

- OpenRouter OAuth for user accounts (optional future).
- Billing/balance display.
- Automatic provider failover on rate limit.
- Scraping web UIs for keys.
- Claiming Responses feature parity for all models.
- Treating Cloudflare CDN notes in codebase as Workers AI integration.

---

## 12. Minimum vertical slice definition (problem-level)

A planner’s first delivery should be able to say **PASS/FAIL** on:

```text
Given a fresh GROK_HOME
When user completes onboarding for ONE provider (recommend OpenRouter first)
And selects one model from the populated catalog
Then headless `goblin -p "ping" --model <catalog-key>` returns EXIT 0
And auth status shows the provider account without secrets
And logout removes credential and catalog entries
And xAI/Codex paths still pass package tests
```

Then replicate for Groq and Cloudflare with provider-specific fields.

---

## 13. What ChatGPT should output next

See README “Suggested prompt”. Expected artifacts from the planner:

1. Architecture decision record (chosen candidate + rejected options).
2. Provider implementation matrix (rows from add-provider checklist adapted for API keys).
3. Phased plan with tests.
4. Explicit answers to every **[HUMAN]** item (or list them as blockers).
5. File-level touch list based on `EVIDENCE_MAP.md`.
