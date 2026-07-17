# UI / identity: system_prompt_label sticky after deferred `--model` switch

| Field | Value |
|---|---|
| **id** | UI-MODEL-IDENTITY-001 |
| **status** | OPEN |
| **type** | bug / product identity |
| **severity** | medium |
| **scope** | system prompt label only (primaryModelId + catalog corner fixed separately) |
| **session evidence** | `019f6fc3-e508-7c32-a3af-89db9322aeaa` (2026-07-17) |

## Symptom

User starts TUI with `--model gpt-5.6-luna` (Codex). Wire inference, `summary.current_model_id`, and chat `model_id` are **`gpt-5.6-luna`**. The model still answers “Sou o **Grok 4.5**” because the system prompt says:

```text
You are Grok 4.5 released by xAI. ...
```

`prompt_context.json` has `"system_prompt_label": "Grok 4.5"`.

## Root cause (not fixed in code yet)

1. Interactive pager creates the session with the **create-time** catalog default (often xAI `grok-4.5` when multi-provider default resolve is late / filtered).
2. At spawn, `resolve_system_prompt_label` reads the xAI catalog entry’s `system_prompt_label: "Grok 4.5"` and bakes it into the agent harness.
3. CLI `--model` is applied as a **deferred** `SetSessionModel` after `session/new`. That path rewrites the system message from `agent.system_prompt()` **without** re-resolving the label for the new model.
4. Headless already stamps `default_model_override` **before** agent build so the label can match `-m` (see `xai-grok-pager/src/headless.rs` comment). TUI deferred switch does not get that early path for identity.

Relevant code:

- `xai-grok-shell` `resolve_system_prompt_label` — tiers: env → user per-model → `[agent]` → GB per-model → GB global → `"Grok"`.
- `xai-grok-agent` template: `You are ${{ system_prompt_label }} released by xAI…`
- Session create resolves label once from `session_model_id` in `mvp_agent` agent_ops.
- `handle_set_session_model` prompt rewrite uses existing agent prompt, not a fresh label for the target model.

## What was fixed separately (do not regress)

- `SetSessionModel` now calls `set_primary_model` so `signals.primaryModelId` tracks the wire model.
- Pager `ModelState::update_catalog` rematches short routing slugs onto unique `codex/{credential}/{slug}` keys so catalog refresh does not clobber the corner UI back to `grok-4.5`.

## Proposed future fix

Pick one (or both):

1. **Create with CLI model** — ensure TUI/leader path stamps `default_model_override` so `session_model_id` at spawn is already luna (label + primary + spawn config). Prefer early stamp over create-then-switch for identity-sensitive fields.
2. **Re-resolve on switch** — on `SetSessionModel` / zero-turn rebuild, re-run `resolve_system_prompt_label` for the target model and rebuild or rewrite the system head with the new label (Codex models may need an explicit `system_prompt_label` in catalog, e.g. display name, or fall through to default `"Grok"` / provider-specific identity policy).

Also decide product policy: should non-xAI models claim “released by xAI” at all?

## Validation when fixing

- Fresh session: `goblin --model gpt-5.6-luna` → first user ask “qual modelo é você?” must not assert Grok 4.5 solely from system identity.
- `prompt_context.system_prompt_label` and `system_prompt.txt` must not hardcode Grok 4.5 when the session model is Codex luna (unless product explicitly wants a fixed Grok persona on all providers).
- Regression: xAI `grok-4.5` sessions still get `Grok 4.5` label from catalog.

## Evidence paths

- Session dir: `~/.grok/sessions/%2Fhome%2Fguilherme%2Fgithub%2Fgrok-goblin/019f6fc3-e508-7c32-a3af-89db9322aeaa/`
- `system_prompt.txt`, `prompt_context.json`, `chat_history.jsonl`, `signals.json`, `summary.json`
- Unified log: create → `model changed: gpt-5.6-luna` without label re-bake
