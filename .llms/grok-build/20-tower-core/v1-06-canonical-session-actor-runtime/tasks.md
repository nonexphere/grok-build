# Tasks — canonical SessionActor product runtime

## 2026-07-19 refinement checkpoint

The capability-truth wave is complete: the product composition now advertises
only storage-backed session/replay operations, while turn, interaction, and
item mutation capabilities remain disabled until a real factory is injected.
The following existing tasks are therefore not considered satisfied by the
passing conformance suites:

- `TW106-02` and `TW106-03` still require the shell-owned ACP/actor host;
  `experimental_local_turn_spawn` is explicitly test-only and cannot satisfy
  them.
- `TW106-04` is only partially satisfied: App Server and MCP share the same
  storage-backed facade, but a provider-backed actor vertical is not yet
  proven.
- `TW106-05` is partially satisfied by fail-closed capability advertisement;
  readiness must additionally reject an incomplete actor factory when turn
  capabilities are requested.
- `TW106-06` through `TW106-09` remain pending until the real host exists.

The next implementation boundary is intentionally narrow: a shell-owned
`ProductSessionHost` running the existing `MvpAgent`/ACP connection on a
dedicated `LocalSet`, exposing a `Send` command bridge and durable notification
sink. It must be tested first with the repository mock inference boundary, then
wired into `RealSpawnFn`. No echo or fake runtime may be promoted to this path.

- [ ] TW106-01 [F-01] Add a failing product-composition test in crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs; run ./scripts/run-rust-test-gate.sh product_runtime_builds_canonical_actor cargo test -p xai-grok-pager-bin product_runtime_builds_canonical_actor; accept RED because current ProductionSpawner has no real spawn.
- [ ] TW106-02 [F-01] Define ProductSessionDependencies and RealSpawnFn assembly in xai-grok-shell/src/app_server_runtime/ with AuthManager, AgentDefinition, ToolContext, GatewaySender, ModelsManager, PersistenceHandle, McpServers, WorkspaceOps, PluginRegistry and SamplingConfig; run cargo check -p xai-grok-shell; accept no optional placeholder dependency in normal mode.
- [ ] TW106-03 [F-01] Wire spawn_session_on_thread and dedicated thread/LocalSet in the Shell adapter; run ./scripts/run-rust-test-gate.sh canonical_actor cargo test -p xai-grok-shell canonical_actor; accept exactly one live command channel and deterministic cleanup.
- [ ] TW106-04 [F-01,F-08] Construct one shared runtime in xai-grok-pager-bin and inject it into App Server, MCP and tools; run ./scripts/run-rust-test-gate.sh shared_product_runtime cargo test -p xai-grok-pager-bin shared_product_runtime; accept pointer/registry identity and no FakeRuntime/echo product edge.
- [ ] TW106-05 [F-01,F-10] Implement readiness/capability fail-fast in supervisor composition; run ./scripts/run-rust-test-gate.sh readiness_requires_actor_factory cargo test -p xai-grok-pager-bin readiness_requires_actor_factory; accept no ready state or executable capability when factory assembly fails.
- [ ] TW106-06 [F-01] Add provider/gateway boundary fixture that runs the real actor without external credentials; run ./scripts/run-rust-test-gate.sh provider_boundary_real_actor cargo test -p xai-grok-shell provider_boundary_real_actor; accept real queue/tools/persistence with only network sampling substituted.
- [ ] TW106-07 [F-01,F-08] Add binary black-box start→send→wait→history→interrupt/archive under scripts/smoke/ and pager-bin tests; run PROFILE=debug ./scripts/install-grok-oss.sh then the named smoke; accept non-empty Turn/Items and terminal states.
- [ ] TW106-08 [F-01] Test spawn failure rollback and idempotency claim durability in xai-grok-shell tests; run ./scripts/run-rust-test-gate.sh spawn_failure_rolls_back cargo test -p xai-grok-shell spawn_failure_rolls_back; accept no false resident token, ready state or winning claim.
- [ ] TW106-09 [F-01] Run concurrency/load cancellation tests for start/resume and actor thread cleanup; run ./scripts/run-rust-test-gate.sh one_product_actor cargo test -p xai-grok-shell -p xai-grok-tower one_product_actor; accept one actor and bounded tasks/threads.
- [ ] TW106-10 [TD] Record RED/GREEN and @human-product-test evidence in this epic; accept fake-only gates labeled conformance and product black-box labeled integration.
