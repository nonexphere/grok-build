//! Credential-scoped catalog keys for multi-provider models.
//!
//! Format: `{provider}/{credential_id}/{provider_model_slug}`
//!
//! Runtime identity always includes the credential. Display aliases are UI-only
//! and never appear in the key (see review B2 / add-provider skill §H).

use uuid::Uuid;
use xai_grok_auth::{CredentialId, ModelBinding, ProviderId};

/// Parsed multi-provider catalog identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderModelKey {
    pub provider: ProviderId,
    pub credential_id: CredentialId,
    /// Upstream model slug (e.g. `gpt-5.4`), not the catalog key.
    pub model: String,
}

/// Build a credential-scoped catalog key.
pub fn format_provider_model_key(
    provider: &ProviderId,
    credential_id: CredentialId,
    model: &str,
) -> String {
    format!("{provider}/{credential_id}/{model}")
}

/// Parse `{provider}/{credential_uuid}/{model...}` catalog keys.
///
/// Returns `None` for legacy xAI / custom keys that are not multi-provider.
pub fn parse_provider_model_key(key: &str) -> Option<ProviderModelKey> {
    let mut parts = key.splitn(3, '/');
    let provider_raw = parts.next()?;
    let cred_raw = parts.next()?;
    let model = parts.next()?;
    if provider_raw.is_empty() || cred_raw.is_empty() || model.is_empty() {
        return None;
    }
    // Multi-provider keys always use a UUID credential segment.
    let uuid = Uuid::parse_str(cred_raw).ok()?;
    let provider = ProviderId::new(provider_raw).ok()?;
    Some(ProviderModelKey {
        provider,
        credential_id: CredentialId::from_uuid(uuid),
        model: model.to_string(),
    })
}

impl ProviderModelKey {
    /// Immutable request binding for this catalog entry.
    pub fn to_model_binding(&self) -> ModelBinding {
        ModelBinding::new(
            self.provider.clone(),
            Some(self.credential_id),
            self.model.clone(),
        )
    }
}

/// Resolve a wire model slug among catalog `(key, wire_slug)` pairs.
///
/// - Exact multi-provider / non-mp handling for short slugs:
///   - 0 matches → `None`
///   - 1 match → that catalog key
///   - ≥2 multi-provider keys (`provider/uuid/slug`) with the same wire slug →
///     `None` (ambiguous; caller must require full catalog key)
///   - multiple non-mp matches → last wins (legacy override semantics)
pub fn resolve_wire_slug_to_catalog_key<K, V>(entries: &[(K, V)], slug: &str) -> Option<String>
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    let matches: Vec<(&str, &str)> = entries
        .iter()
        .filter(|(_, wire)| wire.as_ref() == slug)
        .map(|(k, w)| (k.as_ref(), w.as_ref()))
        .collect();
    match matches.len() {
        0 => None,
        1 => Some(matches[0].0.to_string()),
        _ => {
            let mp: Vec<&str> = matches
                .iter()
                .filter(|(k, _)| parse_provider_model_key(k).is_some())
                .map(|(k, _)| *k)
                .collect();
            if mp.len() > 1 {
                return None;
            }
            if mp.len() == 1 {
                return Some(mp[0].to_string());
            }
            matches.last().map(|(k, _)| (*k).to_string())
        }
    }
}

/// Human-readable error when a short slug is multi-provider-ambiguous.
pub fn ambiguous_multi_provider_slug_message<K, V>(
    entries: &[(K, V)],
    slug: &str,
) -> Option<String>
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mp_hits: Vec<String> = entries
        .iter()
        .filter(|(k, w)| {
            w.as_ref() == slug && parse_provider_model_key(k.as_ref()).is_some()
        })
        .map(|(k, _)| k.as_ref().to_string())
        .collect();
    if mp_hits.len() > 1 {
        Some(format!(
            "ambiguous model '{slug}': multiple Codex accounts expose it. \
             Use a full catalog id, e.g. one of: {}",
            mp_hits.join(", ")
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_codex_key() {
        let provider = ProviderId::new_unchecked("codex");
        let cred = CredentialId::from_uuid(
            Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
        );
        let key = format_provider_model_key(&provider, cred, "gpt-5.4");
        assert_eq!(key, "codex/01234567-89ab-cdef-0123-456789abcdef/gpt-5.4");
        let parsed = parse_provider_model_key(&key).expect("parse");
        assert_eq!(parsed.provider.as_str(), "codex");
        assert_eq!(parsed.credential_id, cred);
        assert_eq!(parsed.model, "gpt-5.4");
        let binding = parsed.to_model_binding();
        assert_eq!(binding.model, "gpt-5.4");
        assert_eq!(binding.credential, Some(cred));
    }

    #[test]
    fn rejects_non_uuid_and_xai_keys() {
        assert!(parse_provider_model_key("grok-4.5").is_none());
        assert!(parse_provider_model_key("codex/gpt-5.4").is_none());
        assert!(parse_provider_model_key("codex/not-a-uuid/gpt-5.4").is_none());
    }

    #[test]
    fn resolve_wire_slug_single_multi_provider() {
        let provider = ProviderId::new_unchecked("codex");
        let a = CredentialId::from_uuid(
            Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
        );
        let key = format_provider_model_key(&provider, a, "gpt-5.6-luna");
        let entries = vec![
            (key.as_str(), "gpt-5.6-luna"),
            ("grok-4.5", "grok-4.5"),
        ];
        assert_eq!(
            resolve_wire_slug_to_catalog_key(&entries, "gpt-5.6-luna").as_deref(),
            Some(key.as_str())
        );
    }

    #[test]
    fn resolve_wire_slug_two_multi_provider_is_ambiguous_not_first_wins() {
        let provider = ProviderId::new_unchecked("codex");
        let a = CredentialId::from_uuid(
            Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
        );
        let b = CredentialId::from_uuid(
            Uuid::parse_str("fedcba98-7654-3210-fedc-ba9876543210").unwrap(),
        );
        let k1 = format_provider_model_key(&provider, a, "gpt-5.6-luna");
        let k2 = format_provider_model_key(&provider, b, "gpt-5.6-luna");
        let entries = vec![(k1.as_str(), "gpt-5.6-luna"), (k2.as_str(), "gpt-5.6-luna")];
        assert!(
            resolve_wire_slug_to_catalog_key(&entries, "gpt-5.6-luna").is_none(),
            "must not silently first-wins"
        );
        let msg = ambiguous_multi_provider_slug_message(&entries, "gpt-5.6-luna").unwrap();
        assert!(msg.contains("ambiguous") && msg.contains(&k1) && msg.contains(&k2));
    }
}
