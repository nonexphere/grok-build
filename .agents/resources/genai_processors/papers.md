Encontrei um núcleo de literatura bastante alinhado ao que você descreveu. A pesquisa sobre **agent harnesses** é especialmente recente: boa parte dos trabalhos que formalizam o termo apareceu em 2026 e ainda está como preprint.

Uma distinção útil: nesses papers, **harness** geralmente significa a camada externa que transforma um LLM-base em agente — gerenciamento de contexto, ferramentas, memória, estado, controle de fluxo, verificação, permissões e observabilidade. Já **swarm** frequentemente descreve uma composição de agentes em grafos ou workflows, e não necessariamente inteligência de enxame descentralizada no sentido clássico. ([arXiv][1])

## 1. LLM agent harnesses e runtimes

1. **Code as Agent Harness: Toward Executable, Verifiable, and Stateful Agent Systems** — 2026
   Provavelmente o melhor ponto de partida para o conceito de harness. Organiza a área em torno de execução, estado explícito, verificação, avaliação do runtime, coordenação entre agentes e supervisão humana. ([arXiv][2])

2. **Natural-Language Agent Harnesses** — 2026
   Propõe representar a lógica de controle do harness em linguagem natural, separada do código do runtime. Introduz contratos explícitos, artefatos persistentes e adapters para portar harnesses entre ambientes. ([arXiv][3])

3. **AI Harness Engineering: A Runtime Substrate for Foundation-Model Software Agents** — 2026
   Modela o sistema como `modelo + harness + ambiente`. Define responsabilidades como seleção de contexto, memória, estado da tarefa, ferramentas, observabilidade, atribuição de falhas, verificação, permissões e auditoria. ([arXiv][4])

4. **Harness-Bench: Measuring Harness Effects across Models and Agent Workflows** — 2026
   Benchmark voltado a medir quanto configurações do harness afetam o desempenho, separando essas variações das capacidades do modelo-base. ([arXiv][5])

5. **MemoHarness: Agent Harnesses That Learn from Experience** — julho de 2026
   Harness adaptativo que aprende com execuções anteriores. Divide o controle em seis dimensões editáveis e mantém memória de diagnósticos por caso e padrões globais. É um dos trabalhos mais recentes dessa linha. ([arXiv][1])

6. **VeRO: A Harness for Agents to Optimize Agents** — 2026
   Estuda otimização automática de harnesses que misturam código determinístico e chamadas estocásticas ao LLM, usando traces e resultados de execução. ([arXiv][6])

7. **Runtime Harness Adaptation for Deterministic LLM Agents / Life-Harness** — 2026
   Trabalha com adaptação do harness em runtime, mantendo o modelo congelado e modificando o suporte oferecido durante o ciclo de vida da execução. ([arXiv][7])

8. **Auditing Agent Harness Safety** — 2026
   Focado nos riscos introduzidos pelo runtime responsável por despachar ferramentas, alocar recursos e rotear mensagens entre agentes. ([arXiv][8])

## 2. Task-Decoupled Planning e decomposição de tarefas

1. **Beyond Entangled Planning: Task-Decoupled Planning for Long-Horizon Agents** — 2026
   É o paper específico de **Task-Decoupled Planning — TDP**. Um supervisor transforma a tarefa em um DAG de subobjetivos; Planner e Executor operam com contextos isolados por subtarefa. Isso limita propagação de erros e permite replanejamento local. Nos experimentos reportados, reduz consumo de tokens em até 82%. ([arXiv][9])

2. **Understanding the Planning of LLM Agents: A Survey** — 2024
   Taxonomia importante para planejamento agêntico: **task decomposition, plan selection, external modules, reflection e memory**. É uma boa base para posicionar TDP em relação aos métodos anteriores. ([arXiv][10])

3. **Agent-Oriented Planning in Multi-Agent Systems** — 2024
   Define três propriedades para uma decomposição adequada: **solvability**, **completeness** e **non-redundancy**. Depois combina decomposição, alocação de subtarefas e avaliação com reward model. ([arXiv][11])

4. **TaskBench: Benchmarking Large Language Models for Task Automation** — 2023/2024
   Divide automação de tarefas em três capacidades: **task decomposition, tool selection e parameter prediction**. Representa workflows como Tool Graphs, sendo particularmente útil para avaliar planners e orchestrators. ([arXiv][12])

