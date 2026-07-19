# Handoff C3-A — WebSocket transport surface map (GLM explore, read-only)

| Field | Value |
|---|---|
| Agent role | **repo-explore** (read-only) |
| Model | `glm-5.2` |
| Wave | C3 prep (item 20–21 inputs) |
| Capability | **read-only** — no product code edits |
| Parallel with | C1-G, C4-A, C5-A (all non-overlapping; you write only ledger map file) |
| Branch | `goblin-implement-epic-tree` |

## Goal

Produce an evidence-backed map of everything needed to implement a **real** WebSocket app-server listener over the shared processor — so C3 implementer can start RED tests without rediscovery.

## Read first

- Corrective contract § Wave C3 (items 20–25)
- `crates/codegen/xai-grok-app-server/src/transport/websocket.rs`
- `crates/codegen/xai-grok-app-server/src/transport/{mod,stdio,in_process}.rs`
- `crates/codegen/xai-grok-app-server/src/{processor,controller,security,lib}.rs`
- Composition: `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs`
- Epic/spec snippets under `.llms/grok-build/` for App Server network tasks (AS104 etc.)

## Deliverable (single file you may create/overwrite)

`.llms/execution/app-server-mcp-tower-corrective/waves/c3-ws-surface-map.md`

Must include:

1. **Current state**: what `websocket.rs` actually does (stub vs real listener vs helper).
2. **Shared processor entry points** (`file:fn`) for request handling reused by stdio/in-process.
3. **Missing pieces** for black-box acceptance: handshake/subprotocol, header auth, text frames, ping/pong, binary/batch/oversize rejection, disconnect, bounded writer, slow-client resync.
4. **Security**: cleartext non-loopback policy, auth header path, threat notes (HUMAN TLS remains HUMAN).
5. **Suggested RED test names** and which crate owns them.
6. **Owned files for future C3-B implementer** (non-overlapping with Shell turn work).
7. **Risks / blockers** with evidence.

## Must NOT

- Edit Rust product sources
- Mark C3 PASS
- Invent external services

## Report back

Path to `c3-ws-surface-map.md` + 10-line executive summary + GO/NO-GO for starting C3-B after C1-G.
