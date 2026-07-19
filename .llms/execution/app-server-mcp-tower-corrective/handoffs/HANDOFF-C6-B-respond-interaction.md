# Handoff C6-B — respond_interaction delivery channel (R10 / AS106-05)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Branch | `goblin-implement-epic-tree` |

## Goal

Implement `ShellSessionActorRuntime::respond_interaction` so it is a **delivery channel** into the existing pending-interaction surface — **not** a second permission engine.

## Design (required)

1. Extend `ResidentHandle` with `pending_interactions: PendingInteractions` (from SessionHandle).
2. Add a process-local delivery hub on the runtime:
   `Arc<Mutex<HashMap<(session_id, interaction_id), oneshot::Sender<String>>>>` **or** fold into Resident.
3. `respond_interaction`:
   - session must exist (storage)
   - if resident present: require `interaction_id` in `pending_interactions` (keys are tool_call_id); map `params.interaction_id` → tool_call_id
   - remove entry (first-answer-wins / idempotent second call → `interaction_not_found` or explicit duplicate code)
   - deliver `params.decision` string via oneshot if registered; **do not** re-evaluate allow/deny policy
   - if no resident: may return `unsupported` or `session_not_loaded` honestly
4. Test seam: inject resident with pre-seeded pending map + oneshot; call facade; assert delivery
5. Gate name fragment: `interaction_facade` for `./scripts/run-rust-test-gate.sh interaction_facade cargo test -p xai-grok-shell interaction_facade`

## Must NOT

- Invent policy allow/deny logic
- Call FakeRuntime
- Silently auto-allow

## Files owned

- `app_server_runtime/**`, shell tests for interaction_facade
- Update AS106-05 checkbox only if GREEN proven
- Ledger waves/tests

## Report

Files, RED/GREEN, REAL vs PARTIAL (production actor auto-register still PARTIAL if only test inject).
