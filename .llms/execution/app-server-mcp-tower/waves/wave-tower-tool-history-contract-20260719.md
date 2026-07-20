# Wave — Tower tool history contract (2026-07-19)

## Scope

Close the portion of `tower_agent_history` that could be implemented against
the current canonical `SessionReadResult`, while keeping event-cursor claims
fail-closed.

## Changes

In `crates/codegen/xai-grok-tower-tools/src/lib.rs`:

- validate required `mode` (`full`/`last`) and `maxBytes` (`1..=1048576`);
- validate optional `historyEpoch` against the session's canonical epoch;
- apply `lastItems` (`1..=100`) for `last` mode;
- enforce serialized output byte limits after the redacted projection;
- return deterministic `truncated` state;
- reject non-zero `afterEventSeq` explicitly because current item projection
  has no authoritative item-to-event sequence mapping.

## Evidence

```text
cargo test -p xai-grok-tower-tools --no-fail-fast
15 unit tests passed
24 integration tests passed
git diff --check passed
```

The existing history path and ACL tests remained green after the change.

## Remaining gap

`afterEventSeq` and a non-zero `nextEventSeq` require the canonical history
projection owned by the App Server/Tower product-runtime epics. This wave does
not synthesize those values or label the operation complete.
