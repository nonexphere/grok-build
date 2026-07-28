//! On-disk schema and atomic load/save for the multi-provider credential
//! store. Owner-only (0o600) writes on Unix via an inline `open_secure_file`.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use serde::{Deserialize, Serialize};

use xai_grok_auth::{CredentialMetadata, CredentialSecret, StoreError};

use super::paths::StorePaths;

pub const ACCOUNTS_SCHEMA_VERSION: u32 = 1;
pub const SECRETS_SCHEMA_VERSION: u32 = 1;

/// The contents of `accounts.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub credentials: Vec<CredentialMetadata>,
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
}

/// The contents of `file-secrets.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretsFile {
    pub schema_version: u32,
    #[serde(default)]
    pub secrets: BTreeMap<String, CredentialSecret>,
}

// ── Reading ──────────────────────────────────────────────────────────

fn read_json_recover_empty<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<Option<T>, StoreError> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let trimmed = contents.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            serde_json::from_str::<T>(trimmed).map(Some).map_err(|e| {
                StoreError::Corrupt(format!(
                    "{}: {e}",
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "credential file".to_string())
                ))
            })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(StoreError::Backend(e.to_string())),
    }
}

pub fn load_accounts(paths: &StorePaths) -> Result<AccountsFile, StoreError> {
    match read_json_recover_empty::<AccountsFile>(paths.accounts_file())? {
        Some(file) => Ok(file),
        None => Ok(AccountsFile {
            schema_version: ACCOUNTS_SCHEMA_VERSION,
            ..Default::default()
        }),
    }
}

pub fn load_secrets(paths: &StorePaths) -> Result<SecretsFile, StoreError> {
    match read_json_recover_empty::<SecretsFile>(paths.secrets_file())? {
        Some(file) => Ok(file),
        None => Ok(SecretsFile {
            schema_version: SECRETS_SCHEMA_VERSION,
            ..Default::default()
        }),
    }
}

// ── Writing ──────────────────────────────────────────────────────────

/// Open a file for writing with owner-only permissions (Unix 0o600).
fn open_secure_file(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options = OpenOptions::new();
    options.truncate(true).write(true).create(true);
    #[cfg(unix)]
    {
        options.mode(0o600);
    }
    options.open(path)
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_to_store)?;
    }

    let bytes =
        serde_json::to_vec_pretty(value).map_err(|e| StoreError::Backend(format!("serialize: {e}")))?;

    let tmp = path.with_extension(format!(
        "{}.{}.tmp",
        path.extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_else(|| "json".to_string()),
        std::process::id()
    ));

    {
        let mut file = open_secure_file(&tmp).map_err(io_to_store)?;
        file.write_all(&bytes).map_err(io_to_store)?;
        file.flush().map_err(io_to_store)?;
        file.sync_all().map_err(io_to_store)?;
    }

    #[cfg(windows)]
    {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp, path).map_err(io_to_store)?;
    Ok(())
}

pub fn save_accounts(paths: &StorePaths, file: &AccountsFile) -> Result<(), StoreError> {
    write_json_atomic(paths.accounts_file(), file)
}

pub fn save_secrets(paths: &StorePaths, file: &SecretsFile) -> Result<(), StoreError> {
    write_json_atomic(paths.secrets_file(), file)
}

/// Intentional dual-file transaction (accounts + secrets) for B6.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialTxnJournal {
    pub schema_version: u32,
    pub accounts: AccountsFile,
    pub secrets: SecretsFile,
}

pub const TXN_JOURNAL_SCHEMA_VERSION: u32 = 1;

/// Persist both files under a write-ahead journal so a crash between
/// secret and metadata writes can complete from the journal on next open.
pub fn commit_accounts_and_secrets(
    paths: &StorePaths,
    accounts: &AccountsFile,
    secrets: &SecretsFile,
) -> Result<(), StoreError> {
    let journal = CredentialTxnJournal {
        schema_version: TXN_JOURNAL_SCHEMA_VERSION,
        accounts: accounts.clone(),
        secrets: secrets.clone(),
    };
    write_json_atomic(paths.txn_journal(), &journal)?;
    save_secrets(paths, secrets)?;
    save_accounts(paths, accounts)?;
    remove_journal_durable(paths.txn_journal())?;
    Ok(())
}

/// Remove a journal file; `NotFound` is success. Other I/O errors propagate
/// (AUD-008: never silently leave a journal that failed to delete).
fn remove_journal_durable(path: &Path) -> Result<(), StoreError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(StoreError::Backend(format!(
            "failed to remove txn journal {}: {e}",
            path.display()
        ))),
    }
}

/// If a journal remains from a crashed commit, finish applying it.
///
/// Corrupt journals are quarantined (renamed) and returned as
/// [`StoreError::Corrupt`] rather than silently ignored (AUD-008).
pub fn recover_pending_txn(paths: &StorePaths) -> Result<(), StoreError> {
    let path = paths.txn_journal();
    if !path.exists() {
        return Ok(());
    }
    match read_json_recover_empty::<CredentialTxnJournal>(path) {
        Ok(None) => {
            // Empty/truncated journal: drop it so we do not block future commits.
            remove_journal_durable(path)?;
            Ok(())
        }
        Ok(Some(j)) => {
            save_secrets(paths, &j.secrets)?;
            save_accounts(paths, &j.accounts)?;
            remove_journal_durable(path)?;
            Ok(())
        }
        Err(e @ StoreError::Corrupt(_)) => {
            // Quarantine so operators can inspect; fail loud to the caller.
            let quarantine = path.with_extension(format!(
                "journal.quarantine.{}",
                std::process::id()
            ));
            if let Err(rename_err) = fs::rename(path, &quarantine) {
                return Err(StoreError::Corrupt(format!(
                    "txn journal corrupt ({e}) and quarantine failed: {rename_err}"
                )));
            }
            Err(StoreError::Corrupt(format!(
                "txn journal corrupt (quarantined to {}): {e}",
                quarantine.display()
            )))
        }
        Err(e) => Err(e),
    }
}

fn io_to_store(e: std::io::Error) -> StoreError {
    StoreError::Backend(e.to_string())
}
