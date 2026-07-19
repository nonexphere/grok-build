# Handoff C7-D — Scripted stdio vertical smoke (Wave 2 evidence)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |

## Goal

Under `/tmp/grok-goal-5598c3040156/implementer/smoke/` capture a **scripted** initialize → session start → turn start → item/transcript path over **real** stdio NDJSON transport using FakeRuntime or real facade (document which). Not a unit test name alone.

Deliver:
1. A small Rust bin test or shell script that drives stdio lines and asserts response content (session id, turn id, non-empty transcript or item)
2. Log: `SCRATCH/smoke/stdio-vertical.txt` with full command output showing primary observables
3. Gate optionally: existing stdio tests still green

Use `FacadeProcessor` + stdio transport black-box if available.

## Report

Path to smoke log + command + asserted observables.
