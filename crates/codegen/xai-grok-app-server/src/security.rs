//! Control-plane security canaries for App Server release hardening.

/// Secret shapes that must never appear in logs, errors, or tool projections.
pub const SECRET_CANARIES: &[&str] = &[
    "sk-",
    "Bearer ",
    "access_token",
    "refresh_token",
    "client_secret",
    "XAI_API_KEY",
    "GROK_TEST_SECRET_CANARY",
];

pub fn assert_no_secret_canaries(surface: &str) -> Result<(), String> {
    for canary in SECRET_CANARIES {
        if surface.contains(canary) {
            return Err(format!("secret canary {canary:?} present in surface"));
        }
    }
    Ok(())
}

/// Label cleartext non-loopback binds as experimental/unsafe.
pub fn remote_bind_label(host: &str, cleartext: bool) -> &'static str {
    let loopback = matches!(host, "127.0.0.1" | "::1" | "localhost");
    match (loopback, cleartext) {
        (true, _) => "loopback",
        (false, true) => "experimental/unsafe-cleartext-remote",
        (false, false) => "remote-tls-required",
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn secret_canaries_detect_common_leaks() {
        assert!(assert_no_secret_canaries("ok path").is_ok());
        assert!(assert_no_secret_canaries("Authorization Bearer sk-abc").is_err());
        assert!(assert_no_secret_canaries("XAI_API_KEY=1").is_err());
    }

    #[test]
    fn remote_cleartext_is_labeled_experimental_unsafe() {
        assert_eq!(remote_bind_label("127.0.0.1", true), "loopback");
        assert_eq!(
            remote_bind_label("0.0.0.0", true),
            "experimental/unsafe-cleartext-remote"
        );
        assert_eq!(remote_bind_label("0.0.0.0", false), "remote-tls-required");
    }
}
