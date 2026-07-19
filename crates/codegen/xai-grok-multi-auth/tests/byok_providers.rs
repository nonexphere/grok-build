//! C5-B offline contract tests for BYOK providers (OpenRouter / Groq / Cloudflare).
//!
//! These tests assert the **intended** provider-surface contract:
//!   1. The three BYOK verticals are registered in the default registry and
//!      advertise the `API_KEY_LOGIN` capability.
//!   2. `LoginCoordinator::run_api_key_login` rejects unknown/unregistered
//!      provider ids (RED against the foundation that ignored the registry).
//!   3. `run_api_key_login` rejects registered providers that do NOT advertise
//!      `API_KEY_LOGIN` (e.g. xAI, Codex).
//!   4. `ByokAuthProvider::build_request_auth` emits a static
//!      `Authorization: Bearer <opaque>` header and nothing else that leaks.
//!   5. `ByokAuthProvider::resolve_endpoint` produces the per-vertical
//!      inference / models URL shape.
//!   6. Third-party BYOK never falls back to `XAI_API_KEY`.
//!
//! No live credentials: all keys are synthetic `sk-test-*` values. Live smoke
//! is gated separately (see `live_byok_*.rs`).

use std::sync::Arc;

use http::header::AUTHORIZATION;
use tempfile::TempDir;
use xai_grok_auth::{
    AuthProvider, CredentialId, CredentialKey, CredentialMetadata, CredentialSecret,
    CredentialStatus, CredentialStore, LogoutRequest, ProviderAccountInfo, ProviderCapabilities,
    ProviderEndpointKind, ProviderEndpointRequest, ProviderId, ProviderRequestAuth,
    RequestAuthContext, RequestKind, SecretBackendKind, SecretString, StoredCredential,
};
use xai_grok_multi_auth::login_coordinator::LoginCoordinator;
use xai_grok_multi_auth::providers::byok;
use xai_grok_multi_auth::registry;
use xai_grok_multi_auth::store::ephemeral::EphemeralCredentialStore;
use xai_grok_multi_auth::store::file::FileCredentialStore;

// ── Helpers ─────────────────────────────────────────────────────────

fn openrouter() -> ProviderId {
    ProviderId::new_unchecked("openrouter")
}
fn groq() -> ProviderId {
    ProviderId::new_unchecked("groq")
}
fn cloudflare() -> ProviderId {
    ProviderId::new_unchecked("cloudflare")
}
fn xai() -> ProviderId {
    ProviderId::new_unchecked("xai")
}
fn codex() -> ProviderId {
    ProviderId::new_unchecked("codex")
}

fn synthetic_key() -> SecretString {
    SecretString::from_str("sk-test-byok-never-log-0123456789")
}

fn stored_credential(
    provider: &ProviderId,
    api_key: &str,
    account_meta: Vec<(&str, &str)>,
) -> StoredCredential {
    let now = chrono::Utc::now();
    let mut account = ProviderAccountInfo::default();
    for (k, v) in account_meta {
        account.metadata.insert(k.to_string(), v.to_string());
    }
    let key = CredentialKey {
        provider: provider.clone(),
        credential_id: CredentialId::new(),
    };
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("api_key".to_string(), SecretString::from_str(api_key));
    StoredCredential {
        metadata: CredentialMetadata {
            schema_version: 1,
            key,
            alias: "default".to_string(),
            account,
            created_at: now,
            updated_at: now,
            last_used_at: None,
            expires_at: None,
            status: CredentialStatus::Ready,
            generation: 1,
            secret_backend: SecretBackendKind::Ephemeral,
        },
        secret: CredentialSecret {
            access_token: SecretString::from_str(api_key),
            refresh_token: None,
            id_token: None,
            fields,
        },
    }
}

// ── Test 1: default registry registers the three BYOK verticals ─────

#[test]
fn byok_registry_registers_three_verticals_with_api_key_login() {
    let registry = registry::build_default_registry();
    for id in [openrouter(), groq(), cloudflare()] {
        let provider = registry.get(&id).expect("BYOK provider must be registered");
        let desc = provider.descriptor();
        assert_eq!(desc.id.as_str(), id.as_str());
        assert!(
            desc.capabilities
                .contains(ProviderCapabilities::API_KEY_LOGIN),
            "{} must advertise API_KEY_LOGIN, got bits={:#x}",
            id.as_str(),
            desc.capabilities.bits()
        );
    }
}

// ── Test 2: unknown provider rejected on API-key login (RED→GREEN) ──

