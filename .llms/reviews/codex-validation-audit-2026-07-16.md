# Codex Multi-Provider Validation Audit — 2026-07-16 (Wave 1+3 re-review)

**Auditor:** read-only validation harness (Grok Build review subagent)
**Repo:** `/home/guilherme/github/grok-goblin` · branch `goblin-multi-provider-codex`
**Mode:** read-only; no product code modified. No shell-execution tool available
in this harness, so `cargo test`/`cargo check` commands were **not executed**
here — see §9. Static code + test-definition evidence cited with file:line.
**Normative sources:** `CODEX_AUDIT_REMEDIATION_PLAN.md`, `CODEX_100_PERCENT_GOAL.md`,
`TO_RELEASE.md`, `.llms/reviews/code-audit-grok-goblin-codex-provider-2026-07-16.md`.

---

## 1. Executive verdict

| Question | Answer |
|----------|--------|
| Is Codex **100%** (per goal definition)? | **NO.** Wire W1/W2 + A1-attempt + A4-pin + opaque-PC-key offline gates are met, but PC8 live proof, cross-process refresh (A2/R5), journal safety (A3), FC-as-sibling (AUD-003), and model-cache hardening (AUD-010) remain open. |
| Is it **production-ready**? | **NO.** Cross-process refresh lock, journal fail-loud recovery, and live cache proof are explicitly required P0 gates for production and are unmet. |
| Is it **Experimental-ready**? | **YES (offline gates proven).** Wave 1 (W1 phase 1:1, W2 materialize-by-index + FC args) + Wave 3 (A1 attempt stamp FIFO, A4 binding pin reuse, opaque capability-gated PC key) offline gates pass by static test evidence. Shipable as Experimental **without** live-cache or cross-process claims. |

**One-line:** The critical Wave 1+3 offline invariants (phase correlation, stream materialize, attempt-bound auth stamp, opaque cache key) are genuinely fixed with tests; the production blockers (cross-process refresh, journal safety, live cache proof, FC sibling fidelity, model-cache hardening) remain honestly open and are correctly documented as such in `TO_RELEASE.md`.

---

## 2. What works (proven with test evidence)

| Item | Evidence (file:line) | Test |
|------|----------------------|------|
| **AUD-001 phase 1:1 (incl. None slots)** | `conversation.rs:2278-2312` — collects `Vec<Option<AssistantPhase>>` preserving None; loop increments `phase_idx` for every assistant, writes phase only when `Some`. | `patch_assistant_phases_preserves_none_slots_no_sliding` (`:4303`); `patch_assistant_phases_on_request_body_injects_wire_phase` (`:4351`); `resend_preserves_phase_via_patch_and_reasoning_order` (`:4386`) asserts byte-stable resend. |
| **AUD-002 materialize by index + FC arg deltas** | `conversation.rs:2323-2348` — merges by `output_index`, stream wins per-index, completed fills gaps. `append_function_call_arguments_delta` (`:2352`) + `set_function_call_arguments_done` (`:2366`) accumulate into the materialize map. Wired in `stream/responses.rs:299,319,339,359,575`. | `materialize_response_output_prefers_stream_when_completed_empty`; `materialize_response_output_prefers_stream_when_index_present`; `materialize_merges_stream_and_completed_by_index` (`:4189-4232`); `append_function_call_arguments_delta_builds_args` (`:4236`). |
| **AUD-004 opaque key, no anonymous** | `conversation.rs:2208-2238` — `derive_prompt_cache_key` returns `None` without identity (no `goblin-sess-anonymous`), emits opaque `gpc_<sha256>`, never embeds raw ids. `prompt_cache_key_log_label` redacts to 8 hex. | `prompt_cache_key_opaque_stable_and_omits_without_identity` (`:4265`) asserts: stable, `gpc_` prefix, no raw session id, `None` without identity, log label redacted. |
| **AUD-005 capability-gated Codex-only** | `client.rs:1927-1930,1969-1972` — `ensure_prompt_cache_key_for_codex` only when `is_codex_responses_backend(&self.base_url)`; else explicitly sets `prompt_cache_key = None`. | Static: non-Codex path zeroes the key. (No dedicated non-Codex regression test found — minor gap.) |
| **AUD-006 attempt-bound stamp (A1)** | `request_stamp.rs` — `AttemptStampLedger` records per-attempt id, FIFO `take_for_recovery`, explicit `take_attempt(id)`. `MultiProviderBearerResolver` (`multi_provider_resolve.rs:466-481`): `current_bearer` → `resolve_attempt` (records stamp); `peek_bearer` → `last_token`/`resolve_token_no_stamp` (no stamp). | `same_ledger_sequential_resolves_recovery_uses_attempt_order` (`:148`); `explicit_attempt_id_survives_later_resolves` (`:168`); `peek_then_send_order_recovery_matches_send_attempt` (`:194`); sampler `auth_info_and_prefix_use_peek_bearer_not_send` (`client.rs:2461`) asserts auth_info calls peek (0 sends), post calls current (1 send, 0 peeks). |
| **AUD-009 binding pin reuse (A4)** | `sampler_turn.rs:318-345` — `reconstruct_full_config` keeps pinned `MultiProviderSessionAuth` when same credential+provider; replaces only on account switch; keeps pin when hints lost. | Static: pin-continuity branch preserves resolver + stamp ledger. (Test coverage is structural; no dedicated "reconstruct does not overwrite pin" unit test found — minor gap.) |
| **C2 commentary→Reasoning routing** | `stream/responses.rs:785` — `commentary_phase_routes_to_reasoning_then_final_to_text` asserts commentary text → Reasoning channel, final → Text. | Test present. |
| **PC7 cached_tokens parse** | `stream/responses.rs:560` — `cached_prompt_tokens: u.input_tokens_details.cached_tokens`. | Static; usage parse present. |
| **Title gen no longer anonymous** | `session_summary.rs:79-100` — title request has no `with_session_id`/agent id → `derive_prompt_cache_key` returns `None` → key omitted (no anonymous shared). | Static (PC11 title path safe by construction). |

