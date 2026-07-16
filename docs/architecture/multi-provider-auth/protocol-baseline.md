# Codex / ChatGPT Protocol Baseline

**Status:** Frozen — Phase 0 deliverable
**Source:** `task.md` §6 (Native Codex / ChatGPT Provider)
**Reference date:** July 2026

This document freezes the observed Codex / ChatGPT OAuth and inference
protocol so that implementation phases can build wire-level contract tests
against a stable baseline. All values are observed from the referenced
Codex source snapshot and are **protocol references**, not authorizations
for production use (see D10).

---

## 1. Endpoints

| Setting                   | Observed value                          |
| ------------------------- | --------------------------------------- |
| Issuer                    | `https://auth.openai.com`               |
| Authorization endpoint     | `/oauth/authorize`                      |
| Token endpoint            | `/oauth/token`                          |
| Revocation endpoint       | `/oauth/revoke`                         |
| ChatGPT Codex base URL    | `https://chatgpt.com/backend-api/codex` |
| Responses path            | `/responses`                            |
| Models path               | `/models?client_version=<version>`      |
| Browser callback path     | `/auth/callback`                        |
| Codex callback ports      | `1455`, fallback `1457`                 |
| Device user-code endpoint | `/api/accounts/deviceauth/usercode`     |
| Device polling endpoint   | `/api/accounts/deviceauth/token`       |
| Device verification path  | `/codex/device`                         |
| Device exchange redirect  | `/deviceauth/callback`                  |
| Observed OAuth client ID  | `app_EMoamEEZ73f0CkXaXp7hrann`          |

### Full resolved URLs

| Purpose                | URL                                                         |
| ---------------------- | ----------------------------------------------------------- |
| Authorization           | `https://auth.openai.com/oauth/authorize`                  |
| Token exchange/refresh | `https://auth.openai.com/oauth/token`                       |
| Revocation             | `https://auth.openai.com/oauth/revoke`                      |
| Responses (inference)  | `https://chatgpt.com/backend-api/codex/responses`           |
| Models                 | `https://chatgpt.com/backend-api/codex/models`             |
| Device user code        | `https://auth.openai.com/api/accounts/deviceauth/usercode` |
| Device poll            | `https://auth.openai.com/api/accounts/deviceauth/token`    |
| Device verification    | `https://auth.openai.com/codex/device`                     |

---

## 2. Observed Client ID — Protocol Reference Only (D10)

The observed OAuth client ID is:

```
app_EMoamEEZ73f0CkXaXp7hrann
```

**This value is a protocol reference only.** It must not be treated as
authorization for third-party production use (D10).

Before public production release, the project must obtain one of:

1. An OpenAI-approved client ID for Grok-build.
2. Written authorization to use the existing public Codex client identity.
3. A documented supported third-party integration mechanism.

The browser redirect URI must be allow-listed for the chosen client. Codex
uses fixed ports (`1455`, `1457`) because its redirect list is registered
accordingly.

The code must support a configurable client ID:

```rust
pub struct CodexOAuthConfig {
    pub issuer: Url,
    pub client_id: String,
    pub browser_redirect_ports: Vec<u16>,
    pub browser_callback_path: String,
}
```

Defaults may be compiled in only after the registration decision is
complete.

---

## 3. Browser OAuth with PKCE

### 3.1 Authorization request query fields

The browser authorization URL must contain:

```text
response_type=code
client_id=<configured-client-id>
redirect_uri=http://localhost:<port>/auth/callback
scope=openid profile email offline_access api.connectors.read api.connectors.invoke
code_challenge=<S256 challenge>
code_challenge_method=S256
id_token_add_organizations=true
codex_cli_simplified_flow=true
state=<cryptographically-random-state>
originator=<approved-grok-originator>
```

When an account or workspace policy is configured, add:

```text
allowed_workspace_id=<comma-separated-workspace-ids>
```

### 3.2 PKCE generation (S256)

- 32 bytes or more from the operating system CSPRNG.
- Base64 URL-safe encoding without padding.
- SHA-256 code challenge.
- No persistence of the verifier.
- One verifier per flow.
- Constant-time state comparison where practical.

### 3.3 Scopes

```text
openid
profile
email
offline_access
api.connectors.read
api.connectors.invoke
```

### 3.4 Originator

The `originator` query field is configurable and must be set to an
approved Grok-build originator value. The exact value is pending provider
approval (OQ3 in `task.md`).

### 3.5 Browser callback

- Callback path: `/auth/callback`
- Callback ports: `1455` (preferred), `1457` (fallback)
- Bind only to loopback (`127.0.0.1`); never `0.0.0.0`.
- Accept only the configured callback path.
- Enforce maximum request-target length.
- Reject unexpected methods.
- Validate exact `state`.
- Reject duplicate completion.
- Shut down after success, cancellation, or timeout.
- Add `Connection: close`.
- Never include tokens in query strings.
- Redact `code`, `state`, and OAuth error descriptions from debug URLs.

