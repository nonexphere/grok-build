# C5-C ProviderBinding projection — RED→GREEN evidence

| Field | Value |
|---|---|
| Wave | C5-C (provider binding projection on session/turn rows) |
| Agent | build (glm-5.2) |
| Branch | `goblin-implement-epic-tree` |
| Date | 2026-07-19 |
| No live credentials | identifier-only `ProviderBinding` (no secrets) |

## Goal

Stop hardcoding `provider_binding: None` on projected `Session`/`Turn` rows
when `SessionStartParams` carried a binding. Persist and re-read the
structured identifier-only `ProviderBinding` (no secrets) and project it on
`list` / `read` / `start` / `resume` / `fork` / `start_turn` / `replay`.

## Approach

- **Durable sidecar** `provider_binding.json` under the session directory
  (`{root}/sessions/{encoded_cwd}/{session_id}/provider_binding.json`).
  Written by `start_session` when `params.provider_binding` is `Some`;
  re-read by every projection surface. Identifier-only by contract —
  `provider_id` / `credential_id` / `model_id` / `backend` / `binding_revision`.
- **Storage adapter** owns the on-disk path layout via a new public
  `JsonlStorageAdapter::provider_binding_file(&self, info)` helper. The
  storage layer stays free of the protocol-type dependency; the runtime
  (`ShellSessionActorRuntime`) does the serde read/write so the
  `ProviderBinding` type lives only in the facade layer.
- **Fork** copies the sidecar (raw bytes) so the forked session inherits the
  parent's binding. New `CopySessionResult::provider_binding_copied` field
  records whether the sidecar was copied.
- **Projection** surfaces (`project_summary_to_session`, `project_updates`)
  now take the binding and project it onto `Session.provider_binding` and
  every inferred `Turn.provider_binding`. `start_turn` loads the binding for
  the session and puts it on the returned `Turn`. `replay`'s
  `SessionChanged` snapshot carries the binding too.
- **No second authority**: the sidecar is the single durable projection
  surface; reads are best-effort (`None` when absent/corrupt — honest for
  pre-C5-C sessions).

## RED (pre-implementation)

Against the pre-change codebase, `project_summary_to_session` hardcoded
`provider_binding: None` (shell_session_actor_runtime.rs:652), and every
`Turn` was built with `provider_binding: None` (lines 935, 1186). A test
asserting `read_session` returns the binding passed to `start_session` would
fail with `None != Some(binding)`.

## GREEN (post-implementation)

`cargo test -p xai-grok-shell --test c5_provider_binding_projection`
→ **10 passed; 0 failed** (see `c5_provider_binding_projection_GREEN.log`).

Regression — c1/c3/c6/c7 shell tests stay green:
`cargo test -p xai-grok-shell --test c1_shell_port --test c1_production_spawn
 --test c1_turn_lifecycle --test c3_history_projection --test c6_respond_interaction
 --test c7_conformance`
→ **79 passed; 0 failed** across 6 binaries
(see `c1_c3_c6_c7_regression_GREEN.log`).

`cargo check -p xai-grok-shell` → clean.

## Tests (10) — contract coverage

| # | Test | Asserts |
|---|---|---|
| 1 | `c5_start_with_binding_read_session_returns_same_identifiers` | `start_session` + `read_session` return the same identifier-only binding; all 5 fields match |
| 2 | `c5_sidecar_json_contains_no_secret_material` | `provider_binding.json` has only structured identifier fields; no `api_key`/`token`/`secret`/`authorization`/`bearer`; round-trips back to `ProviderBinding` |
| 3 | `c5_list_sessions_projects_persisted_binding` | `list_sessions` projects the persisted binding onto the row |
| 4 | `c5_resume_session_projects_persisted_binding` | `resume_session` re-projects the persisted sidecar |
| 5 | `c5_fork_session_inherits_parent_binding` | `fork_session` inherits the parent binding via the copied sidecar; fork's sidecar exists on disk |
| 6 | `c5_start_turn_projects_session_binding_on_turn` | `start_turn` projects the session binding onto the returned `Turn` |
| 7 | `c5_read_session_turns_carry_session_binding` | every inferred `Turn` from `updates.jsonl` carries the session binding |
| 8 | `c5_replay_session_changed_snapshot_projects_binding` | `replay` event 0 (`SessionChanged`) carries the binding |
| 9 | `c5_idempotent_restart_projects_persisted_binding` | idempotent re-start with same key+digest re-projects the persisted binding (no re-write) |
| 10 | `c5_session_without_binding_projects_none_everywhere` | no-regression: a session started without a binding projects `None` everywhere and writes no sidecar |

## No-secret invariant

The `ProviderBinding` type is `deny_unknown_fields` and contains only
`provider_id` / `credential_id` / `model_id` / `backend` / `binding_revision`
(see `provider_binding_is_structured_and_contains_no_secret_material` in
`xai-grok-app-server-protocol/src/lib.rs`). The sidecar is a direct
`serde_json::to_vec_pretty` of that type — there is no path for secret
material to reach the sidecar. Test #2 asserts the on-disk bytes contain no
`api_key`/`token`/`secret`/`authorization`/`bearer` substrings.

## Files

- `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`
  — projection + sidecar read/write helpers
- `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs`
  — `provider_binding_file` path helper + fork sidecar copy
- `crates/codegen/xai-grok-shell/src/session/storage/mod.rs`
  — `CopySessionResult::provider_binding_copied` field
- `crates/codegen/xai-grok-shell/tests/c5_provider_binding_projection.rs`
  — new test binary (10 tests)
