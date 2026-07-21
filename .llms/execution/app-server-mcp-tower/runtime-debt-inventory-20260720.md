# Runtime debt inventory — App Server / MCP / Tower (2026-07-20)

## Método

Busca read-only por `TODO`, `FIXME`, `PARTIAL`, `unsupported`, `experimental`,
`placeholder`, `not implemented`, `todo!` e `unreachable!` nos crates de
controle de runtime. A busca encontrou 124 ocorrências brutas; derives,
comentários de segurança e testes negativos foram separados de debt executável.

## Itens executáveis confirmados

| Área | Evidência | Classificação | Ação/owner |
|---|---|---|---|
| Actor spawn | `shell_session_actor_runtime.rs:519-560` | GAP product P0 | TW106-02..09; Shell + pager composition |
| Turn projection | `shell_session_actor_runtime.rs:1425-1684` | PARTIAL contract | AS109-01/03; definir eventos/ciclo canônico |
| Interaction delivery | `shell_session_actor_runtime.rs:2226-2272` | GAP product | AS109-04; lease/reconnect/timeout |
| Local turn spawn | `experimental_local_turn_spawn` | test-only fixture | manter isolado; nunca promover ao product path |
| MCP product binding | `streamable_http.rs:1652` | guard/documentation | MCP104-08; product bin binding ainda precisa prova |
| TLS | `http_server.rs:404-411`, WS listener | HUMAN gate | MCP105-07 / D-SEC.13; não remover warning |
| Unsupported aliases | `tower-tools/src/lib.rs:238-290` | compatibility projection | manter mapeamento até catálogo App Server convergir |
| `unreachable!` | `tower-tools/src/lib.rs:979` | defensive invariant | revisar quando enum/schema ganhar variantes |
| Placeholder storage root | `shell_session_actor_runtime.rs:655` | test fallback | substituir quando runtime expuser root; não é product authority |

## Itens não classificados como dead code

- `unsupported` em WS/MCP: rejeições de protocolo esperadas e cobertas por
  testes adversariais.
- `experimental/unsafe`: rótulo de segurança obrigatório para cleartext
  remoto, não feature abandonada.
- `FakeRuntime`: autoridade de conformance explicitamente separada do product
  runtime; possui cobertura semântica e não deve ser removido.
- `process_mcp_stdio_batch`: usado pelo transporte stdio e pelos testes de
  paridade/fixture; não é helper morto.

## Verificação

```text
rg ... nos crates alvo: 124 ocorrências brutas
cargo test -p xai-grok-tower-tools --all-targets: 48 passed
cargo test -p xai-grok-app-server: 41 passed
cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http: 41 passed
cargo clippy -p xai-grok-app-server -p xai-grok-mcp-server \
  -p xai-grok-tower-tools --all-targets: clean after collapsing one nested `if`
```

## Conclusão

Não há remoção segura justificada nesta wave. Os itens principais são gaps de
produto ou decisões humanas documentadas; remover `unsupported`, o fixture
local ou os warnings de segurança reduziria a honestidade do contrato.
