# Wave — MCP self-contained tool schemas

## Scope

- Epic: `40-mcp-control-plane/v1-04-mcp-contract-transport-completion`
- Task: `MCP104-01` (implementation partial; independent-client gate remains)
- Findings addressed: F-06, part of F-07

## Change

`xai-grok-tower-tools::tool_schema` now projects each canonical protocol
`$defs/<tool>_{input,output}` entry as a standalone JSON value. The MCP stdio
dispatcher and Streamable HTTP dispatcher publish the input object directly,
instead of exposing an unresolved repository-relative `$ref`.

An unused `process_mcp_stdio_batch` import was removed from the stdio transport.

## Validation

- `cargo test -p xai-grok-tower-tools every_descriptor_resolves_exact_input_and_output_definition` — PASS.
- `cargo test -p xai-grok-mcp-server tools_list_publishes_self_contained_input_schemas` — PASS.
- `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http --no-fail-fast` — 38 passed.
- `cargo test -p xai-grok-tower-tools --no-fail-fast` — 35 passed.
- `cargo check -p xai-grok-mcp-server --features streamable-http` — PASS.
- `cargo test -p xai-grok-mcp-server --no-fail-fast` — 13 passed.
- `git diff --check` — PASS.

The wait path was also corrected to read and return the session's canonical
`history_epoch`; the first RED attempt used the wrong protocol parameter type,
was rejected by the compiler, and was fixed to `SessionReadParams`.

## Remaining acceptance

`MCP104-01` stays unchecked until an independent MCP client/compiler consumes
the returned schemas without repository-private resolution logic. The current
tests prove source projection and transport parity, not external interop.

## Next critical path

The package/adapters are green, but the product readiness gate remains open:
the pager composition still constructs `ShellSessionActorRuntime` without a
real `SessionActor` factory. The next wave must introduce an owned dependency
bundle around the existing `MvpAgent::spawn_and_register_session` path and
prove a product-backed `start -> send -> replay/history` vertical slice.
