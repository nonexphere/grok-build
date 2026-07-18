//! Drain and restart lifecycle for Tower instances.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::RuntimeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainState {
    Running,
    Draining,
    Stopped,
}

#[derive(Debug)]
pub struct DrainController {
    draining: AtomicBool,
    stopped: AtomicBool,
    epoch: AtomicU64,
}

impl Default for DrainController {
    fn default() -> Self {
        Self::new()
    }
}

impl DrainController {
    pub fn new() -> Self {
        Self {
            draining: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            epoch: AtomicU64::new(1),
        }
    }

    pub fn state(&self) -> DrainState {
        if self.stopped.load(Ordering::SeqCst) {
            DrainState::Stopped
        } else if self.draining.load(Ordering::SeqCst) {
            DrainState::Draining
        } else {
            DrainState::Running
        }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::SeqCst)
    }

    pub fn begin_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn complete_stop(&self) {
        self.draining.store(false, Ordering::SeqCst);
        self.stopped.store(true, Ordering::SeqCst);
    }

    /// Simulate crash/restart: bump epoch and return to running.
    pub fn restart(&self) -> u64 {
        self.stopped.store(false, Ordering::SeqCst);
        self.draining.store(false, Ordering::SeqCst);
        self.epoch.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn admit_new_work(&self) -> Result<(), RuntimeError> {
        match self.state() {
            DrainState::Running => Ok(()),
            DrainState::Draining => Err(RuntimeError {
                code: "tower_draining",
                message: "Tower is draining.".into(),
            }),
            DrainState::Stopped => Err(RuntimeError {
                code: "runtime_unavailable",
                message: "Runtime is unavailable.".into(),
            }),
        }
    }
}

#[cfg(test)]
mod drain_tests {
    use super::*;

    #[test]
    fn drain_rejects_new_work_and_restart_bumps_epoch() {
        let d = DrainController::new();
        assert_eq!(d.state(), DrainState::Running);
        d.admit_new_work().unwrap();
        d.begin_drain();
        assert_eq!(d.state(), DrainState::Draining);
        assert_eq!(d.admit_new_work().unwrap_err().code, "tower_draining");
        d.complete_stop();
        assert_eq!(d.state(), DrainState::Stopped);
        let e1 = d.epoch();
        let e2 = d.restart();
        assert!(e2 > e1);
        assert_eq!(d.state(), DrainState::Running);
        d.admit_new_work().unwrap();
    }

    #[test]
    fn restart_epoch_requires_explicit_resync_signal() {
        let d = DrainController::new();
        let before = d.epoch();
        d.begin_drain();
        d.complete_stop();
        let after = d.restart();
        assert_ne!(before, after);
        // Clients must treat epoch change as resync-required (documented contract).
        assert!(after > before);
    }
}
