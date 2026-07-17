# Evidence Map — Paths, Symbols, Commands

Use this map to verify claims in this package against the live tree. Paths are relative to repo root `grok-goblin/`.

---

## 1. Product / architecture docs

| Path | Relevance |
|------|-----------|
| `GOBLIN.md` | Fork contract; multi-provider goals; module layout |
| `task.md` | Normative multi-provider auth; G3 multi-account; §8 model config; G7 custom API-key models remain valid |
| `docs/architecture/multi-provider-auth/PROGRESS.md` | Codex control plane status |
| `docs/architecture/multi-provider-auth/protocol-baseline.md` | Codex wire (Responses) — contrast with BYOK |
| `TO_RELEASE.md` | Release honesty; multi-provider 1.0 still open |
| `crates/codegen/xai-grok-shell/README.md` | **User docs for Custom Models** (manual TOML path) |
| `.agents/skills/add-provider/SKILL.md` | Provider end-to-end skill; API-key exclusion note |
| `docs/architecture/byok-providers-onboarding/` | **This package** |

---

## 2. API backends & sampling

| Symbol / area | Path |
|---------------|------|
| `enum ApiBackend` | `crates/codegen/xai-grok-sampling-types/src/types.rs` |
| `struct SamplingConfig` | same |
| `struct SamplerConfig` + `AuthScheme` | `crates/codegen/xai-grok-sampler/src/config.rs` |
| Backend dispatch `conversation_collect` | `crates/codegen/xai-grok-sampler/src/client.rs` |
| Stream transforms | `crates/codegen/xai-grok-sampler/src/stream/{chat,responses,messages}.rs` (names may vary; see `stream/mod.rs`) |
| Codex backend URL helpers | `crates/codegen/xai-grok-sampling-types/src/conversation.rs` (`is_codex_responses_backend`, `is_xai_api_base_url`) |

---

## 3. Model config & catalog (World A)

| Symbol / area | Path |
|---------------|------|
| `ModelEntryConfig` / resolve sampling | `crates/codegen/xai-grok-shell/src/agent/config.rs` |
| Model overrides parse | `crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs` |
| Catalog resolve / default model / short slug | `crates/codegen/xai-grok-shell/src/agent/models.rs` |
| Credential resolve `api_key`/`env_key`/`XAI_API_KEY` | `config.rs` helpers + `auth_method.rs` |
| CLI models listing | `crates/codegen/xai-grok-shell/src/cli_models.rs` |
| Endpoints `models_base_url` | `config.rs` `Endpoints` section |

---

## 4. Multi-provider auth (World B)

| Symbol / area | Path |
|---------------|------|
| `ProviderId`, `CredentialId`, `ModelBinding` | `crates/codegen/xai-grok-auth/src/types.rs` |
| `CredentialStore` trait | `crates/codegen/xai-grok-auth/src/credential.rs` |
| `AuthProvider`, `ProviderCapabilities` (incl. `API_KEY_LOGIN`) | `crates/codegen/xai-grok-auth/src/provider.rs` |
| `LoginTransport` | `crates/codegen/xai-grok-auth/src/login.rs` |
| File store multi-account | `crates/codegen/xai-grok-multi-auth/src/store/file.rs` |
| Login coordinator (rejects ApiKey) | `crates/codegen/xai-grok-multi-auth/src/login_coordinator.rs` |
| Providers registry modules | `crates/codegen/xai-grok-multi-auth/src/providers/{mod.rs,codex/,xai.rs}` |
| Catalog key format | `crates/codegen/xai-grok-multi-auth/src/provider_model_key.rs` |
| CLI login parse / status / logout helpers | `crates/codegen/xai-grok-multi-auth/src/cli.rs` |
| Token resolve / bearer | `crates/codegen/xai-grok-multi-auth/src/token_resolve.rs`, `token_manager.rs` |
| Session pin policy | `crates/codegen/xai-grok-multi-auth/src/session_pin.rs` |
| Codex merge into shell catalog | `xai-grok-shell/src/agent/models.rs` → `merge_codex_report_into_catalog` |
| Shell multi-provider resolve | `xai-grok-shell/src/auth/multi_provider_resolve.rs` |
| Binary login routing | `crates/codegen/xai-grok-pager-bin/src/main.rs` (`Command::Login`, `Auth`, `Logout`) |

