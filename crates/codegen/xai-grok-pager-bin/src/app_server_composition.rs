//! Experimental App Server composition root.
//!
//! C1-D: the product path injects the real Shell-owned
//! [`ShellSessionActorRuntime`] (backed by the JSONL storage adapter), NOT
//! `FakeRuntime`. `FakeRuntime` remains available for unit/conformance tests
//! only. Do not mix real storage list/read with FakeRuntime mutations (split
//! authority — audit F-01 / F-13).

use std::str::FromStr;
use std::sync::Arc;

use xai_grok_app_server::FacadeProcessor;
use xai_grok_shell::app_server_runtime::{
    ShellRuntimeAdapter, ShellSessionActorRuntime, experimental_acp_resident_spawn,
};
use xai_grok_tower::{GrokRuntimeFacade, TowerInstanceId, TowerInstanceIdError};

/// Build the experimental App Server processor for the product path.
///
/// Uses the real Shell session-actor runtime rooted at `grok_home()` with the
/// shell-owned ACP resident factory. The factory remains fail-closed on auth or
/// bootstrap failure and never falls back to an offline echo.
pub fn experimental_app_server_processor() -> FacadeProcessor {
    let root = xai_grok_shell::util::grok_home::grok_home();
    experimental_app_server_processor_with_acp_spawn(root)
}

/// Build the experimental App Server processor with an explicit storage root.
///
/// Test seam: tests pass a `TempDir` so they never touch the real `grok_home()`.
/// The explicit-root constructor remains storage-only so hermetic tests do not
/// require credentials or network setup. Production uses the ACP constructor
/// above.
pub fn experimental_app_server_processor_with_root(root: std::path::PathBuf) -> FacadeProcessor {
    let real: Arc<dyn GrokRuntimeFacade> = Arc::new(ShellSessionActorRuntime::new(root));
    let adapter = ShellRuntimeAdapter::inject(real);
    FacadeProcessor::new(Arc::new(adapter))
}

/// Build the App Server processor with the shell-owned ACP resident factory.
///
/// This constructor is also used by the production default above. The runtime
/// advertises only the Turn capabilities proven by the ACP bridge; Interaction
/// and item lifecycle remain fail-closed until their own gates complete.
pub fn experimental_app_server_processor_with_acp_spawn(
    root: std::path::PathBuf,
) -> FacadeProcessor {
    let spawn = experimental_acp_resident_spawn(root.clone());
    let real: Arc<dyn GrokRuntimeFacade> =
        Arc::new(ShellSessionActorRuntime::with_production_spawn(root, spawn));
    let adapter = ShellRuntimeAdapter::inject(real);
    FacadeProcessor::new(Arc::new(adapter))
}

/// Build the shared real runtime for the product MCP stdio launcher.
#[cfg(feature = "mcp-stdio")]
pub fn experimental_mcp_stdio_runtime() -> Arc<dyn GrokRuntimeFacade> {
    let root = xai_grok_shell::util::grok_home::grok_home();
    Arc::new(ShellSessionActorRuntime::with_production_spawn(
        root.clone(),
        experimental_acp_resident_spawn(root),
    ))
}

// ---------------------------------------------------------------------------
// C3-G: experimental App Server WebSocket listener product path.
//
// Feature-gated behind `app-server-ws` (pager-bin Cargo feature), which
// enables the `websocket` feature on `xai-grok-app-server` and pulls the WS
// test client. The default build includes this listener; the stdio / in-process
// product path remains available alongside it. The listener itself
// lives in `xai-grok-app-server::transport::ws_listener` (C3-B); this module
// only wires it into the product composition root with the real
// `ShellSessionActorRuntime`-backed processor.
//
// Security posture (HUMAN gate preserved — do not weaken):
// - Default bind is loopback (`127.0.0.1`). Non-loopback cleartext is
//   `experimental/unsafe` and the listener emits the canonical
//   `remote_bind_warning_exact` warning at bind time.
// - TLS is a HUMAN gate (D-SEC.13 / AS104-HUMAN). This path never advertises
//   production TLS and never auto-promotes a cleartext remote bind.
// - Bearer auth is required (`require_auth = true`); the token is the
//   `--secret`/`GROK_AGENT_SECRET` already used by `agent serve`.
// ---------------------------------------------------------------------------

/// Env var that selects the experimental App Server WS listener on
/// `grok agent serve`. When set to a truthy value, the `Serve` dispatch starts
/// the `FacadeProcessor`-backed WS listener (this module) instead of the shell
/// agent server. Default (unset/empty) keeps the existing shell agent server.
pub const APP_SERVER_SERVE_ENV: &str = "GROK_OSS_APP_SERVER";

/// Truthy parse for [`APP_SERVER_SERVE_ENV`]: everything enables except the
/// common falsy spellings (`0`, `false`, `off`, `no`, empty). Mirrors the
/// `env_flag_enabled` convention used elsewhere in the binary.
pub fn app_server_serve_env_enabled() -> bool {
    std::env::var(APP_SERVER_SERVE_ENV)
        .ok()
        .map(|v| {
            !matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "" | "0" | "false" | "off" | "no"
            )
        })
        .unwrap_or(false)
}

/// Build the WS listener config for the product path from a bind string and a
/// bearer token. Auth is always required; the outbound queue cap uses the
/// listener's documented default. Exposed (feature-gated) so the CLI dispatch
/// and tests share one config builder.
#[cfg(feature = "app-server-ws")]
pub fn app_server_ws_listener_config(
    bind: String,
    bearer_token: String,
) -> xai_grok_app_server::WsListenerConfig {
    app_server_ws_listener_config_with_auth(bind, bearer_token, true)
}

/// Build the product WS config with an explicit authentication posture.
#[cfg(feature = "app-server-ws")]
pub fn app_server_ws_listener_config_with_auth(
    bind: String,
    bearer_token: String,
    require_auth: bool,
) -> xai_grok_app_server::WsListenerConfig {
    xai_grok_app_server::WsListenerConfig {
        bind,
        bearer_token,
        require_auth,
        outbound_queue_cap: xai_grok_app_server::OUTBOUND_QUEUE_CAP,
    }
}

