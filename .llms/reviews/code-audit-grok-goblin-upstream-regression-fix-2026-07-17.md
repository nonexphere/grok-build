# Code Audit — grok-goblin upstream regression fix — 2026-07-17

## Resumo Executivo

- **Alvo**: commit `74f0531` sobre o upstream sincronizado `98c3b24`, com foco no suporte Codex/multi-provider.
- **Escopo**: catálogo de modelos e seus filtros, publicidade/seleção de métodos ACP de autenticação, resolução de slugs de subagentes, caminhos de refresh/rebuild do catálogo e regressões Codex já conhecidas.
- **Evidence level**: alto — diff completo, produtores e consumidores, contratos locais e testes direcionados disponíveis.
- **Veredito do commit**: **FAIL parcial**. O commit corrige completamente o bypass de filtros do catálogo e a inconsistência de slug de subagente. Ele corrige o método ACP errado para Codex, mas deixa o usuário Codex-only cair no login interativo xAI.
- **Release-blocking?**: sim para uma experiência Codex-only suportada no TUI; além disso, dois blockers Codex anteriores continuam abertos.

## Mapa Arquitetural Relevante

1. `ModelsManager` constrói e atualiza o catálogo chamando `resolve_model_catalog` no startup, reload de config, refresh remoto e reload de cache.
2. `resolve_model_catalog` combina os modelos xAI/config com entradas Codex credential-scoped e aplica `disabled_models`, `allowed_models`, `hidden_models` e overrides de effort.
3. `MvpAgent::initialize` chama `should_advertise_xai_api_key`, combina o resultado com a existência de token xAI e retorna `auth_methods` ACP.
4. O pager usa **o primeiro** `auth_method` para decidir `needs_login`; `grok.com`/OIDC exige login interativo, enquanto `xai.api_key` e `cached_token` não exigem.
5. A autenticação real de requests Codex não usa `xai.api_key`: usa o binding provider/credential e `BearerResolver`/`TokenManager` em request time.

## Resultado dos Bugs Declarados no Commit

### Catálogo Codex contornava filtros — RESOLVIDO

- **Evidence**: `crates/codegen/xai-grok-shell/src/agent/models.rs:1953-2003` agora executa merge Codex antes de disabled/allowed/hidden e effort.
- Todos os caminhos de produção de `ModelsManager` encontrados usam `resolve_model_catalog`: startup (`:238`), config reload (`:286`), rebuild/refresh (`:559`).
- O teste `allowed_models_applies_to_merged_codex_entries` atravessa o merge real por override one-shot e passou.
- Por inspeção, `disabled_models` usa `retain` depois do merge e `hidden_models` marca as entradas depois do merge, portanto os três filtros cobrem Codex.
- **Limitação**: só `allowed_models` ganhou regression test Codex específico; disabled/hidden estão corretos por fluxo estático, mas sem proteção dedicada.

### Binding Codex anunciava `xai.api_key` — SINTOMA RESOLVIDO, FLUXO INCOMPLETO

- **Evidence**: `crates/codegen/xai-grok-shell/src/agent/auth_method.rs:68-76` passou de `has_own_credentials()` para `own_credential()`, portanto um binding Codex sem API key estática não anuncia mais falsamente `xai.api_key`.
- Isso evita autenticar uma sessão OAuth Codex através de um método ACP cujo nome e contrato são de API key xAI.
- Entretanto, não existe método ACP Codex alternativo nem bypass explícito de login para uma credencial Codex já persistida. Ver finding High abaixo.

### Testes de colisão de slug em Task.model — RESOLVIDO

- **Evidence**: os testes em `agent/subagent/tests/mod.rs:3323-3365` agora refletem a resolução last-wins usada para slugs não multi-provider e mantêm rejeição quando apenas a entrada indisponível casa.
- Os seis testes `fresh_tool_model*` passaram.
- A ambiguidade multi-account Codex continua fail-closed no resolver dedicado e não foi convertida em last-wins.

## Achados Critical / High

### [High][Confirmed] Usuário com somente login Codex é enviado ao login xAI no TUI

