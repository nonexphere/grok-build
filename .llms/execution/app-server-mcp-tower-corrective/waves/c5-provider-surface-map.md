# C5-A — BYOK Provider Surface Map (read-only prep)

| Field | Value |
|---|---|
| Agent | `repo-explore` (read-only) |
| Model | `glm-5.2` |
| Wave | C5 prep (items 31–37 inputs) |
| Branch | `goblin-implement-epic-tree` |
| Status | **MAP ONLY — C5 NOT PASS**. No product code edited. |

Scope: OpenRouter / Groq / Cloudflare verticals (the three BYOK specs in
`crates/codegen/xai-grok-multi-auth/src/providers/byok/mod.rs`). xAI and Codex
are mapped only as the existing registry reference points.

---

## 1. Registration table

| Provider id | Type location | Registered in default registry? | API-key login capability advertised? |
|---|---|---|---|
| `xai` | `crates/codegen/xai-grok-multi-auth/src/providers/xai.rs` — `XaiAuthProvider` | **Yes** — `registry::build_default_registry` (`registry.rs:18`) always registers it | **No** — `descriptor()` advertises `ProviderCapabilities::empty()` (`xai.rs:55`); login deferred to legacy `AuthManager` (`xai.rs:74`) |
| `codex` | `crates/codegen/xai-grok-multi-auth/src/providers/codex/mod.rs` — `CodexAuthProvider` | **Yes** — `registry.rs:22`, gated by `kill_switch::codex_auth_disabled()` | **No** — caps are `BROWSER_PKCE \| DEVICE_CODE \| REFRESH_TOKEN \| TOKEN_REVOCATION \| MULTI_ACCOUNT \| MODEL_DISCOVERY \| ACCOUNT_INFO` (`codex/mod.rs:153-160`); `API_KEY_LOGIN` bit is **not** set |
| `openrouter` | `providers/byok/mod.rs:13` — `ByokProviderSpec` (static const `OPENROUTER`) | **No** — `registry::build_default_registry`/`build_registry` only register xAI + Codex; no BYOK spec is passed to `registry.register` (grep `register(` in `registry.rs:18,22,31,34,43,45`) | **N/A as AuthProvider** — BYOK specs are plain `ByokProviderSpec` structs, **not** `AuthProvider` impls. API-key login is handled generically by `LoginCoordinator::run_api_key_login` (`login_coordinator.rs:202`) which accepts any well-formed `ProviderId` |
| `groq` | `providers/byok/mod.rs:21` — `GROQ` | **No** (same as above) | same — generic `run_api_key_login` |
| `cloudflare` | `providers/byok/mod.rs:27` — `CLOUDFLARE` | **No** (same as above) | same — generic `run_api_key_login` |

Evidence that BYOK specs are descriptors, not providers:
- `byok/mod.rs:1-2`: "BYOK provider **descriptors** for OpenRouter, Groq, and Cloudflare. Login uses `LoginCoordinator::run_api_key_login`; request auth is static bearer."
- `byok/mod.rs:35-38`: `provider_id()` constructs a `ProviderId::new_unchecked` from the spec id; there is no `impl AuthProvider for ByokProviderSpec`.
- `ALL: &[ByokProviderSpec]` (`byok/mod.rs:33`) is a static array with no registry hook.

`ProviderCapabilities::API_KEY_LOGIN` bit exists (`provider.rs:39`) but is
**never set** by any provider descriptor in the repo (grep `API_KEY_LOGIN`
returns only the definition line). The BYOK verticals do not advertise it
because they are not `AuthProvider` impls.

---

## 2. Login path (public entry that must reject unknown/unregistered providers)

### 2a. Multi-provider login coordinator (canonical BYOK path)

`crates/codegen/xai-grok-multi-auth/src/login_coordinator.rs`

