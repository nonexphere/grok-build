# TDD do grok-oss

Este é o método canônico para executar os epics desta árvore. Behavior change
sem teste vermelho observado não satisfaz o plano.

## Gates Rust não-vazios

Um `cargo test <filtro>` termina com sucesso mesmo quando seleciona zero testes,
portanto não é um gate de epic válido sozinho. Todo gate Rust nomeado DEVE usar
`scripts/run-rust-test-gate.sh <fragmento-esperado> cargo test ...`. O wrapper
exige tanto sucesso do Cargo quanto ao menos uma linha
`test ...<fragmento-esperado>... ok`. Comandos de package completo, sem filtro,
continuam válidos quando a aceitação é a suíte integral.

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
cargo test -p xai-grok-app-server-protocol
cargo test -p xai-grok-tower-tools
cargo check -p xai-grok-app-server -p xai-grok-tower -p xai-grok-mcp-server
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

## Named suites and RED/GREEN evidence

| Suite | Planned location | Observation |
|---|---|---|
| `protocol_roundtrip` | protocol crate | serde and schema compile |
| `protocol_goldens` | `schemas/goldens/*.jsonl` | envelopes/order |
| `leader_characterization` | shell leader tests | existing bytes/lifecycle |
| `tower_lifecycle` | Tower integration tests | isolation/restart/drain |
| `runtime_facade_conformance` | Tower tests | real adapter equals faithful fake |
| `tower_tool_contract` | Tower tools tests | nine schemas and ACL |
| `adapter_parity` | MCP server tests | MCP equals in-process |
| `control_plane_security` | integration tests | bearer/limits/canaries |
| `transport_conformance` | App Server tests | in-process/stdio/WS |
| `sdk_drift` | TS scripts | generated output has no drift |

Each behavior task records exact RED command/test/failure, minimal GREEN change,
GREEN command and refactor. A test already green is not RED. Structural
scaffolds use `cargo check`; observable serde/schema and ACL require real tests.

## Canonical conformance layout

```text
crates/codegen/xai-grok-app-server-protocol/
├── src/{lib,methods,events}.rs
├── schemas/{app-server,tower-tools}.schema.json
├── schemas/goldens/{happy-coding,interrupt,multi-session,reconnect}.jsonl
└── tests/{wire_roundtrip,schema_drift,golden_validation}.rs
crates/codegen/xai-grok-tower/tests/
├── instance_id.rs
├── lifecycle.rs
├── facade_conformance.rs
└── one_actor.rs
crates/codegen/xai-grok-app-server/tests/
├── conformance/{mod,in_process,stdio,websocket}.rs
├── initialize_gate.rs
├── subscribe_replay.rs
├── interactions.rs
└── security.rs
crates/codegen/xai-grok-tower-tools/tests/
├── descriptors.rs
├── acl.rs
└── parity_fixtures.rs
crates/codegen/xai-grok-mcp-server/tests/
├── stdio.rs
├── streamable_http.rs
└── adapter_parity.rs
packages/grok-oss-app-server/
├── test/{client,errors,reconnect,drift}.test.ts
└── examples/{stdio,websocket}.ts
```

The App Server `tests/conformance` fixture interface accepts a transport driver
and executes one corpus. MCP parity consumes the same Tower tool fixture values
but compares MCP structured content with the in-process tool adapter. Test code
does not duplicate expected JSON: expected envelopes live in protocol goldens.

## Evidence record format

```text
Task: AS103-03 [D-TR.1]
RED command: cargo test -p xai-grok-app-server stdio_protocol_only_stdout
RED observed: assertion found diagnostic bytes on stdout
GREEN change: transport/stdio.rs routes tracing to stderr writer
GREEN command/result: <same command>, 1 passed
Refactor/gate: cargo test -p xai-grok-app-server stdio, N passed
Commit/PR evidence: <SHA or PR link when execution is authorized>
```

Required security names: `bearer_header_only`, `token_file_permissions`,
`auth_failures_are_indistinguishable`, `remote_bind_warning_exact`,
`redaction_canary_all_lengths_all_sinks`, `oversized_before_deserialize`,
`slow_subscriber_resync`, `acl_before_target_lookup`,
`foreign_mcp_session_rejected`, and `epoch_mismatch_rejected`.

## Critério para `concluído`

Um epic só fica `concluído` quando todas as tasks obrigatórias estão `[x]`, o
RED e GREEN são rastreáveis, os gates do epic e dependências passam, docs/status
foram reconciliados e um delivery report aponta evidência. Skip obrigatório é
`blocked/deferred`, nunca PASS.
