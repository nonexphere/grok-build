# Handoff C5-A — BYOK provider surface map (GLM explore, read-only)

| Field | Value |
|---|---|
| Agent role | **repo-explore** (read-only) |
| Model | `glm-5.2` |
| Wave | C5 prep (items 31–37 inputs) |
| Capability | **read-only** — no product code edits |
| Parallel with | C1-G, C3-A, C4-A |
| Branch | `goblin-implement-epic-tree` |

## Goal

Map AuthProvider registration, credential store, and gaps for OpenRouter / Groq / Cloudflare verticals so C5 implementer can write offline contract RED tests without live secrets.

## Read first

- Corrective contract § Wave C5
- Repo skill `.agents/skills/add-provider/SKILL.md` + checklist
- Search for `AuthProvider`, `ProviderBinding`, API-key login, credential store under `crates/codegen`
- Canonical protocol `ProviderBinding` usage
- Existing provider stubs (OpenRouter/Groq/Cloudflare if any)

## Deliverable

`.llms/execution/app-server-mcp-tower-corrective/waves/c5-provider-surface-map.md`

Must include:

1. **Registration table**: provider id → type location → registered? → API-key capability?
2. **Login path**: public path that must reject unknown/unregistered providers.
3. **Credential store backend policy** (`file:fn`).
4. **Protocol binding**: where `ProviderBinding` is defined vs any duplicate type.
5. **Offline fixture strategy** for HTTP boundary tests (schema-faithful mocks).
6. **Live smoke**: explicit SKIP policy when creds missing (never PASS).
7. **Files for C5-B implementer** + dependency on C1 turns (if turn binding required).

## Must NOT

- Edit product code
- Use or invent live credentials
- Mark provider tasks PASS

## Report back

Path to map + executive summary + whether C5-B can start in parallel with C3/C4 after maps land.
