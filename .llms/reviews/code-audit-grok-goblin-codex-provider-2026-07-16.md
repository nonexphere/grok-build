# Auditoria do Codex provider e do `CODEX_100_PERCENT_GOAL.md`

Data: 2026-07-16
Escopo: implementação atual não commitada do provider Codex, prompt caching, replay Responses, autenticação multi-provider, catálogo de modelos e comparação com `~/forge/forge-responses-api`.
Modo: auditoria read-only; nenhum código de produto foi alterado.
Veredito: **não está 100% e ainda não deve ser tratado como pronto para merge/release**.

## Resumo executivo

O objetivo geral do `CODEX_100_PERCENT_GOAL.md` é correto: preservar o protocolo Responses, tornar o cache observável e verificável, manter afinidade entre sessão e credencial, e tratar refresh/401 sem corridas. O documento é bem melhor que uma lista superficial de features, mas ainda permite um falso “100%” por quatro razões:

1. mistura suporte de wire, afinidade de roteamento e prova real de cache como se fossem a mesma coisa;
2. aceita adiar a trava cross-process de refresh, embora a meta prometa correção concorrente e produção;
3. não exige que o stamp de autenticação pertença à requisição exata, apenas ao resolver/sessão;
4. usa Forge como referência ampla, embora o Forge só prove certos contratos — não prova cache real do Codex.

A implementação avançou: `phase` é capturado do SSE, commentary chega ao headless como thought, `cached_tokens` é parseado, existe chave de cache, existe binding tipado na sessão e o catálogo tem cache por credencial. Porém, os caminhos mais importantes ainda têm falhas estruturais:

- associação incorreta de `phase` ao serializar histórico;
- materialização incompleta do stream e perda possível de argumentos de function call;
- perda/reordenação de itens do protocolo Responses;
- chave de cache global `anonymous` e envio de identificadores crus;
- chave adicionada indiscriminadamente a qualquer backend Responses;
- “request-scoped stamp” que, na prática, é `last_stamp` compartilhado pelo resolver;
- refresh protegido apenas dentro do processo;
- recuperação do journal fora das travas e com erros ignorados;
- cache de modelos não atômico, sem ETag real e mascarando erros permanentes.

## Escopo e evidência

Foram inspecionados:

- `CODEX_100_PERCENT_GOAL.md`;
- `xai-grok-sampling-types`, especialmente conversão Responses, phase, chave de cache e materialização;
- `xai-grok-sampler`, especialmente collect/stream, usage e logs;
- `xai-grok-shell`, especialmente binding de provider, 401 recovery, sessão, title generation e headless;
- `xai-grok-multi-auth`, especialmente TokenManager, store/journal e cache de modelos Codex;
- `~/forge/forge-responses-api/src/providers/codex.ts` e testes de runtime localizados no repositório Forge.

O worktree está amplamente modificado por outro agente. Esta auditoria avalia o estado observado, sem atribuir autoria e sem modificar mudanças existentes.

## Veredito por requisito do goal

