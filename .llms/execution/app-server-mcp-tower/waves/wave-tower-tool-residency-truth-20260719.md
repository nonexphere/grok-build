# Wave — Tower tool residency truth (2026-07-19)

## Scope

Remove the false `resident` claim from `tower_agent_list` and
`tower_agent_status` when the current facade exposes only a durable session
row and no active actor turn.

## Changes

- Archived sessions project as `archived`.
- Sessions with an observable `activeTurnId` project as `resident`.
- Other non-archived sessions project as `dormant`.
- Added regression coverage for list and status outputs.

This is intentionally conservative: a row's existence does not prove an
actor/thread is resident.

## Evidence

```text
cargo test -p xai-grok-tower-tools --no-fail-fast
16 unit tests passed
24 integration tests passed
git diff --check passed
```

## Remaining gap

The facade still lacks canonical `agentType` and an explicit residency
registry/state, so this wave cannot prove live actor residency. Product actor
composition remains owned by the canonical session runtime epics.
