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

