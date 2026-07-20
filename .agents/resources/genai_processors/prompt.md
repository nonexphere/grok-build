input : Deveremos analisar o repo: https://github.com/google-gemini/genai-processors

Nosso objetivo é considerar como reescrever essa lib em rust e integrar em noss harness:
https://github.com/nonexphere/grok-build/tree/goblin-implement-epic-tree

A gente deve implementar um motor para pipelines/graphs ; Reescrevendo a logica atual de conversação apartir dessa novo framework que nos permitirá criar varios tipos de pipelines e graficos.

Crie um prompt bem bom para enviar em um outro chat ele fazer todo esse cruzamento.


prompt gerado:
Atue como um **principal engineer especializado em Rust, runtimes assíncronos, sistemas agentic e motores de execução de grafos**.

Sua missão é analisar profundamente dois repositórios e produzir uma proposta concreta para portar os conceitos do **GenAI Processors** para Rust e integrá-los ao runtime de conversação do **Goblin/Grok Build**.

## Repositórios

### Framework de referência

* Repositório: https://github.com/google-gemini/genai-processors
* Branch: `main`

### Harness de destino

* Repositório: https://github.com/nonexphere/grok-build
* Branch obrigatória: `goblin-implement-epic-tree`
* A branch `main` não é a linha de produto do fork.
* Considere também as regras descritas em `README.md`, `GOBLIN.md`, `AGENTS.md` e demais documentos de arquitetura da branch.

## Objetivo principal

Queremos criar, em Rust, um **motor reutilizável de pipelines e grafos assíncronos para aplicações de IA**.

Esse motor deverá ser usado para reescrever progressivamente a lógica atual de conversação do Grok Build, permitindo representar como componentes e grafos:

* processamento de prompts;
* conteúdo multimodal;
* chamadas de modelo;
* streaming de respostas;
* tool calls e tool results;
* hooks;
* permissões;
* retries;
* autenticação por request;
* compactação;
* subagentes;
* interjeições;
* notificações;
* persistência;
* pós-processamento;
* gates de continuação;
* workflows lineares, condicionais, paralelos e cíclicos.

Não queremos uma tradução literal de Python para Rust. Queremos identificar os princípios bons do GenAI Processors, corrigir suas limitações quando necessário e adaptá-los aos contratos reais do harness.

## Regra fundamental

**Não proponha uma arquitetura abstrata antes de inspecionar o código real dos dois repositórios.**

Use as ferramentas disponíveis para navegar pelos repositórios. Leia implementações, testes, documentação e exemplos. Não se limite aos READMEs.

Para cada afirmação importante, indique:

* repositório;
* branch ou commit analisado;
* caminho do arquivo;
* símbolo, trait, struct ou função relevante.

Quando alguma conclusão for uma inferência, marque-a explicitamente como inferência.

## Parte 1 — Análise do GenAI Processors

Mapeie detalhadamente a semântica da biblioteca Python. No mínimo, inspecione:

* `README.md`
* `llms.txt`
* `genai_processors/processor.py`
* `genai_processors/content_api.py`
* `genai_processors/streams.py`
* `genai_processors/context.py`
* `genai_processors/switch.py`
* `genai_processors/map_processor.py`
* abstrações de cache;
* tracing;
* function calling;
* modelos turn-based;
* realtime/live processors;
* `examples/chat.py`;
* exemplos de research agents e pipelines compostos.

Explique:

1. A diferença semântica entre `Processor` e `PartProcessor`.
2. O padrão de interface dupla:

   * interface de implementação do processor;
   * interface conveniente para o consumidor.
3. Como `ProcessorStream`, `ProcessorContent` e `ProcessorPart` funcionam.
4. Como conteúdo multimodal, MIME type, role, metadata e substreams são representados.
5. Como funcionam:

   * chain;
   * parallel;
   * parallel concat;
   * map;
   * filter;
   * switch/router;
   * split;
   * merge;
   * concat;
   * gather;
   * passthrough.
