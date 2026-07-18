# Handoff C0-B — SessionActor / leader command map (GLM, read-only)

| Field | Value |
|---|---|
| Agent role | **repo-explore** (read-only) |
| Model | `glm-5.2` |
| Wave | C0 item 5 |
| Capability | **read-only** — no file writes except optional note under corrective ledger if write allowed; prefer returning full map in final message and write only if `read-write` granted |
| Branch | `goblin-implement-epic-tree` |

## Goal

Characterize **existing** leader/`SessionActor` commands, lifecycle, permission/elicitation, persistence, and composition entry points **without changing behavior**. Produce an evidence-backed mapping for every `GrokRuntimeFacade` method → real Shell path.

## Authority

1. Corrective contract Wave C0 §5
2. `crates/codegen/xai-grok-tower/src/lib.rs` — `GrokRuntimeFacade` trait
3. `crates/codegen/xai-grok-shell/src/app_server_runtime/mod.rs` — current inject seam
4. `crates/codegen/xai-grok-shell/src/session/` — SessionActor, storage, ACP
5. `crates/codegen/xai-grok-shell/src/leader/` — connect_or_spawn, protocol
6. `_shared/runtime-facade.md`, `_shared/runtime-ownership.md`, `_shared/crate-map.md`

## Deliverable

Write (if write allowed) or return full content for:

`.llms/execution/app-server-mcp-tower-corrective/waves/c0-session-actor-command-map.md`

### Required table columns

| Facade method | Existing Shell symbol (file:fn) | Message/command type | Persistence touch | Permission/interaction? | Test entrypoints | Risk |
|---|---|---|---|---|---|---|

Cover **all** of:

- `list_sessions`, `read_session`, `start_session`, `resume_session`, `fork_session`, `archive_session`
- `start_turn`, `steer_turn`, `interrupt_turn`
- `respond_interaction`, `replay`

Also document:

- How one SessionActor per loaded Session is enforced today
- Foreground turn exclusivity / interrupt / steer hooks
- Where composition root (`xai-grok-pager-bin`) should inject the port
- What **must not** be reinvented (no second actor, no hybrid Fake+JSONL)

## Constraints

- Read-only exploration; no production edits.
- Cite paths and function names; do not invent APIs.
- Mark `UNVERIFIED` where evidence is missing.

## Done when

- Full method map with file evidence
- Explicit “smallest C1 implementation slice” recommendation (3–5 steps)
- List of existing tests that already touch each path

## Report back

- Full map summary
- Top 5 integration risks
- Recommended C1 RED tests (names only)
