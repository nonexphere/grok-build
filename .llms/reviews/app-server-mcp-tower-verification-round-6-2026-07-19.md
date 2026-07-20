# 6ª rodada — verificação + correção contínua

Data: 2026-07-19  
Branch: `goblin-implement-epic-tree`  
Baseline: `d0ea385c9e0e7335da9f831a3031fca6e3f2bd67`  
Método: `implementation-loop`, agente principal, sem subagents.

## Resultado

Foi encontrado e corrigido um defeito residual no mecanismo de idempotência persistente:

1. O nome temporário do claim usava apenas o PID do processo. Dois runtimes independentes no mesmo processo podiam compartilhar o mesmo arquivo temporário e sobrescrever o conteúdo preparado do outro antes do claim exclusivo.
2. O nome final usava `DefaultHasher`, que não é primitivo de identidade resistente a colisões nem contrato estável entre processos.

Correção aplicada em `crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs`:

- nome final agora usa digest BLAKE3 da chave;
- temporário agora usa UUID v7 por tentativa;
- algoritmo de claim `Won/Existing` e limpeza do loser foram preservados.
- se o flush do metadata do diretório falhar após a publicação, o arquivo final
  também é removido, evitando claim órfão durante o rollback.

## Evidência

PASS:

- `cargo test -p xai-grok-shell --test r4_runtime_correctness --test r5_runtime_correctness --no-fail-fast` — 7/7.
- `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http --no-fail-fast` — 38/38.
- `cargo check -p xai-grok-shell -p xai-grok-mcp-server --features xai-grok-mcp-server/streamable-http -p xai-grok-app-server --features xai-grok-app-server/websocket -p xai-grok-pager-bin --features app-server-ws,mcp-streamable-http` — exit 0.
- `git diff --check` — passou.

`cargo fmt --all -- --check` não passa por diffs preexistentes em `xai-chat-state` e `xai-fast-worktree`, fora do escopo desta correção. O arquivo tocado também contém formatação histórica não introduzida por este patch; não foi reform formatado para evitar modificar trabalho alheio.

## Estado dos findings R5 após esta rodada

- R5-01: produto não injeta echo; turns reais continuam dependentes da factory canônica e credenciais externas.
- R5-02: erros hard propagam; `unsupported` permanece explicitamente storage-only.
- R5-03: claim cross-runtime corrigido e agora também resistente a colisão/temporário compartilhado.
- R5-04..R5-07: correções anteriores permanecem cobertas pelos testes R5/MCP.
- R5-08: startup não imprime token nem fingerprint.
- R5-09: hub compartilhado com parked ask-user path; integração completa depende do actor canônico product-wired.
- R5-10: documentos ainda devem ser tratados como ledger de estado, não como prova independente de implementação.
- R5-11: existe checkpoint anterior; esta correção permanece dirty e não foi commitada sem autorização.

## Pendências honestas

- Factory `spawn_session_on_thread` completa no caminho de produto com credenciais reais.
- TLS e threat acceptance para bind remoto.
- npm publish.
- Limpeza de warnings/formatação fora do escopo e sem relação causal com este patch.
- Commit/PR desta correção somente mediante autorização.

## Veredito

**LOCAL PATCH GREEN, programa ainda não COMPLETE.** A correção desta rodada é pequena, testada e limitada ao claim idempotente. Não há base para declarar que credenciais, TLS, integração do actor real ou publicação foram resolvidos.

## R6-01 — correção adicional nesta continuação

Foi encontrado um segundo bug local em `respond_interaction`: o pending interaction era removido antes de confirmar que o delivery hub existia e continha um oneshot. Em erro `unsupported` ou `interaction_not_deliverable`, uma tentativa subsequente não poderia mais entregar a decisão.

Correção: o registro pendente é restaurado quando não há hub ou sender. First-answer-wins continua valendo quando o sender existe e a decisão é efetivamente encaminhada.

Teste adicionado: `interaction_facade_not_deliverable_keeps_pending_for_retry` e `interaction_facade_missing_hub_keeps_pending_for_retry`.

Validação adicional inicial: `cargo test -p xai-grok-shell --test c6_respond_interaction --no-fail-fast` — **12/12**; a suíte foi ampliada pelo R6-03 e está atualmente em **13/13**.

## R6-02 — comentário/invariante de durabilidade do claim

Na revisão de implementação não óbvia, o claim persistente foi fortalecido: o
arquivo temporário é sincronizado (`sync_all`) antes de publicar o hard-link
exclusivo. Em caso de falha no sync, o temporário é removido antes de retornar
erro. O comentário no código explica por que exclusividade do nome não basta
para durabilidade após crash.

Validação adicional: os testes R4/R5 de idempotência continuaram verdes após a
alteração.

## R6-03 — sender fechado não pode ser confirmado como entrega

A revisão comportamental encontrou que `oneshot::Sender::send` tinha seu erro
ignorado. Se o receiver do parked future já tivesse sido cancelado, a API
retornava sucesso e removia o pending interaction sem entregar a decisão.

