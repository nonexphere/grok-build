# Goal v1 Legacy Baseline

**Fonte de verdade.** Este contrato será preenchido por characterization do
Goal atual antes de qualquer refactor v2. Código/testes atuais vencem sobre
suposições deste draft.

## Superfícies a congelar

- `/goal` slash commands e mensagens;
- `get_goal`/`update_goal` schemas e blocked/completion thresholds;
- `GoalTracker`, snapshots/session files e restore;
- continuation trigger/queue/user interruption;
- TUI/pager, ACP e headless projections;
- subagent/worktree interaction existente.

## Compatibility rule

`goal_runtime=v1` deve reproduzir os goldens. `disabled` não inicializa/continua
goal automaticamente. `v2` falha explicitamente enquanto não for implementado;
nunca cai silenciosamente para v1.

## Evidence required

Cada linha do baseline final referencia path/symbol, behavior test, command e
resultado RED/GREEN. Este arquivo não define novos v2 semantics.

