//! Kill switches and product gates for Codex auth (protocol-baseline.md §8, D10).
//!
//! Environment variables:
//! - `GROK_DISABLE_CODEX_AUTH=1` — fully disable Codex provider
//! - `GROK_DISABLE_CODEX_BROWSER_LOGIN=1` — disable browser login only
//! - `GROK_DISABLE_CODEX_DEVICE_LOGIN=1` — disable device login only
//! - `GROK_CODEX_OAUTH_APPROVED=1` — opt-in: allow **new** Codex OAuth login
//!   in product paths (review B5 / D10). Observed client IDs are protocol
//!   evidence only; they are not permission for production login by default.
//! - `GROK_CODEX_CLIENT_ID` — optional explicit client id for approved/dev use
//!   (also counts as an explicit configuration opt-in for login).

/// Whether the entire Codex provider is disabled.
pub fn codex_auth_disabled() -> bool {
    env_enabled("GROK_DISABLE_CODEX_AUTH")
}

/// Whether Codex browser login is disabled.
pub fn codex_browser_login_disabled() -> bool {
    codex_auth_disabled() || env_enabled("GROK_DISABLE_CODEX_BROWSER_LOGIN")
}

/// Whether Codex device login is disabled.
pub fn codex_device_login_disabled() -> bool {
    codex_auth_disabled() || env_enabled("GROK_DISABLE_CODEX_DEVICE_LOGIN")
}

/// Whether **new** Codex OAuth login is allowed in product/CLI paths (B5/D10).
///
/// Fail-closed by default. Existing stored credentials may still resolve
/// tokens/models unless [`codex_auth_disabled`] is set.
///
/// Enable with `GROK_CODEX_OAUTH_APPROVED=1` and/or an explicit
/// `GROK_CODEX_CLIENT_ID` override for approved/dev environments.
pub fn codex_oauth_login_allowed() -> bool {
    if codex_auth_disabled() {
        return false;
    }
    env_enabled("GROK_CODEX_OAUTH_APPROVED")
        || std::env::var("GROK_CODEX_CLIENT_ID")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
}

/// Human-readable reason when login is blocked.
pub fn codex_oauth_login_block_reason() -> Option<&'static str> {
    if codex_auth_disabled() {
        Some("Codex auth is disabled (GROK_DISABLE_CODEX_AUTH)")
    } else if !codex_oauth_login_allowed() {
        Some(
            "Codex OAuth login is fail-closed until approved \
             (set GROK_CODEX_OAUTH_APPROVED=1 or GROK_CODEX_CLIENT_ID for dev/approved use; see D10)",
        )
    } else {
        None
    }
}

fn env_enabled(var: &str) -> bool {
    match std::env::var(var) {
        Ok(v) => v == "1" || v.eq_ignore_ascii_case("true"),
        Err(_) => false,
    }
}

/// Whether the BYOK API-key providers (OpenRouter / Groq / Cloudflare) are
/// disabled. Fail-open by default (BYOK is offline-safe: no OAuth, no
/// refresh, no remote revoke). Set `GROK_DISABLE_BYOK_AUTH=1` to remove
/// them from the default registry.
pub fn byok_auth_disabled() -> bool {
    env_enabled("GROK_DISABLE_BYOK_AUTH")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_login_fail_closed_by_default() {
        // May be set in the environment of the developer machine; only assert
        // the pure logic via direct function composition when disabled.
        if std::env::var_os("GROK_DISABLE_CODEX_AUTH").is_some() {
            assert!(!codex_oauth_login_allowed());
            return;
        }
        // When neither approval nor client id is set, login must be closed.
        // We cannot safely unset env vars of the host; document behavior:
        let approved = std::env::var_os("GROK_CODEX_OAUTH_APPROVED").is_some();
        let client = std::env::var("GROK_CODEX_CLIENT_ID")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        if !approved && !client {
            assert!(!codex_oauth_login_allowed());
            assert!(codex_oauth_login_block_reason().is_some());
        }
    }
}
