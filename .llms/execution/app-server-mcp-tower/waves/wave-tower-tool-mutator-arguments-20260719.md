# Wave — Tower mutator argument contract (2026-07-19)

## Scope

Align `tower_agent_resume`, `tower_agent_interrupt`, and
`tower_agent_archive` with their schemas by removing silent empty/default
argument fallbacks.

## Changes

- `sessionId` is required and non-empty for all three operations;
- `turnId` is required and non-empty for interrupt;
- `idempotencyKey` is validated through the shared 8..=128-character helper;
- malformed requests fail before target lookup or runtime side effects.

## Evidence

```text
cargo test -p xai-grok-tower-tools --no-fail-fast
17 unit tests passed
24 integration tests passed
git diff --check passed
```

## Remaining gap

This closes input validation only. Error envelope parity (`retryable`,
`operationId`, public error catalog) across App Server, MCP HTTP, and stdio
remains owned by the cross-transport contract epics.
