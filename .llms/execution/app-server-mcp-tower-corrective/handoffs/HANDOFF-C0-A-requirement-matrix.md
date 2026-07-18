# Handoff C0-A — Requirement matrix + checkbox reconcile (GLM)

| Field | Value |
|---|---|
| Agent role | **build** / implementer (docs + ledger only) |
| Model | `glm-5.2` |
| Wave | C0 items 1–4 |
| Capability | read-write limited to `.llms/**` task files + corrective ledger |
| Branch | `goblin-implement-epic-tree` |
| Must NOT | edit product code under `crates/` or `packages/` |

## Authority (read fully before writing)

1. `.llms/tasks/20260718-correct-app-server-mcp-tower-execution.md` §2–4 Wave C0
2. `.llms/reviews/app-server-mcp-tower-adversarial-audit-2026-07-18.md`
3. `.llms/execution/app-server-mcp-tower/FINAL_REPORT.md`
4. Epic `tasks.md` under `.llms/grok-build/{10,20,30,40,50,60}-*/v1-*/`

## Deliverables (write these files)

1. `.llms/execution/app-server-mcp-tower-corrective/waves/c0-requirement-matrix.md`
   - One row per v1 task ID in programs 10–60 (exclude 70/80/90 unless referenced).
   - Columns: `task_id | epic | status | evidence | gap | next`
   - Status enum **only**: `OPEN | PARTIAL | BLOCKED | SKIP | HUMAN | PASS`
   - **Only PASS may remain `[x]` in tasks.md**

2. Reopen every `[x]` whose literal criterion lacks production code + non-vacuous test.
   - **Must reopen if still checked** (audit F-02..F-05): RF102-02, RF102-05, AS104 network server claims, AS105 persistence claims, MCP101-03, OR-02/GQ-02/CF-02 live smokes checked as done.
   - Do not invent PASS for FakeRuntime-only tests against production criteria.

3. Update `.llms/execution/app-server-mcp-tower-corrective/STATUS.md` with matrix path and counts (PASS/OPEN/PARTIAL/HUMAN/SKIP/BLOCKED).

4. Append one line to `.llms/execution/app-server-mcp-tower-corrective/CHANGES.md` (create if missing).

## Constraints

- No product code changes.
- SKIP ≠ PASS; live provider without credentials = SKIP/open, never `[x]`.
- Hybrid `SessionStorageHybridRuntime` is rejected; note as removed if still mentioned.

## Done when

- Matrix file exists with all in-scope task IDs labeled.
- Unsupported `[x]` reopened with evidence notes in matrix.
- STATUS updated; handoff reports exact commands used (grep/rg only is fine).

## Report back (final message)

- Counts by status
- List of task IDs reopened this turn
- Paths written
- Any ambiguity requiring human decision
