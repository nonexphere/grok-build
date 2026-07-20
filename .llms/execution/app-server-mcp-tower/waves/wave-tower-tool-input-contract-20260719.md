# Wave — Tower tool input contract (2026-07-19)

## Scope

Close the concrete semantic gap in `tower_agent_send` identified by the
completion audit: the canonical schema accepts structured input blocks, while
the adapter previously projected only the first text block and allowed malformed
input to reach runtime lookup.

## Changes

- `crates/codegen/xai-grok-tower-tools/src/lib.rs`
  - Added pre-dispatch validation for required `agentType`, `sessionId`, mode,
    and idempotency keys on the affected operations.
  - Added canonical `InputBlock` deserialization for all send blocks.
  - Enforced 1..=128 blocks, non-empty per-block text, 1 MiB per-block and
    aggregate text limits.
  - Preserved every `text`, `mention`, and `skill` block through the existing
    Tower facade instead of collapsing the request to the first block.

## RED/GREEN evidence

RED was observed before the fix: malformed send input reached the facade and
returned `session_not_found`; valid multi-block input was silently reduced to a
single block by the old `first()` projection.

GREEN:

```text
cargo test -p xai-grok-tower-tools invoke_tests --no-fail-fast
3 passed; 0 failed
```

The new tests are:

- `send_preserves_all_structured_input_blocks`
- `send_rejects_empty_or_oversized_input_before_runtime_lookup`

`git diff --check` also passed.

## Remaining boundary

This closes only the shared input-contract layer. It does not prove a product
resident actor, true steer semantics, interaction delivery, or complete output
schema validation. Those remain owned by the canonical actor/product-runtime
epics and capabilities remain fail-closed until those gates are green.
