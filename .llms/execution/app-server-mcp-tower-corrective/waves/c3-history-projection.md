# C3-F — Canonical history / RuntimeEvent projection (wave note)

| Field | Value |
|---|---|
| Handoff | `HANDOFF-C3-F-history-projection.md` |
| Branch | `goblin-implement-epic-tree` |
| Implementer | GLM `glm-5.2` (build) |
| Date | 2026-07-19 |
| Depends on | C1-J finished (same shell files) |
| Wave | C3 items 22–23 residual; R2/R11 |

## 1. What landed

A single shared projector over canonical `updates.jsonl` that derives the
`read_session` Turn/Item view (R2) and the `replay` RuntimeEvent stream (R11)
from one pass — **not a second replay buffer**. `read_session` and `replay`
both call `ShellSessionActorRuntime::project_history`, which reads
`updates.jsonl` once via `UpdatesIterator` (the existing Shell symbol) and
runs `project_updates`. There is one projection truth over the canonical
file; no second execution authority, no second buffer, no FakeRuntime.

### Files
- `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`
  — replaced the minimal `project_update_to_event` with a richer
  `project_updates` + `project_line` + `ProjectedHistory` + a
  `project_history` helper on the runtime; updated `read_session` and
  `replay` to use the shared projector; updated module docs (REAL vs
  PARTIAL).
- `crates/codegen/xai-grok-shell/tests/c3_history_projection.rs` (new) — 16
  RED→GREEN tests over real `updates.jsonl` fixtures.

## 2. Projection map (REAL vs PARTIAL)

### REAL (projected from data Shell writes)
| `updates.jsonl` line | `replay` RuntimeEvent | `read_session` Item |
|---|---|---|
| `UserMessageChunk` (text) | `ItemCompleted(UserMessage)` | `Item{UserMessage, Completed}` |
| `AgentMessageChunk` (text) | `ItemDelta{delta}` | `Item{AgentMessage, Completed}` (per-chunk — no grouping) |
| `AgentThoughtChunk` (text) | `ItemCompleted(ReasoningSummary)` | `Item{ReasoningSummary, Completed}` |
| `ToolCall` | `ItemStarted(ToolCall)` | `Item{ToolCall, <status>}` |
| `ToolCallUpdate` (with status) | `ItemCompleted(ToolCall)` | `Item{ToolCall, <status>}` |
| `Plan` | `ItemCompleted(Plan)` | `Item{Plan, Completed}` |

Tool-call lifecycle is correlated: `ToolCall` and its `ToolCallUpdate` share
`item_id = "tc_{tool_call_id}"` (derived from the ACP id already present in
the update — no second buffer), so `replay` emits `ItemStarted` then
`ItemCompleted` for the same item.

### PARTIAL (Shell never writes these — honest)
- **`TurnChanged` is NOT emitted in `replay`.** Shell writes no turn
  lifecycle events. `read_session.turns` are inferred from
  `UserMessageChunk` boundaries with `status: Completed` (inferred from
  persistence; crash-mid-turn is **not** detected — see test
  `c3_read_session_crash_mid_turn_inferred_completed_partial`).
- **`InteractionRequested` is NOT projected.** Shell interaction requests
  are in-memory only (`pending_interaction.rs`); they are never persisted to
  `updates.jsonl`.
- **Item grouping across streaming chunks is not performed.** Each chunk is
  a separate item — Shell writes no item-id correlation for text chunks.
- **`created_at_ms` is `0`.** `UpdatesIterator` parses `SessionUpdate`
  (discarding the `SessionUpdateEnvelope.timestamp`); exposing it would
  require changing the shared `UpdatesIterator` symbol (out of scope for
  this handoff — owned files are `app_server_runtime/**` projection helpers).
- **xAI extension updates** (`RewindMarker`, `AutoCompact*`, `Memory*`,
  `Subagent*`, etc.) are skipped — they have no `RuntimeEvent`
  representation; rewind/compaction/subagent projection is deferred.
- **`SessionInfoUpdate` / `AvailableCommandsUpdate` / `CurrentModeUpdate` /
  `ConfigOptionUpdate`** are session-meta updates, not item lifecycle
  events — skipped (no `RuntimeEvent` variant).
