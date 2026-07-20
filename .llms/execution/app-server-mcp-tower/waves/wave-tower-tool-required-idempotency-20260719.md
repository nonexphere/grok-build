# Wave — Tower required idempotency contract (2026-07-19)

## Scope

Remove the final implicit idempotency fallbacks from the shared Tower tool
semantic core.

## Changes

All mutating operations now require an explicit `idempotencyKey` and validate
the schema range `8..=128` characters. This applies to start, send, resume,
interrupt, and archive. Internal fallback labels are no longer accepted as
substitutes for a missing client key.

## Evidence

```text
cargo test -p xai-grok-tower-tools --no-fail-fast
18 unit tests passed
24 integration tests passed
cargo test -p xai-grok-mcp-server --features streamable-http --no-fail-fast
exit 0; unit and Streamable HTTP targets passed
git diff --check passed
```

## Remaining gap

This validates request-level idempotency inputs. Durable cross-process replay,
operation identity on failures, and App Server error-catalog convergence still
belong to the product-runtime/release epics.
