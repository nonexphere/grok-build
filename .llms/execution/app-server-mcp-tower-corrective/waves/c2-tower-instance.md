# C2-A — Tower instance config & isolation (wave note)

| Field | Value |
|---|---|
| Handoff | `handoffs/HANDOFF-C2-A-tower-instance.md` |
| Branch | `goblin-implement-epic-tree` |
| Implementer | GLM `glm-5.2` (build) |
| Date | 2026-07-18 |
| Predecessor | C1-G (turn lifecycle) + C0 contract |

## 1. What landed

The composition root now resolves a Tower instance id with the canonical
precedence and validates it through `TowerInstanceId`. A new
`resolve_tower_instance_id` returns a parsed `TowerInstanceId` (or
`TowerInstanceIdError`); the legacy `select_tower_instance_id` is kept as a
fail-soft convenience wrapper that falls back to `default` on invalid input.

Precedence (matches `_shared/tower-instance-lifecycle.md` §“Default instance
selection algorithm”):

1. explicit arg (`--tower <id>`)
2. `GROK_OSS_TOWER` (canonical)
3. `GROK_TOWER_INSTANCE` (legacy, transition-only)
4. literal `default`

There is no ambient “last used” pointer. An invalid explicit arg or env value
surfaces `TowerInstanceIdError` (fail-closed) rather than silently defaulting.
The convenience wrapper `select_tower_instance_id` is the only path that
swallows errors (for ergonomic CLI wiring that prefers a usable id over a
hard failure).

A new tower-crate helper `instance_state_root(home, id) -> PathBuf` derives
`<home>/towers/<instance-id>` as a pure path computation (no mkdir, no symlink
follow, no lock). This is the foundation of multi-instance isolation: distinct
`TowerInstanceId`s yield disjoint state roots, registries, endpoints, locks and
tokens.

### Files
- `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs` — added
  `TOWER_INSTANCE_ENV` (`GROK_OSS_TOWER`), `LEGACY_TOWER_INSTANCE_ENV`
  (`GROK_TOWER_INSTANCE`), `resolve_tower_instance_id` (validated,
  fail-closed), and rewrote `select_tower_instance_id` as a thin fail-soft
  wrapper. Replaced the single legacy-precedence test with 9 hermetic
  `#[serial]` precedence tests using an `EnvGuard` that saves/restores env.
- `crates/codegen/xai-grok-tower/src/instance.rs` — added
  `instance_state_root`, `TOWER_INSTANCES_DIR`, and a unit test
  `instance_state_root_is_disjoint_per_instance_id`.
- `crates/codegen/xai-grok-tower/src/lib.rs` — re-export
  `instance_state_root` and `TOWER_INSTANCES_DIR`.
- `crates/codegen/xai-grok-tower/tests/tower_instance_isolation.rs` (new) — 4
  dual-instance isolation integration tests.

## 2. REAL vs PARTIAL summary

### REAL (proven)
- **Canonical precedence:** `GROK_OSS_TOWER` strictly beats
  `GROK_TOWER_INSTANCE`, which strictly beats `default`. Proven by
  `tower_selection_canonical_env_preferred_over_legacy`,
  `tower_selection_legacy_used_when_canonical_absent`, and
  `tower_selection_default_when_nothing_set`.
- **Explicit arg wins over env:** proven by
  `tower_selection_explicit_wins_over_env_and_default` (non-empty explicit
  wins; empty explicit falls through to canonical env).
- **Empty env falls through:** an empty `GROK_OSS_TOWER` does not short-circuit
  to `default`; it falls to the legacy env. Proven by
  `tower_selection_empty_env_falls_through`.
- **Validation via `TowerInstanceId`:** the resolver delegates to
  `TowerInstanceId::from_str`, so wire-format rules (`[a-z0-9][a-z0-9._-]{0,63}`)
  are enforced and invalid explicit/env values return `TowerInstanceIdError`.
  Proven by `tower_selection_validates_via_tower_instance_id_wire_format`,
  `tower_selection_invalid_explicit_returns_error_fail_closed`, and
  `invalid_instance_id_is_rejected_before_path_derivation`.
- **Fail-closed on bad config:** invalid env does not silently default.
  Proven by `tower_selection_invalid_explicit_returns_error_fail_closed`
  (invalid `GROK_OSS_TOWER` → `is_err()`).
- **No ambient pollution:** an unrelated `GROK_TOWER` env var does not
  influence the resolver. Proven by `tower_selection_does_not_read_other_env_vars`.
- **Hermetic tests:** every env-mutating test is `#[serial]` and uses an
  `EnvGuard` that restores prior env state on drop; tests never touch the real
  `~/.grok-oss` (tower tests use `TempDir`).
- **Dual-instance directory isolation:** two `TowerInstanceId`s yield
  disjoint `<home>/towers/<id>` roots (neither is a prefix of the other).
  Proven by `two_instances_have_disjoint_state_roots` and
  `instance_state_root_is_disjoint_per_instance_id`.
- **Dual-instance registry isolation:** the same session-id string in two
  instances’ `SessionRegistry`s gets independent actor tokens; removing from
  one does not affect the other. Proven by
  `two_instances_have_disjoint_registries_and_directories` (integration) and
  the pre-existing `two_instances_have_disjoint_registries` (lib).
