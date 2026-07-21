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
/// Hints absent while pin present → keep pin (same-model rebuild lost headers).
/// Pin absent while hints present → adopt hints. Both absent → none.
///
/// For mid-session **model** switches (Codex → xAI), callers must use
/// [`session_pin_decision_for_turn`] so sticky KeepPin does not attach a Codex
/// bearer to a non-multi-provider sampling config.
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

/// Turn-level pin policy: like [`session_pin_decision`], but when the sampling
/// target is **not** multi-provider (e.g. user switched session model from
/// `gpt-5.6-luna` / Codex to `grok-4.5`), clear a sticky pin instead of KeepPin.
///
/// Without this, Codex OAuth would be sent to `cli-chat-proxy.grok.com` and 401
/// with multi-provider recovery thrash.
pub fn session_pin_decision_for_turn(
    pin_credential: Option<CredentialId>,
    pin_provider: Option<&ProviderId>,
    hint_credential: Option<CredentialId>,
    hint_provider: Option<&ProviderId>,
    sampling_target_is_multi_provider: bool,
) -> SessionPinDecision {
    let base = session_pin_decision(pin_credential, pin_provider, hint_credential, hint_provider);
    if !sampling_target_is_multi_provider
        && matches!(base, SessionPinDecision::KeepPin)
        && hint_credential.is_none()
    {
        return SessionPinDecision::None;
    }
    base
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

    #[test]
    fn turn_policy_clears_pin_when_sampling_leaves_multi_provider() {
        let provider = ProviderId::new_unchecked("codex");
        let cred = CredentialId::from_uuid(
            uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        );
        // Same-model rebuild (still multi-provider): keep pin when hints vanish.
        assert_eq!(
            session_pin_decision_for_turn(Some(cred), Some(&provider), None, None, true,),
            SessionPinDecision::KeepPin,
        );
        // Mid-session switch to grok-4.5 (not multi-provider): clear pin.
        assert_eq!(
            session_pin_decision_for_turn(Some(cred), Some(&provider), None, None, false,),
            SessionPinDecision::None,
            "Codex → xAI model switch must not sticky-keep Codex bearer"
        );
        // Explicit account switch still adopts even if target is multi-provider.
        let cred_b = CredentialId::from_uuid(
            uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
        );
        assert_eq!(
            session_pin_decision_for_turn(
                Some(cred),
                Some(&provider),
                Some(cred_b),
                Some(&provider),
                true,
            ),
            SessionPinDecision::AdoptHints,
        );
        // Matching pin+hints while multi-provider: keep.
        assert_eq!(
            session_pin_decision_for_turn(
                Some(cred),
                Some(&provider),
                Some(cred),
                Some(&provider),
                true,
            ),
            SessionPinDecision::KeepPin,
        );
    }
}
