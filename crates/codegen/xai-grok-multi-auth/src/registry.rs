//! Provider registry builder with kill-switch awareness.

use std::sync::Arc;

use xai_grok_auth::{AuthProvider, ProviderRegistry};

use crate::kill_switch;
use crate::providers::{CodexAuthProvider, XaiAuthProvider};

/// Build the default provider registry, respecting kill switches.
///
/// - xAI is always registered.
/// - Codex is registered unless `GROK_DISABLE_CODEX_AUTH=1`.
pub fn build_default_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();

    // xAI is always available.
    registry.register(Arc::new(XaiAuthProvider::new())).ok();

    // Codex is gated by kill switch.
    if !kill_switch::codex_auth_disabled() {
        registry.register(Arc::new(CodexAuthProvider::new())).ok();
    }

    registry
}

/// Build a registry with a custom Codex config (for testing).
pub fn build_registry_with_codex_config(codex_config: crate::providers::codex::CodexOAuthConfig) -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(XaiAuthProvider::new())).ok();
    if !kill_switch::codex_auth_disabled() {
        registry
            .register(Arc::new(CodexAuthProvider::with_config(codex_config)))
            .ok();
    }
    registry
}

/// Build a registry with explicit disable flags (for testing without env vars).
pub fn build_registry(disable_codex: bool) -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(XaiAuthProvider::new())).ok();
    if !disable_codex {
        registry.register(Arc::new(CodexAuthProvider::new())).ok();
    }
    registry
}