Correção: falha de `send` retorna `interaction_not_deliverable` e restaura o
pending para retry. Teste adicionado:
`interaction_facade_closed_receiver_keeps_pending_for_retry`.

Validação atualizada: `c6_respond_interaction` — **13/13**.

## Auditoria final desta continuação

Também foram executados:

- App Server WebSocket: 59/59.
- Tower lib + isolamento: 39/39.
- BYOK providers: 17/17.
- Provider binding projection: 10/10.
- Pager startup secret canary: 3/3 (um por bin target).
- `cargo check -p xai-grok-shell`: exit 0.
- `git diff --check`: passou.

Foi revisado código não óbvio nos claims, rollback de interactions, composição
sem echo, fail-close, TTL/cancelamento e buffer de eventos. Não foi encontrada
outra falha local confirmada nesta continuação.

## R6-04 — snapshot sintético duplicado no polling MCP

Foi encontrado um defeito no caminho `pull_facade_events`: o replay do Tower
retorna legitimamente o snapshot sintético de `seq=0` enquanto o cursor está em
zero. Em uma sessão silenciosa, cada polling SSE repetia esse snapshot e o
anexava ao buffer MCP, causando eventos duplicados e crescimento artificial do
buffer.

Correção em `transport/http_server.rs`: cada `McpSession` mantém um marcador
atômico local (`snapshot_replayed`) e ignora apenas repetições posteriores do
evento `SessionChanged` sintético, preservando eventos reais subsequentes.
Foi adicionado o teste
`repeated_quiet_tools_call_does_not_duplicate_snapshot_event`.

Validação:

- teste regressivo isolado — passou;
- suíte `streamable_http` — **31/31**;
- warning preexistente de import não usado em `transport/stdio.rs` permanece,
  sem relação causal com esta correção.

## R6-05 — corrida entre polling POST e SSE

A revisão adversarial do R6-04 encontrou uma segunda janela: o POST e o loop
SSE podem chamar `pull_facade_events` ao mesmo tempo. Mesmo com o marcador do
snapshot, ambos poderiam ler o mesmo cursor e anexar eventos reais duplicados.

Correção: `McpSession` agora serializa cada ciclo de leitura/replay/anexação/
avanço de cursor com `replay_pull_lock`. O teste
`concurrent_quiet_tools_calls_do_not_duplicate_replay_page` exerce duas
requisições idempotentes em paralelo e confirma que nenhum evento adicional é
anexado.

Validação atualizada: suíte `streamable_http` — **32/32**.

## Estado final da rodada 6

Após R6-05, os checks diretamente afetados foram repetidos:

- `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http --no-fail-fast` — **32/32**;
- `cargo check -p xai-grok-mcp-server --features streamable-http` — passou;
- `cargo test -p xai-grok-shell --test c6_respond_interaction --no-fail-fast` — **13/13**;
- `git diff --check` — passou.

`rustfmt --check` localizado continua reportando formatação histórica nos dois
arquivos MCP (e `cargo fmt --all -- --check` também possui diferenças
preexistentes em outros pacotes). Não foi aplicado format sweep porque isso
alteraria código não relacionado e trabalho existente. A limitação está
explicitamente registrada, sem tratar o check como verde.

O worktree continua deliberadamente dirty, sem commit, push ou PR. O veredito
permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**: integração do
actor canônico com credenciais reais, TLS remoto, publicação npm e checkpoint
autorizado continuam HUMAN/EXTERNAL.

## R6-06 — gap de replay em SSE já aberto

A revisão do stream de longa duração encontrou que `events_after(after)` podia
retornar cursor expirado depois que o GET SSE já estava conectado (por exemplo,
quando o limite do buffer removia os eventos antigos). O producer simplesmente
encerrava, sem sinalizar ao cliente que havia perdido um gap.

Correção: o producer agora envia um evento `resumption_error` com o cursor
expirado, último evento e limite mínimo retido antes de terminar. O cliente
pode então reconectar e ressincronizar explicitamente.

Teste adicionado: `open_sse_emits_resumption_error_when_buffer_expires`.
Validação atualizada: suíte `streamable_http` — **33/33**.

Após essa alteração, `cargo check -p xai-grok-mcp-server --features
streamable-http` e `git diff --check` também passaram. O único aviso relevante
continua sendo o import não usado preexistente em `transport/stdio.rs`.

## R6-07 — turn concluído mantido como ativo no MCP

Na auditoria de ciclo de vida, `dispatch_jsonrpc` armazenava `turnId` sempre
que um resultado o continha. `tower_agent_send` e `steer_active` retornam
`state: completed` junto com o identificador histórico; isso deixava o turn
marcado como ativo e podia provocar `interrupt_turn` indevido em DELETE ou TTL.

Correção: resultados terminais limpam `active_turn_id`; somente estados não
terminais podem registrar um turn para cancelamento. Teste adicionado:
`completed_tool_result_does_not_leave_an_active_turn`.

Validação atualizada: suíte `streamable_http` — **34/34**.