| Grupo | Estado | Evidência principal |
|---|---|---|
| C1 phase capture/replay | **Parcial, com bug crítico de correlação** | captura por `message_id` existe; patch de request realinha apenas os phases presentes por posição |
| C2 commentary UX | **Parcialmente provado** | stream diferencia commentary; headless trata `AgentThoughtChunk`; falta prova integrada TUI/headless/live |
| C3 uma mensagem por wire message | **Parcial** | mensagens ficam separadas, mas function calls são anexadas à assistant anterior e perdem posição de sibling |
| C4 materialização completa | **Falha** | escolhe vetor por tamanho, não valida/mescla por índice; argumentos delta não alimentam materialização |
| C5 fidelidade de histórico | **Falha** | phase pode ir à mensagem errada; MCP é descartado; outros variants são ignorados; ordem de tool call muda |
| PC1 campo `prompt_cache_key` | **Implementado no wire** | tipo, builder e `CreateResponse` existem |
| PC2 chave estável por sessão | **Parcial e inseguro** | sessão principal é estável; ausência de IDs vira uma chave global; IDs crus são enviados/logados |
| PC3 prefixo estável | **Não provado** | não há snapshot/hash integrado cobrindo instructions, tools, schemas, compaction e caminhos auxiliares |
| PC4 `previous_response_id` | **Política parcial** | código sempre envia `None`; full replay é uma opção válida, mas faltam contrato/testes/documentação explícita |
| PC5 retention | **Omitido corretamente por ora** | `None`; falta discovery/live evidence antes de expor |
| PC6 afinidade de conta | **Parcial** | catálogo e sessão carregam credencial; binding ainda é reconstruído de hints e o stamp não é por requisição |
| PC7 `cached_tokens` E2E | **Parcial** | parser e logs existem; falta propagação comprovada em todas as superfícies e correlação segura |
| PC8 prova live | **Ausente** | não foi encontrado probe Codex de dois turnos com hit obrigatório e controles |
| PC9 accounting stream | **Parcial** | evento usage é parseado; faltam testes de evento ausente/tardio e prova em saída estruturada |
| PC10 compaction | **Ausente** | chave não muda por geração e não há política/invariante implementada |
| PC11 todos os caminhos | **Falha** | title generation não fornece session/conv/agent id e cai em `anonymous`; cobertura completa não existe |
| PC12 docs/runbook | **Ausente/incompleto** | o goal descreve intenção, não um runbook operacional validado |
| A1 request stamp | **Falha estrutural** | `RequestScopedStamp` é um único `Mutex<Option<_>>` no resolver compartilhado e guarda o último valor |
| A2 single-flight | **Parcial** | lock in-process existe; criação do manager tem check-then-insert; não há refresh lock cross-process |
| A3 journal | **Falha de robustez** | recovery ocorre no construtor sem locks e o erro é descartado; remoção do journal ignora erro |
| A4 binding tipado | **Parcial** | tipo existe na sessão, mas é reconstruído de model id/header/base URL e pode ser substituído a cada config rebuild |
| M7 model cache | **Parcial** | TTL/per-credential/fallback existem; persistência, ETag, error policy e stale signaling estão incompletos |

## Findings prioritários

### AUD-CODEX-001 — `phase` pode ser anexado à assistant errada

- Severidade: **Critical**
- Confiança: **Confirmed**
- Evidência: `crates/codegen/xai-grok-sampling-types/src/conversation.rs:2244`
- Causa: `patch_assistant_phases_on_request_body` primeiro remove todos os `None` com `filter_map`, depois distribui os phases restantes sequencialmente sobre todas as mensagens assistant serializadas.
- Reprodução lógica: histórico `[assistant phase=None, assistant phase=commentary]` gera lista `[commentary]`; o patch atribui commentary à primeira assistant.
- Impacto: altera semântica do histórico, quebra replay fiel, muda o prefixo cacheável e pode fazer o modelo interpretar final como commentary ou vice-versa.
- Correção exigida: correlacionar cada item serializado com seu item de origem, preservando slots `None`, ou serializar diretamente uma representação Responses que comporte `phase`. Adicionar teste com assistants intercaladas, incluindo uma sem phase antes e depois das phased.

### AUD-CODEX-002 — materialização do stream pode perder itens e argumentos de ferramenta

