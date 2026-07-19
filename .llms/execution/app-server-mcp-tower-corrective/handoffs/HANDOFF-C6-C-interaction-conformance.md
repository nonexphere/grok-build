# Handoff C6-C — Interaction conformance in-process/stdio/WS (AS106-06)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |

## Goal

Prove `interaction/respond` (and request shape if available) across in-process, stdio, and real WS (feature websocket) with equal error/accept shapes. Use FakeRuntime and/or Shell adapter with interaction delivery hub.

Gate: `./scripts/run-rust-test-gate.sh interaction_conformance cargo test -p xai-grok-app-server interaction_conformance`

## Owned

- xai-grok-app-server tests primarily
- Optionally shell if needed for real adapter path

## Report

Files, RED/GREEN, update AS106-06 if proven.
