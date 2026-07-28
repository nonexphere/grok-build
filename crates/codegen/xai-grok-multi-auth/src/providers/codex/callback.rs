//! Loopback HTTP callback server for browser PKCE login.
//!
//! Binds only to `127.0.0.1` (never `0.0.0.0`), accepts a single successful
//! GET on the configured callback path, and returns the full request URL
//! for state/code validation by the provider.

use std::time::Duration;

use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use url::Url;

/// Errors from the loopback callback server.
#[derive(Debug, Error)]
pub enum CallbackError {
    #[error("failed to bind loopback port(s): {0}")]
    Bind(String),

    #[error("callback wait timed out after {0:?}")]
    Timeout(Duration),

    #[error("callback I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid callback request: {0}")]
    InvalidRequest(String),
}

/// Try to bind `127.0.0.1` on each of `ports` in order.
pub async fn bind_loopback(ports: &[u16]) -> Result<(TcpListener, u16), CallbackError> {
    let mut last_err = String::from("no ports configured");
    for &port in ports {
        match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => return Ok((listener, port)),
            Err(e) => last_err = format!("port {port}: {e}"),
        }
    }
    Err(CallbackError::Bind(last_err))
}

/// Await a single OAuth callback on an already-bound loopback listener.
///
/// Accepts only GET requests whose path matches `callback_path` (e.g.
/// `/auth/callback`). Responds with a minimal success HTML page that does
/// not include tokens. Returns the absolute callback URL including query.
pub async fn await_callback(
    listener: TcpListener,
    callback_path: &str,
    timeout: Duration,
) -> Result<Url, CallbackError> {
    let accept = async {
        let (mut socket, _) = listener.accept().await?;
        let mut buf = vec![0u8; 8192];
        let n = socket.read(&mut buf).await?;
        let req = String::from_utf8_lossy(&buf[..n]);
        let first_line = req.lines().next().unwrap_or("");
        // "GET /auth/callback?code=...&state=... HTTP/1.1"
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let target = parts.next().unwrap_or("");
        if method != "GET" {
            let _ = write_response(
                &mut socket,
                405,
                "Method Not Allowed",
                "Only GET is accepted.",
            )
            .await;
            return Err(CallbackError::InvalidRequest(format!(
                "unexpected method {method}"
            )));
        }
        let path = target.split('?').next().unwrap_or(target);
        if path != callback_path {
            let _ = write_response(&mut socket, 404, "Not Found", "Unknown path.").await;
            return Err(CallbackError::InvalidRequest(format!(
                "unexpected path {path}"
            )));
        }

        // Build absolute URL for the provider validator.
        let abs = format!("http://127.0.0.1{target}");
        let url = Url::parse(&abs).map_err(|e| {
            CallbackError::InvalidRequest(format!("cannot parse callback target: {e}"))
        })?;

        let _ = write_response(
            &mut socket,
            200,
            "Login complete",
            "You can close this tab and return to the terminal.",
        )
        .await;
        Ok(url)
    };

    match tokio::time::timeout(timeout, accept).await {
        Ok(result) => result,
        Err(_) => Err(CallbackError::Timeout(timeout)),
    }
}

async fn write_response(
    socket: &mut tokio::net::TcpStream,
    status: u16,
    title: &str,
    body: &str,
) -> std::io::Result<()> {
    let html = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/><title>{title}</title>\
         <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'\"/>\
         </head><body><h1>{title}</h1><p>{body}</p></body></html>"
    );
    let resp = format!(
        "HTTP/1.1 {status} OK\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Cache-Control: no-store\r\n\
         \r\n\
         {html}",
        html.len()
    );
    socket.write_all(resp.as_bytes()).await?;
    socket.shutdown().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn loopback_accepts_callback_and_returns_url() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = tokio::spawn(async move {
            await_callback(listener, "/auth/callback", Duration::from_secs(5)).await
        });

        // Client hits the callback path.
        let mut client = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .unwrap();
        let req = format!(
            "GET /auth/callback?code=abc&state=xyz HTTP/1.1\r\n\
             Host: 127.0.0.1:{port}\r\n\
             Connection: close\r\n\r\n"
        );
        client.write_all(req.as_bytes()).await.unwrap();
        // Drain response so server finishes.
        use tokio::io::AsyncReadExt as _;
        let mut buf = vec![0u8; 1024];
        let _ = client.read(&mut buf).await;

        let url = server.await.unwrap().unwrap();
        assert_eq!(url.path(), "/auth/callback");
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs.get("code").map(String::as_str), Some("abc"));
        assert_eq!(pairs.get("state").map(String::as_str), Some("xyz"));
    }
}
