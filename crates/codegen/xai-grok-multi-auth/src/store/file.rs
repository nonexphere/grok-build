//! File-backed [`CredentialStore`] — the primary backend.
//!
//! Metadata lives in `<home>/auth/accounts.json`; secret material lives in
//! `<home>/auth/file-secrets.json` keyed by credential id. Both files are
//! written atomically with owner-only permissions. Concurrency: an in-process
//! `tokio::sync::Mutex` serializes mutations; cross-process safety via
//! advisory `flock`s.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;

use xai_grok_auth::{
    CompareAndSwapError, CredentialId, CredentialKey, CredentialLockGuard, CredentialLockPurpose,
    CredentialMetadata, CredentialSecret, CredentialStore, CredentialUpdate, NewCredentialRecord,
    ProviderId, StoreError,
};

use super::lock::{acquire_blocking, acquire_credential_lock, FileLockGuard};
use super::metadata::{
    commit_accounts_and_secrets, load_accounts, load_secrets, recover_pending_txn, save_accounts,
    AccountsFile,
};
use super::paths::StorePaths;

const STORE_LOCK_TIMEOUT: Duration = Duration::from_secs(30);

/// File-backed multi-provider credential store.
#[derive(Debug, Clone)]
pub struct FileCredentialStore {
    paths: Arc<StorePaths>,
    write_lock: Arc<tokio::sync::Mutex<()>>,
}

impl FileCredentialStore {
    /// Create a store handle. Does **not** recover a pending journal here
    /// (AUD-008): recovery runs under the write + file locks on first access
    /// via [`Self::ensure_journal_recovered`] so concurrent constructors cannot
    /// race recover and errors are not swallowed.
    pub fn new(home: PathBuf) -> Self {
        let paths = Arc::new(StorePaths::new(&home));
        Self {
            paths,
            write_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub fn paths(&self) -> &StorePaths {
        &self.paths
    }

    /// Apply a pending dual-file journal under the store write lock + flock.
    /// Fail-loud: corrupt journal is quarantined by [`recover_pending_txn`].
    async fn ensure_journal_recovered(&self) -> Result<(), StoreError> {
        if !self.paths.txn_journal().exists() {
            return Ok(());
        }
        let _guard = self.write_lock.lock().await;
        // Re-check after lock: another task may have finished recover.
        if !self.paths.txn_journal().exists() {
            return Ok(());
        }
        let (_accounts_lock, _secrets_lock) = acquire_both_locks(&self.paths).await?;
        recover_pending_txn(&self.paths)
    }
}

fn find_index(accounts: &AccountsFile, key: &CredentialKey) -> Option<usize> {
    accounts.credentials.iter().position(|m| &m.key == key)
}

fn find_metadata<'a>(
    accounts: &'a AccountsFile,
    key: &CredentialKey,
) -> Option<&'a CredentialMetadata> {
    accounts.credentials.iter().find(|m| &m.key == key)
}

fn resolve_unique_alias(
    accounts: &AccountsFile,
    provider: &ProviderId,
    requested: Option<&str>,
) -> String {
    let base = requested
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "default".to_string());

    let taken: std::collections::HashSet<&str> = accounts
        .credentials
        .iter()
        .filter(|m| &m.key.provider == provider)
        .map(|m| m.alias.as_str())
        .collect();

    if !taken.contains(base.as_str()) {
        return base;
    }
    for i in 2..u64::MAX {
        let candidate = format!("{base}-{i}");
        if !taken.contains(candidate.as_str()) {
            return candidate;
        }
    }
    format!("{base}-{}", CredentialId::new())
}

async fn acquire_both_locks(
    paths: &StorePaths,
) -> Result<(FileLockGuard, FileLockGuard), StoreError> {
    let accounts_lock =
        acquire_blocking(paths.accounts_lock().to_path_buf(), STORE_LOCK_TIMEOUT).await?;
    let secrets_lock =
        acquire_blocking(paths.secrets_lock().to_path_buf(), STORE_LOCK_TIMEOUT).await?;
    Ok((accounts_lock, secrets_lock))
}

