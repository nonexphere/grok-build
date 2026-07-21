# Epic v1-05 — Supervisor `grok-oss tower`
Owner: Tower core/runtime owners
Escopo: conforme a seção Escopo deste epic

Status: concluído
Prioridade: lançamento-bloqueante
Depende de: `../v1-03-multi-instance-daemon-modes/`, `../../30-app-server/v1-04-websocket-remote-auth/`, `../../40-mcp-control-plane/v1-02-remote-security-conformance/`
Proveniência: `[provenance: user-input, conversation, code, doc-tree, inferred]`

## Escopo

### ADICIONAR

- subcomando `tower` como supervisor concorrente de App Server + MCP;
- `--no-mcp`, `--no-app-server`, `--bind`, `--mcp-bind`, `--secret`;
- lifecycle atômico, shutdown coordenado e diagnóstico seguro.

### REFACTORIZAR

- composição duplicada de signal wait/listener guards para uma função comum;
- defaults do produto para que ambos os transports estejam compilados e o
  supervisor seja o caminho recomendado.

### REMOVER

- comportamento em que as duas variáveis de ambiente precisavam ser ativadas
  manualmente para o modo combinado.

### NÃO alterar

- protocolo JSON-RPC, contratos de eventos, ACL Tower ou TLS externo;
- compatibilidade de `agent serve` individual.

## Tasks

- [x] Adicionar parser `tower` e validar exclusão dupla.
- [x] Definir binds padrão separados e permitir overrides independentes.
- [x] Centralizar resolução de secret opcional e redaction.
- [x] Iniciar App Server e MCP em tarefas concorrentes.
- [x] Fazer rollback se qualquer listener falhar ao bindar.
- [x] Implementar shutdown SIGINT/SIGTERM/SIGHUP e await dos joins.
- [x] Testar app-only, mcp-only e combined.
- [x] Testar colisão de portas e ausência de secret.
- [x] Testar auth com token compartilhado e token errado.
- [x] Atualizar help, README e exemplos operacionais.
- [x] Compilar, instalar e executar smoke real nos dois sockets.

## Aceite

`grok-oss tower` sobe ambos sem variáveis; cada `--no-*` remove apenas um;
nenhum estado parcial sobrevive a falha; os endpoints reais respondem aos
handshakes documentados; `cargo test`, build e instalação passam.
## Revisão de implementação

Este epic só pode ser executado quando cada task tiver owner, arquivos ou
contrato afetado, pré-condição, comando de validação e evidência esperada.
Alterações de comportamento exigem Red-Green-Refactor; alterações de contrato
exigem contract test e atualização da matriz de rastreabilidade.

### Gate mínimo

- [ ] dependências e links deste epic foram verificados;
- [ ] interfaces, schemas, estados, erros e compatibilidade estão definidos;
- [ ] caminho fake/conformance está separado do caminho product-backed;
- [ ] testes unitários, integração, black-box e segurança foram classificados;
- [ ] timeout, cancelamento, retry, restart e falhas parciais foram tratados;
- [ ] observabilidade, limites de recurso e redaction foram especificados;
- [ ] comando reproduzível e artefato de evidência foram registrados;
- [ ] bloqueios humanos/externos possuem owner e condição de desbloqueio;
- [ ] status do epic foi reconciliado com `TRACEABILITY.md` e `COMPLETION_COVERAGE.md`.
