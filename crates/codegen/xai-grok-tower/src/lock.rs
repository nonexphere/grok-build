//! Per-instance exclusive flock for Tower instances (C2-B / TW103-03/06).
//!
//! Implements the dual-OS-process isolation contract: a single-winner
//! `instance.lock` file under each instance's state root, acquired via an
//! `fs2` exclusive flock. A second claimer of the same instance id fails
//! while the lock is held; two different instance ids take disjoint locks
//! concurrently because their state roots — and thus their lock files — are
//! disjoint (see [`crate::instance::instance_state_root`]).
//!
//! This module deliberately does NOT depend on `xai-grok-shell`. It mirrors
//! the flock pattern from `xai-grok-shell/src/leader/lock.rs` at the contract
//! level only: open-or-create the lock file, `try_lock_exclusive`, classify
//! contention via `fs2::lock_contended_error()`, release on drop. The
//! connect-or-spawn state machine, endpoint binding, and credential
//! reconciliation are out of scope for this wave (C1-J residual) — this
//! module owns only the single-winner lock and the per-instance scaffold
//! paths (endpoint / token / metadata) under the instance root.

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use fs2::FileExt;

use crate::instance::{instance_state_root, TowerInstanceId};

/// File name of the exclusive per-instance lock under the instance root.
pub const INSTANCE_LOCK_FILE: &str = "instance.lock";
/// File name of the endpoint marker under the instance root.
pub const INSTANCE_ENDPOINT_FILE: &str = "endpoint";
/// File name of the token file under the instance root.
pub const INSTANCE_TOKEN_FILE: &str = "token";
/// File name of the metadata file under the instance root.
pub const INSTANCE_METADATA_FILE: &str = "metadata.json";

/// True if `e` reports that an advisory `flock` is held by another process.
///
/// Unix surfaces this as `WouldBlock`; Windows as `ERROR_LOCK_VIOLATION`,
/// matched via [`fs2::lock_contended_error`]. Inlined here so the Tower
/// crate does not need to depend on `xai-grok-workspace` (or Shell) for a
/// two-line helper.
fn is_lock_contended(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::WouldBlock
        || e.raw_os_error()
            .is_some_and(|code| Some(code) == fs2::lock_contended_error().raw_os_error())
}

/// Errors from acquiring or holding a [`InstanceLock`].
#[derive(Debug)]
pub enum InstanceLockError {
    /// A filesystem operation failed while opening or locking the file.
    Io(io::Error),
    /// The instance lock for `instance_id` is already held by another
    /// process. The loser of `try_acquire` observes this instead of a
    /// generic IO error so the connect-or-spawn state machine can
    /// distinguish contention from real failures.
    AlreadyHeld { instance_id: String },
}

impl fmt::Display for InstanceLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstanceLockError::Io(e) => write!(f, "instance lock IO error: {e}"),
            InstanceLockError::AlreadyHeld { instance_id } => write!(
                f,
                "instance lock for {instance_id} is held by another process"
            ),
        }
    }
}

impl std::error::Error for InstanceLockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InstanceLockError::Io(e) => Some(e),
            InstanceLockError::AlreadyHeld { .. } => None,
        }
    }
}

impl From<io::Error> for InstanceLockError {
    fn from(e: io::Error) -> Self {
        InstanceLockError::Io(e)
    }
}

/// Exclusive per-instance flock on `<home>/towers/<id>/instance.lock`.
///
/// The lock is acquired with a non-blocking `try_lock_exclusive`; the file
/// is opened read+write+create so the first claimer materializes it. The
/// state root directory is created on acquisition (the lock file must have
/// a parent directory). On drop, the OS lock is released by closing the
/// file handle; the lock file is NOT removed — a stale-PID reconciliation
/// step (C1-J residual) owns cleanup. Holding the [`InstanceLock`] for the
/// full lifetime of the Tower process is the intended usage.
///
/// Distinct [`TowerInstanceId`]s yield disjoint state roots and thus
/// disjoint lock files, which is the foundation of dual-instance
/// isolation: two different ids can hold their locks concurrently, while
/// two claimers of the same id race for a single file.
#[derive(Debug)]
pub struct InstanceLock {
    instance_id: String,
    state_root: PathBuf,
    lock_path: PathBuf,
    lock_file: Option<File>,
}