- `LoginCoordinator::start_login` (`login_coordinator.rs:88`) calls
  `self.registry.get(provider_id)` (`login_coordinator.rs:96-98`) and maps
  `ProviderRegistrationError` → `ProviderError::InvalidConfig`. An
  **unregistered** provider id (e.g. `openrouter` when no BYOK `AuthProvider`
  is registered) is rejected here with `ProviderError::InvalidConfig`.
- `LoginCoordinator::run_login` (`login_coordinator.rs:188`) dispatches on
  `LoginTransport`: `BrowserPkce` → `run_browser_login`, `DeviceCode` →
  `run_device_login`, `ApiKey` → `run_api_key_login`.
- `LoginCoordinator::run_api_key_login` (`login_coordinator.rs:202`) is the
  **public BYOK API-key login path**. It does `let _ = self.registry.get(provider_id);`
  (`login_coordinator.rs:211`) — it **ignores** the registry result and
  accepts any well-formed `ProviderId`. This is the documented foundation
  behavior: "BYOK foundation also allows well-formed provider ids so
  openrouter/groq/cloudflare can land before their full AuthProvider impls"
  (`login_coordinator.rs:209-211`).

  **Gap (C5 RED test target):** `run_api_key_login` does NOT reject
  unknown/unregistered provider ids today — it only requires a non-empty
  secret from `api_key` arg or `GROK_BYOK_API_KEY` env
  (`login_coordinator.rs:213-228`). A RED test must assert the intended
  policy: either (a) accept only ids in `byok::ALL`, or (b) accept any
  well-formed id but reject `xai` fallback collisions. Current code only
  enforces the `XAI_API_KEY` fallback guard via
  `byok::reject_xai_api_key_fallback` (`byok/mod.rs:131`), which is **not**
  called from `run_api_key_login` (grep: only its own unit test calls it).

### 2b. Shell CLI login routing (legacy path — does NOT touch BYOK)

`crates/codegen/xai-grok-shell/src/auth/flow.rs` uses `LoginTransportOverride`
(`flow.rs:46`) and routes xAI OAuth/device flows via `auth::oidc::run_login_flow`
(`flow.rs:623,652`). Grep for `run_api_key_login|LoginTransport::ApiKey|byok::|
providers::byok` in `xai-grok-shell/src` returns **no matches** — the shell
does **not** wire the multi-provider BYOK login path. Shell BYOK today is the
legacy per-model `api_key`/`env_key` TOML config
(`crates/codegen/xai-grok-shell/src/cli_models.rs:120-242`:
`byok_and_deployment_toml`, `resolve_model_api_key_byok`,
`resolve_model_env_key_byok`), unrelated to the registry.

### 2c. CLI parse surface

`crates/codegen/xai-grok-multi-auth/src/cli.rs:30` — `parse_login_provider`
accepts only `xai`/`grok`/`codex`/`chatgpt`; `unknown` → `Err("unknown provider")`
(`cli.rs:38`). `openrouter`/`groq`/`cloudflare` are **not** parseable login
providers today. `prompt_provider_selection` (`cli.rs:53`) lists
`registry.list()` descriptors — BYOK specs are not in the registry, so they
never appear in the interactive picker.

**Login path summary:** the only public path that must reject unknown
providers is `LoginCoordinator::start_login` (registry-backed, used by
browser/device). The API-key path (`run_api_key_login`) currently does NOT
reject unknown ids — this is the C5-B contract gap to RED-test and fix.

---

## 3. Credential store backend policy

Canonical store lives in `crates/codegen/xai-grok-multi-auth/src/store/` and
implements `xai_grok_auth::CredentialStore`.

- **Layout / policy doc:** `store/mod.rs:1-30` —
  `{home}/auth/accounts.json` (public metadata + defaults + aliases),
  `accounts.json.lock` (advisory flock), `file-secrets.json` (secret, 0o600),
  `file-secrets.json.lock`, `locks/<provider>/<credential-id>.lock`
  (per-credential refresh lock).
