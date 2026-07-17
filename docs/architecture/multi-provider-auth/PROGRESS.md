# Multi-Provider Auth — Phase Ledger

**Source of truth for phase scope:** `task.md` §12.3
**Fork process:** `GOBLIN.md`
**Living readiness:** `reports/2026-07-16-readiness-matrix.md`
**Prior review:** `reports/2026-07-16-implementation-review.md` (NOT READY)

Status values: `pending` | `in_progress` | `partial` | `done` | `deferred` | `blocked`.

This file has **one** canonical empirical table. Older narrative wave sections that
contradict the table are historical only; the table wins.

---

## Canonical empirical status (2026-07-16)

| Wave / gate | Spec phase | Status | Evidence |
| --- | --- | --- | --- |
| 0 Protocol + D10 | 0 | `partial` | protocol-baseline exists; OAuth client **not** human-approved; product login **fail-closed** (B5) |
| 1 Control-plane types | 1 | `done` | `cargo test -p xai-grok-auth` |
| 2 Credential store | 2 | `partial` | file/ephemeral/CAS + **txn journal (B6)**; keyring still deferred |
| 3 TokenManager | 3 | `partial` | unit + **production** resolve/recover on session path |
| 4 xAI adapter | 4 | `done (legacy boundary)` | empty caps; AuthManager path; not multi-provider xAI |
| 5–6 Codex login/refresh | 5–6 | `partial` | login **fail-closed** unless approved env; refresh via TokenManager |
| 7 Inference/models | 7 | `partial` | credential-scoped catalog; BearerResolver; 401 recover; live smoke blocked |
| 8 CLI | 8 | `partial` | `--provider`, status, non-TTY picker safety; grammar incomplete |
| 9 TUI | 9 | `deferred` | `/model` lists Codex when binary current; no login modal |
| 10–11 Harden/rollout | 10–11 | `deferred` | kill switches + OAuth approval env only |

### Vertical slice checklist (Codex)

| Item | Status |
| --- | --- |
| Login → store (with `GROK_CODEX_OAUTH_APPROVED=1`) | works / gated |
| Catalog `codex/{credential_id}/{slug}`, no OAuth in merge `api_key` | **done** |
| Request-time TokenManager resolve | **done** |
| Session 401 → recover → one resubmit | **done** (code path) |
| Live Responses smoke | **blocked** (D10) |
| Two-account isolation tests (keys) | **done** (unit) |
| Concurrent parent/subagent multi-account | **deferred** |

### Tests (commands)

```bash
cargo test -p xai-grok-auth -p xai-grok-multi-auth --no-fail-fast
```

Includes: provider_model_key, credential_scoped_and_recover, TokenManager, login_e2e, store.

### Install

```bash
CARGO_BUILD_JOBS=1 cargo build -p xai-grok-pager-bin --bin goblin --release
install -m 755 target/release/goblin ~/.local/lib/goblin/goblin-pager
goblin models   # expect codex/{uuid}/… under Available models when logged in
```

Restart any long-lived TUI to pick up a new binary.

### Verdict

**BLOCKERS B1–B6 closed** (with B4 = legacy boundary).
**Not** full multi-provider product READY: MAJORs M1/M2/M4–M13 and skill breadth remain DEFERRED.
Usable path: approved-env Codex login → credential-scoped `/model` catalog → request-time token + one 401 recover.

---

## Historical wave notes

Implementation lives primarily in `crates/codegen/xai-grok-multi-auth` with thin
shell/pager hooks. Do not re-expand optimistic “done” claims for stub phases.

### 2026-07-16 — current-thread panic fix + short slug + effort

- `token_resolve::block_on_safe`: never `block_in_place` on CurrentThread (worker thread).
- Short slug `gpt-5.6-luna` resolves when one Codex account; multi-account ambiguous → clear error
  (`resolve_default_model_for_startup` fail-closed; non-startup reloads may still first-or-fallback after log).
- Codex catalog merge: reasoning effort menu low/medium/high/xhigh; CLI `--reasoning-effort` / `--effort`.
- **Fix:** `merge_codex_provider_models` runs **before** `stamp_reasoning_effort_overrides` so CLI
  `--effort high` overrides the merge default Medium (was stamped only on pre-merge catalog).
- Empirical (SCRATCH `/tmp/grok-goal-bc6cc9a08f49/implementer/`):
  - E1 `goblin models` lists `codex/{uuid}/gpt-5.6-luna` (and other Codex keys).
  - E2/E3 `--model gpt-5.6-luna` and full catalog key: no panic (headless os error 6 TTY; PTY exit 0).
  - E4 binary strings: `block_on_safe`, `single resubmit after TokenManager recover`, ambiguous-slug message.
  - Unit: `current_thread_no_panic` (3), `provider_model_key` slug (4), TokenManager 401 (4) green.
  - Integration: `cargo test -p xai-grok-shell --test codex_effort_after_merge` (2 passed) —
    `resolve_model_catalog` + inject merge → `--effort High` on Codex key + sampling_config.

### 2026-07-16 — Codex live turn: empty.output retry storm

- **Bug:** Codex SSE streams text, but `response.completed.output` is `[]`. Sampler
  classified `empty_response` / `no_visible_content` and retried up to 15× (UI stuck +
  `pongpong…`).
- **Fix:** recover Message from `output_item.done` + `inject_streaming_text_fallback`.
- **Empírico:** `goblin -p '…pong' --model gpt-5.6-luna` → single `pong`, **EXIT=0**,
  log `recovering from output_item.done recovered=1`, no new `inference_retry`.
  Binary `~/.local/lib/goblin/goblin-pager` mtime 2026-07-16 17:17.
