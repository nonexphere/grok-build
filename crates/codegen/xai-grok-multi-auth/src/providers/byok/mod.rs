//! BYOK provider descriptors for OpenRouter, Groq, and Cloudflare.
//! Login uses `LoginCoordinator::run_api_key_login`; request auth is static bearer.

use xai_grok_auth::ProviderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByokProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub default_backend: &'static str,
    pub base_url: &'static str,
}

pub const OPENROUTER: ByokProviderSpec = ByokProviderSpec {
    id: "openrouter",
    display_name: "OpenRouter",
    default_backend: "chat_completions",
    base_url: "https://openrouter.ai/api/v1",
};

pub const GROQ: ByokProviderSpec = ByokProviderSpec {
    id: "groq",
    display_name: "Groq",
    default_backend: "chat_completions",
    base_url: "https://api.groq.com/openai/v1",
};

pub const CLOUDFLARE: ByokProviderSpec = ByokProviderSpec {
    id: "cloudflare",
    display_name: "Cloudflare Workers AI",
    default_backend: "chat_completions",
    base_url: "https://api.cloudflare.com/client/v4/accounts",
};

pub const ALL: &[ByokProviderSpec] = &[OPENROUTER, GROQ, CLOUDFLARE];

pub fn provider_id(spec: &ByokProviderSpec) -> ProviderId {
    ProviderId::new_unchecked(spec.id)
}

/// Catalog key format: `{provider}/{credential_id}/{slug}` — never embeds secrets.
pub fn catalog_model_key(provider: &str, credential_id: &str, slug: &str) -> String {
    format!("{provider}/{credential_id}/{slug}")
}

/// Static bearer header value builder (does not log the key).
pub fn static_bearer_authorization(api_key: &str) -> String {
    format!("Bearer {api_key}")
}

/// Binding projection for protocol/App Server — identifiers only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicProviderBinding {
    pub provider_id: String,
    pub credential_id: String,
    pub model_id: String,
    pub backend: String,
    pub binding_revision: u64,
}

impl PublicProviderBinding {
    pub fn from_parts(
        provider: &str,
        credential_id: &str,
        model_id: &str,
        backend: &str,
        binding_revision: u64,
    ) -> Self {
        Self {
            provider_id: provider.into(),
            credential_id: credential_id.into(),
            model_id: model_id.into(),
            backend: backend.into(),
            binding_revision,
        }
    }

    pub fn contains_secret_material(&self) -> bool {
        let blob = format!("{self:?}");
        blob.contains("sk-")
            || blob.contains("Bearer ")
            || blob.contains("api_key")
            || blob.contains("access_token")
    }
}

#[cfg(test)]
mod byok_tests {
    use super::*;

    #[test]
    fn byok_specs_cover_three_verticals_and_catalog_keys() {
        assert_eq!(ALL.len(), 3);
        for spec in ALL {
            assert!(!spec.id.is_empty());
            assert_eq!(spec.default_backend, "chat_completions");
            let key = catalog_model_key(spec.id, "cred1", "model-a");
            assert!(key.starts_with(&format!("{}/", spec.id)));
            assert!(!key.contains("sk-"));
            let pid = provider_id(spec);
            assert_eq!(pid.as_str(), spec.id);
        }
    }

    #[test]
    fn byok_public_binding_has_no_secret_material() {
        let b = PublicProviderBinding::from_parts(
            "openrouter",
            "work",
            "openai/gpt-4o",
            "chat_completions",
            1,
        );
        assert!(!b.contains_secret_material());
        let auth = static_bearer_authorization("sk-secret");
        assert!(auth.starts_with("Bearer "));
        // auth string itself contains secret by design — never put it in PublicProviderBinding
        assert!(!format!("{b:?}").contains("sk-secret"));
    }

    #[test]
    fn byok_two_credentials_same_slug_do_not_collide_in_catalog_key() {
        let a = catalog_model_key("groq", "alice", "llama-3");
        let b = catalog_model_key("groq", "bob", "llama-3");
        assert_ne!(a, b);
    }
}

/// Third-party BYOK must never fall back to XAI_API_KEY.
pub fn reject_xai_api_key_fallback(provider_id: &str, used_env: &str) -> Result<(), String> {
    if provider_id != "xai" && used_env == "XAI_API_KEY" {
        return Err("third-party BYOK must not use XAI_API_KEY".into());
    }
    Ok(())
}

#[cfg(test)]
mod xai_fallback_tests {
    use super::*;
    #[test]
    fn prohibit_xai_api_key_fallback_for_third_party() {
        assert!(reject_xai_api_key_fallback("openrouter", "XAI_API_KEY").is_err());
        assert!(reject_xai_api_key_fallback("openrouter", "GROK_BYOK_API_KEY").is_ok());
        assert!(reject_xai_api_key_fallback("xai", "XAI_API_KEY").is_ok());
    }
}
