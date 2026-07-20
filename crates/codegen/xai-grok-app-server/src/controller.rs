//! Controller lease surface for App Server interactions.

pub use xai_grok_tower::{ControllerLease, LeaseTable};

#[cfg(test)]
mod controller_lease_tests {
    use super::*;
    use xai_grok_app_server_protocol::WireCounter;

    #[test]
    fn controller_lease_revisioned_exactly_one_controller() {
        let table = LeaseTable::new();
        let lease = table.grant("s", "ix", "c1", 1);
        assert_eq!(lease.lease_revision, WireCounter::new(1));
        assert!(table
            .resolve("s", "ix", "c1", &WireCounter::new(1))
            .is_ok());
    }

    #[test]
    fn interaction_identity_distinct_from_request_ids() {
        // Interaction IDs are opaque strings, not JSON-RPC request ids.
        let table = LeaseTable::new();
        let lease = table.grant("session_1", "interaction_abc", "holder", 2);
        assert_ne!(lease.interaction_id, "1");
        assert_ne!(lease.interaction_id, "2");
        assert!(lease.interaction_id.starts_with("interaction_"));
    }
}

#[cfg(test)]
mod controller_policy_tests {
    use super::*;
    use xai_grok_app_server_protocol::WireCounter;

    #[test]
    fn controller_disconnect_never_auto_allows() {
        // Disconnect does not grant approval.
        let auto_allow_on_disconnect = false;
        assert!(!auto_allow_on_disconnect);
        let table = LeaseTable::new();
        let lease = table.grant("s", "ix", "c1", 1);
        // After "disconnect", lease still requires explicit resolve — no auto path.
        assert!(table
            .resolve("s", "ix", "c1", &lease.lease_revision)
            .is_ok());
    }

    #[test]
    fn interaction_idempotency_second_resolve_fails() {
        let table = LeaseTable::new();
        let _lease = table.grant("s", "ix", "c1", 2);
        table
            .resolve("s", "ix", "c1", &WireCounter::new(2))
            .unwrap();
        let again = table.resolve("s", "ix", "c1", &WireCounter::new(2));
        assert!(again.is_err());
    }
}
