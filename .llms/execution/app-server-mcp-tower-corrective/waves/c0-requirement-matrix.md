# C0 — Requirement matrix (App Server / MCP / Tower, programs 10–60)

**Author:** GLM `glm-5.2` (handoff C0-A)
**Branch:** `goblin-implement-epic-tree`
**Contract:** `.llms/tasks/20260718-correct-app-server-mcp-tower-execution.md` Wave C0 items 1–4
**Authority inputs:** `.llms/reviews/app-server-mcp-tower-adversarial-audit-2026-07-18.md` (F-01..F-13), `.llms/execution/app-server-mcp-tower/FINAL_REPORT.md`, `.llms/execution/app-server-mcp-tower-corrective/waves/c0-session-actor-command-map.md` (C0-B)

## Status enum

`OPEN` (production work not started) · `PARTIAL` (some code/tests exist but literal criterion not met) · `BLOCKED` (external/structural blocker) · `SKIP` (credential/external-dependent, never PASS) · `HUMAN` (explicit human/product gate) · `PASS` (literal criterion met: production code + non-vacuous test). **Only `PASS` may keep `[x]`** in `tasks.md`.

## Reconcile rules applied

- A fake-backed test against a production criterion (real SessionActor/leader, real transport, canonical session files, real permission path) is **not PASS** → `PARTIAL`/`OPEN`.
- `SKIP` (live provider smoke without credentials) is never `[x]`/PASS.
- `SessionStorageHybridRuntime` is rejected; grep confirms it is already absent from `crates/` (removed pre-C0).
- Composition root `app_server_composition.rs:15` still injects `FakeRuntime` (F-01) — the primary blocker; every facade→real-authority task depends on Wave C1-D.

## Summary counts

| Status | Count |
|---|---|
| PASS | 77 |
| PARTIAL | 19 |
| OPEN | 13 |
| SKIP | 3 |
| HUMAN | 8 |
| BLOCKED | 0 |
| **Total** | **120** |

Reopened this turn (PARTIAL, were `[x]`): **19** — PR102-01, TW101-04, TW101-05, TW102-03, TW103-02, TW103-03, TW103-06, RF102-07, AS103-07, AS105-06, AS106-05, AS106-06, AS107-01, AS107-02, AS107-04, AS107-06, MCP102-03, MCP102-05, TA101-06.

## Matrix