---

## 5. Tests worth reusing as patterns

| Test area | Path / command |
|-----------|----------------|
| Multi-auth store alias/default | `cargo test -p xai-grok-multi-auth` (`tests/multi_auth.rs`) |
| Credential-scoped keys / ambiguity | `provider_model_key` tests; shell model resolve |
| Codex effort after merge | `cargo test -p xai-grok-shell --test codex_effort_after_merge` |
| OpenRouter-related conversion | sampling-types conversation tests (kimi / function.arguments) |
| Sampler backends | `cargo test -p xai-grok-sampler` |

Suggested baseline verification before planning implementation:

```bash
cargo test -p xai-grok-auth -p xai-grok-multi-auth --no-fail-fast
# optional broader:
# cargo test -p xai-grok-sampling-types -p xai-grok-sampler --no-fail-fast
```

---

## 6. Storage layout (runtime)

| Path | Content |
|------|---------|
| `~/.grok/config.toml` | Models, endpoints, preferences (World A) |
| `~/.grok/auth.json` | Legacy xAI session |
| `~/.grok/auth/accounts.json` | Multi-auth metadata (no secrets) |
| `~/.grok/auth/file-secrets.json` | Multi-auth secrets |
| Env `GROK_HOME` | Overrides home root |
| Env `XAI_API_KEY` | Global API key fallback |
| Env `GROK_MODELS_BASE_URL` | Single external OpenAI-compatible stack |

---

## 7. CLI surface today (relevant)

```bash
goblin login --provider xai|codex
goblin auth status
goblin logout
goblin models
goblin -p "…" --model <id>
```

Missing:

```bash
goblin login --provider openrouter|groq|cloudflare
goblin logout --provider openrouter
# account-scoped BYOK management
```

---

## 8. External protocol references (re-check at implement time)

| Provider | Doc topics |
|----------|------------|
| OpenRouter | Quickstart chat completions; authentication; Responses beta |
| Groq | OpenAI compatibility base URL; Responses API docs |
| Cloudflare | Workers AI OpenAI-compatible endpoints; account id + token |

---

## 9. Claims ↔ evidence checklist for reviewers

| Claim in this package | Evidence |
|-----------------------|----------|
| Custom models work without native providers | shell README Custom Models; `ModelEntryConfig` |
| Three API backends exist | `ApiBackend` enum + sampler client match |
| Default backend is chat_completions | `#[default]` on enum |
| Codex forces responses + credential keys | `merge_codex_report_into_catalog` |
| Multi-auth ApiKey login unfinished | `login_coordinator` ApiKey error; only codex/xai providers |
| Login CLI limited to xai/codex | `parse_login_provider` in multi-auth `cli.rs` |
| OpenRouter/Groq mentioned as OpenAI-compatible | shell README models endpoint section |
| Cloudflare needs account_id in URL | Cloudflare official OpenAI compat docs + matrix |
| Multi-account catalog pattern exists | `provider_model_key.rs` + Codex merge |

---

## 10. Suggested first code-read order for implementers (after plan)

1. `ApiBackend` + `SamplerConfig`  
2. `resolve` path model → sampling config in shell `agent/config.rs`  
3. Codex merge + bearer resolver (template for BYOK merge)  
4. `CredentialStore` create/list/delete  
5. Login CLI in `pager-bin`  
6. One end-to-end test pattern from multi-auth tests  

---

## 11. Package maintenance

When the architecture decision lands:

- Keep this package as **problem/context** or mark `SUPERSEDED by <plan path>`.  
- Do not silently edit PROBLEM outcomes to match an implementation that was never approved.  
- Update `EVIDENCE_MAP` if symbols move during large refactors.
