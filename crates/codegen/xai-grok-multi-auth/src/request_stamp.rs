//! Attempt-bound credential stamps (A1 / AUD-006).
//!
//! Each successful bearer resolve records a [`SentCredentialStamp`] under a
//! monotonic attempt id. Recovery after 401 must use the stamp from the
//! **attempt that failed**, not whatever `last()` returns after later resolves
//! on the same session resolver.
//!
//! Provider-managed prompt cache is unrelated — this is only OAuth generation
//! identity for multi-provider 401 recovery.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use xai_grok_auth::SentCredentialStamp;

/// Holds stamps keyed by attempt id, plus a FIFO of attempts awaiting recovery.
#[derive(Debug, Default)]
pub struct AttemptStampLedger {
    next_id: AtomicU64,
    by_id: Mutex<HashMap<u64, SentCredentialStamp>>,
    /// Order of resolves that have not yet been consumed by recovery.
    pending_recovery: Mutex<VecDeque<u64>>,
}

impl AttemptStampLedger {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            by_id: Mutex::new(HashMap::new()),
            pending_recovery: Mutex::new(VecDeque::new()),
        }
    }

    /// Record a stamp for a new resolve attempt. Returns the attempt id.
    pub fn record(&self, stamp: SentCredentialStamp) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut map) = self.by_id.lock() {
            map.insert(id, stamp);
        }
        if let Ok(mut q) = self.pending_recovery.lock() {
            q.push_back(id);
        }
        id
    }

    /// Stamp for a specific attempt (does not consume recovery FIFO).
    pub fn get(&self, attempt_id: u64) -> Option<SentCredentialStamp> {
        self.by_id.lock().ok()?.get(&attempt_id).cloned()
    }

    /// Most recent stamp (last-wins peek — **insufficient for recovery alone**).
    pub fn last(&self) -> Option<SentCredentialStamp> {
        let map = self.by_id.lock().ok()?;
        let id = self.next_id.load(Ordering::SeqCst).saturating_sub(1);
        if id == 0 {
            return None;
        }
        map.get(&id).cloned()
    }

    /// Pop the oldest unrecovered attempt stamp (FIFO matches sequential request order).
    pub fn take_for_recovery(&self) -> Option<SentCredentialStamp> {
        let id = self.pending_recovery.lock().ok()?.pop_front()?;
        self.by_id.lock().ok()?.remove(&id)
    }

    /// Recover using an explicit attempt stamp (does not require FIFO order).
    pub fn take_attempt(&self, attempt_id: u64) -> Option<SentCredentialStamp> {
        if let Ok(mut q) = self.pending_recovery.lock() {
            q.retain(|&id| id != attempt_id);
        }
        self.by_id.lock().ok()?.remove(&attempt_id)
    }

    /// Number of stamps still pending recovery (tests).
    pub fn pending_recovery_len(&self) -> usize {
        self.pending_recovery.lock().map(|q| q.len()).unwrap_or(0)
    }
}

// Backward-compatible thin alias used by earlier session code.
#[derive(Debug, Default)]
pub struct RequestScopedStamp {
    ledger: AttemptStampLedger,
}

impl RequestScopedStamp {
    pub fn new() -> Self {
        Self {
            ledger: AttemptStampLedger::new(),
        }
    }

    pub fn record(&self, stamp: SentCredentialStamp) -> u64 {
        self.ledger.record(stamp)
    }

    pub fn last(&self) -> Option<SentCredentialStamp> {
        self.ledger.last()
    }

    pub fn take(&self) -> Option<SentCredentialStamp> {
        self.ledger.take_for_recovery()
    }

    pub fn take_for_recovery(&self) -> Option<SentCredentialStamp> {
        self.ledger.take_for_recovery()
    }

    pub fn get(&self, attempt_id: u64) -> Option<SentCredentialStamp> {
        self.ledger.get(attempt_id)
    }