- **Type**: bug / contract mismatch
- **Component**: `agent/auth_method.rs`, `mvp_agent/acp_agent.rs`, `xai-grok-pager/acp/mod.rs`
- **Evidence**:
  - `auth_method.rs:59-76` exclui corretamente bindings Codex de `has_external_api_key`.
  - `acp_agent.rs:226-229,304-323` fornece ao builder apenas `has_external_api_key`, `has_cached_token` xAI e configuração OIDC; não há `has_codex_credential` ou método Codex.
  - `auth_method.rs:130-139,145-173` faz o caso sem BYOK e sem token xAI terminar com o método interativo `grok.com` e nenhum default autenticado.
  - `xai-grok-pager/src/acp/mod.rs:582-605` interpreta o primeiro método interativo como `needs_login=true`.
  - Não foi encontrado nenhum `AuthMethodId` ACP Codex no shell ou pager.
- **Failure Scenario**: usuário executa `goblin login --provider codex`, não possui `XAI_API_KEY` nem sessão xAI, inicia o TUI e escolhe/tem um modelo Codex. O catálogo contém a credencial Codex, mas o initialize retorna somente `grok.com`; o pager abre o fluxo de login xAI antes de permitir o uso da credencial já válida.
- **Impact**: quebra o onboarding e o uso independente do provider Codex no principal frontend interativo. Headless ou usuários que também possuem auth xAI podem mascarar o defeito.
- **Validation**: teste de composição shell→ACP response→`startup_auth_metadata` com um catálogo contendo somente binding Codex e sem auth xAI/BYOK. O resultado esperado deve ser `needs_login=false` e não pode mentir usando `xai.api_key`.
- **Fix Direction**: modelar Codex como método/autenticação ACP real ou separar o gate de startup do login xAI quando o modelo selecionado possui binding provider autenticável. Não reintroduzir o falso `xai.api_key`.
- **Handoff**: `@issue-lifecycle` seguido de `@implementation-loop`.

### [High][Confirmed] Recovery multi-provider ainda usa fallback FIFO em quebra de invariante

- **Type**: data-consistency / concurrency
- **Component**: `auth/multi_provider_resolve.rs`
- **Evidence**: `crates/codegen/xai-grok-shell/src/auth/multi_provider_resolve.rs:82-95` ainda usa `take_stamp_for_recovery()` quando o attempt id falta ou o stamp exato não é encontrado.
- **Failure Scenario**: requests Codex concorrentes possuem stamps distintos; um erro sem ID ou com ID já consumido retira o stamp da outra request.
- **Impact**: recovery/refresh associado à geração ou conta errada.
- **Validation**: enfileirar outro stamp e verificar que attempt ausente/desconhecido falha fechado sem consumi-lo.
- **Fix Direction**: remover FIFO do caminho multi-provider; tratar ausência/miss como erro de invariante.
- **Handoff**: issue existente `data-001` → `@implementation-loop`.

### [High][Confirmed] Cache key de compactação continua sem provider/credential

- **Type**: contract-drift / account isolation
- **Component**: compaction + sampling types
- **Evidence**: `session/helpers/session_compact.rs:492-496` preenche a chave antes do sampler; `conversation.rs:2804-2812` deriva sem binding; `conversation.rs:2419-2426` preserva a chave existente e impede a derivação account-scoped do sampler.
- **Failure Scenario**: duas credenciais Codex na mesma identidade de sessão/agent enviam a mesma affinity key durante compactação.
- **Impact**: quebra o contrato account-scoped de prompt cache.
- **Validation**: composição de compactação com mesma sessão e duas credenciais deve produzir chaves diferentes.
- **Fix Direction**: deixar a chave vazia até o sampler com binding ou passar provider/credential ao helper.
- **Handoff**: issue existente `data-002` → `@implementation-loop`.

## Achados Medium

### [Medium][Confirmed] Regression coverage dos filtros cobre allowed, mas não disabled/hidden

