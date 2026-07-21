//! Regression: multi-provider blocking helpers must not panic on a Tokio
//! current-thread runtime (session LocalSet workers).

use std::path::PathBuf;

use chrono::Utc;
use std::collections::BTreeMap;
use xai_grok_auth::{
    CredentialSecret, CredentialStore, NewCredentialRecord, ProviderAccountInfo, ProviderId,
    SecretBackendKind, SecretString,
};
use xai_grok_multi_auth::provider_model_key::{
    format_provider_model_key, parse_provider_model_key,
};
use xai_grok_multi_auth::store::file::FileCredentialStore;
use xai_grok_multi_auth::token_resolve;

#[test]
fn block_on_safe_on_current_thread_runtime_does_not_panic() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().to_path_buf();
    let store = FileCredentialStore::new(home.clone());
    let provider = ProviderId::new_unchecked("codex");
    let mut account = ProviderAccountInfo::default();
    account
        .metadata
        .insert("chatgpt_account_id".into(), "acct-ct".into());
    account.provider_account_id = Some("acct-ct".into());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let meta = rt.block_on(async {
        store
            .create(NewCredentialRecord {
                provider: provider.clone(),
                requested_alias: Some("t".into()),
                account,
                secret: CredentialSecret {
                    access_token: SecretString::from_str("access-ct"),
                    refresh_token: Some(SecretString::from_str("refresh-ct")),
                    id_token: None,
                    fields: BTreeMap::new(),
                },
                expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
                backend: SecretBackendKind::File,
            })
            .await
            .unwrap()
    });

    // Nested: already on current-thread handle; must not panic.
    let result = rt.block_on(async {
        // Simulate session worker calling sync BearerResolver path.
        token_resolve::resolve_access_token_blocking(&home, &provider, meta.key.credential_id)
    });
    assert!(
        result.is_ok(),
        "resolve on current-thread must not panic: {result:?}"
    );
    assert_eq!(result.unwrap(), "access-ct");
}

#[test]
fn short_slug_key_helpers_single_and_ambiguous() {
    let provider = ProviderId::new_unchecked("codex");
    let a = xai_grok_auth::CredentialId::new();
    let b = xai_grok_auth::CredentialId::new();
    let k1 = format_provider_model_key(&provider, a, "gpt-5.6-luna");
    let k2 = format_provider_model_key(&provider, b, "gpt-5.6-luna");
    assert_ne!(k1, k2);
    let p1 = parse_provider_model_key(&k1).unwrap();
    let p2 = parse_provider_model_key(&k2).unwrap();
    assert_eq!(p1.model, "gpt-5.6-luna");
    assert_eq!(p2.model, "gpt-5.6-luna");
    assert_ne!(p1.credential_id, p2.credential_id);
}

#[test]
fn block_on_safe_without_runtime_works() {
    // No tokio handle: should create a private runtime.
    let home = PathBuf::from("/nonexistent-goblin-home-for-test");
    let provider = ProviderId::new_unchecked("codex");
    let cred = xai_grok_auth::CredentialId::new();
    let err = token_resolve::resolve_access_token_blocking(&home, &provider, cred);
    // Missing store → Err, not panic.
    assert!(err.is_err());
}
