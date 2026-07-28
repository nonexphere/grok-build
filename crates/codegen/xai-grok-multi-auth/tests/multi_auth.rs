//! Integration tests for the multi-provider credential store and kill switches.
//!
//! Tests 1–6 from the task spec cover the store, and test 14 covers kill
//! switch registration.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tempfile::TempDir;
use xai_grok_multi_auth::fingerprint;
use xai_grok_multi_auth::kill_switch;
use xai_grok_multi_auth::registry;
use xai_grok_multi_auth::store::ephemeral::EphemeralCredentialStore;
use xai_grok_multi_auth::store::file::FileCredentialStore;
use xai_grok_multi_auth::store::paths::StorePaths;

use xai_grok_auth::{
    CredentialSecret, CredentialStore, CredentialUpdate, NewCredentialRecord,
    ProviderAccountInfo, ProviderId, SecretBackendKind, SecretString,
};

// ── Helpers ─────────────────────────────────────────────────────────

fn xai() -> ProviderId {
    ProviderId::new_unchecked("xai")
}
fn codex() -> ProviderId {
    ProviderId::new_unchecked("codex")
}

fn secret_with(token: &str) -> CredentialSecret {
    CredentialSecret {
        access_token: SecretString::from_str(token),
        refresh_token: Some(SecretString::from_str("refresh-token")),
        id_token: None,
        fields: BTreeMap::new(),
    }
}

fn new_record(provider: ProviderId, alias: Option<&str>, token: &str) -> NewCredentialRecord {
    NewCredentialRecord {
        provider,
        requested_alias: alias.map(|s| s.to_string()),
        account: ProviderAccountInfo {
            email: Some("user@example.com".to_string()),
            ..Default::default()
        },
        secret: secret_with(token),
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        backend: SecretBackendKind::File,
    }
}

fn file_store(dir: &TempDir) -> FileCredentialStore {
    FileCredentialStore::new(dir.path().to_path_buf())
}

// ── Test 1: create/load two providers, delete one, other remains ─────

#[tokio::test]
async fn store_create_load_two_providers_delete_one_other_remains() {
    let dir = TempDir::new().unwrap();
    let store = file_store(&dir);

    // Create xAI and Codex credentials.
    let m_xai = store
        .create(new_record(xai(), Some("default"), "xai-token"))
        .await
        .unwrap();
    let m_codex = store
        .create(new_record(codex(), Some("personal"), "codex-token"))
        .await
        .unwrap();

    // Both providers are listed.
    let providers = store.list_providers().await.unwrap();
    assert_eq!(providers.len(), 2);

    // Delete xAI.
    assert!(store.delete(&m_xai.key).await.unwrap());

    // xAI is gone, Codex remains.
    let providers = store.list_providers().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0], codex());

    // Codex credential still loads.
    let secret = store.load_secret(&m_codex.key).await.unwrap().unwrap();
    assert_eq!(secret.access_token.expose(), "codex-token");

    // xAI credential is gone.
    assert!(store.load_metadata(&m_xai.key).await.unwrap().is_none());
    assert!(store.load_secret(&m_xai.key).await.unwrap().is_none());
}

// ── Test 2: CAS generation conflict ──────────────────────────────────

#[tokio::test]
async fn store_cas_generation_conflict() {
    let dir = TempDir::new().unwrap();
    let store = file_store(&dir);

    let metadata = store
        .create(new_record(xai(), Some("default"), "access-1"))
        .await
        .unwrap();

    // CAS with wrong generation fails.
    let update = CredentialUpdate {
        key: metadata.key.clone(),
        account: None,
        secret: Some(secret_with("access-2")),
        expires_at: None,
        status: None,
        updated_at: Utc::now(),
    };
    let err = store
        .compare_and_swap(metadata.generation + 999, update)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        xai_grok_auth::CompareAndSwapError::GenerationChanged
    ));

    // Original secret is untouched.
    let secret = store.load_secret(&metadata.key).await.unwrap().unwrap();
    assert_eq!(secret.access_token.expose(), "access-1");

    // CAS with correct generation succeeds.
    let update = CredentialUpdate {
        key: metadata.key.clone(),
        account: None,
        secret: Some(secret_with("access-2")),
        expires_at: None,
        status: None,
        updated_at: Utc::now(),
    };
    let updated = store
        .compare_and_swap(metadata.generation, update)
        .await
        .unwrap();
    assert_eq!(updated.generation, metadata.generation + 1);
}

// ── Test 3: Unix 0600 on secrets file ────────────────────────────────

#[cfg(unix)]
#[tokio::test]
async fn store_unix_0600_on_secrets_file() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let paths = StorePaths::new(dir.path());
    let store = file_store(&dir);
    store
        .create(new_record(xai(), Some("default"), "secret-token"))
        .await
        .unwrap();

    let mode = std::fs::metadata(paths.secrets_file())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o777,
        0o600,
        "secrets file must be owner-only 0o600"
    );

    // Accounts file should also be 0600.
    let mode = std::fs::metadata(paths.accounts_file())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "accounts file must be owner-only 0o600");
}

// ── Test 4: ephemeral store ──────────────────────────────────────────

