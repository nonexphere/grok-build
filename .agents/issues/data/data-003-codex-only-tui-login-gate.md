---
id: data-003
title: Codex-only TUI forced interactive xAI login
category: data
type: bug
status: DONE
severity: HIGH
owner: multi-provider auth and pager startup
evidence_quality: confirmed
discovered_by: "@code-audit"
discovered_during: upstream-regression-fix-2026-07-17
discovered_at: 2026-07-17
closed: 2026-07-17
---

# Codex-only TUI login gate

## Symptom (historical)

User with only Codex multi-provider credentials (no XAI_API_KEY, no xAI session)
started the TUI and was sent to interactive `grok.com` login because ACP
`auth_methods` advertised only `grok.com`.

## Product policy (closed)

Advertise non-interactive `goblin.multi_provider` when the catalog has multi-provider
bindings. Do **not** fake `xai.api_key` for OAuth Codex.

## Resolution

1. `should_advertise_multi_provider_auth` + `MULTI_PROVIDER_AUTH_METHOD_ID`
2. `build_auth_methods` places multi-provider after BYOK and before cached_token
3. Pager `startup_auth_metadata` sees non-interactive first method → `needs_login=false`
4. Composition tests: shell catalog→auth methods and shell→pager join

## Validation

- `codex_only_catalog_advertises_multi_provider_not_xai_api_key`
- `shell_built_auth_methods_for_codex_only_user_skip_login_screen`
- `startup_auth_multi_provider_no_login`
