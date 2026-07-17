# API Backends — How Goblin Handles Chat Completions vs Responses

This document is **repo-centric**: how *this* monorepo works. External provider marketing may use different names.

## 1. The three backends

Defined in `xai-grok-sampling-types` as `ApiBackend`:

| Value (TOML / serde) | HTTP shape (conceptual) | Primary stream transform |
|----------------------|-------------------------|--------------------------|
| `chat_completions` (**default**) | OpenAI Chat Completions (`…/chat/completions`) | `stream_chat_completions` |
| `responses` | OpenAI Responses / OpenResponses (`…/responses`) | `stream_responses` |
| `messages` | Anthropic Messages (`…/messages`) | `stream_messages` |

Dispatch happens in the sampling client (`conversation_collect` / stream methods): **one client instance is bound to one `api_backend`** via its `SamplerConfig`.

Conversation history is converted into the correct wire format per backend (roles, tools, reasoning, etc.). Compatibility is **not** bit-identical across backends.

## 2. Why this matters for agent products

Goblin is an **agent harness** (tools, streaming, compaction, subagents, web search, etc.). Features assume different wire capabilities:

| Feature area | Chat Completions | Responses | Messages |
|--------------|------------------|-----------|----------|
| Basic chat + tools (many models) | Yes (OpenAI tools schema) | Yes (Responses tools) | Yes (Anthropic tools) |
| Native JSON schema with tools | Supported in enum helper | Supported | **Not** (schema blocks tools → StructuredOutput tool path) |
| Web search tool routing (README) | **Not** the documented path | **Required** | N/A |
| Codex / ChatGPT backend | N/A | **Required** + special headers/URL | N/A |
| OpenResponses `phase` / commentary | N/A | Special-cased for Codex-like backends | N/A |
| `prompt_cache_key` policy | Not Codex | Codex-only gate by binding/URL | N/A |
| Reasoning effort menus | Model-dependent | Model-dependent (+ Codex merge menu) | Thinking blocks (Anthropic) |

**Product implication:** picking `api_backend` is not cosmetic. Wrong backend → 404, 400, silent feature disable, or broken tool streaming.

## 3. Where `api_backend` is set today

| Source | Behavior |
|--------|----------|
| `[model.*] api_backend = "..."` | Explicit override |
| Built-in / remote model metadata | Often Responses for xAI agent models |
| Codex catalog merge | Forced `ApiBackend::Responses` |
| Omitted field | **Default ChatCompletions** |
| Prefetch inheritance | May copy backend from donor model when still default |

There is **no** automatic “probe `/responses` then fall back to chat” in production for custom models.

## 4. Base URL composition (critical)

The sampler uses `base_url` from config and appends the path appropriate to the backend implementation (via OpenAI client / custom paths). Conventions observed in product docs:

| Provider style | Typical `base_url` | Chat path | Responses path |
|----------------|--------------------|-----------|----------------|
| OpenAI API | `https://api.openai.com/v1` | `/chat/completions` | `/responses` |
| OpenRouter | `https://openrouter.ai/api/v1` | `/chat/completions` | `/responses` (beta; see matrix) |
| Groq | `https://api.groq.com/openai/v1` | `/chat/completions` | `/responses` (documented by Groq for some models) |
| Cloudflare Workers AI OpenAI compat | `https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1` | `/chat/completions` | **Not the primary story** |
| Codex ChatGPT | `https://chatgpt.com/backend-api/codex` | N/A | `/responses` (no `/v1` prefix style) |

**Failure mode:** putting a full `…/chat/completions` into `base_url` while backend also appends path → double path / 404.

**Failure mode:** using `responses` against a host that only implements chat → 404/400.

**Failure mode:** using chat against a backend that only implements Responses-shaped agent features → tools/web search broken.

## 5. Per-provider reality (high level)

Detailed in `PROVIDER_MATRIX.md`. Summary for backend choice:

| Provider | Safe default for agent chat today | Responses available? | Recommended first-slice backend |
|----------|-----------------------------------|----------------------|----------------------------------|
| **OpenRouter** | Chat Completions (mature) | Beta Responses API exists | **`chat_completions`** unless model explicitly needs Responses |
| **Groq** | Chat Completions (primary docs) | Documented Responses for some models | **`chat_completions`** for broad models; optional Responses for models that require it |
| **Cloudflare Workers AI** | Chat Completions OpenAI-compat | Not the main product path | **`chat_completions`** |
| **Codex (already)** | — | Required | **`responses`** |
| **xAI agent models** | — | Common | **`responses`** |

## 6. Heterogeneous catalog in one process

Already supported:

```text
Session A model: codex/{uuid}/gpt-…     api_backend=responses
Session B model: openrouter-…           api_backend=chat_completions
Subagent model:  anthropic-…            api_backend=messages
```

Each model’s `SamplingConfig` / `SamplerConfig` carries its own backend. Switching models rebuilds client config (session path). Concurrent parent/subagent mixed backends is an **existing multi-provider goal** (Codex docs); API-key providers should not reintroduce a process-global backend.

## 7. Model-level vs provider-level backend policy

Three design options the planner must choose among (problem-level):

### Option P1 — Provider default only

- OpenRouter always `chat_completions`.
- Simplest onboarding.
- Blocks models that only work well on Responses for that host.

### Option P2 — Per-model backend in catalog

- Model discovery / static allowlist sets `api_backend` per model id.
- Matches World A TOML flexibility.
- Needs metadata source (static map, API tags, user override).

### Option P3 — Capability probe

- On add-provider or first use, probe endpoints.
- Fragile, slow, noisy against rate limits; last resort.

**Recommendation for problem framing:** treat **P2 with strong provider defaults (P1)** as the design target: default chat for Groq/OR/CF; allow overrides; never silent wrong backend for known exceptions.

## 8. Interaction with web search and other Responses-only features

From shell README:

- Web search model target **must** use Responses API.
- Custom web search model requires `[model.*]` with `api_backend = "responses"`.

If a user onboards only Groq chat models, **web search may stay on an xAI/Responses model** (or be disabled). The plan must specify cross-provider feature routing, not assume every provider is a full feature peer of xAI/Codex.

## 9. Conversion and quality risks (Chat vs Responses)

Even when both endpoints “work”:

- Tool-call argument streaming differs.
- Reasoning / thinking fields differ.
- Stop reasons and refusal shapes differ.
- Compaction / prompt cache headers are backend- and provider-specific.
- OpenRouter historically had edge cases (e.g. malformed `function.arguments` JSON causing 400) already patched in conversation conversion.

Onboarding must not claim “Responses feature parity” for Chat Completions models.

## 10. Questions this backend split forces on any plan

1. Is the first vertical slice allowed to support **only Chat Completions** for the three providers?  
2. Should the catalog expose Responses variants as separate entries or as a toggle?  
3. How does `/model` display backend so users understand feature differences?  
4. When a session switches from Responses (Codex) to Chat (Groq), what happens to history items that only exist in Responses form?  
5. Are embeddings-only or image-only models filtered from the agent picker?

These are planner decisions; this doc only records that they are **real** constraints.
