# Tower Instance and Session Lifecycle

**Fonte de verdade.** Tower Core possui este contrato. App Server, MCP e tools
consomem a mesma instance registry.

## Topologia

```text
máquina ── 0..N Towers
Tower   ── 0..N Sessions
Session ── exatamente 1 workspace + 0..1 Turn foreground
```

- Uma Tower pode criar sessions em qualquer workspace acessível pelo processo.
- Várias Towers coexistem por `instance_id`, endpoint, token e state dir.
- UX default: conectar à Tower default; se ausente, spawn; nova Tower somente
  com flag/instance explícita.
- Não há hard cap de sessions no MVP. Telemetria de atual/pico é desejável;
  quota enforcement só após dados reais.

## Session states

```text
absent ─start→ resident.idle ─turn→ resident.running ─done→ resident.idle
disk.dormant ─resume→ resident.idle
resident.* ─archive→ disk.archived
resident.running ─interrupt→ resident.idle|failed
```

`start` aceita workspace e agent type; `resume` reativa identidade persistida;
`fork` cria nova Session relacionada; `archive` não equivale a delete. Operações
mutáveis aceitam idempotency key e retornam erro typed em state/instance stale.

## Discovery

Discovery local é convenience, não singleton global. `[PROPOSED]` state de
instância em `~/.grok-oss/towers/<instance-id>/` contém metadata não secreta e
token em arquivo separado owner-only.

