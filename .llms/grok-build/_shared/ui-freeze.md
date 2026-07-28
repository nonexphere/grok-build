# Dashboard, ACP and roster MVP freeze

[provenance: handoff T3/§13.14, review D-UI.1..3]

The MVP adds parallel App Server/MCP surfaces and does not migrate the dashboard.

## Do-not-modify-for-migration surfaces

The following may receive only separately justified regression fixtures or
narrow Tower adapter seams; they must not be converted to App Server clients in
the MVP:

- `crates/codegen/xai-grok-pager/src/` dashboard/view state and rendering;
- `crates/codegen/xai-grok-pager-bin/src/main.rs` existing interactive boot;
- `crates/codegen/xai-grok-shell/src/agent/roster.rs`;
- `crates/codegen/xai-grok-shell/src/agent/handlers/session.rs` (or current
  equivalent roster/session extension handlers);
- `crates/codegen/xai-grok-shell/src/leader/protocol.rs` ACP/leader bytes;
- existing `x.ai/sessions/list` and `x.ai/sessions/changed` method shapes;
- ACP permission/update paths used by the pager.

Roster ACP remains the dashboard source: resident+dormant snapshot via
`x.ai/sessions/list`, updates via `x.ai/sessions/changed`, existing leader IPC
for multiple clients. The future `30/v2-01` epic must characterize these bytes,
prove App Server observation parity, provide incremental rollback, and receive a
separate human approval before replacement.
