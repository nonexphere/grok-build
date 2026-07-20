# Contrato compartilhado — `grok-oss tower`

**Fonte de verdade.** Este contrato define o comando supervisor local que
coordena o App Server WebSocket e o MCP Streamable HTTP do produto `grok-oss`.
Referenciado pelos epics Tower, App Server e MCP.

Proveniência: `[provenance: user-input, conversation, workspace, code, inferred]`.

## Comando e defaults

```text
grok-oss tower [--bind <ADDR>] [--mcp-bind <ADDR>]
               [--secret <TOKEN>] [--no-mcp] [--no-app-server]
```

- `tower` inicia ambos os listeners por padrão.
- `--no-mcp` desabilita somente MCP.
- `--no-app-server` desabilita somente App Server.
- É inválido usar as duas flags de exclusão simultaneamente: o comando deve
  retornar erro antes de abrir sockets.
- O bind padrão do App Server é `127.0.0.1:2419`.
- O bind padrão do MCP é `127.0.0.1:8788`.
- `--bind` sobrescreve apenas o App Server; `--mcp-bind` controla MCP.
- Bind não-loopback é permitido somente com warning explícito de cleartext;
  TLS permanece gate humano e não é inventado pelo supervisor.

## Secret

- `--secret` é opcional.
- Se omitido, `GROK_AGENT_SECRET` é usado quando não vazio; caso contrário o
  supervisor gera um token aleatório criptograficamente adequado.
- O token gerado é compartilhado pelos dois listeners e exibido somente como
  presença/fingerprint seguro, nunca em texto claro ou URL.
- Secret vazio/whitespace fornecido explicitamente é erro fail-closed.
- O processo não grava o secret em disco por padrão.
- `--insecure-no-auth` é um opt-in explícito que desativa autenticação nos dois
  listeners; deve ser usado somente em integrações locais/controladas.

## Lifecycle

1. Validar flags, binds e secret.
2. Iniciar listeners habilitados em tarefas concorrentes.
3. Se qualquer bind falhar, abortar o listener já iniciado e retornar erro
   não-zero (sem processo parcialmente saudável).
4. Aguardar `SIGINT`, `SIGTERM` ou `SIGHUP`.
5. Sinalizar shutdown, aguardar tarefas e fechar sockets antes de sair.

## Health/acceptance

- App Server aceita WebSocket com `Authorization: Bearer <token>` e responde
  `initialize` com `2026-07-18.experimental-v2`; no modo inseguro aceita sem
  bearer.
- MCP aceita `POST /mcp` com bearer no header e responde `initialize` com
  `2024-11-05`, `Mcp-Session-Id` e `tools/list`.
- Token ausente/incorreto retorna rejeição de autenticação em ambos quando o
  modo seguro está ativo.
- Os dois listeners usam o mesmo Tower instance ID e não registram MCP local
  como cliente outbound (self-loop proibido).

O query bearer atualmente aceito pelo scaffold é compatibilidade insegura a
remover no epic 40/v1-05; secure/release mode rejeita token em URL.

## Compatibilidade

- `grok-oss agent serve` permanece compatível como modo individual legado.
- As variáveis `GROK_OSS_APP_SERVER` e `GROK_OSS_MCP_HTTP` continuam aceitas
  para compatibilidade, mas não são necessárias para `tower`.
- Features `app-server-ws` e `mcp-streamable-http` permanecem nos defaults do
  binário.
