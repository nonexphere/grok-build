# Channel Gateways — SPECS

Status: backlog documentation only. Future gateways consume Session identity,
runtime facade, Interaction, bearer security and replay cursor contracts. This
pass adds no Telegram schema, transport, crate or implementation. [D-BK.1,D-BK.2]

Gateway é client externo App Server/SDK (MCP opcional). Mapeia channel/chat para
Session, envia input, transmite Items e interrupt. Não altera runtime nem cria
plugin system no core.

Validação futura: multi-chat isolation, webhook/bot auth, redaction e reconnect.
