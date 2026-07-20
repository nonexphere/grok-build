# Wave C2-B — Dual-instance flock isolation (TW103-03/06)

| Field | Value |
|---|---|
| Handoff | `handoffs/HANDOFF-C2-B-flock-isolation.md` |
| Branch | working tree on `goblin-implement-epic-tree` (uncommitted) |
| Implementer | GLM `glm-5.2` (build) |
| Scope | `xai-grok-tower/**` only (+ integration tests). No Shell dependency. |

## Goal

Implement true dual-OS-process isolation for Tower instances via a real
`fs2` exclusive flock on `<home>/towers/<id>/instance.lock`, with
single-winner semantics for the same instance id and disjoint concurrent
locks for distinct instance ids.

## What changed

### `crates/codegen/xai-grok-tower/Cargo.toml`
- Added `fs2 = { workspace = true }` (workspace already pins `fs2 = "0.4"`;
  `Cargo.lock` already contains it via other crates). No new external
  dependency introduced to the workspace.

### `crates/codegen/xai-grok-tower/src/lock.rs` (NEW)
- `InstanceLock` — exclusive per-instance flock on `instance.lock` under
  the instance state root. Mirrors the flock pattern from
  `xai-grok-shell/src/leader/lock.rs` at the contract level only; **does
  not import `xai-grok-shell`** (verified by the existing
  `leader_characterization_tower_has_no_second_actor_type` lib test, still
  green).
- `InstanceLockError` — `Io` | `AlreadyHeld { instance_id }`. The loser of
  `try_acquire` gets `AlreadyHeld` (not a generic IO error) so the
  connect-or-spawn state machine can distinguish contention from real
  failures.
- `try_acquire(home, id)` — creates the state root, opens/creates
  `instance.lock`, `try_lock_exclusive`. Contention classified via an
  inlined `is_lock_contended` helper (Unix `WouldBlock` / Windows
  `ERROR_LOCK_VIOLATION` via `fs2::lock_contended_error()`). Inlined so
  Tower does not need to depend on `xai-grok-workspace` for a two-line
  helper.
- `is_held_for(home, id)` — non-blocking contention probe (acquire+release
  if free, report held if contended). For the connect-or-spawn decision.
- `write_pid` / `read_pid` — record/recover the holder PID for diagnostics
  and stale-PID reconciliation.
- `endpoint_path` / `token_path` / `metadata_path` — minimal scaffold paths
  under the instance root (`endpoint`, `token`, `metadata.json`). The lock
  does NOT create these files; the connect-or-spawn state machine (C1-J
  residual) owns their content. The paths are derivable and disjoint per
  instance id.
- `Drop` releases the OS flock by closing the handle. The lock file is
  NOT removed (stale-PID reconciliation owns cleanup; a racing claimer may
  need the file to exist).

### `crates/codegen/xai-grok-tower/src/lib.rs`
- Exported `pub mod lock` and re-exported `InstanceLock`, `InstanceLockError`,
  `INSTANCE_LOCK_FILE`, `INSTANCE_ENDPOINT_FILE`, `INSTANCE_TOKEN_FILE`,
  `INSTANCE_METADATA_FILE`.

### `crates/codegen/xai-grok-tower/tests/tower_instance_isolation.rs`
- Added `mod flock_isolation_tests` with 6 integration tests:
  - `two_instances_take_disjoint_flock_concurrently` — two different ids
    acquire concurrently in two threads (Barrier-synchronized); both win,
    disjoint lock paths.
  - `instance_contention_second_claimer_fails_while_held` — same id, second
    claimer gets `AlreadyHeld`; after the winner drops, a new claimer wins.
  - `instance_contention_single_winner_among_many` — 8 threads race for the
    same id; exactly one winner, 7 losers observe `AlreadyHeld`.
  - `instance_lock_scaffold_files_under_root` — state root materialized,
    lock file under root, endpoint/token/metadata paths under root with
    canonical names and disjoint across instances.
  - `instance_lock_records_holder_pid` — `write_pid`/`read_pid` roundtrip;
    empty file returns None.
  - `instance_lock_probe_reports_held_state` — `is_held_for` tracks
    held/unheld without acquiring.

### `crates/codegen/xai-grok-tower/src/lock.rs` (in-file `#[cfg(test)]`)
- 7 unit tests covering the same invariants at the lib boundary.

## RED → GREEN evidence (under `tests/c2/`)

| File | What |
|---|---|
| `c2_flock_isolation_RED.log` | Pre-impl: `cargo test -p xai-grok-tower --test tower_instance_isolation` fails to compile — `unresolved imports xai_grok_tower::InstanceLock, InstanceLockError` (2 errors). Proves the tests exercise the new API. |
| `c2_flock_isolation_GREEN.log` | Post-impl: 10/10 integration tests pass (4 pre-existing C2-A + 6 new C2-B). |
| `c2_flock_isolation_lib_GREEN.log` | 29/29 lib tests pass (22 pre-existing + 7 new `lock::tests::*`). |
| `c2_flock_isolation_two_instances_GATE.log` | `scripts/run-rust-test-gate.sh two_instances …` exit 0. |
| `c2_flock_isolation_instance_contention_GATE.log` | `scripts/run-rust-test-gate.sh instance_contention …` exit 0. |

## Validation commands (run from repo root)

