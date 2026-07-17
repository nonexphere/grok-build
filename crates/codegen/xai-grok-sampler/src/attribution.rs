//! 401 attribution callback hook for the sampling client.
//!
//! Every 401 response site can optionally emit an attribution event so
//! a downstream observer can split production 401s into "client sent a
//! stale snapshot bearer that the server rejected" vs. "client sent
//! the live token from its auth source and the server still rejected
//! it" buckets.
//!
//! `xai-grok-sampler` is intentionally decoupled from `xai-grok-shell`
//! (no shell types, no logging crate, no auth-manager dependency). The
//! caller wires an implementation of [`Auth401AttributionCallback`]
//! into [`crate::SamplerConfig::attribution_callback`]; the sampler
//! invokes the callback at each UNAUTHORIZED arm with **non-secret**
//! metadata only (`has_sent_auth`, `attempt_id`). The implementation
//! joins that with its own opaque credential identity and emits
//! attribution without token or prefix material.
//!
//! When the callback is `None` (the default), the 401 sites are silent
//! and return the same `SamplingError::Auth` they would otherwise.

use std::sync::Arc;

/// A logical 401-emitting site inside the sampling client. The string
/// identifier ends up in the consumer field of the attribution event
/// so downstream queries can break down 401s by API path.
///
/// # Scope: sampler endpoints only
///
/// This enum enumerates the six HTTP endpoints owned by
/// `SamplingClient` (chat completions, responses, messages -- each in
/// streaming and non-streaming form). It does *not* cover image
/// generation, video generation, web search, or embedding -- those
/// tools live in `xai-grok-tools`
/// (`crates/codegen/xai-grok-tools/src/implementations/`), have their
/// own HTTP clients that do not flow through `SamplingClient`, and
/// hook into the `xai_grok_tools::ApiKeyProvider` trait rather than
/// this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingConsumer {
    /// `chat_completion_stream`: OpenAI-compatible streaming OpenAI Chat Completions API.
    ChatCompletionsStream,
    /// `chat_completion`: OpenAI-compatible non-streaming OpenAI Chat Completions API.
    ChatCompletions,
    /// `create_response_stream`: Responses API streaming.
    ResponsesStream,
    /// `create_response`: Responses API non-streaming.
    Responses,
    /// `messages_stream`: Anthropic Messages API streaming.
    MessagesStream,
    /// `messages`: Anthropic Messages API non-streaming.
    Messages,
}

impl SamplingConsumer {
    /// Stable string identifier for this emit site. Callbacks
    /// typically combine this with a fixed prefix (e.g. the client
    /// type) when building the consumer field of the attribution
    /// event.
    pub fn as_endpoint(self) -> &'static str {
        match self {
            Self::ChatCompletionsStream => "chat_completions_stream",
            Self::ChatCompletions => "chat_completions",
            Self::ResponsesStream => "responses_stream",
            Self::Responses => "responses",
            Self::MessagesStream => "messages_stream",
            Self::Messages => "messages",
        }
    }
}

/// Historical prefix length constant. 401 attribution no longer passes
/// bearer material across the crate boundary; kept only so external
/// references compile. Prefer never logging token material at all.
pub const SENT_BEARER_PREFIX_LEN: usize = 12;
/// Hook invoked by [`crate::SamplingClient`] at every 401 response site.
///
/// Implementations join non-secret request metadata with their own
/// opaque credential identity (provider id, generation, attempt_id)
/// and emit attribution without any token or prefix material.
///
/// Implementations must be cheap to invoke and must not block. They
/// run inside the request's response-handling path and any latency
/// they add is paid by the user-visible 401 error path.
//
// The `Debug` bound is a structural requirement: [`crate::SamplerConfig`]
// derives `Debug` and carries an `Option<Arc<dyn Auth401AttributionCallback>>`
// field, which only compiles when the trait is `Debug`. Do not remove
// the bound when factoring this trait out -- it will break
// `derive(Debug)` on `SamplerConfig`.
pub trait Auth401AttributionCallback: Send + Sync + std::fmt::Debug {
    /// Record a 401 attribution event for one logical 401 response.
    ///
    /// **No credential material** (full token or prefix/suffix) is passed.
    /// Callers may only use non-secret metadata: whether a bearer was present,
    /// multi-provider `attempt_id`, and later opaque generation/fingerprint
    /// fields owned by the shell AuthManager.
    ///
    /// `has_sent_auth` is true when the request carried Authorization/x-api-key.
    /// `attempt_id` is the multi-provider stamp id for the failing request when known.
    fn record_401(
        &self,
        consumer: SamplingConsumer,
        has_sent_auth: bool,
        attempt_id: Option<u64>,
    );
}

/// Shared, cheap-to-clone alias for the attribution callback.
pub type SharedAttributionCallback = Arc<dyn Auth401AttributionCallback>;
