# Wave — MCP public error catalog mapping (2026-07-19)

## Scope

Reconcile internal Tower tool error aliases with the public error enum in
`tower-tools.schema.json` at the MCP boundary.

## Changes

`tool_error_json` now maps:

- `forbidden` → `tower_acl_denied`;
- `invalid_params` → `invalid_arguments`;
- `unsupported` → `runtime_unavailable`;
- `method_not_found` → `internal_error`.

Internal semantic-core callers retain their existing diagnostic aliases; only
the structured public MCP projection changes. HTTP and stdio use the same
projection.

## Evidence

```text
cargo test -p xai-grok-mcp-server --features streamable-http --no-fail-fast
20 unit tests passed
38 Streamable HTTP tests passed
git diff --check passed
```

The MCP parity assertions now require `tower_acl_denied` for denied calls.

## Remaining gap

App Server error mapping and failed-operation identity still need a shared
cross-boundary contract; MCP failure responses currently expose
`operationId: null` because no failed operation identity exists.