- **File backend:** `store/file.rs::FileCredentialStore` (`file.rs:33`).
  - `new(home)` (`file.rs:42`) does NOT recover journal eagerly; recovery
    runs under write+flock on first access via `ensure_journal_recovered`
    (`file.rs:58`) — fail-loud quarantine of corrupt journal.
  - Atomic dual-file writes via `metadata::commit_accounts_and_secrets`
    (imported `file.rs:24`); cross-process safety via advisory `flock`
    (`store/lock.rs` — `acquire_blocking`, `acquire_credential_lock`,
    `FileLockGuard`).
  - In-process serialization: `tokio::sync::Mutex` (`file.rs:36`,
    `write_lock`).
- **Ephemeral backend:** `store/ephemeral.rs::EphemeralCredentialStore`
  (`ephemeral.rs:33`) — in-memory, used by tests and `run_api_key_login`
  unit tests (`login_coordinator.rs:439,459`).
- **Composite:** `store/composite.rs::AutoCredentialStore` (`composite.rs:17`)
  — forwards to a single inner backend; no keyring/plaintext selection logic
  yet. Despite the name, it is **not** a keyring-vs-file policy router today.
- **Backend kind enum:** `xai_grok_auth::SecretBackendKind` (used as
  `NewCredentialRecord.backend`). `run_api_key_login` hard-codes
  `SecretBackendKind::Ephemeral` (`login_coordinator.rs:249`) — **file
  persistence for BYOK API keys is not wired**; the credential lives only in
  whatever store the coordinator was constructed with. The unit tests
  construct the coordinator with `EphemeralCredentialStore`, so nothing is
  persisted to disk in the BYOK path today.

**Policy gap:** there is no keyring preference, no owner-only file permission
enforcement specific to BYOK, and no crash-consistent dual-file transaction
for the API-key path (it reuses the generic store `create`). C5-B must decide
whether BYOK API keys use `File` (0o600) or `Ephemeral` and add the missing
`reject_xai_api_key_fallback` call into `run_api_key_login`.

---

## 4. Protocol binding — where `ProviderBinding` is defined vs duplicates

Two distinct types named "ProviderBinding" exist; they are **not duplicates**
but a wire/protocol type and a BYOK projection helper:

| Type | Crate / file | Role | Fields |
|---|---|---|---|
| `ProviderBinding` (canonical wire/protocol) | `crates/codegen/xai-grok-app-server-protocol/src/lib.rs:96` | Public, immutable inference selection sent over App Server / ACP wire. `Serialize+Deserialize+JsonSchema`, `#[serde(deny_unknown_fields)]`. | `provider_id`, `credential_id`, `model_id`, `backend: String`, `binding_revision: WireCounter` |
| `SessionMetadata.provider_binding` | `crates/codegen/xai-grok-tower/src/metadata.rs:17` | Re-exports the protocol `ProviderBinding` (`use xai_grok_app_server_protocol::{ProviderBinding,…}`) — same type, not a duplicate | — |
| `PublicProviderBinding` (BYOK projection) | `crates/codegen/xai-grok-multi-auth/src/providers/byok/mod.rs:53` | Local, non-serialized projection for "protocol/App Server — identifiers only" (`byok/mod.rs:51`). `binding_revision` is `u64`, not `WireCounter`. | `provider_id`, `credential_id`, `model_id`, `backend`, `binding_revision: u64` |

- The protocol `ProviderBinding` is consumed in:
  - `xai-grok-app-server-protocol/src/methods.rs:141,156` (`Session`,
    `SessionStartParams` test fixtures).
  - `xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs:165`
    — comment documents that `credential_id`/`backend` require actor-side
    resolution and `provider_binding` is left `None` (PARTIAL).
  - `xai-grok-shell/src/app_server_runtime/mod.rs:173,182,209,254,263` —
    all sessions currently project `provider_binding: None`.