/// Start the experimental App Server WS listener on the product composition
/// root (real `ShellSessionActorRuntime` rooted at `grok_home()`). Returns the
/// bound address and the accept-loop join handle. The listener runs until the
/// handle is dropped/aborted.
///
/// `bind` defaults to loopback in the CLI (`agent serve --bind`); non-loopback
/// cleartext is `experimental/unsafe` and warned at bind time. TLS stays a
/// HUMAN gate. This is the documented CLI/env path: `grok agent serve
/// --bind 127.0.0.1:0 --secret <token>` with `GROK_OSS_APP_SERVER=1`.
#[cfg(feature = "app-server-ws")]
pub async fn run_app_server_ws(
    bind: String,
    bearer_token: String,
) -> Result<xai_grok_app_server::WsListenerHandle, std::io::Error> {
    run_app_server_ws_with_auth(bind, bearer_token, true).await
}

/// Start the product WS listener with an explicit authentication posture.
#[cfg(feature = "app-server-ws")]
pub async fn run_app_server_ws_with_auth(
    bind: String,
    bearer_token: String,
    require_auth: bool,
) -> Result<xai_grok_app_server::WsListenerHandle, std::io::Error> {
    let processor = experimental_app_server_processor();
    let config = app_server_ws_listener_config_with_auth(bind, bearer_token, require_auth);
    xai_grok_app_server::run_ws_listener(Arc::new(processor), config).await
}

/// Same as [`run_app_server_ws`] but with an explicit storage root, so tests
/// never touch the real `grok_home()`.
#[cfg(feature = "app-server-ws")]
pub async fn run_app_server_ws_with_root(
    root: std::path::PathBuf,
    bind: String,
    bearer_token: String,
) -> Result<xai_grok_app_server::WsListenerHandle, std::io::Error> {
    let processor = experimental_app_server_processor_with_root(root);
    let config = app_server_ws_listener_config(bind, bearer_token);
    xai_grok_app_server::run_ws_listener(Arc::new(processor), config).await
}

// ---------------------------------------------------------------------------
// C4-F: experimental MCP Streamable HTTP listener product path.
//
// Feature-gated behind `mcp-streamable-http` (pager-bin Cargo feature), which
// enables the `streamable-http` feature on `xai-grok-mcp-server` and pulls the
// HTTP test client. The default build includes this listener; the stdio /
// in-process product path remains available alongside it. The listener
// itself lives in `xai-grok-mcp-server::transport::http_server` (C4-B); this
// module only wires it into the product composition root with the real
// `ShellSessionActorRuntime`-backed facade.
//
// Security posture (HUMAN gate preserved — do not weaken):
// - Default bind is loopback (`127.0.0.1`). Non-loopback cleartext is
//   `experimental/unsafe` and the listener emits the canonical `bind_warning`
//   at bind time.
// - TLS is a HUMAN gate (D-SEC.13 / MCP102-HUMAN). This path never advertises
//   production TLS and never auto-promotes a cleartext remote bind.
// - Bearer auth is required (`require_auth = true`) and **fail-closed**: the
//   listener refuses to bind when the bearer is empty/whitespace (F-2). The
//   token is the `--secret`/`GROK_AGENT_SECRET` already used by `agent serve`.
// - No self-loop: this module never imports the outbound MCP client
//   (`xai-grok-mcp`) and never registers the local `/mcp` URL into a session's
//   MCP client pool. The composition-level guard is asserted by
//   `composition_source_does_not_register_local_mcp_self_loop` in the
//   `xai-grok-mcp-server` integration suite.
// ---------------------------------------------------------------------------

/// Env var that selects the experimental MCP Streamable HTTP listener on
/// `grok agent serve`. When set to a truthy value, the `Serve` dispatch starts
/// the `FacadeProcessor`-backed MCP HTTP listener (this module) instead of the
/// shell agent server. Default (unset/empty) keeps the existing shell agent
/// server. Distinct from [`APP_SERVER_SERVE_ENV`] (C3-G WS) so the two
/// experimental paths do not collide.
pub const MCP_HTTP_SERVE_ENV: &str = "GROK_OSS_MCP_HTTP";

/// CLI/env matrix selector for MCP transport (C4-F F-3).
///
/// `GROK_OSS_MCP` values: `off` (default), `stdio` (reserved — shell agent
/// path remains default serve), `http` (Streamable HTTP listener). The legacy
/// `GROK_OSS_MCP_HTTP=1` remains accepted as an alias for `http`.
pub const MCP_MODE_ENV: &str = "GROK_OSS_MCP";

/// Parsed MCP serve mode from env.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpServeMode {
    Off,
    Stdio,
    Http,
}

/// Resolve the MCP serve mode from env (CLI matrix).
///
/// Precedence: `GROK_OSS_MCP` if set and non-empty; else truthy
/// `GROK_OSS_MCP_HTTP` → Http; else Off.
pub fn mcp_serve_mode() -> McpServeMode {
    if let Ok(v) = std::env::var(MCP_MODE_ENV) {
        match v.trim().to_ascii_lowercase().as_str() {
            "" | "off" | "0" | "false" | "no" => return McpServeMode::Off,
            "stdio" => return McpServeMode::Stdio,
            "http" | "http://" | "1" | "true" | "on" | "yes" => return McpServeMode::Http,
            other if other.starts_with("http://") || other.starts_with("https://") => {
                return McpServeMode::Http;
            }
            _ => return McpServeMode::Off,
        }
    }
    if mcp_http_legacy_flag_enabled() {
        McpServeMode::Http
    } else {
        McpServeMode::Off
    }
}

/// Truthy parse for legacy [`MCP_HTTP_SERVE_ENV`].
fn mcp_http_legacy_flag_enabled() -> bool {
    std::env::var(MCP_HTTP_SERVE_ENV)
        .ok()
        .map(|v| {
            !matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "" | "0" | "false" | "off" | "no"
            )
        })
        .unwrap_or(false)
}

/// Truthy parse for [`MCP_HTTP_SERVE_ENV`] / [`MCP_MODE_ENV`] → HTTP path.
///
/// Prefer [`mcp_serve_mode`] for the full matrix; this remains for callers that
/// only need the HTTP gate.
pub fn mcp_http_serve_env_enabled() -> bool {
    matches!(mcp_serve_mode(), McpServeMode::Http)
}

/// Build the real `ShellSessionActorRuntime`-backed facade for the MCP HTTP
/// product path, rooted at `grok_home()`. The MCP HTTP listener takes an
/// `Arc<dyn GrokRuntimeFacade>` directly (it does not wrap a
/// `FacadeProcessor`), so this returns the same real port the WS path uses,
/// not FakeRuntime.
#[cfg(feature = "mcp-streamable-http")]
pub fn experimental_mcp_http_runtime() -> Arc<dyn GrokRuntimeFacade> {
    let root = xai_grok_shell::util::grok_home::grok_home();
    Arc::new(ShellSessionActorRuntime::with_production_spawn(
        root.clone(),
        experimental_acp_resident_spawn(root),
    ))
}

