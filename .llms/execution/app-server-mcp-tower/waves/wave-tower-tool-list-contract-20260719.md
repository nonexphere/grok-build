# Wave — Tower tool list contract (2026-07-19)

## Scope

Implement the supported part of `tower_agent_list` instead of returning every
session while ignoring the schema's filters and pagination fields.

## Changes

In `crates/codegen/xai-grok-tower-tools/src/lib.rs`:

- applied `workspaceRoot`, `status`, and `includeArchived` filters;
- applied stable `updatedAtMs DESC, sessionId ASC` ordering;
- enforced `pageSize` in the schema range `1..=100`;
- added deterministic offset cursors and invalid/out-of-range cursor errors;
- rejected `agentType` filtering explicitly because the canonical `Session`
  model currently does not carry agent-type metadata.

## Evidence

```text
cargo test -p xai-grok-tower-tools acl_parity_tests --no-fail-fast
6 passed; 0 failed
```

The new cases are:

- `list_applies_workspace_filter_and_cursor_pagination`
- `list_rejects_unsupported_agent_filter_and_invalid_cursor`

The full Tower tools and MCP suites were green immediately before this list
wave; the shared semantic core remains covered by the same integration suite.
`git diff --check` passed.

## Remaining gap

`agentType` cannot be implemented correctly until v1-07 adds canonical session
metadata and the facade exposes it. The tool now fails closed with
`invalid_params` rather than claiming a filter it cannot apply.
