---
name: create-pr
description: >-
  Abrir ou retargetar pull requests no fork Goblin (nonexphere/grok-build)
  com base sempre em `goblin`, nunca em `main` (espelho do upstream
  xai-org). Use quando o usuário pedir PR, /pr, abrir PR, retarget de PR,
  ou publicar branch de feature no GitHub deste fork.
---

# Create PR (Goblin fork)

## When To Use

- Abrir um PR no fork `nonexphere/grok-build`.
- Retargetar PR existente que aponta para `main` por engano.
- Publicar branch de feature após commits no trabalho Goblin.

### Quando NÃO usar

- PR **direto** em `xai-org/grok-build` (upstream) — sem permissão / fora da política do fork.
- Atualizar o espelho `main` a partir de feature work (use sync de upstream, não PR de feature → main).
- Commitar ou force-push no working tree sujo de **outro agente** na mesma worktree — use worktree isolada.

## Prerequisites

- Remotes configurados:
  - `origin` → `https://github.com/xai-org/grok-build.git` (upstream read)
  - `fork` → `git@github.com:nonexphere/grok-build.git` (push do fork)
- `gh` autenticado no owner do fork.
- Branch de feature com commits **ancestrais de `fork/goblin`** (rebase se o histórico divergiu).

## Branch policy (normative)

| Branch | Remote tip | Role |
|--------|------------|------|
| **`main`** | `origin/main` (= `fork/main` espelhado) | **Somente** espelho do upstream. Não recebe feature PRs. |
| **`goblin`** | `fork/goblin` | **Branch principal do fork** / integração. Base de todos os PRs. |
| **feature/** `goblin-*` | `fork/<feature>` | Trabalho; PR **into `goblin`**. |

```text
xai-org/main  ──sync──►  fork/main  (mirror only)
                              │
                         fork/goblin  (fork principal)
                              ▲
                         feature PRs
```

## Responsibility Boundary

**Do:**

- Garantir `fork/main` == `origin/main` antes de criar/atualizar `goblin` se necessário.
- Criar/atualizar `goblin` a partir de `main` (upstream tip).
- Abrir PRs com `--base goblin --head <feature>`.
- Em worktree **suja de outro agente**: rebase/cherry-pick em **worktree isolada**; force-push só no remote feature.

**Do not:**

- Abrir PR com base `main` no fork.
- `git checkout` / `reset` / `stash` no worktree do outro agente sem pedido explícito.
- Force-push em `goblin` com WIP de feature (só avança `goblin` via merge do PR).
- Commitar arquivos do outro agente no seu PR.

## Complementaridade

| Skill | Papel |
|-------|--------|
| **@create-pr** (esta) | Política de PR do fork Goblin |
| `@implementation-loop` | Implementar a feature |
| `gh` / git | Execução |

## Workflow

### 1. Sync mirror `main` (não mexe no working tree atual)

```bash
git fetch origin
git fetch fork
# Atualiza só o ponteiro local (branch NÃO checked-out)
git branch -f main origin/main
git push fork main --force-with-lease   # fork/main espelha upstream
```

### 2. Garantir branch `goblin` (integração)

```bash
# Se goblin ainda não existe ou deve realinhar com upstream tip:
git branch -f goblin origin/main
git push fork goblin --force-with-lease   # só quando recriar a partir de main limpa
git branch -u fork/goblin goblin
```

> Depois que `goblin` tiver merges próprios, **não** force-reset em `origin/main`
> sem decisão humana — sync vira merge/rebase de `main` → `goblin`.

### 3. Feature branch em cima de `goblin`

Se a feature **não** tem histórico comum com `goblin` (upstream reescreveu “Publish…”):

```bash
# Worktree isolada — NÃO usar a worktree suja do outro agente
WT=/tmp/goblin-pr-rebase-$$
git worktree add -B <feature>-rebased "$WT" goblin
cd "$WT"
git cherry-pick <feature-tip-sha>   # ou rebase da range
# resolver conflitos só aqui
git push fork HEAD:<feature-branch> --force-with-lease
cd -
git worktree remove "$WT"
```

Se já tem ancestral comum:

```bash
git push -u fork HEAD
```

### 4. Abrir PR

```bash
gh pr create --repo nonexphere/grok-build \
  --base goblin \
  --head <feature-branch> \
  --title "<title>" \
  --body "$(cat <<'EOF'
## Summary
- …

## Base
- **Base must be `goblin`** (fork integration). Never `main` (upstream mirror).

## Test plan
- [ ] …
EOF
)"
```

### 5. Retarget PR errado

```bash
# Se o PR ainda está OPEN:
gh pr edit <N> --repo nonexphere/grok-build --base goblin

# Se CLOSED e GraphQL recusa reopen/base change: abrir PR novo (passo 4).
```

## Stop Conditions

- Worktree com mudanças de outro agente e operação exige checkout/reset/stash → **parar**, usar worktree isolada.
- Force-push em `goblin` com conteúdo de feature incompleto → **não** fazer; merge via PR.
- Sem ancestral comum e cherry-pick conflita em massa → reportar e pedir estratégia de rebase.

## Conventions

- Nome de feature: `goblin-<topic>` (ex. `goblin-multi-provider-codex`).
- Commits: Conventional Commits; `feat(goblin): …` ok.
- Nunca PR para `xai-org` a partir desta política de fork.

## Common Mistakes

- PR `feature → main` no fork (mistura espelho upstream com produto Goblin).
- Force-push de `main` no fork **com** commits Goblin (main deixa de ser mirror).
- Rebase no working tree sujo compartilhado (apaga/conflita o trabalho do outro agente).
- Assumir que `gh pr edit --base` funciona em PR **CLOSED** (não funciona — recriar).

## Verification

- [ ] `git rev-parse main` == `git rev-parse origin/main`
- [ ] `git ls-remote fork refs/heads/main` == `origin/main`
- [ ] `gh pr view <N> --json baseRefName` → `"goblin"`
- [ ] Working tree do outro agente **inalterada** (`git status` na worktree original)
