# App Server / MCP / Tower — 5ª rodada de verificação

Data: 2026-07-19  
Branch: `goblin-implement-epic-tree`  
HEAD: `71a3c805e6e6c0083193dcfc19dd0b8521bc53ae`  
Base de comparação: findings R4-01..R4-14 da 4ª rodada.  
Método: revisão independente do agente principal, sem subagents e sem editar código de produção.

## Veredito

**FAIL — a afirmação “resolveu tudo” não é sustentada.**

Houve correções substantivas, mas a matriz correta é:

- **FIXED:** 5 findings (`R4-01`, `R4-02`, `R4-08`, `R4-10`, `R4-13` como reconhecimento processual).
- **PARTIAL:** 7 findings (`R4-03`, `R4-04`, `R4-05`, `R4-06`, `R4-07`, `R4-09`, `R4-11`).
- **OPEN:** 2 findings (`R4-12`, `R4-14`).

Além disso, esta rodada encontrou quatro problemas concretos não cobertos pelo gate corretivo: idempotência cross-runtime incorreta, `history_epoch` sem identidade/rotação real, retenção ilimitada de eventos MCP e expiração MCP sem cancelamento do turn ativo.

## Validação executada

PASS:

- `cargo check` dos pacotes Shell, Tower, App Server WS, MCP HTTP, multi-auth e pager-bin.
- Shell: 39 testes direcionados (`r4_runtime_correctness`, production spawn, interaction, C7).
- App Server WebSocket: 59 testes.
- MCP HTTP: 44 testes.
- Composition root em três bin targets: 60 testes.
- Total observado nesta rodada: **202 testes, 202 passaram**.

Warnings permanecem em imports/dead code e no registro do mesmo `main.rs` em três targets. A matriz completa do workspace e integrações reais externas não foram executadas.

## Matriz de revalidação R4

| Finding | Estado R5 | Evidência resumida |
|---|---|---|
| R4-01 segredo em logs | **FIXED** | raw token removido; startup mostra fingerprint |
| R4-02 WS fail-close | **FIXED** | bind recusa token vazio/whitespace; teste existe |
| R4-03 spawn real | **PARTIAL** | composição injeta um actor de echo local, não `spawn_session_on_thread` |
| R4-04 erro de spawn | **PARTIAL** | hard errors propagam; `unsupported` e re-residência idempotente continuam engolidos |
| R4-05 idempotência | **PARTIAL** | durável no restart sequencial; corrida cross-runtime permanece incorreta |
| R4-06 replay | **PARTIAL** | filtro por seq melhorou; teste não cria lacuna e paginação exata tem falso `next_cursor` |
| R4-07 epoch | **PARTIAL** | arquivo existe, mas sempre contém `epoch_1` e nunca é rotacionado |
| R4-08 WS backpressure | **FIXED** | replies usam `send` com timeout e conexão fecha na saturação |
| R4-09 MCP bounds | **PARTIAL** | max sessions/TTL existem; eventos continuam ilimitados e eviction não cancela turn |
| R4-10 Tower fail-fast | **FIXED** | selector agora retorna `Result` e boundary propaga erro |
| R4-11 gates fortes | **PARTIAL** | cobertura cresceu, mas valida echo e possui testes vacuosos/stale |
| R4-12 documentos | **OPEN** | documentos ainda se contradizem e chamam residual local de opcional |
| R4-13 subagents | **FIXED processual** | violação anterior reconhecida; esta rodada não usou subagents |
| R4-14 checkpoint | **OPEN** | HEAD não mudou; implementação inteira permanece dirty/untracked |

## Findings bloqueantes

### R5-01 — HIGH / Confirmed — “produção” foi substituída por um modelo echo local

Evidência:

- `experimental_local_turn_spawn` cria um `mpsc` local e responde `experimental-local-reply-{prompt_id}` (`shell_session_actor_runtime.rs:187-271`).
- A composição de produto injeta esse factory (`app_server_composition.rs:31-38`).
- Não existe chamada de `spawn_session_on_thread` nessa composição; as únicas chamadas reais permanecem nos caminhos preexistentes do agent.