---

## 3. What is partial

| Item | Status | Evidence |
|------|--------|----------|
| **AUD-003 FC as sibling / MCP / unknown** | **PARTIAL** | `response_to_conversation_items_with_phases` (`conversation.rs:2086-2105`) still attaches `FunctionCall` to the most recent assistant's `tool_calls`; `McpCall` only increments `backend_tool_count` and is dropped (`:2130-2132`); `_ => {}` silently drops unknown variants (`:2133`). FC **args** now materialize (AUD-002 fixed), but FC is not a true wire sibling and MCP/unknown are not preserved opaquely. |
| **AUD-009 binding** | **PARTIAL** | Pin reuse works when same credential; but `reconstruct_full_config` still calls `session_auth_for_sampling_hints` (derives from model id/base_url/headers) on every reconstruct and the `(None, Some(auth))` branch installs a hint-derived binding. Hints remain a fallback source of identity, not purely derived state. |
| **AUD-010 model cache** | **PARTIAL** | TTL/per-credential path/`CacheSource` enum (Network/FreshDisk/StaleDisk/Bundled) exist (`model_cache.rs:159`). But: `save_cache` uses non-atomic `std::fs::write`, ignores errors, no 0600 (`:108-112`); fetch always returns `etag: None` (`models.rs:108`), never sends `If-None-Match`; any `Err` → stale/bundled with no 401/403/identity distinction (`:142-154`); `into_model_catalog` drops `stale`/`from_bundled`/`source` (`:48-55`). |
| **AUD-011 hoist URL gate** | **PARTIAL** | `is_codex_responses_backend` (`conversation.rs:2544`) is still URL-substring (`chatgpt.com` / `/codex`); hoist is textual-only, non-text system/developer content discarded. Capability-typed gate not implemented. |
| **PC7 observability** | **PARTIAL** | `cached_tokens` parsed + logged in stream; not proven propagated to all surfaces (headless JSON usage event, session usage) end-to-end. |

---

## 4. What is still open (P0/P1) with impact

### P0 — blocks production-ready

| ID | Gap | Impact if shipped as-is |
|----|-----|--------------------------|
| **AUD-007** | `make_store_and_manager` (`token_resolve.rs:35-52`) does `get` then `insert` (check-then-act race, not `DashMap::entry`); refresh uses only in-process Tokio locks, no `acquire_lock` cross-process around refresh/401-recovery. | Two processes can simultaneously consume the same rotating refresh token; one invalidates the other → auth failure / account lockout under multi-process. |
| **AUD-008** | `FileCredentialStore::new` (`file.rs:43`) calls `let _ = recover_pending_txn(&paths)` in the constructor before any lock, error ignored; journal removal also ignores errors (`metadata.rs:176,179`). | Crash-recovery can run concurrently across processes; corruption/permission errors stay invisible; metadata/secret can diverge. |
| **AUD-012 / PC8** | No gated live probe: turn-2 `cached_tokens > 0` + negative control never produced. | All components can compile and the product may never obtain a cache hit; "cache complete" claim unproven. |
| **AUD-003 residual** | FC not a true wire sibling; MCP/unknown dropped silently. | Replay order `commentary→FC→final` not faithfully preserved on resend; MCP calls lost from history; cache prefix can shift. |

