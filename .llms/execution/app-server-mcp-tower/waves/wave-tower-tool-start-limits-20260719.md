# Wave — Tower start input limits (2026-07-19)

## Scope

Align `tower_agent_start` string bounds with the canonical Tower tools schema.

## Changes

- `workspaceRoot` now requires 1..=4096 characters;
- `agentType` now requires 1..=128 characters;
- validation occurs before facade/runtime dispatch;
- explicit idempotency validation from the previous wave remains enforced.

## Evidence

```text
cargo test -p xai-grok-tower-tools --no-fail-fast
20 unit tests passed
24 integration tests passed
git diff --check passed
```

## Remaining gap

Optional start fields (`model`, `providerBinding`, `sandboxMode`) are not yet
propagated into canonical session metadata or product actor configuration.
