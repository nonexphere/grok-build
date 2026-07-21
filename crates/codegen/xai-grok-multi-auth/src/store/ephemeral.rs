//! In-memory [`CredentialStore`] that never touches disk.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use xai_grok_auth::{
    CompareAndSwapError, CredentialId, CredentialKey, CredentialLockGuard, CredentialLockPurpose,
    CredentialMetadata, CredentialSecret, CredentialStore, CredentialUpdate, NewCredentialRecord,
    ProviderId, StoreError,
};

#[derive(Debug, Default)]
struct ProviderState {
    credentials: HashMap<CredentialId, (CredentialMetadata, CredentialSecret)>,
    aliases: HashMap<String, CredentialId>,
    default: Option<CredentialId>,
}

/// Process-memory-only credential store.
#[derive(Debug, Default)]
pub struct EphemeralCredentialStore {
    state: Arc<tokio::sync::Mutex<BTreeMap<String, ProviderState>>>,
}

#[derive(Debug)]
struct EphemeralLockGuard;
impl CredentialLockGuard for EphemeralLockGuard {}

impl EphemeralCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

fn resolve_unique_alias_ephemeral(p: &ProviderState, requested: Option<&str>) -> String {
    let base = requested
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "default".to_string());

    if !p.aliases.contains_key(&base) {
        return base;
    }
    for i in 2..u64::MAX {
        let candidate = format!("{base}-{i}");
        if !p.aliases.contains_key(&candidate) {
            return candidate;
        }
    }
    format!("{base}-{}", CredentialId::new())
}

#[async_trait]
impl CredentialStore for EphemeralCredentialStore {
    async fn list_providers(&self) -> Result<Vec<ProviderId>, StoreError> {
        let state = self.state.lock().await;
        Ok(state
            .keys()
            .map(|k| ProviderId::new_unchecked(k.clone()))
            .collect())
    }

    async fn list_accounts(
        &self,
        provider: &ProviderId,
    ) -> Result<Vec<CredentialMetadata>, StoreError> {
        let state = self.state.lock().await;
        Ok(state
            .get(provider.as_str())
            .map(|p| p.credentials.values().map(|(m, _)| m.clone()).collect())
            .unwrap_or_default())
    }

    async fn resolve_alias(
        &self,
        provider: &ProviderId,
        alias: &str,
    ) -> Result<Option<CredentialKey>, StoreError> {
        let state = self.state.lock().await;
        Ok(state.get(provider.as_str()).and_then(|p| {
            p.aliases.get(alias).map(|id| CredentialKey {
                provider: provider.clone(),
                credential_id: *id,
            })
        }))
    }

    async fn default_account(
        &self,
        provider: &ProviderId,
    ) -> Result<Option<CredentialKey>, StoreError> {
        let state = self.state.lock().await;
        Ok(state.get(provider.as_str()).and_then(|p| {
            p.default.map(|id| CredentialKey {
                provider: provider.clone(),
                credential_id: id,
            })
        }))
    }

    async fn set_default_account(&self, key: &CredentialKey) -> Result<(), StoreError> {
        let mut state = self.state.lock().await;
        let p = state
            .get_mut(key.provider.as_str())
            .ok_or(StoreError::NotFound)?;
        if !p.credentials.contains_key(&key.credential_id) {
            return Err(StoreError::NotFound);
        }
        p.default = Some(key.credential_id);
        Ok(())
    }

    async fn load_metadata(
        &self,
        key: &CredentialKey,
    ) -> Result<Option<CredentialMetadata>, StoreError> {
        let state = self.state.lock().await;
        Ok(state
            .get(key.provider.as_str())
            .and_then(|p| p.credentials.get(&key.credential_id))
            .map(|(m, _)| m.clone()))
    }

    async fn load_secret(
        &self,
        key: &CredentialKey,
    ) -> Result<Option<CredentialSecret>, StoreError> {
        let state = self.state.lock().await;
        Ok(state
            .get(key.provider.as_str())
            .and_then(|p| p.credentials.get(&key.credential_id))
            .map(|(_, s)| s.clone()))
    }

    async fn create(&self, record: NewCredentialRecord) -> Result<CredentialMetadata, StoreError> {
        let mut state = self.state.lock().await;
        let p = state
            .entry(record.provider.as_str().to_string())
            .or_default();

        let credential_id = CredentialId::new();
        let alias = resolve_unique_alias_ephemeral(p, record.requested_alias.as_deref());
        let now = Utc::now();
        let metadata = CredentialMetadata {
            schema_version: 1,
            key: CredentialKey {
                provider: record.provider.clone(),
                credential_id,
            },
            alias: alias.clone(),
            account: record.account.clone(),
            created_at: now,
            updated_at: now,
            last_used_at: None,
            expires_at: record.expires_at,
            status: xai_grok_auth::CredentialStatus::Ready,
            generation: 1,
            secret_backend: record.backend,
        };

        p.aliases.insert(alias, credential_id);
        p.credentials
            .insert(credential_id, (metadata.clone(), record.secret));

        Ok(metadata)
    }

    async fn compare_and_swap(
        &self,
        expected_generation: u64,
        update: CredentialUpdate,
    ) -> Result<CredentialMetadata, CompareAndSwapError> {
        let mut state = self.state.lock().await;
        let p = state
            .get_mut(update.key.provider.as_str())
            .ok_or(CompareAndSwapError::Store(StoreError::NotFound))?;
        let (metadata, secret) = p
            .credentials
            .get_mut(&update.key.credential_id)
            .ok_or(CompareAndSwapError::Store(StoreError::NotFound))?;

        if metadata.generation != expected_generation {
            return Err(CompareAndSwapError::GenerationChanged);
        }

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

        if let Some(new_secret) = update.secret {
            *secret = new_secret;
        }

        Ok(metadata.clone())
    }

    async fn delete(&self, key: &CredentialKey) -> Result<bool, StoreError> {
        let mut state = self.state.lock().await;
        let Some(p) = state.get_mut(key.provider.as_str()) else {
            return Ok(false);
        };
        if let Some((metadata, _)) = p.credentials.remove(&key.credential_id) {
            p.aliases.remove(&metadata.alias);
            if p.default == Some(key.credential_id) {
                p.default = None;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn acquire_lock(
        &self,
        _key: &CredentialKey,
        _purpose: CredentialLockPurpose,
    ) -> Result<Box<dyn CredentialLockGuard>, StoreError> {
        Ok(Box::new(EphemeralLockGuard))
    }
}
