//! Sampling log — emits `tracing` events with `target: "sampling_log"`.
//! A dedicated layer in `xai-grok-telemetry` routes these to
//! `~/.grok/logs/sampling.jsonl`. Enable with `--log-sampling`.

use crate::types::RequestId;

pub const TARGET: &str = "sampling_log";

/// Auth metadata safe for info-level sampling spans.
///
/// **Never** carries bearer / API-key material (including truncated prefixes).
/// Scheme/presence only — audit CRITICAL secret-prefix logging.
#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub auth_type: &'static str,
    /// Deprecated residual field: always `None` in production. Kept so older
    /// call sites that still pattern-match compile; must not be populated.
    pub auth_prefix: Option<String>,
}

impl AuthInfo {
    pub fn scheme_only(auth_type: &'static str) -> Self {
        Self {
            auth_type,
            auth_prefix: None,
        }
    }
}

pub fn request_span(
    request_id: &RequestId,
    model: &str,
    api_backend: &str,
    base_url: &str,
    auth: &AuthInfo,
) -> tracing::Span {
    // Do not emit auth_prefix (or any secret material) on the info span.
    tracing::info_span!(
        target: TARGET,
        "sampling_request",
        request_id = %request_id,
        model = model,
        api_backend = api_backend,
        base_url = base_url,
        auth_type = auth.auth_type,
        // Recorded from `SamplerConfig` / response usage as the request
        // progresses; `field::Empty` lets callers `record()` them later.
        reasoning_effort = tracing::field::Empty,
        output_tokens = tracing::field::Empty,
        reasoning_tokens = tracing::field::Empty,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RequestId;

    #[test]
    fn request_span_source_contract_has_no_auth_prefix_field() {
        // Structural: the info_span! field list must not emit secret prefixes.
        let src = include_str!("sampling_log.rs");
        let span_fn = src
            .split("pub fn request_span")
            .nth(1)
            .expect("request_span");
        let macro_body = span_fn
            .split("tracing::info_span!")
            .nth(1)
            .and_then(|s| s.split("\n    )").next())
            .expect("info_span! body");
        assert!(
            !macro_body.contains("auth_prefix"),
            "info_span! must not emit auth_prefix field: {macro_body}"
        );
        assert!(macro_body.contains("auth_type"));
    }

    #[test]
    fn auth_info_scheme_only_never_carries_prefix() {
        let info = AuthInfo::scheme_only("bearer");
        assert!(info.auth_prefix.is_none());
        let dbg = format!("{info:?}");
        assert!(!dbg.contains("sk-"));
        let _ = RequestId::random(); // keep import used if needed
    }
}