#[tokio::test]
async fn byok_api_key_login_rejects_unknown_provider() {
    let store = Arc::new(EphemeralCredentialStore::new());
    let registry = Arc::new(registry::build_default_registry());
    let coord = LoginCoordinator::new(store, registry);
    let unknown = ProviderId::new_unchecked("unknown-byok-provider");
    let err = coord
        .run_api_key_login(&unknown, Some(synthetic_key()))
        .await
        .expect_err("unknown provider must be rejected");
    let msg = format!("{err}");
    assert!(
        msg.to_lowercase().contains("not registered")
            || msg.to_lowercase().contains("unknown")
            || msg.to_lowercase().contains("invalid"),
        "expected registry rejection, got: {msg}"
    );
}

// ── Test 3: provider without API_KEY_LOGIN rejected for API-key login ──

#[tokio::test]
async fn byok_api_key_login_rejects_provider_without_api_key_capability() {
    let store = Arc::new(EphemeralCredentialStore::new());
    let registry = Arc::new(registry::build_default_registry());
    let coord = LoginCoordinator::new(store, registry);
    // xAI advertises empty capabilities (no API_KEY_LOGIN).
    let err = coord
        .run_api_key_login(&xai(), Some(synthetic_key()))
        .await
        .expect_err("xAI must be rejected for API-key login (no API_KEY_LOGIN)");
    let msg = format!("{err}").to_lowercase();
    assert!(
        msg.contains("api-key") || msg.contains("api_key") || msg.contains("does not support"),
        "expected API_KEY_LOGIN capability rejection, got: {msg}"
    );
    // Codex also lacks API_KEY_LOGIN.
    let err = coord
        .run_api_key_login(&codex(), Some(synthetic_key()))
        .await
        .expect_err("Codex must be rejected for API-key login");
    let msg = format!("{err}").to_lowercase();
    assert!(
        msg.contains("api-key") || msg.contains("api_key") || msg.contains("does not support"),
        "expected API_KEY_LOGIN capability rejection, got: {msg}"
    );
}

// ── Test 4: registered BYOK providers accept API-key login ──────────

#[tokio::test]
async fn byok_api_key_login_persists_for_registered_byok_providers() {
    let store = Arc::new(EphemeralCredentialStore::new());
    let registry = Arc::new(registry::build_default_registry());
    let coord = LoginCoordinator::new(store.clone(), registry);
    for id in [openrouter(), groq(), cloudflare()] {
        let meta = coord
            .run_api_key_login(&id, Some(synthetic_key()))
            .await
            .expect("registered BYOK provider must accept API-key login");
        assert_eq!(meta.key.provider.as_str(), id.as_str());
        let debug = format!("{meta:?}");
        assert!(
            !debug.contains("sk-test-byok-never-log"),
            "metadata leaked secret for {}: {debug}",
            id.as_str()
        );
        let accounts = store.list_accounts(&id).await.unwrap();
        assert_eq!(accounts.len(), 1, "{} should have one account", id.as_str());
    }
}

// ── Test 5: request auth is a static bearer and nothing else leaks ──

#[test]
fn byok_openrouter_request_auth_is_static_bearer() {
    let registry = registry::build_default_registry();
    let provider = registry.get(&openrouter()).unwrap();
    let cred = stored_credential(&openrouter(), "sk-test-openrouter-key", vec![]);
    let endpoint = url::Url::parse("https://openrouter.ai/api/v1/chat/completions").unwrap();
    let method = http::Method::POST;
    let ctx = RequestAuthContext {
        endpoint: &endpoint,
        method: &method,
        credential: Some(&cred),
        request_kind: RequestKind::Inference,
    };
    let ProviderRequestAuth { headers } = provider.build_request_auth(ctx).unwrap();
    let auth = headers
        .get(AUTHORIZATION)
        .expect("Authorization header must be present")
        .to_str()
        .unwrap();
    assert_eq!(auth, "Bearer sk-test-openrouter-key");
    // No other header carries the secret.
    for (name, value) in headers.iter() {
        if name == AUTHORIZATION {
            continue;
        }
        let v = value.to_str().unwrap_or_default();
        assert!(
            !v.contains("sk-test-openrouter-key"),
            "secret leaked into header {name}: {v}"
        );
    }
}

#[test]
fn byok_groq_request_auth_is_static_bearer() {
    let registry = registry::build_default_registry();
    let provider = registry.get(&groq()).unwrap();
    let cred = stored_credential(&groq(), "sk-test-groq-key", vec![]);
    let endpoint = url::Url::parse("https://api.groq.com/openai/v1/chat/completions").unwrap();
    let method = http::Method::POST;
    let ctx = RequestAuthContext {
        endpoint: &endpoint,
        method: &method,
        credential: Some(&cred),
        request_kind: RequestKind::Inference,
    };
    let ProviderRequestAuth { headers } = provider.build_request_auth(ctx).unwrap();
    let auth = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
    assert_eq!(auth, "Bearer sk-test-groq-key");
}