- **Type**: test gap
- **Component**: `agent/models.rs`
- **Evidence**: o commit declara correção para allowed, hidden e disabled, mas adiciona somente `allowed_models_applies_to_merged_codex_entries` (`models.rs:2943-3002`).
- **Failure Scenario**: uma futura mudança reposiciona apenas disabled ou hidden em relação ao merge; o teste de allowed permanece verde.
- **Impact**: possível reintrodução silenciosa de modelos administrativamente bloqueados ou ocultos.
- **Validation**: testes com report Codex injetado devem provar remoção por full catalog key e por wire slug em disabled, e invisibilidade por ambos os padrões em hidden.
- **Fix Direction**: parametrizar uma fixture Codex e cobrir os três filtros, incluindo refresh/rebuild.
- **Handoff**: `@issue-lifecycle` ou no mesmo implementation loop do finding High de auth.

### [Medium][Confirmed] Não há teste do cenário Codex-only que o commit alterou

- **Type**: test gap
- **Component**: shell auth + pager startup
- **Evidence**: 21 testes de `agent::auth_method::tests` cobrem BYOK, token xAI, OIDC e usuário novo, mas nenhum constrói um catálogo somente Codex. Os 7 testes `startup_auth*` do pager também não recebem uma identidade Codex.
- **Impact**: o commit consegue passar toda a bateria enquanto quebra o principal cenário que pretendia corrigir.
- **Validation/Fix Direction**: adicionar teste de composição descrito no finding High, não dois mocks desconectados.
- **Handoff**: junto ao finding High de auth.

## Achados Low / Info

### [Low][Confirmed] Gate da branch inteira continua falhando em `git diff --check`

- **Type**: branch hygiene
- **Evidence**: `git diff --check 98c3b24..74f0531` reporta trailing whitespace em documentação previamente adicionada. O diff isolado `705a3b4..74f0531` não introduz novos hits.
- **Impact**: não invalida as três correções do commit, mas mantém o gate de readiness da branch vermelho.
- **Handoff**: issue existente `testing-002`.

## Segurança

A mudança de ordem do catálogo melhora enforcement: Codex não contorna mais allowlist/disabled/hidden. Não foi encontrado vazamento novo de token ou bypass de autenticação no diff `74f0531`. O finding de login Codex-only é funcional/contratual; a correção não deve voltar a representar OAuth Codex como API key xAI.

## Validação Executada

- `cargo test -p xai-grok-shell --lib --features native-multi-provider-auth agent::models::tests` — **66 passed**.
- `cargo test -p xai-grok-shell --lib --features native-multi-provider-auth agent::auth_method::tests` — **21 passed**.
- `cargo test -p xai-grok-shell --lib --features native-multi-provider-auth fresh_tool_model` — **6 passed**.
- `cargo test -p xai-grok-pager --lib startup_auth` — **7 passed**.
- `cargo check -q -p xai-grok-shell --features native-multi-provider-auth` — **pass**, com warnings preexistentes de imports/dead code.
- `git diff --check 705a3b4..74f0531` — sem erro no commit isolado.
- `git diff --check 98c3b24..74f0531` — falha por whitespace preexistente no conjunto da branch.

Não foi executado login Codex real nem TUI com credenciais reais; a finding Codex-only é confirmada pela composição determinística dos predicates e consumidores. O workspace inteiro não foi testado.

## Handoff Sugerido

1. Registrar e corrigir o gate Codex-only sem reintroduzir `xai.api_key` falso.
2. Fechar `data-001` e `data-002`, que continuam blockers independentes do upstream update.
3. Adicionar coverage Codex para disabled/hidden e para shell→pager auth startup.
4. Reconciliar claims de release somente depois dos testes sobre o SHA final.

## Referências

- Commit analisado: `74f0531512f49c440381a72fd606d69ce67dec8f`
- Base upstream: `98c3b24`
- `.agents/skills/add-provider/SKILL.md`
- `.agents/issues/data/data-001-codex-401-recovery-fifo-fallback.md`
- `.agents/issues/data/data-002-codex-compaction-cache-key-not-account-scoped.md`
- `.agents/issues/testing/testing-002-branch-diff-check-fails.md`
