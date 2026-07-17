# Current State — What Already Works

Evidence date: **2026-07-17**, branch context multi-provider Codex.

## 1. Summary matrix

| Capability | Status | Where |
|------------|--------|--------|
| Inference to arbitrary OpenAI-compatible base URL | **Works** (manual config) | `[model.*]` + sampler |
| Per-model `api_backend` | **Works** | `ApiBackend` enum + `SamplingConfig` / `SamplerConfig` |
| Per-model `api_key` / `env_key` | **Works** | model resolve priority |
| Auto-fetch models from `{base}/models` | **Works** when `GROK_MODELS_BASE_URL` / `[endpoints].models_base_url` set | shell models catalog |
| Named third-party **provider plugins** for Groq/OR/CF | **No** | only generic custom models |
| `goblin login --provider groq\|openrouter\|cloudflare` | **No** | login only xAI + Codex |
| Multi-auth store API-key credentials for those providers | **No product path** | store exists; API key login rejected in coordinator |
| Credential-scoped catalog keys for BYOK multi-key | **Codex only pattern** | `provider_model_key` |
| Chat Completions / Responses / Messages in one process | **Works** | sampler dispatches by `api_backend` |
| Codex OAuth multi-account | **Partial product** | multi-auth crate + merge into catalog |

## 2. World A — Custom models (primary path for Groq/OR/CF today)

### 2.1 Config surface

Documented in `crates/codegen/xai-grok-shell/README.md` (“Custom Models”):

```toml
[model.my-model]
model = "model-id"
base_url = "https://api.example.com/v1"
name = "Display Name"
api_key = "sk-..."                 # optional inline (discouraged)
env_key = "OPENAI_API_KEY"         # string or array of env names
api_backend = "chat_completions"   # default if omitted = chat_completions
context_window = 256000
extra_headers = { ... }            # optional
```

Credential resolution order for a model:

1. `api_key` on the model entry  
2. `env_key` (first set non-empty env var)  
3. global `XAI_API_KEY` / legacy env  

**Implication:** third-party keys often accidentally fall through to `XAI_API_KEY` if misconfigured — wrong host + wrong key.

### 2.2 Enterprise-style bulk endpoint

```bash
export GROK_MODELS_BASE_URL="https://api.acme.com/v1"
export XAI_API_KEY="..."
```

- Fetches `GET {base}/models`
- Uses **API key auth** for inference
- README explicitly lists **OpenRouter, Groq, Together.ai** as examples of the OpenAI-compatible convention

**Limitation:** one global base URL + one global key name (`XAI_API_KEY`), not multi-provider simultaneous BYOK with distinct keys.

### 2.3 Catalog identity for custom models

- Catalog key = TOML header name (`my-model`) or built-in id.
- Wire model slug = `model` field (or header name if omitted).
- **No** automatic `provider/credential_id/slug` scoping for BYOK.

Two OpenRouter keys cannot be first-class dual catalog entries without the user inventing two `[model.*]` blocks.

## 3. World B — Multi-provider auth (Codex path)

### 3.1 Components

| Component | Crate / path |
|-----------|----------------|
| Types (`ProviderId`, `CredentialId`, `ModelBinding`, store trait) | `xai-grok-auth` |
| Implementations (Codex, file store, TokenManager, CLI helpers) | `xai-grok-multi-auth` |
| Catalog merge, bearer resolver, session pin | `xai-grok-shell` (feature `native-multi-provider-auth`) |
| CLI login/status/logout | `xai-grok-pager-bin` |

### 3.2 What Codex teaches (reusable pattern)

1. **Login** → persist credential in multi-auth store (`~/.grok/auth/…`).  
2. **List models** with that credential.  
3. **Merge** into catalog as `codex/{credential_uuid}/{slug}` with `api_key: None`.  
4. **Request-time** resolve bearer (and headers like `ChatGPT-Account-ID`).  
5. **Immutable** `ModelBinding` for the request lifetime.  
6. **Session pin** so catalog reloads don’t flip accounts mid-session.

