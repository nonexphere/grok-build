//! Provider registry builder with kill-switch awareness.

use std::sync::Arc;

use xai_grok_auth::ProviderRegistry;

use crate::kill_switch;
use crate::providers::byok::{ByokAuthProvider, ALL as BYOK_SPECS, OPENROUTER, GROQ, CLOUDFLARE};
use crate::providers::{CodexAuthProvider, XaiAuthProvider};

/// Build the default provider registry, respecting kill switches.
///
/// - xAI is always registered.
/// - Codex is registered unless `GROK_DISABLE_CODEX_AUTH=1`.
/// - BYOK verticals (OpenRouter, Groq, Cloudflare) are registered unless
///   `GROK_DISABLE_BYOK_AUTH=1`. They advertise only `API_KEY_LOGIN`.
pub fn build_default_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();

    // xAI is always available.
    registry.register(Arc::new(XaiAuthProvider::new())).ok();

    // Codex is gated by kill switch.
    if !kill_switch::codex_auth_disabled() {
        registry.register(Arc::new(CodexAuthProvider::new())).ok();
    }

    // BYOK API-key verticals.
    if !kill_switch::byok_auth_disabled() {
        register_byok(&mut registry);
    }

    registry
}

/// Register the three BYOK verticals into `registry`.
fn register_byok(registry: &mut ProviderRegistry) {
    // Static specs; registration order is stable (openrouter, groq, cloudflare).
    for spec in [OPENROUTER, GROQ, CLOUDFLARE] {
        registry.register(Arc::new(ByokAuthProvider::new(spec))).ok();
    }
    // Defensive: ensure every spec in ALL is covered if the array grows.
    debug_assert_eq!(BYOK_SPECS.len(), 3, "BYOK spec array changed; update register_byok");
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
    if !kill_switch::byok_auth_disabled() {
        register_byok(&mut registry);
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
    // BYOK follows the kill switch in the test builder too, so tests that
    // construct `build_registry` see the same surface as production unless
    // they explicitly disable BYOK via the env gate.
    if !kill_switch::byok_auth_disabled() {
        register_byok(&mut registry);
    }
    registry
}

/// Build a registry with explicit disable flags for both Codex and BYOK
/// (for tests that need a deterministic xAI-only registry without env).
pub fn build_registry_with_flags(disable_codex: bool, disable_byok: bool) -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(XaiAuthProvider::new())).ok();
    if !disable_codex {
        registry.register(Arc::new(CodexAuthProvider::new())).ok();
    }
    if !disable_byok {
        register_byok(&mut registry);
    }
    registry
}
