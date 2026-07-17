# Problem Statement — Automated BYOK Provider Onboarding

## 1. User-visible problem

Today a power user can already point Goblin/Grok at OpenRouter, Groq, Cloudflare, Together, Ollama, etc. by **manually** writing something like:

```toml
[model.my-or-model]
model = "openai/gpt-4o"
base_url = "https://openrouter.ai/api/v1"
env_key = "OPENROUTER_API_KEY"
api_backend = "chat_completions"   # easy to get wrong
context_window = 128000
```

and exporting the env var.

That works for engineers who already know:

- the correct `base_url`;
- whether the model needs `chat_completions` or `responses`;
- which env var name to use;
- how model IDs are spelled on that provider;
- optional headers (OpenRouter referer/title; Cloudflare account id in the URL).

It fails the product goal for a normal user who wants:

```text
goblin login
  → pick OpenRouter
  → paste API key (or open console URL)
  → see available models
  → pick one
  → start chatting
```

**without** editing `config.toml`.

## 2. Desired outcome (product)

| ID | Outcome |
|----|---------|
| O1 | User can add **Cloudflare**, **Groq**, and **OpenRouter** credentials through CLI and/or TUI. |
| O2 | After credential is stored, **models appear in the catalog/picker** automatically. |
| O3 | Selecting a model routes inference with **correct base URL, auth header scheme, and `api_backend`**. |
| O4 | User does **not** need to hand-edit TOML for the happy path. |
| O5 | Existing xAI, Codex OAuth, and hand-written custom models keep working. |
| O6 | Pattern is extensible to future API-key providers (Together, Fireworks, DeepSeek, …). |

## 3. Non-goals (for the first problem scope)

Unless the planner deliberately expands scope:

- Full OAuth for OpenRouter/Cloudflare (API key is the primary auth model).
- Perfect parity with every provider-specific feature (vision, audio, embeddings-only models).
- Automatic payment/billing UI inside Goblin.
- Importing keys from arbitrary third-party CLIs as the *only* path (optional later).
- Replacing the multi-provider Codex OAuth work already in flight.
- Dynamic third-party provider plugins / shared library ABI.

## 4. Why this is hard in *this* repo (not just “prompt for key”)

There are **two parallel worlds** of “provider support” today:

### World A — Custom / BYOK model config (mature, manual)

- Unit of config: **`[model.<alias>]`** in `config.toml` or env `GROK_MODELS_BASE_URL` + `XAI_API_KEY`.
- Auth: static `api_key` / `env_key` / fallback `XAI_API_KEY`.
- Protocol: per-model `api_backend` ∈ {`chat_completions`, `responses`, `messages`}.
- Catalog: user-defined keys, or OpenAI-compatible `GET {base}/models`.
- **No** first-class multi-account control plane, no login wizard for third parties.

### World B — Native multi-provider auth control plane (Codex-first, new)

- Unit of identity: **`(ProviderId, CredentialId)`** + immutable `ModelBinding`.
- Auth: login flows, refresh, TokenManager, request-time bearer.
- Catalog keys: `provider/{credential_uuid}/{slug}` for multi-account safety.
- First real external provider: **Codex ChatGPT OAuth** (`responses` + special headers).
- Capability bit **`API_KEY_LOGIN`** exists on `ProviderCapabilities` but is **not** product-complete for Groq/OR/CF.

The product request sits **between** A and B:

> “Make BYOK feel like native providers (World B UX) without breaking World A inference machinery.”

A bad plan either:

1. only documents better TOML examples (no product delta), or  
2. forces every API-key provider through full OAuth-shaped `AuthProvider` with no reuse of the working sampler path, or  
3. pastes API keys into `ModelEntry.api_key` in a way that blocks multi-key, rotation, and multi-account (the Codex audit already banned static OAuth tokens in `api_key`).

## 5. Success criteria (testable later)

A plan is good if it can define acceptance tests roughly like:

1. Clean home → run onboarding for OpenRouter with a test key → `goblin models` lists at least one OpenRouter model without any pre-existing `[model.*]` for it.
2. User selects that model in headless mode → one successful inference turn.
3. Wrong `api_backend` is not silently applied for a provider that only supports Chat Completions (or is auto-corrected with evidence).
4. Second key for the same provider can coexist without colliding catalog keys (if multi-key is in scope).
5. Logout / remove provider clears secrets and removes synthetic catalog entries.
6. Codex and xAI paths still pass existing tests.
7. Logs/status never contain the raw API key (or short prefixes of length 4/8/12/20).

Exact criteria belong in the planner’s plan; these are the **problem-level** gates.

## 6. Constraints from existing fork policy

From `GOBLIN.md` / `task.md` (must not be violated without explicit decision):

| ID | Constraint |
|----|------------|
| D3 | Provider/account selection immutable for in-flight request |
| D4 | Refresh/lock per credential, not process-global |
| D5 | Existing xAI credentials remain usable |
| D6 | Built-in providers register at compile time in v1 |
| — | No process-global “current provider” as sole auth |
| — | Secrets not in status JSON / telemetry |
| — | Prefer keyring later; file backend acceptable for headless |

API-key providers usually have **no refresh token**. That simplifies TokenManager but changes the meaning of “login” (import/store key) and “reauth” (replace key).

## 7. Stakeholder outcome statement

**As a user**, I want to connect Groq / OpenRouter / Cloudflare with a guided key flow and immediately pick models, so I can use Goblin as a multi-provider agent without learning TOML.

**As a maintainer**, I want one reusable pattern so each new API-key provider is mostly a descriptor (base URL, headers, model list, default backend, validation), not a cross-cutting rewrite of shell/sampler.
