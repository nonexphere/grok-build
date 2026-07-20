# App Server + MCP Server + Tower — execução até prontidão de produto

## Goal Metadata

- Goal type: `execution-loop`
- Version: `20260719-continue`
- Owner/repo: `nonexphere/grok-build` / `/home/guilherme/github/grok-goblin`
- Local path: `/home/guilherme/github/grok-goblin`
- Primary branch: `goblin-implement-epic-tree`
- Task file: `/home/guilherme/github/grok-goblin/.llms/tasks/20260719-app-server-mcp-tower-completion.md`
- Expected duration: `multi-session`

## Objective

Revisar novamente a árvore de epics e executar tudo que for necessário para
tornar App Server, MCP Server, Tower runtime, nove tools e SDK TypeScript
funcionais e comprovados no produto. Antes de cada wave, verificar se o plano
continua alinhado ao código e atualizar contratos/tasks quando a evidência
mostrar uma lacuna. Continuar em ciclos de revisar → implementar → testar →
auditar até que todos os gates possíveis estejam verdes e os restantes estejam
honestamente bloqueados por decisão/estado externo.

## Success Criteria

- A revisão inicial atualiza `COMPLETION_COVERAGE.md`, contratos e tasks quando necessário.
- O composition root usa `SessionActor` real com dependências completas e runtime compartilhado.
- App Server, MCP e Tower têm conformance real em in-process, stdio, HTTP/SSE e WebSocket onde aplicável.
- As nove tools têm semântica, schemas, erros, ACL, replay, lifecycle e parity comprovados.
- SDK TypeScript é gerado, regenerável e testado contra listeners/subprocessos reais.
- Auth, scopes, tokens, TLS, limits, recovery, concorrência, observabilidade e release gates têm evidência.
- Nenhuma task é marcada concluída sem teste/evidência proporcional ao acceptance.

## Target And Context

- Repository/path: `/home/guilherme/github/grok-goblin`
- Primary plan: `.llms/grok-build/README.md`
- Coverage ledger: `.llms/grok-build/COMPLETION_COVERAGE.md`
- Final planning gate: `.llms/grok-build/COMPLETION_PLAN_GATE.json`
- Contracts: `.llms/grok-build/_shared/`, `docs/architecture/`, protocol schemas
- Existing audit: `.agents/reviews/code-audit-grok-goblin-2026-07-19.md`
- Execution evidence: `.llms/execution/`, `.agents/evidence/`, epic-local reports
- External systems/checks: provider credentials, remote TLS and publication are opt-in human gates

## Operating Mode

- Continue until the stop condition or a genuine blocked condition is reached.
- Re-read local `AGENTS.md`, the plan entry point and the selected epic before editing.
- Prefer direct execution in the current workspace; use isolated worktrees for rebase/PR operations only.
- Preserve unrelated dirty changes and never reset, stash or checkout them away.
- Ask the human only for product decisions, credentials, external authorization or destructive actions.

## Invariants

- Never claim product readiness from a fake-only test, `cargo check`, scaffold or static documentation.
- Every behavior change gets a failing regression/behavior test before the production fix when feasible.
- Generated sources are changed through their source definition and generation command.
- Public/cross-boundary contracts require schema and parity evidence before completion.
- Secrets, tokens, credentials and private infrastructure details never enter logs, fixtures or commits.
- Commits, when made, contain only the coherent wave and owned files; existing user changes stay untouched.

## Allowed Actions

- Read and update `.llms/grok-build`, `.llms/tasks`, `.llms/execution`, `.agents/evidence` and relevant project docs.
- Implement and test scoped Rust/TypeScript/transport/runtime changes required by the active epic.
- Add regression, contract, integration, black-box and smoke tests.
- Run package-scoped and repository-defined validation commands.
- Create reports, issue records and progress checkpoints needed for traceability.

## Forbidden Actions

- Do not use `git reset --hard`, destructive cleanup, broad deletion, force-checkout or overwrite another agent’s WIP.
- Do not deploy, publish packages, rotate credentials, change remote infrastructure or run destructive migrations without explicit authorization.
- Do not weaken schemas, gates, security, tests or acceptance criteria to make a wave pass.
- Do not mark blocked, skipped or fake-backed work as done.
- Do not expand into unrelated providers, UI, channels or voice work unless a contract dependency proves it is required.

## Mode-Specific Rules

This is an execution-loop with implementation-program discipline:

- Maintain one active coherent wave at a time and record its files, dependencies and gate.
- Before implementation, perform a fresh scan of the selected epic against current code and tests.
- If the scan discovers a new requirement, add it to the canonical coverage ledger and task file before coding.
- Use red-green-refactor for behavior fixes; run the narrowest gate first, then broader checks.
- After each wave, review the actual diff, update task state/evidence and continue to the next unblocked wave.
- Keep external/human gates explicit: remote TLS, provider live credentials, package publication and product decisions.
- Re-run an independent coverage/audit pass after each major program wave and at the terminal pass.

## Execution Model

Repeat this cycle:

1. Recover state: inspect git status/diff, current task checkboxes, execution logs, tests and active blockers.
2. Re-review plan: compare the full ledger and contracts with current implementation; patch docs/tasks for proven gaps.
3. Select the highest-priority unblocked wave, normally: actor runtime → lifecycle → conformance → MCP/tools → security → SDK → release.
4. Write or confirm the RED test and acceptance evidence.
5. Implement the smallest complete behavior unit.
6. Run targeted tests, package gates, black-box tests and relevant smoke tests.
7. Review diff and downstream contract impact; repair regressions without weakening gates.
8. Record evidence, mark only proven tasks done, and checkpoint the wave.
9. Run a fresh scan for newly exposed dead paths, capability drift, missing tests or contradictory docs.
10. Continue until every in-scope item is `done`, `blocked`, `deferred` or `superseded` with an explicit reason.

## Issue, Decision, And Blocker Rules

- Classify findings as confirmed actionable, decision-needed, duplicate/resolved/non-issue or external blocker.
- Materialize confirmed findings in the relevant epic/task or issue artifact before implementation.
- Mark `Human decision required: yes` only for product semantics, ownership, public breaking changes, credentials, remote security policy or external release authorization.
- A local test/environment failure is not automatically a blocker; diagnose and continue independent waves.
- If the same failure survives two hypothesis-driven fixes, record both hypotheses and change approach.

## Validation Plan

- `git diff --check` and targeted repository lint/format checks after every wave.
- Rust package tests/checks using `scripts/run-rust-test-gate.sh` for named tests.
- Protocol/schema/golden and generated-source drift checks.
- Product-backed actor vertical: start → send → wait → history → interrupt/archive → restart/rebind.
- App Server conformance across in-process, stdio and real WebSocket where implemented.
- MCP independent-client checks for stdio, Streamable HTTP and SSE lifecycle.
- Nine-tool semantic and ACL parity across in-process and MCP adapters.
- TypeScript clean regeneration, typecheck, package tests and real subprocess/listener black-box tests.
- Security tests for bearer/scopes/revocation/URL rejection/TLS/limits/redaction and concurrency/recovery.
- Human-product smoke using the installed `grok-oss` binary when the task requires it.
- If a validation cannot run, record command, exact reason, risk and substitute evidence; do not convert it to PASS.

## Memory And Artifacts

- Session/log path: `.llms/execution/app-server-mcp-tower/` and `.llms/execution/app-server-mcp-tower-corrective/`
- Reports: `.agents/reviews/`, `.llms/reviews/`, epic-local reports
- Decisions: `.llms/grok-build/COMPLETION_COVERAGE.md`, `_shared/`, decision records in execution logs
- Completion report: `.agents/evidence/app-server-mcp-tower-delivery-report.md`
- Final gate: update `.llms/grok-build/COMPLETION_PLAN_GATE.json` with implementation evidence and residual blocked items

## Stop Conditions

- All in-scope tasks are proven done, or terminal as blocked/deferred/superseded with owner and unblock condition.
- The final coverage ledger has no missing, contradictory or fake-only requirement.
- Product gates and release-hardening evidence are reconciled with actual current code.
- A final audit, diff review and validation report are complete.

## Blocked Conditions

- The required behavior conflicts with an external/public contract and no authorized migration exists.
- Progress on every remaining path requires the same unavailable human decision, credential or external system.
- Continuing would overwrite user work or require unsafe/destructive action.
- The repository cannot build/test due to an environment failure that remains after safe alternatives are exhausted.

## Final Report Requirements

- Final state and goal verdict.
- Epics/tasks completed, blocked, deferred or superseded.
- Files and artifacts changed.
- Exact commands/tests and results.
- Commits/PRs only if created under repository policy.
- Cross-repository and downstream impact.
- Residual risks, human gates and release blockers.
- Honest completion or blocked verdict; never imply implementation is complete without product evidence.
