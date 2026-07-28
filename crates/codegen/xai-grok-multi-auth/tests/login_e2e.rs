//! End-to-end login completion tests against the real AuthProvider + store path.
//!
//! These drive `CodexAuthProvider::complete_login` and `LoginCoordinator::complete_login`
//! with mockito HTTP — no live OpenAI network.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use xai_grok_auth::{
    CredentialStore, LoginInput, LoginRequest, LoginTransport, ProviderId, ProviderRegistry,
};
use xai_grok_multi_auth::login_coordinator::{LoginCoordinator, LoginUiEvent};
use xai_grok_multi_auth::providers::codex::{CodexAuthProvider, CodexOAuthConfig};
use xai_grok_multi_auth::providers::xai::XaiAuthProvider;
use xai_grok_multi_auth::store::ephemeral::EphemeralCredentialStore;

fn jwt_payload(json: &str) -> String {
    use base64::Engine;
    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"{\"alg\":\"none\"}");
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes());
    format!("{header}.{payload}.sig")
}

/// Browser: start_login → BrowserCallback → store persists access+refresh.
#[tokio::test]
async fn browser_complete_login_persists_credential() {
    let mut server = mockito::Server::new_async().await;
    let base = server.url();

    let id_token = jwt_payload(
        r#"{"sub":"user-1","email":"u@example.com","https://api.openai.com/auth":{"chatgpt_account_id":"acct-99"}}"#,
    );
    let _token_mock = server
        .mock("POST", "/oauth/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(format!(
            r#"{{"access_token":"access-tok-e2e","refresh_token":"refresh-tok-e2e","id_token":"{id_token}","expires_in":3600}}"#
        ))
        .create_async()
        .await;

    let mut config = CodexOAuthConfig::default();
    config.issuer = url::Url::parse(&base).unwrap();
    // Keep ports unused — we inject BrowserCallback directly.
    config.browser_redirect_ports = vec![18999];

    let provider = Arc::new(CodexAuthProvider::with_config(config));
    let mut registry = ProviderRegistry::new();
    registry.register(provider.clone()).unwrap();
    let store = Arc::new(EphemeralCredentialStore::new());
    let coordinator = LoginCoordinator::new(store.clone(), Arc::new(registry));

    let provider_id = ProviderId::new_unchecked("codex");
    let (flow_id, events) = coordinator
        .start_login(&provider_id, LoginTransport::BrowserPkce, Some("e2e".into()))
        .await
        .unwrap();

    let auth_url = events.iter().find_map(|e| match e {
        LoginUiEvent::OpenBrowser { url } => Some(url.clone()),
        _ => None,
    });
    assert!(auth_url.is_some(), "must emit OpenBrowser");

    // Reconstruct a valid callback using the state from the authorize URL.
    let state = auth_url
        .as_ref()
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.into_owned())
        .expect("state in auth url");
    let callback = url::Url::parse(&format!(
        "http://127.0.0.1:18999/auth/callback?code=authcode-e2e&state={state}"
    ))
    .unwrap();

    let events = coordinator
        .complete_login(
            &provider_id,
            flow_id,
            LoginInput::BrowserCallback { url: callback },
        )
        .await
        .unwrap();

    let key = events.iter().find_map(|e| match e {
        LoginUiEvent::Completed { key } => Some(key.clone()),
        _ => None,
    });
    let key = key.expect("Completed event with key");

    let loaded = store.load(&key).await.unwrap().expect("stored credential");
    assert_eq!(
        loaded.secret.access_token.expose(),
        "access-tok-e2e",
        "must persist the access token from token exchange"
    );
    assert_eq!(
        loaded
            .secret
            .refresh_token
            .as_ref()
            .map(|s| s.expose().to_string())
            .as_deref(),
        Some("refresh-tok-e2e")
    );
    assert_eq!(loaded.metadata.key.provider.as_str(), "codex");
}