6. As garantias e não garantias de ordenação.
7. Como erros e cancelamento atravessam processors concorrentes.
8. Como o contexto e os task groups implementam structured concurrency.
9. Como substreams reservadas carregam debug, status e eventos de UI fora do fluxo principal.
10. Como tracing e cache keys são propagados.
11. Quais componentes são específicos do SDK Gemini e não devem ser copiados para o core.
12. Quais partes da API Python dependem de características que não se traduzem bem para Rust.
13. Limitações existentes, especialmente:

    * uso de filas ilimitadas;
    * backpressure;
    * mutabilidade compartilhada;
    * comportamento de fan-out;
    * determinismo;
    * tratamento de ciclos;
    * isolamento de falhas;
    * streams single-consumer;
    * tarefas criadas fora do contexto.

Produza um **inventário semântico**, separando:

* conceitos que devem ser preservados;
* conceitos que devem ser adaptados;
* detalhes acidentais de implementação Python que devem ser descartados.

## Parte 2 — Análise do harness Goblin/Grok Build

Analise a branch `goblin-implement-epic-tree`, com foco no runtime de sessões e conversação.

Inspecione, no mínimo, os seguintes componentes e suas relações:

* `xai-grok-shell`;
* `xai-chat-state`;
* `xai-grok-agent`;
* `xai-grok-sampler`;
* `xai-grok-sampling-types`;
* `xai-grok-tools`;
* `xai-agent-lifecycle`;
* prompt queue;
* interjection core;
* subagent resolution;
* persistência e replay;
* autenticação multi-provider por request.

Dê atenção especial a:

* `crates/codegen/xai-grok-shell/src/session/acp_session.rs`;
* `session/acp_session_impl/run_loop.rs`;
* `session/acp_session_impl/turn.rs`;
* `session/acp_session_impl/sampler_turn.rs`;
* `session/acp_session_impl/tool_calls.rs`;
* `session/acp_session_impl/tool_dispatch.rs`;
* `session/acp_session_impl/turn_end.rs`;
* `session/acp_session_impl/prompt_build.rs`;
* compactação;
* interjeições;
* notification drain;
* goal orchestration;
* todo/laziness gates;
* hooks;
* memory;
* structured output;
* rewind e fork;
* `xai-chat-state` e seu actor;
* os tipos `ConversationItem`, `ConversationRequest` e `ConversationResponse`.

Mapeie o fluxo real de um turno:

1. Recebimento de um `SessionCommand`.
2. Construção de `InputItem`.
3. Identificação de `PromptOrigin`.
4. Fila, send-now, interjeção ou prompt sintético.
5. Normalização de texto e imagens.
6. Slash commands e skills.
7. Hooks e lifecycle.
8. Escrita no chat state e barreira de persistência.
9. Construção de tool definitions e request.
10. Resolução de modelo, provider, account e credencial.
11. Chamada ao sampler.
12. Streaming de reasoning, texto, tool-call deltas e notificações.
13. Commit da resposta do assistant.
14. Validação de structured output.
15. Execução de ferramentas.
16. Permissões e hooks de ferramentas.
17. Tool results e retorno ao sampler.
18. Retries, refresh de autenticação, compactação e resubmissão.
19. Cancelamento e interrupções.
20. Gates e ações de fim de turno.
21. Persistência, replay, telemetria, memória e notificações.

Identifique:

* onde o controle de fluxo está excessivamente acoplado ao `SessionActor`;
* onde existem “grafos implícitos” escritos como loops, branches e chamadas imperativas;
* quais estados pertencem à sessão;
* quais estados pertencem a uma execução de turno;
* quais estados pertencem a uma chamada de modelo;
* quais estados pertencem a tool calls individuais;
* quais invariantes não podem ser quebradas durante a migração.

Considere especialmente que o runtime atual utiliza elementos como `spawn_local`, `Rc`, `RefCell` e estado actor-based. Avalie explicitamente se o novo core deve exigir `Send + Sync`, suportar execução em `LocalSet`, ou fornecer dois executores.