- `PublicProviderBinding` is **not** referenced outside `byok/mod.rs` (grep
  shows only its own definition + tests). It is a candidate bridge from a
  future BYOK `AuthProvider` to the protocol `ProviderBinding`, but today it
  is unused at the composition root.

**No true duplicate type exists.** `PublicProviderBinding` is a structurally
parallel helper with `u64` revision; C5-B should either reconcile it to
`WireCounter` or document why the BYOK path keeps a separate projection.

`ModelBinding` (runtime request identity) is a separate type in
`xai-grok-auth` and is built via `model_binding::model_binding`
(`crates/codegen/xai-grok-multi-auth/src/model_binding.rs:37`) and
`ProviderModelKey::to_model_binding` (`provider_model_key.rs:52`). Catalog
key format is `{provider}/{credential_uuid}/{slug}` (`provider_model_key.rs:21`,
`byok/mod.rs:43`).

---

## 5. Offline fixture strategy for HTTP boundary tests

Goal: C5-B must write offline contract RED tests for OpenRouter/Groq/Cloudflare
without live secrets. Recommended strategy, grounded in existing patterns:

1. **Ephemeral store + empty registry** for login-persistence tests:
   `EphemeralCredentialStore::new()` + `ProviderRegistry::new()` (empty) —
   pattern from `login_coordinator.rs:439-446`. BYOK `run_api_key_login`
   ignores the registry, so an empty registry is sufficient.
2. **Secret fixtures** via `SecretString::from_str("sk-test-…-never-log")`
   (pattern `login_coordinator.rs:447`); assert `format!("{meta:?}")` does
   not contain the secret (`login_coordinator.rs:449-453`).
3. **Schema-faithful HTTP mocks** for the inference boundary:
   - OpenRouter/Groq use OpenAI-compatible `chat_completions` backend
     (`byok/mod.rs:17,25` — `default_backend: "chat_completions"`). Fixtures
     must mirror the OpenAI `/chat/completions` request/response schema
     (model, messages, stream, choices) and error shapes (401, 429, 5xx).
   - Cloudflare Workers AI uses a different base URL
     (`https://api.cloudflare.com/client/v4/accounts`, `byok/mod.rs:29`)
     and a path that includes the account id; fixtures must model the
     Cloudflare error envelope, not the OpenAI one.
   - Use `mockito`/`wiremock` (already a workspace pattern in
     `xai-grok-sampler`/`xai-grok-tools` tests) and assert exact URL/method/
     `Authorization: Bearer <opaque>` header (`static_bearer_authorization`
     `byok/mod.rs:46`). Never put the real key in the assertion string —
     assert `header == format!("Bearer {}", opaque_id)` only.
4. **Endpoint resolution fixtures:** `CodexAuthProvider::resolve_endpoint`
   (`providers/codex/request_auth.rs:73`) is the reference pattern for a
   provider-owned endpoint resolver. BYOK providers need an equivalent that
   combines `ByokProviderSpec.base_url` + backend + model id. RED tests
   should assert the resolved URL per vertical before any HTTP is sent.
5. **Catalog key fixtures:** use real `CredentialId::from_uuid` values
   (`provider_model_key.rs:138`) — never embed secrets in catalog keys
   (`byok/mod.rs:43-47`, `byok_tests` `byok_specs_cover_three_verticals…`).
6. **No live credentials:** all fixtures use synthetic `sk-test-*` keys;
   `reject_xai_api_key_fallback` (`byok/mod.rs:131`) must be exercised with
   synthetic env names (`byok/mod.rs:143-147`).

---

## 6. Live smoke — explicit SKIP policy when creds missing (never PASS)

Reference policy (already implemented for Codex):
`crates/codegen/xai-grok-multi-auth/tests/live_codex_models.rs:6-10`:

```rust
if std::env::var("RUN_LIVE_CODEX").ok().as_deref() != Some("1") {
    eprintln!("skip: set RUN_LIVE_CODEX=1 to run");
    return;
}
```