Isso torna turns superficialmente operacionais, mas não executa modelo, ferramentas, permissões, MCP, provider binding ou o `SessionActor` canônico. É um simulador no caminho de produto e conflita com a regra do repositório de não apresentar mocks/atalhos como comportamento real.

Correção: integrar a factory canônica com dependências carregadas do ambiente/credential store. Ausência de credencial deve produzir erro runtime explícito; não justifica colocar echo no produto.

### R5-02 — HIGH / Confirmed — propagação de spawn ainda não é consistente

Evidência:

- `start_session` ainda trata `unsupported` como sucesso storage-only (`shell_session_actor_runtime.rs:1363-1368`).
- No caminho idempotente, `ensure_resident` é completamente ignorado (`:1328-1329`).
- `resume_session` também faz re-residência best-effort e retorna sucesso.

Impacto: o mesmo request pode retornar uma sessão operacional ou não operacional conforme factory/estado, sem campo de estado que expresse isso. O finding R4-04 não está DONE.

### R5-03 — HIGH / Confirmed — claim idempotente falha entre runtimes/processos concorrentes

O lock por chave é local a uma instância do runtime. Duas instâncias com o mesmo root podem ambas criar sessões antes do `create_new` do claim. Quando a segunda perde a criação do arquivo:

- tenta ler o claim enquanto o primeiro processo ainda pode estar escrevendo;
- se a leitura/parsing falhar, o branch `AlreadyExists` cai adiante e retorna sucesso;
- mesmo se ler o mesmo digest, `store_idempotency_claim` retorna `Ok(())`, mas o caller continua retornando sua própria sessão recém-criada, não a sessão vencedora.

Os testes usam um único `Arc<ShellSessionActorRuntime>` para concorrência e runtimes sequenciais para restart; não testam dois runtimes concorrentes.

Correção: claim deve ser a primeira autoridade atômica e retornar explicitamente `Won(session_id)` ou `Existing(session_id)`. O loser deve carregar/devolver a sessão vencedora e remover qualquer sessão especulativa; escrita deve ser atômica e durável.

### R5-04 — MEDIUM / Confirmed — `history_epoch` continua constante e sem rotação

`ensure_history_epoch` escreve literalmente `epoch_1`; `history_epoch_for` devolve o mesmo fallback; não existe rotação quando histórico é reescrito, truncado, reconstruído ou substituído (`shell_session_actor_runtime.rs:608-627`).

Um arquivo por sessão evita apenas uma constante global em memória; não fornece identidade de stream nem invalida cursores antigos. R4-07 permanece parcial.

Correção: gerar epoch única por stream e rotacioná-la junto das operações que alteram a identidade/sequenciamento do histórico, com teste de cursor anterior rejeitado após rewrite.

### R5-05 — HIGH / Confirmed — gate de replay não testa o bug alegado

`r4_replay_filters_by_canonical_event_seq_not_vector_index` cria um turn normal, sem inserir linha omitida/corrompida ou uma lacuna de sequência. A asserção final aceita tanto página vazia quanto qualquer `replayed_through >= through`, portanto não prova que um evento posterior à lacuna foi entregue.

Adicionalmente, `more = page.len() == REPLAY_PAGE_SIZE` em `shell_session_actor_runtime.rs:1765-1770` emite `next_cursor` quando há exatamente uma página completa e nenhum evento restante.

Correção: fixture com evento físico não projetável entre dois eventos válidos; verificar IDs exatos, ausência de repetição/perda e paginações nos tamanhos `N-1`, `N` e `N+1`.

### R5-06 — HIGH / Confirmed — MCP limita sessões, mas não eventos

`McpSession.events` continua sendo `Mutex<Vec<McpSessionEvent>>`; `append_event` apenas faz `push`, sem cap, janela ou persistência (`http_server.rs:90-145`). Uma sessão mantida ativa pode crescer indefinidamente, mesmo com `max_sessions` e TTL.