### 3.6 Browser-port selection

```text
for port in configured_browser_ports:
    try bind 127.0.0.1:port
    if success:
        use it
    if address in use:
        optionally cancel an abandoned Grok login server
        retry briefly
continue
return CallbackPortUnavailable
```

Do not choose a random port unless that redirect pattern has been
registered.

---

## 4. Token Exchange

### 4.1 Authorization-code exchange (browser)

```http
POST https://auth.openai.com/oauth/token
Content-Type: application/x-www-form-urlencoded
```

Body:

```text
grant_type=authorization_code
code=<authorization-code>
redirect_uri=http://localhost:<port>/auth/callback
client_id=<configured-client-id>
code_verifier=<pkce-verifier>
```

The response must contain:

- `id_token`
- `access_token`
- `refresh_token`

### 4.2 Refresh-token exchange

```http
POST https://auth.openai.com/oauth/token
Content-Type: application/json
```

```json
{
  "client_id": "<configured-client-id>",
  "grant_type": "refresh_token",
  "refresh_token": "<refresh-token>"
}
```

The response may rotate any subset of:

```json
{
  "id_token": "...",
  "access_token": "...",
  "refresh_token": "..."
}
```

Missing fields mean "retain the previous value," not "erase it."

### 4.3 Revocation

```http
POST https://auth.openai.com/oauth/revoke
Content-Type: application/json
```

Prefer the refresh token:

```json
{
  "token": "<refresh-token>",
  "token_type_hint": "refresh_token",
  "client_id": "<configured-client-id>"
}
```

Fallback for credentials without a refresh token:

```json
{
  "token": "<access-token>",
  "token_type_hint": "access_token"
}
```

---

## 5. Device-Code Flow (NOT Standard RFC 8628)

**Important:** Codex's device flow is **not** exposed through standard RFC
8628 endpoint names. It uses provider-specific endpoints and a
non-standard exchange sequence. It must be implemented as a
provider-specific protocol (D7), not forced into generic RFC 8628
semantics.

### 5.1 Step 1 — Request user code

```http
POST https://auth.openai.com/api/accounts/deviceauth/usercode
Content-Type: application/json
```

```json
{
  "client_id": "<configured-client-id>"
}
```

Expected response:

```json
{
  "device_auth_id": "...",
  "user_code": "...",
  "interval": "5"
}
```

Accept both `user_code` and the historical `usercode` alias if required
for compatibility.

### 5.2 Step 2 — Display approval instructions

```text
Open: https://auth.openai.com/codex/device
Enter code: ABCD-EFGH
```

When the provider later offers a complete verification URL, prefer it.

### 5.3 Step 3 — Poll

```http
POST https://auth.openai.com/api/accounts/deviceauth/token
Content-Type: application/json
```

```json
{
  "device_auth_id": "...",
  "user_code": "ABCD-EFGH"
}
```

Polling rules:

- Sleep before the first poll.
- Respect the returned interval.
- Apply bounded jitter.
- Treat 403 and 404 as pending only when the response matches expected
  pending semantics.
- Stop on cancellation.
- Stop after 15 minutes unless the server supplies a shorter expiration.
- Do not log `device_auth_id`.
- Do not persist device flow state.

The polling response eventually returns:

- `authorization_code`
- `code_challenge`
- `code_verifier`

### 5.4 Step 4 — Exchange authorization code

After approval, exchange:

```http
POST https://auth.openai.com/oauth/token
Content-Type: application/x-www-form-urlencoded
```

```text
grant_type=authorization_code
code=<authorization-code>
redirect_uri=https://auth.openai.com/deviceauth/callback
client_id=<configured-client-id>
code_verifier=<returned-code-verifier>
```

The device polling result supplies the PKCE verifier used for this
exchange. The redirect URI for device flow is
`https://auth.openai.com/deviceauth/callback` (not the loopback callback).

---

## 6. Inference Endpoint

### 6.1 Base URL

```text
https://chatgpt.com/backend-api/codex
```

### 6.2 Responses URL

```text
https://chatgpt.com/backend-api/codex/responses
```

### 6.3 Required authentication headers

```http
Authorization: Bearer <access-token>
ChatGPT-Account-ID: <chatgpt-account-id>
```

When the account claims require FedRAMP routing:

```http
X-OpenAI-Fedramp: true
```

### 6.4 Protocol headers (session/protocol layer)

The following may be generated by the session/protocol layer rather than
the provider:

```text
User-Agent
OpenAI-Beta
x-codex-installation-id
x-codex-window-id
x-codex-parent-thread-id
x-codex-turn-state
x-codex-turn-metadata
x-openai-subagent
traceparent
```