#[test]
fn byok_cloudflare_request_auth_is_static_bearer() {
    let registry = registry::build_default_registry();
    let provider = registry.get(&cloudflare()).unwrap();
    let cred = stored_credential(
        &cloudflare(),
        "sk-test-cloudflare-key",
        vec![("cloudflare_account_id", "acct-test-123")],
    );
    let endpoint = url::Url::parse(
        "https://api.cloudflare.com/client/v4/accounts/acct-test-123/ai/v1/chat/completions",
    )
    .unwrap();
    let method = http::Method::POST;
    let ctx = RequestAuthContext {
        endpoint: &endpoint,
        method: &method,
        credential: Some(&cred),
        request_kind: RequestKind::Inference,
    };
    let ProviderRequestAuth { headers } = provider.build_request_auth(ctx).unwrap();
    let auth = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
    assert_eq!(auth, "Bearer sk-test-cloudflare-key");
}

// ── Test 6: endpoint resolution per vertical ───────────────────────

#[test]
fn byok_openrouter_endpoint_resolution() {
    let registry = registry::build_default_registry();
    let provider = registry.get(&openrouter()).unwrap();
    let cred = stored_credential(&openrouter(), "sk-test", vec![]);
    let inference = provider
        .resolve_endpoint(ProviderEndpointRequest {
            kind: ProviderEndpointKind::Inference,
            credential: Some(&cred),
        })
        .unwrap();
    assert_eq!(
        inference.as_str(),
        "https://openrouter.ai/api/v1/chat/completions"
    );
    let models = provider
        .resolve_endpoint(ProviderEndpointRequest {
            kind: ProviderEndpointKind::Models,
            credential: Some(&cred),
        })
        .unwrap();
    assert_eq!(models.as_str(), "https://openrouter.ai/api/v1/models");
}

#[test]
fn byok_groq_endpoint_resolution() {
    let registry = registry::build_default_registry();
    let provider = registry.get(&groq()).unwrap();
    let cred = stored_credential(&groq(), "sk-test", vec![]);
    let inference = provider
        .resolve_endpoint(ProviderEndpointRequest {
            kind: ProviderEndpointKind::Inference,
            credential: Some(&cred),
        })
        .unwrap();
    assert_eq!(
        inference.as_str(),
        "https://api.groq.com/openai/v1/chat/completions"
    );
    let models = provider
        .resolve_endpoint(ProviderEndpointRequest {
            kind: ProviderEndpointKind::Models,
            credential: Some(&cred),
        })
        .unwrap();
    assert_eq!(models.as_str(), "https://api.groq.com/openai/v1/models");
}

#[test]
fn byok_cloudflare_endpoint_resolution_requires_account_id() {
    let registry = registry::build_default_registry();
    let provider = registry.get(&cloudflare()).unwrap();
    // Without account metadata, endpoint resolution must fail loudly.
    let cred_no_account = stored_credential(&cloudflare(), "sk-test", vec![]);
    let err = provider
        .resolve_endpoint(ProviderEndpointRequest {
            kind: ProviderEndpointKind::Inference,
            credential: Some(&cred_no_account),
        })
        .expect_err("Cloudflare without account id must fail");
    assert!(format!("{err}").to_lowercase().contains("account"));

    // With account metadata, the URL includes the account id in the path.
    let cred = stored_credential(
        &cloudflare(),
        "sk-test",
        vec![("cloudflare_account_id", "acct-test-123")],
    );
    let inference = provider
        .resolve_endpoint(ProviderEndpointRequest {
            kind: ProviderEndpointKind::Inference,
            credential: Some(&cred),
        })
        .unwrap();
    assert_eq!(
        inference.as_str(),
        "https://api.cloudflare.com/client/v4/accounts/acct-test-123/ai/v1/chat/completions"
    );
}

// ── Test 7: third-party BYOK never falls back to XAI_API_KEY ────────

#[test]
fn byok_reject_xai_api_key_fallback_for_third_party() {
    assert!(byok::reject_xai_api_key_fallback("openrouter", "XAI_API_KEY").is_err());
    assert!(byok::reject_xai_api_key_fallback("groq", "XAI_API_KEY").is_err());
    assert!(byok::reject_xai_api_key_fallback("cloudflare", "XAI_API_KEY").is_err());
    assert!(byok::reject_xai_api_key_fallback("openrouter", "GROK_BYOK_API_KEY").is_ok());
    assert!(byok::reject_xai_api_key_fallback("xai", "XAI_API_KEY").is_ok());
}

// ── Test 8: BYOK provider does not advertise OAuth/refresh caps ─────

