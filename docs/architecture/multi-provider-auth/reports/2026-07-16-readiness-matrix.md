---
title: "Multi-provider auth readiness matrix"
date: "2026-07-16"
status: "living"
verdict: "BLOCKERS_PASS_MAJORS_DEFERRED"
status_reason: "B1–B6 closed or dispositioned to PASS under documented product boundaries; many MAJORs remain DEFERRED. Product-complete READY for full multi-provider UX is not claimed."
last_reviewed_at: "2026-07-16"
---

# Readiness matrix (living)

Supersedes optimistic claims in older PROGRESS tables. Evidence from tree audit
2026-07-16 after vertical-slice + GATE 1 production 401 wiring + B5/B6.

**Legend:** PASS | FAIL | BLOCKED | N/A | DEFERRED

## BLOCKERS (B1–B6)

| ID | Item | Status | Evidence |
| --- | --- | --- | --- |
| **B1** | Production request path without static OAuth-in-`api_key` | **PASS** | Catalog merge `api_key: None`; `resolve_credentials` forces multi-provider `api_key: None` (no chat_state OAuth snapshot); BearerResolver + TokenManager only. |
| **B2** | Credential-scoped model identity | **PASS** | `codex/{credential_id}/{slug}`; test `two_accounts_same_slug_get_distinct_catalog_keys`. |
| **B3** | 401 recovery + refresh on real path | **PASS** | `RefreshMultiProviderAuthOnce` + per-turn flag (max 1 resubmit, not xAI MAX=3). Tests: same-gen→RetryAfterRefresh+CAS; stale no refresh; reauth permanent no loop. Live smoke **BLOCKED** (D10). |
| **B4** | xAI provider adapter | **PASS (legacy boundary)** | Stub keeps empty capabilities (no false PKCE/device ads); product xAI login remains legacy AuthManager. Not a full multi-provider xAI adapter; honest boundary per skill. |
| **B5** | Unapproved OAuth client fail-closed | **PASS** | Login requires `GROK_CODEX_OAUTH_APPROVED=1` or `GROK_CODEX_CLIENT_ID`; picker hides Codex otherwise. |
| **B6** | Crash-consistent secret+metadata refresh | **PASS** | `commit_accounts_and_secrets` journal + `recover_pending_txn` on store open; create/CAS-with-secret/delete use journal. Test `journal_recovers_dual_file_commit_after_crash_marker`. |

**Blocker gate for goal criterion 1–2:** B1–B6 **PASS**. Full product READY still limited by DEFERRED MAJORs / skill rows.

## MAJORS (M1–M14)

| ID | Status | Notes |
| --- | --- | --- |
| M1 Registry-driven CLI | DEFERRED | Still enum Xai/Codex |
| M2 Full CLI grammar | DEFERRED | Partial `--provider` |
| M3 Non-TTY picker | **PASS** | `prompt_provider_selection` errors if multi-choice and non-TTY |
| M4 Logout revoke scoped | DEFERRED | Bulk delete; no provider.logout on all paths |
| M5 Runtime feature gates | DEFERRED | Compile feature + env kill switches only |
| M6 Keyring-first | DEFERRED | File store default |
| M7 Account model cache | DEFERRED | Live fetch per catalog merge |
| M8 Generic RequestAuthResolver | DEFERRED | Codex endpoint still hard-coded in unit resolver; production uses TokenManager + headers |
| M9 Full TokenManager contract | DEFERRED | Core get/recover used; no subscribers/cross-process single-flight proof |
| M10 Registration errors | DEFERRED | Still `.ok()` on register |
| M11 Login alias | DEFERRED | No CLI alias pass-through |
| M12 TUI auth UX | DEFERRED | `/model` lists Codex; no login modal |
| M13 Broad test strategy | DEFERRED | Focused multi-auth green; no CLI/TUI PTY suite |
| M14 Ledger honesty | **PASS (this matrix + PROGRESS rewrite)** | |

## Skill checklist A–L (summary)

| Section | Status | Notes |
| --- | --- | --- |
| A Authorization | BLOCKED/FAIL rows | D10 client not human-approved; fail-closed login PASS |
| B Config/rollout | PARTIAL | Kill switches + OAuth approval env; no full runtime `[features]` |
| C Registry | PARTIAL | xAI stub honest caps; registration `.ok()` remains |
| D Login lifecycle | PARTIAL | Native Codex login exists; gated by B5; alias gap |
| E Storage | PARTIAL | File store; B6 journal DEFERRED; keyring DEFERRED |
| F Token manager | **PASS for core path** | resolve + recover + stamps; permanent-failure cache DEFERRED |
| G Request auth | **PASS production path** | BearerResolver + 401 recovery; not RequestAuthResolver type |
| H Models/binding | **PASS** | Credential-scoped keys; multi-account key test |
| I CLI/TUI/session | PARTIAL | Models list; TUI login DEFERRED |
| J Compatibility | PARTIAL | Legacy xAI path preserved; no full regression suite run |
| K Security/obs | PARTIAL | Redaction unit tests exist; more needed |
| L Test layers | PARTIAL | Unit/integration multi-auth; wire 401 live BLOCKED |

## Production consumer search (baseline)

See session scratch `prod-auth-consumers.txt`.

| Symbol | Production consumers |
| --- | --- |
| `token_resolve` / `MultiProviderBearerResolver` | shell `multi_provider_resolve`, `models` merge, `sampling_config_for_model`, `reconstruct_full_config` |
| `recover_unauthorized` (multi-auth) | `token_resolve` + session `sampler_turn` multi-provider branch |
| `RequestAuthResolver` | **tests only** (not composition root) |
| Catalog `api_key: Some(access_token)` | **absent** (removed) |

## Install

| Binary | Path | Notes |
| --- | --- | --- |
| pager | `~/.local/lib/goblin/goblin-pager` | Rebuild after this wave if shell/pager changed |
| wrapper | `~/.local/bin/goblin` | Prefers newer `target/release/goblin` |

## Live smoke

| Check | Status |
| --- | --- |
| Codex Responses live turn | **BLOCKED** (D10 / no approved client for automated smoke) |
| `goblin models` credential-scoped list | PASS when credentials exist and binary current |

## Explicit DEFER policy

Deferred items **must not** be marked `done` in PROGRESS. READY is forbidden while B4 or B6 is DEFERRED unless product accepts a narrower READY definition (not this goal).
