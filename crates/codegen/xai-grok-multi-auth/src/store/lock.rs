//! Advisory file locks for the multi-provider credential store.
//!
//! Per-credential locks use an exclusive `flock` on
//! `<home>/auth/locks/<provider>/<id>.lock`, returned as an RAII guard.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fs2::FileExt;

use xai_grok_auth::{CredentialLockGuard, StoreError};

/// RAII guard holding an exclusive `flock` on a lock file.
#[derive(Debug)]
pub struct FileLockGuard {
    _file: File,
    path: PathBuf,
}

impl FileLockGuard {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl CredentialLockGuard for FileLockGuard {}

/// Acquire a blocking exclusive `flock` on `lock_path`, creating the file
/// (and parent directories) if needed. Runs on a `spawn_blocking` thread.
pub async fn acquire_blocking(
    lock_path: PathBuf,
    timeout: Duration,
) -> Result<FileLockGuard, StoreError> {
    tokio::task::spawn_blocking(move || acquire_blocking_sync(&lock_path, timeout))
        .await
        .map_err(|e| StoreError::Backend(format!("lock task panicked: {e}")))?
}

fn acquire_blocking_sync(lock_path: &Path, timeout: Duration) -> Result<FileLockGuard, StoreError> {
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_to_store)?;
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .map_err(io_to_store)?;

    match file.try_lock_exclusive() {
        Ok(()) => {
            return Ok(FileLockGuard {
                _file: file,
                path: lock_path.to_path_buf(),
            });
        }
        Err(e) if e.kind() != io::ErrorKind::WouldBlock => {
            return Err(io_to_store(e));
        }
        Err(_) => {}
    }

    let deadline = std::time::Instant::now() + timeout;
    loop {
        if deadline
            .saturating_duration_since(std::time::Instant::now())
            .is_zero()
        {
            return Err(StoreError::LockTimeout);
        }
        match file.try_lock_exclusive() {
            Ok(()) => {
                return Ok(FileLockGuard {
                    _file: file,
                    path: lock_path.to_path_buf(),
                });
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(20));
                if std::time::Instant::now() >= deadline {
                    return Err(StoreError::LockTimeout);
                }
            }
            Err(e) => return Err(io_to_store(e)),
        }
    }
}

/// Acquire a per-credential lock, returning an RAII guard.
pub async fn acquire_credential_lock(
    lock_path: PathBuf,
    timeout: Duration,
) -> Result<Box<dyn CredentialLockGuard>, StoreError> {
    let guard = acquire_blocking(lock_path, timeout).await?;
    Ok(Box::new(guard))
}

fn io_to_store(e: io::Error) -> StoreError {
    StoreError::Backend(e.to_string())
}
