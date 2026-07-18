# Channel Gateways & Realtime Voice — backlog de produto

| Campo | Valor |
|-------|--------|
| **Status** | **Backlog / futuro** — **fora** do programa MVP App Server + MCP + Tower |
| **Data** | 2026-07-18 |
| **Produto** | grok-oss |
| **Relação** | Consome Tower/App Server/MCP quando existirem; **não** redefine o core |
| **Handoff core** | [`APP_SERVER_MCP_TOWER_HANDOFF.md`](./APP_SERVER_MCP_TOWER_HANDOFF.md) |

---

## 1. Recomendação (resposta direta)

| Pergunta | Resposta |
|----------|----------|
| Mexer nisso **agora** no plano App Server/MCP/Tower? | **Não** — deixar **fora** do escopo de implementação e dos épicos v1 |
| Criar arquivo separado? | **Sim** — este documento |
| Influencia o desenho do App Server/MCP? | **Só como constraint leve**: APIs estáveis de session/send/stream/interrupt + auth bearer + multi-session. **Não** exige Telegram/voice no MVP |
| Quando planejar de verdade? | **Depois** (ou em paralelo fraco) de: Tower daemon + App Server WS + MCP `tower_agent_*` + session lifecycle |

**Por quê deixar de fora agora**

1. Gateways (Telegram, etc.) e voz realtime são **adaptadores de canal** em cima do control plane — o “hard problem” atual é o **core** (session registry, WS, MCP, ACL, multi-tower).
2. Misturar Telegram/voice no mesmo plano **explode** superfície (auth de bot, webhooks, media, STT/TTS latência, half-duplex vs full-duplex, privacidade).
3. Se o core expõe bem **session start/send/history/stream/interrupt**, bridges viram clientes (como o SDK TS), não forks do runtime.
4. Já existe **voz parcial** no monorepo (`xai-grok-voice` + pager dictation) e **não** é o “falar e ouvir o agent em tempo real full duplex” que você descreveu — evoluir voz é programa próprio.

**O que o core deve *não* quebrar** (para não pintar a parede):

- Session addressable por id, com `send` + stream de eventos + interrupt.
- Auth de cliente (bearer) reutilizável por bridges.
- Multi-session no mesmo Tower (um bot Telegram pode mapear chat → session).
- Sem assumir que o único client é TUI/ACP.

---

## 2. Programa A — Channel Gateways (Telegram first)

### 2.1 Visão

Sistema de **gateways / bridges** para conectar a Tower a **outras plataformas e interfaces** (chat apps, bots, UIs externas).

**Primeiro target citado:** **Telegram** — bridge que sobe, autentica o bot/user flow, e conecta mensagens Telegram ↔ sessions na Tower.

### 2.2 Forma conceitual

```text
Telegram (users/chats)
        │
        v
  grok-oss-telegram-bridge  (processo ou módulo gateway)
        │  App Server WS e/ou MCP (tower_agent_*)
        v
  Tower (sessions multi-workspace)
```

### 2.3 Capacidades esperadas (futuro — não MVP core)

| Capability | Notas |
|------------|--------|
| Subir bridge com config (token bot, tower URL, bearer) | |
| Mapear chat/user Telegram → **session** grok-oss | criar/resume |
| Encaminhar mensagens texto (e depois mídia) | |
| Stream de resposta do agent de volta ao chat | |
| Interrupt / stop | |
| Multi-chat = multi-session | alinhado a multi-session Tower |
| Isolamento de secrets do bot | nunca no protocol log |

### 2.4 Outras plataformas (depois do Telegram)

Placeholder genérico: WhatsApp/Slack/Discord/web widget — **mesmo padrão gateway**, adapters por canal. Não detalhar até Telegram provar o padrão.

### 2.5 O que **não** é

- Não é substituir App Server.
- Não é MCP “especial” só de Telegram — preferir **client** do App Server/MCP existente.
- Não exige mudar `tower_agent_*` além de talvez metadata de origin (`channel: telegram`).

### 2.6 Dependências do core

