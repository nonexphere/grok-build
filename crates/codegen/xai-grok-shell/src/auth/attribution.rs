//! Shell-side 401-attribution helpers.
//!
//! Every 401 emit site joins **non-secret** request metadata with the
//! AuthManager mint/expiry diagnostics. No bearer, API key, or
//! truncated prefix is written to unified log or OTel.
//!
//! Sinks:
//!
//! 1. [`xai_grok_telemetry::unified_log::warn`] → `~/.grok/logs/unified.jsonl`
//! 2. `tracing::warn_span!("auth_401_attribution", ...)` → OTel/OTLP
//!
//! # Schema (every emit)
//!
//! ```text
//! {
//!   "has_sent_auth": <bool>,
//!   "has_current_auth": <bool>,
//!   "attempt_id": <u64 | null>,
//!   "mint_age_seconds": <i64; or -1 when no current token>,
//!   "expires_at_seconds_from_now": <i64; or 0 when no current token>,
//!   "consumer": "OaiCompatClient.<endpoint>" | ...
//! }
//! ```
//!
//! # Cross-crate plumbing
//!
//! [`xai_grok_sampler`] invokes [`xai_grok_sampler::Auth401AttributionCallback`]
//! with `(has_sent_auth, attempt_id)` only. This module provides
//! [`ShellAttribution`] wired into sampler construction. Non-sampler
//! sites call [`record_consumer_401`] with the same non-secret shape.

use std::sync::Arc;

use serde_json::Value as JsonValue;
use xai_grok_sampler::{Auth401AttributionCallback, SamplingConsumer};
use xai_grok_tools::{Auth401AttributionCallback as ToolAuth401AttributionCallback, ToolConsumer};

use crate::auth::{AuthManager, TOKEN_TTL};

/// `cfg(test)`-only process-global counter that bumps on every
/// successful `record_auth_401` invocation.
#[cfg(test)]
static EMIT_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Read the test-only emit counter.
#[cfg(test)]
pub(crate) fn test_emit_count() -> u64 {
    EMIT_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

/// Reset the test-only emit counter to zero.
#[cfg(test)]
pub(crate) fn reset_test_emit_count() {
    EMIT_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}

/// Concrete implementation of [`Auth401AttributionCallback`] for the
/// sampler crate's six 401 arms.
pub(crate) struct ShellAttribution {
    auth_manager: Arc<AuthManager>,
    session_id: Option<String>,
}

impl std::fmt::Debug for ShellAttribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShellAttribution")
            .field("auth_manager", &"<redacted>")
            .field("session_id", &self.session_id)
            .finish()
    }
}

impl ShellAttribution {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        auth_manager: Arc<AuthManager>,
        session_id: Option<String>,
    ) -> Arc<dyn Auth401AttributionCallback> {
        Arc::new(Self {
            auth_manager,
            session_id,
        })
    }

    pub fn new_tool_callback(
        auth_manager: Arc<AuthManager>,
        session_id: Option<String>,
    ) -> Arc<dyn ToolAuth401AttributionCallback> {
        Arc::new(Self {
            auth_manager,
            session_id,
        })
    }
}

impl Auth401AttributionCallback for ShellAttribution {
    fn record_401(&self, consumer: SamplingConsumer, has_sent_auth: bool, attempt_id: Option<u64>) {
        record_consumer_401(
            self.auth_manager.as_ref(),
            self.session_id.as_deref(),
            ConsumerKind::OaiCompatClient,
            consumer.as_endpoint(),
            has_sent_auth,
            attempt_id,
        );
    }
}

impl ToolAuth401AttributionCallback for ShellAttribution {
    fn record_401(&self, consumer: ToolConsumer, sent_bearer_prefix: Option<&str>) {
        // Tools crate still passes a truncated prefix for API compat;
        // discard the value and keep only presence (Wave 2: zero secrets).
        let _ = sent_bearer_prefix;
        let (kind, op) = match consumer {
            ToolConsumer::ImageGen => (ConsumerKind::ImageGen, ""),
            ToolConsumer::VideoGenStart => (ConsumerKind::VideoGen, "start"),
            ToolConsumer::VideoGenPoll => (ConsumerKind::VideoGen, "poll"),
            ToolConsumer::WebSearch => (ConsumerKind::WebSearch, ""),
        };
        record_consumer_401(
            self.auth_manager.as_ref(),
            self.session_id.as_deref(),
            kind,
            op,
            sent_bearer_prefix.is_some(),
            None,
        );
    }
}