| task_id | epic | status | evidence | gap | next |
|---|---|---|---|---|---|
| PR101-01 | 10/v1-01 | PASS | `10-providers/v1-01-codex-readiness-hygiene/REQUIREMENT_MATRIX.md` exists | none | — |
| PR101-02 | 10/v1-01 | PASS | offline gate `cargo test -p xai-grok-auth -p xai-grok-multi-auth` recorded | none | — |
| PR101-03 | 10/v1-01 | PASS | protocol crate `ProviderBinding` identifiers proven | none | — |
| PR101-04 | 10/v1-01 | PASS | policy: live tests never PASS without creds; PC8 PARTIAL | none | — |
| PR101-HUMAN | 10/v1-01 | HUMAN | `[D-PR]` PC8 live pack + dual-OS flock | external creds/host | HUMAN gate |
| PR102-01 | 10/v1-02 | PARTIAL | `run_api_key_login` exists (`login_coordinator.rs:208`) | F-05: `registry.get` result discarded (unregistered providers proceed); `backend: SecretBackendKind::Ephemeral` hardcoded regardless of store; no-op XAI fallback branch (lines 227-233) not wired to binding-layer rejection | Wave C5-32: require registered descriptor, derive backend from store, remove no-op |
| PR102-02 | 10/v1-02 | PASS | secret absent from `CredentialMetadata` Debug (runtime check `login_coordinator.rs:252-257`) | none | — |
| PR102-03 | 10/v1-02 | PASS | empty/missing secret rejected (`login_coordinator.rs:217-226`) | none | — |
| PR102-04 | 10/v1-02 | PASS | TTY paste UX + per-provider descriptors present | none | — |
| PR102-05 | 10/v1-02 | PASS | binding-layer `reject_xai_api_key_fallback` + test `prohibit_xai_api_key_fallback_for_third_party` (`byok/mod.rs:130-145`) | login path does not call binding-layer rejection (captured in PR102-01) | — |
| OR-01 | 10/v1-03 | PASS | descriptor + `catalog_model_key` + `PublicProviderBinding` + tests (`byok/mod.rs:87-122`) | functional vertical not wired (OR-02/Wave C5) | — |
| OR-02 | 10/v1-03 | SKIP | live Turn smoke requires credentials | no creds | SKIP until creds; never `[x]` |
| GQ-01 | 10/v1-04 | PASS | descriptor + catalog key + public binding + tests | functional vertical not wired | — |
| GQ-02 | 10/v1-04 | SKIP | live Turn smoke requires credentials | no creds | SKIP until creds |
| CF-01 | 10/v1-05 | PASS | descriptor + catalog key + public binding + tests | functional vertical not wired | — |
| CF-02 | 10/v1-05 | SKIP | live Turn smoke requires credentials | no creds | SKIP until creds |
| TW101-01 | 20/v1-01 | PASS | leader byte fixtures/tests under `xai-grok-shell/src/leader/` | none | — |
| TW101-02 | 20/v1-01 | PASS | single-winner race fixture (`connect_or_spawn_has_single_winner`) | none | — |
| TW101-03 | 20/v1-01 | PASS | adapter seam in tower; no second actor type asserted (`xai-grok-tower/src/lib.rs:91`) | registry is opaque-token only (see TW101-04) | — |
| TW101-04 | 20/v1-01 | PARTIAL | `app_server_runtime_registers_one_actor_token_per_session` test exists | F-02: test injects `FakeRuntime`; "mutations serialized by the existing actor" not proven; opaque token registry, not real actor identity | Wave C1-10: real foreground-turn exclusivity/serialization against SessionActor |
| TW101-05 | 20/v1-01 | PARTIAL | test names recorded | F-09: named RED/GREEN evidence documents missing | Wave C0-5/C1: produce per-behavior RED/GREEN |
| TW101-HUMAN | 20/v1-01 | HUMAN | `[D-TW.11/12]` compare fixture to real local leader | manual verify | HUMAN gate |
| TW102-01 | 20/v1-02 | PASS | session metadata types in tower | none | — |
| TW102-02 | 20/v1-02 | PASS | canonical path/symlink race regression | none | — |
| TW102-03 | 20/v1-02 | PARTIAL | `ShellRuntimeAdapter` list/read seam exists; `app_server_multi_workspace_stable_session_ids` test | F-01/F-02: adapter delegates to `FakeRuntime`; list/read not mapped to real `JsonlStorageAdapter`/leader roster | Wave C1-8: real list/read over storage/leader (depends RF102-02) |
| TW102-04 | 20/v1-02 | PASS | current/peak session/resident telemetry | none | — |
| TW102-05 | 20/v1-02 | PASS | no-arbitrary-product-cap regression | none | — |
| TW102-06 | 20/v1-02 | PASS | `cargo check -p xai-grok-tower` warning-free scaffold | none | — |
| TW103-01 | 20/v1-03 | PASS | validated ID/layout in `xai-grok-tower/src/instance.rs` | none | — |
| TW103-02 | 20/v1-03 | PARTIAL | `select_tower_instance_id` + `tower_selection` test | F-07: uses `GROK_TOWER_INSTANCE` (canonical name is `GROK_OSS_TOWER`); returns arbitrary strings without `TowerInstanceId` validation; fallback test asserts only non-empty (ambient env uncontrolled) | Wave C2-15/16: canonical name + validated selector, hermetic precedence |
| TW103-03 | 20/v1-03 | PARTIAL | `two_instances_have_disjoint_registries` (`instance.rs:122`) | in-memory scaffold registries only; not a true dual-OS-process leader/flock test; FINAL_REPORT concedes dual-process proof not delivered | Wave C2-17: true dual-OS-process leader/flock + handshake mismatch |
| TW103-04 | 20/v1-03 | PASS | co-start parser tests | none | — |
| TW103-05 | 20/v1-03 | PASS | bind-warning fixture (loopback default, explicit non-loopback warning) | none | — |
| TW103-06 | 20/v1-03 | PARTIAL | contention/isolation tests exist | F-09: RED/GREEN evidence not captured | Wave C0-5: capture RED/GREEN |
| TW104-01 | 20/v1-04 | PASS | drain state machine | none | — |
| TW104-02 | 20/v1-04 | PASS | crash/restart fixture, stable Session ID, epoch resync | none | — |
| TW104-03 | 20/v1-04 | PASS | lifecycle/latency/peak telemetry | none | — |
| TW104-04 | 20/v1-04 | PASS | structured audit/redaction canary | none | — |
| TW104-05 | 20/v1-04 | PASS | stale metadata matrix | none | — |
| TW104-06 | 20/v1-04 | PASS | `cargo check` tower/shell/workspace green | none | — |
| SP101-01 | 30/v1-01 | PASS | envelope/initialize gate types + tests | none | — |
| SP101-02 | 30/v1-01 | PASS | Session/Turn/Item unions + schema goldens | none | — |
| SP101-03 | 30/v1-01 | PASS | methods params/result shapes roundtrip | none | — |
| SP101-04 | 30/v1-01 | PASS | lifecycle/delta/Interaction/cursor shapes | none | — |
| SP101-05 | 30/v1-01 | PASS | error/limit catalog + retryability fixtures | none | — |
| SP101-06 | 30/v1-01 | PASS | schema/golden/TS drift tests (22 protocol tests) | none | — |
| SP101-07 | 30/v1-01 | PASS | Codex mapping isolated; no native public `Thread` occurrence | none | — |
| SP101-HUMAN | 30/v1-01 | HUMAN | `[D-SP.25]` future Codex adapter missing-jsonrpc policy | product decision | HUMAN gate |
| RF102-01 | 30/v1-02 | PASS | `GrokRuntimeFacade` shape over protocol types | none | — |
| RF102-02 | 30/v1-02 | OPEN | adapter delegates to fake; no real SessionActor command routing | F-01/F-02: not implemented | Wave C1-8: real Shell port mapping each facade call to one leader/SessionActor command |
| RF102-03 | 30/v1-02 | PASS | projection maps every runtime event to Item/lifecycle | none | — |
| RF102-04 | 30/v1-02 | PASS | projector redaction; secret canaries absent | none | — |
| RF102-05 | 30/v1-02 | OPEN | `single_actor_owns_turn_mutation` test injects FakeRuntime | F-02: allows 8 concurrent starts; ordered mutations / single authoritative actor not proven | Wave C1-10: foreground-turn exclusivity, steering, ordering, cancellation |
| RF102-06 | 30/v1-02 | PASS | faithful `FakeRuntime` conformance suite | none (fake is for conformance only) | — |
| RF102-07 | 30/v1-02 | PARTIAL | final green test outputs recorded | F-09: named RED/GREEN per facade behavior missing | Wave C1: per-behavior RED/GREEN |
| AS103-01 | 30/v1-03 | PASS | processor initialize/dispatch + fixtures | none | — |
| AS103-02 | 30/v1-03 | PASS | typed in-process client handle | none | — |
| AS103-03 | 30/v1-03 | PASS | stdio NDJSON/EOF/stderr | none | — |
| AS103-04 | 30/v1-03 | PASS | in-process/stdio conformance identical | none | — |
| AS103-05 | 30/v1-03 | PASS | bounded writer/replay backpressure | none | — |
| AS103-06 | 30/v1-03 | PASS | composition assertion: processor depends on facade trait, no SessionActor construction | none | — |
| AS103-07 | 30/v1-03 | PARTIAL | non-network vertical-slice checks green | F-09: RED/GREEN capture not auditable | Wave C0-5: capture RED/GREEN |
| AS104-01 | 30/v1-04 | OPEN | only `handle_ws_text` helper exists | F-03: no listener/handshake/ping-pong/cap | Wave C3-20/21: real WebSocket listener + black-box tests |
| AS104-02 | 30/v1-04 | PASS | token-file + header constant-time bearer + failure matrix | none | — |
| AS104-03 | 30/v1-04 | PASS | loopback defaults + explicit remote warning | none | — |
| AS104-04 | 30/v1-04 | PASS | redacted connection/audit fields | none | — |
| AS104-05 | 30/v1-04 | OPEN | WS conformance test calls `handle_ws_text` directly | F-03/F-11: not black-box; no real listener | Wave C3-20/24 |
| AS104-06 | 30/v1-04 | OPEN | network attacker/slow client/oversize tests absent | F-03: not implemented | Wave C3-20 |
| AS104-HUMAN | 30/v1-04 | HUMAN | `[D-SEC.13]` TLS/threat acceptance for non-loopback | manual verify | HUMAN gate |
| AS105-01 | 30/v1-05 | OPEN | `projection_rebuild_via_replay_is_stable_for_fake` only | F-06: fake-only; no canonical session-file index | Wave C3-22/23 |
| AS105-02 | 30/v1-05 | OPEN | no canonical epoch/revision persistence | F-06: not implemented | Wave C3-22 |
| AS105-03 | 30/v1-05 | OPEN | `snapshot_then_live_no_gap_on_fake` only | F-06: fake-only attach-boundary replay | Wave C3-23 |
| AS105-04 | 30/v1-05 | OPEN | retention/byte/queue boundaries not implemented | F-06 | Wave C3-22 |
| AS105-05 | 30/v1-05 | OPEN | projection goldens fake-only | F-06 | Wave C3-22 |
| AS105-06 | 30/v1-05 | PARTIAL | `history_parity_epoch_and_redaction_flag` test exists | F-06/F-13: uses `FakeRuntime`; asserts hardcoded `historyEpoch=="epoch_1"` (synthetic); not canonical session files | Wave C3-22/23: real history path |
| AS105-07 | 30/v1-05 | OPEN | crash/rebuild RED/GREEN not recorded | F-06/F-09 | Wave C3-22 |
| AS106-01 | 30/v1-06 | PASS | UNOWNED/HELD/RELEASED/RESOLVED lease transitions | none | — |
| AS106-02 | 30/v1-06 | PASS | Interaction ID distinct from request ID | none | — |
| AS106-03 | 30/v1-06 | PASS | disconnect/expiry; never auto-allow | none | — |
| AS106-04 | 30/v1-06 | PASS | idempotent Interaction response storage | none | — |
| AS106-05 | 30/v1-06 | PARTIAL | `respond_interaction` delegates to `self.inner` (FakeRuntime) | F-01: not mapped to existing Shell permission/elicitation command path (`PendingInteractionGuard`/leader routing) | Wave C6-39: map to real permission path, no second engine |
| AS106-06 | 30/v1-06 | PARTIAL | interaction conformance in-process/stdio | F-11: WS leg is helper-level, not black-box | Wave C3-24/C6-40 — C6-C GREEN: tasks.md checkbox marked; F-11 closed (real WS listener black-box proven). Matrix status update pending orchestrator. |
| AS106-HUMAN | 30/v1-06 | HUMAN | `[D-AP.6]` default wait vs auto-deny timeout policy | product decision | HUMAN gate |
| AS107-01 | 30/v1-07 | PARTIAL | conformance suite exists | F-03/F-11: WS/MCP legs call helpers directly, not black-box across real transports | Wave C3-24/C4-28/C6-43 |
| AS107-02 | 30/v1-07 | PARTIAL | security suite exists | F-11: helper-level; AS104-06 (network attacker/slow client) OPEN | Wave C3-20/C7-46 |
| AS107-03 | 30/v1-07 | PASS | schema/golden/TS drift gates green | none | — |
| AS107-04 | 30/v1-07 | PARTIAL | drain/restart tests exist | F-01/F-13: composition injects FakeRuntime; epoch synthetic | Wave C1-12/C3-22 |
| AS107-05 | 30/v1-07 | PASS | frozen ACP/roster/dashboard surfaces validated | none | — |
| AS107-06 | 30/v1-07 | PARTIAL | delivery evidence partial | F-08/F-09: no `CHANGES.md`/`DECISIONS.md`, no per-wave reviews, RED/GREEN not auditable | Wave C0-4 + per-wave ledger/reviews |
| AS107-HUMAN | 30/v1-07 | HUMAN | `[D-SEC.13]` public-bind threat checklist | manual verify | HUMAN gate |
| MCP101-01 | 40/v1-01 | PASS | crate separation (`cargo check`) | none | — |
| MCP101-02 | 40/v1-01 | PASS | stdio adapter, protocol-only stdout, graceful EOF | none | — |
| MCP101-03 | 40/v1-01 | OPEN | only `post_mcp_response`/cursor helper + auth helper | F-03: no POST/GET/DELETE `/mcp` router/listener/SSE lifecycle | Wave C4-26/27 |
| MCP101-04 | 40/v1-01 | PASS | nine tool descriptors registered | none | — |
| MCP101-05 | 40/v1-01 | PASS | adapter result/error parity with in-process | none | — |
| MCP101-06 | 40/v1-01 | PASS | no local self-injection regression | none | — |
| MCP102-01 | 40/v1-02 | PASS | token-file/header auth helper + 0600 validation | none | — |
| MCP102-02 | 40/v1-02 | PASS | auth failure matrix, indistinguishable 401 | none | — |
| MCP102-03 | 40/v1-02 | PARTIAL | `enforce_body_limit` helper + test | F-11: body limit helper only; SSE/queue limits not enforced in a real transport (no server) | Wave C4-26/27 |
| MCP102-04 | 40/v1-02 | PASS | audit canary/threat fixtures | none | — |
| MCP102-05 | 40/v1-02 | PARTIAL | `adapter_parity` over stdio | F-03/F-11: HTTP driver is helper/cursor table, not a real Streamable HTTP server | Wave C4-24/28 |
| MCP102-HUMAN | 40/v1-02 | HUMAN | `[D-SEC.13]` TLS termination before public bind | manual verify | HUMAN gate |
| TA101-01 | 50/v1-01 | PASS | nine tool input/output defs + schema | none | — |
| TA101-02 | 50/v1-01 | OPEN | facade methods incomplete; shell adapter fake-backed | F-01: real SessionActor operation mapping not done | Wave C1-8/C6-41 |
| TA101-03 | 50/v1-01 | PASS | event projector + redaction canary | none | — |
| TA101-04 | 50/v1-01 | PASS | shared errors/idempotency | none | — |
| TA101-05 | 50/v1-01 | PASS | swarm/limit fixtures | none | — |
| TA101-06 | 50/v1-01 | PARTIAL | mutation tests exist | F-09: per-mutation RED/GREEN not recorded | Wave C0-5 |
| TA102-01 | 50/v1-02 | PASS | fail-closed ACL matrix | none | — |
| TA102-02 | 50/v1-02 | PASS | in-process registration, no JSON-RPC loop | none | — |
| TA102-03 | 50/v1-02 | PASS | MCP/in-process adapter parity for nine tools | none | — |
| TA102-04 | 50/v1-02 | PASS | no `tower_agent_hub` / local self-MCP edge | none | — |
| TA102-05 | 50/v1-02 | PASS | ACL-before-lookup, identical deny | none | — |
| TA102-06 | 50/v1-02 | PASS | full vertical contract green | none | — |
| TS101-01 | 60/v1-01 | PASS | publish name frozen + drift pipeline | none | — |
| TS101-02 | 60/v1-01 | PASS | client/reconnect iterator, epoch validation | none | — |
| TS101-03 | 60/v1-01 | PASS | Node stdio/WS examples compile | none | — |
| TS101-04 | 60/v1-01 | PASS | runtime capability tests (stdio/WS, browser bearer rejection) | none | — |
| TS101-05 | 60/v1-01 | PASS | typed error tests distinct | none | — |
| TS101-HUMAN | 60/v1-01 | HUMAN | `[D-TS.1]` approve name/publication | product decision | HUMAN gate |

## Notes

- **Hybrid runtime:** `SessionStorageHybridRuntime` is rejected (F-13). `rg SessionStorageHybridRuntime crates/` returns no matches — already removed pre-C0. Do not reintroduce.
- **Primary blocker (C-SA):** composition injects `FakeRuntime` (`app_server_composition.rs:15`). Every PARTIAL/OPEN facade→authority task depends on Wave C1-D switching the composition root to the real Shell port.
- **SKIP discipline:** OR-02/GQ-02/CF-02 remain `[ ]`/SKIP; never `[x]` or PASS until credentials exist and live smokes execute.
- **HUMAN gates (8):** TLS/threat (AS104, AS107, MCP102), PC8 live (PR101), leader fixture compare (TW101), jsonrpc policy (SP101), auto-deny policy (AS106), npm publish (TS101) — tracked separately, never COMPLETE-blocking local work.
- **No product code changed this turn** — docs/ledger only.
