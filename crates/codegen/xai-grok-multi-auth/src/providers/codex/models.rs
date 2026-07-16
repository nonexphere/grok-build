//! Codex model catalog fetch (protocol-baseline.md §6 /models).

use serde::Deserialize;
use xai_grok_auth::{ModelCatalog, ProviderError, ProviderModel, StoredCredential};

use super::config::CODEX_BASE_URL;
use super::request_auth::build_codex_request_headers;

/// Wire model entry from ChatGPT Codex `/models`.
#[derive(Debug, Deserialize)]
pub struct CodexModelEntry {
    pub slug: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub context_window: Option<u64>,
    #[serde(default)]
    pub max_context_window: Option<u64>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    models: Vec<CodexModelEntry>,
}

/// Client version query value that returns the full catalog in practice.
/// Some versions (e.g. `0.1.0`) return an empty list; `0.0.0` / `1.0.0` work.
pub const DEFAULT_MODELS_CLIENT_VERSION: &str = "0.0.0";

/// Fetch the Codex model catalog using the credential's access token.
pub async fn fetch_codex_models(
    client: &reqwest::Client,
    credential: &StoredCredential,
    client_version: &str,
) -> Result<ModelCatalog, ProviderError> {
    let headers = build_codex_request_headers(credential)?;
    let url = format!(
        "{CODEX_BASE_URL}/models?client_version={}",
        urlencoding_encode(client_version)
    );

    let mut req = client.get(&url);
    for (name, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            req = req.header(name.as_str(), v);
        }
    }

    let resp = req
        .send()
        .await
        .map_err(|e| ProviderError::Transport(e.to_string()))?;
    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| ProviderError::Transport(e.to_string()))?;

    if !status.is_success() {
        return Err(ProviderError::ModelDiscovery(format!(
            "Codex /models HTTP {status}"
        )));
    }

    let parsed: ModelsResponse = serde_json::from_str(&body).map_err(|e| {
        ProviderError::ModelDiscovery(format!("invalid /models JSON: {e}"))
    })?;

    let models: Vec<ProviderModel> = parsed
        .models
        .into_iter()
        .filter(|m| {
            // Hide entries explicitly marked non-list when present.
            m.visibility
                .as_deref()
                .map(|v| v == "list" || v == "visible" || v == "default")
                .unwrap_or(true)
        })
        .map(|m| {
            let mut capabilities = std::collections::BTreeSet::new();
            capabilities.insert("codex".into());
            let id = m.slug;
            ProviderModel {
                display_name: m.display_name.unwrap_or_else(|| id.clone()),
                id,
                description: m.description,
                context_window: m.context_window.or(m.max_context_window),
                priority: m.priority.unwrap_or(100),
                capabilities,
                raw_metadata: serde_json::Value::Null,
            }
        })
        .collect();

    // Stable order by priority then id.
    let mut models = models;
    models.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));

    Ok(ModelCatalog {
        models,
        etag: None,
        fetched_at: chrono::Utc::now(),
    })
}

fn urlencoding_encode(s: &str) -> String {
    // Minimal query encoding for version strings (alphanumeric + . - _).
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