5. **A Scheduler-Theoretic Framework for LLM Agent Execution** — 2026
   Reinterpreta workflows de agentes por uma ótica de escalonamento, dependências e execução de DAGs. Discute explicitamente TDP como mecanismo para isolar contextos e restringir replanejamento. ([arXiv][13])

## 3. Orquestração de swarms e sistemas multiagente

1. **Language Agents as Optimizable Graphs — GPTSwarm** — 2024
   Representa agentes como grafos computacionais: nós são inferências ou ferramentas; arestas representam fluxo de informação. Também otimiza prompts dos nós e a topologia de comunicação entre agentes. ([arXiv][14])

2. **A Dynamic LLM-Powered Agent Network for Task-Oriented Agent Collaboration — DyLAN** — 2023/2024
   Seleciona dinamicamente quais agentes participarão e modifica a estrutura de comunicação segundo a tarefa. Usa um `Agent Importance Score` para manter os agentes mais relevantes. ([arXiv][15])

3. **Scaling Large-Language-Model-based Multi-Agent Collaboration — MacNet** — 2024
   Organiza agentes em DAGs e investiga topologias de colaboração em grande escala, incluindo experimentos com mais de mil agentes. O trabalho relata vantagens de estruturas com propriedades de small-world. ([arXiv][16])

4. **Multi-Agent Collaboration via Evolving Orchestration** — 2025
   Introduz um orquestrador central, chamado de “puppeteer”, treinado por reinforcement learning para selecionar e sequenciar agentes dinamicamente conforme o estado da tarefa. ([arXiv][17])

5. **MAS-Orchestra: Understanding and Improving Multi-Agent Reasoning Through Holistic Orchestration and Controlled Benchmarks** — 2026
   Trata cada subagente como uma função de alto nível e gera a estrutura completa do MAS em uma única decisão de orquestração. Também introduz o MASBENCH, com cinco dimensões estruturais de tarefa: **Depth, Horizon, Breadth, Parallel e Robustness**. ([arXiv][18])

6. **AgentOrchestra: A Hierarchical Multi-Agent Framework for General-Purpose Task Solving** — 2025
   Arquitetura hierárquica com agente planejador central, subobjetivos explícitos, agentes especializados, comunicação entre agentes e alocação adaptativa de papéis. ([arXiv][19])

7. **OrchestrationBench: LLM-Driven Agentic Planning and Tool Use in Multi-Domain Scenarios** — 2026
   Avalia planejamento de workflows e execução de ferramentas sob restrições. Abrange tarefas sequenciais, paralelas e multidomínio com um conjunto amplo de ferramentas virtuais. ([OpenReview][20])

8. **Jointly Optimizing Model Roles and Weights for Multi-LLM Systems — Heterogeneous Swarms** — 2025
   Representa sistemas multi-LLM como DAGs e otimiza simultaneamente os papéis dos modelos e os pesos das conexões. É particularmente relevante quando o swarm contém modelos heterogêneos. ([arXiv][21])

9. **MultiAgentBench: Evaluating the Collaboration and Competition of LLM Agents** — 2025
   Compara topologias star, chain, tree e graph, além de estratégias como discussão em grupo e cognitive planning. Mede tanto conclusão da tarefa quanto qualidade da coordenação. ([arXiv][22])

## 4. Taxonomias, falhas e avaliação

1. **Why Do Multi-Agent LLM Systems Fail?** — 2025
   Um dos papers mais importantes para diagnóstico. Introduz a **Multi-Agent System Failure Taxonomy — MAST**, com 14 modos de falha agrupados em:
   **design/especificação do sistema**, **desalinhamento entre agentes** e **verificação/terminação inadequadas**. ([arXiv][23])

2. **A Taxonomy of Hierarchical Multi-Agent Systems** — 2025
   Classifica sistemas hierárquicos por cinco eixos: hierarquia de controle, fluxo de informação, delegação de papéis/tarefas, hierarquia temporal e estrutura de comunicação. ([arXiv][24])

3. **Multi-Agent Collaboration Mechanisms: A Survey of LLMs** — 2025
   Taxonomia de colaboração baseada em **atores, tipo de relação, estrutura organizacional, estratégia e protocolo de coordenação**. É uma das melhores referências para construir uma taxonomia de swarms. ([arXiv][25])

