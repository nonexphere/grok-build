# Code Audit — grok-goblin Codex provider — 2026-07-17

## Resumo Executivo

- **Alvo:** commits `3464897` e `2ab8963` sobre `origin/main`.
- **Escopo:** provider Codex, autenticação multi-provider, Responses wire/replay, prompt cache, catálogo/binding de modelos, sessão/ACP/UI e refinamentos necessários na skill `add-provider`.
- **Evidence level:** alto para código e testes unitários de wire/sampler; médio para UI integrada e live evidence.
- **Release-blocking:** **sim**.
- **Top risks:** identidade de modelo divergente entre wire e UI; recovery 401 correlacionado por FIFO e não por request exato; prefixos de credenciais registrados em logs.

O provider avançou substancialmente. As suítes `xai-grok-sampling-types` (295/295) e `xai-grok-sampler` (157/157) passaram nesta auditoria. Phase, materialização, function-call siblings, opaque prompt-cache key, capability gate, compaction key e commentary routing possuem cobertura útil. Ainda assim, os invariantes de identidade e segurança abaixo contradizem a alegação de implementação pronta.

## Mapa Arquitetural Relevante

```text
Codex /models
  -> ProviderModel
  -> merge no ModelsManager como chave codex/<credential>/<slug>
  -> ACP ModelInfo.model_id = chave do catálogo
  -> SessionHandle.model_id / SessionModelState.current_model_id
  -> pager ModelState.current
  -> display name da UI

ModelEntry
  -> SamplingConfig.model = slug de wire
  -> BearerResolver.current_bearer()
  -> request Responses
  -> erro 401
  -> SessionAuth.try_recover_unauthorized()
  -> AttemptStampLedger FIFO
```

O contrato correto exige que chave de catálogo, slug de wire e display name sejam distintos, mas semanticamente correlacionados durante seleção, persistência, restore e atualização do catálogo.

## Achados Critical / High

### [HIGH][CONFIRMED] Modelo correto no wire pode aparecer como Grok 4.5 na UI

- **Type:** contract mismatch / user-visible bug
- **Component:** shell `model_state` → ACP → pager `ModelState`
- **Evidence:**
  - `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs:2268-2300`
  - `crates/codegen/xai-grok-shell/src/agent/models.rs:1614-1688`
  - `crates/codegen/xai-grok-pager/src/acp/model_state.rs:308-328`
- **Failure Scenario:** uma sessão usa/persiste o slug `gpt-5.6-luna`, enquanto o catálogo ACP contém `codex/<credential>/gpt-5.6-luna`. O shell publica o slug diretamente; o pager só aceita o current se ele for uma chave literal do catálogo e o descarta. A UI conserva/adota o default anterior, embora `SamplingConfig.model` continue enviando `gpt-5.6-luna`.
- **Impact:** status bar, dashboard, session info ou picker mentem sobre o modelo ativo; decisões de usuário podem ser tomadas sobre uma identidade falsa.
- **Validation:** teste integrado cobrindo seleção por slug curto e chave completa, request capturado, `SessionModelState`, status bar, refresh de catálogo e restore.
- **Fix Direction:** `SessionHandle.model_id` e `SessionModelState.current_model_id` devem usar a chave canônica de catálogo. O slug deve ficar restrito a `ModelEntry`/`SamplingConfig`. Canonicalizar antes de publicar ACP e falhar explicitamente se não houver correspondência.
- **Handoff:** `@implementation-loop`

### [CRITICAL][CONFIRMED] Stamp de 401 continua sem vínculo com a requisição exata

- **Type:** race condition / auth correctness
- **Component:** `AttemptStampLedger`, bearer resolver e recovery da sessão
- **Evidence:**
  - `crates/codegen/xai-grok-multi-auth/src/request_stamp.rs:20-74`
  - `crates/codegen/xai-grok-shell/src/auth/multi_provider_resolve.rs:70-82,388-390,470-478`
  - `crates/codegen/xai-grok-sampler/src/config.rs:169-179`
- **Failure Scenario:** requests A e B são enviados nessa ordem, mas B recebe 401 primeiro. O resolver cria attempt IDs, porém `BearerResolver::current_bearer()` retorna apenas `String` e descarta o ID. Recovery consome o stamp mais antigo por FIFO, associando o 401 de B ao stamp de A.
- **Impact:** refresh/retry pode usar geração ou identidade temporal errada; a propriedade A1 declarada como PASS não está satisfeita sob concorrência real.
- **Validation:** dois requests simultâneos na mesma sessão/resolver, responses deliberadamente invertidas e generations distintas.
- **Fix Direction:** retornar bearer + `AttemptId`/lease; anexar ao request in-flight e devolver o mesmo ID no erro. Recovery autoritativo deve usar `take_attempt(id)`, nunca FIFO/last-wins.
- **Handoff:** `@architecture-spec-authoring` + `@implementation-loop`