- Severidade: **Critical**
- Confiança: **Confirmed para merge por tamanho; Likely para perda de argumentos em stream real**
- Evidência: `conversation.rs:2286`; `xai-grok-sampler/src/stream/responses.rs`; referência Forge `src/providers/codex.ts:399-506`.
- Causa: o Goblin escolhe `completed_output` se seu comprimento for maior/igual ao map, sem comparar índices, IDs ou conteúdo. Quando escolhe o map, retorna apenas seus values. Deltas de `response.function_call_arguments.*` são encaminhados à UI, mas não são acumulados na estrutura usada para materialização.
- Impacto: replay incompleto, chamadas com `{}`/argumentos vazios, duplicação ou desaparecimento de output items, falhas de tool execution no turno seguinte.
- Referência útil: o Forge mantém `outputItems`, `outputIndexByItemId` e `functionArguments`, acumulando delta/done antes de materializar. Mesmo o Forge só substitui output final quando ausente; o goal do Goblin corretamente pede algo mais forte: merge e diagnóstico de inconsistência.
- Correção exigida: um materializador por `output_index`/item id, com acumuladores por variant, merge determinístico entre added/delta/done/completed e invariantes que falhem de modo observável.

### AUD-CODEX-003 — conversão Responses perde ou reordena itens

- Severidade: **High**
- Confiança: **Confirmed**
- Evidência: `conversation.rs:2030-2158`.
- Causa: `FunctionCall` vira `tool_calls` na assistant mais recente; `McpCall` só incrementa contador; variants restantes caem em `_ => {}`; conteúdo de message que não seja `OutputText` é ignorado.
- Impacto: não preserva a sequência original `commentary → reasoning → function_call → final`; perde MCP e conteúdo desconhecido/refusal; full replay deixa de ser full.
- Correção exigida: modelar os siblings Responses explicitamente ou manter um wire-history canônico separado da projeção de UI/chat. Unknown variants não podem desaparecer silenciosamente.

### AUD-CODEX-004 — chave de prompt cache global e vazamento de identificador

- Severidade: **High**
- Confiança: **Confirmed**
- Evidência: `conversation.rs:2202-2238`; sampler `client.rs:1196-1200,1351-1355,1925,1962`.
- Causa: sem session/conv/agent id, a chave é sempre `goblin-sess-anonymous`; com id, envia o valor cru prefixado. O sampler também registra a chave completa.
- Caminho concreto esquecido: `xai-grok-shell/src/session/helpers/session_summary.rs:74-115` cria request de título sem qualquer id, portanto compartilha `anonymous` com toda geração auxiliar equivalente.
- Impacto: afinidade de cache/conta entre sessões não relacionadas, baixa taxa de hit, interferência operacional e exposição de IDs locais ao upstream/logs.
- Correção exigida: gerar chave opaca e estável (hash/HMAC com namespace, provider, credential e sessão), nunca usar fallback global; para requests realmente sem identidade, omitir a chave ou criar identidade de operação apropriada. Logs devem usar hash curto/redigido, não o valor completo.

### AUD-CODEX-005 — `prompt_cache_key` é injetado em todo backend Responses

- Severidade: **High**
- Confiança: **Confirmed no código; impacto depende do backend**
- Evidência: sampler `client.rs:1925,1962`.
- Causa: `ensure_prompt_cache_key` roda nos caminhos Responses genéricos, sem capability/provider gate.
- Impacto: backends xAI/custom/OpenResponses incompatíveis podem receber campo não suportado; uma feature Codex altera contratos de providers alheios.
- Correção exigida: capability explícita no binding/provider, não heurística de URL, e teste de não regressão para Responses não-Codex.

### AUD-CODEX-006 — o “request-scoped stamp” continua sendo last-wins

- Severidade: **Critical**
- Confiança: **Confirmed**
- Evidência: `xai-grok-multi-auth/src/request_stamp.rs:1-51`; `xai-grok-shell/src/auth/multi_provider_resolve.rs:352-410`.
- Causa: há um `Mutex<Option<SentCredentialStamp>>` dentro de `MultiProviderBearerResolver`. Esse resolver é guardado na sessão e reutilizado. Requisições simultâneas ou uma resolução adicional substituem o stamp anterior. O teste existente prova apenas que **dois holders diferentes** não interferem; não prova duas requisições no mesmo holder.
- Impacto: um 401 pode ser recuperado com generation/account stamp de outra requisição. Isso é exatamente o problema que A1 declara resolvido.
- Correção exigida: o contrato de resolução deve retornar bearer + stamp/lease e anexá-lo ao objeto da requisição/in-flight attempt. A resposta/erro precisa carregar essa identidade até recovery. `BearerResolver::current_bearer() -> Option<String>` é insuficiente para esse requisito.