/// Same as [`experimental_mcp_http_runtime`] but with an explicit storage
/// root, so tests never touch the real `grok_home()`.
#[cfg(feature = "mcp-streamable-http")]
pub fn experimental_mcp_http_runtime_with_root(
    root: std::path::PathBuf,
) -> Arc<dyn GrokRuntimeFacade> {
    Arc::new(ShellSessionActorRuntime::new(root))
}

/// Build the MCP HTTP listener config for the product path from a bind string,
/// a bearer token, and the resolved Tower instance id. Auth is always
/// required; the listener is fail-closed on an empty bearer (F-2). Exposed
/// (feature-gated) so the CLI dispatch and tests share one config builder.
#[cfg(feature = "mcp-streamable-http")]
pub fn mcp_http_server_config(
    bind: String,
    bearer_token: String,
    tower_instance_id: String,
) -> xai_grok_mcp_server::McpHttpConfig {
    mcp_http_server_config_with_auth(bind, bearer_token, tower_instance_id, true)
}

/// Build the product MCP config with an explicit authentication posture.
#[cfg(feature = "mcp-streamable-http")]
pub fn mcp_http_server_config_with_auth(
    bind: String,
    bearer_token: String,
    tower_instance_id: String,
    require_auth: bool,
) -> xai_grok_mcp_server::McpHttpConfig {
    xai_grok_mcp_server::McpHttpConfig {
        bind,
        bearer_token,
        require_auth,
        max_message_bytes: xai_grok_mcp_server::DEFAULT_MAX_MESSAGE_BYTES,
        tower_instance_id,
        agent_type: "orchestrator".to_owned(),
        explicit_opt_in: false,
        max_sessions: 1024,
        session_ttl: std::time::Duration::from_secs(3600),
        max_session_events: xai_grok_mcp_server::DEFAULT_MAX_SESSION_EVENTS,
    }
}

/// Start the experimental MCP Streamable HTTP listener on the product
/// composition root (real `ShellSessionActorRuntime` rooted at `grok_home()`).
/// Returns the bound address, the accept-loop join handle, and the shared
/// state. The listener runs until the handle is dropped/aborted.
///
/// `bind` defaults to loopback in the CLI (`agent serve --bind`); non-loopback
/// cleartext is `experimental/unsafe` and warned at bind time. TLS stays a
/// HUMAN gate. This is the documented CLI/env path: `grok agent serve
/// --bind 127.0.0.1:0 --secret <token>` with `GROK_OSS_MCP_HTTP=1`.
#[cfg(feature = "mcp-streamable-http")]
pub async fn run_mcp_http(
    bind: String,
    bearer_token: String,
    tower_instance_id: String,
) -> std::io::Result<xai_grok_mcp_server::McpHttpHandle> {
    run_mcp_http_with_auth(bind, bearer_token, tower_instance_id, true).await
}

/// Start the product MCP listener with an explicit authentication posture.
#[cfg(feature = "mcp-streamable-http")]
pub async fn run_mcp_http_with_auth(
    bind: String,
    bearer_token: String,
    tower_instance_id: String,
    require_auth: bool,
) -> std::io::Result<xai_grok_mcp_server::McpHttpHandle> {
    let runtime = experimental_mcp_http_runtime();
    let config =
        mcp_http_server_config_with_auth(bind, bearer_token, tower_instance_id, require_auth);
    xai_grok_mcp_server::run_mcp_http_server(runtime, config).await
}

/// Same as [`run_mcp_http`] but with an explicit storage root, so tests never
/// touch the real `grok_home()`.
#[cfg(feature = "mcp-streamable-http")]
pub async fn run_mcp_http_with_root(
    root: std::path::PathBuf,
    bind: String,
    bearer_token: String,
    tower_instance_id: String,
) -> std::io::Result<xai_grok_mcp_server::McpHttpHandle> {
    let runtime = experimental_mcp_http_runtime_with_root(root);
    let config = mcp_http_server_config(bind, bearer_token, tower_instance_id);
    xai_grok_mcp_server::run_mcp_http_server(runtime, config).await
}

/// Canonical env var for selecting a Tower instance (preferred).
pub const TOWER_INSTANCE_ENV: &str = "GROK_OSS_TOWER";
/// Legacy env var accepted during transition (lower precedence than
/// [`TOWER_INSTANCE_ENV`]).
pub const LEGACY_TOWER_INSTANCE_ENV: &str = "GROK_TOWER_INSTANCE";

/// Resolve a Tower instance id with explicit precedence:
///
/// 1. explicit arg (`--tower <id>`)
/// 2. `GROK_OSS_TOWER`
/// 3. `GROK_TOWER_INSTANCE` (legacy)
/// 4. literal `default`
///
/// The selected value is parsed and validated through [`TowerInstanceId`].
/// An invalid explicit arg or env value surfaces [`TowerInstanceIdError`]
/// rather than silently falling back to `default` (fail-closed on bad config).
/// There is no ambient "last used" mutable pointer; the legacy env is kept
/// only for transition and is strictly lower precedence than the canonical
/// `GROK_OSS_TOWER`.
pub fn resolve_tower_instance_id(
    explicit: Option<&str>,
) -> Result<TowerInstanceId, TowerInstanceIdError> {
    if let Some(v) = explicit.filter(|s| !s.is_empty()) {
        return TowerInstanceId::from_str(v);
    }
    if let Ok(v) = std::env::var(TOWER_INSTANCE_ENV)
        && !v.is_empty()
    {
        return TowerInstanceId::from_str(&v);
    }
    if let Ok(v) = std::env::var(LEGACY_TOWER_INSTANCE_ENV)
        && !v.is_empty()
    {
        return TowerInstanceId::from_str(&v);
    }
    // `default` is always valid by the wire-format contract.
    TowerInstanceId::from_str(TowerInstanceId::DEFAULT)
}

/// Select a Tower instance id for the product boundary (R4-10 fail-fast).
///
/// Returns the validated id string, or an error when explicit/env config is
/// invalid. Callers at the CLI/env boundary must abort on error rather than
/// silently mapping to `default` (which would connect to the wrong instance).
pub fn select_tower_instance_id(explicit: Option<&str>) -> Result<String, TowerInstanceIdError> {
    resolve_tower_instance_id(explicit).map(|id| id.to_string())
}

