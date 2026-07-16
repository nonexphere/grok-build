# Provider Implementation Checklist

Use this as the mandatory evidence matrix. Mark each item `PASS`, `FAIL`, `BLOCKED`, or
`N/A` with a reason and a file/test/command reference.

## A. Authorization and protocol

- [ ] Provider/client identity is authorized for the intended environment.
- [ ] Supported/unsupported endpoint status and terms are documented.
- [ ] Exact authorize/token/device/revoke/models/inference URLs are frozen.
- [ ] HTTP methods, encodings, query/form/JSON fields, and content types are frozen.
- [ ] Required scopes are least-privilege and justified.
- [ ] Callback URIs/ports/hosts/paths are registered and exact.
- [ ] Device protocol is modeled as observed; it is not forced into an incorrect generic standard.
- [ ] Provider-specific originator/user-agent/version headers are approved.
- [ ] Error schemas, pending semantics, rate limits, retry/backoff, and timeouts are frozen.
- [ ] Frozen fixtures contain no live secrets or personal data.

## B. Configuration and rollout

- [ ] Provider config validates required client/tenant/endpoint values.
- [ ] Endpoint overrides are restricted to explicit development/test modes.
- [ ] Compile-time feature exists if the provider is optional.
- [ ] Runtime provider/login transport gates are implemented and parsed.
- [ ] Environment kill switches are implemented and tested.
- [ ] Disabled providers disappear from new-login UI but existing sessions fail clearly.
- [ ] Unapproved/unsupported configuration fails closed.
- [ ] Stable/default enablement matches the rollout stage.

## C. Provider contract and registry

- [ ] Provider ID validation and canonical naming are tested.
- [ ] Descriptor display fields and priority are correct.
- [ ] Every advertised capability is fully implemented.
- [ ] Unsupported capabilities are absent, not stubbed.
- [ ] `AuthProvider` object safety compiles.
- [ ] Registration is compile-time/explicit and duplicate errors propagate.
- [ ] Disabled/invalid provider registration is tested.
- [ ] Generic consumers use provider ID/descriptors/capabilities, not provider-specific enums.

## D. Login lifecycle

- [ ] Browser PKCE uses cryptographic verifier/state and S256.
- [ ] Callback server binds only approved loopback addresses.
- [ ] State mismatch, missing code, denial, timeout, cancellation, and occupied ports are tested.
- [ ] Device user-code request, display, polling, pending, expiration, cancellation, and exchange are tested.
- [ ] Browser opening failure has a usable manual URL path.
- [ ] Login flow state is bounded, cancellable, expires, and never exposes secrets in `Debug`.
- [ ] Requested alias/account policy survives start → complete → persistence.
- [ ] Account/tenant/workspace selection is explicit and tested.
- [ ] Persistence failure revokes/cleans up best effort and returns failure.
- [ ] Re-login and duplicate-account behavior are defined.

## E. Identity, credentials, and storage

- [ ] Metadata and secret records are separated according to policy.
- [ ] Credential runtime key is provider + credential ID; alias is display/lookup only.
- [ ] Stable account fingerprint uses provider-approved identity fields.
- [ ] Refresh cannot change bound account identity.
- [ ] Keyring is preferred on supported desktops when required by policy.
- [ ] Headless fallback is explicit, secure, and reported to the user.
- [ ] File permissions/Windows ACLs are owner-only.
- [ ] Metadata+secret+generation updates are crash-consistent as one logical transaction.
- [ ] Create/delete/refresh fault points have recovery tests.
- [ ] Corrupt/empty/partial files are distinguished; corruption is never silently wiped.
- [ ] Store and per-credential locks have bounded waits and stale-lock behavior.
- [ ] Two processes updating different credentials do not block unnecessarily.
- [ ] Two processes rotating the same credential cannot reuse a refresh token.
- [ ] Keyring unavailable, disk full, fsync/rename failure, and permission denial are tested.
- [ ] Migration/legacy records are idempotent and backward compatible.

## F. Token manager and failures

- [ ] Token resolution occurs immediately before request construction.
- [ ] Early-refresh window and clock behavior are tested.
- [ ] Single-flight is per credential across tasks and processes.
- [ ] Refresh-token rotation persists before the new token is returned to callers.
- [ ] CAS/generation conflicts reload safely without duplicate refresh.
- [ ] Account fingerprint is verified before accepting refresh output.
- [ ] Transient, permanent, revoked, reused, malformed, and reauth failures are classified.
- [ ] Permanent-failure cache clears after re-login.
- [ ] Logout wakes/cancels token waiters.
- [ ] Sent requests carry a credential-generation stamp.
- [ ] Stale 401 cannot invalidate a newer token.
- [ ] 401 recovery retries at most once and never loops.

## G. Endpoints and request authentication

- [ ] Endpoint resolution is provider-owned and uses the actual request kind.
- [ ] Authorization and account/tenant/FedRAMP headers are injected correctly.
- [ ] Reserved headers cannot be overridden by config, plugins, or user input.
- [ ] Provider headers do not leak into another provider's requests.
- [ ] Request resolver is consumed by the real sampler/client composition root.
- [ ] OAuth tokens are never stored in static API-key model fields.
- [ ] Redirect/retry preserves request body semantics and idempotency constraints.
- [ ] Error bodies are redacted before logs/telemetry/user display.

