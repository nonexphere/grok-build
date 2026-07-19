//! CLI helpers: parse provider arg, status JSON (no secrets), login routing,
//! interactive provider picker, multi-provider logout, model listing.

use std::io::{self, BufRead, Write};
use std::path::Path;

use serde_json::{json, Value};
use xai_grok_auth::{
    AuthProvider, CredentialStore, ModelListRequest, ProviderId, ProviderModel, ProviderRegistry,
};

use crate::kill_switch;
use crate::providers::codex::claims;
use crate::store::file::FileCredentialStore;

/// Which provider a login command targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginProviderArg {
    /// xAI (legacy path via AuthManager).
    Xai,
    /// Codex / ChatGPT (native multi-provider path).
    Codex,
    /// BYOK API-key provider (OpenRouter / Groq / Cloudflare).
    Byok(String),
    /// Interactive selection (no explicit provider).
    Interactive,
}

/// Parse a login provider argument string.
///
/// Accepts: `xai`, `grok` → `Xai`; `codex`, `chatgpt` → `Codex`;
/// `openrouter`, `groq`, `cloudflare` → `Byok(id)`;
/// `None` → `Interactive`; anything else → error.
pub fn parse_login_provider(arg: Option<&str>) -> Result<LoginProviderArg, String> {
    match arg {
        None => Ok(LoginProviderArg::Interactive),
        Some(s) => {
            let lower = s.to_ascii_lowercase();
            match lower.as_str() {
                "xai" | "grok" => Ok(LoginProviderArg::Xai),
                "codex" | "chatgpt" => Ok(LoginProviderArg::Codex),
                "openrouter" | "groq" | "cloudflare" => Ok(LoginProviderArg::Byok(lower)),
                other => Err(format!("unknown provider: {other}")),
            }
        }
    }
}

/// Whether Codex auth is disabled (kill switch).
pub fn codex_auth_disabled() -> bool {
    kill_switch::codex_auth_disabled()
}

/// Interactive provider picker for `grok login` with no `--provider`.
///
/// Lists login-eligible providers. Codex is omitted unless OAuth login is
/// product-approved (B5 / `GROK_CODEX_OAUTH_APPROVED`). Non-TTY environments
/// never block on stdin: they require `--provider` when more than one choice
/// remains, otherwise default to xAI.
pub fn prompt_provider_selection(registry: &ProviderRegistry) -> Result<LoginProviderArg, String> {
    let mut descriptors = registry.list();
    // Fail-closed product surface: hide Codex from the default picker until
    // D10 approval opt-in (existing credentials remain usable for inference).
    if !kill_switch::codex_oauth_login_allowed() {
        descriptors.retain(|d| d.id.as_str() != "codex");
    }
    descriptors.sort_by_key(|d| d.default_priority);

    if descriptors.is_empty() {
        return Err("no auth providers are available for login".into());
    }

    // Single provider → no need to ask.
    if descriptors.len() == 1 {
        return parse_login_provider(Some(descriptors[0].id.as_str()));
    }

    // Non-TTY: never block (M3 / Phase 8).
    if !stdin_is_terminal() {
        return Err(
            "multiple login providers available; pass --provider <xai|codex> in non-interactive mode"
                .into(),
        );
    }

    println!("Which provider do you want to log in with?\n");
    for (i, d) in descriptors.iter().enumerate() {
        let marker = if i == 0 { ">" } else { " " };
        println!("{marker} {}. {} ({})", i + 1, d.display_name, d.id.as_str());
    }
    println!();
    print!("Enter number [1]: ");
    let _ = io::stdout().flush();

    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| format!("failed to read selection: {e}"))?;
    let line = line.trim();
    if line.is_empty() {
        return parse_login_provider(Some(descriptors[0].id.as_str()));
    }
    if let Ok(n) = line.parse::<usize>() {
        if n >= 1 && n <= descriptors.len() {
            return parse_login_provider(Some(descriptors[n - 1].id.as_str()));
        }
    }
    // Also accept id strings.
    parse_login_provider(Some(line))
}

