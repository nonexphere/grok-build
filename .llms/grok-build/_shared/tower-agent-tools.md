# Tower Agent Tools Contract

**Fonte de verdade.** Tower Agent Tools possui nomes e schemas; Tower Core
executa; MCP Control Plane e runtime in-process espelham exatamente.

## Tools MUST do MVP

| Tool | Semântica mínima |
|---|---|
| `tower_agent_list` | listar sessions/agents com filtros e paginação |
| `tower_agent_start` | criar Session top-level em workspace/agent type |
| `tower_agent_send` | enviar input e iniciar/steer Turn conforme state |
| `tower_agent_history` | `mode=full|last`, cursor e limites de bytes |
| `tower_agent_interrupt` | interromper Turn ativo idempotentemente |
| `tower_agent_resume` | reativar Session dormant/archived permitida |
| `tower_agent_archive` | arquivar sem apagar evidência |
| `tower_agent_status` | status de Session/Turn e summary segura |
| `tower_agent_wait` | esperar mudança/terminal com timeout e cursor |

Não existe `tower_agent_hub`: “hub” significa a própria Tower. Peer messaging
agent↔agent sem passar pelo control plane é v2.

## Regras comuns

- Mesmos params/result/error codes em MCP e in-process.
- `session_id` é canônico; nunca `thread_id` na API nativa.
- History aplica paginação, max bytes e secret redaction antes de serializar.
- Mutations usam idempotency key; wait nunca segura lock de runtime.
- ACL in-process default: só `orchestrator`; client MCP usa bearer.
- Tool local chama facade direta; config MCP local automática é proibida.

## Error codes

| Code | Significado |
|---|---|
| `tower_session_not_found` | Session não existe nesta Tower |
| `tower_session_busy` | operação incompatível com Turn ativo |
| `tower_acl_denied` | agent type sem acesso in-process |
| `tower_unauthorized` | bearer ausente/inválido |
| `tower_stale_cursor` | history/wait cursor invalidado |
| `tower_payload_too_large` | input/output excedeu limite |
| `tower_timeout` | wait/operation expirou sem alegar sucesso |

