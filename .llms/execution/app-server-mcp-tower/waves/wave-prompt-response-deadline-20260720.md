# Wave — prompt response deadline (2026-07-20)

## Objetivo

Impedir que App Server/MCP fique aguardando indefinidamente o `oneshot` de um
ator ou provedor que morreu, travou ou não concluiu a solicitação.

## Implementação

`ShellSessionActorRuntime::start_turn` agora aguarda a resposta com o limite
canônico `TOOL_WAIT_MAX_MS` (300 segundos). Ao expirar, limpa o turno corrente,
envia cancelamento cooperativo ao ator e retorna `runtime_unavailable`; não
produz um sucesso sintético.

Foi extraído `receive_prompt_response` para permitir teste determinístico com
deadline de 1 ms.

## Validação

`cargo fmt --manifest-path crates/codegen/xai-grok-shell/Cargo.toml -- --check`
e `git diff --check` passaram. Tanto a configuração padrão quanto
`--no-default-features` foram tentadas com timeout de 180 segundos; ambas
ficaram compilando dependências grandes do workspace e não produziram o
binário de teste. Portanto o teste determinístico da expiração continua
pendente e esta wave não é considerada validada.

O binário padrão posteriormente compilou e o teste de invariantes
`shell_session_actor_runtime_does_not_use_fake_runtime` passou. Esse artefato,
contudo, não contém `prompt_response_deadline_returns_timeout_without_fake_success`,
provavelmente por ter sido produzido antes da última edição/conclusão
concorrente; ele não serve como evidência do timeout.

## Limites

O deadline não substitui a fixture de provider real nem prova o fluxo completo
stdio `send → wait → history → interrupt`. Também não resolve a decisão de
capability/readiness quando credenciais ACP estão ausentes.