```bash
# Dual-instance flock isolation (integration):
./scripts/run-rust-test-gate.sh two_instances \
  cargo test -p xai-grok-tower --test tower_instance_isolation

# Contention single-winner (integration):
./scripts/run-rust-test-gate.sh instance_contention \
  cargo test -p xai-grok-tower --test tower_instance_isolation

# Tower lib (no regression, includes new lock unit tests):
cargo test -p xai-grok-tower --lib

# Tower full (lib + integration):
cargo test -p xai-grok-tower

# TW103-02 regression (composition root, canonical precedence):
cargo test -p xai-grok-pager-bin --bins tower_selection

# Composition root regression:
cargo test -p xai-grok-pager-bin --bins composition

# Lint:
cargo clippy -p xai-grok-tower --all-targets
```

## Results

- `two_instances` gate: **exit 0** (10/10 integration pass).
- `instance_contention` gate: **exit 0** (10/10 integration pass).
- Tower lib: **29/29 pass** (was 22; +7 new `lock::tests`).
- Tower integration: **10/10 pass** (was 4; +6 new `flock_isolation_tests`).
- TW103-02 `tower_selection`: **9/9 + 9/9 pass** (both bins, no regression).
- Composition root: **11/11 pass** (no regression).
- `cargo clippy -p xai-grok-tower --all-targets`: no new warnings from
  `lock.rs`. Pre-existing warnings in `lifecycle.rs` (`nonminimal_bool`) and
  `workspace.rs` (`disallowed_methods` / `canonicalize`) are unchanged and
  out of scope for this handoff.

## REAL vs PARTIAL

- **REAL (this wave):** real `fs2` exclusive flock on a real file under a
  `TempDir` home; single-winner among 8 racing threads for the same id;
  disjoint concurrent locks for distinct ids; `AlreadyHeld` classification
  distinct from IO errors; PID roundtrip; non-blocking held-state probe;
  per-instance endpoint/token/metadata scaffold paths disjoint across
  instances. No Shell dependency. No ambient env. Hermetic `TempDir` tests.
- **PARTIAL (out of scope, C1-J residual):** the connect-or-spawn state
  machine that turns a won lock into a bound endpoint + credential token +
  running process; endpoint-in-use detection across instances; stale-PID
  reconciliation (dead holder + missing socket → reclaim lock); cleanup of
  the lock file on graceful shutdown. The lock file is intentionally NOT
  removed on drop to support these follow-ons. CLI `--tower <id>` wiring
  through the composition root remains a CLI-surface follow-on (C2-A
  already proved canonical precedence).

## Task checkbox status

- **TW103-03** (`two_instances_have_disjoint_registries` → true dual-OS-process
  leader/flock): the dual-OS-process flock half is now REAL (single-winner
  `instance.lock` + disjoint concurrent locks). The full
  connect-or-spawn + handshake mismatch remains C1-J residual. Marking
  the flock-isolation slice proven; the task stays PARTIAL until the
  state machine lands.
- **TW103-06** (contention/isolation RED/GREEN evidence): RED and GREEN
  captured under `tests/c2/` with gate logs. Proven for the flock slice.
- **TW103-02** (canonical precedence + validated selector): re-run, still
  9/9 + 9/9 green. Unchanged.

Per the handoff ("update task checkboxes only if proven"), the flock
isolation slice is proven; the broader TW103-03/06 PARTIAL status depends
on the C1-J connect-or-spawn state machine and is not flipped to fully
`[x]` here. The parent orchestrator owns the canonical task.md checkbox
update.

## Assumptions

- `fs2` workspace dep is acceptable for Tower (already used by Shell,
  workspace, multi-auth, tools, marketplace; `Cargo.lock` already
  resolves it). No new external dependency introduced.
- Inlining `is_lock_contended` (rather than depending on
  `xai-grok-workspace::util::is_lock_contended`) is the conservative
  choice: it keeps Tower's dependency surface minimal and avoids coupling
  Tower → workspace for a two-line helper. The Shell leader lock is
  referenced as a pattern only, not imported.
- The lock file is intentionally not removed on drop. This matches the
  Shell leader-lock pattern (cleanup is the stale-PID reconciliation
  step's job) and leaves the file in place for racing claimers.

## Files

- `crates/codegen/xai-grok-tower/Cargo.toml` (modified: +`fs2`)
- `crates/codegen/xai-grok-tower/src/lib.rs` (modified: +`pub mod lock`, re-exports)
- `crates/codegen/xai-grok-tower/src/lock.rs` (new)
- `crates/codegen/xai-grok-tower/tests/tower_instance_isolation.rs` (modified: +`flock_isolation_tests`)
- `.llms/execution/app-server-mcp-tower-corrective/tests/c2/c2_flock_isolation_RED.log` (new)
- `.llms/execution/app-server-mcp-tower-corrective/tests/c2/c2_flock_isolation_GREEN.log` (new)
- `.llms/execution/app-server-mcp-tower-corrective/tests/c2/c2_flock_isolation_lib_GREEN.log` (new)
- `.llms/execution/app-server-mcp-tower-corrective/tests/c2/c2_flock_isolation_two_instances_GATE.log` (new)
- `.llms/execution/app-server-mcp-tower-corrective/tests/c2/c2_flock_isolation_instance_contention_GATE.log` (new)
- `.llms/execution/app-server-mcp-tower-corrective/SCRATCH/waves/c2-b.md` (new, this file)
