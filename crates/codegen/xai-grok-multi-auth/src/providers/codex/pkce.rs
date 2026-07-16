//! PKCE (Proof Key for Code Exchange) generation — S256 (protocol-baseline.md §3.2).

use base64::Engine;
use sha2::{Digest, Sha256};

/// A PKCE verifier and its S256 challenge.
///
/// `Debug` redacts the verifier (and challenge) so secrets never hit logs.
#[derive(Clone)]
pub struct PkceVerifier {
    verifier: String,
    challenge: String,
}

impl std::fmt::Debug for PkceVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PkceVerifier")
            .field("verifier", &"<redacted>")
            .field("challenge", &"<redacted>")
            .finish()
    }
}

impl PkceVerifier {
    /// Generate a new PKCE pair: 32 random bytes from the OS CSPRNG,
    /// base64url-no-padding verifier, SHA-256 S256 challenge.
    pub fn new() -> Self {
        let bytes: [u8; 32] = rand::random();
        let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge_bytes = hasher.finalize();
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge_bytes);

        Self {
            verifier,
            challenge,
        }
    }

    /// The code verifier (sent to the token endpoint).
    pub fn verifier(&self) -> &str {
        &self.verifier
    }

    /// The S256 code challenge (sent in the authorization URL).
    pub fn challenge(&self) -> &str {
        &self.challenge
    }
}

impl Default for PkceVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a cryptographically random state string (base64url, 16 bytes).
pub fn generate_state() -> String {
    let bytes: [u8; 16] = rand::random();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_is_s256_base64url() {
        let pkce = PkceVerifier::new();
        // Verifier should be 43 chars (32 bytes base64url no pad).
        assert_eq!(pkce.verifier().len(), 43);
        // Challenge should be 43 chars (SHA-256 = 32 bytes base64url no pad).
        assert_eq!(pkce.challenge().len(), 43);
        // Challenge must not equal verifier.
        assert_ne!(pkce.verifier(), pkce.challenge());
    }

    #[test]
    fn pkce_challenge_matches_verifier() {
        // Verify the challenge is actually SHA-256(verifier) base64url.
        let pkce = PkceVerifier::new();
        let mut hasher = Sha256::new();
        hasher.update(pkce.verifier().as_bytes());
        let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());
        assert_eq!(pkce.challenge(), expected);
    }

    #[test]
    fn state_is_unique() {
        let s1 = generate_state();
        let s2 = generate_state();
        assert_ne!(s1, s2);
        assert!(!s1.is_empty());
    }
}
