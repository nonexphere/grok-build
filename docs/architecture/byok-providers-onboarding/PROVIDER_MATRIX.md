# Provider Matrix — Cloudflare, Groq, OpenRouter

**Purpose:** Capture wire/auth/model facts for planning.  
**Sources:** Official docs research (2026-07), plus Goblin code evidence for how we would attach.  
**Not** a promise of full API surface support.

---

## 1. Comparative overview

| Dimension | OpenRouter | Groq | Cloudflare Workers AI |
|-----------|------------|------|------------------------|
| Auth model | API key (Bearer) | API key (Bearer) | API **token** (Bearer) + **Account ID** |
| Typical env name | `OPENROUTER_API_KEY` | `GROQ_API_KEY` | `CLOUDFLARE_API_TOKEN` + `CLOUDFLARE_ACCOUNT_ID` |
| Base URL (OpenAI SDK style) | `https://openrouter.ai/api/v1` | `https://api.groq.com/openai/v1` | `https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1` |
| Primary inference API | Chat Completions | Chat Completions | Chat Completions (OpenAI compat) |
| Responses API | **Beta** `/api/v1/responses` | Documented for some models | Not the primary OpenAI-compat story |
| Model list | `GET /api/v1/models` | `GET /openai/v1/models` | List via CF APIs / catalog (not always identical to simple OpenAI `/models`) |
| Model ID style | `vendor/model` (e.g. `openai/gpt-4o`) | Provider slugs (e.g. `llama-3.3-70b-versatile`) | `@cf/org/model` (e.g. `@cf/meta/llama-3.1-8b-instruct`) |
| Extra headers | Optional `HTTP-Referer`, `X-Title` for rankings | Usually none | Account already in URL; token scopes matter |
| Multi-account | Multiple API keys | Multiple API keys | Multiple account IDs × tokens |
| OAuth for apps | Possible (docs mention OAuth for keys) | Not primary for this product | Not primary |
| Fits Goblin `AuthScheme::Bearer` | Yes | Yes | Yes |
| Fits default `api_backend=chat_completions` | Yes | Yes | Yes |
| Needs URL templating with account | No | No | **Yes** |

---

## 2. OpenRouter

### 2.1 Auth

- Header: `Authorization: Bearer <OPENROUTER_API_KEY>`
- Key created at openrouter.ai keys UI
- Optional headers (docs):
  - `HTTP-Referer` — site URL
  - `X-Title` — site title  
  Goblin already has `extra_headers` infrastructure; product may set Goblin defaults.

### 2.2 Endpoints

| Use | URL |
|-----|-----|
| Chat Completions | `POST https://openrouter.ai/api/v1/chat/completions` |
| Models | `GET https://openrouter.ai/api/v1/models` |
| Responses (beta) | `POST https://openrouter.ai/api/v1/responses` |

Base URL for Goblin `base_url`: `https://openrouter.ai/api/v1`.

### 2.3 Backend policy for Goblin

- **Default:** `chat_completions` (mature path; matches most community tooling).
- **Optional later:** expose Responses for models/features that need it; do not force Responses as default (beta + different client code path).
- Tool calling: works via chat completions for many models; known past edge cases with argument JSON validation (already hardened in conversation code).

### 2.4 Model catalog considerations

- Large catalog (hundreds of models).
- Onboarding UX needs **filter** (top N, modality text-only, tool-capable, user search) — dumping entire list into `/model` may be unusable.
- Pricing/context metadata available from models API — useful for `context_window`.
- Model IDs contain `/` — catalog keys must not break on that (today custom TOML keys avoid `/`; multi-provider keys use `provider/uuid/slug` where slug can contain path segments depending on parse rules — **must verify** `format_provider_model_key` / `parse_provider_model_key` which use `splitn(3, '/')` → **slugs with extra `/` are OK** as the third segment can contain `/`).

### 2.5 Manual config equivalent (today)

```toml
[model.openrouter-gpt4o]
model = "openai/gpt-4o"
base_url = "https://openrouter.ai/api/v1"
env_key = "OPENROUTER_API_KEY"
api_backend = "chat_completions"
name = "GPT-4o (OpenRouter)"
```

### 2.6 Product onboarding sketch (not a plan)

```text
login openrouter → paste key → validate GET /models → store credential
→ merge selected or curated models into catalog → user picks model
```

---

## 3. Groq

### 3.1 Auth

- Header: `Authorization: Bearer <GROQ_API_KEY>`
- Console: console.groq.com keys

### 3.2 Endpoints

| Use | URL |
|-----|-----|
| Chat Completions | `POST https://api.groq.com/openai/v1/chat/completions` |
| Models | `GET https://api.groq.com/openai/v1/models` (convention) |
| Responses | Same base, Responses API documented for supported models |

Base URL: `https://api.groq.com/openai/v1`  
**Note:** missing `/v1` is a common integration bug (404).

### 3.3 Backend policy for Goblin

- **Default:** `chat_completions` for broad model coverage and speed-oriented use.
- Groq documents Responses API for certain models (e.g. OSS lines). Planner should allow per-model override once catalog metadata exists; first slice can stay chat-only.
- Do **not** use Groq usage `cached_tokens` as evidence of Codex prompt cache (existing audit note).

### 3.4 Model catalog considerations

- Smaller, faster-changing model list than OpenRouter.
- Context windows and tool support vary by model — store metadata when available.
- Streaming + tools are primary agent needs; validate with one tool-using smoke test per release class.

### 3.5 Manual config equivalent (today)