## Parte 3 — Cruzamento das arquiteturas

Crie uma matriz de correspondência entre os dois sistemas.

Exemplos de itens que devem aparecer na matriz:

* `ProcessorPart` versus `ContentPart`/`ConversationItem`/eventos do harness;
* `ProcessorStream` versus streams do sampler e canais de sessão;
* `Processor` versus nó de transformação;
* `PartProcessor` versus transformação por envelope/item;
* reserved substreams versus notificações, status, reasoning e eventos de UI;
* processor context versus contexto de turno/request;
* chain versus pipeline sequencial;
* parallel versus fan-out;
* switch versus branching;
* function calling processor versus sampler + tool loop;
* realtime processor versus session actor;
* tracing do GenAI Processors versus tracing e unified logs do harness;
* conversation history versus `ChatStateActor`;
* cache versus replay/persistência/cache de requests;
* cancelamento de task group versus cancelamento de turno, tool call e subagente.

Para cada correspondência, classifique:

* reutilização direta do conceito;
* adaptação;
* substituição;
* conceito ausente;
* incompatibilidade importante.

## Parte 4 — Arquitetura Rust proposta

Projete uma arquitetura implementável, evitando tanto uma abstração simplista quanto um framework excessivamente genérico.

A proposta deve responder:

### Modelo de dados

Defina um envelope unificado capaz de representar, sem converter tudo prematuramente para texto:

* texto;
* imagem;
* áudio;
* blobs;
* JSON;
* prompt;
* resposta do assistant;
* reasoning;
* tool call;
* tool result;
* status;
* debug;
* UI event;
* cancelamento;
* erro;
* metadados arbitrários.

Considere campos como:

* role;
* MIME/media type;
* stream ou port;
* correlation ID;
* parent ID;
* tool-call ID;
* session ID;
* turn ID;
* node ID;
* sequence number;
* timestamps;
* metadata tipada ou extensível.

Explique se o core deve usar:

* enum fechado;
* envelope dinâmico;
* `Any`;
* serialização;
* generics;
* ports tipadas;
* ou uma arquitetura híbrida.

### Abstrações centrais

Proponha traits e tipos para, no mínimo:

* `Processor` ou `Node`;
* processor de streams;
* processor de item individual;
* source;
* sink;
* router;
* join;
* stateful node;
* graph;
* graph builder;
* executor;
* execution context;
* node context;
* error policy;
* retry policy;
* loop policy;
* cancellation;
* event/output channel.

Mostre esqueletos de API Rust suficientemente concretos para avaliar:

* object safety;
* lifetimes;
* streams boxed;
* `Send` e `Sync`;
* ownership;
* clonagem;
* serialização;
* testabilidade.

Não use `async_trait` automaticamente. Compare as opções e justifique a escolha.

### Composição

O motor deve suportar explicitamente:

* sequência;
* fan-out;
* merge intercalado;
* concatenação ordenada;
* broadcast/split;
* branch/switch;
* filter;
* map;
* join;
* race;
* timeout;
* retry;
* fallback;
* circuit breaker opcional;
* subgraph;
* loops controlados;
* workflows agentic modelo→ferramenta→modelo;
* processors stateful;
* side outputs;
* eventos out-of-band.

Não permita ciclos arbitrários sem uma política explícita. Modele ciclos com limites como:

* máximo de iterações;
* condição de saída;
* orçamento de tokens;
* deadline;
* cancellation token;
* máximo de tool rounds.

### Concorrência

Defina claramente:

* estratégia de backpressure;
* filas bounded por padrão;
* fairness;
* propagação de cancelamento;
* isolamento ou propagação de erro;
* fechamento de canais;
* comportamento quando um consumidor abandona o stream;
* ordenação dentro de uma branch;
* ordenação entre branches;
* deduplicação;
* replay;
* exatamente uma vez versus pelo menos uma vez;
* lifecycle de tasks filhas;
* limpeza de recursos.

Compare o comportamento proposto com o GenAI Processors original.

### Contexto de execução

