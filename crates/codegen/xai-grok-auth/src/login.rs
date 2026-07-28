//! Login flow types used by `AuthProvider::start_login` /
//! `complete_login` / `cancel_login`. Mirrors `task.md` Appendix A.

use std::collections::BTreeSet;
use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::credential::NewCredentialRecord;

/// Which transport a login flow uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginTransport {
    BrowserPkce,
    DeviceCode,
    ApiKey,
}

/// The client surface initiating a login. Providers may surface different
/// UX (browser vs. printed URL) depending on the surface.
#[derive(Debug, Clone)]
pub enum ClientSurface {
    Cli,
    Tui,
    Ide,
    Headless,
}

/// Policy constraints applied to an account during/after login.
#[derive(Debug, Clone, Default)]
pub struct AccountPolicy {
    pub allowed_provider_account_ids: BTreeSet<String>,
    pub require_workspace: bool,
}

/// A request to begin a login flow.
#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub transport: LoginTransport,
    pub requested_alias: Option<String>,
    pub force_reauthentication: bool,
    pub open_browser: bool,
    pub account_policy: AccountPolicy,
    pub client_surface: ClientSurface,
}

/// A unique identifier for an in-flight login flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoginFlowId(Uuid);

impl LoginFlowId {
    /// Generate a new random `LoginFlowId` (UUID v4).
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wrap an existing `Uuid`.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// The underlying `Uuid`.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for LoginFlowId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for LoginFlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// The initial state of a login flow, returned by `start_login`.
#[derive(Debug)]
pub enum LoginStart {
    Browser {
        flow_id: LoginFlowId,
        authorization_url: Url,
        expires_at: DateTime<Utc>,
    },
    Device {
        flow_id: LoginFlowId,
        verification_uri: Url,
        verification_uri_complete: Option<Url>,
        user_code: String,
        expires_at: DateTime<Utc>,
        interval: Duration,
    },
}

/// Input supplied to `complete_login` to advance a flow.
#[derive(Debug)]
pub enum LoginInput {
    BrowserCallback { url: Url },
    Poll,
}

/// The outcome of a `complete_login` step.
#[derive(Debug)]
pub enum LoginCompletion {
    Pending { retry_after: Duration },
    Complete { credential: NewCredentialRecord },
}
