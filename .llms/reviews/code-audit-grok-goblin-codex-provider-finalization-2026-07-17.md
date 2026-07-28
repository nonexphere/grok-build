# Code Audit — Codex provider finalization — 2026-07-17

## Resumo executivo

- **Alvo:** estado atual da branch `goblin-multi-provider-codex`, HEAD `c152f29`.
- **Escopo:** integração Codex/multi-provider, auth por request, 401/refresh, logs de
  credenciais, identidade de modelo na UI/prompt, prompt cache, logout, documentação,
  testes e robustez da skill `add-provider`.
- **Evidence level:** alto — contratos, código, commits, testes e evidência de release
  disponíveis localmente.
- **Veredito:** **FAIL — a implementação não está finalizada.**
- **Release-blocking:** **sim.** Permanecem dois defeitos High confirmados: associação
  incorreta de 401 sob concorrência e exposição de material de credencial em logs/
  telemetria. O bug de identidade “Grok 4.5” também permanece parcialmente aberto.

## O que avançou e está comprovado

Desde `2ab8963`, os commits `b00e58b`, `8cb0daf` e `c152f29` corrigiram partes reais do
review anterior:

1. `primaryModelId` passa a acompanhar o wire slug após `SetSessionModel`.
2. O pager remapeia um slug curto para uma catalog key única e evita o fallback visual
   para Grok quando não há ambiguidade.
3. O ledger de stamps ganhou IDs explícitos e testes de retirada fora de ordem.
4. `AuthInfo` e o span normal de request não carregam mais `auth_prefix`.
5. `DeviceUserCode` tem `Debug` redigido.
6. Conversões Responses auditadas foram tornadas falíveis.
7. Logout Codex mantém `remote_revoked=false` quando o revoke HTTP falha, com teste na
   implementação real.
8. `prompt_cache_key` é opaca, Codex-only no sampler, chega ao wire e a métrica de cache
   é lida; o teste offline e o build passam.

Essas correções são importantes, mas não fecham os contratos end-to-end abaixo.

## Mapa do critical path

```text
ModelBinding
  -> MultiProviderBearerResolver::resolve_for_request()
  -> ResolvedBearer { token, attempt_id }
  -> SamplingClient::post()
  -> HTTP request
  -> response 401
  -> SamplingError::Auth { attempt_id }
  -> MultiProviderSessionAuth::try_recover_unauthorized_for_attempt()
  -> take_attempt(id) -> generation-aware refresh/retry
```

O contrato exige que `attempt_id` viaje com **a própria request/future**. Hoje ele é
retirado desse fluxo e colocado em um `AtomicU64` compartilhado pelo cliente e seus
clones. Esse é o root cause do principal defeito remanescente.

## Achados Critical / High

### [High][Confirmed] 401 ainda usa estado last-wins, não a tentativa que falhou

- **Type:** race condition / auth correctness / contract drift
- **Component:** `xai-grok-sampler::SamplingClient`
- **Evidence:** `client.rs:324-326` declara `Arc<AtomicU64>`; `client.rs:638-648`
  sobrescreve o valor a cada `post`; `client.rs:633-634` lê o valor somente quando a
  resposta vira erro. Clones compartilham o mesmo `Arc`.
- **Contrato violado:** `CODEX_AUDIT_REMEDIATION_PLAN.md` 1.1 exige stamp **por
  attempt**; a própria skill proíbe FIFO e last-wins.
- **Failure scenario:** request A resolve attempt 10; request B resolve attempt 11;
  B sobrescreve o atomic; A retorna 401 antes/depois de B e recebe attempt 11. A
  recovery consome o stamp de B. Em interleavings com refresh, uma resposta pode ser
  tratada como stale incorretamente, consumir o stamp errado ou deixar de executar o
  único retry válido.
- **Por que os testes não pegam:** `concurrent_out_of_order_401_uses_explicit_attempt_not_fifo`
  testa somente `AttemptStampLedger`. Não cria duas requests reais do mesmo
  `SamplingClient`, inverte as respostas e inspeciona os dois `SamplingError`.
- **Fix direction:** retornar um wrapper request-scoped (por exemplo
  `PreparedRequest { builder, attempt_id }`) e capturar esse ID dentro da future que
  envia e processa aquela resposta. Remover `last_request_attempt_id` do cliente.
  Ausência de stamp no caminho multi-provider deve falhar de forma controlada, não
  cair silenciosamente em FIFO.
- **Gate:** teste de integração HTTP com duas requests concorrentes e 401 em ordem
  invertida; cada erro deve carregar o seu ID original.

### [High][Confirmed] Segredos e prefixos de segredos continuam em observabilidade

- **Type:** security / privacy
- **Components:** sampler construction e auth 401 attribution
- **Evidence 1:** `xai-grok-sampler/src/client.rs:441-458` registra `%api_key` completo
  quando a chave não pode virar um header HTTP.