fn parse_credential_id(s: &str) -> Option<CredentialId> {
    uuid::Uuid::parse_str(s).ok().map(CredentialId::from_uuid)
}

#[async_trait]
impl CredentialStore for FileCredentialStore {
    async fn list_providers(&self) -> Result<Vec<ProviderId>, StoreError> {
        self.ensure_journal_recovered().await?;
        let accounts = load_accounts(&self.paths)?;
        let mut providers: BTreeMap<String, ProviderId> = BTreeMap::new();
        for m in &accounts.credentials {
            providers
                .entry(m.key.provider.as_str().to_string())
                .or_insert_with(|| m.key.provider.clone());
        }
        Ok(providers.into_values().collect())
    }

    async fn list_accounts(
        &self,
        provider: &ProviderId,
    ) -> Result<Vec<CredentialMetadata>, StoreError> {
        self.ensure_journal_recovered().await?;
        let accounts = load_accounts(&self.paths)?;
        Ok(accounts
            .credentials
            .into_iter()
            .filter(|m| &m.key.provider == provider)
            .collect())
    }

    async fn resolve_alias(
        &self,
        provider: &ProviderId,
        alias: &str,
    ) -> Result<Option<CredentialKey>, StoreError> {
        self.ensure_journal_recovered().await?;
        let accounts = load_accounts(&self.paths)?;
        Ok(accounts
            .credentials
            .into_iter()
            .find(|m| &m.key.provider == provider && m.alias == alias)
            .map(|m| m.key))
    }

