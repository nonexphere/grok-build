# Handoff C2-A — Tower instance config & isolation (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Wave | C2 items 15–18 (bounded) |
| Branch | `goblin-implement-epic-tree` |

## Goal

- Prefer canonical env `GROK_OSS_TOWER` over `GROK_TOWER_INSTANCE` (accept both with explicit precedence: explicit arg > GROK_OSS_TOWER > GROK_TOWER_INSTANCE legacy > default)
- Parse/validate via `TowerInstanceId`
- Hermetic tests: explicit > env > default without ambient pollution
- Prove two instances isolate directories/registries where feasible

## Read first

- Corrective contract § Wave C2
- `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs`
- Tower instance types in app-server/tower crates
- `_shared/tower-instance-lifecycle.md` if present

## Owned files

- `app_server_composition.rs` and related pager-bin tests only as needed
- Tower instance helpers if already in app-server/tower
- Ledger `waves/c2-tower-instance.md`, `tests/c2/*`

## Must NOT edit

- shell `app_server_runtime/**` (C1-J)
- mcp-server (C4-E)
- multi-auth

## Acceptance

1. Precedence tests hermetic GREEN
2. Dual-instance isolation test if implementable without dual-OS-process flock (document PARTIAL if flock needs more infra)
3. Wave note + evidence under tests/c2 and SCRATCH optional

## Report back

Files, RED/GREEN, REAL vs PARTIAL.
