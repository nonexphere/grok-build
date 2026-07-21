# Wave — CI conformance jobs (2026-07-20)

## Objetivo

Transformar a distinção entre conformance fake e integração product-backed em
jobs CI explícitos, evitando que um resultado `SKIP` seja tratado como sucesso.

## Alterações

Em `.github/workflows/ci-grok-oss.yml` foram adicionados:

- `fake-conformance`: protocol, App Server conformance e matriz HTTP MCP.
- `product-integration`: composição real do pager-bin e cliente rmcp contra o
  launcher stdio real.
- Todos os quatro gates Cargo usam `--locked` para impedir atualização
  silenciosa do lockfile em CI.

Todos os passos usam comandos `cargo test` que falham naturalmente em erro;
nenhum passo tem `continue-on-error` ou conversão de SKIP em PASS.

## Limite

Os jobs ainda não foram executados pelo GitHub Actions nesta sessão. Proteção
de branch, required-check names e disponibilidade dos executáveis no ambiente
CI precisam ser verificados após o próximo push/PR.

## Validação local dos comandos locked

Os comandos do job `fake-conformance` foram executados localmente com
`--locked`: 22 testes de protocolo, 10 testes App Server filtrados por
`conformance` e 41 testes MCP HTTP passaram.