### [CRITICAL][CONFIRMED] Prefixos de bearer e API key são registrados

- **Type:** security / privacy
- **Component:** sampler HTTP logging
- **Evidence:** `crates/codegen/xai-grok-sampler/src/client.rs:642-663`; `GOBLIN.md` Security Rules.
- **Failure Scenario:** cada request registra 20 caracteres do header Authorization e 12 de `x-api-key` em nível info.
- **Impact:** material parcial de credencial em arquivos de log, telemetry pipelines ou suporte; viola o contrato explícito de não registrar tokens.
- **Validation:** secret canary e busca por prefixos/sufixos de 4/8/12/20 caracteres em todos os sinks.
- **Fix Direction:** remover os valores; registrar apenas presença, provider ID, credential ID opaco e attempt ID.
- **Handoff:** `@implementation-loop`

### [HIGH][CONFIRMED] Logout declara revogação mesmo ignorando falha upstream

- **Type:** contract mismatch / security UX
- **Component:** `CodexAuthProvider::logout`
- **Evidence:** `crates/codegen/xai-grok-multi-auth/src/providers/codex/mod.rs:484-512`.
- **Failure Scenario:** `revoke_token` falha; o erro é ignorado; `remote_revoked` recebe `request.revoke` e retorna true.
- **Impact:** usuário acredita que a credencial remota foi invalidada quando apenas o logout local ocorreu.
- **Validation:** mock de revoke 4xx/5xx/timeout.
- **Fix Direction:** true somente com confirmação; falha best-effort retorna false + warning tipado, separando remote revoke de local delete.
- **Handoff:** `@implementation-loop`

### [HIGH][CONFIRMED] Device user code aparece em `Debug`

- **Type:** security / secret exposure
- **Component:** `DeviceFlow::Debug`
- **Evidence:** `crates/codegen/xai-grok-multi-auth/src/providers/codex/mod.rs:60-79`.
- **Failure Scenario:** debug incidental do flow inclui o código transitório apresentado ao usuário.
- **Impact:** código de autorização pode chegar a logs/dumps.
- **Fix Direction:** redigir user code, device auth ID, flow IDs correlacionáveis e verification URLs parametrizadas.
- **Handoff:** `@implementation-loop`

### [HIGH][CONFIRMED] Item Responses não mapeável causa panic no caminho de produção

- **Type:** operational risk / error handling
- **Component:** Responses request conversion
- **Evidence:** `crates/codegen/xai-grok-sampling-types/src/conversation.rs:2621-2637`; sampler usa `From<&ConversationRequest>`.
- **Failure Scenario:** histórico contém `OpaqueWire` que não pode ser convertido; `build_responses_input` executa `panic!`.
- **Impact:** request/task pode encerrar em vez de retornar erro recuperável preservando a sessão.
- **Fix Direction:** produção deve consumir o helper fallible e mapear para erro tipado/user-visible. Fail-loud não significa panic.
- **Handoff:** `@implementation-loop`

## Achados Medium

### [MEDIUM][CONFIRMED] Inventário de release contradiz a própria prova PC8

- **Type:** spec/documentation drift
- **Evidence:** `TO_RELEASE.md` afirma live PC8 PASS e `cached_tokens=17920`, mas depois diz “Not claimed: live cache hits”; o README blurb também diz que a prova permanece gated. O relatório anterior diz NOT claimed, enquanto o commit se intitula live proof.
- **Impact:** reviewers não conseguem determinar qual claim é autorizado.
- **Fix Direction:** persistir artefato live redigido fora de `/tmp` e reconciliar status, matrices, README, goal e reports.

### [MEDIUM][CONFIRMED] Teste chamado cross-process não cria dois processos do SO

- **Type:** evidence gap
- **Evidence:** `TO_RELEASE.md` reconhece que a prova usa dois TokenManagers + file flock e mantém o teste real multiprocess como deferred.
- **Impact:** lock file é exercitado, mas lifecycle/lock ownership/process crash não são provados end-to-end.
- **Fix Direction:** processo pai cria home/credential; dois filhos sincronizados tentam refresh; assert de uma chamada upstream e adoção correta.

### [MEDIUM][CONFIRMED] `git diff --check origin/main..HEAD` falha

