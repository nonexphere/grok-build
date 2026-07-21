# Wave — ACP JSONL to facade replay (2026-07-19)

## Objetivo

Fechar o último elo do gate PSH-06: demonstrar que uma notificação produzida
por ACP, persistida pelo sink lifecycle-owned, reaparece pela facade Tower sem
depender do snapshot/buffer em memória do host.

## Implementação do teste

`acp_persisted_notifications_are_replayed_through_shell_facade`:

1. inicia o mock de inferência e a factory ACP real;
2. executa um prompt real e aguarda sua resposta;
3. instancia `ShellSessionActorRuntime` sobre o mesmo root durável;
4. consulta `GrokRuntimeFacade::replay` com `SubscribeParams`;
5. aguarda o flush assíncrono e exige evento projetado `ItemCompleted` ou
   `ItemDelta` além do snapshot `SessionChanged`.

## Validação

```text
cargo test -p xai-grok-shell --test product_acp_host
PASS: 5 testes
git diff --check
PASS
```

## Estado atualizado

PSH-06 agora tem evidência de notification → JSONL → replay/facade e de
cancelamento observável. Continuam pendentes o actor canônico produtivo,
Interaction real, capacidades promovidas e os gates black-box/soak.
