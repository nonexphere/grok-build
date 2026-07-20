# Wave — core quality and clippy gate (2026-07-19)

## Scope

Run the strict warning gate over the touched Tower Tools, MCP Server, and App
Server crates and correct actionable findings without changing protocol
semantics.

## Changes

- switched Tower workspace canonicalization to existing workspace dependency
  `dunce` for cross-platform path behavior;
- added `InstanceDirectory::is_empty` alongside `len`;
- documented the intentional `MethodDocument` large-variant representation;
- replaced MCP SSE cursor `Result<_, ()>` with typed `EventCursorExpired`;
- simplified collapsible conditionals and redundant iterator/test expressions;
- removed unused imports/variables in App Server tests.

## Evidence

```text
cargo clippy -p xai-grok-tower-tools -p xai-grok-mcp-server \
  -p xai-grok-app-server --features xai-grok-mcp-server/streamable-http \
  --all-targets -- -D warnings
Finished successfully

cargo test -p xai-grok-tower-tools --no-fail-fast
21 unit + 24 integration passed

cargo test -p xai-grok-mcp-server --features streamable-http --no-fail-fast
20 unit + 38 Streamable HTTP passed

cargo test -p xai-grok-app-server --no-fail-fast
39 passed

git diff --check
passed
```

## Remaining warning

Cargo still emits the pre-existing pager-bin warning that the same `main.rs`
is registered as three binary targets (`goblin`, `grok-oss`, `xai-grok-pager`).
It does not affect these crate gates, but remains an inventory item for release
hardening.
