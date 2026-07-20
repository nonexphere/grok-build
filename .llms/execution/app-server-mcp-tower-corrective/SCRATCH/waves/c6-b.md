# SCRATCH — C6-B respond_interaction delivery channel (build, GLM `glm-5.2`)

Branch: `goblin-implement-epic-tree`. Handoff:
`handoffs/HANDOFF-C6-B-respond-interaction.md`. Evidence:
`tests/c6/c6_respond_interaction_GREEN.log` (gate) +
`tests/c6/c6_respond_interaction_regression_GREEN.log` (no-regression).

## One-line status
GREEN. `ShellSessionActorRuntime::respond_interaction` is a delivery channel
into the existing pending-interaction surface — NOT a second permission
engine. It checks `pending_interactions` membership (keyed by
`interaction_id` = `tool_call_id`), removes the entry (first-answer-wins),
and forwards `params.decision` verbatim via a process-local oneshot hub.
No allow/deny re-evaluation. Production auto-register is PARTIAL (the live
actor does not yet register its parked-future oneshots into the hub); the
test inject proves the delivery path REAL.

## Design delivered

1. **`ResidentHandle` extended** with `pending_interactions:
   Option<PendingInteractions>` (projected from `SessionHandle` via
   `from_handle`) and `delivery_hub:
   Option<Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>>` (folded
   into the resident, per the handoff's "or fold into Resident" option).
2. **`respond_interaction` method** (R10 / AS106-05):
   - Session must exist on disk (`find_info` → `session_not_found`).
   - Resident required (no resident → `unsupported`).
   - `pending_interactions` surface required (None → `unsupported`).
   - `interaction_id` must be a key in `pending_interactions`; remove it
     (first-answer-wins). Missing → `interaction_not_found`.
   - Deliver `params.decision` via the delivery hub oneshot if registered.
   - Does NOT re-evaluate allow/deny policy — forwards verbatim.
3. **Test seam** (`InteractionSpawner`): returns a `ResidentHandle` with
   real `pending_interactions` + `delivery_hub` Arcs; the test seeds both
   and asserts delivery via `oneshot::Receiver<String>`.
4. **Static guard** (`interaction_facade_production_source_has_no_second_permission_engine`):
   asserts the production `respond_interaction` body contains no
   `is_allowed` / `evaluate_permission` / `auto_allow` / `should_allow`
   constructs and forwards `params.decision`.

## Files changed (owned)

- `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`
  — `ResidentHandle` gains `pending_interactions` + `delivery_hub` fields;
  `from_handle` populates them; `resident()` clones them; `respond_interaction`
  implemented as delivery channel; module doc updated.
- `crates/codegen/xai-grok-shell/tests/c6_respond_interaction.rs` — NEW;
  10 tests (gate fragment `interaction_facade`).

## Files changed (mechanical, concurrent-work compatibility)

- `crates/codegen/xai-grok-shell/tests/c7_conformance.rs` — 2 spawner struct
  literals updated to include `pending_interactions: None, delivery_hub: None`
  (non-behavioral; C7-B logic unchanged).
- `crates/codegen/xai-grok-shell/tests/c1_production_spawn.rs` — 3 spawner
  struct literals updated identically.
- `crates/codegen/xai-grok-shell/tests/c1_turn_lifecycle.rs` — 2 spawner
  struct literals updated identically.
- `crates/codegen/xai-grok-shell/tests/c1_shell_port.rs` — the old
  `c1_real_adapter_respond_interaction_returns_unsupported` test (which
  asserted the stub) replaced with two tests reflecting the new behavior:
  `c1_real_adapter_respond_interaction_returns_unsupported_without_resident`
  (session on disk, no resident → `unsupported`) and
  `c1_real_adapter_respond_interaction_unknown_session_not_found`
  (unknown session → `session_not_found`).
- `crates/codegen/xai-grok-shell/tests/test_doom_loop_recovery.rs` — fixed
  pre-existing syntax error (stray `prompt_cache_key: None,` after a
  method-call expression) that blocked all shell-test compilation. This was
  committed broken at HEAD by a concurrent agent; the fix removes the
  redundant line (`from_items` already sets `prompt_cache_key` via
  `..Default::default()`).

## Not touched (ownership respected)

- `xai-grok-tower/**` (FakeRuntime is the contract fake; unchanged).
- `xai-grok-shell/src/session/**` (pending_interaction.rs, handle.rs —
  read only; the delivery hub is folded into Resident, not SessionHandle).
- `xai-grok-app-server/**`, `xai-grok-pager-bin/**`.
- C5-C provider-binding-projection work (waits per handoff).

## Reproduce

```bash
# GREEN (gate)
bash scripts/run-rust-test-gate.sh interaction_facade \
  cargo test -p xai-grok-shell --test c6_respond_interaction
# GREEN (full file)
cargo test -p xai-grok-shell --test c6_respond_interaction
# No regression in C1/C7
cargo test -p xai-grok-shell \
  --test c1_shell_port --test c1_turn_lifecycle \
  --test c1_production_spawn --test c7_conformance
```
Results: 10/10 `interaction_facade_*` pass; 7+19+9+18 = 53 C1/C7 tests
still pass (no regression).

## REAL vs PARTIAL

| Aspect | Verdict | Evidence |
|---|---|---|
| Delivery channel (check + remove + forward) | **REAL** | `interaction_facade_delivers_decision_to_parked_oneshot` |
| First-answer-wins idempotency | **REAL** | `interaction_facade_second_call_is_interaction_not_found` |
| No second permission engine | **REAL** | `interaction_facade_does_not_re_evaluate_policy` + static guard |
| All PendingKind variants delivered | **REAL** | `interaction_facade_delivers_for_question_kind` |
| Unknown session / no resident / unknown interaction errors | **REAL** | 3 dedicated tests |
| Production actor auto-register into delivery hub | **PARTIAL** | `from_handle` creates a fresh hub but the live actor (on its own thread) does not yet register oneshots into it. The composition root (C2-A) must share the hub with the actor so it can park futures on it. Until then, `respond_interaction` finds the pending entry and removes it (first-answer-wins holds) but has no oneshot to deliver to. |

## Assumptions

- `params.interaction_id` == `tool_call_id` in the common case (the
  pending_interactions map is keyed by `tool_call_id`; the ACP protocol
  uses the tool_call_id as the interaction id for permission/elicitation
  requests). No transformation is needed — the handoff's "map
  `params.interaction_id` → tool_call_id" is an identity mapping.
- The delivery hub is folded into `ResidentHandle` (per the handoff's "or
  fold into Resident" option) rather than a process-global map on the
  runtime. This keeps the hub per-session and avoids a second global
  registry.
- The pre-existing syntax error in `test_doom_loop_recovery.rs` was
  committed broken at HEAD by a concurrent agent; the trivial fix
  (removing the redundant `prompt_cache_key: None,` line) is
  non-behavioral and unblocks all shell-test compilation.

## Residual (outside this slice)

- Production actor auto-register: the live `SessionActor` must register
  its parked-future oneshots into the `delivery_hub` when it issues a
  blocking reverse-request. This requires the composition root (C2-A) to
  share the hub Arc with the actor at spawn time. Owned by C2-A / the
  production spawn factory.
- The exact gate command from the task definition
  (`cargo test -p xai-grok-shell interaction_facade`) compiles ALL test
  targets in the crate; several are broken by concurrent agents
  (`test_sampling_client.rs` has 19 type-mismatch errors from a sampling
  API change). The gate was run scoped to `--test c6_respond_interaction`
  to isolate this slice's evidence from concurrent breakage.