Checks pós-R6-07: `cargo check -p xai-grok-mcp-server --features
streamable-http` passou e `git diff --check` passou. O worktree segue dirty e
nenhum commit/push/PR foi criado.

## R6-08 — epoch do replay MCP não era propagado

A auditoria de identidade de histórico confirmou que o adaptador MCP sempre
enviava `history_epoch: None` ao Tower. Assim, depois de um rewrite/restart que
rotasse o epoch, o cursor poderia continuar sem validação e misturar eventos de
histórias diferentes.

Correção:

- `McpSession` captura o `history_epoch` do primeiro `SessionChanged` recebido;
- cada replay subsequente envia esse epoch ao Tower;
- `epoch_mismatch`, `resync_required` e `cursor_too_old` são propagados ao SSE
  como `resumption_error` em vez de serem descartados silenciosamente.

Teste adicionado: `open_sse_emits_resumption_error_on_tower_epoch_mismatch`, com
runtime de teste que força a troca de epoch após o snapshot.

Validação atualizada: suíte `streamable_http` — **35/35**.

Após R6-08, `cargo check -p xai-grok-mcp-server --features streamable-http` e
`git diff --check` passaram novamente. O warning de `process_mcp_stdio_batch`
não usado permanece preexistente e não foi alterado nesta rodada.

## R6-09 — rebind de Tower carregava cursor/epoch antigos

Depois de introduzir a validação de epoch, a revisão encontrou que um MCP
session que recebesse um novo `tower_session_id` mantinha `last_replayed_seq`,
`history_epoch` e o marcador de snapshot da sessão anterior. O próximo replay
falhava ou misturava identidade de histórico.

Correção: `bind_tower_session` reseta cursor, epoch e snapshot somente quando o
ID Tower realmente muda; rebind para o mesmo ID permanece idempotente.
Teste adicionado: `tower_session_rebind_resets_mcp_replay_identity`.

Validação atualizada: suíte `streamable_http` — **36/36**.

Checks pós-R6-09: `cargo check -p xai-grok-mcp-server --features
streamable-http` passou e `git diff --check` passou.

## R6-10 — buffer SSE ainda misturava eventos após rebind

A revisão de rebind mostrou que resetar somente cursor/epoch não bastava: o
buffer de eventos MCP ainda continha eventos da sessão Tower anterior e os
reentregava ao SSE após o novo bind.

Correção: `bind_tower_session` agora é serializado com o lock de replay e limpa
o buffer transportado, atualiza a janela mínima e notifica consumidores quando
a identidade Tower muda. O teste R6-09 também verifica que o workspace antigo
não reaparece no stream.

Validação: suíte `streamable_http` — **36/36**.

Checks pós-R6-10: `cargo check -p xai-grok-mcp-server --features
streamable-http` passou e `git diff --check` passou.

## R6-11 — SSE aberto não era invalidado durante rebind

Mesmo com o buffer limpo no rebind, um producer SSE já aberto mantinha seu
cursor local antigo. Após a troca de identidade ele poderia continuar aceitando
eventos novos sem informar que a continuidade havia sido quebrada.

Correção: `McpSession` mantém uma geração monotônica de bind; cada SSE captura a
geração na abertura e emite `resumption_error` (`session_rebound`) quando ela
muda, encerrando o stream antigo. Teste adicionado:
`open_sse_is_invalidated_when_tower_session_rebinds`.

Validação atualizada: suíte `streamable_http` — **37/37**.

Checks pós-R6-11: `cargo check -p xai-grok-mcp-server --features
streamable-http` passou e `git diff --check` passou.

## R6-12 — fallback de claim ignorava falha de durabilidade

Na revisão final de persistência, o caminho fallback de `claim_idempotency`
chamava `f.sync_all().ok()`. Isso podia retornar `Won` mesmo quando o arquivo
publicado não havia sido sincronizado com armazenamento estável.

Correção: a falha de `sync_all` agora é propagada e o claim não é confirmado
como vencedor. Os testes de idempotência e replay foram repetidos:

- `r4_runtime_correctness` — **4/4**;
- `r5_runtime_correctness` — **3/3**.

Checks pós-R6-12: `cargo check -p xai-grok-shell -p xai-grok-mcp-server
--features xai-grok-mcp-server/streamable-http` passou e `git diff --check`
passou. O worktree contém artefatos adicionais não relacionados (incluindo
`.agents/evidence/`, `human-product-test/` e documentação de arquitetura); eles
foram preservados e não foram revertidos.

`cargo fmt --all -- --check` continua falhando por diferenças históricas e/ou
de outros pacotes (por exemplo `xai-chat-state`, `xai-fast-worktree`,
`xai-grok-app-server` e `xai-grok-tower-tools`), não introduzidas por esta
rodada. Nenhum format sweep amplo foi aplicado para não sobrescrever trabalho
alheio.

## R6-13 — active turn atravessava rebind de Tower

