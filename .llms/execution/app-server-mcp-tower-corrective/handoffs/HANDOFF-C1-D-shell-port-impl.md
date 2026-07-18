# Handoff C1-D — Shell SessionActor-backed facade port (GLM implementer)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Wave | C1 items 7–13 |
| Capability | read-write product code |
| Start only after | C0 architecture review = **GO** (or primary waives with written risk) |
| Branch | `goblin-implement-epic-tree` |

## Goal

Implement the smallest Shell-owned `GrokRuntimeFacade` that maps **every** facade method to existing leader/`SessionActor` commands. Switch composition root off `FakeRuntime` for the experimental product path. Keep FakeRuntime for unit/conformance only.

## Non-negotiables

- No second actor; Tower still must not import Shell
- No hybrid Fake mutation + real JSONL list authority
- RED tests first for each behavior; use `./scripts/run-rust-test-gate.sh`
- Do not mark epic PASS without real-adapter evidence

## Files likely owned (exclusive writer)

- `crates/codegen/xai-grok-shell/src/app_server_runtime/**`
- `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs`
- Tests under shell for real adapter
- Possibly thin hooks only in session/leader if required (minimize)

## Must not edit concurrently

- Protocol crate (unless primary assigns)
- MCP/HTTP server (C3/C4)
- Provider verticals (C5)

## Deliverables

1. Real port type + tests covering all facade methods against real actor/fixtures
2. Composition root uses real port
3. Evidence under `tests/c1/` with RED then GREEN logs
4. Wave note `waves/c1-shell-port.md`
5. Update corrective STATUS

## Report back

- Files changed
- RED/GREEN commands + counts
- Remaining gaps (honest)