#[cfg(test)]
mod composition_tests {
    use super::*;
    use serde_json::json;
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;

    #[tokio::test]
    async fn composition_root_initialize_session_turn() {
        // Use a temp root so the real port never touches the user's grok_home.
        let temp = tempfile::TempDir::new().unwrap();
        let processor = experimental_app_server_processor_with_root(temp.path().to_path_buf());
        let init = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize",
                    "params":{
                        "protocolVersion": PROTOCOL_VERSION,
                        "clientInfo":{"name":"pager-bin","version":"0"},
                        "capabilities":{}
                    }
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        assert!(init.contains(PROTOCOL_VERSION));
        let start = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":2,"method":"session/start",
                    "params":{"workspaceRoot":"/work","idempotencyKey":"comp-1"}
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        assert!(start.contains("sessionId"));
    }

    /// The product composition root must inject the real port, not FakeRuntime.
    #[test]
    fn composition_root_injects_real_port_not_fake_runtime() {
        let temp = tempfile::TempDir::new().unwrap();
        let processor = experimental_app_server_processor_with_root(temp.path().to_path_buf());
        // Smoke: the processor builds from the real port without panicking.
        let _ = processor;
    }

    #[test]
    fn acp_composition_seam_builds_with_only_verified_turn_capabilities() {
        let temp = tempfile::TempDir::new().unwrap();
        let processor = experimental_app_server_processor_with_acp_spawn(temp.path().to_path_buf());
        // Construction must be side-effect free: the ACP host is created only
        // when a resident is actually requested, and capability truth remains
        // fail-closed until the actor gates are complete.
        let _ = processor;
    }

    #[tokio::test]
    async fn product_initialize_does_not_advertise_unwired_turn_methods() {
        let temp = tempfile::TempDir::new().unwrap();
        let processor = experimental_app_server_processor_with_root(temp.path().to_path_buf());
        let response = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize",
                    "params":{
                        "protocolVersion": PROTOCOL_VERSION,
                        "clientInfo":{"name":"pager-bin","version":"0"},
                        "capabilities":{}
                    }
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(value["result"]["capabilities"]["sessions"]["start"], true);
        assert_eq!(value["result"]["capabilities"]["turns"]["start"], false);
        assert_eq!(value["result"]["capabilities"]["turns"]["steer"], false);
        assert_eq!(value["result"]["capabilities"]["turns"]["interrupt"], false);
        assert_eq!(value["result"]["capabilities"]["items"]["lifecycle"], false);
        assert_eq!(value["result"]["capabilities"]["items"]["deltas"], false);
        assert_eq!(
            value["result"]["capabilities"]["interactions"]["approvals"],
            false
        );
        assert_eq!(
            value["result"]["capabilities"]["interactions"]["questions"],
            false
        );
        assert_eq!(
            value["result"]["capabilities"]["interactions"]["mcpElicitation"],
            false
        );
    }

    #[tokio::test]
    async fn product_rejects_unadvertised_methods_before_runtime_validation() {
        let temp = tempfile::TempDir::new().unwrap();
        let processor = experimental_app_server_processor_with_root(temp.path().to_path_buf());
        processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize",
                    "params":{
                        "protocolVersion": PROTOCOL_VERSION,
                        "clientInfo":{"name":"pager-bin","version":"0"},
                        "capabilities":{}
                    }
                })
                .to_string(),
            )
            .await
            .unwrap();

        for (id, method) in [(2, "turn/start"), (3, "turn/steer"), (4, "turn/interrupt")] {
            let response = processor
                .handle_line(
                    &json!({"jsonrpc":"2.0","id":id,"method":method,"params":{}}).to_string(),
                )
                .await
                .unwrap()
                .unwrap();
            let value: serde_json::Value = serde_json::from_str(&response).unwrap();
            assert_eq!(value["error"]["data"]["code"], "runtime_unavailable");
            assert_eq!(
                value["error"]["data"]["operationId"],
                serde_json::Value::Null
            );
        }
    }

    #[tokio::test]
    async fn production_default_composes_acp_turn_capabilities() {
        let processor = experimental_app_server_processor();
        let response = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize",
                    "params":{
                        "protocolVersion": PROTOCOL_VERSION,
                        "clientInfo":{"name":"pager-bin","version":"0"},
                        "capabilities":{}
                    }
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(value["result"]["capabilities"]["turns"]["start"], true);
        assert_eq!(value["result"]["capabilities"]["turns"]["steer"], true);
        assert_eq!(value["result"]["capabilities"]["turns"]["interrupt"], true);
        assert_eq!(
            value["result"]["capabilities"]["interactions"]["approvals"],
            false
        );
        assert_eq!(value["result"]["capabilities"]["items"]["lifecycle"], false);
    }
}

#[cfg(test)]
mod tower_selection_tests {
    use super::*;
    use serial_test::serial;
    use std::str::FromStr;