### AUD-CODEX-007 — single-flight de refresh não é cross-process e o registry tem corrida

- Severidade: **Critical para refresh token rotativo**
- Confiança: **Confirmed**
- Evidência: `xai-grok-multi-auth/src/token_manager.rs:98-280`; `token_resolve.rs:31-52`.
- Causa: TokenManager usa apenas locks Tokio locais. `CredentialStore::acquire_lock` existe, mas não é usado ao redor de refresh/reload/CAS. `make_store_and_manager` faz get seguido de insert no DashMap, permitindo duas criações concorrentes no mesmo processo.
- Impacto: dois processos podem consumir simultaneamente o mesmo refresh token rotativo; um deles pode invalidar o outro. Mesmo no processo, a janela de criação pode formar managers com locks distintos.
- Correção exigida: `DashMap::entry`; lock cross-process por credencial durante refresh e 401 recovery, reload após adquirir lock e CAS. Isso não pode ficar em R5 se o claim for “100%/production-ready”.

### AUD-CODEX-008 — journal não é recuperado sob as mesmas travas e falha silenciosamente

- Severidade: **High**
- Confiança: **Confirmed**
- Evidência: `xai-grok-multi-auth/src/store/file.rs:31-45`; `store/metadata.rs:132-180`.
- Causa: `FileCredentialStore::new` chama `recover_pending_txn` antes de adquirir locks e ignora o resultado. A remoção do journal após commit/recovery também ignora erros.
- Impacto: dois processos podem recuperar/escrever simultaneamente; corrupção ou erro de permissão pode permanecer invisível; estado de credencial pode divergir entre metadata e secret.
- Correção exigida: recovery lazy/async sob locks canônicos, erro fail-loud/quarentena, remoção durável e testes multiprocess/crash points.

### AUD-CODEX-009 — binding tipado existe, mas ainda depende de hints frágeis

- Severidade: **High**
- Confiança: **Confirmed**
- Evidência: `xai-grok-shell/src/auth/multi_provider_resolve.rs:157-270`; `sampler_turn.rs:231-330`.
- Causa: a sessão tem `MultiProviderSessionAuth`, mas `reconstruct_full_config` volta a derivá-lo de model id, `x-goblin-credential-id`, `ChatGPT-Account-ID` ou `base_url.contains(...)`, e substitui o valor guardado. O slug de wire não carrega necessariamente a key de catálogo.
- Impacto: risco de account drift, false positive de endpoint Codex e quebra da afinidade que o prompt cache pressupõe.
- Correção exigida: persistir `ModelBinding` como estado autoritativo na seleção da sessão/turno e transportar explicitamente ao sampler. Headers e URL devem ser efeitos derivados, nunca a fonte de identidade.

### AUD-CODEX-010 — cache de modelos é funcional, mas não production-grade

- Severidade: **Medium/High**
- Confiança: **Confirmed**
- Evidência: `providers/codex/model_cache.rs`; `providers/codex/models.rs`.
- Problemas:
  - `save_cache` usa `std::fs::write` não atômico, ignora erros e não força 0600;
  - corrupção vira cache ausente sem diagnóstico;
  - ETag é armazenado, mas fetch sempre retorna `None` e não envia `If-None-Match`;
  - qualquer `Err(String)` permite stale/bundled fallback, inclusive 401/403/identity mismatch;
  - `stale` e `from_bundled` somem ao converter para `ModelCatalog`;
  - fallback hardcodes modelos/capacidades/context windows sem versionamento ou fonte;
  - I/O síncrono acontece em caminho async.
