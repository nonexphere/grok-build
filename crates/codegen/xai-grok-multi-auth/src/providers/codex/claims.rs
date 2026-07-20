//! JWT claim extraction for Codex ID/access tokens (protocol-baseline.md §11).
//!
//! This is NOT signature validation — parsed claims are used for display,
//! local identity fingerprinting, routing headers, and expiration hints only.
//!
//! Real OpenAI tokens nest ChatGPT fields under the object key
//! `https://api.openai.com/auth` (not flattened dotted keys).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use xai_grok_auth::{AccountKind, ProviderAccountInfo};

/// Claims extracted from a Codex ID or access token payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexClaims {
    pub email: Option<String>,
    pub name: Option<String>,
    pub chatgpt_plan_type: Option<String>,
    pub chatgpt_user_id: Option<String>,
    pub user_id: Option<String>,
    pub chatgpt_account_id: Option<String>,
    pub chatgpt_account_is_fedramp: Option<bool>,
    /// Standard JWT `exp` claim (Unix seconds).
    pub exp: Option<u64>,
    /// Standard JWT `sub` claim.
    pub sub: Option<String>,
}

/// Base64url decode helper (no padding required).
fn decode_base64url(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    let mut s = input.trim_end_matches('=').to_string();
    while !s.len().is_multiple_of(4) {
        s.push('=');
    }
    base64::engine::general_purpose::URL_SAFE
        .decode(&s)
        .map_err(|e| format!("base64 decode: {e}"))
}

/// Parse the payload of a JWT (no signature validation) into raw JSON.
pub fn parse_jwt_payload(token: &str) -> Result<Value, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return Err("invalid JWT: expected at least 2 parts".into());
    }
    let payload_bytes = decode_base64url(parts[1])?;
    serde_json::from_slice(&payload_bytes).map_err(|e| format!("json parse: {e}"))
}

/// Parse claims from an ID token (preferred) or access token.
pub fn parse_id_token_claims(id_token: &str) -> Result<CodexClaims, String> {
    let v = parse_jwt_payload(id_token)?;
    extract_claims_from_payload(&v)
}

/// Extract Codex/ChatGPT fields from a JWT payload value.
///
/// Supports both nested (`https://api.openai.com/auth: { chatgpt_account_id }`)
/// and flat dotted keys for forward compatibility.
pub fn extract_claims_from_payload(v: &Value) -> Result<CodexClaims, String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "JWT payload is not an object".to_string())?;

    let auth = obj
        .get("https://api.openai.com/auth")
        .and_then(|x| x.as_object());

    let get_nested = |key: &str| -> Option<String> {
        auth.and_then(|a| a.get(key))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // Flat dotted form
                let flat = format!("https://api.openai.com/auth.{key}");
                obj.get(&flat)
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string())
            })
    };

    let get_nested_bool = |key: &str| -> Option<bool> {
        auth.and_then(|a| a.get(key))
            .and_then(|x| x.as_bool())
            .or_else(|| {
                let flat = format!("https://api.openai.com/auth.{key}");
                obj.get(&flat).and_then(|x| x.as_bool())
            })
    };

    // Profile email may live under https://api.openai.com/profile
    let profile_email = obj
        .get("https://api.openai.com/profile")
        .and_then(|p| p.get("email"))
        .and_then(|e| e.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            obj.get("https://api.openai.com/profile.email")
                .and_then(|e| e.as_str())
                .map(|s| s.to_string())
        });

    Ok(CodexClaims {
        email: obj
            .get("email")
            .and_then(|e| e.as_str())
            .map(|s| s.to_string())
            .or(profile_email),
        name: obj
            .get("name")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string()),
        chatgpt_plan_type: get_nested("chatgpt_plan_type"),
        chatgpt_user_id: get_nested("chatgpt_user_id"),
        user_id: get_nested("user_id"),
        chatgpt_account_id: get_nested("chatgpt_account_id"),
        chatgpt_account_is_fedramp: get_nested_bool("chatgpt_account_is_fedramp")
            .or_else(|| get_nested_bool("chatgpt_account_is_fedramp")),
        exp: obj.get("exp").and_then(|e| e.as_u64()),
        sub: obj
            .get("sub")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
    })
}

