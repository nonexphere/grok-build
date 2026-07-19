# C7-B conformance suite test evidence (RED / GREEN)

Evidence files in this directory:
- `c7_conformance_RED.log` — archive honesty assertion inverted
  (`assert_eq!(r_err.code, "ok-archive")`): the test fails because the real
  adapter returns `unsupported` (the honest archive gap). Confirms the
  archive-honesty test is non-vacuous.
- `c7_conformance_GREEN.log` — 18/18 `c7_conformance_*` pass with the real
  `ShellSessionActorRuntime` (JSONL storage + real `cmd_tx` consumer
  spawners) compared against `FakeRuntime` via normalized outcomes.
- `c7_conformance_GREEN_gate.log` — `./scripts/run-rust-test-gate.sh
  c7_conformance cargo test -p xai-grok-shell --test c7_conformance` (exit 0).

## Commands (from repo root)
```bash
# GREEN (gate)
bash scripts/run-rust-test-gate.sh c7_conformance \
  cargo test -p xai-grok-shell --test c7_conformance
# GREEN (full)
cargo test -p xai-grok-shell --test c7_conformance
# No regression in C1
cargo test -p xai-grok-shell --test c1_shell_port --test c1_turn_lifecycle
```

## Test inventory (18 conformance comparisons)

1. `c7_conformance_start_session_shape_matches_modulo_fresh_status`
2. `c7_conformance_start_session_idempotency_conforms`
3. `c7_conformance_invalid_workspace_rejected_by_both`
4. `c7_conformance_list_sessions_count_and_workspace_conform`
5. `c7_conformance_read_session_fresh_conforms_on_empty_projection`
6. `c7_conformance_fork_session_creates_distinct_session_with_workspace`
7. `c7_conformance_resume_session_returns_same_session_id`
8. `c7_conformance_resume_unknown_session_not_found_by_both`
9. `c7_conformance_replay_fresh_session_projects_session_changed_snapshot`
10. `c7_conformance_replay_epoch_mismatch_rejected_by_both`
11. `c7_conformance_archive_session_honest_divergence`
12. `c7_conformance_start_turn_returns_turn_with_matching_kind`
13. `c7_conformance_start_turn_without_resident_real_returns_unsupported`
14. `c7_conformance_steer_turn_returns_item_against_running_turn`
15. `c7_conformance_interrupt_turn_running_turn_conforms`
16. `c7_conformance_interrupt_unknown_turn_rejected_by_both`
17. `c7_conformance_replay_after_turn_projects_events_on_both`
18. `c7_conformance_suite_covers_all_minimum_scenarios` (non-vacuity guard)

## What it proves

The same facade scenarios run against `FakeRuntime` (in-memory contract
fake) and `ShellSessionActorRuntime` (real JSONL storage adapter + real
`cmd_tx` command routing via injected test spawners) produce conforming
normalized results where the real adapter is REAL, and honestly documented
divergences where the real adapter is PARTIAL (archive R6, production spawn
C1-J/C2-A, turn-lifecycle projection C3-F, fresh-session status, turn
status snapshot timing, ordinal offset, steer body type).

The real spawners route `SessionCommand::{Prompt,Interject,Cancel}` and
persist through the real `JsonlStorageAdapter` — they do NOT mix
FakeRuntime authority into the real adapter path.

## C7-E adversarial local suite (hermetic)

Evidence file: `adversarial_GREEN.log` — master log of 8 adversarial gates
(all exit 0), plus SCRATCH copy at
`/tmp/grok-goal-5598c3040156/implementer/waves/c7-e/adversarial_GREEN.log`
and report at `…/c7-e/REPORT.md`.

| # | Gate | Result |
|---|------|--------|
| 1 | `cargo test -p xai-grok-app-server --features websocket` | 50 passed |
| 2 | `cargo test -p xai-grok-mcp-server --features streamable-http` | 16+27 passed |
| 3 | `cargo test -p xai-grok-tower --test tower_instance_isolation` | 10 passed |
| 4 | `cargo test -p xai-grok-app-server security` (canaries) | 5 passed |
| 5 | `cargo build -p xai-grok-pager-bin --bin grok-oss` | exit 0 |
| 6 | `cargo test -p xai-grok-tower workspace_symlink` (fail-closed) | 1 passed |
| 7 | `cargo test -p xai-grok-shell --test c1_turn_lifecycle` | 9 passed |
| 8 | `cargo test -p xai-grok-tower --test tower_instance_isolation flock_isolation` | 6 passed |

New tests added to consolidate the gate:
- `xai-grok-app-server/src/lib.rs::adversarial_rejection_tests` (8 tests):
  malformed JSON-RPC → -32700, batch array → -32600, stdio parse-error
  propagation, WS oversize → -32021, WS batch → -32600, well-formed
  negative control, secret-canary matrix, remote-cleartext bind label.
- `xai-grok-mcp-server/src/transport/stdio.rs::stdio_batch_drops_malformed_line_gracefully`:
  malformed NDJSON line is dropped, well-formed neighbors still respond.

HUMAN-deferred: live remote TLS (D-SEC.13), live provider smoke. No live
secrets were used; secret canaries are hermetic shape detectors only.