- **Evidence 2:** `client.rs:689-743` extrai 12 caracteres do bearer; `auth/attribution.rs:
  317-367` grava `sent_key_prefix` e `current_key_prefix` no unified log e em OTel;
  `auth/attribution.rs:394-406` deriva os prefixos de tokens reais.
- **Contrato violado:** `add-provider` exige zero hits para token e substrings
  4/8/12/20. O comentário “only the prefix” contradiz diretamente esse contrato.
- **Impact:** credenciais completas inválidas e material correlacionável de tokens
  válidos podem parar em arquivo local, upload diagnóstico e backend OTel. Prefixo de
  segredo continua sendo segredo; ele também permite correlação de sessões/contas.
- **Por que os testes não pegam:** o canary atual cobre `AuthInfo` e o source do span
  normal, mas não dispara o erro de header inválido nem captura todos os sinks da
  callback de 401. Há testes que explicitamente esperam o prefixo, institucionalizando
  o comportamento inseguro.
- **Fix direction:** remover valores de credencial da callback e dos sinks. Usar
  `attempt_id`, provider, credential ID opaco, geração e fingerprint/hash não reversível
  com chave/process salt quando correlação for indispensável. Nunca logar `%api_key`.
- **Gate:** canary end-to-end em sucesso, header inválido, 401 e erro upstream,
  capturando unified log + tracing/OTel, com busca por valor completo e substrings.

## Achados Medium

### [Medium][Confirmed] “Grok 4.5” foi corrigido no canto da UI, mas permanece no system prompt

- **Type:** product identity / contract drift
- **Evidence:** `.agents/issues/ui-model-identity-system-prompt-label-sticky.md` está
  `OPEN` e documenta reprodução real. `model_switch.rs` atualiza `primaryModelId`, mas
  reusa `self.agent.borrow().system_prompt()` sem re-resolver `system_prompt_label`.
- **Impact:** o wire usa `gpt-5.6-luna` e a UI pode mostrá-lo corretamente, enquanto o
  modelo recebe “You are Grok 4.5 released by xAI”. Isso explica exatamente o mismatch
  observado pelo usuário e pode fazer o modelo se identificar falsamente.
- **Fix direction:** aplicar o modelo CLI antes da construção do harness e/ou
  reconstruir o prompt com política de identidade provider-aware durante troca de
  modelo. Decidir explicitamente se providers não-xAI podem dizer “released by xAI”.
- **Gate:** TUI real iniciada com `--model gpt-5.6-luna`, validando catalog/status,
  `prompt_context.json`, `system_prompt.txt` e primeira resposta.

### [Medium][Confirmed] `prompt_cache_key` não cumpre a derivação account-scoped normativa

- **Type:** spec drift / cache affinity
- **Evidence:** `conversation.rs:2299-2327` deriva somente de namespace fixo `goblin`
  e IDs de sessão/conversa/agente. Provider e credential ID não entram no material.
  `CODEX_AUDIT_REMEDIATION_PLAN.md` 1.5 exige hash de `(namespace, provider,
  credential_id, session_id)`; PC6 exige afinidade de conta.
- **Impact:** a mesma sessão trocada/recriada sob outra credencial reutiliza a mesma
  affinity key. O backend pode isolar caches por conta, mas a implementação não prova
  nem expressa a identidade prometida e dificulta diagnóstico correto multi-account.
- **Fix direction:** derivar a chave no boundary que possui `ModelBinding`, usando
  provider + credential opacos + session/subagent identity. Não transportar IDs crus.
- **Gate:** mesma sessão + mesma conta = mesma key; mesma sessão + conta/provider
  diferente = key diferente; subagent independente = key diferente.

### [Medium][Confirmed] PC8 está marcado PASS sem o pacote de prova exigido pelo próprio goal

- **Type:** evidence gap / release honesty
- **Evidence:** `.llms/evidence/pc8-live-2026-07-17.md` contém apenas dois números e diz
  que a fonte anterior era ephemeral. Não contém comando, session/request correlation,
  key estável redigida, SSE/usage artifact, terceiro turno ou negative control.
  `CODEX_100_PERCENT_GOAL.md` PC8.4-6 exige negative control, artefatos completos e
  probe reproduzível; `TO_RELEASE.md` declara PASS.
- **Impact:** `cached_tokens=17920` é indício forte de um hit, mas não prova que o
  mecanismo implementado — em especial a key e a estabilidade do prefixo — causou o
  resultado, nem satisfaz o gate “cache complete”.
- **Fix direction:** reclassificar como `PARTIAL / live hit observed` ou executar o
  probe gated completo e persistir evidência redigida reproduzível.

### [Medium][Likely] Rematch de modelo no pager é mitigação heurística, não contrato canônico