- Impacto: catálogo truncado/corrupto, credencial inválida mascarada como sucesso, UI incapaz de sinalizar stale e modelos fallback possivelmente não autorizados.
- Correção exigida: write atomic/secure, erro tipado e policy transitório vs permanente, ETag real, source/stale no contrato público e fallback versionado/testado.

### AUD-CODEX-011 — system/developer hoist ainda é lossy e baseado em URL

- Severidade: **High**
- Confiança: **Confirmed**
- Evidência: conversão/hoist no sampler e detecção `chatgpt.com`/`/codex`.
- Causa: conteúdo não textual de system/developer é descartado e os papéis são colapsados em instructions; a ativação usa substring de URL.
- Impacto: alteração silenciosa do prompt e do prefixo cacheável, inclusive em endpoints custom que coincidam com a substring.
- Observação: o Forge também faz hoist textual; ele é referência de interoperabilidade, não prova de correção para conteúdo multimodal/unknown.
- Correção exigida: capability tipada e política explícita para conteúdo não representável (rejeitar com erro ou preservar em wire model adequado).

### AUD-CODEX-012 — ausência de prova empírica do prompt cache Codex

- Severidade: **High (release gate)**
- Confiança: **Confirmed**
- Evidência: nenhum probe Codex equivalente ao PC8 foi encontrado. O script citado no Forge verifica cache Groq; os testes de Codex encontrados provam stickiness/forwarding, não `cached_tokens > 0`.
- Impacto: todos os componentes podem compilar e ainda assim o produto nunca obter cache hit.
- Correção exigida: probe live gated, com turn 1/turn 2, prefixo grande e estável, mesma credencial e chave, captura de `cached_tokens`, controle negativo por mutação precoce e teste de compaction. Artefato deve redigir IDs/tokens.

## Prompt cache: modelo mental correto

O goal deve separar quatro contratos independentes:

1. **Prefix caching do provider:** depende principalmente da igualdade do prefixo tokenizado elegível e dos limites/políticas do upstream.
2. **`prompt_cache_key`:** é um hint de roteamento/afinidade; não substitui prefix equality e não deve ser descrito como condição universal para cache hit.
3. **Afinidade de credencial/conta:** a mesma sessão deve continuar na mesma identidade quando isso é parte do contrato Codex. A key não corrige um binding errado.
4. **Replay/state:** full replay com `store=false` e `previous_response_id=None` é uma política válida e robusta. Server-side chain é outra política e deve ser feature separada.

“Byte-stable prefix” é uma boa disciplina de engenharia, mas o critério externo real é estabilidade do conteúdo/tokenização recebido pelo provider. O documento deve exigir serialização determinística, snapshots estruturais e hash diagnóstico do prefixo, sem prometer que igualdade de bytes locais por si só garante hit.

## O que o Forge realmente valida

O Forge é uma boa referência para:

- encaminhar `prompt_cache_key`;
- manter afinidade de uma key à mesma conta no runtime multi-account;
- full replay com `store=false` e remoção de `previous_response_id` no upstream;
- preservar reasoning encrypted content, commentary, custom tools e output items;
- acumular argumentos de function call durante SSE e materializar output quando o final não o contém.

O Forge **não** deve ser usado como evidência de:

- hit real de prompt cache Codex;
- necessidade universal de `prompt_cache_key` para caching;
- correção de cache após compaction;
- request-scoped auth stamp;
- refresh cross-process;
- merge perfeito entre snapshot final parcial e itens streamed.

O script `verify-groq-prompt-cache.ts` é específico de Groq e prova cache por prefixo sem `prompt_cache_key`; ele reforça, na verdade, que key e prefix caching são conceitos separados.

## Mudanças necessárias no `CODEX_100_PERCENT_GOAL.md`

### Correções de definição

