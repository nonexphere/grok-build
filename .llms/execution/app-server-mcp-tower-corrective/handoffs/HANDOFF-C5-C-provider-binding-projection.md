# Handoff C5-C — ProviderBinding projection on session/turn rows

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |

## Goal

Stop hardcoding `provider_binding: None` on projected Session/Turn rows when start params carried a binding. Persist and re-read structured identifier-only ProviderBinding (no secrets).

## Approach

- On `start_session`, store binding in summary extension / sidecar / adapter map keyed by session id (prefer durable on Summary if field exists; else session-local map persisted under session dir as JSON without secrets)
- Project on list/read/start/resume/fork/turn results
- Tests: start with ProviderBinding → read_session returns same identifiers; no api_key material in JSONL

## Owned

- shell `app_server_runtime/**` (coordinate with C6-B — sequential if same files; **C6-B first for R10 if conflict, then C5-C**)
- tests

Actually: prefer **wait if C6-B edits same file**; implement binding map carefully with minimal conflict.

## Report

Files, RED/GREEN.
