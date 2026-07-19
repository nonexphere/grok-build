# Residual review — C7-B Shared FakeRuntime vs real adapter conformance

| Field | Value |
|---|---|
| Wave | C7-B (C1 residual item 13 + C7 gap G3) |
| Mode | implementation review (residual) |
| Reviewer | review harness (read-only, glm-5.2) |
| Date | 2026-07-19 |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

One normalized conformance suite runs the same facade scenarios against
`FakeRuntime` and `ShellSessionActorRuntime` (real JSONL storage + real
`cmd_tx` command routing via injected test spawners — NOT `FakeRuntime` for
the real side) and compares normalized results. Conformance is asserted
where it holds; divergences are documented honestly and asserted with the
exact expected difference. No live credentials required. Findings Low.

## Severity summary

- Critical: 0
- High: 0
- Medium: 0
- Low: 3 (F-1, F-2, F-3)

## Contract non-negotiables (re-checked)

- **No second actor / no Fake hybrid.** The real side uses real `cmd_tx`
  consumer spawners (`AutoCompleteSpawner` / `HeldTurnSpawner`) routing
  `SessionCommand::{Prompt,Interject,Cancel}` and persisting through the
  real `JsonlStorageAdapter`. `FakeRuntime` is the contract fake for the
  Fake side only. The static guards
  `shell_session_actor_runtime_defines_no_session_actor` /
  `shell_session_actor_runtime_does_not_use_fake_runtime` remain (per C1-J
  GREEN). PASS.
- **No live credentials / no full production spawn required.** The
  `start_turn_without_resident` scenario exercises the real adapter's
  no-spawner path (`unsupported`); turn scenarios use the test spawner, not
  production spawn. Handoff "Must NOT" satisfied. PASS.
- **Tower ≠ Shell.** No tower or shell source edits; only a new test file
  `crates/codegen/xai-grok-shell/tests/c7_conformance.rs`. PASS.
- **Secrets.** Normalization strips non-deterministic ids/timestamps/
  revisions; no secret canaries introduced or weakened. PASS.

## Evidence reviewed

- Wave note: `.llms/execution/app-server-mcp-tower-corrective/waves/c7-conformance.md`
- Handoff: `.llms/.../handoffs/HANDOFF-C7-B-conformance-suite.md`
- SCRATCH: `.llms/.../SCRATCH/waves/c7-b.md`
- GREEN gate: `.llms/.../tests/c7/c7_conformance_GREEN_gate.log`
  (18/18 `c7_conformance_*` pass; gate exit 0).
- RED: `tests/c7/c7_conformance_RED.log` (referenced in SCRATCH).
- No regression: `c1_real_adapter_*` 18/18 + `c1_turn_*` 9/9 still pass.

## Findings

### F-1 — `archive_session` divergence is honest but unresolved (Low, high confidence)
Fake supports archive (marks `Archived`); Real returns `unsupported` because
mapping `archive` → `delete_session` is data loss (R6 product decision
pending). The divergence is asserted (`c7_conformance_archive_session_honest_divergence`)
and documented. This is correct honesty, but R6 remains an open product
decision; the real adapter has no archive path.

### F-2 — Turn status snapshot + ordinal offset divergences (Low, high confidence)
Fake `start_turn` returns the pre-completion snapshot (`InProgress`); Real
returns `Completed` after the consumer resolves the oneshot. Fake numbers
ordinals from 1; Real seeds from `num_messages.max(1)` then `fetch_add(1)+1`
so the first resident turn ordinal is 2 (C1-H F-2). Both divergences are
documented and asserted. Acceptable, but they mean the two adapters are not
interchangeable for status/ordinal-sensitive consumers.

### F-3 — Replay turn-lifecycle + steer body-type divergences (Low, high confidence)
Fake emits `TurnChanged` + full item lifecycle; Real projects only what
`updates.jsonl` carries (Shell writes no turn lifecycle events — C3-F
PARTIAL). Fake steer returns a `UserMessage` item; Real returns an
`AgentMessage` envelope (Shell `Interject` is fire-and-forget; the adapter
synthesizes an Item to satisfy `steer_turn -> Item`). Both documented and
asserted. These are cross-wave dependencies (C3-F, R8), not C7-B defects.

## Required fixes

None for this wave's bounded scope.

## Residual risk / dependencies

- Closing the documented divergences is owned by:
  - C1-J / C2-A — production `spawn_session_on_thread` assembly (HUMAN
    creds) closes the `start_turn_without_resident` divergence.
  - C3-F — turn-lifecycle projection (Shell writes no turn events) closes
    the replay turn-lifecycle divergence.
  - R6 — archive product decision (keep-on-disk vs delete) closes the
    archive divergence.
  - R8 — steer `Item` shape product decision closes the steer body-type
    divergence.

## Commands / results

- `bash scripts/run-rust-test-gate.sh c7_conformance cargo test -p xai-grok-shell --test c7_conformance` → exit 0, 18/18 pass (GREEN gate log).
- `cargo test -p xai-grok-shell --test c1_shell_port --test c1_turn_lifecycle` → 18/18 + 9/9 (no regression).
