# Goblin multi-provider / Codex — open-source release inventory

**Status:** Codex offline production path **PASS** + **live PC8 prompt-cache hit proven** via `goblin` headless multi-turn (2026-07-17). Multi-provider 1.0 R-items still open.
**Last updated:** 2026-07-17
**Goal docs:** `CODEX_GOAL_PRODUCTION.md`, `CODEX_100_PERCENT_GOAL.md`, `CODEX_AUDIT_REMEDIATION_PLAN.md`
**Validation:** `.llms/reviews/codex-validation-relaunch-2026-07-17.md` + implementer scratch
**Normative detail:** `docs/architecture/multi-provider-auth/`, `task.md`; forge = wire reference only, not Codex cache proof.

---

## Launch tiers

| Tier | Claim | Gate |
|------|--------|------|
| **Experimental OSS** | “Codex preview works for final-answer turns” | Honest README + install; known limitations listed below |
| **Beta / production-ready (single-machine multi-process offline)** | Auth+wire+catalog+capability gate offline | A1–A4, A2+R5, A3, AUD-003, M7, AUD-011, PC10, P4 + docs honesty |
| **1.0 multi-provider** | Full control plane | Beta + R-items (xAI adapter, keyring, D10, subagent multi-account) + live PC8 if claiming cache |

---

## Works today

- Catalog keys `codex/{credential_id}/{slug}`; no OAuth access token in `ModelEntry.api_key`
- Request-time `BearerResolver` + shared TokenManager resolve; multi-provider 401 ≤ 1 resubmit
- Current-thread safe multi-provider I/O (`block_on_safe` / no LocalSet panic)
- Short slug (single account) + full catalog key; ambiguous multi-account error on startup
- Codex wire: system/developer → `instructions`; empty `response.completed.output` recovery / materialize
- Effort menu + CLI `--reasoning-effort` / `--effort`
- Login fail-closed without `GROK_CODEX_OAUTH_APPROVED=1` (or explicit client id for dev)

### Offline gates (proven this branch)

| Gate | Status | Evidence |
|------|--------|----------|
| **W1 AUD-001** phase 1:1 (incl. `None` slots) | **PASS** | `patch_assistant_phases_preserves_none_slots_no_sliding` |
| **W2 AUD-002** materialize by index + FC arg deltas | **PASS** | `materialize_*` + `append_function_call_arguments_delta_*` |
| **AUD-003** FC sibling + MCP / opaque | **PASS** | hot path fail-loud; mixed legacy+sibling (`create_response_mixed_legacy_and_sibling_fc_preserves_both_once`) |
| **A1 attempt-bound stamp + peek** | **PASS** | `AttemptStampLedger` (`take_attempt` + concurrent out-of-order test) + `resolve_for_request` + peek-not-send |
| **A2+R5 refresh single-flight** | **PASS (dual TokenManager + file flock)** | `two_managers_same_file_home_one_refresh_via_xproc_lock` — **not** dual OS processes (see deferred) |
| **A3 journal fail-loud** | **PASS** | `corrupt_journal_quarantines_and_fails_loud`; `store_load_recovers_pending_journal_lazily` |
| **A4 binding pin** | **PASS** | `session_pin_wins_over_matching_and_missing_hints` |
| **Opaque Codex-only key** | **PASS** | `prompt_cache_key_opaque_stable_and_omits_without_identity` |
| **M7 / AUD-010 model cache** | **PASS** | `resolve_after_fetch_auth_error_does_not_serve_stale`; atomic save |
| **AUD-011** capability/binding Codex gate | **PASS** | `classify_codex_backend_prefers_binding_over_url`; sampler `is_codex_backend()` + `x-goblin-provider-id`; hoist non-text policy `SYSTEM_HOIST_NON_TEXT_POLICY` / `hoist_counts_non_text_system_parts_dropped` |
| **PC10** compaction prompt_cache_key | **PASS** | `compaction_prompt_cache_key_policy_codex_only_opaque`; `session_compact` Responses path sets `prompt_cache_key_for_compaction` |
| **P4** title routing | **PASS** | `title_inference_blocks_xai_when_codex_pinned`; `generate_session_summary` skips LLM when Codex pin + xAI base_url |
| **PC8 live cache** | **PASS (live, 2026-07-17)** | turn1 cached=0, turn2 cached=**17920**; durable redacted note: [`.llms/evidence/pc8-live-2026-07-17.md`](.llms/evidence/pc8-live-2026-07-17.md) |

Prompt cache mental model: **backend owns cache**; client passes stable prefix + optional opaque affinity key + observes `cache_read_input_tokens` / `cached_tokens`. Live multi-turn hit for Codex luna was proven once (2026-07-17) and recorded in `.llms/evidence/`.

**Production-ready claim scope:** single-machine offline Codex path (dual-manager flock refresh, journal, wire, pin, catalog, capability gate, compaction key, title guard) **plus** the one-shot live PC8 evidence above. **Not** claimed: multi-host refresh, dual OS-process spawn test, multi-provider 1.0 (keyring/xAI/D10), continuous CI live cache without credentials.

---

## Still open (EXTERNAL / 1.0 deferrals only)

| ID | Gap | Class |
|----|-----|--------|
| **AUD-012 PC8** | Live multi-turn cache hit | **CLOSED (PASS)** — see offline matrix + `pc8-live.md` |
| **True multi-OS-process spawn test** | Two OS processes (not only dual TokenManager + flock) | **DEFERRED** — multi-manager+flock proven |
| **R1–R6** | xAI adapter, keyring, composition root, subagent isolation, D10 product | **1.0 multi-provider** (out of offline Codex path) |
| **P1–P3, P5–P7** | TUI login polish, accounts CLI, /model, user docs, CI live job | **Product UX** (non-blocking offline path) |

---

## Suggested README blurb

> Goblin includes a multi-provider Codex path: credential-scoped models, request-time OAuth bearer with attempt-bound stamps and dual-manager refresh flock, OpenResponses phase/commentary + function-call wire siblings, capability/binding-aware Codex gate, opaque `prompt_cache_key` (incl. compaction), and per-credential model catalog cache.
> One-shot **live prompt-cache hit** evidence is in `.llms/evidence/pc8-live-2026-07-17.md` (not a standing CI job). Login is fail-closed / opt-in (D10). See `TO_RELEASE.md`.

---

## Tracking
