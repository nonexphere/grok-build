# Matriz de rastreabilidade

Esta matriz liga decisão humana a contrato, epic e prova planejada. Tasks
granulares vivem nos `tasks.md` de cada epic.

| Requisito | Contrato | Epic owner | Prova de aceite |
|---|---|---|---|
| Session canônico | `_shared/session-turn-item-identity.md` | `30/v1-01` | schema/TS/MCP sem `thread` nativo |
| Tower promove leader | `_shared/runtime-ownership.md` | `20/v1-01` | actor/registry ownership + connect-or-spawn tests |
| N sessions/workspaces | `_shared/tower-instance-lifecycle.md` | `20/v1-02` | multi-session integration |
| N Towers/machine | mesmo | `20/v1-03` | isolated endpoint/token/state smoke |
| T4 connect/spawn default | mesmo | `20/v1-03` | CLI lifecycle tests |
| WS early | App Server contract | `30/v1-04` | transport conformance WS |
| MCP remoto same release | MCP contract | `40/v1-01..02` | stdio + HTTP/SSE conformance |
| bearer full-control | `_shared/control-plane-security.md` | `30/v1-04`, `40/v1-02` | auth/redaction/threat tests |
| sem Origin/scopes/TLS mandatory | mesmo | `30/v1-04` | config/warning tests + docs |
| tools MUST | `_shared/tower-agent-tools.md` | `50/v1-01` | one contract suite por operação |
| in-process + MCP semantic parity | runtime/tools contracts | `50/v1-02` | differential conformance |
| ACL orchestrator default | security/tools | `50/v1-02` | allow/deny/config tests |
| local tool sem MCP loop | runtime ownership | `50/v1-02` | composition assertion |
| TS SDK real | Session contract | `60/v1-01` | Node/browser WS scripts + drift gate |
| dashboard intocado | runtime ownership | `30/v2-01` | MVP regression ACP/roster |
| Goal fora do core | Goal docs | `70/*` | DAG sem Goal v2 em release core |
| goal v1/v2 flags | Goal contract | `70/v2-01..07` | dual-version/rollback fixtures |
| BYOK OR/Groq/CF | provider docs | `10/v1-02..05` | onboarding→catalog→turn→logout |
| TDD | `TDD.md` | todos | RED/GREEN + package/conformance gate |
| gateways/voice só backlog | project docs | `80/v1-01`, `90/v1-01` | nenhum dependency inverso/core code task |

## Issues persistentes incorporadas

| Issue | Epic que deve reconciliar | Resultado esperado |
|---|---|---|
| `data-001` Codex 401 FIFO fallback | `10/v1-01` | attempt-bound recovery provada ou status corrigido |
| `data-002` compaction cache key | `10/v1-01` | account-scoped key na composition real |
| `data-003` Codex-only login gate | `10/v1-01..02` | provider-driven gate sem regressão Codex |
| `docs-001` release contradiction | `10/v1-01` | uma única readiness matrix |
| `docs-002` harness audit artifact | `10/v1-01` | evidence durável ou claim removida |
| `operations-001` remote tip validation | `10/v1-01` | validação rotulada no scope real |
| `testing-001` live skip as pass | `10/v1-01` | skip/blocked nunca PASS |
| `testing-002` branch diff check | `10/v1-01` | gate reproduzível ou issue honesta |
| `ui-model-identity-system-prompt-label-sticky` | `10/v1-01` | harness label e model identity separados |