### P1 — product/OSS UX (blocks Beta, not Experimental)

| ID | Gap |
|----|-----|
| **AUD-010** | Model cache non-atomic, no ETag, no 401/403 policy, source lost in public contract. |
| **AUD-011** | Hoist URL-gate + lossy non-text; capability-typed gate missing. |
| **P4** | Title gen must not hit xAI proxy on Codex sessions (now safe re: key, but routing not verified). |
| **P6/P7** | User docs + CI live gate job. |
| **PC10** | Compaction key generation policy absent. |

---

## 5. AUD-001..012 status matrix

| ID | Severity | Wave1+3 scope? | Status | Evidence |
|----|----------|----------------|--------|----------|
| **001** phase mis-correlation | Critical | Wave 3 (3.1) | **CONFIRMED FIXED** | `conversation.rs:2278-2312` preserves None slots; tests `:4303,:4351,:4386`. |
| **002** materialize by size / FC args lost | Critical | Wave 3 (3.2) | **CONFIRMED FIXED** | `conversation.rs:2323-2375` merge-by-index + FC delta/done; wired `stream/responses.rs:339,359,575`; tests `:4189-4260`. |
| **003** FC attached to assistant / MCP drop / `_=>{}` | High | Wave 3 (3.3-3.4) | **PARTIAL** | FC args fixed (AUD-002); FC still attached to last assistant (`:2086-2105`); MCP dropped (`:2130`); unknown `_=>{}` (`:2133`). |
| **004** anonymous key + raw id leak | High | Wave 1 (1.5) | **CONFIRMED FIXED** | `conversation.rs:2208-2238` opaque `gpc_<sha256>`, None without identity, redacted log; test `:4265`. |
| **005** key on all Responses backends | High | Wave 1 (1.6) | **CONFIRMED FIXED** | `client.rs:1927-1930,1969-1972` gates by `is_codex_responses_backend`, zeroes non-Codex. (No non-Codex regression test — minor.) |
| **006** request stamp last-wins | Critical | Wave 1 (1.1-1.2) | **CONFIRMED FIXED** | `request_stamp.rs` `AttemptStampLedger` FIFO + explicit id; `multi_provider_resolve.rs:466-481` peek vs current split; tests `:148,:168,:194`; sampler `:2461`. |
| **007** single-flight not cross-process / DashMap race | Critical | Wave 2 (2.1-2.3) | **STILL OPEN** (not Wave1+3) | `token_resolve.rs:35-52` get+insert race; no cross-process lock. |
| **008** journal recovery unsafe / fail-silent | High | Wave 2 (2.4-2.5) | **STILL OPEN** (not Wave1+3) | `file.rs:43` `let _ = recover_pending_txn` in `new`; `metadata.rs:176,179` ignore removal errors. |
| **009** binding from fragile hints | High | Wave 1 (1.3-1.4) | **PARTIAL** | `sampler_turn.rs:318-345` pin reuse on same credential; but hints still fallback source on `(None, Some(auth))`. |
| **010** model cache fragile | Med/High | Wave 5 (not Wave1+3) | **PARTIAL** | `model_cache.rs:108-112` non-atomic write; `models.rs:108` etag None; `:142-154` no 401/403 policy; `:48-55` source dropped. |
| **011** hoist lossy + URL gate | High | Wave 3 (3.5) | **PARTIAL** | `conversation.rs:2544` URL-substring gate; hoist textual-only. Capability-typed gate not implemented. |
| **012** no live PC8 proof | High (release gate) | Wave 4 (4.6) | **STILL OPEN** (not Wave1+3) | No gated live probe found. |

---

## 6. Wave1/Wave3 gate status vs plan acceptance criteria

