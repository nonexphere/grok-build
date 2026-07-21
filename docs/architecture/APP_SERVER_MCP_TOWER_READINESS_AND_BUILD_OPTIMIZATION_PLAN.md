# App Server, MCP e Tower — Readiness, Gaps e Otimização de Build

**Status:** execução em andamento  
**Snapshot inicial:** 2026-07-21  
**Escopo:** App Server, MCP Server, Tower, Tower Tools, Shell, composição do `grok-oss`, SDK, CI e compilação  
**Fonte canônica:** este documento  

## 1. Objetivo

Este documento transforma a auditoria do App Server, MCP e Tower em um programa
executável de implementação. Ele tem quatro objetivos simultâneos:

1. listar o que já está comprovado e o que ainda não está pronto;
2. identificar desalinhamentos entre contratos, runtime, transportes e produto;
3. definir epics, dependências, testes e gates para chegar a uma versão realmente
   product-backed;
4. medir e reduzir o custo de compilação, linkedição, CI e distribuição sem
   sacrificar correção, segurança ou observabilidade.

“100%” significa cobertura de requisitos e evidência de comportamento. Nenhum
percentual de aceleração é prometido antes de medir cold build, warm build,
incremental build, release build, memória, link e tamanho do artefato.

## 2. Evidência atual

### 2.1 Componentes analisados

| Área | Crate/alvo | Estado observado |
|---|---|---|
| Contrato | `xai-grok-app-server-protocol` | Tipos, schemas e erros tipados existentes; geração e drift ainda precisam de gate contínuo. |
| App Server | `xai-grok-app-server` | In-process/stdio/WS e conformance avançados; runtime product-backed completo ainda aberto. |
| Tower | `xai-grok-tower` | Lifecycle/facade e isolamento cobertos por testes; actor real ligado ao produto ainda aberto. |
| Tools | `xai-grok-tower-tools` | Semântica e ACL dos nove tools cobertas no core; matriz product-backed completa ainda aberta. |
| MCP | `xai-grok-mcp-server` | HTTP Streamable e stdio têm cobertura de transporte; parity completa, segurança remota e provider-backed continuam abertas. |
| Runtime | `xai-grok-shell` | Runtime Shell amplo e ACP bridge experimental; factory canônico `SessionActor` ainda não é a autoridade completa do produto. |
| Produto | `xai-grok-pager-bin` | Composition root possui listeners e testes de composição; vertical real start/send/wait/history/restart não está comprovado. |
| SDK | TypeScript/geração | Contrato previsto no plano; regeneração limpa e black-box contra listeners reais permanecem pendentes. |

### 2.2 Gates já comprovados

O gate leve consolidado executado anteriormente aprovou 164 testes:

```text
cargo test -p xai-grok-app-server-protocol \
  -p xai-grok-app-server \
  -p xai-grok-mcp-server \
  --features xai-grok-mcp-server/streamable-http \
  -p xai-grok-tower --all-targets
```

Evidências registradas:

- App Server protocol: 22 testes;
- App Server: 41 testes;
- MCP library: 21 testes;
- MCP Streamable HTTP: 41 testes;
- Tower unit: 29 testes;
- Tower integration: 10 testes;
- `rmcp` HTTP: initialize/list/call/error;
- `rmcp` child-process stdio: discovery/call/EOF básico;
- matriz HTTP dos nove Tower tools;
- parity stdio/HTTP de `code`, `retryable` e `operationId`;
- geração de schema verificada;
- clippy limpo nos crates principais após correção de debt.

### 2.3 Limitações comprovadas

- o caminho completo do actor real no binário de produto ainda não tem evidência
  vertical suficiente;
- o ACP bridge não substitui o factory canônico de `SessionActor`;
- testes de timeout do Shell ficaram limitados pelo custo/instabilidade da
  compilação dos binários grandes;
- `--no-default-features` do Shell possui problema de feature-gating preexistente;
- execução no GitHub, branch protection e required checks ainda não foram
  verificados localmente;
- TLS remoto continua sujeito a gate humano e configuração externa;
- provider, credenciais e APIs externas não são assumidos como disponíveis;
- a árvore de trabalho contém alterações de outros escopos e não pode ser
  resetada, stashada ou sobrescrita.

## 3. Critério de conclusão

Um item só pode ser marcado como concluído quando tiver:

- contrato definido;
- owner e arquivos afetados identificados;
- teste comportamental ou contract test apropriado;
- comando reproduzível;
- evidência real do caminho relevante;
- tratamento explícito de erro, timeout, cancelamento e restart;
- revisão de segurança e impacto downstream.

Compilar não prova comportamento. Teste de FakeRuntime não prova integração de
produto. Capability só pode ser anunciada se o caminho executável correspondente
passar o gate product-backed.