```toml
[model.groq-llama]
model = "llama-3.3-70b-versatile"
base_url = "https://api.groq.com/openai/v1"
env_key = "GROQ_API_KEY"
api_backend = "chat_completions"
name = "Llama 3.3 70B (Groq)"
```

---

## 4. Cloudflare Workers AI

### 4.1 Auth + tenancy

Unlike Groq/OpenRouter, Cloudflare needs **two** identity pieces for the common OpenAI-compat path:

1. **API Token** with Workers AI permission  
2. **Account ID** embedded in the URL  

```text
https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1
Authorization: Bearer {api_token}
```

Chat: `POST …/ai/v1/chat/completions`  
Models: OpenAI-compat models list under that base (verify during implementation).

There is also a native `…/ai/run/@cf/...` path — **not** what Goblin’s Chat Completions client expects. Prefer OpenAI-compat base.

### 4.2 AI Gateway variant (optional complexity)

Cloudflare AI Gateway uses different bases, e.g.:

```text
https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_id}/compat
```

or workers-ai-specific gateway paths.  
**Recommendation for problem scope:** first-class **Workers AI OpenAI-compat** with account_id + token; treat AI Gateway as a second profile later.

### 4.3 Backend policy for Goblin

- **`chat_completions` only** for v1 product claim.
- Model IDs like `@cf/meta/llama-3.1-8b-instruct` must pass through wire slug unchanged.
- Tool/function calling support is model-dependent and historically thinner than OpenAI/Groq — agent UX must tolerate limited tool models or filter them.

### 4.4 Onboarding fields (product)

| Field | Required | Notes |
|-------|----------|-------|
| API token | Yes | Secret |
| Account ID | Yes | Non-secret; used in base URL |
| Gateway ID | No (v1) | Gateway profile only |
| Default model | Optional | After list |

### 4.5 Manual config equivalent (today)

```toml
[model.cf-llama]
model = "@cf/meta/llama-3.1-8b-instruct"
base_url = "https://api.cloudflare.com/client/v4/accounts/ACCOUNT_ID/ai/v1"
env_key = "CLOUDFLARE_API_TOKEN"
api_backend = "chat_completions"
name = "Llama 3.1 8B (Cloudflare)"
```

User must substitute `ACCOUNT_ID` — exactly the pain automated onboarding should remove.

---

## 5. Cross-cutting protocol notes

### 5.1 `/models` discovery

| Provider | Reliability of OpenAI-style `/models` |
|----------|----------------------------------------|
| OpenRouter | High — primary discovery |
| Groq | High — OpenAI compat |
| Cloudflare | Medium — confirm exact list endpoint + filters (text generation only) |

Fallback strategy for plan: bundled curated model list if discovery fails, with clear UI that list is stale.

### 5.2 Validation after key entry

Minimum validation before persist:

1. Reject empty key / whitespace.  
2. `GET /models` or a tiny authenticated call → 401 = bad key.  
3. Cloudflare: validate account id format + 401/403 distinction.  
4. Never log key material (including prefixes).

### 5.3 Multi-key semantics

| Question | Implication |
|----------|-------------|
| One key per provider enough for MVP? | Simplifies aliases/default |
| Multiple keys (work/personal OpenRouter)? | Need credential-scoped catalog keys like Codex |
| Cloudflare multi-account | Different account_id ⇒ different base_url per credential |

Codex already solved multi-account identity; BYOK should **reuse the same key shape** if multi-key is in scope:

```text
openrouter/{credential_uuid}/{model_slug}
groq/{credential_uuid}/{model_slug}
cloudflare/{credential_uuid}/{model_slug}
```

### 5.4 Secret storage

Reuse multi-auth file store (`~/.grok/auth/`) vs writing into `config.toml`:

| Approach | Pros | Cons |
|----------|------|------|
| Store API key in multi-auth secret backend | Consistent with Codex; status can list accounts; no TOML secrets | Need API-key login transport + catalog merge |
| Write `env_key` + instruct user to export | Minimal code | Not “automatic”; still env management |
| Write plaintext `api_key` into config.toml | “Works offline” | Security smell; conflicts with redaction policy |
| OS keychain only | Best security | Keyring still deferred in multi-auth progress |

Problem preference (from fork direction): **store secrets out of TOML**, generate **synthetic catalog entries** at runtime (Codex pattern), optionally **write non-secret preferences** (default model alias) to config if needed.

---

## 6. What “support” means per provider (definition of done levels)

| Level | Meaning | OpenRouter | Groq | CF |
|-------|---------|------------|------|-----|
| L0 | Manual TOML works | Yes | Yes | Yes (with account in URL) |
| L1 | Guided key capture + validate | Missing | Missing | Missing |
| L2 | Auto model list in picker | Missing | Missing | Missing |
| L3 | Multi-key + logout + status | Missing | Missing | Missing |
| L4 | Per-model backend overrides + feature matrix | Missing | Missing | Missing |
| L5 | Feature parity with xAI/Codex agent tools | Unrealistic as hard goal | Partial | Partial |

Product request maps to **L1–L2 minimum**, **L3 desirable**, **L4–L5 phased**.

---

## 7. External doc references (for planners)

- OpenRouter quickstart / auth / Responses beta — openrouter.ai docs  
- Groq OpenAI compatibility + Responses API — console.groq.com docs  
- Cloudflare Workers AI OpenAI compatibility — developers.cloudflare.com/workers-ai  

Re-verify base paths during implementation; do not freeze unverified edge URLs into production without a live smoke.