- The test **returns** (skips) when the env gate is unset; it does **not**
  `#[ignore]` (which would still PASS in `cargo test --ignored` semantics
  depending on harness). It only asserts when `RUN_LIVE_CODEX=1` is set, and
  then requires real models (`live_codex_models.rs:18-28`).
- **C5-B must follow the same gate pattern** for any OpenRouter/Groq/Cloudflare
  live smoke, e.g. `RUN_LIVE_BYOK_OPENROUTER=1`, `RUN_LIVE_BYOK_GROQ=1`,
  `RUN_LIVE_BYOK_CLOUDFLARE=1`. Each live test must:
  1. `return` (not pass) when its gate is unset;
  2. require a non-empty credential from the store;
  3. assert real inference output and that no seeded secret appears in any
     output/diagnostic surface (per skill §K and `secret-prefix telemetry`
     common mistake).
- **Never** mark a BYOK provider PASS based on a skipped test. The skill's
  "PASS/not-claimed contradiction" rule (`add-provider/SKILL.md` Common
  Mistakes) forbids simultaneous PASS + deferred + not-claimed.

---

## 7. Files for C5-B implementer + dependency on C1 turns

### 7a. Owned files (C5-B may edit; non-overlapping with C3/C4)

BYOK provider surface (all under `crates/codegen/xai-grok-multi-auth/src/`):
- `providers/byok/mod.rs` — descriptors + `PublicProviderBinding` +
  `static_bearer_authorization` + `reject_xai_api_key_fallback`. C5-B likely
  promotes this into real `AuthProvider` impls (one per vertical, or a
  generic `ByokAuthProvider` driven by `ByokProviderSpec`).
- `login_coordinator.rs::run_api_key_login` — wire `reject_xai_api_key_fallback`,
  decide `SecretBackendKind` (File vs Ephemeral), and reject unknown
  provider ids if policy (a) is chosen.
- `registry.rs` — if BYOK providers become `AuthProvider`s, register them
  here (gated by a new kill switch if needed).
- `providers/{openrouter,groq,cloudflare}/` — new modules for endpoint
  resolution + request auth (mirror `providers/codex/request_auth.rs`).
- `cli.rs::parse_login_provider` — extend to accept `openrouter`/`groq`/
  `cloudflare` if CLI login is in scope.
- `kill_switch.rs` — add BYOK gates mirroring Codex pattern.

Tests (C5-B owned):
- `crates/codegen/xai-grok-multi-auth/tests/` — new
  `byok_openrouter.rs`, `byok_groq.rs`, `byok_cloudflare.rs` (offline
  contract + wire) and `live_byok_*.rs` (gated smoke).
- Existing: `tests/credential_scoped_and_recover.rs`,
  `tests/multi_auth.rs`, `tests/login_e2e.rs` — extend, do not break.

### 7b. Non-overlapping boundaries (must NOT touch)

- `crates/codegen/xai-grok-app-server/**` — C3-B (WebSocket transport).
- `crates/codegen/xai-grok-mcp/**` + tower tools — C4-B (MCP Streamable HTTP).
- `crates/codegen/xai-grok-shell/src/app_server_runtime/**` — C1-G (Shell
  turn lifecycle / session actor). C5-B may later need to project a real
  `ProviderBinding` into `shell_session_actor_runtime.rs:168` (currently
  `None`), but that is a **follow-up** after C1-G lands and must be
  coordinated, not part of the C5-B map slice.
- `crates/codegen/xai-grok-app-server-protocol/src/lib.rs::ProviderBinding`
  — wire schema is frozen by C0/C3; C5-B must **not** change its fields.

### 7c. Dependency on C1 turns

The handoff asks: "dependency on C1 turns (if turn binding required)."

