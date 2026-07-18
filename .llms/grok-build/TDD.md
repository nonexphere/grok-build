# TDD do grok-oss

Este é o método canônico para executar os epics desta árvore. Behavior change
sem teste vermelho observado não satisfaz o plano.

## Ciclo obrigatório

1. Escolher uma regra observável do epic e seu entrypoint real.
2. Escrever o menor behavior/regression/contract test que a demonstra.
3. Rodar e registrar **RED** pelo motivo esperado.
4. Implementar a menor mudança robusta.
5. Rodar e registrar **GREEN**.
6. Refatorar mantendo o teste verde.
7. Rodar o gate package-scoped e revisar o diff.

Se o teste passa antes da produção mudar, não prova o delta: corrigir o teste
ou demonstrar que o epic é documental. Mocks não podem inventar schema menor
que o real.

## Pirâmide deste monorepo

| Camada | Deve provar | Exemplos |
|---|---|---|
| unit/property | state machines, parsers, IDs, redaction, backpressure | crate local, proptest quando útil |
| behavior/component | facade + runtime fake fiel; store/registry reais | processor, provider login/store/catalog |
| contract | wire/schema e cross-program ownership | Session/Turn/Item, MCP tools, TS generated |
| conformance | mesma semantic core em cada transport | in-process, stdio, WebSocket, MCP, SDK |
| integration | composition root real sem rede externa | shell + leader/Tower + session files |
| e2e/PTY | binary e experiência humana | `xai-grok-pager`/`pager-bin`, headless scripts |
| live opt-in | provider/rede real autorizada | credencial humana, evidence redigida |

## O que mockar

Mockar somente rede externa, OAuth/browser, provider nondeterminístico, relógio
e recursos caros. Fixtures devem preservar método, URL, headers, SSE frames,
status/error e schema reais. Não mockar:

- registry/Tower quando a regra é actor único ou multi-session;
- processor ao testar transport conformance;
- credential store/request binding ao testar provider end-to-end;
- session files/projection ao testar replay/recovery;
- ACL/auth gate ao testar `tower_agent_*`.

## Gates por programa

### Tower / App Server / MCP / tools / SDK

- unit + component do processor/facade;
- contract snapshots de schema, errors e generated TS;
- a mesma black-box suite contra in-process, stdio, WS e MCP;
- concurrency: duplicate start, competing clients, wait/interrupt e slow client;
- security: bearer absent/invalid/revoked, ACL deny, secret canaries e size limits;
- replay/reconnect sem gap, duplicate ou double effect;
- SDK script cria Session, envia Turn, recebe Item stream e interrompe.

### Providers

- provider descriptor/login/store/catalog/binding/request auth num vertical slice;
- duas credentials com mesmo slug não colidem;
- chave nunca cai em `XAI_API_KEY` nem aparece em logs/status;
- backend correto (`chat_completions` default BYOK; `responses` Codex);
- 401 estático pede reauth uma vez, sem refresh inventado;
- live smoke é opt-in e nunca substitui fixtures/offline tests.

### Goal v2

- characterization do v1 antes do refactor;
- state/property/concurrency/crash tests;
- dual-version flag e rollback com fixtures v1/v2;
- completion audit exige evidence atual e falha closed.

## Comandos de referência

Confirmar nomes no Cargo workspace antes de executar; preferir escopo mínimo:

```bash
cargo test -p xai-grok-auth -p xai-grok-multi-auth --no-fail-fast
cargo test -p xai-grok-shell --lib
cargo test -p xai-grok-mcp --no-fail-fast
cargo test -p xai-grok-voice --no-fail-fast
cargo test -p xai-grok-pager --no-fail-fast
cargo check -p xai-grok-shell --lib
cargo build -p xai-grok-pager-bin --bin grok-oss
cargo fmt --check
git diff --check
```

Epics novos devem substituir/expandir esta lista com os crates que criarem.
`cargo check` não atualiza o binário humano: mudanças CLI/TUI terminam com build
debug de `grok-oss` e PTY/smoke real.

## Critério para `concluído`

Um epic só fica `concluído` quando todas as tasks obrigatórias estão `[x]`, o
RED e GREEN são rastreáveis, os gates do epic e dependências passam, docs/status
foram reconciliados e um delivery report aponta evidência. Skip obrigatório é
`blocked/deferred`, nunca PASS.