4. **Large Language Model Based Multi-Agents: A Survey of Progress and Challenges** — 2024
   Survey amplo sobre comunicação, perfis dos agentes, percepção, ação, memória e diferentes estruturas multiagente. ([arXiv][26])

5. **A Survey on LLM-Based Multi-Agent Systems** — 2024/2025
   Oferece uma visão sistemática das arquiteturas, workflows e desafios recentes de LLM-MAS. ([arXiv][27])

6. **AgentBench: Evaluating LLMs as Agents** — 2023
   Benchmark fundacional com oito ambientes interativos. Identifica problemas recorrentes de planejamento de longo prazo, tomada de decisão e seguimento de instruções. ([arXiv][28])

## 5. Geração e otimização automática de harnesses/workflows

1. **Automated Design of Agentic Systems — ADAS** — 2024
   Formula o projeto de sistemas agênticos como um problema de busca. Um meta-agente escreve novos agentes em código e mantém um arquivo dos designs descobertos. ([arXiv][29])

2. **AFlow: Automating Agentic Workflow Generation** — 2024/2025
   Faz busca sobre workflows representados em código usando Monte Carlo Tree Search, feedback de execução e refinamento iterativo. ([arXiv][30])

3. **AgentSquare: Automatic LLM Agent Search in Modular Design Space** — 2024
   Divide agentes em quatro módulos: **Planning, Reasoning, Tool Use e Memory**, realizando evolução e recombinação automática desses módulos. ([arXiv][31])

4. **Multi-Agent Architecture Search via Agentic Supernet — MaAS** — 2025
   Aprende uma distribuição de arquiteturas e amostra um sistema multiagente diferente conforme a dificuldade e o domínio de cada consulta. ([arXiv][32])

5. **AutoFlow: Automated Workflow Generation for Large Language Model Agents** — 2024
   Gera workflows em linguagem natural e os otimiza iterativamente, com variantes baseadas em fine-tuning e in-context learning. ([arXiv][33])

## Taxonomia operacional sugerida

Combinando esses trabalhos, uma taxonomia prática para seu tema poderia ter cinco níveis:

1. **Estrutura da tarefa:** atomicidade, profundidade, horizonte, largura, paralelismo, dependências e necessidade de robustez.
2. **Decomposição e alocação:** DAG de subtarefas, papéis fixos ou emergentes, seleção estática ou dinâmica de agentes.
3. **Topologia de coordenação:** centralizada, hierárquica, peer-to-peer, star, tree, graph, small-world ou topology learned.
4. **Harness de execução:** contexto global ou isolado, memória, estado persistente, ferramentas, permissões, tracing, verificação e recuperação.
5. **Avaliação:** sucesso final, completude dos subobjetivos, custo/tokens, latência, overhead de comunicação, resiliência e modo de falha MAST.

Essa síntese deriva principalmente de TDP, MASBENCH, GPTSwarm, da taxonomia de HMAS e dos trabalhos de harness engineering. ([arXiv][9])

### Ordem de leitura recomendada

Começaria por: **Understanding the Planning of LLM Agents → Task-Decoupled Planning → Code as Agent Harness → GPTSwarm → MAS-Orchestra/MASBENCH → Why Do Multi-Agent LLM Systems Fail? → ADAS/AFlow**. Essa sequência vai de taxonomia e planejamento para runtime, topologias, avaliação e otimização automática.