/// Device: first poll Pending keeps flow; second poll Completes and persists.
#[tokio::test]
async fn device_multi_poll_pending_then_complete_persists() {
    let mut server = mockito::Server::new_async().await;
    let base = server.url();

    let _usercode = server
        .mock("POST", "/api/accounts/deviceauth/usercode")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"device_auth_id":"dev-auth-1","user_code":"ABCD-EFGH","interval":"1"}"#,
        )
        .create_async()
        .await;

    // First poll: pending (403)
    let _poll_pending = server
        .mock("POST", "/api/accounts/deviceauth/token")
        .with_status(403)
        .with_body(r#"{"error":"pending"}"#)
        .expect(1)
        .create_async()
        .await;

    // Second poll: returns authorization_code + verifier
    let _poll_done = server
        .mock("POST", "/api/accounts/deviceauth/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"authorization_code":"dev-code","code_challenge":"ch","code_verifier":"ver"}"#,
        )
        .expect(1)
        .create_async()
        .await;

    let id_token = jwt_payload(r#"{"sub":"u2","email":"dev@example.com"}"#);
    let _token = server
        .mock("POST", "/oauth/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(format!(
            r#"{{"access_token":"dev-access","refresh_token":"dev-refresh","id_token":"{id_token}","expires_in":3600}}"#
        ))
        .create_async()
        .await;

    let mut config = CodexOAuthConfig::default();
    config.issuer = url::Url::parse(&base).unwrap();

    let provider = Arc::new(CodexAuthProvider::with_config(config));
    let mut registry = ProviderRegistry::new();
    registry.register(provider.clone()).unwrap();
    registry
        .register(Arc::new(XaiAuthProvider::new()))
        .ok(); // optional

    let store = Arc::new(EphemeralCredentialStore::new());
    let coordinator = LoginCoordinator::new(store.clone(), Arc::new(registry));
    let provider_id = ProviderId::new_unchecked("codex");

    let (flow_id, start_events) = coordinator
        .start_login(&provider_id, LoginTransport::DeviceCode, None)
        .await
        .unwrap();
    assert!(
        start_events
            .iter()
            .any(|e| matches!(e, LoginUiEvent::ShowDeviceCode { .. })),
        "device start must show user code"
    );

    // Poll 1: pending — flow must remain usable.
    let e1 = coordinator
        .complete_login(&provider_id, flow_id, LoginInput::Poll)
        .await
        .unwrap();
    assert!(
        e1.iter()
            .any(|e| matches!(e, LoginUiEvent::WaitingForApproval)),
        "first poll should be pending: {e1:?}"
    );

    // Small delay so mockito can distinguish sequential expectations if needed.
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Poll 2: complete + persist.
    let e2 = coordinator
        .complete_login(&provider_id, flow_id, LoginInput::Poll)
        .await
        .unwrap();
    let key = e2.iter().find_map(|e| match e {
        LoginUiEvent::Completed { key } => Some(key.clone()),
        _ => None,
    });
    let key = key.expect("second poll must complete");

    let loaded = store.load(&key).await.unwrap().expect("credential stored");
    assert_eq!(loaded.secret.access_token.expose(), "dev-access");
    assert_eq!(
        loaded
            .secret
            .refresh_token
            .as_ref()
            .map(|s| s.expose().to_string())
            .as_deref(),
        Some("dev-refresh")
    );

    // A third poll must fail (flow removed after success).
    let err = coordinator
        .complete_login(&provider_id, flow_id, LoginInput::Poll)
        .await;
    assert!(err.is_err(), "flow should be gone after success");
}

/// Loopback server accepts a real TCP callback (unit path of shipped server).
#[tokio::test]
async fn loopback_callback_driver_accepts_get() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use xai_grok_multi_auth::providers::codex::callback::{await_callback, bind_loopback};

    // Use ephemeral: bind 0 via DEFAULT then re-bind — helper takes port list.
    // Bind a free port ourselves by passing a high unused port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    // Re-wrap through public API with that port.
    drop(listener);
    let (listener, bound) = bind_loopback(&[port]).await.unwrap();
    assert_eq!(bound, port);

    let server = tokio::spawn(async move {
        await_callback(
            listener,
            "/auth/callback",
            Duration::from_secs(5),
        )
        .await
    });

    let mut client = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    let req = format!(
        "GET /auth/callback?code=from-loopback&state=s1 HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    client.write_all(req.as_bytes()).await.unwrap();
    let mut buf = [0u8; 512];
    let _ = client.read(&mut buf).await;

    let url = server.await.unwrap().unwrap();
    assert!(url.as_str().contains("code=from-loopback"));
    assert!(url.as_str().contains("state=s1"));
}

/// AuthProvider-level multi-poll without coordinator (Pending does not drop flow).
#[tokio::test]
async fn auth_provider_device_pending_keeps_flow() {
    use xai_grok_auth::{AuthProvider, LoginTransport};

    let mut server = mockito::Server::new_async().await;
    let base = server.url();

    let _usercode = server
        .mock("POST", "/api/accounts/deviceauth/usercode")
        .with_status(200)
        .with_body(r#"{"device_auth_id":"d1","user_code":"WXYZ-1234","interval":"1"}"#)
        .create_async()
        .await;

    let _poll_pending = server
        .mock("POST", "/api/accounts/deviceauth/token")
        .with_status(404)
        .expect_at_least(2)
        .create_async()
        .await;

    let mut config = CodexOAuthConfig::default();
    config.issuer = url::Url::parse(&base).unwrap();
    let provider = CodexAuthProvider::with_config(config);

    let start = provider
        .start_login(LoginRequest {
            transport: LoginTransport::DeviceCode,
            requested_alias: None,
            force_reauthentication: false,
            open_browser: false,
            account_policy: Default::default(),
            client_surface: xai_grok_auth::ClientSurface::Cli,
        })
        .await
        .unwrap();

    let flow_id = match start {
        xai_grok_auth::LoginStart::Device { flow_id, .. } => flow_id,
        _ => panic!("expected device start"),
    };

    let c1 = provider
        .complete_login(flow_id, LoginInput::Poll)
        .await
        .unwrap();
    assert!(matches!(
        c1,
        xai_grok_auth::LoginCompletion::Pending { .. }
    ));

    // Second pending poll must still work (flow not dropped).
    let c2 = provider
        .complete_login(flow_id, LoginInput::Poll)
        .await
        .unwrap();
    assert!(matches!(
        c2,
        xai_grok_auth::LoginCompletion::Pending { .. }
    ));
}
