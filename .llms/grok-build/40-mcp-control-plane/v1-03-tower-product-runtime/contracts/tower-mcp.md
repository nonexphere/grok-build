# Contrato MCP — integração com `tower`

**Fonte de verdade local do epic.** O supervisor chama o listener Streamable
HTTP com `bind`, `bearer_token` e `tower_instance_id`, recebe
`McpHttpHandle`, e nunca registra o endpoint local no pool outbound MCP.

O listener rejeita secret vazio quando auth está ativa, exige bearer no header
ou em `?bearer=<token>` e publica `POST /mcp`. `--insecure-no-auth` desativa a
exigência apenas por opt-in explícito. O supervisor aborta o handle em
rollback/shutdown e garante que um bind parcial não fica acessível.
