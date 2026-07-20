# Product runtime readiness

**Fonte de verdade.** Este contrato define quando App Server, MCP e Tower estão ligados ao runtime Grok real e podem anunciar capacidade de execução.

Proveniência: [provenance: user-input, skill-output, code, doc-tree, inferred].

## Autoridade única

A composição em xai-grok-pager-bin deve construir uma única instância compartilhada do adapter Shell-backed e injetá-la no App Server, MCP Server e tools in-process. Tower e adapters nunca criam SessionActor, provider stack, permission engine, MCP client pool ou persistence paralelos.

O único caminho de criação/resume de actor é o factory canônico baseado em spawn_session_on_thread. O factory recebe dependências já resolvidas pela composition root:

- AuthManager e credencial/provider binding;
- AgentDefinition efetiva e capability Tower;
- ToolContext com cwd, trust, sandbox e permissions;
- GatewaySender, ModelsManager e SamplingConfig;
- PersistenceHandle e canonical session files;
- MCP servers outbound, WorkspaceOps e PluginRegistry;
- thread dedicada e LocalSet exigidos pelo actor.

Nenhum test echo, FakeRuntime ou factory experimental pode ser selecionado no binário de produto.

### ACP bridge boundary (2026-07-19)

The Shell-owned ACP host and experimental resident bridge now prove real
initialize/auth/session/prompt/cancel, live JSONL persistence, rollback, and
one-actor prompt serialization against `MockInferenceServer`. They are not the
canonical product actor factory: ACP has no native equivalent of Shell's
`SessionCommand::Interject`, and mapping Tower steer to a second
`session/prompt` would violate turn semantics. Until the canonical
`spawn_session_on_thread` actor factory owns steer/interactions and the product
black-box gate passes, turn/steer/interrupt/item/interaction capabilities stay
false and the experimental bridge must not be wired into the binary.

## Startup e readiness

Liveness significa processo vivo. Readiness significa que os listeners habilitados, registry, storage e actor factory estão disponíveis.

Se o actor factory não puder ser construído, o supervisor deve:

1. falhar startup antes de publicar readiness; ou
2. entrar em modo explicitamente read-only/degraded escolhido por flag.

O modo normal nunca aceita session/start como completed e depois falha o primeiro turn por wiring ausente. Capabilities e health/readiness devem refletir a capacidade real.

## Vertical slice obrigatório

O gate mínimo de produto executa, pelo binário real e por cada adapter:

1. initialize;
2. start Session com workspace, agent type, provider binding e sandbox;
3. start Turn com input estruturado;
4. observar user item, agent/tool events e status monotônicos;
5. wait/replay com epoch e cursor canônicos;
6. history equivalente ao stream;
7. interrupt de Turn ativo e corrida complete-versus-interrupt;
8. archive, dormant resume e restart;
9. verificar canonical transcript e ausência de secrets.

Rede externa pode ser substituída somente no boundary do provider/gateway por um double fiel. Registry, actor, files, permissions e adapters são reais.

## Capability truth

Uma capability só pode ser true quando o caminho product-wired correspondente passa o black-box gate. Scaffold, FakeRuntime, type existence e unit test não habilitam capability.

Capabilities obrigatórias: session list/read/start/resume/fork/archive/subscribe; turn start/steer/interrupt; item lifecycle/deltas; interactions configuradas; replay/resync. Capabilities indisponíveis devem ser false ou omitidas com erro stable de negotiation.

## Estados e falhas

Spawn failure não publica ready, não confirma idempotency claim e não deixa actor token/residency falso. Falha depois da criação de storage coloca Session em failed/dormant com diagnóstico seguro e retry policy explícita.

Runtime errors públicos usam catálogo canônico, retryability e operation ID. Diagnóstico interno de dependências nunca vira contrato permanente de cliente.

## Tests nomeados

- product_runtime_builds_canonical_actor
- no_product_fake_or_echo_path
- readiness_requires_actor_factory
- start_send_wait_history_product_vertical
- interrupt_complete_race_single_terminal
- restart_resume_preserves_identity
- capability_matrix_matches_product_paths
- spawn_failure_rolls_back_registry_and_claim
- provider_double_preserves_real_actor_boundary
