# C1-D test evidence (RED / GREEN)

## Status: PENDING execution (no command-execution tool in this subagent)

This subagent (`glm-5.2` build) implemented the C1-D real port and tests but
**does not have a shell/command-execution tool available** in its tool set, so
it could not run `cargo test` / `./scripts/run-rust-test-gate.sh` itself. The
parent orchestrator / fresh reviewer must run the validation commands below and
drop the captured logs into this directory (`red.log`, `green.log`).

The code was statically reviewed for compile correctness (imports, types,
trait impls, public symbol paths) against the current repo state. Any compile
failure surfaced by the commands below is a correction-cycle input.

## Commands to run (from repo root)

### 1. RED baseline (before any fix) — optional, for the record
The tests were written against the real port already implemented, so a true
"RED first" run requires temporarily reverting the port impl. Skip if not
reproducing; the GREEN run below is the authoritative gate.

### 2. GREEN — real-adapter integration tests
```bash
./scripts/run-rust-test-gate.sh c1_real_adapter \
  cargo test -p xai-grok-shell --test c1_shell_port -- --nocapture \
  2>&1 | tee .llms/execution/app-server-mcp-tower-corrective/tests/c1/green-real-adapter.log
```
Expected: 18 `c1_real_adapter_*` tests pass.

### 3. GREEN — composition root
```bash
./scripts/run-rust-test-gate.sh composition_root \
  cargo test -p xai-grok-pager-bin --lib composition -- --nocapture \
  2>&1 | tee .llms/execution/app-server-mcp-tower-corrective/tests/c1/green-composition.log
```
Expected: `composition_root_initialize_session_turn` and
`composition_root_injects_real_port_not_fake_runtime` pass.

### 4. GREEN — invariant guards (lib)
```bash
./scripts/run-rust-test-gate.sh shell_session_actor_runtime \
  cargo test -p xai-grok-shell --lib app_server_runtime -- --nocapture \
  2>&1 | tee .llms/execution/app-server-mcp-tower-corrective/tests/c1/green-invariants.log
```
Expected: `shell_session_actor_runtime_defines_no_session_actor` and
`shell_session_actor_runtime_does_not_use_fake_runtime` pass, plus the existing
Fake-backed conformance tests still pass (FakeRuntime retained).

### 5. Full crate
```bash
cargo test -p xai-grok-shell 2>&1 | tee green-shell-full.log
cargo test -p xai-grok-pager-bin 2>&1 | tee green-pager-bin-full.log
```

## Test inventory (18 real-adapter + 2 composition + 2 invariant)

Real-adapter (`tests/c1_shell_port.rs`):
1. `c1_real_adapter_list_sessions_reads_jsonl_summaries_not_dormant_stub`
2. `c1_real_adapter_read_session_projects_session_row_from_summary`
3. `c1_real_adapter_start_session_persists_summary_and_returns_real_id`
4. `c1_real_adapter_start_session_idempotency_key_dedups_same_session_id`
5. `c1_real_adapter_start_session_idempotency_conflict_on_different_input`
6. `c1_real_adapter_resume_session_loads_persisted_summary`
7. `c1_real_adapter_resume_session_unknown_returns_not_found`
8. `c1_real_adapter_fork_session_copies_history_to_new_cwd`
9. `c1_real_adapter_archive_session_returns_unsupported_not_delete`
10. `c1_real_adapter_start_turn_returns_unsupported_actor_gap`
11. `c1_real_adapter_steer_turn_returns_unsupported_actor_gap`
12. `c1_real_adapter_interrupt_turn_returns_unsupported_actor_gap`
13. `c1_real_adapter_respond_interaction_returns_unsupported`
14. `c1_real_adapter_replay_projects_updates_jsonl_into_runtime_events`
15. `c1_real_adapter_replay_epoch_mismatch_returns_error`
16. `c1_real_adapter_replay_cursor_pagination_advances_after_event_seq`
17. `c1_real_adapter_no_hybrid_authority_real_list_with_fake_mutation_rejected`
18. `c1_real_adapter_shell_runtime_adapter_wraps_real_port`

Composition (`app_server_composition.rs`):
- `composition_root_initialize_session_turn`
- `composition_root_injects_real_port_not_fake_runtime`

Invariant guards (`shell_session_actor_runtime.rs`):
- `shell_session_actor_runtime_defines_no_session_actor`
- `shell_session_actor_runtime_does_not_use_fake_runtime`
