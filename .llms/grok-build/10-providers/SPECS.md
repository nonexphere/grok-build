# Providers — SPECS

## 1. Identidade e binding

Credential é `(ProviderId, CredentialId)`; catalog key é
`provider/{credential_uuid}/{wire_slug}`; binding imutável inclui provider,
credential, model, backend, URL e headers. Alias não é runtime key.

## 2. Credenciais

Codex usa OAuth/refresh. BYOK usa API key/token estático no multi-auth store,
nunca em `ModelEntry.api_key`, TOML, argv ou status. Request resolve secret no
momento do envio; 401 estático pede reauth e não inventa refresh.

## 3. Backends

- Codex: `responses`.
- OpenRouter/Groq: `chat_completions` default; per-model override futuro.
- Cloudflare v1: `chat_completions`.

## 4. Catálogo

Provider list/fallback filtra modelos úteis ao agent e mantém metadata de
capability sem alegar parity. Custom `[model.*]` permanece compatível.

## 5. Validação

Seguir [TDD](../TDD.md) e `.agents/skills/add-provider` para cada vertical slice.

