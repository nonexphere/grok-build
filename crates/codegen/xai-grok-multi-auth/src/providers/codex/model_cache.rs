//! Per-credential Codex model catalog cache (D9 / M7 / AUD-010).
//!
//! Path: `{grok_home}/cache/models/codex/<credential-id>.json`
//!
//! Policy (protocol-baseline.md §7):
//! - Fresh TTL: 5 minutes
//! - Separate cache per credential
//! - Serve stale when offline / transient 5xx
//! - **Never** serve stale as healthy on 401/403/identity failures
//! - Bundled fallback when no cache exists
//! - Atomic write (tmp + rename), owner-only mode when possible
//! - Do not delete cache on transient errors

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use xai_grok_auth::{CredentialId, ModelCatalog, ModelCatalogSource, ProviderModel};

/// Freshness TTL for the on-disk model catalog.
pub const MODEL_CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// On-disk cache envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModelCatalog {
    pub models: Vec<ProviderModel>,
    pub etag: Option<String>,
    pub fetched_at: DateTime<Utc>,
    /// True when the catalog came from the bundled fallback, not network.
    #[serde(default)]
    pub from_bundled: bool,
    /// True when served past TTL (offline / error fallback).
    #[serde(default)]
    pub stale: bool,
}

impl CachedModelCatalog {
    pub fn is_fresh(&self, now: DateTime<Utc>) -> bool {
        if self.from_bundled {
            return false;
        }
        let age = now.signed_duration_since(self.fetched_at);
        age.to_std().map(|d| d < MODEL_CACHE_TTL).unwrap_or(false)
    }

    pub fn into_model_catalog(self, source: ModelCatalogSource) -> ModelCatalog {
        ModelCatalog {
            models: self.models,
            etag: self.etag,
            fetched_at: self.fetched_at,
            source,
            is_stale: self.stale || matches!(source, ModelCatalogSource::StaleDisk),
        }
    }
}

/// Resolve cache path for a credential under a grok home.
pub fn cache_path(grok_home: &Path, credential_id: CredentialId) -> PathBuf {
    grok_home
        .join("cache")
        .join("models")
        .join("codex")
        .join(format!("{credential_id}.json"))
}

/// Minimal bundled fallback when no cache and network fail.
pub fn bundled_fallback_catalog() -> CachedModelCatalog {
    let mut capabilities = std::collections::BTreeSet::new();
    capabilities.insert("codex".into());
    let models = vec![
        ProviderModel {
            id: "gpt-5.4".into(),
            display_name: "GPT-5.4".into(),
            description: Some("Bundled Codex fallback model".into()),
            context_window: Some(256_000),
            priority: 10,
            capabilities: capabilities.clone(),
            raw_metadata: serde_json::json!({"source": "bundled"}),
        },
        ProviderModel {
            id: "gpt-5.6-luna".into(),
            display_name: "GPT-5.6 Luna".into(),
            description: Some("Bundled Codex fallback model".into()),
            context_window: Some(256_000),
            priority: 20,
            capabilities,
            raw_metadata: serde_json::json!({"source": "bundled"}),
        },
    ];
    CachedModelCatalog {
        models,
        etag: None,
        fetched_at: Utc::now(),
        from_bundled: true,
        stale: false,
    }
}

/// Load cache from disk if present.
pub fn load_cache(path: &Path) -> Option<CachedModelCatalog> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Persist a successful fetch with atomic tmp+rename. Logs errors; never panics.
///
/// On Unix, the temp file is created with mode 0o600 (owner read/write only).
pub fn save_cache(path: &Path, catalog: &CachedModelCatalog) {
    if let Err(e) = save_cache_inner(path, catalog) {
        // multi-auth has no tracing dep; keep failure observable on stderr.
        eprintln!("codex model cache: failed to save {}: {e}", path.display());
    }
}

fn save_cache_inner(path: &Path, catalog: &CachedModelCatalog) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create_dir: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(catalog).map_err(|e| format!("serialize: {e}"))?;
    let tmp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    {
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&tmp).map_err(|e| format!("open tmp: {e}"))?;
        f.write_all(&bytes).map_err(|e| format!("write tmp: {e}"))?;
        f.sync_all().map_err(|e| format!("sync tmp: {e}"))?;
    }
    #[cfg(windows)]
    {
        let _ = std::fs::remove_file(path);
    }
    std::fs::rename(&tmp, path).map_err(|e| format!("rename: {e}"))?;
    Ok(())
}

/// Convert a live [`ModelCatalog`] into a cache envelope.
pub fn from_live_catalog(catalog: ModelCatalog) -> CachedModelCatalog {
    CachedModelCatalog {
        models: catalog.models,
        etag: catalog.etag,
        fetched_at: catalog.fetched_at,
        from_bundled: false,
        stale: false,
    }
}

/// True when a fetch error string indicates auth/identity failure (AUD-010).
///
/// These must **not** fall back to serving stale disk as if the catalog were OK.
pub fn is_auth_or_identity_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("401")
        || lower.contains("403")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("reauth")
        || lower.contains("identity")
        || lower.contains("account mismatch")
}

