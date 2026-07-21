# 20 — Tower Core

## O que é

Promoção do leader atual a daemon/control plane nomeado **Tower**, proprietário
do lifecycle de instâncias e do registry multi-session.

## Estado atual

`connect_or_spawn`, `run_leader_server`, roster `x.ai/sessions/*`, dashboard e
SessionActors residentes constituem o proto-Tower. Falta identidade multi-
instance, workspace arbitrário por start e facade estável para novos clients.

## Issues conhecidos

- leader é tratado como substrate de TUI/ACP, não produto control plane;
- discovery/state ainda pressupõe UX default mais singleton;
- resource telemetry/caps não têm contrato; cap não será enforced no MVP.

## Epics

- [v1-01-leader-characterization-promotion](./v1-01-leader-characterization-promotion/)
- [v1-02-multi-session-workspace-registry](./v1-02-multi-session-workspace-registry/)
- [v1-03-multi-instance-daemon-modes](./v1-03-multi-instance-daemon-modes/)
- [v1-04-operations-hardening](./v1-04-operations-hardening/)
- [v1-05-tower-supervisor](./v1-05-tower-supervisor/) — `grok-oss tower`
- [v1-06-canonical-session-actor-runtime](./v1-06-canonical-session-actor-runtime/) — P0 actor product-wired
- [v1-07-lifecycle-metadata-recovery](./v1-07-lifecycle-metadata-recovery/) — state/metadata/recovery truth
- [v1-08-product-session-host](./v1-08-product-session-host/) — ACP/LocalSet boundary for the real product actor (P0, pending)
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
