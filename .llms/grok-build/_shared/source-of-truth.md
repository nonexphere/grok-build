# Source-of-truth precedence

[provenance: AGENTS.md, handoff §13–14, review D-00.3]

When artifacts conflict, the higher row wins for its stated domain. A lower row
does not silently reinterpret a locked decision; it must be corrected.

| Priority/domain | Authoritative source | Wins over |
|---:|---|---|
| 1 repository governance | `AGENTS.md` and more-local AGENTS | every plan/scaffold |
| 2 locked product intent | `docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md` §13–14 + linked transcript | older defaults/questions and seed specs |
| 3 deepening inventory | review §10–14 + execution prompt | task summaries and shallow contracts |
| 4 native protocol | Rust serde/schemars types → generated schema snapshot → operational envelope schema → TS structural mirror, reconciled by tests | MCP/TS adapters and examples |
| 5 Tower lifecycle/ownership | `_shared/tower-instance-lifecycle.md`, `runtime-ownership.md`, `runtime-facade.md` | App Server/MCP/tool convenience behavior |
| 6 security | `_shared/control-plane-security.md` | transport defaults and user docs |
| 7 tools | `_shared/tower-agent-tools.md` + tool schema | MCP descriptors/in-process registration |
| 8 runtime current behavior | characterized `xai-grok-shell` leader/SessionActor/session storage | inferred new implementation details |
| 9 generated bindings | Rust protocol types/schema source, then generated TS | handwritten client conveniences |
| 10 seed/reference | `changes/grok_app_server_spec_bundle/*`, Codex references | nothing normative above; adapt, do not copy blindly |

If Rust, checked-in schema and Markdown disagree during this experimental pass,
the discrepancy is a blocking contract-drift failure until all three are
reconciled. No one artifact is allowed to mask drift merely by being compiled.
