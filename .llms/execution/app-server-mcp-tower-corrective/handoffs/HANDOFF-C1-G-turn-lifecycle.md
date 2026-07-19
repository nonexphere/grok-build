# Handoff C1-G — Wire turn lifecycle via SessionHandle channels (GLM implementer)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Wave | C1 follow-on (items 8–10 residual): `start_turn` / `steer_turn` / `interrupt_turn` |
| Capability | read-write product code under owned paths only |
| Depends on | C1-D landed (`ShellSessionActorRuntime` storage-backed); C1-E/F PASS |
| Branch | `goblin-implement-epic-tree` |
| Parallel writers | **none** — exclusive product writer this wave |

## Goal

Close the C1-D **actor fixture gap** for turn methods. `ShellSessionActorRuntime` currently returns `unsupported` for `start_turn`, `steer_turn`, and `interrupt_turn`. Implement the smallest real path that routes these methods through a **live** Shell `SessionActor` via its existing `Send` `SessionHandle` (`cmd_tx` + `SessionCommand::{Prompt,Interject,Cancel}`).

**Do not invent a second actor, FakeRuntime hybrid, or turn state machine.**

## Authority (non-negotiable)

- One runtime authority: existing Shell `SessionActor` on dedicated thread + `LocalSet`.
- `SessionHandle` is `Clone + Send`; actor is `!Send` — never move the actor across threads.
- Tower must not import Shell; keep all wiring in Shell + composition.
- No hybrid Fake mutation + JSONL read authority.
- RED → GREEN with `./scripts/run-rust-test-gate.sh` (or package-scoped `cargo test` if gate script rejects vacuous filters).
- experimental-v2 / product composition only; do not claim full dashboard/70-80-90 scope.

## Evidence map (read first)

| Artifact | Path |
|---|---|
| Corrective contract | `.llms/tasks/20260718-correct-app-server-mcp-tower-execution.md` § Wave C1 |
| Command map | `.llms/execution/app-server-mcp-tower-corrective/waves/c0-session-actor-command-map.md` §1.2 |
| C1-D wave note (gaps) | `.llms/execution/app-server-mcp-tower-corrective/waves/c1-shell-port.md` |
| Real port (PARTIAL turns) | `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs` |
| SessionHandle | `crates/codegen/xai-grok-shell/src/session/handle.rs` (`cmd_tx`, `current_prompt_id`) |
| SessionCommand | `crates/codegen/xai-grok-shell/src/session/commands.rs` |
| Spawn | `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` — `spawn_session_on_thread` |
| Prompt dispatch | `MvpAgent::prompt` → `SessionCommand::Prompt` (`acp_agent.rs` ~2017; `run_loop.rs`) |
| Interject | `SessionCommand::Interject` (`commands.rs` ~669; `run_loop.rs` ~734) |
| Cancel | `SessionCommand::Cancel` (`commands.rs` ~566; `run_loop.rs` ~420) |
| Existing C1 tests | `crates/codegen/xai-grok-shell/tests/c1_shell_port.rs` |
| Composition | `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` |

## Design constraints (from C0-B)

| Facade | Map to | Notes |
|---|---|---|
| `start_turn` | `SessionCommand::Prompt` (oneshot result) | Convert `InputBlock` → `ContentBlock` / prompt text; honor session residency |
| `steer_turn` | `SessionCommand::Interject` | Verify `turn_id` vs `current_prompt_id` when possible; synthesize `Item` envelope if Shell returns none |
| `interrupt_turn` | `SessionCommand::Cancel` | Verify `turn_id` matches running turn when possible |
| `respond_interaction` | **OUT OF SCOPE for C1-G** | R10 — leave `unsupported` unless trivial channel already exists |
| `archive_session` | **OUT OF SCOPE** | R6 HUMAN product decision |

## Recommended implementation sketch

1. **Resident map** on `ShellSessionActorRuntime` (or adjacent type under `app_server_runtime/`):
   - `session_id → (SessionHandle, SessionThread)` under `Mutex` / `DashMap` as appropriate.
   - `SessionHandle` is `Send`; keep `SessionThread` (JoinHandle) for reaping only.
