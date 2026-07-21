# Wave — product MCP stdio launcher (2026-07-20)

## Implementação

- `grok-oss tower --stdio` foi adicionado como modo explícito do supervisor.
- O launcher usa `run_mcp_stdio` e o mesmo runtime Shell/ACP product-wired,
  sem criar runtime Tower paralelo.
- O modo é feature-gated por `mcp-stdio`; HTTP continua separado por
  `mcp-streamable-http`.
- stdout recebe somente respostas JSON-RPC; EOF e diagnósticos vão para stderr.
- O agente padrão do launcher é `orchestrator`, com override por
  `GROK_OSS_TOWER_AGENT_TYPE`.

## Evidência

Script reproduzível:

```text
scripts/smoke/tower-mcp-stdio.sh
```

Smoke executado contra o binário real:

- `tools/list` retornou exatamente 9 ferramentas;
- `tools/call tower_agent_start` retornou `state=completed` e `structuredContent`;
- EOF terminou com código 0;
- stdout continha somente duas linhas JSON-RPC válidas;
- `mcp stdio eof` apareceu somente em stderr.

Também passou:

```text
cargo check -p xai-grok-pager-bin --features mcp-stdio --bin grok-oss
```

## Limites restantes

O smoke usa uma credencial XAI controlada e valida a fronteira do launcher;
turns product reais continuam sujeitos aos gates de provider/auth. Interop
com um SDK MCP independente e token scopes/TLS permanecem tasks separadas.
