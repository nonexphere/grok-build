# STATUS — Corrective App Server / MCP / Tower

| Field | Value |
|---|---|
| Contract | `.llms/tasks/20260718-correct-app-server-mcp-tower-execution.md` |
| Branch | `goblin-implement-epic-tree` |
| Wave | **C0** — reconcile truth (GLM handoffs in flight) |
| Baseline | FakeRuntime green + adversarial FAIL + hybrid removed |
| Handoffs | `.llms/execution/app-server-mcp-tower-corrective/handoffs/` |
| Commit | `54ebd1f` handoff pack |

## Spawned GLM subagents (model `glm-5.2`)

| ID | Handoff | Role | Subagent |
|---|---|---|---|
| C0-A | `HANDOFF-C0-A-requirement-matrix.md` | build / matrix | running |
| C0-B | `HANDOFF-C0-B-session-actor-map.md` | repo-explore | running |
| C0-C | `HANDOFF-C0-C-architecture-review.md` | review GO/NO-GO | running |

Staged (not spawned until C0 GO): **C1-D** implement port, **C1-E** code review, **C1-F** test review.

## C0 progress
- Hybrid runtime removed (grep `SessionStorageHybridRuntime crates/` → no matches)
- Adversarial audit ingested
- GLM handoff contracts committed
- **C0-A complete:** requirement matrix written and 19 false-`[x]` reopened

## C0-A requirement matrix (GLM `glm-5.2`)

| Field | Value |
|---|---|
| Matrix path | `.llms/execution/app-server-mcp-tower-corrective/waves/c0-requirement-matrix.md` |
| Scope | programs 10–60, all v1 task IDs (120 rows) |
| PASS | 77 |
| PARTIAL | 19 |
| OPEN | 13 |
| SKIP | 3 |
| HUMAN | 8 |
| BLOCKED | 0 |

Reopened this turn (19): PR102-01, TW101-04, TW101-05, TW102-03, TW103-02, TW103-03, TW103-06, RF102-07, AS103-07, AS105-06, AS106-05, AS106-06, AS107-01, AS107-02, AS107-04, AS107-06, MCP102-03, MCP102-05, TA101-06.

Verified: `rg -c '^- \[x\]'` across 10/20/30/40/50/60 tasks.md = 77 `[x]` remaining == PASS count. Only PASS keeps `[x]`.

## Blocker
SessionActor-backed facade for all methods under one authority.

## Next (primary)
Collect C0-A/B/C results → triage → spawn C1-D only on GO.
