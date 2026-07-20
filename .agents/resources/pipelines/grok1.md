**Aqui está o prompt completo, otimizado e pronto para copiar-colar no GPT-5.6 Sol (ou modelo frontier equivalente).** 

Ele foi construído seguindo o padrão de handoff de alta precisão para gerar **exatamente 2 arquivos Markdown 100% completos, prontos para commit** no seu fork do repositório `https://github.com/nonexphere/grok-build`. Os arquivos serão profissionais, com tabelas cheias, mapa exaustivo de features, roadmap faseado com links de issues, diagramas Mermaid e posicionamento competitivo honesto e estratégico.

```markdown
# Role
You are a world-class Staff Technical Writer + Product Strategist + Competitive Intelligence expert specialized in 2026 open-source AI agent harnesses, Rust TUIs, terminal-native developer tools, and dual-market positioning (coding agents + personal AI assistants). You know the grok-build architecture in detail (nonexphere/grok-build fork of SpaceXAI's monorepo: xai-grok-pager TUI, xai-grok-shell, xai-grok-tools, xai-grok-workspace, ACP, plugins/hooks/skills/MCP, sandboxing, mouse+fullscreen, headless, theming).

# Context & Goal
We are forking and strategically extending https://github.com/nonexphere/grok-build (SpaceXAI's production Rust-based fullscreen mouse-interactive TUI coding agent harness, the `grok` CLI). 

Our current strengths: superior interactive terminal UX, native mouse support, clean sandbox, ACP for IDE embedding, extensible plugin/hook/skill/MCP system, strong codebase awareness, file edit + shell + web search tools, headless mode, and Rust performance/safety.

The mission is to create two definitive, living, production-grade documentation files that will live in `docs/` of our fork. These files will serve as:
- The canonical feature taxonomy (single source of truth)
- Competitive positioning weapon against both coding harnesses and personal assistant harnesses
- Clear phased roadmap with GitHub issue links for the team and contributors

The files must be **100% complete v1.0** — no summaries, no "to be filled", no fluff. They must be immediately commit-ready, high-signal, and valuable for planning, onboarding, and external communication.

# Critical Requirements (Non-Negotiable)
- Output **ONLY** the two complete Markdown files, clearly separated by these exact markers:
  ```
  === FILE 1: docs/FEATURE_MAP.md ===
  [FULL CONTENT OF FILE 1 HERE — nothing else before or after this block]
  ```
  ```
  === FILE 2: docs/COMPETITOR_COMPARISON.md ===
  [FULL CONTENT OF FILE 2 HERE — nothing else before or after this block]
  ```
- Every table must be fully populated with realistic 2026 assessments.
- Use professional open-source documentation tone (precise, structured, tables, emojis for status, Mermaid diagrams).
- Be honest about gaps while confidently highlighting grok-build differentiators (Rust TUI + mouse + fullscreen + ACP + clean sandbox + crate modularity).
- All roadmap items must have concrete GitHub issue links in the format [#42](https://github.com/nonexphere/grok-build/issues/42). Use plausible existing-style numbers or mark #NEW-TO-CREATE and list them in an appendix.
- Since this is part of the **FORK REPO strategy for https://github.com/nonexphere/grok-build**, explicitly reference the fork and the goal of positioning grok-build as the premier terminal-native, extensible harness that competes in **both** the coding agent market and the personal assistant agent market.

# Required Deliverables — File 1: docs/FEATURE_MAP.md (Complete)
Create a comprehensive, hierarchical Feature Map / Taxonomy that serves as the single source of truth for all possible capabilities of modern AI agent harnesses (coding + personal assistant).

Structure:
- Short executive intro explaining purpose and relation to grok-build vision + fork strategy.
- At least one Mermaid mindmap or hierarchical flowchart showing the top-level taxonomy.
- 12–15 top-level categories (expand each with 6–12 leaf/sub-features). Minimum categories to cover exhaustively:
  1. Core Agentic Loop & Reasoning (planning, ReAct/tool-calling, reflection, multi-step workflows, dynamic harness generation)
  2. Context, Memory & Knowledge Management (short-term, persistent/long-term, local Markdown storage, vector/RAG, graph memory, user preference modeling, session resume)
  3. Tooling, Execution & Sandboxing (file read/write/edit/diff, shell execution, browser automation, web search, API calls, custom tools, permission models, containerization)
  4. Terminal UI / TUI / CLI / Headless / Embedded Experiences (fullscreen mouse-interactive TUI, scrollback, theming, accessibility, ACP/IDE embedding, headless/CI mode)
  5. Extensibility & Modularity (plugins, hooks, skills as code/MD+YAML, MCP servers ecosystem, hot-reload, custom protocols)
  6. Multi-Provider & Model Ecosystem (Grok, Claude, GPT, Gemini, local Ollama/vLLM, routing, cost-aware selection)
  7. Security, Privacy, Auditing & Compliance (sandboxing, least-privilege, local-only execution, audit logs, data retention controls)
  8. Autonomy, Proactivity & Scheduling (heartbeat/daemon, cron/scheduled tasks, background agents, proactive notifications)
  9. External Integrations & Messaging Gateways (WhatsApp, Telegram, Discord, Slack, Email, GitHub, Calendar, smart home, MCP-powered 32k+ servers)
  10. Observability, Tracing, Cost & Analytics (token usage, session replay, performance metrics, cost attribution per task)
  11. Multi-Agent Orchestration & Collaboration (sub-agent delegation, parallel workstreams, shared workspaces, team modes)
  12. Personalization, Learning & Self-Improvement (closed learning loop, autonomous skill creation/generation of SKILL.md, reflection, preference modeling)
  13. Coding-Specific Workflows (codebase indexing/RAG, multi-file refactor, test/debug/fix loops, git workflows, PR creation, long-running dev tasks)
  14. Deployment, Operations & Scalability (containerized by default, VPS/server mode, CI/CD integration, Docker, scaling)
  15. Emerging & Advanced Capabilities (multi-modal, voice, advanced memory graphs, etc. — keep realistic for 2026)

For every leaf feature include: short description, why it matters in 2026, real-world examples from the competitive landscape, and "Implementation notes for grok-build (relevant crates: xai-grok-*)".

Make it exhaustive and future-proof.

# Required Deliverables — File 2: docs/COMPETITOR_COMPARISON.md (Complete)
Create the definitive competitive positioning document.

Structure:
- Strong intro: positioning statement for the fork of grok-build — "the terminal-native, high-performance, extensible harness that leads in interactive coding UX while evolving to match the autonomy, persistence, and integration depth of top personal assistant harnesses."
- Section: Coding Harnesses Comparison (tables or category-grouped tables)
  Competitors to include as columns: grok-build (Current) | grok-build (Roadmap) | Claude Code | OpenAI Codex | OpenCode | Pi | (meta) harness tool
- Section: Personal Assistant Harnesses Comparison (tables or category-grouped tables)
  Competitors: grok-build (Current) | grok-build (Roadmap) | OpenClaw | Hermes Agent
- Use consistent status emojis + short explanatory text in cells: ✅ Full | 🟡 Partial | 🔵 Planned ([#123](link)) | ❌ None | 🌟 Key Differentiator
- Section: grok-build Strategic Roadmap (2026–2027)
  Phased table or detailed sections (Phase 1 Q3 2026, Phase 2 Q4 2026, Phase 3 H1 2027, Phase 4 H2 2027+). 
  For each item: Feature (reference FEATURE_MAP), Why it matters (gap close or differentiator), Key Deliverables, Estimated Effort, GitHub Issue link, Priority, Dependencies.
  Include 10–14 concrete, actionable items total. Balance quick coding parity wins with strategic moves into personal assistant territory (persistent memory, proactivity, integrations, self-improvement).
- Section: Competitive Advantages of grok-build (bullet list with evidence)
- Section: Known Gaps & Risks (honest)
- Section: Maintenance & Contribution Guide (how to keep docs updated, PR template suggestion, link new issues here)
- Appendix: Assumptions & Data Sources (as of July 2026), List of #NEW-TO-CREATE issues for the roadmap

Populate every cell with high-signal information. Reference real capabilities accurately (e.g., OpenClaw’s MCP ecosystem + local Markdown memory + heartbeat; Hermes’ autonomous SKILL.md generation + closed learning loop + subagents + multi-messaging; Claude Code’s dynamic workflows + git + plugins; grok-build’s current TUI/ACP/sandbox strengths).

# Final Strict Instructions
- These two files must be the **complete, production-ready first version**. A senior maintainer should be able to `git add docs/FEATURE_MAP.md docs/COMPETITOR_COMPARISON.md && git commit` immediately after generation.
- No meta text, no apologies, no "this is a starting point".
- Prioritize depth, accuracy, and strategic clarity for the fork of https://github.com/nonexphere/grok-build.
- Use Mermaid for at least one taxonomy diagram (FEATURE_MAP) and one roadmap visualization (COMPARISON).
- Output exactly in the two labeled blocks as specified.
```

**Como usar:**
1. Copie **todo** o conteúdo acima (a partir de `# Role` até o final).
2. Cole diretamente no GPT-5.6 Sol (ou modelo equivalente).
3. O modelo vai retornar exatamente os dois arquivos completos, separados pelos marcadores `=== FILE 1 ===` e `=== FILE 2 ===`.
4. Salve como `docs/FEATURE_MAP.md` e `docs/COMPETITOR_COMPARISON.md` no seu fork do repositório.
5. Depois disso, você pode criar as issues do roadmap e abrir PR.

Se quiser algum ajuste fino no prompt (ex.: adicionar mais categorias específicas, mudar o tom, incluir mais competitors, ou focar mais em um lado), é só falar que eu refino na hora. 

Pronto para gerar a primeira versão 100% completa dos docs estratégicos do fork. 🚀