- **InstanceDirectory contention guard:** duplicate instance ids are rejected.
  Proven by `instance_contention_duplicate_id_rejected` and
  `two_instances_have_disjoint_registries_and_directories`.

### PARTIAL (honest, not claimed PASS)
- **Dual-OS-process flock isolation:** the single-winner `instance.lock`,
  endpoint-in-use detection, and stale-PID-vs-live-endpoint reconciliation
  belong to the connect-or-spawn state machine, which needs the full spawn
  path + credentials (C1-J residual). This wave proves the in-process,
  lock-free slice (disjoint roots + disjoint registries + contention guard).
  The dual-process flock half is documented PARTIAL — it needs more infra
  than this wave owns.
- **`instance_state_root` does not validate ownership/permissions:** it is a
  pure path computation. A caller that materializes the directory MUST
  separately reject symlinked/non-owned components per the lifecycle contract.
  Wiring that validation is a follow-on (it belongs with the connect-or-spawn
  state machine, not the resolver).
- **CLI `--tower <id>` wiring:** the resolver is in the composition root but
  the product bin does not yet parse `--tower` from CLI args and feed it as
  the explicit arg. That CLI-matrix wiring is owned by the composition/CLI
  surface (out of scope for C2-A; the resolver is ready to receive it).

## 3. Invariants preserved (re-verified)

- **No second `SessionActor`.** Tower still does not define `SessionActor`;
  the static guard `leader_characterization_tower_has_no_second_actor_type`
  passes (22/22 lib tests).
- **Tower must not depend on Shell.** Unchanged — `xai-grok-tower` Cargo.toml
  has no `xai-grok-shell` dependency; `instance_state_root` is pure path math.
- **No shell `app_server_runtime` edits.** C2-A did not touch
  `xai-grok-shell/src/app_server_runtime/**` (C1-J owns it). The composition
  root only consumes the existing `ShellSessionActorRuntime::new(root)`.
- **No mcp-server edits.** C2-A did not touch `xai-grok-mcp-server`. The
  static guard `composition_source_does_not_register_local_mcp_self_loop`
  still passes.
- **No FakeRuntime in the product path.** Unchanged — the composition root
  still injects the real `ShellSessionActorRuntime`.

## 4. RED / GREEN evidence

Tests live in:
- `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs`
  (`tower_selection_tests` module)
- `crates/codegen/xai-grok-tower/tests/tower_instance_isolation.rs`
- `crates/codegen/xai-grok-tower/src/instance.rs` (unit test)

Evidence logs are captured under
`.llms/execution/app-server-mcp-tower-corrective/tests/c2/`.

**Validation commands** (run from repo root):
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

### Results
- **RED** (`c2_tower_instance_RED.log`): with the canonical `GROK_OSS_TOWER`
  branch stubbed out of `resolve_tower_instance_id`, 3 precedence tests fail —
  `tower_selection_canonical_env_preferred_over_legacy` (legacy wins instead
  of canonical), `tower_selection_explicit_wins_over_env_and_default` (empty
  explicit falls to legacy instead of canonical), and
  `tower_selection_invalid_explicit_returns_error_fail_closed` (invalid
  canonical env no longer fails closed). Confirms the tests exercise the
  canonical-precedence path.
- **GREEN** (`c2_tower_instance_GREEN.log`, `c2_tower_instance_GREEN_gate.log`):
  9/9 `tower_selection_*` tests pass (gate exit 0).
- **GREEN** (`c2_tower_isolation_GREEN.log`, `c2_tower_isolation_GREEN_gate.log`):
  4/4 dual-instance isolation tests pass (gate exit 0).
- **No regression:** 22/22 tower lib tests pass; 11/11 composition-root tests
  pass (incl. the 2 original `composition_tests`); the MCP static guard
  `composition_source_does_not_register_local_mcp_self_loop` passes.
- **Clippy:** clean on all new code (tower `--all-targets` and pager-bin
  `--bins`); only pre-existing warnings remain in unrelated tower modules.

## 5. Honest remaining gaps (PARTIAL — not claimed PASS)

- **Dual-OS-process flock isolation** (single-winner `instance.lock`,
  endpoint-in-use, stale-PID reconciliation) — needs connect-or-spawn state
  machine + credentials (C1-J residual).
- **`instance_state_root` ownership/symlink validation** — pure path today;
  materialization-time validation belongs with the spawn state machine.
- **CLI `--tower <id>` parsing** in the product bin — resolver is ready, CLI
  wiring is a composition/CLI-surface follow-on.
- **Per-instance endpoint/lock/token derivation** from the state root —
  structure is in place (`instance_state_root`); the concrete endpoint/lock
  files are spawn-state-machine work.

## 6. What did NOT change (out of scope)

- No shell `app_server_runtime/**` edits (C1-J owns it).
- No `xai-grok-mcp-server` edits (C4-E owns it).
- No multi-auth edits (C5 owns it).
- No protocol crate changes.
- No `MvpAgent` / `SessionActor` / `spawn_session_on_thread` edits.
- `FakeRuntime` retained for unit/conformance; the product path still injects
  the real port.
