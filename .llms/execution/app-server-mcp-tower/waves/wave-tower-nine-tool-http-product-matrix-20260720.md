# Wave: nine-tool MCP HTTP product matrix — 2026-07-20

## Change

- Added public `validate_tool_output` to the shared Tower semantic core.
- Added a real Streamable HTTP integration fixture that initializes one MCP
  session and invokes all nine published Tower tools.
- Eight successful `structuredContent` results are validated against their
  published output schemas.
- `tower_agent_interrupt` is exercised through its real no-active-turn error
  path (`turn_not_found`), because the test `FakeRuntime` completes turns
  synchronously; the test does not mislabel that error as a successful output.

## Evidence

```text
cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http
41 passed; 0 failed
```

The existing semantic-core suite remains green with 24 unit and 24 integration
tests.

## Remaining scope

TA103-12 remains partial: the equivalent full matrix through the real stdio
product process is not yet implemented, and a successful interrupt requires a
live-turn runtime fixture rather than the synchronous FakeRuntime.

An attempted rmcp stdio extension reached `tower_agent_send` but exceeded the
test's 60-second harness window while waiting for the provider/actor. The
stable stdio test was restored and re-run successfully; the hang is recorded as
a product timeout/provider-fixture gap, not treated as a passing case.
