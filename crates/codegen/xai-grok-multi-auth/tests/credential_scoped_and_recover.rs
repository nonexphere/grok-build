//! GATE 1/2: credential-scoped catalog keys + generation-aware 401 recovery.

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Utc;
use xai_grok_auth::{
    CredentialSecret, CredentialStore, NewCredentialRecord, ProviderAccountInfo, ProviderId,
    SecretBackendKind, SecretString, TokenUseReason,
};
use xai_grok_multi_auth::provider_model_key::{format_provider_model_key, parse_provider_model_key};
use xai_grok_multi_auth::registry;
use xai_grok_multi_auth::store::ephemeral::EphemeralCredentialStore;
use xai_grok_multi_auth::token_manager::TokenManager;
use xai_grok_auth::{CredentialBinding, SentCredentialStamp};

#[test]
fn two_accounts_same_slug_get_distinct_catalog_keys() {
    let provider = ProviderId::new_unchecked("codex");
    let a = xai_grok_auth::CredentialId::new();
    let b = xai_grok_auth::CredentialId::new();
    let key_a = format_provider_model_key(&provider, a, "gpt-5.4");
    let key_b = format_provider_model_key(&provider, b, "gpt-5.4");
    assert_ne!(key_a, key_b);
    let pa = parse_provider_model_key(&key_a).unwrap();
    let pb = parse_provider_model_key(&key_b).unwrap();
    assert_eq!(pa.model, "gpt-5.4");
    assert_eq!(pb.model, "gpt-5.4");
    assert_ne!(pa.credential_id, pb.credential_id);
    // Runtime binding is independent of catalog iteration order.
    assert_eq!(pa.to_model_binding().credential, Some(a));
    assert_eq!(pb.to_model_binding().credential, Some(b));
}

#[tokio::test]
async fn stale_generation_401_does_not_double_refresh() {
    let store = Arc::new(EphemeralCredentialStore::new());
    let provider = ProviderId::new_unchecked("codex");
    let mut account = ProviderAccountInfo::default();
    account
        .metadata
        .insert("chatgpt_account_id".to_string(), "acct-stale".into());
    let meta = store
        .create(NewCredentialRecord {
            provider: provider.clone(),
            requested_alias: Some("a".into()),
            account: account.clone(),
            secret: CredentialSecret {
                access_token: SecretString::from_str("access-1"),
                refresh_token: Some(SecretString::from_str("refresh-1")),
                id_token: None,
                fields: BTreeMap::new(),
            },
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            backend: SecretBackendKind::Ephemeral,
        })
        .await
        .unwrap();

    // Simulate concurrent refresh: bump generation with a new secret.
    let _ = store
        .compare_and_swap(
            meta.generation,
            xai_grok_auth::CredentialUpdate {
                key: meta.key.clone(),
                account: None,
                secret: Some(CredentialSecret {
                    access_token: SecretString::from_str("access-2"),
                    refresh_token: Some(SecretString::from_str("refresh-2")),
                    id_token: None,
                    fields: BTreeMap::new(),
                }),
                expires_at: Some(Some(Utc::now() + chrono::Duration::hours(1))),
                status: None,
                updated_at: Utc::now(),
            },
        )
        .await
        .unwrap();

    let registry = Arc::new(registry::build_registry(false));
    let tm = TokenManager::with_issuer(
        store.clone(),
        registry,
        "https://auth.openai.com".into(),
    );
    let fp = xai_grok_multi_auth::fingerprint::compute_fingerprint(
        &provider,
        "https://auth.openai.com",
        &account,
    );
    let binding = CredentialBinding {
        key: meta.key.clone(),
        expected_account: fp.clone(),
    };
    // Sent stamp still at generation 1 while store is at 2.
    let sent = SentCredentialStamp {
        key: meta.key.clone(),
        generation: 1,
        account_fingerprint: fp,
    };
    let outcome = tm.recover_unauthorized(&binding, &sent, 401).await.unwrap();
    assert!(matches!(
        outcome,
        xai_grok_auth::UnauthorizedRecovery::RetryWithCurrentCredential
    ));
    let _ = TokenUseReason::UnauthorizedRecovery;
}

#[test]
fn codex_oauth_login_block_reason_when_unapproved() {
    // Document fail-closed: when neither approval nor client id is set.
    let approved = std::env::var_os("GROK_CODEX_OAUTH_APPROVED").is_some();
    let client = std::env::var("GROK_CODEX_CLIENT_ID")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    if !approved && !client && std::env::var_os("GROK_DISABLE_CODEX_AUTH").is_none() {
        assert!(
            xai_grok_multi_auth::kill_switch::codex_oauth_login_block_reason().is_some(),
            "B5 fail-closed must block product login without approval"
        );
    }
}

#[tokio::test]
async fn journal_recovers_dual_file_commit_after_crash_marker() {
    use xai_grok_multi_auth::store::file::FileCredentialStore;
    use xai_grok_multi_auth::store::metadata::{
        commit_accounts_and_secrets, load_accounts, load_secrets, recover_pending_txn,
        AccountsFile, SecretsFile, CredentialTxnJournal, TXN_JOURNAL_SCHEMA_VERSION,
    };
    use xai_grok_multi_auth::store::paths::StorePaths;
    use std::fs;

    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().to_path_buf();
    let store = FileCredentialStore::new(home.clone());
    let provider = ProviderId::new_unchecked("codex");
    let mut account = ProviderAccountInfo::default();
    account.metadata.insert("chatgpt_account_id".into(), "a1".into());
    let meta = store
        .create(NewCredentialRecord {
            provider: provider.clone(),
            requested_alias: Some("j".into()),
            account,
            secret: CredentialSecret {
                access_token: SecretString::from_str("tok-a"),
                refresh_token: Some(SecretString::from_str("ref-a")),
                id_token: None,
                fields: BTreeMap::new(),
            },
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            backend: SecretBackendKind::File,
        })
        .await
        .unwrap();

    let paths = StorePaths::new(&home);
    let mut accounts = load_accounts(&paths).unwrap();
    let mut secrets = load_secrets(&paths).unwrap();
    // Simulate post-refresh state in a journal left behind (crash after journal write).
    secrets.secrets.insert(
        meta.key.credential_id.to_string(),
        CredentialSecret {
            access_token: SecretString::from_str("tok-b"),
            refresh_token: Some(SecretString::from_str("ref-b")),
            id_token: None,
            fields: BTreeMap::new(),
        },
    );
    for m in accounts.credentials.iter_mut() {
        if m.key == meta.key {
            m.generation = 99;
        }
    }
    let journal = CredentialTxnJournal {
        schema_version: TXN_JOURNAL_SCHEMA_VERSION,
        accounts: accounts.clone(),
        secrets: secrets.clone(),
    };
    // Leave only the journal (as if crash after journal, before apply finished).
    let raw = serde_json::to_vec_pretty(&journal).unwrap();
    fs::create_dir_all(paths.auth_dir()).unwrap();
    fs::write(paths.txn_journal(), raw).unwrap();

    recover_pending_txn(&paths).unwrap();
    assert!(!paths.txn_journal().exists());
    let reloaded = store.load(&meta.key).await.unwrap().unwrap();
    assert_eq!(reloaded.secret.access_token.expose(), "tok-b");
    assert_eq!(reloaded.metadata.generation, 99);
    let _ = commit_accounts_and_secrets; // silence if unused
}
