---
title: "Multi-provider auth readiness follow-up"
date: "2026-07-16"
status: "relevant"
verdict: "BLOCKERS_PASS — product MAJORs still DEFERRED"
---

# Follow-up after 2026-07-16 implementation review

## Prior verdict

`2026-07-16-implementation-review.md`: **NOT READY** (B1–B6, M1–M14).

## Current verdict

| Scope | Verdict |
| --- | --- |
| Review **BLOCKERS B1–B6** | **PASS** (see readiness matrix) |
| Full multi-provider product (all MAJORs + skill A–L) | **NOT READY** — DEFERRED majors remain |
| Codex vertical slice (login gated, catalog, request auth, 401 once) | **Implemented with evidence** |

## What closed the blockers

1. **B1/B2:** Credential-scoped catalog keys; no OAuth in merge `api_key`; TokenManager request path via BearerResolver.
2. **B3:** Session sampler 401 → multi-provider `recover_unauthorized` → one resubmit.
3. **B4:** xAI stub no longer advertises false capabilities; legacy AuthManager remains product path.
4. **B5:** Codex **login** fail-closed without `GROK_CODEX_OAUTH_APPROVED` / `GROK_CODEX_CLIENT_ID`.
5. **B6:** Write-ahead `credential-txn.journal` for dual-file commits + recovery on store open.

## Explicitly still open (not claimed done)

M1, M2, M4–M13 (and skill breadth): registry-driven CLI, full grammar, keyring, model cache, full TUI, RequestAuthResolver as composition type, parent/subagent concurrency suite, live OAuth smoke.

## How to enable Codex login (dev)

```bash
export GROK_CODEX_OAUTH_APPROVED=1
# or: export GROK_CODEX_CLIENT_ID=...
goblin login --provider codex
```

Existing stored credentials work for models/inference without re-approval.

## Evidence artifacts

- Matrix: `2026-07-16-readiness-matrix.md`
- PROGRESS: single canonical table
- Tests: `cargo test -p xai-grok-auth -p xai-grok-multi-auth`
- Session scratch: `prod-auth-consumers.txt`, `multi-auth-tests.log`, `multi-account-tests.log`, `live-smoke.txt`