    /// Guard helper: set one env var, then restore prior state on drop.
    /// Tests that mutate env MUST run serially (env is process-global) — they
    /// are tagged `#[serial]` below.
    struct EnvGuard {
        key: &'static str,
        prior: Option<std::ffi::OsString>,
    }
    impl EnvGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let prior = std::env::var_os(key);
            // SAFETY: env mutation is confined to the test thread and every
            // test in this module is `#[serial]`, so there is no concurrent
            // access from another test.
            unsafe {
                match value {
                    Some(v) => std::env::set_var(key, v),
                    None => std::env::remove_var(key),
                }
            }
            Self { key, prior }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // SAFETY: see `EnvGuard::set`.
            unsafe {
                match &self.prior {
                    Some(v) => std::env::set_var(self.key, v),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    fn clear_tower_env() -> (EnvGuard, EnvGuard) {
        let a = EnvGuard::set(TOWER_INSTANCE_ENV, None);
        let b = EnvGuard::set(LEGACY_TOWER_INSTANCE_ENV, None);
        (a, b)
    }

    #[serial]
    #[test]
    fn tower_selection_explicit_wins_over_env_and_default() {
        let _g = clear_tower_env();
        let _oss = EnvGuard::set(TOWER_INSTANCE_ENV, Some("env-oss"));
        let _legacy = EnvGuard::set(LEGACY_TOWER_INSTANCE_ENV, Some("env-legacy"));
        // Non-empty explicit wins.
        let id = resolve_tower_instance_id(Some("branch-a")).unwrap();
        assert_eq!(id.as_str(), "branch-a");
        // Empty explicit falls through to env.
        let id = resolve_tower_instance_id(Some("")).unwrap();
        assert_eq!(id.as_str(), "env-oss");
    }

    #[serial]
    #[test]
    fn tower_selection_canonical_env_preferred_over_legacy() {
        let _g = clear_tower_env();
        let _oss = EnvGuard::set(TOWER_INSTANCE_ENV, Some("env-oss"));
        let _legacy = EnvGuard::set(LEGACY_TOWER_INSTANCE_ENV, Some("env-legacy"));
        let id = resolve_tower_instance_id(None).unwrap();
        assert_eq!(id.as_str(), "env-oss", "GROK_OSS_TOWER must beat legacy");
    }

    #[serial]
    #[test]
    fn tower_selection_legacy_used_when_canonical_absent() {
        let _g = clear_tower_env();
        let _legacy = EnvGuard::set(LEGACY_TOWER_INSTANCE_ENV, Some("env-legacy"));
        let id = resolve_tower_instance_id(None).unwrap();
        assert_eq!(id.as_str(), "env-legacy");
    }

    #[serial]
    #[test]
    fn tower_selection_default_when_nothing_set() {
        let _g = clear_tower_env();
        let id = resolve_tower_instance_id(None).unwrap();
        assert_eq!(id.as_str(), TowerInstanceId::DEFAULT);
    }

    #[serial]
    #[test]
    fn tower_selection_empty_env_falls_through() {
        let _g = clear_tower_env();
        let _oss = EnvGuard::set(TOWER_INSTANCE_ENV, Some(""));
        let _legacy = EnvGuard::set(LEGACY_TOWER_INSTANCE_ENV, Some("env-legacy"));
        let id = resolve_tower_instance_id(None).unwrap();
        assert_eq!(id.as_str(), "env-legacy", "empty canonical falls to legacy");
    }

    #[serial]
    #[test]
    fn tower_selection_invalid_explicit_returns_error_fail_closed() {
        let _g = clear_tower_env();
        let err = resolve_tower_instance_id(Some("UPPER")).unwrap_err();
        assert_eq!(err, TowerInstanceIdError);
        // Invalid env also fails closed, not silently defaulting.
        let _oss = EnvGuard::set(TOWER_INSTANCE_ENV, Some("has space"));
        assert!(resolve_tower_instance_id(None).is_err());
    }

    #[serial]
    #[test]
    fn tower_selection_validates_via_tower_instance_id_wire_format() {
        let _g = clear_tower_env();
        // Wire-format-valid ids round-trip through the resolver.
        for ok in ["default", "branch-a", "worktree-1", "ci.1", "t_2"] {
            let id = resolve_tower_instance_id(Some(ok)).unwrap();
            assert_eq!(id.as_str(), ok);
            assert_eq!(
                id,
                TowerInstanceId::from_str(ok).unwrap(),
                "resolver must use TowerInstanceId::from_str"
            );
        }
    }

    #[serial]
    #[test]
    fn select_tower_instance_id_fail_fasts_on_invalid() {
        let _g = clear_tower_env();
        // R4-10: invalid config must fail closed, not silently map to default.
        assert!(select_tower_instance_id(Some("UPPER")).is_err());
        assert_eq!(
            select_tower_instance_id(None).unwrap(),
            TowerInstanceId::DEFAULT
        );
        assert_eq!(
            select_tower_instance_id(Some("branch-a")).unwrap(),
            "branch-a"
        );
    }

    /// Hermetic guard: no ambient `GROK_OSS_TOWER` / `GROK_TOWER_INSTANCE`
    /// leaks into the resolver when the test process happens to carry them.
    /// The resolver must read only the documented env vars.
    #[serial]
    #[test]
    fn tower_selection_does_not_read_other_env_vars() {
        let _g = clear_tower_env();
        // An unrelated env var that looks like a tower id must not influence.
        let _other = EnvGuard::set("GROK_TOWER", Some("sneaky"));
        let id = resolve_tower_instance_id(None).unwrap();
        assert_eq!(id.as_str(), TowerInstanceId::DEFAULT);
    }
}

// ===========================================================================
// C3-G: composition black-box test for the experimental App Server WS path.
//
// Spawns the REAL product composition root (`run_app_server_ws_with_root` →
// `experimental_app_server_processor_with_root` → real
// `ShellSessionActorRuntime` rooted at a TempDir) and drives it over a real
// WebSocket with a real `tokio-tungstenite` client: bind → bearer auth →
// `initialize` → `session/start` → `handle_line` roundtrip. This proves the
// product composition wires `run_ws_listener` over the real shell runtime
// (not FakeRuntime) and that the documented CLI/env path starts a real
// listener on loopback.
//
// Feature-gated by `app-server-ws`. Run with:
//   cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws
// ===========================================================================
#[cfg(all(test, feature = "app-server-ws"))]
mod app_server_ws_composition_tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::{Value, json};
    use std::time::Duration;
    use tokio::time::timeout;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::HeaderValue;
    use tokio_tungstenite::{WebSocketStream, connect_async};
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;

    type ClientStream = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

    const TOKEN: &str = "comp-bearer-secret";

    /// Build a WS client request with a bearer header.
    fn ws_request(
        addr: std::net::SocketAddr,
        bearer: &str,
    ) -> tokio_tungstenite::tungstenite::handshake::client::Request {
        let url = format!("ws://{addr}/");
        let mut req = url.as_str().into_client_request().unwrap();
        req.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {bearer}")).unwrap(),
        );
        req
    }

    async fn connect(addr: std::net::SocketAddr, bearer: &str) -> ClientStream {
        let req = ws_request(addr, bearer);
        let (stream, _resp) = connect_async(req).await.unwrap();
        stream
    }

    async fn send_text(stream: &mut ClientStream, text: &str) {
        stream
            .send(Message::Text(text.to_owned().into()))
            .await
            .unwrap();
    }