fn stdin_is_terminal() -> bool {
    if std::env::var_os("CI").is_some() {
        return false;
    }
    use std::io::IsTerminal;
    io::stdin().is_terminal()
}

/// Logout multi-provider credentials for `provider` (or all when `None`),
/// optionally also clearing legacy xAI auth.json via `legacy_logout`.
pub async fn logout_providers(
    store: &dyn CredentialStore,
    provider: Option<&ProviderId>,
    legacy_logout: impl FnOnce() -> Result<(), String>,
) -> Result<LogoutReport, String> {
    let mut removed = 0u32;
    let providers = if let Some(p) = provider {
        vec![p.clone()]
    } else {
        store
            .list_providers()
            .await
            .map_err(|e| e.to_string())?
    };

    for p in providers {
        let accounts = store
            .list_accounts(&p)
            .await
            .map_err(|e| e.to_string())?;
        for meta in accounts {
            if store
                .delete(&meta.key)
                .await
                .map_err(|e| e.to_string())?
            {
                removed += 1;
            }
        }
    }

    // Always attempt legacy xAI logout when no provider filter or when xai.
    let run_legacy = match provider {
        None => true,
        Some(p) => p.as_str() == "xai",
    };
    let mut legacy = false;
    if run_legacy {
        legacy_logout()?;
        legacy = true;
    }

    Ok(LogoutReport {
        multi_provider_removed: removed,
        legacy_xai_cleared: legacy,
    })
}

/// Summary of a multi-provider logout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogoutReport {
    pub multi_provider_removed: u32,
    pub legacy_xai_cleared: bool,
}

/// Codex models fetched for every stored Codex credential under `home`.
#[derive(Debug, Clone)]
pub struct CodexModelsReport {
    pub accounts: Vec<CodexAccountModels>,
}

#[derive(Debug, Clone)]
pub struct CodexAccountModels {
    pub alias: String,
    pub email: Option<String>,
    /// Stable runtime credential id (UUID). Required for catalog keys.
    pub credential_id: xai_grok_auth::CredentialId,
    pub account_id: Option<String>,
    pub models: Vec<ProviderModel>,
    pub error: Option<String>,
}

/// Load Codex credentials from the multi-provider store and fetch `/models`
/// for each account. Also re-enriches account metadata from tokens so older
/// logins missing `chatgpt_account_id` still work.
pub async fn list_codex_models(home: &Path) -> Result<CodexModelsReport, String> {
    if kill_switch::codex_auth_disabled() {
        return Err("Codex auth is disabled (GROK_DISABLE_CODEX_AUTH)".into());
    }
    let store = FileCredentialStore::new(home.to_path_buf());
    let provider_id = ProviderId::new_unchecked("codex");
    let accounts = store
        .list_accounts(&provider_id)
        .await
        .map_err(|e| e.to_string())?;
    if accounts.is_empty() {
        return Ok(CodexModelsReport {
            accounts: Vec::new(),
        });
    }

    let registry = crate::registry::build_default_registry();
    let provider = registry
        .get(&provider_id)
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for meta in accounts {
        let loaded = store
            .load(&meta.key)
            .await
            .map_err(|e| e.to_string())?;
        let Some(mut cred) = loaded else {
            out.push(CodexAccountModels {
                alias: meta.alias,
                email: meta.account.email,
                credential_id: meta.key.credential_id,
                account_id: meta.account.provider_account_id,
                models: Vec::new(),
                error: Some("secret missing for credential".into()),
            });
            continue;
        };

        // Repair missing ChatGPT-Account-ID from JWT claims (in-memory).
        cred.metadata.account = claims::enrich_account_from_tokens(
            &cred.metadata.account,
            cred.secret.id_token.as_ref().map(|s| s.expose()),
            Some(cred.secret.access_token.expose()),
        );
        // Persist repaired metadata so future requests have the account id.
        if meta.account.provider_account_id.is_none()
            && cred.metadata.account.provider_account_id.is_some()
        {
            let _ = store
                .compare_and_swap(
                    meta.generation,
                    xai_grok_auth::CredentialUpdate {
                        key: meta.key.clone(),
                        account: Some(cred.metadata.account.clone()),
                        secret: None,
                        expires_at: None,
                        status: None,
                        updated_at: chrono::Utc::now(),
                    },
                )
                .await;
        }

        let account_id = cred.metadata.account.provider_account_id.clone();
        let email = cred.metadata.account.email.clone();
        let alias = cred.metadata.alias.clone();

        let result = provider
            .list_models(ModelListRequest {
                credential: Some(&cred),
                client_version: crate::providers::codex::models::DEFAULT_MODELS_CLIENT_VERSION,
                etag: None,
            })
            .await;

        match result {
            Ok(catalog) => out.push(CodexAccountModels {
                alias,
                email,
                credential_id: meta.key.credential_id,
                account_id,
                models: catalog.models,
                error: None,
            }),
            Err(e) => out.push(CodexAccountModels {
                alias,
                email,
                credential_id: meta.key.credential_id,
                account_id,
                models: Vec::new(),
                error: Some(e.to_string()),
            }),
        }
    }

    Ok(CodexModelsReport { accounts: out })
}