O rebind já resetava replay e buffer, mas ainda podia carregar `active_turn_id`
da sessão Tower anterior. Um DELETE/TTL posterior poderia então tentar
interromper esse identificador contra a nova sessão.

Correção: o rebind agora limpa `active_turn_id` junto com os demais estados
escopados à identidade Tower. O teste `tower_session_rebind_resets_mcp_replay_identity`
foi ampliado para cobrir essa invariante.

Validação: suíte `streamable_http` — **37/37**.

Checks pós-R6-13: `cargo check -p xai-grok-mcp-server --features
streamable-http` passou e `git diff --check` passou.

## R6-14 — entrada de diretório do claim sem flush após publicação

A revisão de crash/restart identificou a última lacuna de durabilidade: mesmo
com o corpo do claim sincronizado, a entrada de nome criada por hard-link ou
exclusive-create podia não estar persistida no diretório após uma queda.

Correção: o caminho vencedor agora executa `sync_all` também no diretório de
claims antes de inserir o resultado no cache em memória. Falha nessa etapa
retorna `internal_error` em vez de confirmar a operação como durável.

Validação:

- `r4_runtime_correctness` — **4/4**;
- `r5_runtime_correctness` — **3/3**;
- `cargo check -p xai-grok-shell` — passou.

Checks pós-R6-14: `cargo check -p xai-grok-mcp-server --features
streamable-http` passou e `git diff --check` passou.

## R6-15 — fsync de diretório precisava ser portável

A correção R6-14 usava `File::open(directory)` diretamente. O Shell também
declara dependências Windows, onde não há equivalente portátil no `std` para
abrir e sincronizar um handle de diretório; o código poderia transformar todo
claim em `internal_error` nessa plataforma.

Correção: o flush de metadata foi encapsulado em `sync_idempotency_directory`:
Unix sincroniza o diretório; outras plataformas preservam o `sync_all` do
arquivo e não falham artificialmente por ausência desse primitive.

Validação: `cargo check -p xai-grok-shell` passou e os testes R4/R5 de claim
continuaram verdes.

Checks pós-R6-15: `cargo check -p xai-grok-mcp-server --features
streamable-http` passou e `git diff --check` passou.

## R6-16 — filtro de snapshot descartava mudanças reais de sessão

A revisão adversarial do R6-04 revelou que o filtro usado para impedir
duplicação descartava qualquer `SessionChanged` depois do primeiro snapshot.
Isso também eliminava atualizações legítimas de revisão/status da sessão.

Correção: o MCP guarda o payload do último snapshot e ignora somente uma
repetição idêntica, em cursor zero; uma mudança real de `SessionChanged` é
preservada e passa a ser o novo snapshot conhecido. O estado é limpo durante
rebind.

Validação: suíte `streamable_http` — **37/37**; `cargo check` MCP passou.

Checks pós-R6-16: `git diff --check` passou e `cargo check -p
xai-grok-mcp-server --features streamable-http` passou.

## R6-17 — identidade sintética era sobrescrita por evento real

A revisão de R6-16 encontrou uma segunda sutileza: se a mesma página contivesse
o snapshot sintético e um `SessionChanged` real, o código poderia substituir a
identidade do snapshot pela mudança real. Uma repetição posterior do snapshot
seria então anexada novamente.

Correção: a identidade usada para deduplicação fica fixada no primeiro payload
observado e só é limpa em rebind; eventos reais posteriores não a substituem.

Validação: suíte `streamable_http` — **37/37**.

Checks pós-R6-17: `git diff --check` passou e `cargo check -p
xai-grok-mcp-server --features streamable-http` passou.

## Matriz de regressão final repetida

Após R6-25, a matriz mínima completa permanece verde; a suíte MCP foi ampliada
com a cobertura de autorização do DELETE:

- App Server WebSocket/lib: **59/59**;
- Tower lib + isolamento: **29/29 + 10/10**;
- BYOK providers: **17/17**;
- Shell provider binding projection: **10/10**;
- Shell `respond_interaction`: **13/13**;
- MCP Streamable HTTP: **38/38**.

Os comandos incorretos tentados inicialmente (`--test ws_listener` e
`--test providers`) foram corrigidos para os targets reais (`--lib` e
`--test byok_providers`); não são falhas de implementação.

## R6-18 — erros não relacionados a epoch encerravam SSE silenciosamente

O producer SSE tratava apenas `epoch_mismatch`, `resync_required` e
`cursor_too_old` como resync. Erros como `session_not_found` ou
`runtime_unavailable` simplesmente fechavam o stream, sem que o cliente pudesse
identificar a perda de continuidade.

Correção: toda falha de replay agora produz `resumption_error`; somente os
códigos de resync expõem a mensagem específica do runtime. Outros erros usam
mensagem genérica para evitar vazamento de detalhes internos.

Validação: suíte `streamable_http` — **37/37**; `cargo check` MCP passou.

Checks pós-R6-18: `git diff --check` passou e `cargo check -p
xai-grok-mcp-server --features streamable-http` passou.

## R6-19 — corrida na leitura inicial do SSE ainda encerrava silenciosamente