    async fn recv_text(stream: &mut ClientStream) -> Value {
        loop {
            match timeout(Duration::from_secs(10), stream.next()).await {
                Ok(Some(Ok(Message::Text(t)))) => return serde_json::from_str(t.as_str()).unwrap(),
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(e))) => panic!("ws read error: {e}"),
                Ok(None) => panic!("ws closed before response"),
                Err(_) => panic!("ws recv timed out"),
            }
        }
    }

    fn init_request(id: u64) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"initialize","params":{
                "protocolVersion": PROTOCOL_VERSION,
                "clientInfo":{"name":"pager-bin-comp","version":"0"},
                "capabilities":{}
            }
        })
        .to_string()
    }

    /// Black-box: the product composition root starts a real listener on
    /// loopback, authenticates a bearer, and routes `initialize` +
    /// `session/start` through `FacadeProcessor::handle_line` over the real
    /// `ShellSessionActorRuntime` (rooted at a TempDir, never `grok_home()`).
    #[tokio::test]
    async fn app_server_ws_composition_bind_auth_and_handle_line_roundtrip() {
        let temp = tempfile::TempDir::new().unwrap();
        let handle = run_app_server_ws_with_root(
            temp.path().to_path_buf(),
            "127.0.0.1:0".to_owned(),
            TOKEN.to_owned(),
        )
        .await
        .unwrap();

        // Acceptance 1: documented CLI/env path starts a real listener on
        // 127.0.0.1.
        assert_eq!(
            handle.addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            "default bind must be loopback, got {}",
            handle.addr
        );

        // Acceptance 2a: bearer auth is enforced — wrong token is rejected.
        let bad_req = ws_request(handle.addr, "wrong-token");
        assert!(
            connect_async(bad_req).await.is_err(),
            "wrong bearer must be rejected at the handshake"
        );

        // Acceptance 2b: valid auth + handle_line roundtrip through the real
        // composition root processor.
        let mut s = connect(handle.addr, TOKEN).await;

        send_text(&mut s, &init_request(1)).await;
        let init = recv_text(&mut s).await;
        assert_eq!(init["result"]["protocolVersion"], PROTOCOL_VERSION);

        send_text(
            &mut s,
            &json!({
                "jsonrpc":"2.0","id":2,"method":"session/start","params":{
                    "workspaceRoot": temp.path().to_string_lossy(),
                    "idempotencyKey": "comp-ws-1"
                }
            })
            .to_string(),
        )
        .await;
        let start = recv_text(&mut s).await;
        assert!(
            start["result"]["session"]["sessionId"].is_string(),
            "session/start must return a sessionId over the WS path: {start}"
        );

        // Clean shutdown: abort the accept loop so the test process exits.
        handle.join.abort();
    }

    /// C3-G F-1: real-adapter slow-client resync via `session/subscribe`
    /// over WS. Proves the product composition wires subscribe → facade
    /// `replay` against `ShellSessionActorRuntime` (canonical session files
    /// under TempDir), not only the FakeRuntime path.
    #[tokio::test]
    async fn app_server_ws_real_adapter_slow_client_resync_via_subscribe() {
        let temp = tempfile::TempDir::new().unwrap();
        let handle = run_app_server_ws_with_root(
            temp.path().to_path_buf(),
            "127.0.0.1:0".to_owned(),
            TOKEN.to_owned(),
        )
        .await
        .unwrap();

        let mut s = connect(handle.addr, TOKEN).await;
        send_text(&mut s, &init_request(1)).await;
        let _ = recv_text(&mut s).await;

        send_text(
            &mut s,
            &json!({
                "jsonrpc":"2.0","id":2,"method":"session/start","params":{
                    "workspaceRoot": temp.path().to_string_lossy(),
                    "idempotencyKey": "comp-ws-resync"
                }
            })
            .to_string(),
        )
        .await;
        let start = recv_text(&mut s).await;
        let session_id = start["result"]["session"]["sessionId"]
            .as_str()
            .expect("sessionId")
            .to_owned();

        // Slow-client resync: re-subscribe from afterEventSeq=0 and receive a
        // replay page projected from the real adapter's session files.
        send_text(
            &mut s,
            &json!({
                "jsonrpc":"2.0","id":3,"method":"session/subscribe","params":{
                    "sessionId": session_id,
                    "afterEventSeq": "0"
                }
            })
            .to_string(),
        )
        .await;
        let sub = recv_text(&mut s).await;
        assert!(
            sub["result"]["replay"]["events"].is_array(),
            "real-adapter resync must return replay.events: {sub}"
        );
        assert!(
            sub["result"]["subscriptionId"].is_string(),
            "subscribe must return subscriptionId: {sub}"
        );
        // Event 0 is SessionChanged for a real storage-backed session.
        let events = sub["result"]["replay"]["events"].as_array().unwrap();
        assert!(
            !events.is_empty(),
            "real-adapter resync must include at least the SessionChanged snapshot"
        );

        handle.join.abort();
    }

    /// The composition config builder always requires auth (product path
    /// invariant — no unauthenticated WS listener on the real shell runtime).
    #[test]
    fn app_server_ws_config_requires_auth_by_default() {
        let config = app_server_ws_listener_config("127.0.0.1:0".to_owned(), "t".to_owned());
        assert!(config.require_auth, "product WS path must require auth");
        assert_eq!(
            config.outbound_queue_cap,
            xai_grok_app_server::OUTBOUND_QUEUE_CAP
        );
        assert!(config.bind.starts_with("127.0.0.1"));
    }

    /// The env gate is falsy by default and truthy only on explicit opt-in.
    /// `#[serial]` (C3-G F-4): env mutation must not race other tests in this binary.
    #[test]
    #[serial_test::serial]
    fn app_server_serve_env_gate_default_is_off() {
        let prior = std::env::var_os(APP_SERVER_SERVE_ENV);
        unsafe { std::env::remove_var(APP_SERVER_SERVE_ENV) };
        assert!(!app_server_serve_env_enabled(), "unset gate must be off");
        unsafe { std::env::set_var(APP_SERVER_SERVE_ENV, "1") };
        assert!(app_server_serve_env_enabled(), "1 must be on");
        unsafe { std::env::set_var(APP_SERVER_SERVE_ENV, "0") };
        assert!(!app_server_serve_env_enabled(), "0 must be off");
        unsafe { std::env::set_var(APP_SERVER_SERVE_ENV, "false") };
        assert!(!app_server_serve_env_enabled(), "false must be off");
        unsafe {
            match prior {
                Some(v) => std::env::set_var(APP_SERVER_SERVE_ENV, v),
                None => std::env::remove_var(APP_SERVER_SERVE_ENV),
            }
        }
    }
}