    pub fn ledger(&self) -> &AttemptStampLedger {
        &self.ledger
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_auth::{AccountFingerprint, CredentialId, CredentialKey, ProviderId};

    fn stamp(cred: &str, generation: u64) -> SentCredentialStamp {
        let uuid = uuid::Uuid::parse_str(cred).unwrap();
        SentCredentialStamp {
            key: CredentialKey {
                provider: ProviderId::new_unchecked("codex"),
                credential_id: CredentialId::from_uuid(uuid),
            },
            generation,
            account_fingerprint: AccountFingerprint::from([generation as u8; 32]),
        }
    }

    #[test]
    fn concurrent_holders_keep_distinct_stamps() {
        let a = RequestScopedStamp::new();
        let b = RequestScopedStamp::new();
        a.record(stamp("11111111-1111-1111-1111-111111111111", 1));
        b.record(stamp("22222222-2222-2222-2222-222222222222", 99));
        assert_eq!(a.last().unwrap().generation, 1);
        assert_eq!(b.last().unwrap().generation, 99);
    }

    #[test]
    fn same_ledger_sequential_resolves_recovery_uses_attempt_order() {
        // AUD-006: two resolves on the SAME holder; recovery of the first
        // 401 must use stamp gen=1, not last() which is gen=2.
        let ledger = AttemptStampLedger::new();
        let id1 = ledger.record(stamp("11111111-1111-1111-1111-111111111111", 1));
        let id2 = ledger.record(stamp("11111111-1111-1111-1111-111111111111", 2));
        assert_ne!(id1, id2);
        assert_eq!(ledger.last().unwrap().generation, 2);

        let recovered_first = ledger.take_for_recovery().expect("first");
        assert_eq!(
            recovered_first.generation, 1,
            "FIFO recovery must return attempt 1 stamp, not last()"
        );
        let recovered_second = ledger.take_for_recovery().expect("second");
        assert_eq!(recovered_second.generation, 2);
    }

    #[test]
    fn explicit_attempt_id_survives_later_resolves() {
        let ledger = AttemptStampLedger::new();
        let id1 = ledger.record(stamp("11111111-1111-1111-1111-111111111111", 10));
        let _id2 = ledger.record(stamp("11111111-1111-1111-1111-111111111111", 20));
        let s1 = ledger.get(id1).expect("id1 still stored");
        assert_eq!(s1.generation, 10);
        let taken = ledger.take_attempt(id1).expect("take id1");
        assert_eq!(taken.generation, 10);
        // Remaining FIFO head should be attempt 2
        assert_eq!(ledger.take_for_recovery().unwrap().generation, 20);
    }

    #[test]
    fn take_clears_only_one_holder() {
        let a = RequestScopedStamp::new();
        let b = RequestScopedStamp::new();
        a.record(stamp("11111111-1111-1111-1111-111111111111", 3));
        b.record(stamp("22222222-2222-2222-2222-222222222222", 4));
        assert_eq!(a.take().unwrap().generation, 3);
        assert_eq!(a.pending_recovery_len_for_test(), 0);
        assert_eq!(b.last().unwrap().generation, 4);
    }

    /// Models SamplingClient order: peek (auth_info/prefix) then current_bearer
    /// (post). Only the latter must enqueue recovery stamps.
    #[test]
    fn peek_then_send_order_recovery_matches_send_attempt() {
        let ledger = AttemptStampLedger::new();
        // peeks must not record
        // send attempt 1
        ledger.record(stamp("11111111-1111-1111-1111-111111111111", 1));
        // peeks again (attribution)
        // send attempt 2 (retry)
        ledger.record(stamp("11111111-1111-1111-1111-111111111111", 2));
        assert_eq!(ledger.pending_recovery_len(), 2);
        assert_eq!(ledger.last().unwrap().generation, 2);
        assert_eq!(
            ledger.take_for_recovery().unwrap().generation,
            1,
            "first 401 recovery must use send attempt 1, not last peek/last()"
        );
        assert_eq!(ledger.take_for_recovery().unwrap().generation, 2);
    }

    impl RequestScopedStamp {
        fn pending_recovery_len_for_test(&self) -> usize {
            self.ledger.pending_recovery_len()
        }
    }
}
