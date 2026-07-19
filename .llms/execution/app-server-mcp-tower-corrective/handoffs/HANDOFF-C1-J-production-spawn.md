# Handoff C1-J — ProductionSpawner + Medium turn findings (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Capability | read-write owned paths only |
| Branch | `goblin-implement-epic-tree` |
| Parallel | C4-E, C2-A (non-overlapping) |

## Goal

1. Close the C1-G residual: wire `ProductionSpawner` (or successor) so product `start_session`/`start_turn` can obtain a real resident `SessionHandle` when a minimal offline/test spawn path is available; if full `spawn_session_on_thread` is too heavy for hermetic CI, implement the largest real path and document remaining HUMAN/creds PARTIAL honestly — **do not** claim production spawn DONE without evidence.
2. Fix accepted Medium findings from C1-H where cheap and correct:
   - F-1: `steer_turn` `Item.event_seq` must be monotonic (not wall-clock alone)
   - F-2: seed `next_ordinal` from summary when available
   - F-3/F-4: reduce TOCTOU/stale-handle risk (document if full SessionThread reaping needs larger design)
   - F-5: stop overclaiming `dispatch_lock` in tests/wave notes if only mailbox is proven

## Read first

- `waves/c1-turn-lifecycle.md`
- `reviews/c1/code-review-turn.md`
- `shell_session_actor_runtime.rs`
- `session/acp_session_impl/spawn.rs` (`spawn_session_on_thread`)
- C0-B command map §2–3

## Non-negotiables

- No second actor; no Fake hybrid
- RED→GREEN under `tests/c1/`
- Exclusive: `app_server_runtime/**`, shell C1 tests, minimal spawn hooks

## Must NOT edit

- mcp-server, multi-auth, app-server transport, pager-bin composition (C2-A owns composition)

## Acceptance

1. At least one of: real offline spawn factory that creates a live actor path **or** documented BLOCKER with exact missing dependency for production spawn
2. Medium F-1 and F-2 fixed with tests
3. Wave note `waves/c1-production-spawn.md` + STATUS/CHANGES
4. Existing c1_turn + c1_shell_port tests still green

## Report back

Files, RED/GREEN, REAL vs PARTIAL, residual blockers.