- The protocol `ProviderBinding` is carried by `SessionStartParams`
  (`methods.rs:156`) and `Session` (`lib.rs:222`). The Shell actor currently
  projects `provider_binding: None` for all sessions
  (`shell_session_actor_runtime.rs:168`, `app_server_runtime/mod.rs:173…`).
- A full BYOK end-to-end test (login → persist → bind → resolve request auth
  → inference) requires the **composition root** to consume `ModelBinding`
  and project a real `ProviderBinding` into the session — which is the C1-G
  Shell turn-lifecycle surface (`HANDOFF-C1-G-turn-lifecycle.md`).
- **Therefore:** C5-B can land the **offline contract + wire + store + CLI
  login** slice (items 31–34) without C1-G. The **end-to-end composition
  test** (item 37, "production composition root consumes `ModelBinding`")
  requires C1-G's session/turn binding to be in place; otherwise C5-B can
  only prove the resolver in isolation, which the skill explicitly forbids
  as completion ("Dead resolver" common mistake, `add-provider/SKILL.md`).
- **Recommended sequencing:** C5-B starts the offline/wire slice in
  parallel with C3/C4 (see §8); the composition-root test is a follow-up
  gate after C1-G.

---

## 8. Can C5-B start in parallel with C3/C4 after maps land?

**YES — for the offline contract + wire + store + CLI slice.**

Evidence:
- C3-B owns `xai-grok-app-server/src/transport/**` (WebSocket listener).
- C4-B owns `xai-grok-mcp/**` + tower tools (Streamable HTTP + nine-tool core).
- C5-B owns `xai-grok-multi-auth/src/providers/byok/**` + `login_coordinator`
  + `registry` + `cli.rs` + new `tests/byok_*.rs`.
- The three file sets are **disjoint** (grep confirms no shared edit surface
  across `byok/`, `transport/`, `mcp/`).
- The only shared *type* is `xai_grok_app_server_protocol::ProviderBinding`,
  which is frozen by C0 and read-only for all three waves.

**Caveats / coordination points (not blockers for the parallel start):**
1. **Composition-root test (C5 item 37)** depends on C1-G's Shell turn
   binding (`shell_session_actor_runtime.rs:168` currently `None`). C5-B
   must time-box the composition test after C1-G; the offline slice can
   proceed now.
2. If C5-B promotes BYOK to real `AuthProvider`s and registers them in
   `registry::build_default_registry`, it must not change the xAI/Codex
   registration order or capabilities (regression risk for
   `tests/multi_auth.rs`, `tests/credential_scoped_and_recover.rs`).
3. C5-B must not edit `xai-grok-app-server-protocol::ProviderBinding`
   (owned by C0/C3 wire freeze).
4. Live smoke tests must use the `RUN_LIVE_BYOK_*` gate pattern (§6) and
   must never PASS on skip.

**GO / NO-GO for C5-B in parallel with C3/C4:** **GO** for the offline
contract + wire + store + CLI slice (items 31–34). Defer the
composition-root end-to-end test (item 37) until C1-G lands.

---

## Evidence index (file:fn)