[1]: https://arxiv.org/abs/2607.14159?utm_source=chatgpt.com "MemoHarness: Agent Harnesses That Learn from Experience"
[2]: https://arxiv.org/abs/2605.18747?utm_source=chatgpt.com "[2605.18747] Code as Agent Harness"
[3]: https://arxiv.org/abs/2603.25723?utm_source=chatgpt.com "Natural-Language Agent Harnesses"
[4]: https://arxiv.org/abs/2605.13357?utm_source=chatgpt.com "AI Harness Engineering: A Runtime Substrate for Foundation-Model Software Agents"
[5]: https://arxiv.org/html/2605.27922v1?utm_source=chatgpt.com "Harness-Bench: Measuring Harness Effects across Models ..."
[6]: https://arxiv.org/abs/2602.22480?utm_source=chatgpt.com "VeRO: A Harness for Agents to Optimize Agents"
[7]: https://arxiv.org/html/2605.22166v1?utm_source=chatgpt.com "Runtime Harness Adaptation for Deterministic LLM Agents"
[8]: https://arxiv.org/abs/2605.14271?utm_source=chatgpt.com "[2605.14271] Auditing Agent Harness Safety"
[9]: https://arxiv.org/abs/2601.07577?utm_source=chatgpt.com "Beyond Entangled Planning: Task-Decoupled Planning for Long-Horizon Agents"
[10]: https://arxiv.org/abs/2402.02716?utm_source=chatgpt.com "Understanding the planning of LLM agents: A survey"
[11]: https://arxiv.org/abs/2410.02189?utm_source=chatgpt.com "Agent-Oriented Planning in Multi-Agent Systems"
[12]: https://arxiv.org/abs/2311.18760?utm_source=chatgpt.com "TaskBench: Benchmarking Large Language Models for Task Automation"
[13]: https://arxiv.org/pdf/2604.11378?utm_source=chatgpt.com "A Scheduler-Theoretic Framework for LLM Agent Execution"
[14]: https://arxiv.org/abs/2402.16823?utm_source=chatgpt.com "Language Agents as Optimizable Graphs"
[15]: https://arxiv.org/abs/2310.02170?utm_source=chatgpt.com "A Dynamic LLM-Powered Agent Network for Task-Oriented Agent Collaboration"
[16]: https://arxiv.org/abs/2406.07155?utm_source=chatgpt.com "Scaling Large-Language-Model-based Multi-Agent Collaboration"
[17]: https://arxiv.org/abs/2505.19591?utm_source=chatgpt.com "Multi-Agent Collaboration via Evolving Orchestration"
[18]: https://arxiv.org/abs/2601.14652?utm_source=chatgpt.com "MAS-Orchestra: Understanding and Improving Multi-Agent Reasoning Through Holistic Orchestration and Controlled Benchmarks"
[19]: https://arxiv.org/abs/2506.12508?utm_source=chatgpt.com "AgentOrchestra: A Hierarchical Multi-Agent Framework for General-Purpose Task Solving"
[20]: https://openreview.net/forum?id=Oljnxmf4pc&utm_source=chatgpt.com "OrchestrationBench: LLM-Driven Agentic Planning and ..."
[21]: https://arxiv.org/html/2502.04510v1?utm_source=chatgpt.com "Jointly Optimizing Model Roles and Weights for Multi-LLM ..."
[22]: https://arxiv.org/abs/2503.01935?utm_source=chatgpt.com "MultiAgentBench: Evaluating the Collaboration and Competition of LLM agents"
[23]: https://arxiv.org/abs/2503.13657?utm_source=chatgpt.com "Why Do Multi-Agent LLM Systems Fail?"
[24]: https://arxiv.org/abs/2508.12683?utm_source=chatgpt.com "A Taxonomy of Hierarchical Multi-Agent Systems: Design Patterns, Coordination Mechanisms, and Industrial Applications"
[25]: https://arxiv.org/abs/2501.06322?utm_source=chatgpt.com "Multi-Agent Collaboration Mechanisms: A Survey of LLMs"
[26]: https://arxiv.org/abs/2402.01680?utm_source=chatgpt.com "[2402.01680] Large Language Model based Multi-Agents"
[27]: https://arxiv.org/html/2412.17481v2?utm_source=chatgpt.com "A Survey on LLM-based Multi-Agent System"
[28]: https://arxiv.org/abs/2308.03688?utm_source=chatgpt.com "AgentBench: Evaluating LLMs as Agents"
[29]: https://arxiv.org/abs/2408.08435?utm_source=chatgpt.com "Automated Design of Agentic Systems"
[30]: https://arxiv.org/abs/2410.10762?utm_source=chatgpt.com "AFlow: Automating Agentic Workflow Generation"
[31]: https://arxiv.org/abs/2410.06153?utm_source=chatgpt.com "AgentSquare: Automatic LLM Agent Search in Modular Design Space"
[32]: https://arxiv.org/abs/2502.04180?utm_source=chatgpt.com "Multi-agent Architecture Search via Agentic Supernet"
[33]: https://arxiv.org/abs/2407.12821?utm_source=chatgpt.com "AutoFlow: Automated Workflow Generation for Large Language Model Agents"
