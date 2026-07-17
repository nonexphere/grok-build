# Tasks — v1-architecture-protocol

## Current-state characterization
- [ ] Mapear leader frames/routing, ACP, SessionHandle e tracker — Follow @repository-exploration
- [ ] Capturar wire/serde snapshots atuais relevantes
- [ ] Validar quais IDs existentes são estáveis
- [ ] Prototipar vertical ownership boundary sem production mutation

## ADRs
- [ ] Leader promotion e one-registry ownership
- [ ] Thread/Turn/Item/Interaction identity
- [ ] source-of-truth/projection e protocol strictness
- [ ] controller lease e remote threat model
- [ ] stable vs experimental method/capability inventory

## Protocol crate
- [ ] Criar JSON-RPC envelopes e error taxonomy
- [ ] Criar IDs/entities/statuses/input/model selection
- [ ] Criar initialize/capability negotiation
- [ ] Criar core thread/turn/item/approval method types

## Generation
- [ ] Gerar JSON Schema da fonte Rust
- [ ] Gerar TypeScript declarations e SDK skeleton
- [ ] Migrar/expandir JSONL examples como snapshots validados
- [ ] CI drift check e reproducible generation

## Robustez
- [ ] Serde round-trip/unknown/additive compatibility tests
- [ ] Fuzz malformed/oversized/deep payloads
- [ ] Secret/provider-neutral field review
- [ ] Cross-check schema/TS/examples parity

## Validação
- [ ] Protocol crate tests/check/docs green
- [ ] Compare generated bundle with `changes/` and explain deltas
- [ ] Independent contract review — Follow @code-review

## Specs e docs
- [ ] Atualizar app-server SPECS e protocol compatibility policy
- [ ] Atualizar root status/decision log

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar missing-jsonrpc compatibility — type: product-decision — blocking: compat listener
- [ ] (HUMAN) Aprovar stable/experimental surface — type: product-decision — blocking: public v1 freeze
