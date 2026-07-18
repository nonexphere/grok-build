# Session / Turn / Item Identity

**Fonte de verdade.** App Server possui o protocolo; Tower Core possui a
identidade da session carregada. MCP, SDK e tools espelham os nomes.

| Grok OSS | Codex equivalente | Regra |
|---|---|---|
| Session | Thread | termo público canônico; `thread` só em adapter/referência Codex |
| Turn | Turn | uma rodada de input e execução dentro da Session |
| Item | Item | unidade observável/streamável de um Turn |
| Tower | app-server/host daemon | control plane multi-session do grok-oss |

IDs de Session derivam/preservam a UUID da sessão persistida. Turn reutiliza
prompt/run identity estável quando disponível. Item usa source identity +
history epoch + kind, de forma reproduzível no rebuild.

Métodos nativos usam `session/*`, `turn/*`, `item/*`. Um adapter Codex pode
traduzir `thread/* ↔ session/*`; nunca vaza `thread` para core, MCP
`tower_agent_*`, SDK nativo ou documentação de produto.

