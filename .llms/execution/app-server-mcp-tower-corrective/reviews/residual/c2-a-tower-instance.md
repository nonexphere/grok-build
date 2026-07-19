# Residual review — C2-A Tower instance config & isolation

| Field | Value |
|---|---|
| Wave | C2-A (items 15–18) |
| Mode | implementation review (residual) |
| Reviewer | review harness (read-only, glm-5.2) |
| Date | 2026-07-19 |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

Contract non-negotiables hold. The slice is honestly scoped (REAL for the
in-process, lock-free isolation slice; PARTIAL for the dual-OS-process flock
half, which is explicitly deferred to the C1-J connect-or-spawn state
machine). Findings below are Medium/Low and do not block this wave's bounded
acceptance.

## Severity summary

- Critical: 0
- High: 0
- Medium: 1 (F-2)
- Low: 2 (F-1, F-3)

## Contract non-negotiables (re-checked against source)

- **No second `SessionActor`.** Tower still defines none; static guard
  `leader_characterization_tower_has_no_second_actor_type` present in
  `crates/codegen/xai-grok-tower/src/lib.rs:98`. PASS.
- **No Fake hybrid on product path.** Composition root injects the real
  `ShellSessionActorRuntime`; `instance_state_root` is pure path math. PASS.
- **Tower ≠ Shell.** `crates/codegen/xai-grok-tower/Cargo.toml` has no
  `xai-grok-shell` (grep: no matches) and no `xai-grok-mcp-server`; a
  source-level guard at `xai-grok-tower/src/lib.rs:123` asserts
  `!cargo.contains("xai-grok-shell")`. PASS.
- **No MCP self-loop.** C2-A did not touch `xai-grok-mcp-server`; the
  `composition_source_does_not_register_local_mcp_self_loop` guard is
  preserved. PASS.
- **Secrets.** No secrets introduced; env-only config. PASS.

## Evidence reviewed

- Wave note: `.llms/execution/app-server-mcp-tower-corrective/waves/c2-tower-instance.md`
- Handoff: `.llms/execution/app-server-mcp-tower-corrective/handoffs/HANDOFF-C2-A-tower-instance.md`
- GREEN gate log: `.llms/execution/app-server-mcp-tower-corrective/tests/c2/c2_tower_instance_GREEN_gate.log`
  (9/9 `tower_selection_*` pass across all three bin targets; gate exit 0).
- Isolation gate: `tests/c2/c2_tower_isolation_GREEN_gate.log` (referenced in
  README; 4/4 dual-instance isolation).
- Tower lib regression: `tests/c2/c2_tower_lib_GREEN.log` (22/22).
- Source guards: `xai-grok-tower/src/lib.rs:98`, `:123`;
  `xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs:1308,1319`.

## Findings

### F-1 — `select_tower_instance_id` is fail-soft (Low, high confidence)
`select_tower_instance_id` swallows `TowerInstanceIdError` and falls back to
`default`. The wave note documents this as ergonomic CLI wiring, and the
fail-closed path is the canonical `resolve_tower_instance_id`. Evidence:
`c2_tower_instance_GREEN_gate.log` shows
`select_tower_instance_id_falls_back_to_default_on_invalid` passing — i.e.
invalid input silently yields `default` on that wrapper. Acceptable as long
as the product CLI uses the fail-closed resolver for explicit `--tower`
args; CLI wiring is a documented follow-on. Residual risk: a future caller
that uses the fail-soft wrapper for an explicit arg would silently mask a
bad config.

### F-2 — `instance_state_root` does not validate ownership/symlink (Medium, high confidence)
`instance_state_root` is a pure path computation (`<home>/towers/<id>`); it
performs no mkdir, no symlink-follow, no ownership check. The wave note
flags this as PARTIAL and assigns materialization-time validation to the
connect-or-spawn state machine (C1-J residual). This is correct scoping, but
it means the isolation guarantee is currently directory-naming isolation
only; a symlinked `<home>/towers` could collapse two instances' roots until
the spawn state machine validates. Not a defect in C2-A's bound; tracked as
a dependency on C1-J.

### F-3 — Dual-OS-process flock not proven (Low, high confidence)
Single-winner `instance.lock`, endpoint-in-use, stale-PID reconciliation
are deferred (PARTIAL). The in-process slice (disjoint roots + registries +
contention guard) is proven. Acceptable per handoff acceptance #2
("document PARTIAL if flock needs more infra").

## Required fixes

None for this wave's bounded scope.

## Residual risk / dependencies

- C1-J connect-or-spawn state machine must add ownership/symlink validation
  at materialization time and the single-winner `instance.lock`.
- CLI `--tower <id>` parsing in the product bin is a composition/CLI
  follow-on; the resolver is ready to receive it.

## Commands / results

- `./scripts/run-rust-test-gate.sh tower_selection cargo test -p xai-grok-pager-bin --bins tower_selection` → exit 0, 9/9 pass (GREEN gate log).
- `cargo test -p xai-grok-tower --test tower_instance_isolation` → 4/4 (per README).
- `cargo test -p xai-grok-tower --lib` → 22/22 (no regression).
- Static guards present in source (independently re-read).