/// Convert parsed claims into a `ProviderAccountInfo`.
pub fn claims_to_account_info(claims: &CodexClaims) -> ProviderAccountInfo {
    use std::collections::BTreeMap;
    let mut metadata = BTreeMap::new();
    if let Some(ref uid) = claims.chatgpt_user_id {
        metadata.insert("chatgpt_user_id".to_string(), uid.clone());
    }
    if let Some(ref aid) = claims.chatgpt_account_id {
        metadata.insert("chatgpt_account_id".to_string(), aid.clone());
    }
    if let Some(ref plan) = claims.chatgpt_plan_type {
        metadata.insert("chatgpt_plan_type".to_string(), plan.clone());
    }
    ProviderAccountInfo {
        subject: claims.sub.clone(),
        provider_account_id: claims.chatgpt_account_id.clone(),
        email: claims.email.clone(),
        display_name: claims.name.clone(),
        workspace_id: None,
        workspace_name: None,
        plan: claims.chatgpt_plan_type.as_ref().map(|p| {
            xai_grok_auth::AccountPlan::Known {
                raw: p.clone(),
                display_name: p.clone(),
            }
        }),
        account_kind: AccountKind::Personal,
        fedramp: claims.chatgpt_account_is_fedramp.unwrap_or(false),
        metadata,
    }
}

/// Extract the `exp` claim as a `chrono::DateTime<Utc>`.
pub fn extract_expiration(claims: &CodexClaims) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::TimeZone;
    claims
        .exp
        .and_then(|s| chrono::Utc.timestamp_opt(s as i64, 0).single())
}

/// Enrich account info from id_token and/or access_token when metadata is missing.
pub fn enrich_account_from_tokens(
    account: &ProviderAccountInfo,
    id_token: Option<&str>,
    access_token: Option<&str>,
) -> ProviderAccountInfo {
    let mut out = account.clone();
    for tok in [id_token, access_token].into_iter().flatten() {
        if let Ok(claims) = parse_id_token_claims(tok) {
            let info = claims_to_account_info(&claims);
            if out.provider_account_id.is_none() {
                out.provider_account_id = info.provider_account_id.clone();
            }
            if out.email.is_none() {
                out.email = info.email.clone();
            }
            if out.display_name.is_none() {
                out.display_name = info.display_name.clone();
            }
            if out.subject.is_none() {
                out.subject = info.subject.clone();
            }
            if out.plan.is_none() {
                out.plan = info.plan.clone();
            }
            for (k, v) in info.metadata {
                out.metadata.entry(k).or_insert(v);
            }
            if info.fedramp {
                out.fedramp = true;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_openai_auth_object() {
        // Shape observed from real OpenAI id_tokens (July 2026).
        let payload = serde_json::json!({
            "sub": "auth0|abc",
            "email": "u@example.com",
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "acct-123",
                "chatgpt_plan_type": "plus",
                "chatgpt_user_id": "user-9",
                "user_id": "user-9"
            }
        });
        let claims = extract_claims_from_payload(&payload).unwrap();
        assert_eq!(claims.chatgpt_account_id.as_deref(), Some("acct-123"));
        assert_eq!(claims.chatgpt_plan_type.as_deref(), Some("plus"));
        assert_eq!(claims.email.as_deref(), Some("u@example.com"));
        let info = claims_to_account_info(&claims);
        assert_eq!(info.provider_account_id.as_deref(), Some("acct-123"));
        assert_eq!(
            info.metadata.get("chatgpt_account_id").map(String::as_str),
            Some("acct-123")
        );
    }
}
