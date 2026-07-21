# MCP multi-session binding contract (MCP104-07)

## Current verified behavior

An HTTP MCP transport session owns exactly one optional `tower_session_id`,
one replay cursor, one history epoch, one event log, and one active turn. When
a mutating tool returns a different `sessionId`, the implementation resets the
cursor/epoch/snapshot state and invalidates existing SSE producers. The current
tests intentionally prove this rebind behavior.

This does not satisfy “one MCP connection controls multiple Tower Sessions”:
events and cursors from session A cannot remain subscribed while session B is
selected, and DELETE/TTL interruption has only one active-turn slot.

## Required contract before implementation

The protocol must make subscription identity explicit. A viable contract needs:

1. Every session-scoped tool call carries or returns a canonical Tower
   `sessionId`; the server must reject an omitted or foreign target where the
   tool requires one.
2. Event subscription requests identify the target Tower session independently
   of the MCP transport session (for example `towerSessionId` in the GET query
   or a versioned MCP extension header).
3. Replay cursors are scoped by `(towerSessionId, historyEpoch)` rather than by
   the MCP transport alone; event IDs must not collide across subscriptions.
4. Disconnect/TTL cancellation tracks active turns per Tower session, not one
   scalar `active_turn_id`.
5. A binding change never silently changes the subscription target or discards
   another target's replay state.

## Decision required

The repository does not currently define whether the public extension should be
an MCP request parameter, a dedicated subscription method, or a query/header
extension for GET `/mcp`. Implementing one unilaterally would create a new
cross-client contract and could break existing SSE clients. MCP104-07 therefore
remains open pending that product/protocol decision; the current rebind tests
are retained as a safety regression, not as acceptance of the desired state.