Uma auditoria de conclusão encontrou uma janela residual no GET SSE: a
validação de `Last-Event-ID` ocorre antes do primeiro `events_after`, portanto o
buffer podia expirar entre as duas operações. Nesse caso, o ramo inicial ainda
fazia `return` sem enviar qualquer evento, embora o ramo de polling já emitisse
`resumption_error`.

Correção em `transport/http_server.rs`: a leitura inicial agora emite o mesmo
contrato explícito de `resumption_error` usado no polling. O formato foi
centralizado em `replay_resumption_error`, com comentário explicando que as
duas janelas precisam sinalizar o gap de forma idêntica e sem detalhes internos.

Teste de contrato adicionado para validar tipo, cursor, janela retida e ausência
de token no payload.

Validação adicional:

- `cargo test -p xai-grok-mcp-server --features streamable-http --lib --no-fail-fast` — **17/17**;
- `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http --no-fail-fast` — **37/37**;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**. A
integração do actor canônico com credenciais reais, TLS remoto, publicação npm
e checkpoint autorizado continuam fora da autoridade local desta sessão.

## R6-20 — escrita de history epoch ocultava falhas de persistência

A auditoria contra o requisito de invalidação após restart encontrou que
`ensure_history_epoch` e `rotate_history_epoch` descartavam o resultado de
`std::fs::write`. Uma falha de disco podia deixar o arquivo antigo/ausente e,
mesmo assim, devolver um epoch novo ao consumidor, permitindo mistura de
cursores após reinício.

Correção em `shell_session_actor_runtime.rs`:

- epoch é escrito em temporário UUID, sincronizado e publicado por rename;
- erros de criação, escrita, sync ou publicação viram `internal_error` explícito;
- `start_session` e `fork_session` removem a sessão parcial antes de propagar a
  falha;
- `rotate_history_epoch` agora retorna `Result<String, RuntimeError>`, evitando
  anunciar um epoch que não foi persistido.

Validação:

- `cargo test -p xai-grok-shell --test r5_runtime_correctness --no-fail-fast` — **3/3**;
- `cargo check -p xai-grok-shell` — passou;
- `git diff --check` — passou.

O veredito continua **LOCAL PATCH GREEN, programa ainda não COMPLETE**; os
bloqueios HUMAN/EXTERNAL permanecem inalterados.

## R6-21 — leitura de epoch corrompida caía silenciosamente no fallback legado

Mesmo com a escrita corrigida em R6-20, a leitura ainda tratava qualquer erro
como ausência do arquivo e devolvia `epoch_1`. Um sidecar vazio, truncado ou
inacessível poderia portanto aceitar um cursor incompatível em vez de falhar
fechado.

Correção: `history_epoch_for` agora distingue `NotFound` (único caso que usa o
fallback para sessões legadas) de arquivo vazio ou outros erros de I/O, que
retornam `internal_error`. Todos os caminhos de sessão, fork, leitura e replay
propagam o resultado.

Validação:

- `cargo check -p xai-grok-shell` — passou;
- `cargo test -p xai-grok-shell --test r5_runtime_correctness --no-fail-fast` — **3/3**;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-35 — matriz de aceitação contra o contrato completo

| Invariante | Estado atual | Evidência / limite |
|---|---|---|
| Auth fail-closed | **DONE local** | testes App Server/MCP de bearer, query e bind vazio |
| Tokens/fingerprints fora de logs e payloads | **DONE local** | canaries e testes de redaction; credenciais reais não foram usadas |
| Turns reais somente com actor canônico | **LOCAL_OPEN** | `unsupported` honesto sem factory; wiring product-wired permanece aberto |
| Idempotência cross-runtime | **DONE local** | R5 4/4, claim BLAKE3/UUID/sync/rollback |
| Epoch/cursor após rewrite | **DONE local** | Shell R5, MCP epoch mismatch e rebind |
| Replay sem gaps/página falsa | **DONE local** | App/Tower replay tests e MCP 38/38 |
| Limite MCP e cancelamento TTL/disconnect | **DONE local** | buffer cap, TTL e disconnect tests |
| `respond_interaction` em parked future real | **LOCAL_OPEN** | delivery hub implementado; auto-register do actor canônico ainda aberto |
| Documentação de estados | **DONE local** | handoff e ledger distinguem parcial/HUMAN/EXTERNAL |
| TLS remoto, credenciais reais, npm, checkpoint | **EXTERNAL/HUMAN** | requer autoridade/ambiente externo; nenhum foi simulado como PASS |

Esta matriz impede que o green local seja interpretado como prontidão de
produção. O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-36 — reavaliação das pendências HUMAN/EXTERNAL

As pendências foram reavaliadas para distinguir trabalho local possível de
ações que exigem estado externo ou decisão humana:

- **Factory canônica / actor real:** não é seguro inventar os argumentos de
  `spawn_session_on_thread`; eles dependem de credenciais, `GatewaySender`,
  `ModelsManager`, `PersistenceHandle`, plugins, permissões e configuração de
  produto que não estão disponíveis na composição App Server. O caminho local
  permanece fail-closed com `unsupported`.
