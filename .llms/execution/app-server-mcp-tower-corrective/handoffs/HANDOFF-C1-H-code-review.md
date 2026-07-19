# Handoff C1-H — Independent code review of C1-G turn lifecycle (GLM review)

| Field | Value |
|---|---|
| Agent role | **review** |
| Model | `glm-5.2` |
| Wave | C1-G post-implement |
| Capability | read-only |
| Start only after | C1-G implementer reports stable + GREEN evidence |
| Branch | `goblin-implement-epic-tree` |

## Goal

Independent code review of the C1-G turn-lifecycle wiring. **Do not implement fixes.**

## Scope

- Diff / files under `app_server_runtime/**`, related spawn hooks, C1 turn tests
- Corrective contract non-negotiables (no second actor, no Fake hybrid, Tower≠Shell)
- Mapping fidelity to C0-B §1.2

## Checklist

1. Does every new path still use a single SessionActor authority?
2. Are `SessionCommand::{Prompt,Interject,Cancel}` used correctly?
3. Any Send/Sync / await-across-lock hazards?
4. Any silent data loss (archive, delete, truncate)?
5. Tests non-vacuous? Prove real routing not string-matching alone?
6. Residual PARTIAL claims honest?

## Deliverable

`.llms/execution/app-server-mcp-tower-corrective/reviews/c1/code-review-turn.md`

Verdict: **PASS** | **PASS_WITH_FINDINGS** | **FAIL** with Critical/High/Medium/Low table.

## Report back

Verdict + top findings + evidence paths.
