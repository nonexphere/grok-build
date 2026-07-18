# Codex readiness requirement matrix — Wave 0

[provenance: TO_RELEASE.md 2026-07-17, epic README, task program 10/v1-01]

Status values: **PASS** | **PARTIAL** | **SKIP** (no credentials) | **DEFERRED** | **OPEN**.
Live checks never report PASS without credentials and a reproducible pack.

| ID | Requirement | Composition / test | Status | Evidence |
|---|---|---|---|---|
| W1 AUD-001 | phase 1:1 including None slots | `patch_assistant_phases_preserves_none_slots_no_sliding` | PASS | TO_RELEASE.md |
| W2 AUD-002 | materialize by index + FC arg deltas | `materialize_*`, `append_function_call_arguments_delta_*` | PASS | TO_RELEASE.md |
| AUD-003 | FC sibling + MCP opaque | mixed legacy+sibling FC tests | PASS | TO_RELEASE.md |
| A1 | attempt-bound stamp + peek | `AttemptStampLedger`, `resolve_for_request` | PASS | TO_RELEASE.md |
| A2+R5 | refresh single-flight (dual TokenManager + flock) | `two_managers_same_file_home_one_refresh_via_xproc_lock` | PASS | TO_RELEASE.md (not dual OS process) |
| A3 | journal fail-loud | corrupt journal + lazy recover tests | PASS | TO_RELEASE.md |
| A4 | binding pin | `session_pin_wins_over_matching_and_missing_hints` | PASS | TO_RELEASE.md |
| Opaque key | prompt_cache_key omits without identity | `prompt_cache_key_opaque_stable_and_omits_without_identity` | PASS | TO_RELEASE.md |
| M7 / AUD-010 | model cache auth error | `resolve_after_fetch_auth_error_does_not_serve_stale` | PASS | TO_RELEASE.md |
| AUD-011 | Codex capability/binding gate | `classify_codex_backend_prefers_binding_over_url` | PASS | TO_RELEASE.md |
| PC10 | compaction prompt_cache_key | compaction policy tests | PASS | TO_RELEASE.md |
| P4 | title routing | title inference Codex pin tests | PASS | TO_RELEASE.md |
| D10 | OAuth fail-closed | login requires `GROK_CODEX_OAUTH_APPROVED=1` | PASS | TO_RELEASE.md / product path |
| PC8 live | multi-turn cache hit full gate | live opt-in only | **PARTIAL** | `.llms/evidence/pc8-live-2026-07-17.md` — hit observed; full pack open |
| Dual OS process | two OS processes on flock | — | **DEFERRED** | dual-manager proven only |
| R1–R6 1.0 | xAI adapter, keyring, composition, D10 product, subagent multi-account | multi-provider 1.0 | **OPEN** | TO_RELEASE.md |
| Live suite honesty | tests without credentials must not PASS | package gates + this matrix | **PASS** (policy) | live = PARTIAL/SKIP only |

## Commands (offline Wave 0 hygiene)

```bash
cargo test -p xai-grok-auth -p xai-grok-multi-auth --no-fail-fast
git diff --check
```

Live PC8 remains **PARTIAL** until credentials + full evidence pack exist. Do not mark PASS.
