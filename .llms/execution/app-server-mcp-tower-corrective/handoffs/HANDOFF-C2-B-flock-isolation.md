# Handoff C2-B — Dual-instance flock isolation (TW103-03/06)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |

## Goal

Implement true dual-OS-process (or dual-thread with real OS flock) isolation for Tower instances:

- Per-instance state root under `towers/<id>/`
- `instance.lock` via `fs2` exclusive lock (workspace already has fs2)
- Second claimer of same instance id fails while held
- Two different instance ids take disjoint locks concurrently
- Endpoint/token/metadata files under instance root (minimal scaffold OK)
- RED→GREEN evidence; gate fragments `two_instances` / `instance_contention`

## Read

- `xai-grok-tower/src/instance.rs`
- `xai-grok-shell/src/leader/lock.rs` (pattern only — do not couple Tower→Shell)
- tasks TW103-02 (may already be green — re-run), TW103-03, TW103-06

## Owned

- `xai-grok-tower/**` only (+ tests/lifecycle.rs or extend tower_instance_isolation.rs)
- Evidence tests/c2

## Must NOT

- Depend on xai-grok-shell from tower
- Edit shell app_server_runtime (C6-B owns R10)

## Report

Files, RED/GREEN, update task checkboxes only if proven.
