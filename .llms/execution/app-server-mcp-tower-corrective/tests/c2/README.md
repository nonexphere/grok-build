# C2-A — Tower instance config & isolation evidence

| Field | Value |
|---|---|
| Handoff | `handoffs/HANDOFF-C2-A-tower-instance.md` |
| Wave note | `waves/c2-tower-instance.md` |
| Branch | `goblin-implement-epic-tree` |
| Implementer | GLM `glm-5.2` (build) |

## Evidence logs

| File | What |
|---|---|
| `c2_tower_instance_RED.log` | Precedence tests with `GROK_OSS_TOWER` branch stubbed out — 3 fail (canonical-preferred, explicit-empty-falls-to-oss, invalid-env-fail-closed). Proves the tests exercise canonical precedence. |
| `c2_tower_instance_GREEN.log` | 9/9 `tower_selection_*` precedence tests pass (pager-bin bins). |
| `c2_tower_instance_GREEN_gate.log` | `scripts/run-rust-test-gate.sh tower_selection …` exit 0. |
| `c2_tower_isolation_GREEN.log` | 4/4 dual-instance isolation tests pass (tower crate integration). |
| `c2_tower_isolation_GREEN_gate.log` | `scripts/run-rust-test-gate.sh two_instances …` exit 0. |
| `c2_tower_lib_GREEN.log` | 22/22 tower lib tests pass (no regression). |

## C2-B — Dual-instance flock isolation evidence

| Field | Value |
|---|---|
| Handoff | `handoffs/HANDOFF-C2-B-flock-isolation.md` |
| Wave note | `SCRATCH/waves/c2-b.md` |
| Implementer | GLM `glm-5.2` (build) |

| File | What |
|---|---|
| `c2_flock_isolation_RED.log` | Pre-impl: integration test compile fails — `unresolved imports xai_grok_tower::InstanceLock, InstanceLockError`. Proves tests exercise the new `InstanceLock` API. |
| `c2_flock_isolation_GREEN.log` | Post-impl: 10/10 integration tests pass (4 C2-A + 6 new `flock_isolation_tests`). |
| `c2_flock_isolation_lib_GREEN.log` | 29/29 lib tests pass (22 pre-existing + 7 new `lock::tests`). |
| `c2_flock_isolation_two_instances_GATE.log` | `scripts/run-rust-test-gate.sh two_instances …` exit 0. |
| `c2_flock_isolation_instance_contention_GATE.log` | `scripts/run-rust-test-gate.sh instance_contention …` exit 0. |

### C2-B validation commands (run from repo root)

```bash
./scripts/run-rust-test-gate.sh two_instances \
  cargo test -p xai-grok-tower --test tower_instance_isolation

./scripts/run-rust-test-gate.sh instance_contention \
  cargo test -p xai-grok-tower --test tower_instance_isolation

cargo test -p xai-grok-tower --lib   # 29/29 (includes lock unit tests)
cargo test -p xai-grok-tower         # full crate
cargo clippy -p xai-grok-tower --all-targets
```

### C2-B REAL vs PARTIAL

- **REAL:** `fs2` exclusive flock on `<home>/towers/<id>/instance.lock`;
  single-winner among 8 racing threads for the same id; disjoint concurrent
  locks for distinct ids; `AlreadyHeld` classification distinct from IO
  errors; PID roundtrip; non-blocking `is_held_for` probe; per-instance
  endpoint/token/metadata scaffold paths disjoint across instances. No
  Shell dependency. Hermetic `TempDir` tests.
- **PARTIAL (C1-J residual):** connect-or-spawn state machine, endpoint
  binding, credential handshake, stale-PID reconciliation, lock-file
  cleanup on graceful shutdown. The lock file is intentionally NOT removed
  on drop to support these follow-ons.

## Validation commands (run from repo root)

```bash
# Precedence (composition root):
./scripts/run-rust-test-gate.sh tower_selection \
  cargo test -p xai-grok-pager-bin --bins tower_selection

# Dual-instance isolation (tower crate):
./scripts/run-rust-test-gate.sh two_instances \
  cargo test -p xai-grok-tower --test tower_instance_isolation

# Tower lib (no regression):
cargo test -p xai-grok-tower --lib

# Composition root (no regression):
cargo test -p xai-grok-pager-bin --bins composition

# MCP static guard (no regression):
cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http composition_source
```

## REAL vs PARTIAL

- **REAL:** canonical precedence (`GROK_OSS_TOWER` > `GROK_TOWER_INSTANCE` >
  `default`), explicit > env, empty-env fall-through, `TowerInstanceId`
  validation, fail-closed on bad config, no ambient pollution, hermetic
  `#[serial]` env tests, dual-instance directory isolation, dual-instance
  registry isolation, `InstanceDirectory` contention guard.
- **PARTIAL:** dual-OS-process flock isolation (single-winner `instance.lock`,
  endpoint-in-use, stale-PID reconciliation) needs the connect-or-spawn state
  machine + credentials (C1-J residual). `instance_state_root` is a pure path
  computation; ownership/symlink validation belongs with the spawn state
  machine. CLI `--tower <id>` parsing is a composition/CLI-surface follow-on.
