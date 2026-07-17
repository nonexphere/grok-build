# Security and Authority Boundaries

**Fonte de verdade.** Runtime Grok é dono de sandbox/permissões; Goal Runtime e
App Server aplicam restrições adicionais sem enfraquecê-lo.

## Dados não confiáveis

Objetivos, prompts, mensagens, arquivos do repositório, tool output, MCP,
skills, hooks, schemas de cliente e metadata remota são dados não confiáveis.
Eles não alteram instruções, autoridade, budgets, scopes ou verifier policy.

## Segredos e privacidade

- tokens, credentials, ambiente secreto e hidden reasoning não entram em
  eventos, reports, logs ou protocolos;
- reasoning remoto é safe-summary-only por padrão;
- caminhos são canonicalizados e verificados contra workspace/worktree;
- outputs persistidos têm redaction, size limit e retention explícitos.

## Autoridade

| Origem | Pode | Não pode |
|---|---|---|
| modelo primário | executar tools permitidas; reportar intent | administrar goal; conceder permissões |
| verifier | produzir verdict/evidence | editar workspace; completar diretamente |
| cliente controller | responder prompts/approvals dentro do scope | burlar sandbox/hook/policy |
| observer | ler projeções permitidas | mutar ou aprovar |
| remote client | operações autenticadas e scoped | acesso implícito a arquivos/processos/secrets |

## Remote defaults

Remote é desligado por padrão. Habilitação exige autenticação, scopes,
expiração/revogação, TLS quando não-loopback, Origin allowlist, rate/size
limits e audit log sem payload secreto. `acceptAlways` remoto exige decisão de
produto e policy explícita; não é inferido.

## Falhas

Falha de verificação, autenticação, autorização, projection consistency ou
recovery é explícita e fail-closed. Slow clients são isolados por filas
limitadas; lifecycle não é descartado, deltas podem ser coalescidos.