- `crates/codegen/xai-grok-multi-auth/src/providers/byok/mod.rs:13,21,27` — `OPENROUTER/GROQ/CLOUDFLARE` specs.
- `crates/codegen/xai-grok-multi-auth/src/providers/byok/mod.rs:33` — `ALL` array (no registry hook).
- `crates/codegen/xai-grok-multi-auth/src/providers/byok/mod.rs:46` — `static_bearer_authorization`.
- `crates/codegen/xai-grok-multi-auth/src/providers/byok/mod.rs:53` — `PublicProviderBinding`.
- `crates/codegen/xai-grok-multi-auth/src/providers/byok/mod.rs:131` — `reject_xai_api_key_fallback` (NOT called from login path).
- `crates/codegen/xai-grok-multi-auth/src/registry.rs:14,18,22,29,41` — `build_default_registry`/`build_registry` (xAI + Codex only).
- `crates/codegen/xai-grok-multi-auth/src/login_coordinator.rs:202` — `run_api_key_login` (ignores registry).
- `crates/codegen/xai-grok-multi-auth/src/login_coordinator.rs:211` — `let _ = self.registry.get(provider_id);` (gap).
- `crates/codegen/xai-grok-multi-auth/src/login_coordinator.rs:249` — `SecretBackendKind::Ephemeral` hard-coded.
- `crates/codegen/xai-grok-multi-auth/src/cli.rs:30,38` — `parse_login_provider` (no BYOK ids).
- `crates/codegen/xai-grok-multi-auth/src/store/mod.rs:1-30` — store layout/policy.
- `crates/codegen/xai-grok-multi-auth/src/store/file.rs:42,58` — `FileCredentialStore::new`/`ensure_journal_recovered`.
- `crates/codegen/xai-grok-multi-auth/src/store/composite.rs:17` — `AutoCredentialStore` (no keyring policy).
- `crates/codegen/xai-grok-auth/src/provider.rs:39` — `ProviderCapabilities::API_KEY_LOGIN` (defined, never set).
- `crates/codegen/xai-grok-auth/src/provider.rs:149` — `trait AuthProvider`.
- `crates/codegen/xai-grok-auth/src/provider.rs:220` — `ProviderRegistry`.
- `crates/codegen/xai-grok-multi-auth/src/providers/xai.rs:55,74` — xAI empty caps, legacy deferral.
- `crates/codegen/xai-grok-multi-auth/src/providers/codex/mod.rs:153-160` — Codex caps (no API_KEY_LOGIN).
- `crates/codegen/xai-grok-multi-auth/src/providers/codex/request_auth.rs:73` — `resolve_codex_endpoint` (reference pattern).
- `crates/codegen/xai-grok-app-server-protocol/src/lib.rs:96` — canonical `ProviderBinding`.
- `crates/codegen/xai-grok-tower/src/metadata.rs:17` — re-export (not duplicate).
- `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs:165,168` — `provider_binding: None` (PARTIAL, C1-G).
- `crates/codegen/xai-grok-shell/src/auth/flow.rs:46,623,652` — legacy xAI login (no BYOK wiring).
- `crates/codegen/xai-grok-multi-auth/src/provider_model_key.rs:21,52` — catalog key + `to_model_binding`.
- `crates/codegen/xai-grok-multi-auth/tests/live_codex_models.rs:6-10` — live SKIP policy reference.
- `crates/codegen/xai-grok-multi-auth/tests/credential_scoped_and_recover.rs` — generation/401 test pattern.
- `crates/codegen/xai-grok-multi-auth/tests/multi_auth.rs` — store/registry test pattern.

---

## Recommended next reading (for C5-B implementer)

1. `crates/codegen/xai-grok-multi-auth/src/login_coordinator.rs` (full) — the
   API-key login path to harden.
2. `crates/codegen/xai-grok-multi-auth/src/providers/codex/request_auth.rs` —
   the reference endpoint/header resolver to mirror for BYOK verticals.
3. `crates/codegen/xai-grok-multi-auth/src/store/file.rs` + `store/metadata.rs`
   — crash-consistent dual-file transaction pattern to reuse for BYOK file
   persistence.
4. `crates/codegen/xai-grok-multi-auth/tests/credential_scoped_and_recover.rs`
   — generation-aware 401 recovery test pattern (attempt-id contract).
5. `.agents/skills/add-provider/references/provider-checklist.md` — mandatory
   evidence matrix (sections D, E, G, H, L for BYOK).
6. `docs/architecture/multi-provider-auth/protocol-baseline.md` — frozen
   protocol contracts before writing wire fixtures.
7. `HANDOFF-C1-G-turn-lifecycle.md` — the C1-G dependency for the
   composition-root end-to-end test (item 37).

---

*Read-only prep. No product code edited. C5 is NOT marked PASS.*