O contexto deve conseguir transportar, sem globais mutáveis:

* session binding;
* turn binding;
* model binding;
* provider/account/model imutáveis por request;
* auth resolver;
* token manager;
* tool registry/bridge;
* chat-state handle;
* persistence handle;
* notification sink;
* tracing span;
* cancellation token;
* budgets;
* deadlines;
* feature flags.

A integração deve respeitar a arquitetura multi-provider da branch: uma execução em andamento não pode trocar silenciosamente de provider, account ou model por causa de mudanças globais.

### Observabilidade

Projete:

* tracing por graph run;
* span por node;
* IDs de execução;
* métricas de latência;
* queue wait;
* throughput;
* tokens;
* retries;
* tool calls;
* cancellation;
* erro;
* visualização do grafo;
* snapshots para debugging;
* compatibilidade com unified logs e telemetria existentes.

## Parte 5 — Nova conversação representada como grafos

Não modele toda a aplicação como um único grafo gigantesco.

Proponha grafos ou subgrafos separados, por exemplo:

1. **Session command graph**

   * comandos, eventos, completions, notificações e idle tasks.

2. **Prompt ingestion graph**

   * origem, queue, normalização multimodal, skills, slash commands e prompt build.

3. **Conversation turn graph**

   * estado, hooks, request, sampler, streaming e commit da resposta.

4. **Agentic tool loop graph**

   * resposta do modelo;
   * detecção de tool calls;
   * structured output;
   * permissões;
   * hooks;
   * dispatch;
   * tool results;
   * retorno ao sampler;
   * condição de encerramento.

5. **Post-turn graph**

   * todo gate;
   * goal continuation;
   * verification;
   * laziness;
   * compactação;
   * memória;
   * notificações;
   * persistência;
   * telemetria.

6. **Subagent graph**

   * spawn;
   * execução;
   * usage attribution;
   * background completion;
   * parent notification;
   * cancellation.

Apresente diagramas Mermaid mostrando:

* arquitetura atual simplificada;
* arquitetura alvo;
* fluxo do turn graph;
* fluxo do tool loop;
* fronteiras entre estado, side effects e processamento puro.

## Parte 6 — Estratégia de migração

A migração deve ser incremental. Não proponha substituir todo o `SessionActor` em um único PR.

Crie um plano em fases, semelhante a:

### Fase 0 — Characterization

* testes golden do comportamento atual;
* transcripts;
* eventos ACP;
* persistência;
* ordenação;
* tool loops;
* retries;
* cancelamento;
* compactação;
* autenticação;
* structured output;
* prompts sintéticos;
* subagentes.

### Fase 1 — Core independente

* novo crate de engine;
* tipos básicos;
* executor;
* composição;
* cancelamento;
* backpressure;
* tracing;
* testes unitários;
* sem dependência de Grok-specific types.

### Fase 2 — Adapters

* sampler node;
* chat-state node;
* tool-dispatch node;
* notification sink;
* persistence sink;
* auth/request binding;
* adapters entre envelopes e tipos atuais.

### Fase 3 — Primeiro vertical slice

Escolha um fluxo pequeno, mas real, para executar pelo motor, como:

* sampler streaming sem tool calls; ou
* normalização de prompt; ou
* tool execution subgraph.

Justifique a escolha.

### Fase 4 — Tool loop completo

* model→tools→model;
* retries;
* structured output;
* cancelamento;
* max turns.

### Fase 5 — Turn graph

* prompt ingestion;
* chat state;
* streaming;
* persistência;
* post-turn.

### Fase 6 — Session orchestration

* comandos;
* notificações;
* idle tasks;
* interjeições;
* subagentes.

Para cada fase, informe:

* arquivos/crates afetados;
* APIs novas;
* adapters temporários;
* feature flags;
* critérios de aceite;
* testes;
* riscos;
* estratégia de rollback.

Preserve a branch policy do fork. PRs de produto devem apontar para `goblin`, nunca para `main`.

## Parte 7 — Plano concreto de crates e PRs

