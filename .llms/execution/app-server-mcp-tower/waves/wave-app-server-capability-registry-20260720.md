# Wave: App Server capability registry — 2026-07-20

## Change

`FacadeProcessor` now contains a typed mapping from every callable App Server
method to its `RuntimeCapabilities` bit. Before dispatch, a known method whose
capability is false returns the canonical `runtime_unavailable` error and does
not invoke the facade. Unknown method names continue through the canonical
`method_not_found` path.

This makes `initialize` capability output an enforcement boundary rather than
metadata only.

## Evidence

```text
cargo test -p xai-grok-app-server capability_registry -- --nocapture
1 passed

cargo test -p xai-grok-app-server
40 passed; 0 failed
```

## Remaining scope

AS109-01 remains partial until product composition tests prove the exact
initialize-to-dispatch matrix for every runtime variant and the protocol
meaning of item lifecycle/delta capabilities is resolved (they are currently
event capabilities rather than callable methods).
