# Codex Multi-Provider Validation Relaunch — 2026-07-17

**Auditor:** primary session (local static + cargo) — GLM/Kimi subagents hit Cloudflare RPM 429; validation completed without subagent dependency.
**Repo:** `/home/guilherme/github/grok-goblin` · branch `goblin-multi-provider-codex`
**Evidence log:** `/tmp/grok-goal-7c2bf1e0a316/implementer/verification-plan-full.log`
**Related:** `.llms/reviews/codex-aud003-hotpath-review-2026-07-17.md` (Kimi earlier: AUD-003 PASS)

---

## 1. Executive verdict

| Criterion | Verdict |
|-----------|---------|
| 1. Concurrent refresh single-flight + attempt stamps | **PASS** |
| 2. Journal recover under lock, fail-loud | **PASS** |
| 3. AUD-003 FC sibling / OpaqueWire fail-loud / no double FC | **PASS** |
| 4. Model cache M7 (atomic, 401≠stale, source) | **PASS** |
| 5. PC8 live honesty | **PASS (NOT claimed)** |

**One-line:** Offline production gates for Codex multi-provider are met with tests green; live prompt-cache hit remains honestly NOT claimed. Mixed-history FC residual was fixed (per-`call_id` skip of dual-write projections; see `create_response_mixed_legacy_and_sibling_fc_preserves_both_once`).

---

## 2. Criterion evidence

### 1 — Refresh single-flight + attempt stamps — **PASS**

| Item | Evidence |
|------|----------|
| Cross-process flock on refresh | `token_manager.rs:135,243` — `acquire_lock(..., CredentialLockPurpose::Refresh)` in `get_valid_token` and `recover_unauthorized` |
| Shared manager atomic entry | `token_resolve.rs` — `SHARED_MANAGERS.entry` (DashMap) |
| Multi-manager single refresh | Test `two_managers_same_file_home_one_refresh_via_xproc_lock` **ok** |
| In-process 50 concurrent | `refresh_single_flight_50_concurrent_one_refresh` **ok** |
| Attempt stamp FIFO + peek | `request_stamp` 5 tests **ok**; sampler `auth_info_and_prefix_use_peek_bearer_not_send` **ok** |

### 2 — Journal fail-loud — **PASS**

| Item | Evidence |
|------|----------|
| No silent recover in `new` | `file.rs:44` — `new` only builds paths/lock; no `recover_pending_txn` |
| Lazy under lock | `file.rs:58-68` `ensure_journal_recovered`; mutations call `recover_pending_txn` under write+flock |
| Corrupt quarantine | `metadata.rs:182+` + tests `corrupt_journal_quarantines_and_fails_loud`, `store_load_recovers_pending_journal_lazily` **ok** |
| Journal suite | `credential_scoped_and_recover` **6/6 ok** |

### 3 — AUD-003 wire / hot path — **PASS**

| Item | Evidence |
|------|----------|
| Production resend path | sampler `client.rs` `(&request).into()` → `From<&ConversationRequest>` → `build_responses_input` |
| Fail-loud OpaqueWire | `conversation.rs:2621-2626` panics `AUD-003 fail-loud, no silent drop` on `try_build_responses_input` Err |
| Soft path | `try_build_responses_input` (`:2634`) → `conversation_items_to_responses_input` Err for unmapped opaque |
| No silent filter | No `filter OpaqueWire` fallback remains (grep clean) |
| No double FC | `conversation_items_to_responses_input` `:2906` `has_fc_siblings` → Assistant content only (`:2912`) |
| Hot-path tests | `create_response_from_request_fails_loud_on_opaque_wire` (should_panic) **ok**; `create_response_from_request_single_fc_sibling_order` **ok**; `try_build_responses_input_errs_on_opaque_no_silent_filter` **ok** |
| Convert fixtures | `convert_resend_preserves_function_call_sibling_order`, `convert_preserves_mcp_and_opaque_unknown` **ok** |

**Mixed-history residual:** **FIXED** after review flag — skip dual-write projections by matching `call_id` against FunctionCall siblings only (not a global `has_fc_siblings` gate). Test: `create_response_mixed_legacy_and_sibling_fc_preserves_both_once` (hot path `From` / CreateResponse).

### 4 — Model catalog M7 — **PASS**

| Item | Evidence |
|------|----------|
| File size | `model_cache.rs` **12909** bytes (non-empty) |
| Atomic write | `save_cache_inner` tmp+rename + unix 0o600 |
| Auth ≠ stale | `is_auth_or_identity_error` + `resolve_after_fetch` AuthFailure path |
| Source on catalog | `into_model_catalog(source)` / `ModelCatalogSource` |
| Tests | **8/8** model_cache tests **ok** |

### 5 — PC8 honesty — **PASS (NOT claimed)**

| Item | Evidence |
|------|----------|
| TO_RELEASE | `PC8 live cache | **NOT claimed**` |
| Scratch | `pc8-skipped.md` — `GROK_LIVE_CODEX` unset |
| Mental model | provider-managed cache; offline does not prove hits |

---

## 3. Live cargo results (this relaunch)

From `verification-plan-full.log` (2026-07-16T22:42):

| Suite / filter | Result |
|----------------|--------|
| two_managers_same_file_home | ok |
| request_stamp (5) | ok |
| refresh_single_flight_50 | ok |
| credential_scoped_and_recover (6) | ok |
| create_response_from_request (2) | ok |
| try_build_responses_input opaque | ok |
| convert FC + MCP | ok |
| session_pin | ok |
| model_cache (8) | ok |
| Experimental offline filters (phase/materialize/key/peek/commentary) | ok |
| xai-grok-sampling-types --lib | **290** ok |
| xai-grok-sampler --lib | **157** ok |
| xai-grok-multi-auth --lib | **38** ok |
| cargo check -p xai-grok-shell | Finished ok |

---

## 4. Claims allowed / forbidden

| Claim | Allowed? |
|-------|----------|
| Offline production gates (auth+journal+wire+pin+catalog) PASS | **Yes** |
| Experimental offline invariants PASS | **Yes** |
| Prompt cache hits proven / PC8 | **No** |
| Multi-OS-process spawn in CI | **No** (proven: multi TokenManager + file flock) |
| Multi-provider product 1.0 complete | **No** |

---

## 5. Subagent note

GLM/Kimi subagent launches repeatedly hit Cloudflare **429 rate limit**. This report is the durable substitute, plus local cargo evidence under implementer scratch.
