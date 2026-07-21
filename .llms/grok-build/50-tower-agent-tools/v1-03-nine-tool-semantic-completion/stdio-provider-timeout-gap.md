# Stdio provider timeout gap

## Evidence

The real `rmcp::TokioChildProcess` test successfully performs MCP initialize,
`tools/list`, and `tower_agent_start`. Extending the same process to
`tower_agent_send` exceeded the 60-second test harness window while waiting for
the actor/provider response.

The call path is:

```text
tower_agent_send
  → ShellRuntimeAdapter::start_turn
  → ShellSessionActorRuntime::start_turn
  → SessionCommand::Prompt { respond_to }
  → oneshot receiver await (no deadline)
```

## Impact

An unavailable, unauthenticated, or stalled provider can leave an MCP stdio
request pending indefinitely. This prevents a truthful full-stdio nine-tool
matrix and is an operational readiness risk for product use.

## Required contract before implementation

Define a canonical prompt/turn deadline and its cancellation semantics:

- whether timeout is client-supplied, server-configured, or both;
- the numeric/domain error and retryability;
- whether timeout emits an interrupted/failed Turn and which operationId is
  returned;
- how a later provider response is discarded and how the actor clears its
  active-turn slot;
- parity across App Server, Tower tools, MCP HTTP, and MCP stdio.

No arbitrary timeout is added in this wave because it would silently change
long-running turn behavior without a protocol decision.
