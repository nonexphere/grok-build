//! Multi-provider credential store backends (canonical implementation).
//!
//! Implements the [`xai_grok_auth::CredentialStore`] trait with file-backed
//! and ephemeral backends. This is the canonical, fully-tested
//! implementation; the shell's `auth::store` module re-exports from here.
//!
//! # Layout
//!
//! All state lives under `<home>/auth/`:
//!
//! ```text
//! {home}/auth/
//!   accounts.json          # public metadata + defaults + aliases
//!   accounts.json.lock     # advisory flock serializing metadata writes
//!   file-secrets.json      # secret material (0o600)
//!   file-secrets.json.lock # advisory flock serializing secret writes
//!   locks/<provider>/<credential-id>.lock  # per-credential refresh lock
//! ```

pub mod composite;
pub mod ephemeral;
pub mod file;
pub mod lock;
pub mod metadata;
pub mod paths;

pub use composite::AutoCredentialStore;
pub use ephemeral::EphemeralCredentialStore;
pub use file::FileCredentialStore;
