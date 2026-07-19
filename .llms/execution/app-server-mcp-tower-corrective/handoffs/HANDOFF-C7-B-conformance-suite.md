# Handoff C7-B — Shared FakeRuntime vs real adapter conformance (GLM build)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |
| Wave | C1 residual item 13 + C7 gap G3 |

## Goal

One normalized conformance suite that runs the same facade scenarios against `FakeRuntime` and `ShellSessionActorRuntime` (storage + command-routing where resident inject possible) and compares normalized results.

## Owned

- Tests under shell or tower as appropriate
- Prefer `crates/codegen/xai-grok-shell/tests/c7_conformance.rs` or tower tests
- Ledger `waves/c7-conformance.md`, `tests/c7/*`

## Scope of scenarios (minimum)

list/start/read/fork/replay; turn start/steer/interrupt when real has resident via test spawner; unsupported archive honesty

## Must NOT

Require live credentials or full production spawn

## Report

Files, RED/GREEN, gaps.