#[test]
fn byok_providers_advertise_only_api_key_login() {
    let registry = registry::build_default_registry();
    for id in [openrouter(), groq(), cloudflare()] {
        let desc = registry.get(&id).unwrap().descriptor();
        let caps = desc.capabilities;
        assert!(caps.contains(ProviderCapabilities::API_KEY_LOGIN));
        assert!(!caps.contains(ProviderCapabilities::BROWSER_PKCE));
        assert!(!caps.contains(ProviderCapabilities::DEVICE_CODE));
        assert!(!caps.contains(ProviderCapabilities::REFRESH_TOKEN));
    }
}

// ── Test 9: BYOK logout is honest (no remote revoke claim) ─────────

#[tokio::test]
async fn byok_logout_does_not_claim_remote_revoke() {
    let registry = registry::build_default_registry();
    let provider = registry.get(&openrouter()).unwrap();
    let cred = stored_credential(&openrouter(), "sk-test", vec![]);
    let outcome = AuthProvider::logout(
        provider.as_ref(),
        LogoutRequest {
            credential: &cred,
            revoke: true,
        },
    )
    .await
    .unwrap();
    assert!(
        !outcome.remote_revoked,
        "BYOK API-key logout must not claim remote revoke"
    );
}

// ── Test 10: BYOK get_valid_token returns the stored key (no refresh) ──

#[tokio::test]
async fn byok_get_valid_token_returns_stored_key_without_refresh() {
    let registry = registry::build_default_registry();
    let provider = registry.get(&groq()).unwrap();
    let cred = stored_credential(&groq(), "sk-test-groq-static", vec![]);
    let now = chrono::Utc::now();
    let resolution = AuthProvider::get_valid_token(
        provider.as_ref(),
        xai_grok_auth::TokenRequest {
            credential: &cred,
            reason: xai_grok_auth::TokenUseReason::Inference,
            now,
            early_refresh_window: chrono::Duration::seconds(30),
        },
    )
    .await
    .unwrap();
    assert_eq!(resolution.token.expose(), "sk-test-groq-static");
    assert!(
        resolution.update.is_none(),
        "BYOK must not produce a refresh update"
    );
}

// ── Test 11: BYOK file store persists API-key credential ─────────────

#[tokio::test]
async fn byok_api_key_login_persists_to_file_store() {
    let dir = TempDir::new().unwrap();
    let store = Arc::new(FileCredentialStore::new(dir.path().to_path_buf()));
    let registry = Arc::new(registry::build_default_registry());
    let coord = LoginCoordinator::new(store.clone(), registry);
    let meta = coord
        .run_api_key_login(&openrouter(), Some(synthetic_key()))
        .await
        .unwrap();
    assert_eq!(meta.key.provider.as_str(), "openrouter");
    let accounts = store.list_accounts(&openrouter()).await.unwrap();
    assert_eq!(accounts.len(), 1);
    let loaded = store.load(&meta.key).await.unwrap().unwrap();
    assert_eq!(
        loaded.secret.access_token.expose(),
        "sk-test-byok-never-log-0123456789"
    );
}

// ── Test 12: kill switch disables BYOK registration ─────────────────
//
// Uses the flag-based builder to avoid env mutation that would race with
// concurrent tests calling `build_default_registry()` (which reads the
// `GROK_DISABLE_BYOK_AUTH` env internally).

#[test]
fn byok_kill_switch_disables_registration() {
    // BYOK enabled → all three verticals registered.
    let registry = registry::build_registry_with_flags(false, false);
    for id in [openrouter(), groq(), cloudflare()] {
        assert!(
            registry.get(&id).is_ok(),
            "{} must be registered when BYOK is enabled",
            id.as_str()
        );
    }
    // xAI + Codex remain.
    assert!(registry.get(&xai()).is_ok());
    assert!(registry.get(&codex()).is_ok());

    // BYOK disabled → none of the three registered; xAI + Codex remain.
    let registry = registry::build_registry_with_flags(false, true);
    for id in [openrouter(), groq(), cloudflare()] {
        assert!(
            registry.get(&id).is_err(),
            "{} must not be registered when BYOK is disabled",
            id.as_str()
        );
    }
    assert!(registry.get(&xai()).is_ok());
    assert!(registry.get(&codex()).is_ok());
}

// ── Test 13: BYOK provider supports_credential only for its own id ──

#[test]
fn byok_supports_credential_only_for_own_provider() {
    let registry = registry::build_default_registry();
    let openrouter_provider = registry.get(&openrouter()).unwrap();
    let own_cred = stored_credential(&openrouter(), "sk-test", vec![]);
    let other_cred = stored_credential(&groq(), "sk-test", vec![]);
    assert!(openrouter_provider.supports_credential(&own_cred.metadata));
    assert!(!openrouter_provider.supports_credential(&other_cred.metadata));
}
