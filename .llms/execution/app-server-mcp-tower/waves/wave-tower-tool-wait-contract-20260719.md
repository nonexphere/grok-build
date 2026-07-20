# Wave — Tower tool wait contract (2026-07-19)

## Scope

Harden `tower_agent_wait` input handling without claiming that the current
facade already provides a blocking subscription.

## Changes

- require `sessionId`, `afterEventSeq`, and `timeoutMs`;
- validate decimal non-negative event cursors;
- validate timeout range `1..=300000` ms;
- pass the parsed cursor to `SubscribeParams` instead of silently converting
  malformed input to zero;
- preserve canonical history epoch resolution and replay output.

## Evidence

```text
cargo test -p xai-grok-tower-tools --no-fail-fast
17 unit tests passed
24 integration tests passed
git diff --check passed
```

New regression coverage:

- `wait_rejects_malformed_cursor_and_timeout_before_runtime_lookup`

## Remaining gap

The current facade's `replay` operation is immediate. A true cancellation-safe
blocking wait with live event notification, terminal/interaction wake reasons,
and bounded timeout requires the canonical product event subscription owned by
the App Server/Tower runtime epics.
