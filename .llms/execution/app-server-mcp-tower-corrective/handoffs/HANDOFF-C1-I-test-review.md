# Handoff C1-I — Independent test review of C1-G turn lifecycle (GLM review)

| Field | Value |
|---|---|
| Agent role | **review** |
| Model | `glm-5.2` |
| Wave | C1-G post-implement |
| Capability | read-only |
| Start only after | C1-G implementer reports stable + GREEN evidence |
| Branch | `goblin-implement-epic-tree` |
| Parallel with | C1-H after C1-G stable |

## Goal

Independent test-adequacy review for C1-G. **Do not implement.**

## Scope

- `crates/codegen/xai-grok-shell/tests/c1_*.rs` and related unit tests
- Evidence under `.llms/execution/app-server-mcp-tower-corrective/tests/c1/`
- Vacuous filters, Fake-as-production claims, SKIP-as-PASS

## Checklist

1. RED→GREEN evidence present and non-empty.
2. At least one test exercises real actor/handle path (not only FakeRuntime).
3. Concurrent start / interrupt / steer coverage vs acceptance item 10.
4. No empty `cargo test` filter that always passes.
5. Gaps listed as OPEN/PARTIAL with unblock condition.

## Deliverable

`.llms/execution/app-server-mcp-tower-corrective/reviews/c1/test-review-turn.md`

Verdict: **PASS** | **PASS_WITH_FINDINGS** | **FAIL**.

## Report back

Verdict + coverage gaps + evidence paths.