/// Decide what to serve after a fetch attempt.
///
/// - Live Ok → save and return fresh (source=Network)
/// - Live Err auth/identity → empty catalog with source=AuthFailure (no stale)
/// - Live Err transient + fresh cache → FreshDisk
/// - Live Err transient + stale cache → StaleDisk
/// - Live Err transient + no cache → Bundled
pub fn resolve_after_fetch(
    fetch_result: Result<ModelCatalog, String>,
    cache_path: &Path,
    now: DateTime<Utc>,
) -> (ModelCatalog, CacheSource) {
    match fetch_result {
        Ok(catalog) => {
            let cached = from_live_catalog(catalog);
            save_cache(cache_path, &cached);
            let mut out = cached.into_model_catalog(ModelCatalogSource::Network);
            out.source = ModelCatalogSource::Network;
            out.is_stale = false;
            (out, CacheSource::Network)
        }
        Err(err) if is_auth_or_identity_error(&err) => {
            eprintln!("codex model cache: auth/identity fetch failure; not serving stale: {err}");
            (
                ModelCatalog {
                    models: Vec::new(),
                    etag: None,
                    fetched_at: now,
                    source: ModelCatalogSource::AuthFailure,
                    is_stale: false,
                },
                CacheSource::AuthFailure,
            )
        }
        Err(_err) => {
            if let Some(mut c) = load_cache(cache_path) {
                let (source, msrc) = if c.is_fresh(now) {
                    (CacheSource::FreshDisk, ModelCatalogSource::FreshDisk)
                } else {
                    c.stale = true;
                    (CacheSource::StaleDisk, ModelCatalogSource::StaleDisk)
                };
                (c.into_model_catalog(msrc), source)
            } else {
                let bundled = bundled_fallback_catalog();
                (
                    bundled.into_model_catalog(ModelCatalogSource::Bundled),
                    CacheSource::Bundled,
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheSource {
    Network,
    FreshDisk,
    StaleDisk,
    Bundled,
    /// 401/403/identity — do not treat as a usable model list.
    AuthFailure,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn cache_path_is_per_credential() {
        let home = Path::new("/tmp/grok-home");
        let a = CredentialId::from_uuid(
            uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        );
        let b = CredentialId::from_uuid(
            uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
        );
        let pa = cache_path(home, a);
        let pb = cache_path(home, b);
        assert_ne!(pa, pb);
        assert!(pa.to_string_lossy().contains("cache/models/codex/"));
        assert!(pa.to_string_lossy().ends_with(&format!("{a}.json")));
    }

    #[test]
    fn fresh_within_ttl_stale_after() {
        let mut c = bundled_fallback_catalog();
        c.from_bundled = false;
        c.fetched_at = Utc::now();
        assert!(c.is_fresh(Utc::now()));
        c.fetched_at = Utc::now() - chrono::Duration::minutes(10);
        assert!(!c.is_fresh(Utc::now()));
    }

    #[test]
    fn save_load_roundtrip_atomic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cred.json");
        let mut c = bundled_fallback_catalog();
        c.from_bundled = false;
        c.etag = Some("etag-1".into());
        save_cache(&path, &c);
        assert!(path.exists());
        // No leftover tmp files.
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty(), "atomic save must rename tmp away");
        let loaded = load_cache(&path).expect("load");
        assert_eq!(loaded.models.len(), c.models.len());
        assert_eq!(loaded.etag.as_deref(), Some("etag-1"));
    }

    #[test]
    fn resolve_after_fetch_saves_network() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("c.json");
        let live = ModelCatalog {
            models: vec![],
            etag: Some("e".into()),
            fetched_at: Utc::now(),
            source: ModelCatalogSource::Unknown,
            is_stale: false,
        };
        let (out, src) = resolve_after_fetch(Ok(live), &path, Utc::now());
        assert_eq!(src, CacheSource::Network);
        assert_eq!(out.source, ModelCatalogSource::Network);
        assert!(!out.is_stale);
        assert!(path.exists());
        assert_eq!(out.etag.as_deref(), Some("e"));
    }

    #[test]
    fn resolve_after_fetch_uses_stale_on_transient_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("c.json");
        let mut c = bundled_fallback_catalog();
        c.from_bundled = false;
        c.fetched_at = Utc::now() - chrono::Duration::hours(1);
        c.models[0].id = "cached-model".into();
        save_cache(&path, &c);
        let (out, src) = resolve_after_fetch(Err("5xx gateway".into()), &path, Utc::now());
        assert_eq!(src, CacheSource::StaleDisk);
        assert_eq!(out.source, ModelCatalogSource::StaleDisk);
        assert!(out.is_stale);
        assert_eq!(out.models[0].id, "cached-model");
        assert!(path.exists());
    }

    #[test]
    fn resolve_after_fetch_auth_error_does_not_serve_stale() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("c.json");
        let mut c = bundled_fallback_catalog();
        c.from_bundled = false;
        c.fetched_at = Utc::now() - chrono::Duration::hours(1);
        c.models[0].id = "should-not-appear".into();
        save_cache(&path, &c);
        let (out, src) =
            resolve_after_fetch(Err("Codex /models HTTP 401".into()), &path, Utc::now());
        assert_eq!(src, CacheSource::AuthFailure);
        assert_eq!(out.source, ModelCatalogSource::AuthFailure);
        assert!(out.models.is_empty(), "401 must not mask as healthy stale");
        assert!(path.exists(), "cache file still kept for later");
    }

    #[test]
    fn resolve_after_fetch_bundled_when_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.json");
        let (out, src) = resolve_after_fetch(Err("down".into()), &path, Utc::now());
        assert_eq!(src, CacheSource::Bundled);
        assert_eq!(out.source, ModelCatalogSource::Bundled);
        assert!(!out.models.is_empty());
    }

    #[test]
    fn is_auth_or_identity_error_classifies() {
        assert!(is_auth_or_identity_error("HTTP 401"));
        assert!(is_auth_or_identity_error("forbidden 403"));
        assert!(is_auth_or_identity_error("Unauthorized"));
        assert!(!is_auth_or_identity_error("HTTP 503"));
        assert!(!is_auth_or_identity_error("timeout"));
    }
}
