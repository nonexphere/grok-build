//! Composite / "auto" backend selection. Forwards to a single inner backend.

use std::sync::Arc;

use async_trait::async_trait;

use xai_grok_auth::{
    CompareAndSwapError, CredentialKey, CredentialLockGuard, CredentialLockPurpose,
    CredentialMetadata, CredentialSecret, CredentialStore, CredentialUpdate, NewCredentialRecord,
    ProviderId, StoreError,
};

/// A composite store that forwards to a single inner backend.
#[derive(Debug)]
pub struct AutoCredentialStore {
    inner: Arc<dyn CredentialStore>,
}

impl AutoCredentialStore {
    pub fn new(inner: Arc<dyn CredentialStore>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl CredentialStore for AutoCredentialStore {
    async fn list_providers(&self) -> Result<Vec<ProviderId>, StoreError> {
        self.inner.list_providers().await
    }
    async fn list_accounts(
        &self,
        provider: &ProviderId,
    ) -> Result<Vec<CredentialMetadata>, StoreError> {
        self.inner.list_accounts(provider).await
    }
    async fn resolve_alias(
        &self,
        provider: &ProviderId,
        alias: &str,
    ) -> Result<Option<CredentialKey>, StoreError> {
        self.inner.resolve_alias(provider, alias).await
    }
    async fn default_account(&self, provider: &ProviderId) -> Result<Option<CredentialKey>, StoreError> {
        self.inner.default_account(provider).await
    }
    async fn set_default_account(&self, key: &CredentialKey) -> Result<(), StoreError> {
        self.inner.set_default_account(key).await
    }
    async fn load_metadata(&self, key: &CredentialKey) -> Result<Option<CredentialMetadata>, StoreError> {
        self.inner.load_metadata(key).await
    }
    async fn load_secret(&self, key: &CredentialKey) -> Result<Option<CredentialSecret>, StoreError> {
        self.inner.load_secret(key).await
    }
    async fn create(&self, record: NewCredentialRecord) -> Result<CredentialMetadata, StoreError> {
        self.inner.create(record).await
    }
    async fn compare_and_swap(
        &self,
        expected_generation: u64,
        update: CredentialUpdate,
    ) -> Result<CredentialMetadata, CompareAndSwapError> {
        self.inner.compare_and_swap(expected_generation, update).await
    }
    async fn delete(&self, key: &CredentialKey) -> Result<bool, StoreError> {
        self.inner.delete(key).await
    }
    async fn acquire_lock(
        &self,
        key: &CredentialKey,
        purpose: CredentialLockPurpose,
    ) -> Result<Box<dyn CredentialLockGuard>, StoreError> {
        self.inner.acquire_lock(key, purpose).await
    }
}