2. **Ensure resident** on `start_session` / `start_turn` / `resume_session`:
   - Prefer reusing existing spawn path (`spawn_session_on_thread` or a thinner test-friendly factory if one can be extracted without a second actor).
   - `spawn_session_on_thread` has a large arg list — build the **minimal** defaults needed for experimental-v2 local sessions (no live cloud creds required for RED/GREEN unit path).
   - If full production spawn cannot be completed in this slice without HUMAN credentials, still wire command send paths and prove them with a **real** actor fixture in tests (not FakeRuntime). Document any residual product-path gap honestly as PARTIAL.
3. **Command send helpers** (async):
   - `start_turn`: send `Prompt` with oneshot; map result → protocol `Turn`.
   - `steer_turn`: send `Interject`; return protocol `Item` (adapter-side envelope OK).
   - `interrupt_turn`: send `Cancel`; return `Ok(())` when channel accepts.
4. **Persistence proof**: after `start_turn`, assert real disk side effects under TempDir:
   - `updates.jsonl` and/or `chat_history.jsonl` grow, **or**
   - actor-visible turn state via `current_prompt_id` / signals if full model turn is not hermetic without creds.
5. **Foreground rules** (item 10): at least one test for concurrent start serialization or second start while busy (match existing Shell `dispatch_lock` / mailbox behavior — do not invent new rules).

## Files owned (exclusive writer)

- `crates/codegen/xai-grok-shell/src/app_server_runtime/**`
- `crates/codegen/xai-grok-shell/tests/c1_shell_port.rs` (and new `c1_turn_*.rs` if cleaner)
- Minimal hooks in `session/acp_session_impl/spawn.rs` **only if required** for a testable factory (prefer pub(crate) thin wrapper, not redesign)
- Ledger only under `.llms/execution/app-server-mcp-tower-corrective/{waves,tests,STATUS,CHANGES}.md`

## Must NOT edit

- MCP HTTP server, WebSocket transport implementation (C3/C4 explorers own mapping only)
- Provider verticals (C5)
- Protocol crate public schema (unless a type conversion is impossible without a tiny private helper)
- FakeRuntime production composition reintroduction
- Unrelated cleanup

## Acceptance criteria (all required for DONE)

1. `start_turn` / `steer_turn` / `interrupt_turn` no longer unconditionally return `code: "unsupported"` when a resident handle exists.
2. RED tests written first; GREEN captured under  
   `.llms/execution/app-server-mcp-tower-corrective/tests/c1/` (e.g. `c1_turn_lifecycle.txt`).
3. At least one test proves command routing against a **real** SessionActor path or equivalent real `cmd_tx` consumer that is not FakeRuntime.
4. Existing 18 `c1_real_adapter_*` tests still pass (or updated honestly if signatures change).
5. Wave note: `waves/c1-turn-lifecycle.md` with map table REAL vs PARTIAL.
6. STATUS.md + CHANGES.md updated; do **not** mark Wave C1 fully complete if R6/R10/R11 remain.

## Out of scope / honest PARTIAL allowed

- Full live model inference (no live creds) — use offline/stub sampling if Shell already has a test mode; else prove command delivery + persistence intent without claiming end-to-end model PASS.
- `respond_interaction` (R10), archive (R6), full RuntimeEvent projection (R11).
- C2 Tower instance env rename, dual-process flock, etc.

## Report back (mandatory structure)

```text
## C1-G report
- Files changed:
- RED command + result:
- GREEN command + result:
- Evidence paths under tests/c1/:
- REAL methods:
- PARTIAL remaining:
- Risks / blockers:
- Suggested next handoff:
```

## Stop if

- Completing would require a second SessionActor or Fake hybrid → STOP, document BLOCKER.
- Disk full / cargo ENOSPC → `cargo clean` targeted then retry once; if still blocked, report.
- Same compile error twice with no new hypothesis → stop third blind patch; report.
