# Handoff C5-B — Register OpenRouter / Groq / Cloudflare AuthProviders (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Wave | C5 offline slice (items 31–34, partial 36) |
| Capability | read-write under owned paths only |
| Depends on | C5-A surface map complete |
| Parallel with | C1-G (non-overlapping files) |
| Branch | `goblin-implement-epic-tree` |

## Goal

Make OpenRouter, Groq, and Cloudflare real registered BYOK `AuthProvider` verticals with offline contract tests. Fix the login path that currently ignores the registry for API-key login. No live credentials.

## Read first

- Handoff map: `.llms/execution/app-server-mcp-tower-corrective/waves/c5-provider-surface-map.md`
- Skill: `.agents/skills/add-provider/SKILL.md` + checklist
- Corrective contract § Wave C5 items 31–36
- Existing: `providers/byok/mod.rs`, `registry.rs`, `login_coordinator.rs`, `cli.rs`

## Non-negotiables

- No live secrets; offline fixtures only
- Reuse protocol `ProviderBinding`; do not invent a second public binding type for wire
- `run_api_key_login` must honor registry + API-key capability (reject unregistered / non-API-key)
- Never mark live smoke PASS when creds missing
- Do **not** edit `xai-grok-shell/src/app_server_runtime/**` (C1-G exclusive)

## Owned files

- Multi-provider / auth crates under providers, registry, login_coordinator, CLI parse for BYOK ids
- New offline tests `tests/byok_*.rs` or package-local tests
- Ledger: `waves/c5-byok-providers.md`, `tests/c5/*`, STATUS/CHANGES updates only if no conflict

## Must NOT edit

- `app_server_runtime/**`, SessionActor turn path (C1-G)
- `app-server/transport/websocket.rs` (C3)
- MCP server HTTP (C4)

## Acceptance (DONE this slice)

1. OpenRouter, Groq, Cloudflare registered in default registry with API_KEY_LOGIN capability.
2. Unknown/unregistered provider rejected on public login path (RED→GREEN test).
3. Provider without API-key capability rejected for API-key login.
4. Offline contract tests for credential selection / request auth shape (schema-faithful fixtures).
5. Wave note + test evidence under corrective ledger `tests/c5/`.
6. Honest PARTIAL if composition-root Turn binding still None until C1-G/C5 follow-on.

## Report back

Files, RED/GREEN, REAL vs PARTIAL, risks.