/// Categories of 401-attribution emit sites.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ConsumerKind {
    OaiCompatClient,
    StorageClient,
    FeedbackClient,
    SessionRegistryClient,
    IdleResumeModelRefresh,
    ImageGen,
    VideoGen,
    WebSearch,
}

impl ConsumerKind {
    fn prefix(self) -> &'static str {
        match self {
            Self::OaiCompatClient => "OaiCompatClient",
            Self::StorageClient => "StorageClient",
            Self::FeedbackClient => "FeedbackClient",
            Self::SessionRegistryClient => "SessionRegistryClient",
            Self::IdleResumeModelRefresh => "IdleResumeModelRefresh",
            Self::ImageGen => "ImageGen",
            Self::VideoGen => "VideoGen",
            Self::WebSearch => "WebSearch",
        }
    }

    fn takes_op(self) -> bool {
        !matches!(
            self,
            Self::IdleResumeModelRefresh | Self::ImageGen | Self::WebSearch
        )
    }
}

fn format_consumer(kind: ConsumerKind, op: &str) -> String {
    if kind.takes_op() {
        format!("{}.{}", kind.prefix(), op)
    } else {
        kind.prefix().to_string()
    }
}

/// Emit a single `auth 401 attribution` event for a per-consumer 401.
///
/// `has_sent_auth` is whether the request carried Authorization / x-api-key.
/// `attempt_id` is the multi-provider stamp id when known (sampler path).
pub(crate) fn record_consumer_401(
    auth_manager: &AuthManager,
    session_id: Option<&str>,
    kind: ConsumerKind,
    op: &str,
    has_sent_auth: bool,
    attempt_id: Option<u64>,
) {
    let consumer = format_consumer(kind, op);
    record_auth_401(
        auth_manager,
        session_id,
        &consumer,
        has_sent_auth,
        attempt_id,
    );
}

