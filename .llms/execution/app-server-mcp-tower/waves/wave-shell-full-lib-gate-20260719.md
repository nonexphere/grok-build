# Wave — Shell full library gate (2026-07-19)

## Correções

- O teste Claude compat agora configura o override que o crate Workspace
  realmente lê, mantendo a fixture cross-crate hermética.
- O teste de classificação de workspace cria a fixture em `/var/tmp`, porque
  `/tmp` é um Git worktree neste ambiente e, portanto, corretamente classifica
  descendentes como `git`.

## Validação

```text
CARGO_BUILD_JOBS=1 cargo test -p xai-grok-shell --lib
PASS: 5688 testes, 0 falhas, 7 ignorados

git diff --check
PASS
```

O gate amplo do Shell está verde. Os integration tests de sampling também
passam (28 testes) após a atualização da assinatura de Responses.
