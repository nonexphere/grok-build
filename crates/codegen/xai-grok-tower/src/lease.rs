//! Controller lease for Interaction responses (deterministic first-writer wins).

use std::collections::HashMap;
use std::sync::Mutex;

use xai_grok_app_server_protocol::WireCounter;

use crate::RuntimeError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerLease {
    pub session_id: String,
    pub interaction_id: String,
    pub lease_revision: WireCounter,
    pub holder_id: String,
}

#[derive(Debug, Default)]
pub struct LeaseTable {
    inner: Mutex<HashMap<String, ControllerLease>>,
}

impl LeaseTable {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(session_id: &str, interaction_id: &str) -> String {
        format!("{session_id}::{interaction_id}")
    }

    pub fn grant(
        &self,
        session_id: &str,
        interaction_id: &str,
        holder_id: &str,
        revision: u64,
    ) -> ControllerLease {
        let lease = ControllerLease {
            session_id: session_id.into(),
            interaction_id: interaction_id.into(),
            lease_revision: WireCounter::new(revision),
            holder_id: holder_id.into(),
        };
        self.inner
            .lock()
            .unwrap()
            .insert(Self::key(session_id, interaction_id), lease.clone());
        lease
    }

    /// First valid response wins; stale/wrong holder is a no-op error.
    pub fn resolve(
        &self,
        session_id: &str,
        interaction_id: &str,
        holder_id: &str,
        lease_revision: &WireCounter,
    ) -> Result<ControllerLease, RuntimeError> {
        let mut guard = self.inner.lock().unwrap();
        let key = Self::key(session_id, interaction_id);
        let Some(lease) = guard.get(&key).cloned() else {
            return Err(RuntimeError {
                code: "interaction_not_found",
                message: "Interaction was not found.".into(),
            });
        };
        if lease.holder_id != holder_id || &lease.lease_revision != lease_revision {
            return Err(RuntimeError {
                code: "controller_lease_required",
                message: "A controller lease is required.".into(),
            });
        }
        guard.remove(&key);
        Ok(lease)
    }
}

#[cfg(test)]
mod lease_tests {
    use super::*;

    #[test]
    fn interaction_lease_first_writer_wins_stale_rejected() {
        let table = LeaseTable::new();
        let lease = table.grant("s1", "ix1", "client-a", 3);
        let ok = table
            .resolve("s1", "ix1", "client-a", &lease.lease_revision)
            .unwrap();
        assert_eq!(ok.holder_id, "client-a");
        let stale = table.resolve("s1", "ix1", "client-a", &lease.lease_revision);
        assert_eq!(stale.unwrap_err().code, "interaction_not_found");
        let lease2 = table.grant("s1", "ix2", "client-a", 1);
        let wrong = table.resolve("s1", "ix2", "client-b", &lease2.lease_revision);
        assert_eq!(wrong.unwrap_err().code, "controller_lease_required");
    }
}
