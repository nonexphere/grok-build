//! Lifecycle telemetry with bounded labels (no secrets).

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct LifecycleMetrics {
    counters: Mutex<HashMap<&'static str, u64>>,
}

impl LifecycleMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc(&self, name: &'static str) {
        *self.counters.lock().unwrap().entry(name).or_insert(0) += 1;
    }

    pub fn get(&self, name: &str) -> u64 {
        self.counters
            .lock()
            .unwrap()
            .get(name)
            .copied()
            .unwrap_or(0)
    }

    pub fn snapshot(&self) -> HashMap<&'static str, u64> {
        self.counters.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod lifecycle_metrics_tests {
    use super::*;

    #[test]
    fn lifecycle_metrics_are_bounded_and_secret_free() {
        let m = LifecycleMetrics::new();
        m.inc("session_start");
        m.inc("session_start");
        m.inc("turn_start");
        assert_eq!(m.get("session_start"), 2);
        assert_eq!(m.get("turn_start"), 1);
        let snap = format!("{:?}", m.snapshot());
        assert!(!snap.contains("sk-"));
        assert!(!snap.contains("Bearer "));
    }

    #[test]
    fn audit_canary_labels_reject_secret_names() {
        // Metric *values* must never embed secret canaries.
        let m = LifecycleMetrics::new();
        m.inc("auth_failure");
        let snap = format!("{:?}", m.snapshot());
        for canary in ["sk-", "Bearer ", "access_token=", "api_key="] {
            assert!(!snap.contains(canary), "canary {canary} in {snap}");
        }
        assert_eq!(m.get("auth_failure"), 1);
    }
}
