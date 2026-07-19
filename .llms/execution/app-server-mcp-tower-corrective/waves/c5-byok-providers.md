# Wave C5-B — BYOK Providers (OpenRouter / Groq / Cloudflare)

| Field | Value |
|---|---|
| Wave | C5-B (items 31–34, partial 36) |
| Agent | build (glm-5.2) |
| Branch | `goblin-implement-epic-tree` |
| Status | **REAL** for the offline contract + registry + login + CLI parse slice. **PARTIAL** for composition-root Turn binding (depends on C1-G). |
| No live credentials | synthetic `sk-test-*` only; live smoke deferred |

## Scope delivered

1. **Registered BYOK `AuthProvider`s** — OpenRouter, Groq, Cloudflare are now
   real `ByokAuthProvider` instances registered in
   `registry::build_default_registry` (and the test builders), each
   advertising `ProviderCapabilities::API_KEY_LOGIN` and only that bit.
   Gated by a new `GROK_DISABLE_BYOK_AUTH` kill switch
   (`kill_switch::byok_auth_disabled`).
2. **Hardened `run_api_key_login`** — now honors the registry:
   rejects unknown/unregistered ids (`ProviderError::InvalidConfig` from
   `ProviderRegistrationError`) and registered-but-non-API-key providers
   (xAI, Codex). Wires `byok::reject_xai_api_key_fallback` with the env
   source that supplied the secret. Closes the foundation gap
   (`login_coordinator.rs:211` previously did `let _ = self.registry.get`).
3. **Request auth + endpoint resolution** — `ByokAuthProvider` implements
   `build_request_auth` (static `Authorization: Bearer <opaque>`) and
   `resolve_endpoint` (OpenAI-compatible path for OpenRouter/Groq;
   account-id-in-path for Cloudflare, failing loudly when the account id is
   missing).
4. **Honest lifecycle** — `logout` never claims `remote_revoked` for a
   static key; `get_valid_token` returns the stored key without a refresh
   update; `list_models` returns an empty catalog with `Unknown` provenance
   (model discovery is the shell catalog's job in this slice, not the
   AuthProvider seam).
5. **CLI parse** — `parse_login_provider` now accepts
   `openrouter`/`groq`/`cloudflare` → `LoginProviderArg::Byok(id)`. The
   pager-bin `Command::Login` match wires the `Byok` arm to
   `run_api_key_login` against the durable file store (reads
   `GROK_BYOK_API_KEY` for non-TTY; interactive prompt is a follow-on).
6. **Offline contract tests** — 17 tests in `tests/byok_providers.rs` +
   4 new unit tests in `login_coordinator::api_key_login_tests` covering
   registry rejection, capability rejection, persistence, and secret
   non-leakage. RED→GREEN evidence under `tests/c5/`.

## Files changed

### Product code
- `crates/codegen/xai-grok-multi-auth/src/providers/byok/mod.rs` —
  re-export `ByokAuthProvider` + `resolve_byok_endpoint`; module doc.
- `crates/codegen/xai-grok-multi-auth/src/providers/byok/auth_provider.rs`
  — **new** `ByokAuthProvider` + `resolve_byok_endpoint`.
- `crates/codegen/xai-grok-multi-auth/src/registry.rs` — register the three
  BYOK verticals in `build_default_registry` / `build_registry` /
  `build_registry_with_codex_config`; add `build_registry_with_flags`.
- `crates/codegen/xai-grok-multi-auth/src/kill_switch.rs` —
  `byok_auth_disabled()` (`GROK_DISABLE_BYOK_AUTH`).
- `crates/codegen/xai-grok-multi-auth/src/login_coordinator.rs` —
  `run_api_key_login` honors registry + `API_KEY_LOGIN` capability + xAI
  fallback guard; updated existing in-crate tests to use the default
  registry; added rejection unit tests.
- `crates/codegen/xai-grok-multi-auth/src/cli.rs` —
  `LoginProviderArg::Byok(String)` + `parse_login_provider` accepts BYOK
  ids; extended `parse_login_provider_variants` test.
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — `Byok` match arm in
  `Command::Login` (mechanical update to keep the consumer building; wires
  to `run_api_key_login`).

### Tests
- `crates/codegen/xai-grok-multi-auth/tests/byok_providers.rs` — **new**
  17 offline contract tests.

### Ledger
- `.llms/execution/app-server-mcp-tower-corrective/tests/c5/README.md`
- `.llms/execution/app-server-mcp-tower-corrective/tests/c5/byok_providers_green.txt`
- `.llms/execution/app-server-mcp-tower-corrective/tests/c5/full_suite_green.txt`
- `.llms/execution/app-server-mcp-tower-corrective/waves/c5-byok-providers.md`
  (this file)

## Validation commands + results

