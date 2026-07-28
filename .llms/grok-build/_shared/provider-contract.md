# Provider descriptor and onboarding contract

[provenance: existing multi-auth implementation/docs, BYOK onboarding docs, review D-PR.1..7]

This is P2 context and does not make providers a dependency of the App Server
processor. Normative discovery inputs are
`docs/architecture/byok-providers-onboarding/{README,API_BACKENDS,PROVIDER_MATRIX,EVIDENCE_MAP,CURRENT_STATE,GAPS_AND_QUESTIONS,ARCHITECTURE_TENSIONS}.md`.

## Descriptor schema

```json
{
  "providerId": "openrouter",
  "displayName": "OpenRouter",
  "credentialKind": "api_key",
  "apiBackend": "chat_completions",
  "baseUrl": "https://openrouter.ai/api/v1",
  "authHeader": "Authorization",
  "authScheme": "Bearer",
  "catalogStrategy": "static_or_remote",
  "credentialEnvImport": "OPENROUTER_API_KEY"
}
```

Required fields are providerId, credentialKind, backend, URL/auth metadata and
catalog strategy. Descriptor never contains a credential value. Provider IDs
are stable lowercase identifiers. Credential-specific runtime catalog keys are
`provider/{credential_uuid}/{wire_slug}`.

## Immutable binding

A request binding fixes provider ID, Credential ID, model wire slug, backend,
base URL and safe headers at Turn creation. Alias/display changes cannot retarget
an in-flight Turn. Request-time resolution obtains the secret from the existing
multi-auth store; no App Server/MCP payload carries it.

## CLI onboarding flows

| Provider | Steps | Offline acceptance fixture |
|---|---|---|
| OpenRouter | `grok-oss auth login openrouter` → masked key prompt → validate shape → store → catalog refresh → select model | login/store/catalog/request/logout with OpenAI-compatible fixture |
| Groq | `grok-oss auth login groq` → masked key prompt → store → catalog refresh → select model | Groq base URL/auth/catalog fixture; static 401 requests reauth |
| Cloudflare | `grok-oss auth login cloudflare` → account ID + masked token → compose account-scoped URL → store → catalog | account/token URL fixture and messages mapped to supported backend |

No live credential is required for contract tests. Fixtures must preserve URL,
method, headers, status, JSON/SSE body and complete schema. Live tests are
opt-in and a skipped live test is BLOCKED/SKIPPED, never PASS.

## Hygiene mapping

`10/v1-01` reconciles persistent issues `docs-001`, `docs-002`,
`operations-001`, `testing-001`, `testing-002`, `data-001..003` and
`ui-model-identity-system-prompt-label-sticky`. Each issue is closed only with
code/test/doc evidence or remains explicitly open; App Server work does not
silently claim multi-provider 1.0 complete.