- **Credenciais reais:** não podem ser criadas, inferidas ou expostas pelo
  agente; qualquer smoke real sem segredo seria falso.
- **TLS remoto/threat acceptance:** código local mantém bind remoto marcado como
  experimental/unsafe; aprovação de certificado, proxy confiável e ameaça
  aceita exigem decisão/infraestrutura externa.
- **npm:** pack e workflows já existem e não requerem token; publicação exige
  organização/conta e `NPM_TOKEN`, portanto não foi simulada.
- **Commit/checkpoint:** permanece sem autorização explícita e sem alteração de
  histórico.

Não há ação local segura adicional que feche esses itens sem inventar contexto,
enfraquecer fail-closed ou expor segredo. O veredito permanece **LOCAL PATCH
GREEN, programa ainda não COMPLETE**.

Validação local adicional: `node npm/scripts/assert-package-identity.mjs` —
**OK** (`@brasalabs/grok-oss` 0.2.102, 5 plataformas). A publicação continua
dependente de `NPM_TOKEN` e autorização do registry.

## R6-34 — matriz completa repetida após as correções finais

Após R6-31, R6-32 e R6-33, a matriz mínima foi executada novamente no
worktree atual:

- App Server WebSocket/lib: **59/59**;
- Tower lib + isolamento: **29/29 + 10/10**;
- Shell `respond_interaction` + runtime R5: **13/13 + 4/4**;
- MCP Streamable HTTP: **38/38**.

Também passaram `cargo check` nos crates afetados e `git diff --check`. Os
warnings observados são preexistentes (targets duplicados do pager, imports e
funções não usadas) e não foram mascarados por esta rodada.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-33 — handoff arquitetural afirmava que App Server/MCP ainda não existiam

A revisão documental encontrou claims históricos em
`docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md` dizendo que a implementação
de App Server não havia começado e que não havia MCP server. O worktree atual
já contém implementação parcial de App Server, MCP Streamable HTTP e isolamento
Tower; o gap real continua sendo o wiring product-wired do actor canônico,
credenciais reais e validação operacional.

Correção: o handoff agora distingue implementação local parcial de integração
de produto ainda aberta, sem converter nenhuma pendência externa em DONE.

Validação: revisão textual com `rg` dos claims de status e `git diff --check`.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-32 — SSE repassava mensagem interna de replay

O caminho SSE que tratava `epoch_mismatch`, `resync_required` e
`cursor_too_old` usava diretamente `RuntimeError.message`. Essas mensagens
podem conter caminhos de storage ou detalhes internos desnecessários para o
cliente.

Correção: mensagens SSE agora são estáticas e seguras por código de erro; o
cliente continua recebendo o código e os cursores necessários para reconectar,
sem receber detalhes do runtime.

Validação:

- contrato de replay/sanitização — **3/3**;
- `streamable_http` — **38/38**;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-31 — falha de replay durante POST era descartada silenciosamente

Os chamadores de `pull_facade_events` após `tools/call` ignoravam o erro com
`let _ =`. Assim, o JSON-RPC podia retornar normalmente enquanto o consumidor
SSE não recebia qualquer indicação de que seu cursor não havia sido atualizado.

Correção: falhas agora geram evento transport-level `resumption_error` no
buffer MCP, contendo apenas o código estável e coordenadas de cursor; a
mensagem interna do runtime não é exposta.

Teste unitário adicionado para confirmar o evento e a ausência de mensagem
privada. Validação adicional:

- contrato de erro MCP — **2/2**;
- `streamable_http` — **38/38**;
- `cargo check -p xai-grok-mcp-server --features streamable-http` — passou;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-30 — leitura falível ainda ocorria depois de `Won`

Na auditoria transacional de `start_session`, o claim era publicado antes de
uma nova leitura do `history_epoch`. Mesmo com rollback, outro runtime poderia
observar o vencedor durante a janela de limpeza se essa leitura falhasse.

Correção: `ensure_history_epoch` agora devolve o epoch já validado (lendo o
sidecar existente ou retornando o valor recém-publicado), e `start_session`
reutiliza esse valor após `Won`. Não há mais I/O falível entre a publicação do
claim e a projeção da resposta.

Validação:

- `cargo check -p xai-grok-shell` — passou;
- `cargo test -p xai-grok-shell --test r5_runtime_correctness --no-fail-fast` — **4/4**;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-29 — falha no fsync do claim deixava publicação parcial

Na auditoria de conclusão, o caminho de claim já havia criado o arquivo final
quando `sync_directory_metadata` falhava. O rollback removia apenas o
temporário; isso podia deixar no diretório um claim que o chamador trataria
como falha e cuja sessão seria removida.

Correção: quando a sincronização do diretório falha, o arquivo final publicado
também é removido antes de retornar `internal_error`, preservando a transação
claim + sessão.

Validação:

