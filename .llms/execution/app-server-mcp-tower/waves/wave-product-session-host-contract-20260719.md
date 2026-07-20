# Wave: ProductSessionHost contract refinement — 2026-07-19

## Evidence inspected

- `agent/mvp_agent/acp_agent.rs`: real ACP `MvpAgent` already implements
  `initialize`, `new_session`, `prompt`, `cancel`, and session lifecycle.
- `agent/app.rs`: existing `spawn_agent_local` owns the canonical
  `MvpAgent` + `AgentSideConnection` + `GatewayReceiver` bootstrap.
- `session/acp_session_impl/spawn.rs`: direct actor construction is a private
  ~80-argument path and must not be reconstructed in Tower or Pager.
- `app_server_runtime/shell_session_actor_runtime.rs`: `RealSpawnFn` is the
  current facade seam; its `ResidentHandle` is the correct `Send` command
  projection, but the default product composition has no factory.

## Decision

The product host must reuse the ACP bootstrap and bridge ACP commands through a
dedicated current-thread Tokio runtime/`LocalSet`. Directly calling the private
actor spawn function from the composition root would violate Shell ownership and
duplicate agent/tool/persistence setup. Promoting
`experimental_local_turn_spawn` would be a false product claim because it is an
offline echo fixture.

The new epic `20-tower-core/v1-08-product-session-host` is the canonical task
tree. It defines the required dependency ownership, command/event boundary,
failure semantics, and proof gates.

## Current result

PSH-01 contract is documented. PSH-02 has a first green slice: the canonical
Shell `spawn_agent_local` bootstrap is now public and documented as the shared
ACP transport bootstrap, so a future host can reuse it without duplicating
`MvpAgent`/gateway construction. PSH-03 now has an initial typed command host
(`AcpHostHandle`) with a dedicated current-thread runtime, ACP client bridge,
fail-closed permissions, live notification subscription, and notification sink. Its consuming `shutdown(self)`
now owns and joins the dedicated thread after the command loop exits. It is not
yet product-wired: a real mock inference vertical test and resident-actor
adaptation are still required before it can back `RealSpawnFn`.

No capabilities were promoted and no fake/echo path was changed.

Validation for this slice:

```text
cargo check -p xai-grok-shell -p xai-grok-pager-bin
cargo test -p xai-grok-shell --lib app_server_runtime --no-fail-fast
cargo test -p xai-grok-shell --lib app_server_runtime::acp_host --no-fail-fast
git diff --check
```

All passed. Existing non-blocking warnings remain documented elsewhere. The
passing integration gate
`cargo test -p xai-grok-shell --test product_acp_host --no-fail-fast` now proves
real initialize → authenticate → session/new → prompt → ACP notification →
cancel → shutdown/join against `MockInferenceServer`. It does not yet prove
history projection or Tower `ResidentHandle`/`RealSpawnFn` integration.
The live subscription is exercised before the snapshot assertion, so the event
bridge is not polling-only.

The new `persist_notifications` consumer writes the live stream through the
canonical `JsonlStorageAdapter` and fails closed on broadcast lag. The vertical
test now verifies that at least one ACP notification reaches `updates.jsonl`.
`AcpHostHandle::start_persistence` owns that task and `shutdown(self)` awaits it,
so the caller cannot accidentally leave an orphan persistence task.

The experimental `experimental_acp_resident_spawn` factory now proves the next
boundary: it creates a real ACP resident with canonical session identity and
model metadata, routes `SessionCommand::Prompt` through ACP, maps the terminal
stop reason, and persists resulting notifications. Its targeted test passes
against `MockInferenceServer`. It remains experimental because steer,
interaction policy, rollback, concurrency/one-actor guarantees, and product
composition wiring are not complete.

The rollback path now has a fail-safe `Drop` on `AcpHostHandle`; bootstrap
errors send shutdown to a partially-created LocalSet. The invalid-session
integration test passes alongside the prompt/persistence vertical.
That vertical now queues two prompts before awaiting either response; both
complete through the single resident command loop, providing the first
one-actor serialization evidence.

Steer remains intentionally unpromoted: the current ACP command loop awaits a
prompt RPC, so an interject arriving concurrently would be processed only after
that RPC completes rather than inside the active turn. A true steer bridge
needs concurrent ACP request handling plus running-turn ownership; silently
mapping it to a queued prompt would be semantically wrong.

The transport boundary now has a cloneable `AcpCommandHandle` separate from the
thread/persistence owner. The experimental resident bridge uses it to dispatch
prompt/interject tasks concurrently and drains all tasks before shutdown. This
removes the command-loop serialization bottleneck, but does not yet prove ACP
interject semantics or Tower steer identity/turn ownership.

Regression gate passed: Shell app-server runtime 9 tests, App Server 39 tests,
Tower 29 unit + 10 integration tests, MCP 20 unit + 38 Streamable HTTP tests,
and `git diff --check`.

The three product-ACP integration tests also pass, including the resident
prompt/persistence, invalid-session rollback, and host lifecycle cases.

## Next executable slice

Extract a reusable Shell-owned ACP bootstrap from `spawn_agent_local`, then add
an in-process mock-inference test that proves initialization and a real session
notification through the new host before adapting `RealSpawnFn`.