Proponha uma divisão de crates. Compare pelo menos duas alternativas e escolha uma.

Uma possibilidade a avaliar, sem assumir que seja a melhor:

* crate de tipos/envelopes;
* crate de core graph/pipeline;
* crate executor Tokio;
* crate de adapters do Grok;
* crate de conversation graph.

Evite dependências circulares.

Forneça:

1. Grafo de dependências entre crates.
2. Estrutura de diretórios.
3. Principais módulos.
4. API pública inicial.
5. Itens que devem permanecer internos.
6. Primeiro conjunto de PRs, cada um pequeno e revisável.
7. Ordem recomendada de implementação.
8. Estimativa relativa de complexidade: pequena, média, grande ou crítica.

## Parte 8 — Testes e validação

Descreva uma estratégia completa:

* testes unitários;
* testes de integração;
* property tests;
* testes de concorrência;
* testes com tempo pausado do Tokio;
* backpressure;
* cancelamento em cada await point relevante;
* falha de um branch paralelo;
* consumidor lento;
* abandono do stream;
* loops infinitos;
* retries;
* auth refresh;
* event ordering;
* replay determinístico;
* persistência;
* snapshot/fork/rewind;
* compatibilidade ACP;
* golden transcripts;
* benchmarks.

Defina métricas para comparar runtime antigo e novo:

* time-to-first-token;
* throughput;
* memória;
* número de tasks;
* tamanho das filas;
* latência de tool calls;
* custo de fan-out;
* custo de tracing;
* comportamento sob cancelamento.

## Parte 9 — Licença e proveniência

Verifique as licenças dos dois projetos.

Diferencie:

* reimplementação baseada em conceitos;
* port direto de código;
* trechos que exigiriam preservação de notices;
* componentes Gemini-specific que não devem entrar no core;
* documentação de proveniência e mudanças.

Não dê aconselhamento jurídico definitivo, mas indique os cuidados de engenharia e documentação necessários.

## Formato obrigatório da resposta

Entregue a resposta nesta ordem:

1. **Resumo executivo**
2. **Commits/refs e arquivos inspecionados**
3. **Como o GenAI Processors realmente funciona**
4. **Como a conversação atual do Grok Build realmente funciona**
5. **Problemas arquiteturais encontrados**
6. **Matriz de correspondência**
7. **Decisões arquiteturais recomendadas**
8. **Arquitetura alvo**
9. **Diagramas Mermaid**
10. **Esqueletos de API Rust**
11. **Representação do conversation/tool loop como grafo**
12. **Plano incremental de migração**
13. **Plano de crates**
14. **Sequência de PRs**
15. **Plano de testes e benchmarks**
16. **Riscos e questões em aberto**
17. **Recomendação do primeiro vertical slice**
18. **Checklist de critérios de aceite**

Use tabelas quando facilitarem comparação.

## Critérios de qualidade

A resposta será considerada incompleta se:

* apenas traduzir classes Python para traits Rust;
* ignorar o código real da branch alvo;
* tratar o `SessionActor` inteiro como um único processor;
* converter toda multimodalidade para `String`;
* ignorar backpressure;
* ignorar cancelamento estruturado;
* ignorar ordenação e replay;
* ignorar autenticação imutável por request;
* ignorar persistência e compatibilidade ACP;
* não explicar `Send` versus execução local;
* não apresentar API Rust concreta;
* não produzir uma estratégia incremental;
* sugerir um big-bang rewrite;
* inventar arquivos, símbolos ou comportamentos não encontrados no repositório.

Quando houver trade-offs, não responda apenas “depende”. Apresente as alternativas, os custos e uma recomendação explícita.

Não peça esclarecimentos antes de realizar a análise. Faça o melhor esforço com o código disponível, registre suposições e destaque lacunas verificáveis.

Antes de concluir, responda objetivamente:

> Qual é a menor arquitetura que nos permite substituir o loop de conversação atual por grafos sem transformar o projeto em um framework genérico impossível de manter?
