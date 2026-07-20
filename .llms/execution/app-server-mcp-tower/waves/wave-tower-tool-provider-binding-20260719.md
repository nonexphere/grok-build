# Wave — Tower provider binding propagation (2026-07-19)

## Scope

Close the concrete start-path gap where `tower_agent_start` accepted a
`providerBinding` schema field but always sent `None` to the facade.

## Changes

- deserialize and validate identifier-only `ProviderBinding` from tool input;
- pass it into `SessionStartParams.provider_binding`;
- preserve the binding through the facade/session projection;
- reject malformed binding data before runtime dispatch.

## Evidence

```text
cargo test -p xai-grok-tower-tools --no-fail-fast
21 unit tests passed
24 integration tests passed
git diff --check passed
```

New regression coverage:

- `start_preserves_provider_binding_through_facade`

## Remaining gap

The current `SessionStartParams` has no `model` or `sandboxMode` fields, so
those inputs cannot be propagated without a protocol/metadata contract change.
That dependency is recorded rather than silently dropped as implemented.
