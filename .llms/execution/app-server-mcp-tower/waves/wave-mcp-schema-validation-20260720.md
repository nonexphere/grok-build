# Wave — MCP/Tower schema boundary (2026-07-20)

## Problema encontrado

`tools/list` projetava apenas o `$defs` específico de cada ferramenta. Alguns
schemas publicados ainda tinham `$ref` internos, mas as definições compartilhadas
não eram transportadas; um cliente independente não conseguia compilá-los.

## Correção

- `tool_schema` mantém o `$defs` selecionado como raiz e inclui as definições
  canônicas compartilhadas;
- os nove input e nove output schemas são compilados com `jsonschema` como
  documentos independentes;
- `invoke_tower_tool` valida argumentos após ACL e antes de qualquer lookup ou
  efeito no runtime;
- falhas de schema retornam `invalid_params`, sem expor estado do alvo.

## Gates

```text
cargo test -p xai-grok-tower-tools
cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http post_tools_lists_exactly_nine_descriptors_matching_in_process
```

Resultados: 24 unitários + 24 integração Tower passaram; o caso MCP de
listagem/parity passou.
