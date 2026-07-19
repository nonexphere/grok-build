# Residual review — C5-B BYOK providers (OpenRouter / Groq / Cloudflare)

| Field | Value |
|---|---|
| Wave | C5-B (items 31–34, partial 36) |
| Mode | implementation review (residual) |
| Reviewer | review harness (read-only, glm-5.2) |
| Date | 2026-07-19 |
| Branch | `goblin-implement-epic-tree` |

## Verdict

**PASS_WITH_FINDINGS**

The offline contract + registry + login + CLI parse slice is REAL and proven.
The composition-root Turn binding is honestly PARTIAL (depends on C1-G/C1-J),
and the wave explicitly does NOT claim item 37 (composition root) done. No
secrets leak; no live smoke is claimed PASS on skip. Findings are Medium/Low.

## Severity summary

- Critical: 0
- High: 0
- Medium: 2 (F-2, F-4)
- Low: 2 (F-1, F-3)

## Contract non-negotiables (re-checked)

- **No second public binding type for wire.** The wave reuses the protocol
  `ProviderBinding`; the local `byok::PublicProviderBinding` (u64 revision)
  is left unreconciled and unused at the composition root — flagged as a
  follow-on, not introduced as a second wire binding. PASS (with debt noted).
- **No live secrets / no PASS on skip.** Synthetic `sk-test-*` only; live
  smoke deferred behind `RUN_LIVE_BYOK_<PROVIDER>=1` and never claimed PASS.
  Evidence: `byok_providers_green.txt` uses `sk-test-*` fixtures; README §
  "Live smoke — SKIP policy". PASS.
- **`run_api_key_login` honors registry + capability.** Rejects unknown
  (`byok_api_key_login_rejects_unknown_provider`) and non-API-key providers
  (`byok_api_key_login_rejects_provider_without_api_key_capability`). Closes
  the `login_coordinator.rs:211` `let _ = self.registry.get` foundation gap.
  PASS.
- **No second actor / Tower ≠ Shell / no Fake hybrid.** No edits to
  `app_server_runtime/**`, tower, or mcp-server; only multi-auth + a
  mechanical `Byok` match arm in pager-bin `main.rs`. PASS.
- **Secret non-leakage.** `byok_api_key_login_persists_for_registered_byok_providers`
  asserts metadata `Debug` does not leak the key; request-auth tests assert
  only `Authorization: Bearer <opaque>` carries the secret. PASS.

## Evidence reviewed

- Wave note: `.llms/execution/app-server-mcp-tower-corrective/waves/c5-byok-providers.md`
- Handoff: `.llms/.../handoffs/HANDOFF-C5-B-byok-providers.md`
- GREEN: `.llms/.../tests/c5/byok_providers_green.txt` (17/17 pass).
- Full suite: `.llms/.../tests/c5/full_suite_green.txt` (89 passed; 0 failed
  across lib + 6 test binaries).
- RED evidence summary: `.llms/.../tests/c5/README.md` (pre-change 7 compile
  errors + assertion failures against the registry-ignoring login path).

## Findings

### F-1 — `SecretBackendKind::Ephemeral` recorded for persisted BYOK keys (Low, high confidence)
`run_api_key_login` records `SecretBackendKind::Ephemeral` in the
`NewCredentialRecord`, even though the pager-bin path persists via
`FileCredentialStore`. The wave documents this as a follow-on File-vs-Keyring
backend policy decision. The credential does land on disk in the pager-bin
path, but the recorded backend kind is dishonest relative to where the key
actually lives. Low severity (no security impact; metadata accuracy).

### F-2 — Composition-root Turn binding still `None` (Medium, high confidence)
The Shell actor still projects `provider_binding: None` for sessions
(`app_server_runtime/**`, owned by C1-G/C1-J). The end-to-end composition
test (login → persist → bind → resolve request auth → inference) is NOT
proven by this slice. The wave is honest about this and does not claim item
37. The `AuthProvider` seam is proven in isolation only. This is the
principal residual; closing it requires the C1-G/C1-J composition follow-on.

### F-3 — Interactive API-key prompt not implemented (Low, high confidence)
The pager-bin `Byok` arm reads `GROK_BYOK_API_KEY` (non-TTY) only; a TTY
`rpassword` prompt is a documented follow-on. Acceptable for an offline
slice; non-interactive env path is the contract surface.

### F-4 — `PublicProviderBinding` reconciliation debt (Medium, medium confidence)
The local `byok::PublicProviderBinding` (u64 revision) is unused at the
composition root and not reconciled to the protocol `ProviderBinding`
(`WireCounter`). Left unchanged. While not a second wire binding today, it
is dead/projection debt that could later be mis-wired as a second authority.
Recommend reconcile-or-delete in the C1-G/C5 follow-on.

### F-5 — Registry ordering / picker surface (Low, medium confidence)
BYOK providers register at `default_priority: 50` (below xAI 0 / Codex 10)
and now appear in `prompt_provider_selection`. Not a regression (BYOK was
absent before), but if BYOK should not appear in the default interactive
picker, a follow-on should gate it (mirroring `codex_oauth_login_allowed`).

## Required fixes

None for this wave's bounded scope.

## Residual risk / dependencies

- C1-G/C1-J must project a real `ProviderBinding` to close item 37 and the
  composition-root end-to-end test (OpenRouter is the simplest vertical).
- Decide File vs Keyring backend and reconcile or delete
  `PublicProviderBinding`.

## Commands / results

- `cargo test -p xai-grok-multi-auth --test byok_providers` → 17 passed; 0 failed (`byok_providers_green.txt`).
- `cargo test -p xai-grok-multi-auth` → 89 passed; 0 failed (`full_suite_green.txt`).
- `cargo check -p xai-grok-pager-bin` → clean (per wave note).
- `cargo clippy -p xai-grok-multi-auth --all-targets` → no new warnings in edited/new files.