- `cargo test -p xai-grok-shell --test r5_runtime_correctness --no-fail-fast` — **4/4**;
- `cargo check -p xai-grok-shell` — passou;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-28 — publicação do history epoch não sincronizava metadata do diretório

A revisão de durabilidade observou que o arquivo temporário do epoch era
sincronizado, mas o rename podia permanecer apenas no cache de metadata do
diretório após um crash. Isso deixava a nova identidade ausente no restart.

Correção: o helper foi generalizado para `sync_directory_metadata` e agora é
executado após publicar claims e epochs. Unix faz `fsync` do diretório; outras
plataformas preservam o `sync_all` do arquivo sem falhar artificialmente.

Validação:

- R5 runtime: **4/4**;
- teste C7 de fork: **1/1**;
- `cargo check -p xai-grok-shell` — passou;
- `git diff --check` — passou.

O veredito continua **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-27 — fork podia deixar destino parcial após erro de cópia/leitura

A revisão transacional ampliada encontrou que `fork_session` propagava falhas
de `copy_session_data` ou `load_summary` sem remover o destino já criado. Isso
deixava diretórios órfãos que poderiam aparecer em listagens posteriores.

Correção: ambos os erros agora tentam remover o destino antes de retornar; o
fork só prossegue para binding e rotação de epoch depois que a cópia e o
summary estão íntegros.

Validação:

- teste C7 de fork: **1/1**;
- `cargo check -p xai-grok-shell` — passou;
- `git diff --check` — passou.

O veredito continua **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-24 — falhas pós-spawn deixavam estado especulativo no runtime

Após R6-23, a revisão de rollback encontrou dois caminhos adicionais: erro ao
persistir `provider_binding` e erro ao publicar o claim depois de um spawn já
bem-sucedido. Ambos podiam deixar sessão/actor especulativos vivos, mesmo sem
uma resposta válida ao chamador.

Correção: esses erros agora removem a sessão persistida e, no segundo caso,
também removem o residente especulativo antes de propagar a falha. O caminho
normal `Won`/`Existing` permanece inalterado.

Validação:

- `cargo test -p xai-grok-shell --test r5_runtime_correctness --no-fail-fast` — **4/4**;
- `cargo check -p xai-grok-shell` — passou;
- `git diff --check` — passou.

O veredito continua **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-25 — DELETE não aplicava o binding da sessão negociada

A revisão de autorização encontrou que `DELETE /mcp` verificava apenas o bearer
global e removia o ID diretamente. Diferentemente de GET/POST, não validava o
fingerprint do bearer usado no `initialize` nem o `tower_instance_id` da sessão.

Correção: DELETE agora resolve a sessão por `lookup_session` antes de remover,
reutilizando as mesmas validações de bearer/instância e mantendo a interrupção
do turno somente após a remoção autorizada.

Teste black-box adicionado: uma sessão substituída por fingerprint estrangeiro
recebe **401** e continua presente no mapa de sessões.

Validação:

- `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http --no-fail-fast` — **38/38**;
- `cargo check -p xai-grok-mcp-server --features streamable-http` — passou;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-22 — erro inicial de replay precisava preservar cancelamento por desconexão

Na revisão final do diff de R6-19, o ramo que sinaliza cursor expirado durante
a leitura inicial do SSE enviava `resumption_error`, mas ignorava a falha desse
envio. Se o cliente já tivesse desconectado, o turno ativo poderia permanecer
sem a interrupção que os outros caminhos de desconexão executam.

Correção: falha no envio inicial agora chama `interrupt_active_turn`, alinhando
esse caminho com expiração durante polling, envio de eventos e DELETE.

Validação:

- `cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http --no-fail-fast` — **37/37**;
- `cargo test -p xai-grok-mcp-server --features streamable-http --lib --no-fail-fast` — **17/17**;
- `git diff --check` — passou.

O veredito continua **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## R6-23 — claim era publicado antes do spawn e podia expor rollback concorrente

A revisão transacional de R5-03 encontrou uma janela que os testes de sucesso
não cobriam: `start_session` publicava o claim antes de `ensure_resident`. Se o
spawn falhasse enquanto outro runtime lesse o claim, esse segundo runtime podia
observar uma sessão vencedora que o primeiro ainda removeria no rollback.

Correção:

- o spawn (ou `unsupported` storage-only explícito) ocorre antes da publicação
  do claim;
- falha de spawn remove somente a sessão especulativa e nunca precisa apagar um
  claim já visível;
- um loser que já iniciou um actor remove seu residente especulativo antes de
  apagar a linha persistida e carregar o vencedor;
- teste bloqueia o spawn e confirma que nenhum claim aparece enquanto a decisão
  de residência está pendente.

Validação:

- `cargo test -p xai-grok-shell --test r5_runtime_correctness --no-fail-fast` — **4/4**;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.

## Índice cronológico consolidado da continuação R6

Para evitar ambiguidade causada pelas inserções incrementais deste ledger, a
ordem canônica dos findings finais é:

