# Wave: ACP host readiness and integrated validation — 2026-07-19

## Objective

Advance the product-runtime gap without promoting the hermetic echo fixture to
the production App Server/Tower path.

## Findings

- `xai-grok-shell` already owns the real ACP `MvpAgent` and the existing
  `AgentSideConnection` lifecycle.
- `ShellSessionActorRuntime` already exposes `RealSpawnFn`, but the default
  product composition intentionally injects no factory and therefore reports
  turn/interaction capabilities as unavailable.
- `experimental_local_turn_spawn` is an offline echo fixture and remains
  test-only. It is not evidence of a live provider-backed turn path.
- The real factory must own a dedicated Tokio `LocalSet`/thread, construct the
  ACP gateway and client notification bridge, assemble the agent/tool/model/
  auth context, retain the actor command handle, and map ACP notifications into
  durable App Server events. The current code does not provide that complete
  composition boundary.

## Validation

All commands passed:

```text
cargo test -p xai-grok-tower --no-fail-fast
cargo test -p xai-grok-app-server --no-fail-fast
cargo test -p xai-grok-mcp-server --features streamable-http --no-fail-fast
cargo test -p xai-grok-shell --lib app_server_runtime --no-fail-fast
cargo test -p xai-grok-pager-bin --bin goblin --bin grok-oss --bin xai-grok-pager \
  app_server_composition::tests::product_initialize_does_not_advertise_unwired_turn_methods \
  --no-fail-fast
git diff --check
```

Observed suites: Tower 29 unit + 10 integration; App Server 39; MCP 20 unit
+ 38 Streamable HTTP; Shell runtime 8 targeted; product capability smoke for
all three binaries; formatting diff check clean.

## Status

AS109-01 / C2-A remains **PARTIAL**. Capability truth is now proven and
fail-closed. The next implementation wave must add a real shell-owned ACP host
with a hermetic handshake test and then a provider-backed vertical test using
the repository's mock inference server. Until those exist, the product must
not advertise turn/item/interaction capabilities.

## Non-blocking warnings

Existing warnings remain: duplicate pager binary target declaration, unrelated
unused imports/dead helper functions, and App Server test-only warnings. They
did not fail this validation and are separate cleanup tasks.