- **Type:** quality gate failure
- **Evidence:** trailing whitespace em código Rust, goals, reports e arquivos da skill.
- **Impact:** o gate explicitamente exigido pela `add-provider` não está verde.

## Gaps de Teste

1. Não existe teste end-to-end que compare simultaneamente:
   - chave selecionada;
   - slug enviado;
   - binding/credential;
   - `SessionModelState.current_model_id`;
   - display name na UI;
   - valor após restore/catalog refresh.
2. Os testes de stamp provam FIFO sequencial, não responses concorrentes fora de ordem.
3. Redaction tests não detectam prefixes/suffixes de secrets.
4. Logout não cobre revoke failure com outcome honesto.
5. Opaque wire cobre panic esperado, não erro de produção recuperável.
6. Evidência PC8 não inclui, em superfície durável inspecionada, o controle negativo definido pelo goal.

## Gaps de Spec / Contrato

- O termo “request-scoped” é usado para uma fila por resolver/sessão; deve significar request attempt exato.
- Não há contrato canônico reunindo catalog ID, wire slug, persisted ID, ACP ID e display name.
- `TO_RELEASE.md` mistura “PASS live” e “not claimed”.
- “Cross-process” é usado para dual-manager/flock e, em outros trechos, reservado para dois processos reais.

## Segurança

Findings confirmados: prefixos de credencial em logs; device user code em Debug; logout remote status enganoso. Nenhum token completo foi deliberadamente exibido nesta auditoria.

## Validação Executada

- `cargo test -p xai-grok-sampling-types --lib`: **295 passed**.
- `cargo test -p xai-grok-sampler --lib`: **157 passed**.
- Primeira tentativa das demais suítes: bloqueada por `No space left on device` em `/home` e `/tmp`.
- Após liberação: nova bateria iniciada, mas aguardou lock de build ocupado por outro `cargo`; resultado terminal deve ser anexado ao handoff quando disponível.
- `git diff --check origin/main..HEAD`: **FAIL**, trailing whitespace.

## Refinamentos Propostos para `add-provider`

Lifecycle: **active → refine**; não requer recriação estrutural.

1. **P0 — Model identity matrix:** exigir provider, credential, catalog key, wire slug, persisted ID, ACP current ID e display name, com round-trip seleção → request → persistence → restore → UI.
2. **P0 — Attempt-bound auth:** proibir last-wins e FIFO; attempt ID/lease deve atravessar HTTP e recovery. Testar responses invertidas.
3. **P0 — Secret substring redaction:** procurar token completo, prefixos, sufixos, hashes inseguros e headers truncados.
4. **P0 — UI/wire consistency scenario:** picker, status bar, `/session-info`, headless JSON e wire request devem concordar semanticamente.
5. **P1 — Honest logout:** remote revoke confirmado, local delete e warning são estados distintos.
6. **P1 — Transient login secrets:** explicitar user code, verification URI parametrizada, flow ID e callback code.
7. **P1 — Fallible production conversion:** adicionar common mistake “fail-loud panic”.
8. **P1 — Vertical slice through restore/UI:** não encerrar a slice em inference; reiniciar/restaurar e inferir de novo com a mesma binding.
9. **P1 — Durable live evidence:** `/tmp` não pode ser a única fonte; persistir artefato redigido ou CI artifact.
10. **P1 — Documentation consistency gate:** um requirement não pode ser simultaneamente PASS, deferred e not claimed.
11. **P2 — Concurrency taxonomy:** task, thread, dual-manager, OS-process e multi-host devem ser provas distintas.
12. **P2 — Validation preflight:** espaço em home/tmp/target, toolchain e capacidade de link antes da bateria final.

Common mistakes a adicionar:

- Wire/UI identity split.
- FIFO request stamp.
- Secret-prefix telemetry.
- Best-effort success lie.
- Fail-loud panic.
- Ephemeral proof.
- PASS/not-claimed contradiction.
- Manager concurrency theater.

Pelo contrato de `refine-skill`, estes refinamentos foram apenas propostos; a skill não foi editada sem aprovação do usuário.

## Handoff Sugerido

1. Corrigir canonicalização do model ID ACP/UI.
2. Remover token prefixes imediatamente.
3. Redesenhar auth attempt context antes de declarar A1 PASS.
4. Corrigir logout/Debug/panic.
5. Reconciliar PC8 docs/evidence.
6. Aplicar os refinamentos aprovados à `add-provider`.
7. Reexecutar shell, pager, multi-auth, check, diff-check e um smoke Codex final.