### 3.3 What Codex does *not* teach

- API-key paste login UX (capability bit exists; coordinator rejects `LoginTransport::ApiKey`).
- Providers that default to **Chat Completions**.
- Account ID embedded in **URL path** (Cloudflare).
- Optional marketing headers (OpenRouter `HTTP-Referer` / `X-Title`).
- Models that only partially support tools/streaming/reasoning under OpenAI schemas.

### 3.4 Registered AuthProviders today

```text
xai-grok-multi-auth/src/providers/
  codex/
  xai.rs          # legacy boundary / empty multi-provider caps
```

No `groq`, `openrouter`, or `cloudflare` modules.

## 4. Sampler / inference reality

`xai-grok-sampler` client is already multi-backend:

```text
ApiBackend::ChatCompletions → /chat/completions path + stream_chat_completions
ApiBackend::Responses       → /responses path + stream_responses
ApiBackend::Messages        → Anthropic /messages path + stream_messages
```

Auth schemes on `SamplerConfig`:

- `AuthScheme::Bearer` (default) — `Authorization: Bearer …`
- `AuthScheme::XApiKey` — Anthropic-style `x-api-key`

Extra headers are first-class (`extra_headers`).

**So:** protocol support for “OpenAI-compatible chat” and “OpenAI-compatible responses” already exists. The gap is **product onboarding + correct default backend + credential lifecycle**, not inventing HTTP clients.

## 5. Mentions of target providers in code/docs

| Provider | Evidence of prior contact |
|----------|---------------------------|
| OpenRouter | README lists as OpenAI-compatible; CHANGELOG / tests for kimi malformed tool args via OpenRouter; optional `x-openrouter-api-key` header notes in session code |
| Groq | README lists as OpenAI-compatible; audit docs warn not to use Groq `cached_tokens` as proof of **Codex** prompt cache |
| Cloudflare | Mostly CDN/proxy body-size notes; **not** a first-class Workers AI provider integration |

Conclusion: **compatibility is assumed via OpenAI-shaped endpoints**, not dedicated provider modules.

## 6. Login CLI surface today

| Command | Behavior |
|---------|----------|
| `goblin login` / `--provider xai\|grok` | Legacy AuthManager OIDC |
| `goblin login --provider codex` | Native multi-auth Codex (gated by OAuth approval env) |
| `goblin auth status` | Multi-auth status JSON (no secrets) |
| `goblin logout` | Clears multi-auth store + legacy xAI |
| `goblin login --provider openrouter` | **Unknown provider** (parse rejects) |

There is **no** interactive “paste API key” product path for third parties.

## 7. Default `api_backend`

```rust
// xai-grok-sampling-types
pub enum ApiBackend {
    #[default]
    ChatCompletions,  // /v1/chat/completions
    Responses,        // /v1/responses
    Messages,         // /v1/messages
}
```

- If user omits `api_backend` in TOML → **Chat Completions**.
- Built-in xAI agent models often use **Responses**.
- Codex merge forces **Responses**.
- Web search tooling **requires Responses** (documented in README).

This default is friendly to Groq/OpenRouter classic usage and **hostile** if someone expects Responses-shaped agent features without setting the field.

## 8. Feature flags / kill switches relevant later

- `native-multi-provider-auth` (shell feature) — gates Codex merge and related paths.
- `GROK_DISABLE_CODEX_AUTH`, `GROK_CODEX_OAUTH_APPROVED` — Codex-specific.
- `GROK_DISABLE_API_KEY_AUTH` — admin kill for xAI API key method (enterprise).

A BYOK onboarding design must decide whether new providers share kill switches or get per-provider flags.

## 9. Honest “we already support them”

**True if:** user can make inference work with manual TOML + env.

**False if:** user expects app-guided multi-provider setup like Codex login + model catalog without config editing.

The gap is **productization of World A** (and/or bridging into World B), not raw HTTP reachability.