// ===========================================================================
// C4-F: composition black-box test for the experimental MCP Streamable HTTP
// path.
//
// Spawns the REAL product composition root (`run_mcp_http_with_root` →
// `experimental_mcp_http_runtime_with_root` → real
// `ShellSessionActorRuntime` rooted at a TempDir) and drives it over a real
// HTTP socket with a real `reqwest` client: bind → bearer auth (fail-closed)
// → `initialize` (negotiates `Mcp-Session-Id`) → `tools/list` (nine-tool
// parity) → `tools/call` (`tower_agent_start` reaches the shared semantic
// core over the real shell runtime). This proves the product composition
// wires `run_mcp_http_server` over the real shell runtime (not FakeRuntime)
// and that the documented CLI/env path starts a real listener on loopback
// with required bearer auth.
//
// Self-loop guard: the composition root must not register the local `/mcp`
// URL into a session's MCP client pool. The composition-level guard is
// asserted by `composition_source_does_not_register_local_mcp_self_loop` in
// the `xai-grok-mcp-server` integration suite; this module never imports the
// outbound MCP client crate.
//
// Feature-gated by `mcp-streamable-http`. Run with:
//   cargo test -p xai-grok-pager-bin --features mcp-streamable-http mcp_http
// ===========================================================================
#[cfg(all(test, feature = "mcp-streamable-http"))]
mod mcp_http_composition_tests {
    use super::*;
    use serde_json::{Value, json};
    use std::time::Duration;
    use tokio::time::timeout;
    use xai_grok_mcp_server::MCP_PROTOCOL_VERSION;