| Gate (plan §2) | Criterion | Status | Evidence |
|----------------|-----------|--------|----------|
| **W1 Wire fidelity** | Round-trip: order, phase per item (incl. None intercalated), reasoning, FC with args, custom/web/code, refusal; unknown preserved or explicit error | **PASS (phase/None/reasoning/FC-args); PARTIAL (FC sibling, MCP/unknown drop)** | AUD-001 fixed, AUD-002 fixed, AUD-003 partial. |
| **W2 Stream materialize** | Merge by output_index/id; FC args delta/done; completed partial vs stream; never silent drop | **PASS** | `materialize_response_output` + FC delta/done + tests. |
| **A1 Attempt stamp** | 2 concurrent resolves same resolver → 2 stamps; 401 of attempt N uses stamp N | **PASS** | `AttemptStampLedger` FIFO + `peek_then_send_order_recovery_matches_send_attempt`. |
| **A4 Binding** | ModelBinding persisted; rebuild does not re-derive from header/URL if pinned | **PARTIAL** | Pin reuse on same credential; hints remain fallback. |
| **PC key** | Opaque (hash), capability-gated Codex-only, no anonymous global; title/subagent distinct or omitted | **PASS** | `derive_prompt_cache_key` opaque/None-without-identity; gated in sampler; title omits. |
| **A2+R5 Refresh** | 2 processes: 1 refresh; journal recovery under lock fail-loud | **NOT CLAIMED** (Wave 2) | Open. |
| **PC live** | Turn2 cached_tokens>0 + negative control | **NOT CLAIMED** (Wave 4) | Open. |
| **M7 catalog** | Atomic write, 401/403≠stale, ETag, source in contract | **NOT CLAIMED** (Wave 5) | Partial. |

**Experimental milestone (plan §2):** W1+W2+A4+PC key opaque+capability + A1 attempt stamp, without cross-process and without live cache claim → **MET** (with A4/PC-key minor caveats).

---

## 7. Risks if shipped as-is (Experimental tier)

| Risk | Likelihood | Mitigation present |
|------|-----------|--------------------|
| 401 recovery uses wrong generation stamp under concurrency | **Low** (A1 fixed with FIFO + peek/current split) | Yes — tests prove attempt-order recovery. |
| Prompt cache never hits (no live proof) | **Unknown** | Documented as not-claimed; README/TO_RELEASE honest. |
| FC/MCP/unknown lost on replay → degraded multi-turn tool loops | **Medium** | Partially mitigated by FC args materialize; full sibling fidelity open. |
| Cross-process refresh token race | **N/A for single-process Experimental** | Documented limitation; must not claim multi-process. |
| Model catalog corruption/401-masked-as-stale | **Low-Medium** | Bundled fallback prevents crash; but 401 can be masked. |
| URL-substring Codex gate false-positive on custom endpoints | **Low** | Documented; capability gate is P1. |

---

## 8. Recommended next waves (ordered)

1. **Wave 2 — AUD-007/008 (P0 production):** `DashMap::entry` for `make_store_and_manager`; cross-process `acquire_lock` around refresh/401-recovery with CAS; lazy journal recovery under lock, fail-loud/quarantine, durable removal with error propagation. Multi-thread (and ideally multi-process) single-refresh test.
2. **Wave 3 residual — AUD-003 (P0 wire):** Model FC as a true wire sibling (or canonical wire-history separate from UI projection); opaque `WireItem` store for MCP/unknown with explicit reject path; no silent `_ => {}`.
3. **Wave 4 — AUD-012/PC8 (P0 release gate):** Gated live probe (`GROK_LIVE_CODEX=1`): turn1/turn2 stable prefix, same credential+key, assert `cached_tokens > 0`, negative control by mutating early history, redacted SCRATCH artifact.
4. **Wave 5 — AUD-010 (P1):** Atomic `tmp`+rename write, mode 0600, logged errors; `If-None-Match` + real ETag; typed error policy (401/403/identity → not stale; 5xx/timeout → stale ok); `source` in public `ModelCatalog` contract; versioned bundled fallback.
5. **AUD-011 (P1):** Capability-typed Codex gate replacing URL-substring; explicit policy for non-text system/developer content.
6. **AUD-009 residual (P1):** Make `ModelBinding` authoritative; hints only when no pin exists, never overwriting a live pin.
7. **P4/P6/P7:** Title routing on Codex; user docs; CI live gate job.

---

## 9. Commands run and results

**No `cargo` commands were executed by this auditor.** This review harness
has no shell/command-execution tool available. The mission-specified commands:

```
cargo test -p xai-grok-sampling-types --lib preserves_none_slots
cargo test -p xai-grok-sampling-types --lib materialize
cargo test -p xai-grok-sampling-types --lib prompt_cache_key_opaque
cargo test -p xai-grok-sampling-types --lib append_function
cargo test -p xai-grok-multi-auth --lib request_stamp
cargo test -p xai-grok-sampler --lib auth_info_and_prefix_use_peek
cargo test -p xai-grok-sampler --lib commentary_phase
cargo test -p xai-grok-multi-auth --lib --quiet
cargo test -p xai-grok-sampling-types --lib --quiet
cargo test -p xai-grok-sampler --lib --quiet
cargo check -p xai-grok-shell 2>&1 | tail -20
```