Not every header is required for a minimal Responses request.

### 6.5 Header conflict policy

A user-supplied model configuration must not override:

```text
Authorization
ChatGPT-Account-ID
X-OpenAI-Fedramp
```

when using native Codex authentication.

---

## 7. Models Endpoint

```http
GET https://chatgpt.com/backend-api/codex/models?client_version=<grok-compatible-version>
Authorization: Bearer <access-token>
ChatGPT-Account-ID: <account-id>
```

The model client appends `client_version` and parses an ETag from the
response.

### Model cache

```text
~/.grok/cache/models/codex/<credential-id>.json
```

Policy:

- Fresh TTL: 5 minutes.
- Revalidate using ETag where supported.
- Separate cache per credential/account.
- Never reuse a workspace model catalog for another workspace.
- Use stale cache when offline.
- Use bundled fallback when no cache exists.
- Mark stale/bundled results in the UI.
- Do not delete a cache because of a transient 5xx.
- Invalidate after account identity changes.

---

## 8. Kill Switches

### Environment variables

```text
GROK_DISABLE_CODEX_AUTH=1
GROK_DISABLE_CODEX_BROWSER_LOGIN=1
GROK_DISABLE_CODEX_DEVICE_LOGIN=1
```

When `GROK_DISABLE_CODEX_AUTH=1` is set, the Codex provider is fully
disabled and disappears from the login UI. Existing sessions show a clear
provider-disabled state.

`GROK_DISABLE_CODEX_BROWSER_LOGIN` and
`GROK_DISABLE_CODEX_DEVICE_LOGIN` selectively disable individual login
transports while leaving the provider otherwise available.

### Compile-time feature flags

```text
native-multi-provider-auth
native-codex-auth
```

### Runtime config

```toml
[features]
multi_provider_auth = true
codex_provider = true
codex_browser_login = true
codex_device_login = true
```

---

## 9. Error Mapping (Refresh)

| Backend code                | Grok status        | User message                                      |
| --------------------------- | ------------------ | ------------------------------------------------- |
| `refresh_token_expired`     | `ReauthRequired`   | ChatGPT session expired                           |
| `refresh_token_reused`      | `ReauthRequired`   | Session token was already rotated                 |
| `refresh_token_invalidated` | `ReauthRequired`   | ChatGPT session was revoked                       |
| HTTP 401 unknown            | `ReauthRequired`   | ChatGPT rejected the saved session                |
| 429                         | `TransientFailure` | Authentication service is rate-limiting requests  |
| 5xx                         | `TransientFailure` | Authentication service is temporarily unavailable |
| Timeout                     | `TransientFailure` | Authentication refresh timed out                  |

---

## 10. Open Questions for Client Registration

These must be resolved before stable release. They are tracked as OQ1–OQ10
in `task.md` §13.1.

| ID   | Question                                                             | Required resolution                                         |
| ---- | -------------------------------------------------------------------- | ----------------------------------------------------------- |
| OQ1  | Is Grok-build authorized to use the observed Codex OAuth client ID?  | Obtain written confirmation or provision a dedicated client |
| OQ2  | Which callback URIs will be registered?                              | Freeze browser ports and path                               |
| OQ3  | What `originator` value should Grok send?                             | Provider approval                                           |
| OQ4  | Are the ChatGPT backend endpoints supported for third-party clients? | Product/legal/technical confirmation                        |
| OQ5  | Are connector scopes necessary for Grok's use case?                  | Confirm least-privilege scope set                           |
| OQ6  | How should users choose among several ChatGPT workspaces?           | Decide browser selection vs explicit policy                |
| OQ7  | Is FedRAMP use supported in Grok?                                     | Compliance review                                           |
| OQ8  | Which provider-specific request metadata is mandatory?               | Wire-level testing                                           |
| OQ9  | How should model catalog compatibility be versioned against Grok?    | Define client-version strategy                              |
| OQ10 | Is automatic import from Codex permitted and desirable?             | Separate product/security decision                           |

---

## 11. Claim Extraction

The ID token should be parsed for display and routing metadata. Observed
claims:

```text
email
https://api.openai.com/profile.email
https://api.openai.com/auth.chatgpt_plan_type
https://api.openai.com/auth.chatgpt_user_id
https://api.openai.com/auth.user_id
https://api.openai.com/auth.chatgpt_account_id
https://api.openai.com/auth.chatgpt_account_is_fedramp
```

**Security rule:** JWT payload parsing is not signature validation. Use
parsed claims for display, local identity fingerprinting, routing
headers, and expiration hints. Do not use unverified claims to authorize
local privileged operations.

### Account fingerprint

```text
AccountFingerprint::sha256(
    provider_id
    || issuer
    || chatgpt_user_id
    || chatgpt_account_id
)
```

Refresh is rejected if the fingerprint changes unexpectedly.
