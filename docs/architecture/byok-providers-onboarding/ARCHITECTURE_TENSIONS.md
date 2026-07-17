# Architecture Tensions — Bridging BYOK UX and Existing Systems

This document names the **hard design collisions** a planner must resolve. It does not pick a winner.

---

## Tension T1 — Two identity systems

| World A (custom models) | World B (multi-auth) |
|-------------------------|----------------------|
| Catalog key = user string (`my-model`) | Catalog key = `provider/credential_uuid/slug` |
| Secret lives in TOML / env | Secret lives in credential store |
| No first-class provider id | `ProviderId` is first-class |
| Account = none | Account = credential (+ optional provider_account_id) |
| Binding ≈ model key + static api_key | Binding = immutable `ModelBinding` |

**Product ask** wants World B UX with World A economics (API keys, no OAuth).

### Resolution options (planner must choose)

1. **Extend World B** with `LoginTransport::ApiKey` providers for openrouter/groq/cloudflare; merge catalogs like Codex.
2. **Productize World A** with a wizard that writes `[model.*]` blocks (and maybe a secrets file).
3. **Hybrid:** store secrets in multi-auth; materialize ephemeral `ModelEntry`s without persisting keys in TOML.

Codex lessons strongly push away from “put live tokens in `ModelEntry.api_key`” for anything that needs lifecycle. API keys don’t refresh, but multi-key + logout + status still benefit from the store.

---

## Tension T2 — “Provider” vs “model endpoint”

Users say “I use Groq”. The codebase thinks in **models** that happen to share a base URL.

Risks:

- User expects one Groq login → all models appear.
- Config model assumes one row per selectable model.
- Features like web search are **per model**, not per provider.

Any design must define:

```text
ProviderCredential 1 ──N→ CatalogModel entries ──1→ SamplingConfig (backend, headers, slug)
```

---

## Tension T3 — Chat Completions vs Responses (capability cliff)

See `API_BACKENDS.md`.

Agent features are uneven:

- Codex path heavily Responses-optimized.
- Groq/OR/CF happy path is Chat Completions.
- Web search documented as Responses-only.

If onboarding only enables Chat Completions models:

- Users may think “Goblin is broken” when web search / some agent paths expect Responses.
- Session history conversion across backend switch needs clear rules.

**Planner must specify feature matrix per backend**, not imply full parity.

---

## Tension T4 — Global `XAI_API_KEY` fallback

Credential resolve order falls through to `XAI_API_KEY`. For third-party base URLs this is dangerous:

- Wrong key sent to OpenRouter/Groq → confusing 401.
- Accidental leak of xAI key to third party if misconfigured.

Onboarded providers should use **own credentials only** (fail closed if missing), never silent fallback to `XAI_API_KEY`.

---

## Tension T5 — `GROK_MODELS_BASE_URL` is single-tenant

Enterprise env vars assume **one** external OpenAI-compatible stack.
User wants **simultaneous** OpenRouter + Groq + Cloudflare + xAI + Codex.

Therefore `GROK_MODELS_BASE_URL` is **not** the multi-provider solution; at best a migration path or power-user override.

---

## Tension T6 — API_KEY_LOGIN capability exists but is unfinished

`ProviderCapabilities::API_KEY_LOGIN` is defined.
`LoginCoordinator::run_login` rejects `LoginTransport::ApiKey`.
Codex/xAI providers error on ApiKey transport.

Implementing three providers means **finishing a transport class**, not only three modules.

---

## Tension T7 — Multi-account pattern vs single-key MVP

Codex invested in multi-account (aliases, defaults, ambiguous short slugs).

For BYOK:

| MVP | Full |
|-----|------|
| One key per provider | N keys per provider |
| Short model slugs OK if unique in catalog | Always credential-scoped keys |
| Simple logout removes provider | Per-key logout |

Maintainer already has multi-account Codex; consistency argues for credential-scoped keys from day one **even if UI only allows one key** (avoids B2-class collision when second key is added).

---

## Tension T8 — Cloudflare is not “key only”

Cloudflare needs **account_id in base URL**. That is provider-specific config, not pure secret storage.

Compare:

| Provider | Secret | Non-secret config |
|----------|--------|-------------------|
| OpenRouter | key | optional headers |
| Groq | key | none |
| Cloudflare | token | account_id (and maybe gateway_id) |

`AuthProvider` / store metadata must allow **structured account fields** (already `ProviderAccountInfo.metadata` / workspace fields exist for Codex).

---

## Tension T9 — Catalog size (OpenRouter)

OpenRouter can return a huge model list. Codex lists a manageable set.

UX tension:

- Import all → unusable picker.
- Import none until user types model id → weaker “automatic models” story.
- Curated defaults + search/fetch → more product work.

Must be explicit in plan.

---

## Tension T10 — Skill `@add-provider` scope

`.agents/skills/add-provider/SKILL.md` says:

> Do not use for generic API-key model configuration that does not own login, credentials, refresh, or provider lifecycle; follow the existing custom-model path.

The maintainer goal **does** own login + credentials + lifecycle UX. So this work **is** `@add-provider` territory **if** framed as AuthProvider plugins — or a **new skill/path** for “API-key providers” if AuthProvider is overkill.

Planner should decide whether to:

- widen `add-provider` for API-key providers, or
- create `add-api-key-provider` skill, or
- keep custom-model wizard outside AuthProvider.

---

## Tension T11 — Security vs convenience

Wizard will handle raw secrets (paste in TTY). Requirements from multi-auth lessons:

- No secrets in status JSON.
- No prefix logging.
- Prefer not writing secrets into `config.toml`.
- File backend 0600 atomic writes already exist for multi-auth.
- Keyring deferred — plan must not claim keyring without implementing it.

---

## Tension T12 — Concurrent work on Codex multi-provider

The repo is mid multi-provider/Codex. Risk of:

- Conflicting catalog merge order.
- Dual TokenManager assumptions.
- Feature flag interactions.
- Docs drift.

BYOK work should **compose** with Codex merge, not fork a third auth system. Ideal: one catalog merge pipeline with provider-specific sources.

---

## Tension T13 — Default model selection ambiguity

With many providers logged in:

```text
Who is default on cold start?
```

Codex already has multi-account ambiguity rules for short slugs. Multi-provider multi-key needs a total policy:

```text
explicit CLI --model
> session pin
> config default
> last used
> sole available model
> prompt / error
```

Silent first-wins is forbidden by existing multi-auth philosophy.

---

## Architecture sketch candidates (for planner evaluation)

### Candidate A — “Codex-shaped API-key providers”

```text
AuthProvider{openrouter,groq,cloudflare}
  capabilities: API_KEY_LOGIN | MODEL_DISCOVERY | MULTI_ACCOUNT?
Login: prompt key (+ account_id)
Store: multi-auth secrets
list_models → merge catalog keys provider/uuid/slug
api_backend: provider default or per-model map
request auth: Bearer from store (no refresh)
```

Pros: consistent with Goblin direction; multi-key ready.
Cons: more code than a TOML writer; must implement ApiKey transport properly.

### Candidate B — “Config writer wizard”

```text
CLI wizard → appends [model.*] blocks + sets env instructions
Optional: write keys to a secrets include file
```

Pros: reuses 100% of World A.
Cons: multi-key and cleanup are messy; secrets in config risk; feels less “native”.

### Candidate C — “Provider profiles in config without per-model rows”

```toml
[provider.openrouter]
api_key_env = "OPENROUTER_API_KEY"
base_url = "..."
api_backend = "chat_completions"
models = ["auto"] # or curated
```

Pros: cleaner than N model rows.
Cons: **schema does not exist today**; shell catalog would need new loader.

### Candidate D — Hybrid A + C

Non-secret provider profile in config; secrets in multi-auth store; runtime catalog merge.

Often the best product, highest design cost.

---

## Invariants any candidate must keep

1. In-flight request binding does not flip.
2. No secret leakage in logs/status.
3. No silent `XAI_API_KEY` fallback for third-party hosts.
4. Correct `api_backend` for the selected model.
5. Existing custom `[model.*]` continues to work.
6. Codex OAuth path remains independent and non-regressed.
7. Compile-time registration of built-in providers (D6) if using AuthProvider registry.
