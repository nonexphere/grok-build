# Wave — runtime capability truth

## Scope

- Epic: `30-app-server/v1-09-capability-contract-product-conformance`
- Task: `AS109-01` (partial; product actor factory remains open)
- Findings addressed: F-10 and part of F-01/F-08

## Change

`GrokRuntimeFacade` now exposes a concrete `RuntimeCapabilities` projection.
The App Server initialize response derives its advertised methods from that
projection. `FakeRuntime` keeps the complete capability set for protocol
conformance tests; `ShellSessionActorRuntime` advertises storage/replay
capabilities but explicitly disables turn mutation, interaction response and
item lifecycle/delta until the real SessionActor factory is wired.

This makes the current product limitation visible to clients rather than
announcing methods that deterministically return `unsupported`.

## Validation

- `cargo test -p xai-grok-app-server --no-fail-fast` — 39 passed.
- `cargo test -p xai-grok-shell --lib app_server_runtime --no-fail-fast` — 7 passed.
- `cargo test -p xai-grok-tower --no-fail-fast` — 39 passed.
- `cargo test -p xai-grok-pager-bin product_initialize_does_not_advertise_unwired_turn_methods --no-fail-fast` — 3 binary targets passed.
- `git diff --check` — PASS.

The first product-root attempt exposed that `ShellRuntimeAdapter` fell back to
the trait's all-capabilities default instead of forwarding the inner runtime's
projection. The adapter now delegates `capabilities()`, and the product-root
test passes for all three binary targets.

## Remaining acceptance

`AS109-01` remains partial until product composition supplies the real actor
factory and the initialize capability matrix is proven against a product
start/send/replay smoke.
