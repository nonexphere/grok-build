# Runtime Ownership

**Fonte de verdade.** Goal Runtime é dono do lifecycle de goals; App Server
espelha o contrato pela runtime facade.

## Autoridades

| Estado/efeito | Autoridade | Consumidores |
|---|---|---|
| prompt queue, inferência, cancelamento | `SessionActor`/runtime Grok | Goal Runtime, App Server |
| lifecycle, contrato, verificação e budget do goal | `GoalRuntime` | TUI, ACP, headless, App Server |
| Thread/Turn/Item, subscriptions e conexões | App Server | clientes |
| ferramentas, MCP, sandbox, hooks e skills | runtime Grok | ambos os grupos |
| aprovação/interação em execução | runtime decide; App Server roteia | controller client |
| arquivos de sessão | storage Grok | projection store |

## Regras

1. Nenhum adapter cria um segundo actor para uma sessão já carregada.
2. O modelo só envia intents de progresso, completion request ou blocker; não
   administra lifecycle.
3. O App Server não decide conclusão, continuação, permissões ou resultado de
   ferramentas.
4. Projeções podem ser reconstruídas sem mudar o estado autoritativo.
5. Lifecycle mutation exige comando tipado, revision/CAS e origem registrada.

## Integração Goal/App Server

```text
GoalRuntime → GoalService/GoalEvent → GrokRuntime facade → Item projector
App client  → typed user command   → GrokRuntime facade → GoalCommandFacade
```

Falha de App Server não pausa nem corrompe automaticamente um goal. Falha da
infraestrutura necessária à verificação pausa o goal como `InfraPaused`.

## Compatibilidade

ACP e `GoalUpdated` continuam como adapters/projeções durante migração. Formatos
legados não ganham autoridade sobre o domínio v2.