- **`provider_binding`** on projected `Turn` is `None` (unchanged — Shell
  `Summary` carries `current_model_id` but not a full `ProviderBinding`).

## 3. Design: one projector, two views (no second buffer)

```text
updates.jsonl ──UpdatesIterator──► project_updates(session_id, iter)
                                        │
                                        ▼
                                 ProjectedHistory
                                  { events, turns, items }
                                        │
                          ┌─────────────┴──────────────┐
                          ▼                            ▼
                   replay (events +            read_session (turns + items)
                   SessionChanged snapshot)
```

- `replay` prepends the `SessionChanged` snapshot (event 0, projected from
  the real `Summary`) then extends with `history.events`. Pagination over
  `after_event_seq` is unchanged (page size 100, matches Fake conformance).
- `read_session` returns `history.turns` / `history.items` (gated by
  `include_turns` / `include_items`).
- Both views come from the **same pass** over the **same file** — there is
  no second replay buffer and no second execution truth.

## 4. Invariants preserved (re-verified)

- **No second `SessionActor`** — static guard
  `shell_session_actor_runtime_defines_no_session_actor` still passes.
- **No hybrid Fake+JSONL authority** — static guard
  `shell_session_actor_runtime_does_not_use_fake_runtime` still passes.
- **No second replay buffer** — the projector reads `updates.jsonl` once
  via `UpdatesIterator`; `read_session` and `replay` share the result. The
  `tool_call_id → item_id` correlation is derived from the ACP id already in
  the update, not a parallel state machine.
- **Tower does not gain a Shell dependency** — unchanged.

## 5. RED / GREEN evidence

Tests: `crates/codegen/xai-grok-shell/tests/c3_history_projection.rs` (16 tests).
Evidence logs:
`.llms/execution/app-server-mcp-tower-corrective/tests/c3/`.

| File | What |
|---|---|
| `c3_history_projection_RED.log` | RED: source reverted to C1-J (empty turns/items, minimal projection). 9/16 FAIL (turn/item/tool-call-lifecycle/thought/plan projections missing). 7 PASS = the honest-PARTIAL absence tests (no TurnChanged, no InteractionRequested, skips xAI, cursor beyond end, snapshot event 0, empty updates, agent delta) — these pass in both phases because they assert the ABSENCE of synthesized events. |
| `c3_history_projection_GREEN.log` | GREEN: 16/16 pass with the shared projector. |
| `c3_history_projection_GREEN_gate.log` | `scripts/run-rust-test-gate.sh c3_read_session cargo test -p xai-grok-shell --test c3_history_projection` → exit 0 (16 passed; gate fragment `c3_read_session` matched). |

**Validation commands** (run from repo root):
```bash
# Real-adapter GREEN gate:
bash scripts/run-rust-test-gate.sh c3_read_session \
  cargo test -p xai-grok-shell --test c3_history_projection

# Regression: c1 + c3 shell tests:
cargo test -p xai-grok-shell \
  --test c1_shell_port --test c1_turn_lifecycle --test c1_production_spawn \
  --test c3_history_projection

# Invariant guards:
cargo test -p xai-grok-shell --lib app_server_runtime
cargo test -p xai-grok-tower --lib
```

## 6. Honest remaining gaps (PARTIAL — not claimed PASS)

- **Turn lifecycle:** `TurnChanged` not emitted in replay; turn status
  inferred `Completed` from persistence (crash-mid-turn not detected).
- **Item grouping:** streaming chunks are not grouped into single items.
- **Timestamps:** `created_at_ms` is 0 (`UpdatesIterator` drops the envelope
  timestamp).
- **`InteractionRequested`:** not projected (in-memory only in Shell).
- **xAI extension updates:** skipped (rewind/compaction/subagent projection
  deferred).
- **`provider_binding`** on projected `Turn`/`Session` still `None`.
- **`read_session` pagination:** not implemented (returns all turns/items);
  the `SessionReadResult` contract has no pagination cursor, so this matches
  the contract. Large histories are unbounded — flagged for a future
  paginated `read_session` contract if needed.

## 7. What did NOT change (out of scope)

- No `UpdatesIterator` / `StorageAdapter` changes (shared Shell symbols).
- No MCP HTTP / WS server work (C3/C4).
- No `MvpAgent` / `SessionActor` source edits.
- No Tower crate changes.
- No composition root changes.