## 4. Gaps funcionais e arquiteturais

### 4.1 Runtime canônico e Tower

Implementar e comprovar:

- factory único de `SessionActor` product-backed;
- uma única autoridade para criar, resumir, arquivar, bifurcar e reiniciar sessão;
- injeção compartilhada do actor/adapters em App Server, MCP e Tower Tools;
- binding imutável de provider, modelo, workspace, sandbox, permissões e agent type;
- start → turn → eventos → wait → history com dados reais;
- cursor, epoch, replay e resync monotônicos;
- archive, dormant resume e restart sem perder identidade;
- rollback completo em falha de spawn ou persistência;
- concorrência entre sessões sem duplicar actor;
- interrupt-versus-complete com um único terminal state;
- limites de fila, memória, payload, duração e backpressure;
- readiness que não publica `ready` sem actor factory funcional.

### 4.2 App Server

Completar a convergência de:

- catálogo de métodos;
- capability registry e negociação;
- envelopes de resultado e erro;
- `code`, `retryable`, `operationId` e diagnóstico seguro;
- identidade de operação parcialmente concluída;
- projeção de session/turn/item/event/interaction;
- lifecycle e transições inválidas;
- cancelamento, timeout, desconexão, reconexão e resync;
- equivalência entre in-process, stdio e WebSocket;
- schemas gerados e drift detection;
- falha de startup sem claims falsos de sessão ou readiness.

### 4.3 MCP

Fechar:

- parity semântica completa entre stdio e Streamable HTTP;
- schemas de entrada e saída dos nove tools compiláveis independentemente;
- validação antes de qualquer efeito runtime;
- lifecycle POST/GET/DELETE, TTL, rebind e reconexão;
- múltiplas sessões MCP e isolamento por token;
- cancelamento, disconnect, resync e limites de stream;
- scopes, criação/listagem/revogação e corrida de revogação;
- rejeição de bearer em query string quando seguro;
- TLS real/proxy TLS e bind seguro;
- health/readiness do listener;
- cliente independente `rmcp` com provider double fiel;
- matriz stdio com turn real, interrupção e parity de erro.

### 4.4 Produto

Provar pelo binário `grok-oss`:

1. initialize;
2. start session com workspace, agent type, provider e sandbox;
3. start turn com input estruturado;
4. user item, agent item, tool events e status monotônicos;
5. wait/replay com epoch e cursor;
6. history equivalente ao stream;
7. interrupt e corrida terminal;
8. archive, dormant resume e restart;
9. transcript canônico sem secrets;
10. capability matrix derivada do caminho real.

O provider externo pode ser substituído apenas no boundary por um double fiel.
Registry, actor, arquivos, permissões, adapters e transportes devem continuar
reais.

### 4.5 SDK e geração

- deletar outputs gerados antes de regenerar;
- comprovar clean diff;
- detectar drift em CI;
- testar SDK contra stdio e HTTP/WS reais;
- validar erro, abort, reconnect, replay e capability negotiation;
- documentar versionamento e compatibilidade do wire contract.

### 4.6 Segurança e operação

- ACL por método e agent type;
- tokens scoped e revogáveis;
- TLS e política de bind;
- rate limiting e limites de payload;
- secret canary em logs, erros, eventos e arquivos;
- auditoria de ações;
- métricas de latência, fila, erro, retry, timeout e reconexão;
- graceful shutdown e recuperação após crash;
- retenção e limpeza de sessões;
- comportamento explícito sem credenciais/provider.

### 4.7 Dead code e caminhos experimentais

O inventário de `TODO`, `FIXME`, placeholders, fakes, features e APIs não
utilizadas deve ser classificado em remover, substituir, manter com justificativa,
transformar em contrato, bloquear por decisão humana ou falso positivo. Nenhuma
remoção será feita apenas por busca textual.

## 5. Programa de otimização de compilação

### 5.1 Baseline

Medir em três execuções cada:

| Cenário | Comando/medição |
|---|---|
| check incremental | `cargo check -p <crate>` |
| teste por crate | `cargo test -p <crate>` |
| integração MCP | `cargo test -p xai-grok-mcp-server --features streamable-http` |
| binário debug | `cargo build -p xai-grok-pager-bin --bin grok-oss` |
| binário release | `cargo build -p xai-grok-pager-bin --bin grok-oss --release` |
| distribuição | `cargo build -p xai-grok-pager-bin --bin grok-oss --profile release-dist` |
| timing | `cargo build --timings ...` |
| memória/link | `/usr/bin/time -v` e logs do linker |
| tamanho | `stat`, `size`, símbolos e debug artifacts |