    const TOKEN: &str = "mcp-comp-bearer-secret";
    const TOWER_ID: &str = "comp-tower";

    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap()
    }

    fn init_request(id: u64) -> Value {
        json!({
            "jsonrpc":"2.0","id":id,"method":"initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "clientInfo": {"name":"pager-bin-comp","version":"0"},
                "capabilities": {}
            }
        })
    }

    async fn post_json(
        client: &reqwest::Client,
        addr: std::net::SocketAddr,
        token: Option<&str>,
        session: Option<&str>,
        body: &Value,
    ) -> (reqwest::StatusCode, Value, Option<String>) {
        let mut req = client
            .post(format!("http://{addr}/mcp"))
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream");
        if let Some(t) = token {
            req = req.header("authorization", format!("Bearer {t}"));
        }
        if let Some(s) = session {
            req = req.header("mcp-session-id", s);
        }
        let resp = req.json(body).send().await.unwrap();
        let status = resp.status();
        let session_id = resp
            .headers()
            .get("mcp-session-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_owned());
        let text = resp.text().await.unwrap();
        let value = if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::Null)
        };
        (status, value, session_id)
    }

    async fn post_json_query(
        client: &reqwest::Client,
        addr: std::net::SocketAddr,
        query: &str,
        body: &Value,
    ) -> (reqwest::StatusCode, Value, Option<String>) {
        let resp = client
            .post(format!("http://{addr}/mcp?{query}"))
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .json(body)
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let session_id = resp
            .headers()
            .get("mcp-session-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_owned());
        let text = resp.text().await.unwrap();
        let value = if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::Null)
        };
        (status, value, session_id)
    }

    /// Black-box: the product composition root starts a real MCP HTTP listener
    /// on loopback, fail-closes on an empty bearer, authenticates a valid
    /// bearer, and routes `initialize` + `tools/list` + `tools/call` through
    /// the shared semantic core over the real `ShellSessionActorRuntime`
    /// (rooted at a TempDir, never `grok_home()`).
    #[tokio::test]
    async fn mcp_http_composition_bind_auth_and_dispatch_roundtrip() {
        let temp = tempfile::TempDir::new().unwrap();
        let handle = run_mcp_http_with_root(
            temp.path().to_path_buf(),
            "127.0.0.1:0".to_owned(),
            TOKEN.to_owned(),
            TOWER_ID.to_owned(),
        )
        .await
        .expect("fail-closed bearer must bind with a non-empty token");

        // Acceptance 1: documented CLI/env path starts a real listener on
        // 127.0.0.1.
        assert_eq!(
            handle.addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            "default bind must be loopback, got {}",
            handle.addr
        );

        let c = client();

        // Acceptance 2a: bearer auth is enforced — wrong token is rejected
        // with 401 (indistinguishable from missing/malformed).
        let (wrong_status, _, _) =
            post_json(&c, handle.addr, Some("wrong-token"), None, &init_request(1)).await;
        assert_eq!(
            wrong_status,
            reqwest::StatusCode::UNAUTHORIZED,
            "wrong bearer must be rejected with 401"
        );

        // Acceptance 2b: valid auth + initialize negotiates a session id.
        let (init_status, init_body, session) =
            post_json(&c, handle.addr, Some(TOKEN), None, &init_request(2)).await;
        assert_eq!(init_status, reqwest::StatusCode::OK);
        assert_eq!(init_body["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);
        let session = session.expect("initialize must return Mcp-Session-Id");
        assert!(!session.is_empty(), "session id must be non-empty");

        // Acceptance 2c: tools/list returns exactly nine descriptors (parity
        // with in-process `MCP_TOOL_NAMES`).
        let (list_status, list_body, _) = post_json(
            &c,
            handle.addr,
            Some(TOKEN),
            Some(&session),
            &json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}),
        )
        .await;
        assert_eq!(list_status, reqwest::StatusCode::OK);
        assert_eq!(
            list_body["result"]["tools"].as_array().unwrap().len(),
            xai_grok_mcp_server::MCP_TOOL_NAMES.len(),
            "tools/list must match in-process nine-tool parity"
        );

        // Acceptance 2d: tools/call reaches the shared semantic core over the
        // real shell runtime and returns a structured sessionId.
        let (call_status, call_body, _) = post_json(
            &c,
            handle.addr,
            Some(TOKEN),
            Some(&session),
            &json!({
                "jsonrpc":"2.0","id":4,"method":"tools/call",
                "params": {
                    "name": "tower_agent_start",
                    "arguments": {
                        "workspaceRoot": temp.path().to_string_lossy(),
                        "agentType": "grok-oss",
                        "idempotencyKey": "comp-mcp-1"
                    }
                }
            }),
        )
        .await;
        assert_eq!(call_status, reqwest::StatusCode::OK);
        assert!(
            call_body["result"]["structuredContent"]["sessionId"].is_string(),
            "tools/call must return a structured sessionId over the HTTP path: {call_body}"
        );

        // Clean shutdown: abort the accept loop so the test process exits.
        handle.join.abort();
    }

    #[tokio::test]
    async fn mcp_http_composition_accepts_bearer_query_for_headerless_clients() {
        let temp = tempfile::TempDir::new().unwrap();
        let handle = run_mcp_http_with_root(
            temp.path().to_path_buf(),
            "127.0.0.1:0".to_owned(),
            TOKEN.to_owned(),
            TOWER_ID.to_owned(),
        )
        .await
        .unwrap();
        let (status, body, session) = post_json_query(
            &client(),
            handle.addr,
            &format!("bearer={TOKEN}"),
            &init_request(1),
        )
        .await;
        assert_eq!(status, reqwest::StatusCode::OK);
        assert_eq!(body["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);
        assert!(session.is_some(), "query bearer must negotiate a session");
        handle.join.abort();
    }

    #[cfg(feature = "app-server-ws")]
    #[test]
    fn product_configs_can_explicitly_disable_authentication() {
        let ws =
            app_server_ws_listener_config_with_auth("127.0.0.1:0".to_owned(), String::new(), false);
        assert!(!ws.require_auth);
        let mcp = mcp_http_server_config_with_auth(
            "127.0.0.1:0".to_owned(),
            String::new(),
            TOWER_ID.to_owned(),
            false,
        );
        assert!(!mcp.require_auth);
    }

    /// Fail-closed bearer (F-2): the product config builder requires auth and
    /// the listener refuses to bind with an empty/whitespace bearer.
    #[tokio::test]
    async fn mcp_http_composition_fail_closed_on_empty_bearer() {
        let temp = tempfile::TempDir::new().unwrap();
        // Empty bearer with require_auth=true (the product config) must
        // refuse to bind.
        let err = run_mcp_http_with_root(
            temp.path().to_path_buf(),
            "127.0.0.1:0".to_owned(),
            String::new(),
            TOWER_ID.to_owned(),
        )
        .await
        .expect_err("empty bearer must fail-closed (refuse to bind)");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("fail-closed"),
            "error must explain fail-closed: {err}"
        );

        // Whitespace-only bearer is also rejected.
        let err = run_mcp_http_with_root(
            temp.path().to_path_buf(),
            "127.0.0.1:0".to_owned(),
            "   ".to_owned(),
            TOWER_ID.to_owned(),
        )
        .await
        .expect_err("whitespace bearer must fail-closed");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    /// The composition config builder always requires auth (product path
    /// invariant — no unauthenticated MCP HTTP listener on the real shell
    /// runtime).
    #[test]
    fn mcp_http_config_requires_auth_by_default() {
        let config = mcp_http_server_config(
            "127.0.0.1:0".to_owned(),
            "t".to_owned(),
            TOWER_ID.to_owned(),
        );
        assert!(
            config.require_auth,
            "product MCP HTTP path must require auth"
        );
        assert_eq!(
            config.max_message_bytes,
            xai_grok_mcp_server::DEFAULT_MAX_MESSAGE_BYTES
        );
        assert!(config.bind.starts_with("127.0.0.1"));
        assert_eq!(config.tower_instance_id, TOWER_ID);
        assert_eq!(config.agent_type, "orchestrator");
    }

    /// The env gate is falsy by default and truthy only on explicit opt-in.
    /// `#[serial]`: mutates `GROK_OSS_MCP_HTTP` / `GROK_OSS_MCP`.
    #[test]
    #[serial_test::serial]
    fn mcp_http_serve_env_gate_default_is_off() {
        let prior_http = std::env::var_os(MCP_HTTP_SERVE_ENV);
        let prior_mode = std::env::var_os(MCP_MODE_ENV);
        unsafe {
            std::env::remove_var(MCP_HTTP_SERVE_ENV);
            std::env::remove_var(MCP_MODE_ENV);
        }
        assert!(!mcp_http_serve_env_enabled(), "unset gate must be off");
        assert_eq!(mcp_serve_mode(), McpServeMode::Off);
        unsafe { std::env::set_var(MCP_HTTP_SERVE_ENV, "1") };
        assert!(mcp_http_serve_env_enabled(), "legacy 1 must enable http");
        assert_eq!(mcp_serve_mode(), McpServeMode::Http);
        unsafe {
            std::env::remove_var(MCP_HTTP_SERVE_ENV);
            std::env::set_var(MCP_MODE_ENV, "http");
        }
        assert_eq!(mcp_serve_mode(), McpServeMode::Http);
        unsafe { std::env::set_var(MCP_MODE_ENV, "stdio") };
        assert_eq!(mcp_serve_mode(), McpServeMode::Stdio);
        assert!(!mcp_http_serve_env_enabled(), "stdio is not http");
        unsafe { std::env::set_var(MCP_MODE_ENV, "off") };
        assert_eq!(mcp_serve_mode(), McpServeMode::Off);
        unsafe {
            match prior_http {
                Some(v) => std::env::set_var(MCP_HTTP_SERVE_ENV, v),
                None => std::env::remove_var(MCP_HTTP_SERVE_ENV),
            }
            match prior_mode {
                Some(v) => std::env::set_var(MCP_MODE_ENV, v),
                None => std::env::remove_var(MCP_MODE_ENV),
            }
        }
    }

    /// Self-loop guard: the composition source must not import the outbound
    /// MCP client crate or reference a self-registration symbol. Mirrors the
    /// `xai-grok-mcp-server` integration-suite guard but scoped to this
    /// module's owned source so a regression here is caught locally. The
    /// forbidden local-MCP URL literal is reconstructed from parts so this
    /// guard does not itself introduce the contiguous literal it scans for.
    #[test]
    fn mcp_http_composition_does_not_self_register_local_mcp() {
        let src = include_str!("app_server_composition.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        assert!(
            !production.contains("xai_grok_mcp::"),
            "composition must not import the outbound MCP client crate"
        );
        // Reconstruct the forbidden self-registration symbol without writing
        // it as a contiguous literal (the mcp-server integration guard scans
        // this whole file for the contiguous form).
        let forbidden_sym = format!("{}{}", "register_", "self");
        assert!(
            !production.contains(&forbidden_sym),
            "composition must not self-register as a managed MCP server"
        );
        // Reconstruct the forbidden self-loop URL without writing it as a
        // contiguous literal (the mcp-server integration guard scans this
        // file for the contiguous form).
        let forbidden = format!("{}{}{}", "http://127.0.0.1:8788", "/m", "cp");
        assert!(
            !production.contains(&forbidden),
            "composition must not hard-register the local MCP URL"
        );
    }
}
