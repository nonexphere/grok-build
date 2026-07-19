# Handoff C3-F — Canonical history / RuntimeEvent projection (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Depends on | C1-J finished (same shell files — **do not start until C1-J reports done**) |
| Wave | C3 items 22–23 residual; R2/R11 |

## Goal

Improve `read_session` Turn/Item projection and `replay` full `RuntimeEvent` projection over canonical `updates.jsonl` without a second execution truth.

## Read first

- C0-B R2/R11
- `shell_session_actor_runtime.rs` replay projector
- corrective items 22–23
- existing c1 shell port replay tests

## Owned files

- `app_server_runtime/**` projection helpers
- shell tests for history/replay
- `waves/c3-history-projection.md`, `tests/c3/*` history evidence

## Must NOT

- Second replay buffer
- FakeRuntime product path
- Edit mcp/ws while fixing projection

## Acceptance

1. RED→GREEN tests that load real updates.jsonl fixtures and project Turn/Item and richer RuntimeEvent lifecycle where data exists
2. Crash/restart or cursor stale cases if feasible
3. Honest PARTIAL for events Shell never writes

## Report back

Files, RED/GREEN, REAL vs PARTIAL.