impl InstanceLock {
    /// Try to acquire the exclusive instance lock for `id` under `home`
    /// without blocking.
    ///
    /// Returns:
    /// - `Ok(InstanceLock)` if the lock was acquired (this process is the
    ///   single winner for the instance).
    /// - `Err(InstanceLockError::AlreadyHeld)` if another process holds the
    ///   lock.
    /// - `Err(InstanceLockError::Io)` for any other filesystem failure.
    ///
    /// The state root (`<home>/towers/<id>/`) is created if missing. The lock
    /// file (`instance.lock` under the root) is created if missing. Neither
    /// is removed on release/drop (stale-PID reconciliation owns cleanup).
    pub fn try_acquire(home: &Path, id: &TowerInstanceId) -> Result<Self, InstanceLockError> {
        let state_root = instance_state_root(home, id);
        fs::create_dir_all(&state_root)?;
        let lock_path = state_root.join(INSTANCE_LOCK_FILE);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)?;
        match file.try_lock_exclusive() {
            Ok(()) => Ok(Self {
                instance_id: id.as_str().to_owned(),
                state_root,
                lock_path,
                lock_file: Some(file),
            }),
            Err(e) if is_lock_contended(&e) => Err(InstanceLockError::AlreadyHeld {
                instance_id: id.as_str().to_owned(),
            }),
            Err(e) => Err(InstanceLockError::Io(e)),
        }
    }

    /// Non-blocking contention probe: true if the instance lock for `id`
    /// under `home` is currently held by some process. Does NOT acquire the
    /// lock. Used by the connect-or-spawn state machine to decide between
    /// connecting to an existing instance and spawning a new one.
    pub fn is_held_for(home: &Path, id: &TowerInstanceId) -> bool {
        let state_root = instance_state_root(home, id);
        let lock_path = state_root.join(INSTANCE_LOCK_FILE);
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
        {
            Ok(f) => f,
            Err(_) => return false,
        };
        match file.try_lock_exclusive() {
            // We acquired it → it was not held. Release immediately and
            // report unheld. The lock file is left in place (cleanup is the
            // stale-PID reconciliation step's responsibility).
            Ok(()) => {
                let _ = file.unlock();
                false
            }
            Err(e) if is_lock_contended(&e) => true,
            Err(_) => false,
        }
    }

    /// True if this handle currently holds the OS lock.
    pub fn is_held(&self) -> bool {
        self.lock_file.is_some()
    }

    /// The instance id this lock was acquired for.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// The canonical state root for this instance (`<home>/towers/<id>/`).
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    /// The lock file path (`<state_root>/instance.lock`).
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }

    /// The endpoint marker path under the instance root. NOT created by the
    /// lock scaffold; the connect-or-spawn state machine writes the bound
    /// endpoint here.
    pub fn endpoint_path(&self) -> PathBuf {
        self.state_root.join(INSTANCE_ENDPOINT_FILE)
    }

    /// The token file path under the instance root. NOT created by the lock
    /// scaffold; the credential handshake writes the instance token here.
    pub fn token_path(&self) -> PathBuf {
        self.state_root.join(INSTANCE_TOKEN_FILE)
    }

    /// The metadata file path under the instance root. NOT created by the
    /// lock scaffold; the lifecycle writer records instance metadata here.
    pub fn metadata_path(&self) -> PathBuf {
        self.state_root.join(INSTANCE_METADATA_FILE)
    }

    /// Write the holding process id to the lock file. Call after
    /// [`InstanceLock::try_acquire`] for diagnostics and stale-PID
    /// reconciliation.
    pub fn write_pid(&mut self) -> io::Result<()> {
        if let Some(ref mut file) = self.lock_file {
            file.set_len(0)?;
            write!(file, "{}", std::process::id())?;
            file.sync_all()?;
        }
        Ok(())
    }

    /// Read the holder PID recorded in the lock file, if any. Returns None
    /// when the file is empty, missing, or non-numeric.
    pub fn read_pid(&self) -> Option<u32> {
        let mut content = String::new();
        File::open(&self.lock_path)
            .and_then(|mut f| f.read_to_string(&mut content))
            .ok()?;
        content.trim().parse().ok()
    }

    /// Release the OS lock explicitly. After this, [`InstanceLock::is_held`]
    /// returns false and drop is a no-op. The lock file is NOT removed.
    pub fn release(&mut self) -> io::Result<()> {
        if let Some(file) = self.lock_file.take() {
            file.unlock()?;
        }
        Ok(())
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        // Closing the file handle releases the OS flock. We do not remove
        // the lock file: stale-PID reconciliation (C1-J residual) owns
        // cleanup, and a racing claimer may need the file to exist.
        if let Some(file) = self.lock_file.take() {
            let _ = file.unlock();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn try_acquire_succeeds_when_unlocked() {
        let home = TempDir::new().unwrap();
        let id: TowerInstanceId = "default".parse().unwrap();
        let lock = InstanceLock::try_acquire(home.path(), &id).unwrap();
        assert!(lock.is_held());
        assert!(lock.lock_path().is_file());
        assert!(lock.lock_path().starts_with(lock.state_root()));
    }

    #[test]
    fn try_acquire_fails_when_held() {
        let home = TempDir::new().unwrap();
        let id: TowerInstanceId = "default".parse().unwrap();
        let _first = InstanceLock::try_acquire(home.path(), &id).unwrap();
        let second = InstanceLock::try_acquire(home.path(), &id);
        assert!(matches!(
            second,
            Err(InstanceLockError::AlreadyHeld { .. })
        ));
    }

    #[test]
    fn drop_releases_lock() {
        let home = TempDir::new().unwrap();
        let id: TowerInstanceId = "default".parse().unwrap();
        {
            let _lock = InstanceLock::try_acquire(home.path(), &id).unwrap();
        }
        // After drop, a new claimer can acquire.
        let lock = InstanceLock::try_acquire(home.path(), &id);
        assert!(lock.is_ok());
    }

    #[test]
    fn two_instances_acquire_concurrently() {
        let home = TempDir::new().unwrap();
        let a: TowerInstanceId = "default".parse().unwrap();
        let b: TowerInstanceId = "worktree-1".parse().unwrap();
        let lock_a = InstanceLock::try_acquire(home.path(), &a).unwrap();
        let lock_b = InstanceLock::try_acquire(home.path(), &b).unwrap();
        assert!(lock_a.is_held() && lock_b.is_held());
        assert_ne!(lock_a.lock_path(), lock_b.lock_path());
    }

    #[test]
    fn is_held_probe_tracks_lock_state() {
        let home = TempDir::new().unwrap();
        let id: TowerInstanceId = "default".parse().unwrap();
        assert!(!InstanceLock::is_held_for(home.path(), &id));
        let _lock = InstanceLock::try_acquire(home.path(), &id).unwrap();
        assert!(InstanceLock::is_held_for(home.path(), &id));
    }

    #[test]
    fn write_and_read_pid_roundtrip() {
        let home = TempDir::new().unwrap();
        let id: TowerInstanceId = "default".parse().unwrap();
        let mut lock = InstanceLock::try_acquire(home.path(), &id).unwrap();
        assert!(lock.read_pid().is_none());
        lock.write_pid().unwrap();
        assert_eq!(lock.read_pid(), Some(std::process::id()));
    }

    #[test]
    fn scaffold_paths_under_state_root() {
        let home = TempDir::new().unwrap();
        let id: TowerInstanceId = "branch-a".parse().unwrap();
        let lock = InstanceLock::try_acquire(home.path(), &id).unwrap();
        assert_eq!(
            lock.endpoint_path().file_name().unwrap().to_str().unwrap(),
            "endpoint"
        );
        assert_eq!(
            lock.token_path().file_name().unwrap().to_str().unwrap(),
            "token"
        );
        assert_eq!(
            lock.metadata_path().file_name().unwrap().to_str().unwrap(),
            "metadata.json"
        );
        for p in [lock.endpoint_path(), lock.token_path(), lock.metadata_path()] {
            assert!(p.starts_with(lock.state_root()));
        }
    }
}