1. Renomear “100% Codex” para uma definição mensurável, por exemplo “Codex production-ready”, e proibir conclusão por porcentagem informal.
2. Separar gates: wire fidelity, auth identity, refresh concurrency, prompt cache, model discovery e UX.
3. Tornar R5 (cross-process refresh) obrigatório antes do claim final; pode ser adiado apenas para um milestone explicitamente não-production.
4. Redefinir A1: “stamp/lease ligado ao request attempt exato”, com teste concorrente no mesmo resolver/sessão.
5. Redefinir PC2: chave opaca, sem PII/IDs crus, namespace por provider/credential/session, sem fallback global.
6. Redefinir PC3: estabilidade do prefixo estruturado/tokenizável; byte determinism como ferramenta de diagnóstico.
7. Redefinir PC6: binding tipado autoritativo precede headers, URL e cache key.
8. Dividir PC8 em prova de forwarding/stickiness e prova live de `cached_tokens`; uma não substitui a outra.
9. Adicionar capability gate para campos Responses por provider.
10. Adicionar matriz de variants Responses: message content, reasoning, function/custom/MCP/web/code interpreter, unknown item e refusal.

### Ordem recomendada

1. Corrigir identidade/binding e request-attempt auth stamp.
2. Implementar single-flight cross-process e journal recovery seguro.
3. Corrigir wire model, phase correlation e materializer.
4. Definir política full replay/chain e compaction generation.
5. Implementar chave opaca/capability-gated e prefix diagnostics.
6. Fechar usage propagation e observabilidade redigida.
7. Executar probe live com controles.
8. Endurecer cache de modelos e docs.
9. Só então executar gates integrados e declarar conclusão.

## Testes e validações

Executado nesta auditoria:

- `cargo test -p xai-grok-multi-auth model_cache --lib`
  - resultado: **6 passed**, 0 failed; warnings de imports/variáveis não usados.
- dois comandos com filtro `--exact` para testes de prompt key/phase:
  - resultado: **0 testes executados**, 282 filtrados; portanto não contam como prova.
- `git diff --check`
  - resultado: **falhou** com dezenas de ocorrências de trailing whitespace no diff atual, inclusive em sampling types/chat state/shell.

Não executado:

- suíte completa do workspace: o worktree está sendo alterado/compilado por outro agente e a suíte é ampla; uma execução agora não provaria estabilidade terminal.
- probe live Codex: exige credencial real e artefato seguro; não foi encontrado harness pronto no estado auditado.
- testes multiprocess/crash: não existem como gate consolidado no estado observado.

## Gate de conclusão recomendado

O provider só deve receber PASS quando houver evidência para todos os itens abaixo:

- replay round-trip preserva ordem, IDs, phase, reasoning e todos os tool variants;
- stream truncado/parcial é materializado ou falha de modo explícito, nunca silenciosamente;
- duas requisições concorrentes na mesma sessão conservam stamps distintos por attempt;
- dois processos concorrentes produzem um único refresh upstream;
- crash em cada etapa do journal recupera ou falha loud sem misturar identidade;
- chave de cache é opaca, estável, capability-gated e distinta por sessão/conta;
- title, subagent, compaction, retry e collect/stream têm política comprovada;
- live probe mostra `cached_tokens > 0` no segundo turno e queda nos controles negativos;
- auth errors permanentes não são mascarados por stale/bundled model catalog;
- `git diff --check`, fmt, clippy/check e suítes relevantes passam no snapshot final.

## Conclusão

O plano faz sentido como direção, mas ainda não é um contrato seguro de “100%”. A implementação atual contém progresso real e vários componentes úteis, porém os invariantes centrais — fidelidade de replay, identidade por request, refresh cross-process e prova live de cache — permanecem contraditos ou ausentes. O risco maior não é apenas “cache não bater”; é o sistema reenviar histórico semanticamente alterado e recuperar um 401 usando a identidade temporal errada. Esses pontos devem bloquear o claim final e a liberação do provider como estável.
