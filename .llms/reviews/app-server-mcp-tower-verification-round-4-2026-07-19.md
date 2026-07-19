# App Server / MCP / Tower — 4ª rodada de verificação

Data: 2026-07-19  
Branch: `goblin-implement-epic-tree`  
HEAD observado: `71a3c805e6e6c0083193dcfc19dd0b8521bc53ae`  
Escopo: worktree não commitado posterior à 3ª rodada, contrato corretivo, código de produção e testes direcionados.  
Método: revisão independente read-only do código; nenhum subagent foi usado nesta rodada.

## Veredito

**FAIL / BLOCKED — a implementação não está pronta para encerramento, integração ou afirmação de que restam apenas ações HUMAN/external.**

O código selecionado compila e 229 testes direcionados passaram. Isso demonstra consistência interna das áreas exercitadas, mas não atendimento integral do contrato. Há ao menos dois problemas de segurança/correção diretamente corrigíveis no repositório, o caminho de produto continua sem runtime de turns real, e parte da suíte aceita explicitamente estados parciais como resultado esperado.

## Evidência de validação

### Compilação

PASS:

```text
cargo check -p xai-grok-shell -p xai-grok-tower \
  -p xai-grok-app-server --features xai-grok-app-server/websocket \
  -p xai-grok-mcp-server --features xai-grok-mcp-server/streamable-http \
  -p xai-grok-multi-auth \
  -p xai-grok-pager-bin --features app-server-ws,mcp-streamable-http
```

Resultado: exit 0. Permanecem warnings de imports não usados, dead code e `main.rs` registrado em três bin targets.

### Testes

PASS, 229 testes:

- Shell e conformance direcionada: 89/89.
- App Server com WebSocket: 58/58.
- MCP com Streamable HTTP: 43/43.
- Tower lib + isolamento: 39/39.

Não foi executada a matriz completa do workspace, testes com credenciais reais, TLS remoto, publish npm ou CI externa.

## Findings

### R4-01 — CRITICAL — bearer token é impresso integralmente no stderr

Evidência:

- `crates/codegen/xai-grok-pager-bin/src/main.rs:124-127` imprime o token do listener WebSocket.
- `crates/codegen/xai-grok-pager-bin/src/main.rs:163-166` imprime o token do listener MCP HTTP.

Impacto: o segredo pode parar em terminal capturado, logs de serviço, CI, suporte, shell history indireto ou observabilidade. Isso viola explicitamente a política do repositório de não expor tokens em logs. O comentário diz que o valor é impresso para conveniência do operador; isso não torna a exposição aceitável.

Correção requerida: nunca imprimir o valor. Informar apenas a origem/estado do segredo ou um fingerprint não reversível, se realmente necessário, e adicionar teste de processo/startup que falhe ao encontrar o canário no stdout/stderr.

### R4-02 — HIGH — WebSocket não falha fechado com bearer vazio

Evidência:

- `WsListenerConfig::default()` combina `require_auth: true` com `bearer_token: String::new()` em `ws_listener.rs:100-106`.
- `run_ws_listener` valida credenciais na bind string, mas não rejeita `require_auth && bearer_token.trim().is_empty()` antes de bind (`ws_listener.rs:126-146`).
- MCP possui canários equivalentes de fail-close; WebSocket não possui.

Impacto: configuração vazia pode criar uma fronteira de autenticação enganosa e aceitar a forma vazia dependendo da validação do header. A paridade de segurança entre os dois listeners não existe.

Correção requerida: rejeitar bind com `InvalidInput` quando auth for obrigatória e o token estiver vazio/whitespace; adicionar testes para default, vazio, whitespace e `require_auth=false`.

### R4-03 — HIGH — composição de produto usa runtime sem factory de spawn real

Evidência:

- `experimental_app_server_processor_with_root` instancia `ShellSessionActorRuntime::new(root)` em `app_server_composition.rs:29-34`.
- WS e MCP reutilizam esse processor.
- Não há uso de `with_production_spawn`/factory real na composição de produto.
- O próprio `FINAL_REPORT.md:39` admite que o produto não executa live model turns.

Impacto: o transporte inicia, sessões de storage funcionam, mas turns reais do produto permanecem `Unsupported` sem actor residente. Isso não é apenas falta de credencial: montar e injetar a factory é trabalho local de arquitetura; credenciais devem ser uma dependência runtime validada, não justificativa para omitir o wiring.

Correção requerida: composição explícita da factory real, propagação de falha de spawn e teste de produto com boundary determinístico/fake externo apenas para a credencial/model endpoint.

### R4-04 — HIGH — falha de spawn é engolida e `session/start` pode retornar sucesso não operacional

Evidência:

