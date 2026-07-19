# Handoff C6-A — Tower tools ACL + cross-surface parity (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Wave | C6 items 41–43 partial |
| Branch | `goblin-implement-epic-tree` |

## Goal

Prove all nine `tower_agent_*` tools through real adapter semantic core with fail-closed ACL; normalized error shapes; parity notes for in-process vs MCP.

## Read first

- corrective § Wave C6
- `xai-grok-tower-tools`
- `_shared/tower-agent-tools.md`
- existing tool tests

## Owned

- tower-tools + related tests only
- ledger `waves/c6-tools-acl.md`, `tests/c6/*`

## Must NOT

- shell session actor rewrites
- WS/MCP server rewrites (only call through tools)

## Acceptance

1. Tests cover all nine tools (at least invoke path + ACL deny)
2. Fail-closed default
3. Idempotency/limits if already in contract
4. Wave note + evidence

## Report

Files, RED/GREEN, residual.
