# Wave: product capability truth — 2026-07-20

## Evidence

The real pager-bin composition test initializes the Shell-backed runtime,
observes its fail-closed capability set, and invokes the disabled turn methods
with empty parameters. Every method is rejected at the capability boundary
before parameter validation or runtime effect:

```text
cargo test -p xai-grok-pager-bin \
  product_rejects_unadvertised_methods_before_runtime_validation -- --nocapture
1 passed per binary target; 0 failed
```

Assertions cover `turn/start`, `turn/steer`, `turn/interrupt`, canonical
`runtime_unavailable`, and explicit `operationId: null`.

## Remaining scope

The registry is now product-backed for the tested composition. AS109-01 remains
partial until item lifecycle/delta capability meaning and every runtime variant
are represented in the same executable matrix.