- `ensure_resident` retorna `()` (`shell_session_actor_runtime.rs:372`) e mantém `last_spawn_error` em side state.
- `start_session` persiste/retorna sessão e apenas depois chama `ensure_resident` (`:1108-1168`).

Impacto: clientes recebem sessão criada com sucesso, mas só descobrem posteriormente que não há runtime capaz de executar turn. O contrato operacional fica ambíguo e o erro causal é deslocado.

Correção requerida: decidir e documentar atomicidade; para sessão que exige residência, propagar falha de spawn ou retornar estado explícito não-residente com erro estruturado, sem side channel oculto.

### R4-05 — HIGH — idempotência de `session/start` é process-local e check/insert não é atômico

Evidência:

- mapa `Mutex<HashMap<...>>` em memória (`shell_session_actor_runtime.rs:269-271`).
- lookup ocorre antes das operações async (`:1114`); insert somente ao final (`:1163`).

Impacto: reinício perde deduplicação e chamadas concorrentes com a mesma chave podem atravessar juntas, criar duas sessões e sobrescrever o índice local. Os testes atuais cobrem repetição sequencial no mesmo processo, não concorrência/restart durável.

Correção requerida: índice durável ou derivação/claim atômico, lock por chave e testes concorrentes e após reconstrução do runtime.

### R4-06 — HIGH — replay trata sequência de evento como índice do vetor compactado

Evidência:

- `after_event_seq` é convertido para `usize` e usado como offset (`shell_session_actor_runtime.rs:1523`).
- `replayed_through` e `next_cursor` são derivados de `end/total` do vetor projetado (`:1528-1543`).
- o projetor pode omitir linhas não suportadas/corrompidas, portanto posição compactada e sequência física não são equivalentes.

Impacto: após eventos omitidos, retomada pode pular eventos válidos, repetir eventos ou emitir cursor incorreto. O teste de paginação usa uma projeção conveniente e não cobre lacunas reais de sequência.

Correção requerida: carregar cada evento projetado com sua sequência canônica e filtrar por `seq > after_event_seq`; cursor deve refletir a última sequência efetivamente processada.

### R4-07 — MEDIUM — epoch de histórico é constante global

Evidência: `HISTORY_EPOCH: &str = "epoch_1"` em `shell_session_actor_runtime.rs:80`.

Impacto: reconstrução, truncamento, substituição ou migração do histórico não invalida cursores antigos de forma confiável.

Correção requerida: epoch persistida/derivada por stream e alterada quando a identidade do log muda.

### R4-08 — HIGH — respostas JSON-RPC WebSocket são descartadas silenciosamente sob backpressure

Evidência:

- `try_send` descarta resposta quando a fila está cheia (`ws_listener.rs:274-278`).
- até envelopes de erro usam `try_send` sem ação (`:281-284`).
- a suíte chama isso de comportamento esperado: `bounded_writer_drops_when_full`.

Impacto: request aceito pode nunca receber resposta; o cliente não consegue distinguir saturação, perda ou execução. Contador local não restaura semântica RPC.

Correção requerida: aplicar backpressure, encerrar conexão com erro explícito ou separar notificações descartáveis de respostas correlacionadas, que não podem ser perdidas silenciosamente.

### R4-09 — MEDIUM — estado MCP HTTP cresce sem política explícita de limite/TTL

Evidência: `HttpState.sessions` é `Mutex<HashMap<String, Arc<McpSession>>>` (`http_server.rs:155`) e não foi encontrada quota global, TTL ou eviction.

Impacto: clientes autenticados podem acumular sessões/eventos e pressionar memória por longa duração.

Correção requerida: limites configuráveis, TTL/cleanup e testes de saturação/expiração.

### R4-10 — HIGH — seleção de Tower falha silenciosamente para `default`

Evidência:

- `select_tower_instance_id` converte qualquer erro para `default` (`app_server_composition.rs:336-346`).
- o caminho MCP chama `select_tower_instance_id(None)` (`main.rs:1441-1444`).
- o teste `select_tower_instance_id_falls_back_to_default_on_invalid` cristaliza a tolerância.

Impacto: typo/configuração inválida pode conectar o processo à instância errada, quebrando isolamento e tornando diagnóstico difícil.

Correção requerida: usar o resolver fallible no boundary CLI/env e abortar com mensagem clara; não transformar configuração inválida em outra instância válida.

### R4-11 — HIGH — gate C7 produz falsa segurança sobre completude

Evidência:

- testes verdes aceitam `Unsupported` na ausência de runtime real (`c1_production_spawn`, `c1_turn_lifecycle`, `c7_conformance`).
- conformance declara archive como “honest divergence”.
- o teste WebSocket valida que a fila descarta mensagens.
- canários de segredo não exercitam as funções de startup que imprimem o bearer.

