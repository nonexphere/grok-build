//! Path resolution for the multi-provider credential store layout.
//!
//! All credentials live under `<home>/auth/`. The store takes an explicit
//! `home` directory so tests can isolate state under a `tempfile::TempDir`.

use std::path::{Path, PathBuf};

use xai_grok_auth::CredentialKey;

/// The subdirectory under grok home holding all multi-provider credential state.
pub const AUTH_DIR: &str = "auth";

const ACCOUNTS_FILE: &str = "accounts.json";
const SECRETS_FILE: &str = "file-secrets.json";
const LOCKS_DIR: &str = "locks";
/// Write-ahead record for multi-file refresh transactions (B6).
const TXN_JOURNAL: &str = "credential-txn.journal";

/// Resolved on-disk layout for the multi-provider credential store rooted
/// at a grok home directory.
#[derive(Debug, Clone)]
pub struct StorePaths {
    auth_dir: PathBuf,
    accounts_file: PathBuf,
    accounts_lock: PathBuf,
    secrets_file: PathBuf,
    secrets_lock: PathBuf,
    locks_dir: PathBuf,
    txn_journal: PathBuf,
}

impl StorePaths {
    /// Resolve the layout for `home`. Does not create any directories.
    pub fn new(home: &Path) -> Self {
        let auth_dir = home.join(AUTH_DIR);
        let accounts_file = auth_dir.join(ACCOUNTS_FILE);
        let accounts_lock = auth_dir.join(format!("{ACCOUNTS_FILE}.lock"));
        let secrets_file = auth_dir.join(SECRETS_FILE);
        let secrets_lock = auth_dir.join(format!("{SECRETS_FILE}.lock"));
        let locks_dir = auth_dir.join(LOCKS_DIR);
        let txn_journal = auth_dir.join(TXN_JOURNAL);
        Self {
            auth_dir,
            accounts_file,
            accounts_lock,
            secrets_file,
            secrets_lock,
            locks_dir,
            txn_journal,
        }
    }

    pub fn auth_dir(&self) -> &Path {
        &self.auth_dir
    }
    pub fn accounts_file(&self) -> &Path {
        &self.accounts_file
    }
    pub fn accounts_lock(&self) -> &Path {
        &self.accounts_lock
    }
    pub fn secrets_file(&self) -> &Path {
        &self.secrets_file
    }
    pub fn secrets_lock(&self) -> &Path {
        &self.secrets_lock
    }
    pub fn locks_dir(&self) -> &Path {
        &self.locks_dir
    }
    pub fn txn_journal(&self) -> &Path {
        &self.txn_journal
    }

    /// `<home>/auth/locks/<provider>/<credential-id>.lock`.
    pub fn credential_lock(&self, key: &CredentialKey) -> PathBuf {
        self.locks_dir
            .join(key.provider.as_str())
            .join(format!("{}.lock", key.credential_id))
    }
}
