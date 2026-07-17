# Identity and Event Ordering

**Fonte de verdade.** App Server possui Thread/Turn/Item e event sequencing;
Goal Runtime possui Goal/Task/Verifier/Evidence IDs e publica correlações.

## Identidades estáveis

| Entidade | Fonte preferida | Regra |
|---|---|---|
| Thread | UUID de sessão existente | não muda em rebuild |
| Turn | prompt ID existente | allocator determinístico apenas quando ausente |
| Item | tipo + source identity + epoch | estável entre replay/rebuild |
| Interaction | ID durável próprio | diferente de JSON-RPC request ID |
| Goal | UUID/ULID persistido | independente da sessão, embora MVP limite um ativo |
| Goal event/task/report | ID persistido | inclui revision/causal relation |

## Ordenação

1. `eventSeq` cresce estritamente por Thread.
2. revision de Item nunca diminui.
3. eventos de goal carregam `goal_id`, `objective_revision`, `record_revision`
   e, quando projetados, `thread_id`/`turn_id`.
4. rewind incrementa `history_epoch` e invalida cursors da história removida.
5. snapshot-then-live captura watermark, bufferiza eventos posteriores, entrega
   snapshot até o watermark e então drena o buffer sem gap ou duplicação.
6. resultado stale pode ser auditado/contabilizado, mas não muta estado atual.

## Idempotência

Mutations externas carregam `idempotencyKey`. Repetição com mesmo payload
retorna o resultado original; mesma chave com payload diferente falha com
conflito explícito.

## Invariantes verificáveis

- replay + live equivale a live ininterrupto;
- projection rebuild preserva IDs já expostos;
- um Item terminal nunca volta a in-progress;
- conclusão de Turn não deixa interaction obrigatória sem resolução;
- completion de goal só referencia verifier report da revisão atual.