## H. Models and binding

- [ ] `ModelBinding` contains provider, credential/default decision, and model ID.
- [ ] Binding is immutable for an in-flight request/turn.
- [ ] Provider default account is resolved once at the correct boundary.
- [ ] Two accounts exposing the same model ID remain independently selectable.
- [ ] Catalog/cache key includes provider and credential/account identity.
- [ ] Authenticated `/models`, ETag, TTL, offline cache, and bundled fallback are tested.
- [ ] Cache from one account is never shown as authorization for another.
- [ ] Model capabilities/context/reasoning/tool support are mapped conservatively.
- [ ] Provider/account labels are presentation only and do not replace runtime IDs.

## I. CLI, TUI, session, and agents

- [ ] Positional and flag login grammar match the spec.
- [ ] Provider/account aliases can be list/use/rename/remove/logout as specified.
- [ ] Status human and JSON outputs are distinct, stable, and secret-free.
- [ ] Non-TTY login never blocks on an interactive picker.
- [ ] Logout calls provider revocation best effort before/alongside local deletion.
- [ ] TUI provider/transport/device/account/model/error states are implemented.
- [ ] Session persistence stores binding IDs without tokens.
- [ ] Session restore handles missing/disabled/revoked credentials clearly.
- [ ] Parent and subagent can bind different providers simultaneously.
- [ ] Parent and subagent can bind different accounts of one provider simultaneously.
- [ ] Provider/account switches affect only future turns, never in-flight requests.

## J. Compatibility

- [ ] Existing xAI browser/device/API-key flows pass unchanged.
- [ ] Existing xAI auth file remains byte/schema compatible or has an approved migration.
- [ ] Enterprise/external/ACP auth modes remain supported.
- [ ] Existing custom OpenAI-compatible/Responses/Anthropic model configs pass.
- [ ] Existing API-key/env-key precedence remains unchanged.
- [ ] Provider-specific changes do not rename upstream crates/artifacts without approval.

## K. Security, privacy, and observability

- [ ] Tokens, refresh tokens, ID tokens, codes, verifiers, device IDs, cookies, and sensitive bodies are redacted.
- [ ] `Debug`, `Display`, serialization errors, traces, metrics, status JSON, and TUI errors are seeded-secret tested.
- [ ] JWT decoding is not treated as signature validation or local authorization.
- [ ] Callback request size/path/method/host/timeouts are constrained.
- [ ] TLS verification is never weakened.
- [ ] Audit events contain provider/credential opaque IDs and outcomes, not secrets/PII.
- [ ] Threat model covers token theft, callback hijack, cross-account contamination, refresh races, and protocol drift.

## L. Required test layers

- [ ] Unit: IDs, config, PKCE, claims, plan mapping, errors, headers, fingerprint, generations.
- [ ] Wire: exact URL/method/encoding/headers/body, response/error parsing, retries.
- [ ] Store: permissions, corruption, alias/defaults, CAS, crash/fault injection.
- [ ] Integration: loopback, login persistence, real composition root request.
- [ ] Concurrency: task/thread/process, same and different credentials.
- [ ] Runtime: expired token, rotated refresh, 401 exactly-once retry.
- [ ] Multi-account/provider: simultaneous requests and model collision isolation.
- [ ] CLI: parse/snapshot/non-TTY/status/logout/account commands.
- [ ] TUI: PTY/modal transitions where TUI is in scope.
- [ ] Migration/session: old records, restore, disabled/missing credentials.
- [ ] Fuzz: callback, errors, JWT payload, polling, metadata, redaction, model catalog.
- [ ] Real protocol: manual/protected smoke only with authorization and ephemeral credential home.

## M. Documentation and evidence gate

- [ ] Product spec, code spec/layout, protocol baseline, state flows, and phase ledger agree.
- [ ] Intentional architecture deviations are approved and recorded before coding.
- [ ] One canonical phase status exists.
- [ ] Every completion claim maps requirement → symbol → test → command result → commit.
- [ ] Mock-only evidence is labeled as such.
- [ ] External authorization blockers are not presented as code completion.
- [ ] Dirty state and commands not run are reported.
- [ ] Final review explicitly checks the production composition root.

## Mandatory release scenarios

The provider cannot be stable until all applicable scenarios pass:

1. New provider login works without another provider's CLI/config directory.
2. Legacy xAI and the new provider coexist.
3. Two accounts of the new provider coexist and select the same model independently.
4. Parent/subagent issue concurrent requests using different providers.
5. Parent/subagent issue concurrent requests using different accounts.
6. Expiration refreshes transparently with one cross-process refresh.
7. A stale 401 does not overwrite or invalidate a newer generation.
8. Refresh identity mismatch aborts without persisting the new token.
9. Model cache and headers remain account-scoped.
10. No seeded secret appears in any output/diagnostic surface.
11. Kill switches disable new use without corrupting stored credentials.
12. Authorization/client registration and supported endpoint use are documented as approved.

