# Wave — Tower status input contract (2026-07-19)

## Scope

Remove the final empty-string fallback from `tower_agent_status`.

## Change

Missing or empty `sessionId` now returns `invalid_params` before facade lookup;
valid status requests retain the conservative archived/resident/dormant
projection introduced in the previous residency wave.

## Evidence

```text
cargo test -p xai-grok-tower-tools --no-fail-fast
19 unit tests passed
24 integration tests passed
git diff --check passed
```