    async fn default_account(&self, provider: &ProviderId) -> Result<Option<CredentialKey>, StoreError> {
        self.ensure_journal_recovered().await?;
        let accounts = load_accounts(&self.paths)?;
        match accounts.defaults.get(provider.as_str()) {
            Some(id_str) => match parse_credential_id(id_str) {
                Some(credential_id) => {
                    let key = CredentialKey {
                        provider: provider.clone(),
                        credential_id,
                    };
                    if find_metadata(&accounts, &key).is_some() {
                        Ok(Some(key))
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    async fn set_default_account(&self, key: &CredentialKey) -> Result<(), StoreError> {
        let _guard = self.write_lock.lock().await;
        // Both locks: journal recover may rewrite secrets + accounts.
        let (_accounts_lock, _secrets_lock) = acquire_both_locks(&self.paths).await?;
        recover_pending_txn(&self.paths)?;

        let mut accounts = load_accounts(&self.paths)?;
        if find_index(&accounts, key).is_none() {
            return Err(StoreError::NotFound);
        }
        accounts
            .defaults
            .insert(key.provider.as_str().to_string(), key.credential_id.to_string());
        save_accounts(&self.paths, &accounts)?;
        Ok(())
    }

    async fn load_metadata(&self, key: &CredentialKey) -> Result<Option<CredentialMetadata>, StoreError> {
        self.ensure_journal_recovered().await?;
        let accounts = load_accounts(&self.paths)?;
        Ok(find_metadata(&accounts, key).cloned())
    }

    async fn load_secret(&self, key: &CredentialKey) -> Result<Option<CredentialSecret>, StoreError> {
        self.ensure_journal_recovered().await?;
        let secrets = load_secrets(&self.paths)?;
        Ok(secrets
            .secrets
            .get(&key.credential_id.to_string())
            .cloned())
    }

    async fn create(&self, record: NewCredentialRecord) -> Result<CredentialMetadata, StoreError> {
        let _guard = self.write_lock.lock().await;
        let (_accounts_lock, _secrets_lock) = acquire_both_locks(&self.paths).await?;
        recover_pending_txn(&self.paths)?;

        let mut accounts = load_accounts(&self.paths)?;
        let mut secrets = load_secrets(&self.paths)?;

        let credential_id = CredentialId::new();
        let alias =
            resolve_unique_alias(&accounts, &record.provider, record.requested_alias.as_deref());
        let now = Utc::now();

        let metadata = CredentialMetadata {
            schema_version: 1,
            key: CredentialKey {
                provider: record.provider.clone(),
                credential_id,
            },
            alias,
            account: record.account.clone(),
            created_at: now,
            updated_at: now,
            last_used_at: None,
            expires_at: record.expires_at,
            status: xai_grok_auth::CredentialStatus::Ready,
            generation: 1,
            secret_backend: record.backend,
        };

        secrets
            .secrets
            .insert(credential_id.to_string(), record.secret);
        accounts.credentials.push(metadata.clone());
        commit_accounts_and_secrets(&self.paths, &accounts, &secrets)?;

        Ok(metadata)
    }

    async fn compare_and_swap(
        &self,
        expected_generation: u64,
        update: CredentialUpdate,
    ) -> Result<CredentialMetadata, CompareAndSwapError> {
        let _guard = self.write_lock.lock().await;
        // Always hold both file locks: journal recover may rewrite secrets, and
        // CAS with secret update is a dual-file commit (AUD-008).
        let (_accounts_lock, _secrets_lock) = acquire_both_locks(&self.paths)
            .await
            .map_err(CompareAndSwapError::Store)?;
        recover_pending_txn(&self.paths).map_err(CompareAndSwapError::Store)?;

        let mut accounts = load_accounts(&self.paths)?;
        let idx = find_index(&accounts, &update.key)
            .ok_or(CompareAndSwapError::Store(StoreError::NotFound))?;

        if accounts.credentials[idx].generation != expected_generation {
            return Err(CompareAndSwapError::GenerationChanged);
        }

        let metadata = &mut accounts.credentials[idx];
        if let Some(account) = update.account {
            metadata.account = account;
        }
        if let Some(status) = update.status {
            metadata.status = status;
        }
        if let Some(expires_at) = update.expires_at {
            metadata.expires_at = expires_at;
        }
        metadata.updated_at = update.updated_at;
        metadata.generation = metadata.generation.checked_add(1).ok_or_else(|| {
            CompareAndSwapError::Store(StoreError::Backend("generation overflow".to_string()))
        })?;

        let new_metadata = metadata.clone();

        if let Some(secret) = update.secret {
            let mut secrets = load_secrets(&self.paths)?;
            secrets
                .secrets
                .insert(update.key.credential_id.to_string(), secret);
            // Journaled dual-file commit: secrets + metadata as one logical txn.
            commit_accounts_and_secrets(&self.paths, &accounts, &secrets)?;
        } else {
            save_accounts(&self.paths, &accounts)?;
        }

        Ok(new_metadata)
    }

    async fn delete(&self, key: &CredentialKey) -> Result<bool, StoreError> {
        let _guard = self.write_lock.lock().await;
        let (_accounts_lock, _secrets_lock) = acquire_both_locks(&self.paths).await?;
        recover_pending_txn(&self.paths)?;

        let mut accounts = load_accounts(&self.paths)?;
        let Some(idx) = find_index(&accounts, key) else {
            return Ok(false);
        };

        accounts.credentials.remove(idx);
        let id_str = key.credential_id.to_string();
        if accounts.defaults.get(key.provider.as_str()) == Some(&id_str) {
            accounts.defaults.remove(key.provider.as_str());
        }

        let mut secrets = load_secrets(&self.paths)?;
        secrets.secrets.remove(&key.credential_id.to_string());
        commit_accounts_and_secrets(&self.paths, &accounts, &secrets)?;

        Ok(true)
    }

    async fn acquire_lock(
        &self,
        key: &CredentialKey,
        _purpose: CredentialLockPurpose,
    ) -> Result<Box<dyn CredentialLockGuard>, StoreError> {
        let lock_path = self.paths.credential_lock(key);
        acquire_credential_lock(lock_path, STORE_LOCK_TIMEOUT).await
    }
}