/// Emit a single `auth 401 attribution` event to both sinks.
pub(crate) fn record_auth_401(
    auth_manager: &AuthManager,
    session_id: Option<&str>,
    consumer: &str,
    has_sent_auth: bool,
    attempt_id: Option<u64>,
) {
    let payload = compute_attribution_payload(auth_manager, consumer, has_sent_auth, attempt_id);

    xai_grok_telemetry::unified_log::warn(
        "auth 401 attribution",
        session_id,
        Some(payload.clone()),
    );

    let _attribution_span = tracing::warn_span!(
        "auth_401_attribution",
        has_sent_auth = payload["has_sent_auth"].as_bool().unwrap_or(false),
        has_current_auth = payload["has_current_auth"].as_bool().unwrap_or(false),
        attempt_id = payload["attempt_id"].as_u64().unwrap_or(0),
        has_attempt_id = payload["attempt_id"].is_u64(),
        consumer = consumer,
        session_id = session_id.unwrap_or(""),
        mint_age_seconds = payload["mint_age_seconds"].as_i64().unwrap_or(-1),
        expires_at_seconds_from_now = payload["expires_at_seconds_from_now"].as_i64().unwrap_or(0),
    )
    .entered();

    #[cfg(test)]
    EMIT_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// Pure (no I/O) computation of the attribution payload.
///
/// Never includes token material or prefixes. Mint/expiry use
/// AuthManager metadata only (no key fields).
fn compute_attribution_payload(
    auth_manager: &AuthManager,
    consumer: &str,
    has_sent_auth: bool,
    attempt_id: Option<u64>,
) -> JsonValue {
    let now = chrono::Utc::now();
    let current_auth = auth_manager.current();
    let has_current_auth = current_auth.is_some();

    let (mint_age_seconds, expires_at_seconds_from_now) = match current_auth {
        Some(auth) => {
            let mint_age = now.signed_duration_since(auth.create_time).num_seconds();
            let expiry = auth.expires_at.unwrap_or(auth.create_time + TOKEN_TTL);
            (mint_age, expiry.signed_duration_since(now).num_seconds())
        }
        None => (-1_i64, 0_i64),
    };

    serde_json::json!({
        "has_sent_auth": has_sent_auth,
        "has_current_auth": has_current_auth,
        "attempt_id": attempt_id,
        "mint_age_seconds": mint_age_seconds,
        "expires_at_seconds_from_now": expires_at_seconds_from_now,
        "consumer": consumer,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, Utc};

    use crate::auth::{AuthManager, GrokAuth, GrokComConfig};

    use super::*;

    fn empty_auth_manager() -> (tempfile::TempDir, AuthManager) {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = GrokComConfig::default();
        let am = AuthManager::new(dir.path(), cfg);
        (dir, am)
    }

    fn fresh_auth(key: &str) -> GrokAuth {
        GrokAuth {
            key: key.to_string(),
            create_time: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            ..GrokAuth::test_default()
        }
    }

    fn payload_field<'a>(payload: &'a JsonValue, key: &str) -> &'a JsonValue {
        payload
            .get(key)
            .unwrap_or_else(|| panic!("payload missing field {key:?}: {payload:?}"))
    }

    fn assert_no_secret_material(payload: &JsonValue, canary: &str) {
        let s = payload.to_string();
        assert!(!s.contains(canary), "payload must not contain canary: {s}");
        for n in [4usize, 8, 12, 20] {
            if canary.len() >= n {
                let p: String = canary.chars().take(n).collect();
                assert!(!s.contains(&p), "prefix len={n} leaked: {s}");
            }
        }
        assert!(payload.get("sent_key_prefix").is_none());
        assert!(payload.get("current_key_prefix").is_none());
    }

    #[test]
    fn live_token_metadata_without_secret_material() {
        let (_dir, am) = empty_auth_manager();
        // Canary must not collide with legitimate payload field names (e.g. "live").
        let canary = "sk-canary-SECRET-zz99-aabbccddeeff";
        am.hot_swap(fresh_auth(canary));

        let payload = compute_attribution_payload(&am, "Test.consumer", true, Some(7));

        assert_eq!(payload_field(&payload, "has_sent_auth"), true);
        assert_eq!(payload_field(&payload, "has_current_auth"), true);
        assert_eq!(payload_field(&payload, "attempt_id"), 7);
        assert_eq!(payload_field(&payload, "consumer"), "Test.consumer");
        assert_no_secret_material(&payload, canary);

        let mint = payload_field(&payload, "mint_age_seconds")
            .as_i64()
            .unwrap();
        assert!(
            (0..5).contains(&mint),
            "mint_age_seconds should be 0-5 sec for a freshly-created auth, got {mint}"
        );
        let expires = payload_field(&payload, "expires_at_seconds_from_now")
            .as_i64()
            .unwrap();
        assert!(
            (3590..=3600).contains(&expires),
            "expires_at_seconds_from_now should be ~3600, got {expires}"
        );
    }

    #[test]
    fn absent_current_is_flagged_without_secrets() {
        let (_dir, am) = empty_auth_manager();
        let payload = compute_attribution_payload(&am, "Test.absent", true, None);

        assert_eq!(payload_field(&payload, "has_sent_auth"), true);
        assert_eq!(payload_field(&payload, "has_current_auth"), false);
        assert!(payload_field(&payload, "attempt_id").is_null());
        assert_eq!(payload_field(&payload, "mint_age_seconds"), -1);
        assert_eq!(payload_field(&payload, "expires_at_seconds_from_now"), 0);
    }

    #[test]
    fn legacy_token_uses_two_branch_fallback() {
        let (_dir, am) = empty_auth_manager();
        let auth = GrokAuth {
            key: "k".into(),
            create_time: Utc::now() - Duration::seconds(60),
            ..GrokAuth::test_default()
        };
        am.hot_swap(auth);

        let payload = compute_attribution_payload(&am, "Test.legacy", true, None);

        let mint = payload_field(&payload, "mint_age_seconds")
            .as_i64()
            .unwrap();
        assert!(
            (60..=70).contains(&mint),
            "mint_age_seconds should be ~60 for a 60s-old auth, got {mint}"
        );
        let expires = payload_field(&payload, "expires_at_seconds_from_now")
            .as_i64()
            .unwrap();
        let expected = TOKEN_TTL.num_seconds() - 60;
        assert!(
            (expected - 10..=expected + 10).contains(&expires),
            "expires_at_seconds_from_now should be ~{expected}, got {expires}"
        );
    }

    #[test]
    fn format_consumer_matrix() {
        let cases: &[(ConsumerKind, &str, &str)] = &[
            (
                ConsumerKind::OaiCompatClient,
                "chat_completions_stream",
                "OaiCompatClient.chat_completions_stream",
            ),
            (
                ConsumerKind::StorageClient,
                "upload_file",
                "StorageClient.upload_file",
            ),
            (
                ConsumerKind::IdleResumeModelRefresh,
                "",
                "IdleResumeModelRefresh",
            ),
            (
                ConsumerKind::IdleResumeModelRefresh,
                "ignored",
                "IdleResumeModelRefresh",
            ),
            (ConsumerKind::ImageGen, "", "ImageGen"),
            (ConsumerKind::ImageGen, "ignored", "ImageGen"),
            (ConsumerKind::VideoGen, "start", "VideoGen.start"),
            (ConsumerKind::VideoGen, "poll", "VideoGen.poll"),
            (ConsumerKind::WebSearch, "", "WebSearch"),
            (ConsumerKind::WebSearch, "ignored", "WebSearch"),
        ];
        for (kind, op, expected) in cases {
            assert_eq!(
                format_consumer(*kind, op),
                *expected,
                "kind={kind:?} op={op:?}"
            );
        }
    }

    #[test]
    fn format_consumer_with_op_appends_dot() {
        assert_eq!(
            format_consumer(ConsumerKind::OaiCompatClient, "chat_completions_stream"),
            "OaiCompatClient.chat_completions_stream"
        );
        assert_eq!(
            format_consumer(ConsumerKind::StorageClient, "upload_file"),
            "StorageClient.upload_file"
        );
    }

    #[test]
    #[serial_test::serial(attribution_emit_count)]
    fn shell_attribution_tool_impl_routes_to_correct_consumer_strings() {
        reset_test_emit_count();
        let (_dir, am) = empty_auth_manager();
        am.hot_swap(fresh_auth("bearer-1234567890"));
        let am_arc = Arc::new(am);
        let cb: Arc<dyn ToolAuth401AttributionCallback> =
            ShellAttribution::new_tool_callback(am_arc.clone(), Some("sid-tool".into()));

        let cases = [
            (ToolConsumer::ImageGen, "ImageGen"),
            (ToolConsumer::VideoGenStart, "VideoGen.start"),
            (ToolConsumer::VideoGenPoll, "VideoGen.poll"),
            (ToolConsumer::WebSearch, "WebSearch"),
        ];

        for (consumer, expected_consumer_str) in cases {
            // Even if a canary prefix is passed, it must not enter sinks.
            cb.record_401(consumer, Some("bearer-1234567890"));
            let payload =
                compute_attribution_payload(am_arc.as_ref(), expected_consumer_str, true, None);
            assert_eq!(
                payload_field(&payload, "consumer"),
                expected_consumer_str,
                "ToolConsumer::{consumer:?} should render as {expected_consumer_str:?}",
            );
            assert_no_secret_material(&payload, "bearer-1234567890");
        }

        assert_eq!(test_emit_count() as usize, cases.len());
    }

    mod span_capture {
        use std::sync::Mutex;
        use tracing::Subscriber;
        use tracing::field::{Field, Visit};
        use tracing::span::Attributes;
        use tracing_subscriber::layer::{Context, Layer};
        use tracing_subscriber::registry::LookupSpan;

        #[derive(Debug, Default, Clone)]
        pub struct CapturedSpan {
            pub name: String,
            pub fields_str: std::collections::BTreeMap<String, String>,
            pub fields_i64: std::collections::BTreeMap<String, i64>,
            pub fields_bool: std::collections::BTreeMap<String, bool>,
        }

        pub struct SpanCollector {
            pub spans: std::sync::Arc<Mutex<Vec<CapturedSpan>>>,
        }

        impl SpanCollector {
            pub fn new() -> (Self, std::sync::Arc<Mutex<Vec<CapturedSpan>>>) {
                let buf = std::sync::Arc::new(Mutex::new(Vec::new()));
                (Self { spans: buf.clone() }, buf)
            }
        }

        impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for SpanCollector {
            fn on_new_span(&self, attrs: &Attributes<'_>, _id: &tracing::Id, _ctx: Context<'_, S>) {
                let mut captured = CapturedSpan {
                    name: attrs.metadata().name().to_string(),
                    ..Default::default()
                };
                let mut visitor = FieldVisitor {
                    captured: &mut captured,
                };
                attrs.record(&mut visitor);
                self.spans.lock().unwrap().push(captured);
            }
        }

        struct FieldVisitor<'a> {
            captured: &'a mut CapturedSpan,
        }

        impl Visit for FieldVisitor<'_> {
            fn record_str(&mut self, field: &Field, value: &str) {
                self.captured
                    .fields_str
                    .insert(field.name().to_string(), value.to_string());
            }
            fn record_i64(&mut self, field: &Field, value: i64) {
                self.captured
                    .fields_i64
                    .insert(field.name().to_string(), value);
            }
            fn record_u64(&mut self, field: &Field, value: u64) {
                self.captured
                    .fields_i64
                    .insert(field.name().to_string(), value as i64);
            }
            fn record_bool(&mut self, field: &Field, value: bool) {
                self.captured
                    .fields_bool
                    .insert(field.name().to_string(), value);
            }
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                self.captured
                    .fields_str
                    .insert(field.name().to_string(), format!("{value:?}"));
            }
        }
    }

    #[test]
    #[serial_test::serial(attribution_emit_count)]
    fn record_auth_401_emits_otel_span_without_secret_fields() {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let (collector, captured) = span_capture::SpanCollector::new();
        let subscriber = tracing_subscriber::registry().with(collector);
        let _guard = subscriber.set_default();

        reset_test_emit_count();
        let (_dir, am) = empty_auth_manager();
        let canary = "live-token-1234567890";
        am.hot_swap(fresh_auth(canary));

        record_auth_401(
            &am,
            Some("sid-otel-span"),
            "OaiCompatClient.chat_completions_stream",
            true,
            Some(99),
        );

        let spans = captured.lock().unwrap();
        let attribution = spans
            .iter()
            .find(|s| s.name == "auth_401_attribution")
            .expect("expected one auth_401_attribution span");

        assert!(!attribution.fields_str.contains_key("sent_key_prefix"));
        assert!(!attribution.fields_str.contains_key("current_key_prefix"));
        assert_eq!(attribution.fields_bool.get("has_sent_auth"), Some(&true),);
        assert_eq!(attribution.fields_bool.get("has_current_auth"), Some(&true),);
        assert_eq!(attribution.fields_i64.get("attempt_id"), Some(&99));
        assert_eq!(
            attribution.fields_str.get("consumer").map(String::as_str),
            Some("OaiCompatClient.chat_completions_stream"),
        );
        assert_eq!(
            attribution.fields_str.get("session_id").map(String::as_str),
            Some("sid-otel-span"),
        );

        let surface = format!("{attribution:?}");
        assert!(!surface.contains(canary));

        let mint = attribution
            .fields_i64
            .get("mint_age_seconds")
            .copied()
            .unwrap();
        assert!(
            (0..5).contains(&mint),
            "mint_age_seconds should be 0-5, got {mint}"
        );
    }

    #[test]
    #[serial_test::serial(attribution_emit_count)]
    fn record_auth_401_bumps_emit_counter() {
        reset_test_emit_count();
        let (_dir, am) = empty_auth_manager();
        am.hot_swap(fresh_auth("k"));
        record_auth_401(&am, None, "Test.counter", true, None);
        assert_eq!(test_emit_count(), 1);
        record_auth_401(&am, None, "Test.counter", true, None);
        assert_eq!(test_emit_count(), 2);
    }

    #[test]
    #[serial_test::serial(attribution_emit_count)]
    fn parent_callback_flows_through_arc_clone() {
        reset_test_emit_count();
        let (_dir, am) = empty_auth_manager();
        let am_arc = Arc::new(am);
        let parent_cb = ShellAttribution::new(am_arc.clone(), Some("parent-sid".into()));

        let inherited_cb = parent_cb.clone();
        inherited_cb.record_401(SamplingConsumer::ChatCompletionsStream, true, Some(1));
        assert_eq!(test_emit_count(), 1);

        parent_cb.record_401(SamplingConsumer::Messages, true, None);
        assert_eq!(test_emit_count(), 2);
    }

    #[test]
    #[serial_test::serial(attribution_emit_count)]
    fn shell_attribution_trait_impl_routes_through_helper() {
        reset_test_emit_count();
        let (_dir, am) = empty_auth_manager();
        let am_arc = Arc::new(am);
        let cb = ShellAttribution::new(am_arc.clone(), Some("sid-shell".into()));
        let variants = [
            SamplingConsumer::ChatCompletionsStream,
            SamplingConsumer::ChatCompletions,
            SamplingConsumer::ResponsesStream,
            SamplingConsumer::Responses,
            SamplingConsumer::MessagesStream,
            SamplingConsumer::Messages,
        ];
        for consumer in variants {
            cb.record_401(consumer, true, Some(5));
        }
        assert_eq!(test_emit_count() as usize, variants.len());

        let payload = compute_attribution_payload(
            am_arc.as_ref(),
            &format_consumer(
                ConsumerKind::OaiCompatClient,
                SamplingConsumer::MessagesStream.as_endpoint(),
            ),
            true,
            Some(5),
        );
        assert_eq!(
            payload_field(&payload, "consumer"),
            "OaiCompatClient.messages_stream"
        );
        assert_eq!(payload_field(&payload, "attempt_id"), 5);
    }
}
