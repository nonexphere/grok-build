# Wave — ACP cancel lifecycle (2026-07-19)

## RED/GREEN

Foi adicionado o teste
`experimental_acp_resident_cancel_is_observable_and_persists_terminal_update`.
O primeiro resultado foi RED: o bridge retornava `PromptCompletionKind::Completed`
mesmo após `SessionCommand::Cancel`.

## Implementação

- Cada prompt ACP agora observa um canal `watch` de cancelamento compartilhado
  pelo resident.
- `SessionCommand::Cancel` incrementa a geração de cancelamento e envia o
  `session/cancel` ACP.
- O prompt cancelado resolve seu `respond_to` com
  `StopReason::Cancelled` e `PromptCompletionKind::Cancelled`, sem deixar a
  chamada pendurada.
- A geração compartilhada preserva múltiplos prompts concorrentes; um sender
  one-shot por prompt causava cancelamento falso quando o próximo prompt era
  enfileirado.
- Corrigido também um deadlock de `watch`: o próximo valor é calculado antes de
  chamar `send`, sem manter o borrow durante a escrita.

## Validação

```text
cargo test -p xai-grok-shell --test product_acp_host
PASS: 4 testes
git diff --check
PASS
```

## Gap restante

A suíte prova updates duráveis no JSONL e cancelamento ACP observável, mas ainda
falta um teste atravessando `ShellSessionActorRuntime::replay/history` após o
host persistir as notificações. Capabilities de Turn/Interaction continuam
fail-closed até esse gate e os gates de actor canônico.
