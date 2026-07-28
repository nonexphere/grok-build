# BYOK Providers Onboarding — Analysis Package

**Status:** Problem definition / discovery (not an implementation plan)
**Date:** 2026-07-17
**Repo:** `grok-goblin` (fork of `xai-org/grok-build`)
**Branch context:** multi-provider auth (Codex OAuth) already in progress
**Audience for this package:** human maintainer + external planner (e.g. ChatGPT) that will produce an implementation-ready plan

## Goal of this package

Define the **problem** of giving end users a first-class, automated flow to:

1. Configure API keys for **Cloudflare (Workers AI)**, **Groq**, and **OpenRouter** (and a pattern for more later);
2. Discover and select models from those providers;
3. **Without** requiring hand-editing of `~/.grok/config.toml`.

This is **not** a finished architecture or PR plan. It is an evidence-backed problem statement so a stronger planner can design the solution without rediscovering the codebase.

## How to use (for ChatGPT / planners)

Read in this order:

| # | File | Purpose |
|---|------|---------|
| 1 | [`PROBLEM.md`](./PROBLEM.md) | User goal, non-goals, success criteria, constraints |
| 2 | [`CURRENT_STATE.md`](./CURRENT_STATE.md) | What Goblin/Grok-build already supports (BYOK vs multi-auth) |
| 3 | [`API_BACKENDS.md`](./API_BACKENDS.md) | `chat_completions` vs `responses` vs `messages` in *this* codebase |
| 4 | [`PROVIDER_MATRIX.md`](./PROVIDER_MATRIX.md) | Cloudflare / Groq / OpenRouter wire facts + auth + models |
| 5 | [`ARCHITECTURE_TENSIONS.md`](./ARCHITECTURE_TENSIONS.md) | OAuth multi-auth vs static API-key models; binding; storage |
| 6 | [`GAPS_AND_QUESTIONS.md`](./GAPS_AND_QUESTIONS.md) | Decisions the plan must resolve; open questions |
| 7 | [`EVIDENCE_MAP.md`](./EVIDENCE_MAP.md) | Paths, symbols, commands for verification |

Related existing docs (do not ignore):

- `GOBLIN.md` — fork contract (multi-provider auth)
- `task.md` — multi-provider auth architecture (Codex-first; API-key models as G7)
- `docs/architecture/multi-provider-auth/` — Codex control plane progress
- `.agents/skills/add-provider/SKILL.md` — end-to-end provider skill (OAuth-oriented; notes API-key path is different)
- `crates/codegen/xai-grok-shell/README.md` — “Custom Models” user docs (manual TOML today)

## One-sentence product goal

> After `goblin login --provider openrouter` (or equivalent interactive wizard), the user has a stored credential and a populated model picker for that provider, with the correct API backend per model, without opening a text editor.

## Confirmed product intent (from maintainer)

- Initial targets: **Cloudflare**, **Groq**, **OpenRouter**.
- “We already have support” means: inference works if the user **manually** configures `[model.*]` / env vars (OpenAI-compatible path).
- Desired delta: **onboarding UX** (key capture + model discovery + selection), not inventing a fourth HTTP protocol.
- Must respect that providers differ on **Chat Completions vs Responses** (and capability gaps).

## Explicit non-claims in this package

- No complete implementation plan, wave DAG, or acceptance checklist (that is the next planner’s job).
- No guarantee that Cloudflare/Groq/OpenRouter official APIs remain stable; wire facts are as of research date.
- No authorization claim for any third-party OAuth; these providers are primarily **API-key / token** auth.

## Suggested prompt for ChatGPT (copy-paste)

```text
You are an implementation architect for the Goblin fork of grok-build (Rust monorepo).

Read the entire folder docs/architecture/byok-providers-onboarding/ in order
(README → PROBLEM → CURRENT_STATE → API_BACKENDS → PROVIDER_MATRIX →
ARCHITECTURE_TENSIONS → GAPS_AND_QUESTIONS → EVIDENCE_MAP).

Also skim GOBLIN.md, task.md §1–§2 and §8, and the add-provider skill boundaries.

Produce:
1. Problem restatement and architecture options (at least 2) with tradeoffs.
2. Recommended approach for API-key providers vs existing Codex OAuth multi-auth.
3. Per-provider vertical slice (Cloudflare, Groq, OpenRouter): auth, storage,
   model catalog keys, default api_backend, headers, account_id if any.
4. How api_backend is chosen (static per provider vs per-model vs probe).
5. UX: CLI + TUI flows without config.toml editing; multi-key / multi-account if any.
6. Security (secret storage, redaction, never log keys).
7. Phased implementation plan with acceptance tests and failure modes.
8. Explicit list of human decisions still required.

Do not invent code paths that contradict EVIDENCE_MAP. Flag unknowns.
Prefer the smallest complete product slice over a perfect abstraction.
```