Registrar cold cache, warm cache, alteração em crate leaf, protocolo e Shell,
CI com cache e CI sem cache.

### 5.2 Grafo e features

O Shell possui aproximadamente 150 dependências diretas declaradas e o
composition root aproximadamente 43. Investigar e medir:

- separar composition root de TUI/runtime pesado;
- retirar clients e fixtures de teste do caminho principal;
- tornar listeners opcionais sem ativação acidental;
- criar targets de conformance menores;
- reduzir features default do binário quando o comportamento permitir;
- eliminar duplicação de dependências e aliases legados;
- avaliar `default-features = false` sem alterar contratos de segurança;
- separar crates leaf de crates de integração.

### 5.3 Cargo profiles

Comparar antes/depois:

- `dev` com incremental e codegen units altos;
- profile de conformance com otimização moderada;
- profile product-integration separado;
- `release` para uso local;
- `release-dist` para distribuição;
- `lto = false`, `thin` e `fat`;
- diferentes `codegen-units`;
- debug info e split debug;
- incremental em release;
- panic abort/unwind conforme target.

Nenhuma mudança será aceita sem medir tempo, memória, tamanho e runtime.

### 5.4 Linker, cache e CI

Avaliar:

- `sccache`/rustc wrapper;
- `mold`/`lld` por plataforma;
- jobs ajustados à memória;
- cache correto de registry, git e target;
- cache separado por toolchain, target, profile e features;
- `cargo-nextest`;
- reuse de artefatos entre jobs;
- evitar recompilação entre fake-conformance e product-integration;
- cache de dependências nativas;
- medição isolada do link.

### 5.5 Build scripts e reprodutibilidade

Auditar downloads de ripgrep, cópias de `bfs`/`ugrep`, invalidação por
`.git/HEAD`, dependências `reqwest`/`flate2`/`tar`, modo offline, checksum,
cache compartilhado e reexecução desnecessária. Release deve ser reproduzível
e não depender de download implícito sem evidência e integridade verificável.

## 6. Epics executáveis

As especificações executáveis estão materializadas na
[árvore `.llms/grok-build`](../../.llms/grok-build/README.md). O programa 05
contém build, release e limpeza; os programas 20–60 contêm runtime, contratos,
segurança, SDK, observabilidade e readiness.

| ID | Epic | Dependência | Gate |
|---|---|---|---|
| E0 | Baseline e instrumentação de build | nenhuma | métricas reproduzíveis |
| E1 | Runtime product-backed e actor canônico | E0 | vertical start/turn/history |
| E2 | Lifecycle, persistence, recovery e identity | E1 | restart/resume/archive |
| E3 | App Server contract e capability truth | E1/E2 | matrix real de capabilities |
| E4 | MCP transport parity e multi-session | E3 | stdio/HTTP parity completa |
| E5 | Auth, scopes, TLS e hardening | E4 | security matrix |
| E6 | Tower tools semantic/product completion | E1/E3 | nove tools product-backed |
| E7 | SDK generation e black-box compatibility | E3/E4 | regeneração limpa + listener real |
| E8 | Dependency graph e feature slicing | E0 | redução medida sem regressão |
| E9 | Cargo profiles, linker, cache e CI | E0/E8 | build matrix otimizada |
| E10 | Dead code e caminhos experimentais | E1–E7 | inventário terminal |
| E11 | Observability, limits e fault testing | E1–E6 | carga/falha/secret canary |
| E12 | Release readiness e validação humana | E1–E11 | release evidence completo |

## 7. Acceptance matrix final

O gate final exige:

- todo requisito com owner, contrato e teste;
- nenhuma capability falsa;
- actor product-backed comprovado;
- equivalência entre transportes;
- geração sem drift;
- autenticação, scopes e TLS validados;
- build reproduzível documentado;
- baseline e ganhos de compilação medidos;
- bloqueios humanos/externalizados explícitos;
- nenhum item marcado como concluído apenas por FakeRuntime ou compilação.

## 8. Registro de execução

| Data | Onda | Resultado | Evidência |
|---|---|---|---|
| 2026-07-21 | E0/documentação | Em execução | estado, manifests, grafo e gates coletados; este documento criado |

## 9. Referências de autoridade

- `.llms/grok-build/COMPLETION_COVERAGE.md`
- `.llms/grok-build/_shared/product-runtime-readiness.md`
- `.llms/execution/app-server-mcp-tower/runtime-debt-inventory-20260720.md`
- `.llms/execution/app-server-mcp-tower/FINAL_REPORT.md`
- `.github/workflows/ci-grok-oss.yml`
- `Cargo.toml`
- `.cargo/config.toml`
- `rust-toolchain.toml`