| Command | Result |
|---|---|
| `cargo build -p xai-grok-multi-auth` | ok |
| `cargo test -p xai-grok-multi-auth --test byok_providers` | **17 passed; 0 failed** |
| `cargo test -p xai-grok-multi-auth` (full crate) | **89 passed; 0 failed** (lib 51 + byok_providers 17 + credential_scoped_and_recover 6 + current_thread_no_panic 3 + login_e2e 1 + multi_auth 7 + live_codex_models 4(skip) + doctests 0) |
| `cargo check -p xai-grok-pager-bin` | clean |
| `cargo clippy -p xai-grok-multi-auth --all-targets` | no new warnings in edited/new files |

## REAL vs PARTIAL

**REAL** (this slice):
- BYOK verticals are registered `AuthProvider`s with `API_KEY_LOGIN`.
- `run_api_key_login` honors the registry + capability contract.
- Request auth shape + endpoint resolution proven offline.
- CLI parse + pager-bin login wiring for BYOK ids.
- Offline contract tests prove the above.

**PARTIAL** (explicitly deferred, not claimed as done):
- **Composition-root Turn binding** — the Shell actor still projects
  `provider_binding: None` for sessions
  (`xai-grok-shell/src/app_server_runtime/**`, owned by C1-G). The
  end-to-end composition test (login → persist → bind → resolve request
  auth → inference through the production composition root) is therefore
  **not** proven by this slice. The skill's "Dead resolver" rule forbids
  calling a resolver complete without composition-root consumption; this
  wave does **not** claim item 37 (composition root) done. The resolver is
  proven in isolation against the `AuthProvider` trait, which is the
  contract seam — composition wiring is the C1-G follow-on.
- **File vs Ephemeral backend** — `run_api_key_login` still records
  `SecretBackendKind::Ephemeral` in the `NewCredentialRecord`; the
  credential is persisted via whatever store the coordinator was
  constructed with (`FileCredentialStore` in the pager-bin path, so it
  does land on disk). A deliberate File/Keyring backend policy for BYOK is
  a follow-on (see C5-A surface map §3 "Policy gap").
- **Interactive API-key prompt** — the pager-bin `Byok` arm reads
  `GROK_BYOK_API_KEY` (non-TTY). A TTY rpassword prompt is a follow-on.
- **Model discovery** — `ByokAuthProvider::list_models` returns an empty
  catalog; BYOK model catalogs are driven by the shell TOML/remote catalog,
  not the AuthProvider seam, in this slice.
- **Live smoke** — none added; deferred per SKIP policy
  (`RUN_LIVE_BYOK_<PROVIDER>=1` gate pattern documented in C5-A §6).

## Risks

- **Registry ordering** — BYOK providers are registered after xAI + Codex
  with `default_priority: 50` (Codex is 10, xAI is 0). The interactive
  picker sorts by `default_priority`, so BYOK appears below xAI/Codex.
  `prompt_provider_selection` now lists BYOK verticals in the picker; if
  that is not desired in the product surface, a follow-on should filter
  BYOK out of the default picker (mirroring the Codex `codex_oauth_login_allowed`
  gate). Not a regression: previously BYOK was absent from the picker
  entirely.
- **`PublicProviderBinding`** — the local `byok::PublicProviderBinding`
  projection (u64 revision) is still unused at the composition root and
  not reconciled to the protocol `ProviderBinding` (`WireCounter`). Left
  unchanged; reconciliation is a C1-G/C5 follow-on (C5-A surface map §4).
- **Env race** — the `byok_kill_switch_disables_registration` test uses
  `build_registry_with_flags` (flag-based) rather than env mutation to
  avoid racing concurrent tests that call `build_default_registry()`
  (which reads `GROK_DISABLE_BYOK_AUTH`).

## Non-overlap confirmation

No edits to C1-G's `xai-grok-shell/src/app_server_runtime/**`, C3's
`xai-grok-app-server/src/transport/**`, or C4's `xai-grok-mcp/**`. The
only shell-tree edit is the mechanical `Byok` match arm in
`xai-grok-pager-bin/src/main.rs` (a consumer of the public
`LoginProviderArg` enum), which is outside the C1-G `app_server_runtime`
boundary.

## Next handoff suggestion

- **C1-G follow-on** — once the Shell actor projects a real
  `ProviderBinding`, add the composition-root end-to-end test
  (login → persist → bind → resolve request auth → inference) for one
  BYOK vertical (OpenRouter is the simplest: OpenAI-compatible, no account
  id in path). That closes item 37 and removes the PARTIAL.
- **C5 follow-on** — decide File vs Keyring backend for BYOK API keys;
  add an interactive `rpassword` prompt; reconcile
  `PublicProviderBinding` to the protocol `ProviderBinding` or delete it.
- **Live smoke** — add `tests/live_byok_openrouter.rs` etc. behind
  `RUN_LIVE_BYOK_<PROVIDER>=1` when credentials are authorized.
