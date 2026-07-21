//! C2-A: Tower instance isolation (dual-instance).
//!
//! Proves that two distinct [`TowerInstanceId`]s produce disjoint state roots
//! and disjoint resident registries. This is the in-process, lock-free slice
//! of the multi-instance isolation contract: the dual-OS-process flock half
//! (single-winner `instance.lock`, endpoint-in-use detection) is PARTIAL — it
//! needs the connect-or-spawn state machine + credentials and is out of scope
//! for this wave (see `waves/c2-tower-instance.md`).
//!
//! These tests deliberately do NOT touch the real `~/.grok-oss`; they use
//! `TempDir` homes and pure in-memory registries.

use std::path::PathBuf;

use tempfile::TempDir;
use xai_grok_tower::{
    InstanceDirectory, InstanceLock, InstanceLockError, SessionRegistry, TowerHandle,
    TowerInstanceId, instance_state_root,
};

/// Two instances get disjoint state-root directories under the same home,
/// matching the lifecycle contract `~/.grok-oss/towers/<instance-id>/`.
#[test]
fn two_instances_have_disjoint_state_roots() {
    let home = TempDir::new().expect("home tempdir");
    let home_path = home.path().to_path_buf();
    let a: TowerInstanceId = "default".parse().unwrap();
    let b: TowerInstanceId = "worktree-1".parse().unwrap();

    let root_a = instance_state_root(&home_path, &a);
    let root_b = instance_state_root(&home_path, &b);

    assert_ne!(root_a, root_b, "instance state roots must differ");
    assert!(root_a.ends_with("towers/default"));
    assert!(root_b.ends_with("towers/worktree-1"));

    // Materialize both roots and prove they are distinct directories. The
    // paths are derived from the same TempDir home with distinct final
    // segments, so equality of the PathBufs is a sufficient disjointness
    // proof (no need to canonicalize — and `std::fs::canonicalize` is a
    // workspace-disallowed method on Windows-verbatim grounds).
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    assert!(root_a.is_dir());
    assert!(root_b.is_dir());
    assert_ne!(root_a, root_b);
    // Neither root is a prefix of the other (no lexical containment leak).
    assert!(!root_a.starts_with(&root_b));
    assert!(!root_b.starts_with(&root_a));
}

/// A session id may exist in two instances' registries with independent actor
/// tokens; removing from one does not affect the other. This mirrors the
/// `two_instances_have_disjoint_registries` lib test but at the integration
/// boundary, paired with the directory-isolation claim.
#[test]
fn two_instances_have_disjoint_registries_and_directories() {
    let home = TempDir::new().expect("home tempdir");
    let home_path = home.path().to_path_buf();
    let a: TowerInstanceId = "default".parse().unwrap();
    let b: TowerInstanceId = "branch-a".parse().unwrap();

    // Disjoint directories.
    let root_a = instance_state_root(&home_path, &a);
    let root_b = instance_state_root(&home_path, &b);
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    assert_ne!(root_a, root_b);
    assert!(!root_a.starts_with(&root_b));
    assert!(!root_b.starts_with(&root_a));

    // Disjoint registries: same session id string in both, no shared token.
    let mut reg_a = SessionRegistry::new();
    let mut reg_b = SessionRegistry::new();
    let (ta, created_a) = reg_a.get_or_insert_with("s1", |_| Ok(())).unwrap();
    let (tb, created_b) = reg_b.get_or_insert_with("s1", |_| Ok(())).unwrap();
    assert!(created_a && created_b);
    assert_eq!(ta.as_u64(), 1);
    assert_eq!(tb.as_u64(), 1);
    reg_a.remove("s1");
    assert!(reg_a.get("s1").is_none());
    assert!(
        reg_b.get("s1").is_some(),
        "instance B registry is independent"
    );

    // InstanceDirectory refuses a duplicate id (contention guard).
    let mut dir = InstanceDirectory::default();
    dir.insert(TowerHandle::scaffold(a.clone())).unwrap();
    dir.insert(TowerHandle::scaffold(b.clone())).unwrap();
    assert!(dir.insert(TowerHandle::scaffold(a.clone())).is_err());
    assert_eq!(dir.len(), 2);
}

/// The default instance id and a named instance id are distinct and produce
/// distinct roots; `default` is not a special alias for any other id.
#[test]
fn default_instance_is_distinct_from_named_instances() {
    let home = PathBuf::from("/home/u/.grok-oss");
    let default: TowerInstanceId = "default".parse().unwrap();
    let named: TowerInstanceId = "ci-1".parse().unwrap();
    assert_ne!(default, named);
    assert_ne!(
        instance_state_root(&home, &default),
        instance_state_root(&home, &named)
    );
}

