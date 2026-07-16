//! Multi-provider credential store — re-exports from `xai-grok-multi-auth`.
//!
//! The canonical, fully-tested implementation lives in the
//! `xai-grok-multi-auth` crate. This module re-exports the store backends
//! so existing `crate::auth::store::*` users compile unchanged.
//!
//! See `crates/codegen/xai-grok-multi-auth/src/store/` for the source.

pub use xai_grok_multi_auth::store::{
    AutoCredentialStore, EphemeralCredentialStore, FileCredentialStore,
};