#[tokio::test]
async fn store_ephemeral_works_without_disk() {
    let store = EphemeralCredentialStore::new();

    let m = store
        .create(new_record(codex(), Some("personal"), "ephemeral-token"))
        .await
        .unwrap();
    assert_eq!(m.generation, 1);

    let providers = store.list_providers().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0], codex());

    let secret = store.load_secret(&m.key).await.unwrap().unwrap();
    assert_eq!(secret.access_token.expose(), "ephemeral-token");

    // Delete works.
    assert!(store.delete(&m.key).await.unwrap());
    assert!(store.load_metadata(&m.key).await.unwrap().is_none());

    // CAS works.
    let m2 = store
        .create(new_record(xai(), Some("default"), "token-a"))
        .await
        .unwrap();
    let update = CredentialUpdate {
        key: m2.key.clone(),
        account: None,
        secret: Some(secret_with("token-b")),
        expires_at: None,
        status: None,
        updated_at: Utc::now(),
    };
    let updated = store.compare_and_swap(m2.generation, update).await.unwrap();
    assert_eq!(updated.generation, m2.generation + 1);
    let secret = store.load_secret(&m2.key).await.unwrap().unwrap();
    assert_eq!(secret.access_token.expose(), "token-b");
}

// ── Test 5: alias + default account ──────────────────────────────────

#[tokio::test]
async fn store_alias_and_default_account() {
    let dir = TempDir::new().unwrap();
    let store = file_store(&dir);

    // No default initially.
    assert!(store.default_account(&xai()).await.unwrap().is_none());

    let m1 = store
        .create(new_record(xai(), Some("personal"), "t1"))
        .await
        .unwrap();
    let m2 = store
        .create(new_record(xai(), Some("work"), "t2"))
        .await
        .unwrap();

    // Alias resolution.
    assert_eq!(
        store.resolve_alias(&xai(), "personal").await.unwrap().unwrap(),
        m1.key
    );
    assert_eq!(
        store.resolve_alias(&xai(), "work").await.unwrap().unwrap(),
        m2.key
    );
    assert!(store
        .resolve_alias(&xai(), "nonexistent")
        .await
        .unwrap()
        .is_none());

    // Set and get default.
    store.set_default_account(&m1.key).await.unwrap();
    assert_eq!(
        store.default_account(&xai()).await.unwrap().unwrap(),
        m1.key
    );

    store.set_default_account(&m2.key).await.unwrap();
    assert_eq!(
        store.default_account(&xai()).await.unwrap().unwrap(),
        m2.key
    );

    // Duplicate alias gets unique suffix.
    let m3 = store
        .create(new_record(xai(), Some("personal"), "t3"))
        .await
        .unwrap();
    assert_ne!(m3.alias, "personal");
    assert!(m3.alias.starts_with("personal"));

    // No alias defaults to "default".
    let m4 = store
        .create(new_record(codex(), None, "t4"))
        .await
        .unwrap();
    assert_eq!(m4.alias, "default");
}

// ── Test 6: secret Debug redaction ──────────────────────────────────

#[test]
fn secret_debug_redaction() {
    let secret = CredentialSecret {
        access_token: SecretString::from_str("super-secret-access-token"),
        refresh_token: Some(SecretString::from_str("super-secret-refresh-token")),
        id_token: Some(SecretString::from_str("super-secret-id-token")),
        fields: {
            let mut m = BTreeMap::new();
            m.insert("api_key".to_string(), SecretString::from_str("secret-api-key"));
            m
        },
    };

    let debug = format!("{secret:?}");
    assert!(!debug.contains("super-secret-access-token"));
    assert!(!debug.contains("super-secret-refresh-token"));
    assert!(!debug.contains("super-secret-id-token"));
    assert!(!debug.contains("secret-api-key"));
    assert!(debug.contains("redacted"));

    // SecretString Debug also redacts.
    let s = SecretString::from_str("my-secret-value");
    let debug = format!("{s:?}");
    assert!(!debug.contains("my-secret-value"));
    assert!(debug.contains("redacted"));
}

// ── Test 14: kill switch disables codex registration ─────────────────

#[test]
#[serial_test::serial]
fn kill_switch_disables_codex_registration() {
    // Save the original value to restore later.
    let was_set = std::env::var("GROK_DISABLE_CODEX_AUTH").ok();

    // Rust 2024: env set/remove are unsafe (process-global).
    unsafe {
        std::env::set_var("GROK_DISABLE_CODEX_AUTH", "1");
    }
    assert!(kill_switch::codex_auth_disabled());

    // Build registry — Codex should NOT be registered.
    let registry = registry::build_default_registry();
    let descriptors = registry.list();
    let has_codex = descriptors.iter().any(|d| d.id.as_str() == "codex");
    assert!(!has_codex, "Codex should not be registered when kill switch is on");

    // xAI should still be registered.
    let has_xai = descriptors.iter().any(|d| d.id.as_str() == "xai");
    assert!(has_xai, "xAI should still be registered");

    // Unset the kill switch.
    unsafe {
        std::env::remove_var("GROK_DISABLE_CODEX_AUTH");
    }
    assert!(!kill_switch::codex_auth_disabled());

    // Now Codex should be registered.
    let registry = registry::build_default_registry();
    let descriptors = registry.list();
    let has_codex = descriptors.iter().any(|d| d.id.as_str() == "codex");
    assert!(has_codex, "Codex should be registered when kill switch is off");

    // Restore original value.
    if let Some(val) = was_set {
        unsafe {
            std::env::set_var("GROK_DISABLE_CODEX_AUTH", val);
        }
    }
}
