# Provider Skeleton

This is a structure template, not copy-paste production code. Replace every placeholder
and remove unsupported capabilities.

## Suggested module tree

```text
crates/codegen/xai-grok-multi-auth/src/providers/<provider>/
├── mod.rs           # AuthProvider implementation and lifecycle composition
├── config.rs        # validated endpoints/client/tenant/feature configuration
├── browser.rs       # optional PKCE authorization construction/state validation
├── callback.rs      # optional loopback server and bounded callback parsing
├── device.rs        # optional provider-specific device flow
├── token.rs         # exchange/refresh/revoke wire operations
├── claims.rs        # display/routing hints; never local authorization
├── models.rs        # authenticated model discovery/cache schema
├── request_auth.rs  # endpoints, reserved headers, failure classification
├── errors.rs        # provider wire errors → generic provider failures
└── fixtures/        # frozen sanitized wire fixtures
```

Only create files that match real capabilities. Small providers may combine modules.

## Composition skeleton

```rust
pub struct ExampleAuthProvider {
    id: ProviderId,
    config: ExampleConfig,
    http: reqwest::Client,
    // Bounded login-flow state only; no global current credential/account.
}

#[async_trait]
impl AuthProvider for ExampleAuthProvider {
    fn id(&self) -> &ProviderId { &self.id }

    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: self.id.clone(),
            display_name: "<display-name>".into(),
            short_name: "<provider-id>".into(),
            icon_key: Some("<icon-key>".into()),
            capabilities: /* only capabilities proven by implementation/tests */,
            default_priority: <priority>,
        }
    }

    fn validate_config(&self) -> Result<(), ProviderError> {
        // Fail closed on missing authorization-critical config.
        todo!()
    }

    async fn start_login(&self, request: LoginRequest) -> Result<LoginStart, ProviderError> {
        // Dispatch only supported transports; preserve alias/account policy in flow state.
        todo!()
    }

    async fn complete_login(
        &self,
        flow_id: LoginFlowId,
        input: LoginInput,
    ) -> Result<LoginCompletion, ProviderError> {
        // Validate state, exchange once, establish stable account identity, return record.
        todo!()
    }

    async fn refresh(
        &self,
        request: RefreshRequest<'_>,
    ) -> Result<ProviderCredentialUpdate, ProviderError> {
        // Return rotated tokens and identity evidence; TokenManager owns CAS persistence.
        todo!()
    }

    async fn logout(&self, request: LogoutRequest<'_>) -> Result<LogoutOutcome, ProviderError> {
        // Best-effort remote revocation with explicit local-delete policy.
        todo!()
    }

    async fn list_models(
        &self,
        request: ModelListRequest<'_>,
    ) -> Result<ModelCatalog, ProviderError> {
        // Credential/account-scoped discovery with ETag/cache metadata.
        todo!()
    }

    fn resolve_endpoint(
        &self,
        request: ProviderEndpointRequest<'_>,
    ) -> Result<Url, ProviderError> {
        // Resolve from request kind and validated configuration.
        todo!()
    }

    fn build_request_auth(
        &self,
        request: RequestAuthContext<'_>,
    ) -> Result<ProviderRequestAuth, ProviderError> {
        // Inject reserved auth/account/tenant headers from the bound credential.
        todo!()
    }

    fn classify_auth_failure(&self, response: &AuthFailureResponse) -> AuthFailureClass {
        // Distinguish auth from policy/rate-limit/server failures.
        todo!()
    }
}
```

## Production request flow

```text
Session/Agent ModelBinding
  -> RequestAuthResolver (actual endpoint/method/kind)
  -> TokenManager.get_valid_token(bound credential)
  -> AuthProvider.resolve_endpoint + build_request_auth
  -> Sampler sends request and records SentCredentialStamp
  -> on authentication 401 only:
       TokenManager.recover_unauthorized(stamp)
       -> retry once with newly resolved auth
```

The production code must follow this flow. A provider model may carry non-secret model
capabilities, but must not carry the OAuth access token as a static API key.

## Minimum tests to add with the skeleton

```text
provider config validation
descriptor capabilities match implemented methods
exact login/token/refresh/revoke/model wire requests
callback/device error state machine
account fingerprint and refresh identity invariant
reserved request headers and endpoint resolution
production sampler request with bound credential
expired token refresh and generation-aware 401 retry
two-account same-model isolation
mixed parent/subagent provider isolation
seeded secret redaction across all output surfaces
```