/// Invalid instance ids cannot be turned into state roots via the public API:
/// parsing fails before any path is derived. Fail-closed on bad config.
#[test]
fn invalid_instance_id_is_rejected_before_path_derivation() {
    for invalid in ["", "UPPER", "-leading", "contains space", "é", "with/slash"] {
        assert!(
            invalid.parse::<TowerInstanceId>().is_err(),
            "expected {invalid:?} to be rejected"
        );
    }
    // The default id is always valid, so a path is always derivable for the
    // fallback case — but an invalid explicit/env value surfaces an error at
    // the composition root (see pager-bin precedence tests).
    let home = PathBuf::from("/home/u/.grok-oss");
    let id: TowerInstanceId = "default".parse().unwrap();
    assert!(instance_state_root(&home, &id).ends_with("towers/default"));
}

/// C2-B: Dual-OS-process flock isolation.
///
/// Proves the single-winner `instance.lock` half of the multi-instance
/// isolation contract: a second claimer of the same instance id fails while
/// the lock is held, and two different instance ids take disjoint locks
/// concurrently because their state roots (and thus their lock files) are
/// disjoint. The lock is a real `fs2` exclusive flock on
/// `<home>/towers/<id>/instance.lock`.
///
/// These tests do NOT touch the real `~/.grok-oss`; they use `TempDir` homes.
mod flock_isolation_tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    /// Two different instance ids take disjoint locks concurrently: both
    /// `try_acquire` succeed at the same time because their lock files live
    /// under disjoint state roots.
    #[test]
    fn two_instances_take_disjoint_flock_concurrently() {
        let home = TempDir::new().expect("home tempdir");
        let home_path = home.path().to_path_buf();
        let a: TowerInstanceId = "default".parse().unwrap();
        let b: TowerInstanceId = "worktree-1".parse().unwrap();

        let barrier = Arc::new(Barrier::new(2));
        let home_a = home_path.clone();
        let home_b = home_path.clone();
        let barrier_a = barrier.clone();
        let barrier_b = barrier.clone();

        let h_a = thread::spawn(move || {
            let lock = InstanceLock::try_acquire(&home_a, &a).expect("acquire A");
            barrier_a.wait();
            lock
        });
        let h_b = thread::spawn(move || {
            let lock = InstanceLock::try_acquire(&home_b, &b).expect("acquire B");
            barrier_b.wait();
            lock
        });

        let lock_a = h_a.join().expect("thread A");
        let lock_b = h_b.join().expect("thread B");

        assert!(lock_a.is_held());
        assert!(lock_b.is_held());
        // Disjoint lock files (disjoint state roots).
        assert_ne!(lock_a.lock_path(), lock_b.lock_path());
        assert_ne!(lock_a.state_root(), lock_b.state_root());
        // Neither lock path is a prefix of the other.
        assert!(!lock_a.lock_path().starts_with(lock_b.lock_path()));
        assert!(!lock_b.lock_path().starts_with(lock_a.lock_path()));
    }

    /// A second claimer of the same instance id fails while the first holds
    /// the lock. The loser observes `InstanceLockError::AlreadyHeld`, not a
    /// generic IO error.
    #[test]
    fn instance_contention_second_claimer_fails_while_held() {
        let home = TempDir::new().expect("home tempdir");
        let home_path = home.path().to_path_buf();
        let id: TowerInstanceId = "default".parse().unwrap();

        let winner = InstanceLock::try_acquire(&home_path, &id).expect("first acquire");
        assert!(winner.is_held());

        // A second claimer of the same id (same lock file) must fail.
        let loser = InstanceLock::try_acquire(&home_path, &id);
        assert!(
            matches!(loser, Err(InstanceLockError::AlreadyHeld { .. })),
            "expected AlreadyHeld, got {:?}",
            loser
        );

        // The lock file exists on disk (it was created by the winner).
        assert!(winner.lock_path().is_file());

        // Releasing the winner lets a new claimer acquire.
        drop(winner);
        let reacquired = InstanceLock::try_acquire(&home_path, &id);
        assert!(
            reacquired.is_ok(),
            "reacquire after release: {reacquired:?}"
        );
        assert!(reacquired.unwrap().is_held());
    }

    /// Many contenders racing for the SAME instance id produce exactly one
    /// winner; every loser observes `AlreadyHeld`. This is the core
    /// connect-or-spawn single-spawn invariant for Tower instances.
    #[test]
    fn instance_contention_single_winner_among_many() {
        let home = TempDir::new().expect("home tempdir");
        let home_path = home.path().to_path_buf();
        let id: TowerInstanceId = "default".parse().unwrap();

        let n = 8;
        let barrier = Arc::new(Barrier::new(n));
        let winners = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..n {
            let home_path = home_path.clone();
            let barrier = barrier.clone();
            let winners = winners.clone();
            // TowerInstanceId is Clone.
            let id = id.clone();
            handles.push(thread::spawn(move || {
                barrier.wait();
                match InstanceLock::try_acquire(&home_path, &id) {
                    Ok(lock) => {
                        winners.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        // Hold briefly so losers observe contention.
                        thread::sleep(std::time::Duration::from_millis(50));
                        drop(lock);
                        true
                    }
                    Err(InstanceLockError::AlreadyHeld { .. }) => false,
                    Err(other) => panic!("unexpected error: {other:?}"),
                }
            }));
        }
        let outcomes: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(
            winners.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "exactly one contender must win the instance lock"
        );
        assert_eq!(outcomes.iter().filter(|won| **won).count(), 1);
        assert_eq!(outcomes.iter().filter(|won| !**won).count(), n - 1);
    }

    /// The instance lock materializes the instance state root and exposes
    /// endpoint/token/metadata file paths under that root (minimal scaffold).
    /// The lock file itself lives at `<root>/instance.lock`.
    #[test]
    fn instance_lock_scaffold_files_under_root() {
        let home = TempDir::new().expect("home tempdir");
        let home_path = home.path().to_path_buf();
        let id: TowerInstanceId = "branch-a".parse().unwrap();

        let lock = InstanceLock::try_acquire(&home_path, &id).expect("acquire");

        // The state root was materialized by try_acquire.
        assert!(lock.state_root().is_dir());
        // The lock file lives under the state root.
        assert!(lock.lock_path().is_file());
        assert!(lock.lock_path().starts_with(lock.state_root()));
        assert!(lock.lock_path().file_name().unwrap().to_str().unwrap() == "instance.lock");

        // Endpoint/token/metadata paths are under the state root and have the
        // canonical file names. They are NOT created by the lock scaffold
        // (the connect-or-spawn state machine owns their content), but the
        // paths are derivable and disjoint per instance.
        let endpoint = lock.endpoint_path();
        let token = lock.token_path();
        let metadata = lock.metadata_path();
        for p in [&endpoint, &token, &metadata] {
            assert!(p.starts_with(lock.state_root()));
        }
        assert_eq!(endpoint.file_name().unwrap().to_str().unwrap(), "endpoint");
        assert_eq!(token.file_name().unwrap().to_str().unwrap(), "token");
        assert_eq!(
            metadata.file_name().unwrap().to_str().unwrap(),
            "metadata.json"
        );
        // Disjoint from a second instance's scaffold paths.
        let id2: TowerInstanceId = "branch-b".parse().unwrap();
        let lock2 = InstanceLock::try_acquire(&home_path, &id2).expect("acquire 2");
        assert_ne!(lock.endpoint_path(), lock2.endpoint_path());
        assert_ne!(lock.token_path(), lock2.token_path());
        assert_ne!(lock.metadata_path(), lock2.metadata_path());
    }

    /// `write_pid` records the holding process id; `read_pid` recovers it.
    /// A lock file with no PID written returns None.
    #[test]
    fn instance_lock_records_holder_pid() {
        let home = TempDir::new().expect("home tempdir");
        let home_path = home.path().to_path_buf();
        let id: TowerInstanceId = "default".parse().unwrap();

        let mut lock = InstanceLock::try_acquire(&home_path, &id).expect("acquire");
        // Before write_pid, read_pid returns None (no PID recorded yet).
        assert!(lock.read_pid().is_none());
        lock.write_pid().expect("write pid");
        assert_eq!(lock.read_pid(), Some(std::process::id()));
    }

    /// A non-blocking contention probe reports held/unheld without acquiring.
    #[test]
    fn instance_lock_probe_reports_held_state() {
        let home = TempDir::new().expect("home tempdir");
        let home_path = home.path().to_path_buf();
        let id: TowerInstanceId = "default".parse().unwrap();

        assert!(!InstanceLock::is_held_for(&home_path, &id));
        let _lock = InstanceLock::try_acquire(&home_path, &id).expect("acquire");
        assert!(InstanceLock::is_held_for(&home_path, &id));
        // After release, probe reports unheld again.
        drop(_lock);
        assert!(!InstanceLock::is_held_for(&home_path, &id));
    }
}