were **not run**. Evidence is instead drawn from:
- static reading of test function definitions and their assertions (file:line cited per finding);
- prior documented test outcomes in `TO_RELEASE.md` (which records Wave1+3 offline gates as PASS with named tests);
- the remediation plan's validation table.

**Honest limitation:** static test presence proves the test exists and asserts
the intended invariant; it does **not** prove the test currently compiles/pass
on this exact worktree. A subsequent run of the commands above by an
executor with shell access is required to convert "test exists + asserts X"
into "test passes". The `TO_RELEASE.md` record of prior PASS is treated as
weak evidence, not proof, per the audit's own caution about uncommitted
worktree state.

---

## 10. A1 skeptic gap — re-check (peek vs current split)

**Question:** Confirm `peek_bearer` vs `current_bearer` split exists on
`MultiProviderBearerResolver`, `SamplingClient` uses peek for auth_info/prefix
and current for post, and no remaining call path records a stamp on peek-only use.

**Findings:**

1. **Trait contract** (`sampler/src/config.rs:165-181`): `BearerResolver::current_bearer` documents "record an attempt stamp here"; `peek_bearer` default delegates to `current_bearer` but multi-provider overrides to avoid stamp pollution.

2. **MultiProviderBearerResolver** (`multi_provider_resolve.rs:466-481`):
   - `current_bearer` → `resolve_attempt` (`:432`) which calls `self.stamps.record(stamp)` (`:438`). ✅ records.
   - `peek_bearer` → reads `last_token` mutex, else `resolve_token_no_stamp` (`:404`) which resolves token+stamp but **does not** call `stamps.record`. ✅ does not record.

3. **SamplingClient** (`client.rs`):
   - `post` (`:582-585`): `resolver.current_bearer()` → records stamp. ✅
   - `current_sent_bearer_prefix` / `auth_info` (`:631-636,691-700`): `resolver.peek_bearer()` → no stamp. ✅
   - Test `auth_info_and_prefix_use_peek_bearer_not_send` (`:2461`): asserts auth_info → 1 peek, 0 sends; post → 1 send, 0 peeks. ✅

4. **No remaining peek-only path records a stamp:** `resolve_token_no_stamp` (`:404-426`) updates `last_token` only; `last_stamp()` (`:382`) is a read-only peek helper; `take_stamp_for_recovery`/`take_last_stamp` consume from the ledger without recording. The only `stamps.record` call site is `resolve_attempt` (`:438`), reachable only via `current_bearer`.

**Verdict: A1 skeptic gap CLOSED.** The peek/current split is real, enforced by
the trait contract, implemented on the resolver, consumed correctly by the
client, and guarded by a test that asserts the send/peek counts. No call path
records a recovery stamp on a peek-only resolve.

---

## 11. Prompt cache mental model (honest)

Per the audit's correct framing and `TO_RELEASE.md`: prompt cache is
**provider-managed**. The client's job is to (a) pass a stable prefix, (b) pass
an opaque affinity `prompt_cache_key` when identity is present, (c) observe
`cached_tokens`. The client does **not** prove cache hits offline. No live
`cached_tokens > 0` evidence exists in this worktree — do not claim it.

---

## 12. Conclusion

**PASS for Experimental-tier offline gates (Wave 1 + Wave 3):** the critical
fixes the remediation plan targeted — phase 1:1 correlation (AUD-001),
stream materialize by index with FC arg accumulation (AUD-002), attempt-bound
auth stamp with peek/current split (AUD-006), opaque capability-gated cache
key (AUD-004/005), and binding pin reuse (AUD-009 partial) — are implemented
with tests that assert the intended invariants.

**FAIL for production-ready / "100%":** cross-process refresh (AUD-007),
journal safety (AUD-008), live cache proof (AUD-012/PC8), FC sibling fidelity
(AUD-003 residual), and model-cache hardening (AUD-010) remain open and are
correctly documented as not-claimed in `TO_RELEASE.md`.

**Residual risk of this audit:** test outcomes are static (test exists +
asserts X), not dynamically executed. An executor with shell access should
run the §9 commands to convert static evidence into passing-test proof before
any release claim.
