# Tower Core — VISION

## Project Role

Ser o único control plane local/remoto de sessions grok-oss, promovendo o
leader sem redesenhar o runtime.

## Design Principles

1. Uma Tower, um registry, N Sessions.
2. N Towers podem coexistir sem singleton escondido.
3. Workspace pertence à Session, não à Tower.
4. Default simples; multiplicidade sempre explícita.
5. Dashboard ACP não migra no MVP.

## Out of Scope

Peer-to-peer sem control plane, quota enforcement e scheduling distribuído.

