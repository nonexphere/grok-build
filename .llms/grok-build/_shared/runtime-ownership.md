# Runtime Ownership

**Fonte de verdade.** Tower Core possui registry/daemon lifecycle; App Server
possui protocolo; runtime Grok possui execução. MCP, tools e SDK espelham esses
contratos, nunca os redefinem.

| Estado/efeito | Autoridade | Consumidores |
|---|---|---|
| sessão carregada, prompt queue, inferência, cancelamento | `SessionActor` / runtime Grok | Tower, App Server |
| registry multi-session e instância daemon | Tower Core promovida do leader | App Server, MCP, dashboard ACP |
| Session/Turn/Item e subscriptions | App Server | MCP, SDK, clients |
| `tower_agent_*` semantics | Tower Agent Tools sobre facade Tower | runtime in-process e MCP |
| sandbox, tools, hooks, approvals policy | runtime Grok | todos os adapters |
| provider credential/model binding | multi-auth/provider layer | sessão/sampler |
| goal lifecycle | goal v1 hoje; `GoalRuntime` v2 quando flag selecionada | clients/projeções |

## Invariantes

1. Uma session carregada possui exatamente um actor autoritativo por Tower.
2. Adapter ou transport não cria registry, actor, permission engine ou state machine paralelo.
3. MCP server e tools in-process chamam a mesma facade e retornam os mesmos erros.
4. App Server não decide provider auth, sandbox, approval policy ou goal completion.
5. Dashboard permanece ACP/roster no MVP; não é forçado pelo App Server.
6. Arquivos em `~/.grok-oss/sessions` continuam source of truth; índices são rebuildable.
7. Uma Tower pode gerir N sessions em N workspaces; uma session pertence a uma Tower por vez.

## Boundary futuro do Goal

App Server toca goal apenas por uma porta versionada. O inventário de hot paths
deve permitir `goal_runtime = "v1" | "v2" | "disabled"` no futuro, mas nenhuma
dual-version é exigida dos outros subsistemas. [provenance: user-input]
