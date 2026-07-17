//! Session ModelBinding pin policy (A4 / AUD-009 residual).
//!
//! Pure decision function so multi-provider sessions keep the live pin
//! (resolver + attempt stamp ledger) unless the user switches account.

use xai_grok_auth::{CredentialId, ProviderId};

/// Outcome of reconciling a live session pin with hint-derived auth (A4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPinDecision {
    /// Keep the existing pin; do not overwrite session storage.
    KeepPin,
    /// Replace session pin with the hint-derived auth (account switch or first pin).
    AdoptHints,
    /// No multi-provider auth available from pin or hints.
    None,
}

/// Pure pin policy: when a live pin's credential+provider still match hints,
/// the pin wins (resolver + stamp ledger continuity). Conflicting hints only
/// win when credential or provider differs (explicit account switch).
///
/// Hints absent while pin present → keep pin. Pin absent while hints present
/// → adopt hints. Both absent → none.
pub fn session_pin_decision(
    pin_credential: Option<CredentialId>,
    pin_provider: Option<&ProviderId>,
    hint_credential: Option<CredentialId>,
    hint_provider: Option<&ProviderId>,
) -> SessionPinDecision {
    let pin = pin_credential.zip(pin_provider);
    let hint = hint_credential.zip(hint_provider);
    match (pin, hint) {
        (Some((pc, pp)), Some((hc, hp))) if pc == hc && pp == hp => SessionPinDecision::KeepPin,
        (Some(_), Some(_)) => SessionPinDecision::AdoptHints,
        (Some(_), None) => SessionPinDecision::KeepPin,
        (None, Some(_)) => SessionPinDecision::AdoptHints,
        (None, None) => SessionPinDecision::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A4 residual: pin wins when credential+provider match; pin kept when
    /// hints vanish; adopt only on credential/provider switch.
    #[test]
    fn session_pin_wins_over_matching_and_missing_hints() {
        let provider = ProviderId::new_unchecked("codex");
        let other = ProviderId::new_unchecked("other");
        let cred_a = CredentialId::from_uuid(
            uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        );
        let cred_b = CredentialId::from_uuid(
            uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
        );

        assert_eq!(
            session_pin_decision(Some(cred_a), Some(&provider), Some(cred_a), Some(&provider)),
            SessionPinDecision::KeepPin,
            "same credential+provider: pin wins"
        );
        assert_eq!(
            session_pin_decision(Some(cred_a), Some(&provider), None, None),
            SessionPinDecision::KeepPin,
            "hints lost: keep pin"
        );
        assert_eq!(
            session_pin_decision(Some(cred_a), Some(&provider), Some(cred_b), Some(&provider)),
            SessionPinDecision::AdoptHints,
            "credential switch: adopt hints"
        );
        assert_eq!(
            session_pin_decision(Some(cred_a), Some(&provider), Some(cred_a), Some(&other)),
            SessionPinDecision::AdoptHints,
            "provider switch: adopt hints"
        );
        assert_eq!(
            session_pin_decision(None, None, Some(cred_a), Some(&provider)),
            SessionPinDecision::AdoptHints,
            "first pin from hints"
        );
        assert_eq!(
            session_pin_decision(None, None, None, None),
            SessionPinDecision::None
        );
    }
}