- **Type:** architecture gap
- **Evidence:** o shell ainda publica o wire slug em `SessionModelState`; o pager tenta
  encontrar uma catalog key por último segmento e só funciona quando é globalmente
  única. A correção está no consumer, não no owner do binding/catalog identity.
- **Impact:** providers diferentes com o mesmo slug tornam o rematch ambíguo e o pager
  volta ao fallback. Persistência/restore e outros ACP consumers podem repetir o bug.
- **Fix direction:** o shell deve publicar a catalog key canônica derivada do
  `ModelBinding`; o wire slug continua restrito ao sampler. O pager pode manter o
  rematch apenas como compatibilidade de sessões antigas.

## Avaliação da skill `add-provider`

### O que melhorou

A atualização incorporou corretamente os padrões observados: wire/UI identity, request
stamp, segredo em substring, logout honesto, conversão falível, evidência durável e
taxonomia de concorrência. O checklist amplo continua bom.

### O que ainda precisa ser tornado normativo

1. **Request context ownership:** exigir que attempt ID/stamp seja propriedade da
   request/future, e proibir explicitamente `Atomic last_request`, thread-local e campo
   mutável compartilhado no client.
2. **Composition test, não ledger test:** o gate concorrente deve atravessar resolver →
   HTTP → response/error → recovery com ordem invertida.
3. **Redaction matrix por sink e por erro:** capturar tracing, unified log, OTel,
   user-facing error, Debug/Display e callback; incluir header inválido e raw upstream
   errors. Busca estática isolada não basta.
4. **Prompt cache binding:** acrescentar provider/credential/session/subagent à matriz
   de derivação e testar troca de conta na mesma sessão.
5. **Identity prompt:** a matriz precisa incluir `system_prompt_label`, conteúdo real do
   system prompt e claims de vendor, não somente picker/status/wire.
6. **Live evidence schema:** “durable path” sozinho não é prova. Exigir comando/versão,
   correlation IDs redigidos, inputs invariantes, key label, usage de todos os turnos,
   negative control e resultado bruto redigido.
7. **No fallback on invariant breach:** multi-provider 401 sem attempt ID não deve
   degradar para FIFO como comportamento de produção silencioso.

O problema atual não é ausência dos princípios na skill; é que os gates ainda permitem
testar o componente auxiliar e declarar o fluxo completo aprovado.

## Validação executada

| Comando | Resultado |
|---|---|
| `git diff --check origin/main..HEAD` | PASS |
| `cargo test -p xai-grok-sampler --lib` | PASS — 160 testes |
| `cargo test -p xai-grok-multi-auth --lib` | PASS — 41 testes |
| `cargo test -p xai-grok-sampling-types --lib prompt_cache` | PASS — 2 testes |
| `cargo test -p xai-grok-pager --lib model_state` | PASS — 18 testes |
| `cargo check -p xai-grok-shell` | PASS |

Warnings observados: imports/variáveis não usados em `xai-grok-multi-auth`, funções
dead-code em sampling types e `main.rs` presente em dois bin targets. Não são blockers
do provider, mas mostram que a bateria não está warning-clean.

Não foi executado novo teste live com credencial externa; a auditoria não altera
sistemas externos e a evidência PC8 existente foi avaliada como insuficiente para o
claim completo.

## Veredito por área

| Área | Estado |
|---|---|
| Codex Responses wire / phase / function calls | PASS nos testes inspecionados |
| Login/device redaction | PASS no escopo testado |
| Logout/revoke truthfulness | PASS |
| Model corner/status visual | PASS para slug único |
| System prompt/vendor identity | FAIL |
| Request-time bearer resolution | PASS sequencial |
| Attempt-bound recovery concorrente end-to-end | **FAIL — release blocker** |
| Secret-free observability | **FAIL — release blocker** |
| Prompt cache wire/offline behavior | PASS parcial |
| Prompt cache account-scoped derivation | FAIL contra spec |
| PC8 “cache complete” evidence | PARTIAL, claim inflado |
| Multi-provider 1.0 completo | não alegado; R-items continuam abertos |

## Ordem recomendada de remediação

1. Eliminar o atomic last-wins e adicionar o teste HTTP concorrente real.
2. Remover tokens/prefixos de todos os sinks e executar canary por sink/error path.
3. Corrigir identidade no harness/system prompt e canonicalizar ACP model identity no
   shell.
4. Tornar cache key provider+credential scoped.
5. Reexecutar PC8 com negative control ou reduzir honestamente o claim.
6. Só então repetir toda a bateria e declarar finalização.

## Estado do worktree

A auditoria criou apenas este relatório. Já existiam itens untracked fora do escopo:
`.agents/skills/create-pr/`, `.agents/skills/push-grok-updates-to-goblin/`,
`.llms/grok-build/`, `AGENTS.md` e `changes/`; eles não foram modificados.
