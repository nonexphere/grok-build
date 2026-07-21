# Wave: Tower interrupt targeting — 2026-07-20

## Existing product evidence

```text
cargo test -p xai-grok-shell c1_turn_interrupt_turn -- --nocapture
2 passed; 0 failed
```

The Shell-backed runtime checks that the requested `turnId` equals the
currently running actor turn before sending cancellation. A mismatched or
stale turn returns `turn_not_found`; a matching running turn is cancelled and
the start future resolves after cancellation.

The implementation also clears the stale current-turn slot when the actor
mailbox is closed, preserving the targeting invariant.

## Remaining scope

TA103-07 remains partial. The current gate does not yet prove an explicit
complete-versus-interrupt race, repeated idempotent interrupt convergence, or
the same exact-target behavior through every Tower/MCP transport.