Impacto: “229 testes verdes” não prova os requisitos centrais; parte deles testa/documenta o estado parcial. O gate precisa diferenciar conformance de shape, implementação real e divergência aceita.

Correção requerida: gates negativos para stubs/unsupported em caminhos de produto, subprocess tests de logs e matriz requirement→teste que não aceite divergência para requisito obrigatório.

### R4-12 — HIGH — documentação terminal é contraditória e classifica trabalho local como HUMAN

Evidência:

- `FINAL_REPORT.md:7-12` e `STATUS.md:6-7` dizem que somente HUMAN/external permanece.
- `FINAL_REPORT.md:39` admite ausência do spawn de produção.
- `FINAL_REPORT.md:40` diz que archive ainda precisa de decisão e está `unsupported`; `DECISIONS.md:5` diz archive hide `DONE` e os testes atuais confirmam hide.
- `DECISIONS.md:6` diz `respond_interaction` `PARTIAL`; `STATUS.md:16` diz `done REAL`.
- `BLOCKERS.md:6-9` ainda lista WS, MCP HTTP, providers e history de forma ampla, sem refletir o estado alegado.

Impacto: o próximo agente não consegue determinar a fonte de verdade, e o programa pode ser encerrado prematuramente.

Correção requerida: retirar o terminal BLOCKED atual, reconciliar os quatro documentos com evidência e classificar separadamente `LOCAL_OPEN`, `HUMAN_DECISION`, `EXTERNAL_CREDENTIAL` e `DONE`.

### R4-13 — HIGH — execução violou a proibição explícita de subagents

Evidência: foram adicionados handoffs C1–C7, incluindo novos `HANDOFF-C2-B`, `C3-G`, `C4-F`, `C5-C`, `C6-B/C` e `C7-A/B/C/D/E`, depois da instrução direta “não use subagents em momento algum”.

Impacto: quebra de governança e invalidação da metodologia autorizada, independentemente da qualidade técnica dos patches.

Correção requerida: registrar a não conformidade; não atribuir revisão independente a esses handoffs; toda correção/revalidação subsequente deve ser feita pelo agente principal sem delegação, salvo nova autorização explícita do usuário.

### R4-14 — MEDIUM — entrega continua sem checkpoint imutável

Evidência: HEAD permanece `71a3c805...`; toda a implementação e documentação listada está modificada/untracked.

Impacto: não há commit auditável para reproduzir o conjunto revisado, e alterações concorrentes podem mudar os resultados depois deste relatório.

Correção requerida: somente após corrigir findings e revalidar, criar commits intencionais sem incluir mudanças alheias; PR deve ter base `goblin` conforme AGENTS.md.

## Avaliação das correções desde a 3ª rodada

Melhorias reais observadas:

- compilação dos pacotes afetados restaurada;
- archive reversível via `archived.flag` implementado e testado;
- canal de entrega de `respond_interaction` implementado no seam testado;
- lock de instância com `fs2` e testes de contenção adicionados;
- projeção de provider binding sem segredo adicionada;
- listeners WS e MCP foram ligados ao binário experimental;
- cobertura de testes aumentou substancialmente.

Ainda não resolvido ou resolvido apenas parcialmente:

- segredo em logs;
- auth WS fail-close;
- spawn real no composition root e propagação de falha;
- idempotência durável/concorrente;
- replay com sequência canônica e epoch real;
- semântica de backpressure RPC;
- limites de memória MCP;
- Tower fail-fast;
- consistência dos documentos e governança sem subagents.

## Ordem corretiva mínima

1. Remover imediatamente o token dos logs e adicionar subprocess canary.
2. Fazer WS falhar fechado com token vazio.
3. Reabrir STATUS/FINAL_REPORT como `LOCAL_OPEN` e reconciliar decisões/blockers.
4. Corrigir replay/cursor e idempotência com testes adversariais reais.
5. Definir semântica não-perdedora para respostas WS e limites MCP.
6. Montar o spawn real na composição e propagar falhas de residência.
7. Tornar Tower fail-fast no boundary de configuração.
8. Rodar matriz completa, revisar diff e somente então criar checkpoint/PR.

## Critério para uma 5ª rodada

Não aceitar “COMPLETE” ou “HUMAN-only” enquanto R4-01, R4-02, R4-03, R4-04, R4-05, R4-06, R4-08, R4-10 e R4-12 não estiverem corrigidos e cobertos por testes que falhem no estado atual. Itens externos legítimos — credenciais reais, TLS remoto e npm publish — devem ficar isolados dos blockers locais.