| Core deliverable | Gateway precisa? |
|------------------|------------------|
| App Server WS + session/* | **Sim** |
| Bearer auth | **Sim** |
| MCP opcional | Útil; não mandatório se WS SDK bastar |
| Dashboard TUI | **Não** |
| Goal v2 | **Não** |

### 2.7 Decisões humanas futuras (quando for planejar)

- Bot API vs user client / MTProto  
- Um bot global vs por usuário  
- Onde roda o bridge (mesmo host da Tower vs cloud)  
- Policy de cwd/workspace default por chat  
- Aprovações (ask_user) no Telegram  

---

## 3. Programa B — Realtime voice (full duplex)

### 3.1 Visão

Falar com o agent e **ouvir resposta em tempo real** (loop de voz), no **hardware** do usuário — não só ditado para o prompt box.

### 3.2 Baseline no repo (hoje)

| Peça | Estado |
|------|--------|
| `xai-grok-voice` | STT/streaming pipeline, capture, probe |
| Pager voice | Ditado → texto no prompt / dashboard dispatch (`voice/handle.rs`) |
| Full duplex agent speech + barge-in | **Não** é o produto descrito (ainda) |

### 3.3 Capacidades futuras (esboço)

| Capability | Notas |
|------------|--------|
| Captura mic contínua / VAD | hardware local |
| STT streaming | partials |
| Encaminhar utterance → session turn | via Tower/App Server ou path in-process TUI |
| TTS streaming da resposta do agent | |
| Barge-in / interrupt por voz | |
| Integração TUI e/ou bridge (Telegram voice notes depois) | canais separados |

### 3.4 Dependências do core

| Core | Voz precisa? |
|------|----------------|
| Session + turn stream estável | **Sim** (para falar com o agent de verdade) |
| Interrupt | **Sim** (barge-in) |
| Baixa latência de eventos | **Sim** — constraint de design do App Server (não bloquear deltas) |
| MCP | **Não** necessário para voice local |

### 3.5 Decisões humanas futuras

- Só TUI local vs também remote clients  
- Provider STT/TTS (xAI / OpenAI / local)  
- Privacidade (áudio nunca sobe vs cloud STT)  
- Relação com voice notes do Telegram (programa A ∩ B)  

---

## 4. Como encaixa no roadmap (sem misturar planos)

```text
AGORA (programa core — handoff Tower)
  App Server + MCP + tower_agent_* + multi-session + SDK TS
        │
        │  (APIs estáveis)
        ▼
DEPOIS (este doc)
  ├── Channel gateways → Telegram bridge first
  └── Realtime voice full duplex (evoluir xai-grok-voice + clients)
```

| Programa | Prioridade relativa sugerida |
|----------|------------------------------|
| Core Tower/App Server/MCP | **P0** |
| Telegram gateway | **P2** (após core usável) |
| Realtime voice | **P2** (pode paralelizar research cedo; ship depois do stream estável) |

**Não** criar épicos v1 em `.llms/grok-build/app-server/` para Telegram/voice.  
Quando for a hora: árvore própria, ex. `.llms/grok-build/channel-gateways/` e `.llms/grok-build/realtime-voice/`, ou epics sob este doc.

---

## 5. Influência mínima no plano do App Server (checklist para o Codex)

Ao planejar o **core**, o Codex **pode** anotar (1 linha cada), **sem** implementar:

1. Events de session/turn com **backpressure** ok para clients lentos (bridges/voice).  
2. Campo opcional de **client/channel identity** em initialize (ex. `telegram-bridge`, `voice-client`).  
3. Não acoplar session lifecycle à TUI/dashboard.  
4. Interrupt e stream deltas devem ser first-class (voz e chat usam).  

**Não** inventar plugin system de gateway no MVP core.

---

## 6. Transcrição da intenção (2026-07-18)

> Além do App Server/MCP/Tower, depois vamos ter sistema de **gateways** para conectar outras plataformas/interfaces — praticamente conectar no **Telegram**: bridge para subir e conectar. Arquivo separado; acho que não mexe nisso agora — não influencia no plano do App Server/MCP; deixar de fora?  
> Também **voz em tempo real**: falar com o agent e ele responder em tempo real; mecanismos no **hardware**.

**Interpretação canônica:** dois programas futuros; core primeiro; este arquivo é o placeholder.

---

## 7. Próximo passo (quando o humano autorizar)

1. Core Tower shippable.  
2. Spike Telegram bridge como **client** App Server (1 chat → 1 session).  
3. Spike voice: full-duplex mínimo em cima de session stream + `xai-grok-voice`.  
4. Só então épicos detalhados e implementação.
