# UI / identity: system_prompt_label sticky after deferred `--model` switch

| Field | Value |
|---|---|
| **id** | UI-MODEL-IDENTITY-001 |
| **status** | DONE |
| **type** | bug / product identity |
| **severity** | medium |
| **scope** | system prompt label only (primaryModelId + catalog corner fixed separately) |
| **session evidence** | `019f6fc3-e508-7c32-a3af-89db9322aeaa` (2026-07-17) |
| **closed** | 2026-07-17 — harness-only policy |

## Symptom (historical)

User starts TUI with `--model gpt-5.6-luna` (Codex). Wire inference and UI can show
`gpt-5.6-luna`, while the model answered “Sou o **Grok 4.5**” because the system
prompt said `You are Grok 4.5 released by xAI`.

## Product policy (closed)

**`system_prompt_label` identifies the harness only: `Grok Build`.**

The model does **not** need to know which LLM it is. UI corner and wire model
remain honest about the selected model; the system prompt does not.

### Resolution

1. `DEFAULT_SYSTEM_PROMPT_LABEL` = `"Grok Build"` (`xai-grok-agent`).
2. `resolve_system_prompt_label` tiers: env → user per-model → user `[agent]` →
   hard default. **Catalog / remote marketing labels are ignored.**
3. Mid-session switch no longer needs to re-map model → marketing name; default
   harness identity is stable.

## What remains separate (not this issue)

- ACP catalog key vs wire slug (finalization Wave 4).
- `primaryModelId` / pager rematch (already fixed earlier on this branch).

## Validation

- Fresh resolve with catalog `system_prompt_label = "Grok 4.5"` → **Grok Build**.
- Template: `You are Grok Build released by xAI…`
- Env / user overrides still win when set.
