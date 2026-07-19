# Handoff C4-E — MCP Streamable HTTP review fixes (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Branch | `goblin-implement-epic-tree` |

## Goal

Triage C4-C/C4-D Medium findings:

1. **F-2 empty bearer footgun:** `McpHttpConfig::default()` must not silently accept empty token when `require_auth: true`. Fail closed (reject all or refuse to bind without token).
2. **Fingerprint mismatch test:** add a real test that wrong bearer fingerprint rejects (not just missing session header).
3. Capture a RED log artifact under `tests/c4/` if re-proving a behavior change (or document why RED is structural from C4-A empty pre-state).

## Owned files

- `xai-grok-mcp-server/**` only
- `tests/c4/*` ledger

## Must NOT edit

- shell, multi-auth, app-server transport, pager-bin

## Acceptance

1. Default config is fail-closed for auth
2. Fingerprint mismatch test GREEN
3. Full streamable-http suite still green
4. Wave note update / CHANGES

## Report back

Files, RED/GREEN, residual.