/// Blocking wrapper for shell catalog merge (safe from sync `resolve_model_catalog`).
///
/// Prefer the async [`list_codex_models`] when already on a Tokio runtime.
/// Current-thread / LocalSet safe — does not use `block_in_place`.
pub fn list_codex_models_blocking(home: &Path) -> Result<CodexModelsReport, String> {
    let home = home.to_path_buf();
    crate::token_resolve::block_on_safe(async move { list_codex_models(&home).await })
}

/// Pretty-print Codex models for the CLI (no secrets).
pub fn format_codex_models_report(report: &CodexModelsReport) -> String {
    let mut s = String::new();
    if report.accounts.is_empty() {
        s.push_str("No Codex credentials stored. Run: goblin login --provider codex\n");
        return s;
    }
    for acct in &report.accounts {
        s.push_str(&format!(
            "Codex account `{}`{}\n",
            acct.alias,
            acct.email
                .as_ref()
                .map(|e| format!(" <{e}>"))
                .unwrap_or_default()
        ));
        if let Some(id) = &acct.account_id {
            s.push_str(&format!("  account_id: {id}\n"));
        }
        if let Some(err) = &acct.error {
            s.push_str(&format!("  error: {err}\n"));
            continue;
        }
        if acct.models.is_empty() {
            s.push_str("  (no models returned)\n");
            continue;
        }
        s.push_str("  models:\n");
        for m in &acct.models {
            let ctx = m
                .context_window
                .map(|c| format!("  ctx={c}"))
                .unwrap_or_default();
            s.push_str(&format!(
                "  - {} ({}){ctx}\n",
                m.id,
                m.display_name
            ));
            if let Some(d) = &m.description {
                s.push_str(&format!("      {d}\n"));
            }
        }
    }
    s
}