Correção: buffer circular/limite por bytes e contagem, semântica explícita de cursor expirado e teste de longa duração.

### R5-07 — HIGH / Confirmed — TTL eviction remove sessão sem cancelar turn ativo

`evict_expired_sessions` usa apenas `retain` (`http_server.rs:824-834`). Diferentemente de DELETE ou disconnect do sender, não chama `interrupt_active_turn`. O produtor SSE detecta que o id sumiu e apenas sai do loop.

Impacto: expirar transporte pode deixar execução no Tower continuando sem controller/cliente.

Correção: coletar sessões expiradas fora do lock e cancelar seus turns de forma assíncrona antes/depois da remoção, com teste que observa `interrupt_turn`.

### R5-08 — MEDIUM / Confirmed — fingerprint de startup é curto e não é um hash criptográfico

`secret_fingerprint` usa `DefaultHasher` e publica apenas 32 bits (`main.rs:108-115`). Isso permite confirmação offline de tokens de baixa entropia e o algoritmo não é um contrato criptográfico/estável.

Correção preferida: não imprimir nenhum derivado do segredo. Se correlação operacional for indispensável, usar identificador aleatório independente ou HMAC com chave separada e política documentada.

### R5-09 — HIGH / Confirmed — `respond_interaction` não está ligado ao actor real

O próprio source header chama a função de PARTIAL; `ResidentHandle::from_handle` cria um delivery hub novo que o actor live não conhece; `CHANGES.md` registra que o actor de produção não registra parked oneshots. O echo local também cria mapas vazios e nunca produz interação pendente.

Logo, os testes provam apenas um seam injetado; o caminho de produto não consegue completar uma interação real. A documentação não pode simultaneamente declarar C6 done REAL.

### R5-10 — HIGH / Confirmed — reconciliação documental continua incorreta

- `FINAL_REPORT` declara R4-03/04/05/06/07/09/11/12 DONE.
- `BLOCKERS.md` admite `LOCAL-SPAWN-UPGRADE`, mas o chama de opcional.
- `DECISIONS.md` mantém `respond_interaction` como PARTIAL.
- `STATUS.md` diz `respond_interaction` REAL e todos os itens locais R4 DONE.
- C7 ainda contém teste chamado `archive_session_honest_divergence`, apesar de archive real ter sido implementado.

O estado honesto é `LOCAL_OPEN`, não “terminal complete local”.

### R5-11 — MEDIUM / Confirmed — não existe checkpoint reprodutível

HEAD continua no mesmo commit da 4ª rodada e todas as correções continuam no worktree compartilhado. Este relatório verifica um snapshot mutável, não uma entrega imutável.

## Correções reais confirmadas

Não devem ser perdidas em nova implementação:

- raw bearer removido dos logs;
- WS recusa bearer vazio;
- respostas JSON-RPC não são mais silenciosamente descartadas;
- Tower selector propaga configuração inválida;
- replay agora tenta usar `event_seq` canônica;
- idempotência possui persistência e lock intra-runtime;
- MCP possui limite de sessões e TTL básico;
- testes de composição WS/MCP e turn local passaram.

## Critério mínimo para declarar resolvido

1. Remover o echo do caminho de produto ou classificá-lo explicitamente como fixture/demo, nunca production spawn.
2. Fazer todos os caminhos que exigem residência propagarem estado/erro consistentemente.
3. Corrigir e testar idempotência com dois runtimes concorrentes.
4. Implementar epoch única e rotação verificável.
5. Fortalecer replay com lacuna real e paginação exata.
6. Limitar eventos MCP e cancelar turns na expiração.
7. Ligar `respond_interaction` aos parked futures do actor canônico.
8. Reconciliar ledgers como `LOCAL_OPEN` até esses itens passarem.
9. Rodar matriz completa e criar checkpoint intencional antes do gate final.
