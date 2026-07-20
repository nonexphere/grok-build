# Wave — ACP composition seam (2026-07-19)

## Mudanças

- `xai-grok-pager-bin` agora expõe
  `experimental_app_server_processor_with_acp_spawn(root)`.
- A função injeta `experimental_acp_resident_spawn` em
  `ShellSessionActorRuntime::with_production_spawn` e mantém a adaptação
  compartilhada `ShellRuntimeAdapter`.
- O construtor default continua usando runtime fail-closed sem factory; a nova
  seam não anuncia turn/steer/interrupt/Interaction antes dos gates restantes.
- O teste de composição cobre construção side-effect-free da seam.
- O teste MCP HTTP existente foi atualizado para o contrato atual: `agentType`
  é obrigatório em `tower_agent_start`.

## Validação

```text
cargo test -p xai-grok-pager-bin --bin goblin mcp_http_composition_bind_auth_and_dispatch_roundtrip
PASS (1 test)

git diff --check
PASS
```

O primeiro gate amplo revelou o fixture obsoleto e falhou com
`invalid_arguments: agentType required`; após a correção, o teste passou.

## Ainda pendente

Esta seam não prova prontidão do runtime: faltam capabilities verdadeiras,
actor canônico completo, persistência de todos os itens/turnos, semântica de
steer/interação, cancelamento observado e gates black-box/soak.

O clippy estrito do binário foi executado, mas falhou antes de alcançar esta
área por quatro diagnósticos preexistentes em `xai-grok-auth`:
`large_enum_variant`, `should_implement_trait` e dois `derivable_impls`. Esses
findings permanecem abertos em quality/release hardening e não foram alterados
nesta wave.