`R6-19` leitura inicial SSE → `R6-20` escrita de epoch → `R6-21` leitura de
epoch corrompida → `R6-22` cancelamento no erro inicial SSE → `R6-23` claim após
spawn → `R6-24` limpeza pós-spawn → `R6-25` autorização no DELETE → `R6-26`
rotação de epoch em fork → `R6-27` limpeza de fork parcial → `R6-28` fsync de
metadata do diretório → `R6-29` rollback de claim após falha de fsync → `R6-30`
rollback pós-claim durante projeção → `R6-31` erro de replay no POST convertido
em evento de resumption → `R6-32` sanitização de mensagens de replay no SSE.
→ `R6-33` claims históricos do handoff alinhados ao estado parcial atual →
`R6-34` matriz completa repetida após as correções finais → `R6-35` matriz de
aceitação contra o contrato completo → `R6-36` reavaliação das pendências
HUMAN/EXTERNAL → `R6-37` human-product-test do binário real → `R6-38`
instalação/PATH isolados → `R6-39` TUI com tmux portátil.

## R6-37 — human-product-test do binário real

A skill `human-product-test` foi aplicada sem subagents. O binário foi
rebuildado e validado em home isolado:

- L_build: **proven**; `grok-oss` executável, versão `0.2.102 (d0ea385)`;
- L1 CLI: **proven**; help, version e flag inválida sem panic;
- L2 no-auth: **proven para honestidade**, mas inferência ao vivo **blocked**
  por ausência de credenciais; o processo retornou `Not signed in`;
- L3 TUI: **proven** em sessão tmux real, com binário portátil extraído no
  scratch root e sem sudo; o pane mostrou o fluxo de sign-in e `Ctrl-C` encerrou
  a sessão;
- L4 auth: **skipped/blocked** sem autorização e credenciais;
- npm pack dry-run meta/Linux x64: **proven**.

Evidência durável: `.agents/evidence/product-qa/2026-07-19-grok-oss-full/REPORT.md`.
O verdict de produto é **BLOCKED/PARTIAL**, não PASS, até haver observabilidade
PTY e autorização de auth/live inference. A observabilidade TUI foi resolvida
localmente em `R6-39`; permanece bloqueada somente a autorização de auth/live.

## R6-38 — instalação/PATH isolados do binário real

A camada L5 da skill também foi executada em prefixo temporário, sem alterar
`~/.local`:

- `PROFILE=debug PREFIX=/tmp/.../prefix ./scripts/install-grok-oss.sh` — exit 0;
- launcher isolado encontrado no PATH temporário;
- launcher executou o binário rebuildado e reportou a versão/SHA esperados;
- home scratch criou layout inicial sem usar o home real do operador.

Evidência: `.agents/evidence/product-qa/2026-07-19-grok-oss-full/l5-install.txt`
e `l1-home-layout.txt`. O verdict geral de produto permanece BLOCKED/PARTIAL
por L3/L4, não por instalação.

As seções individuais preservam suas evidências e comandos; este índice é a
referência de sequência para a revisão final.

## R6-39 — TUI observável com tmux portátil

Para eliminar o bloqueio de observabilidade sem instalar pacotes no sistema,
foi baixado o pacote Debian `tmux` e sua dependência `libjemalloc2` para o
diretório scratch da QA, ambos extraídos sem sudo. Uma sessão real foi criada
com socket dentro de `GROK_OSS_HOME`; o pane capturado mostrou o logo, código de
sign-in, estado “Waiting for approval...” e a affordance `ctrl+q quit`. Um
`Ctrl-C` posterior encerrou a sessão sem panic/backtrace.

Evidências: `.agents/evidence/product-qa/2026-07-19-grok-oss-full/l3-tui-pane-portable.txt`
e `l3-tui-pane-portable-after-ctrl-c.txt`.

Validação: `tmux 3.5a` executado via `LD_LIBRARY_PATH` no scratch; sessão
encerrada e socket temporário removido pelo próprio ciclo de QA. L3 agora é
**LOCAL PASS**. L4/live inference continua **EXTERNAL/HUMAN**, pois exige
credenciais e autorização explícitas.

## R6-26 — fork ignorava falha ao remover epoch herdado

A revisão do caminho de fork encontrou `remove_file(history_epoch)` tratado
como best-effort. Se a remoção falhasse, `ensure_history_epoch` via o sidecar
existente e o fork reutilizava o epoch da origem, aceitando potencialmente
cursores antigos.

Correção: `fork_session` agora chama `rotate_history_epoch`, que grava um epoch
novo em temporário sincronizado e publica atomicamente. Falhas são propagadas
como `internal_error` e a sessão copiada é removida antes do retorno.

Validação:

- `cargo test -p xai-grok-shell --test c7_conformance c7_conformance_fork_session_creates_distinct_session_with_workspace --no-fail-fast` — **1/1**;
- `cargo check -p xai-grok-shell` — passou;
- `git diff --check` — passou.

O veredito permanece **LOCAL PATCH GREEN, programa ainda não COMPLETE**.