/// Build a JSON status of all providers and credentials, with NO secrets.
///
/// The output never contains `access_token` or `refresh_token` substrings.
pub async fn auth_status_json(
    store: &dyn CredentialStore,
    registry: &ProviderRegistry,
) -> Value {
    let descriptors = registry.list();
    let mut providers = serde_json::Map::new();

    for desc in &descriptors {
        let provider_id = desc.id.clone();
        let accounts = store.list_accounts(&provider_id).await.unwrap_or_default();
        let default_key = store.default_account(&provider_id).await.unwrap_or(None);

        let accounts_json: Vec<Value> = accounts
            .iter()
            .map(|m| {
                json!({
                    "credential_id": m.key.credential_id.to_string(),
                    "alias": m.alias,
                    "status": format!("{:?}", m.status),
                    "generation": m.generation,
                    "is_default": default_key.as_ref() == Some(&m.key),
                    "email": m.account.email,
                    "display_name": m.account.display_name,
                    "expires_at": m.expires_at.map(|dt| dt.to_rfc3339()),
                    "created_at": m.created_at.to_rfc3339(),
                    "updated_at": m.updated_at.to_rfc3339(),
                })
            })
            .collect();

        providers.insert(
            provider_id.as_str().to_string(),
            json!({
                "display_name": desc.display_name,
                "short_name": desc.short_name,
                "capabilities": desc.capabilities.bits(),
                "default_priority": desc.default_priority,
                "account_count": accounts.len(),
                "accounts": accounts_json,
            }),
        );
    }

    json!({
        "providers": providers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;
    use crate::store::ephemeral::EphemeralCredentialStore;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use xai_grok_auth::{
        CredentialSecret, NewCredentialRecord, ProviderAccountInfo, ProviderId,
        SecretBackendKind, SecretString,
    };

    /// Test 16: cli parse_login_provider.
    #[test]
    fn parse_login_provider_variants() {
        assert_eq!(
            parse_login_provider(None).unwrap(),
            LoginProviderArg::Interactive
        );
        assert_eq!(
            parse_login_provider(Some("xai")).unwrap(),
            LoginProviderArg::Xai
        );
        assert_eq!(
            parse_login_provider(Some("grok")).unwrap(),
            LoginProviderArg::Xai
        );
        assert_eq!(
            parse_login_provider(Some("codex")).unwrap(),
            LoginProviderArg::Codex
        );
        assert_eq!(
            parse_login_provider(Some("chatgpt")).unwrap(),
            LoginProviderArg::Codex
        );
        assert_eq!(
            parse_login_provider(Some("CODEX")).unwrap(),
            LoginProviderArg::Codex
        );
        assert_eq!(
            parse_login_provider(Some("openrouter")).unwrap(),
            LoginProviderArg::Byok("openrouter".into())
        );
        assert_eq!(
            parse_login_provider(Some("Groq")).unwrap(),
            LoginProviderArg::Byok("groq".into())
        );
        assert_eq!(
            parse_login_provider(Some("cloudflare")).unwrap(),
            LoginProviderArg::Byok("cloudflare".into())
        );
        assert!(parse_login_provider(Some("unknown")).is_err());
    }

    /// Test 17: auth_status_json contains no access_token/refresh_token
    /// substrings when credential present.
    #[tokio::test]
    #[serial_test::serial]
    async fn auth_status_json_no_secrets() {
        let store = EphemeralCredentialStore::new();
        let registry = registry::build_registry(false);

        // Create a credential with a real access token value.
        let provider_id = ProviderId::new_unchecked("codex");
        let mut account = ProviderAccountInfo::default();
        account.email = Some("user@example.com".to_string());
        account
            .metadata
            .insert("chatgpt_account_id".to_string(), "acct-456".to_string());

        store
            .create(NewCredentialRecord {
                provider: provider_id.clone(),
                requested_alias: Some("personal".to_string()),
                account,
                secret: CredentialSecret {
                    access_token: SecretString::from_str("super-secret-access-token-value"),
                    refresh_token: Some(SecretString::from_str("super-secret-refresh-token-value")),
                    id_token: None,
                    fields: BTreeMap::new(),
                },
                expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
                backend: SecretBackendKind::Ephemeral,
            })
            .await
            .unwrap();

        let status = auth_status_json(&store, &registry).await;
        let json_str = serde_json::to_string(&status).unwrap();

        // Must NOT contain secret substrings.
        assert!(
            !json_str.contains("access_token"),
            "JSON status must not contain 'access_token': {json_str}"
        );
        assert!(
            !json_str.contains("refresh_token"),
            "JSON status must not contain 'refresh_token': {json_str}"
        );
        assert!(
            !json_str.contains("super-secret"),
            "JSON status must not contain secret values: {json_str}"
        );

        // But SHOULD contain the alias and email.
        assert!(json_str.contains("personal"));
        assert!(json_str.contains("user@example.com"));
    }
}
