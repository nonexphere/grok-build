# Tasks — v1-approvals-control

## Interaction model
- [ ] Persistir Interaction ID/status/type/subject/deadline/controller epoch
- [ ] Separar interaction ID de per-connection JSON-RPC request ID
- [ ] Implementar first-valid-answer CAS e resolution notification
- [ ] Mapear runtime cancellation/timeout/already-resolved errors

## Controller leases
- [ ] Implementar observe/control access e claim/release/renew
- [ ] Aplicar epoch fencing a reverse responses
- [ ] Implementar disconnect/reclaim/failover policy
- [ ] Testar competing controllers e stale takeover

## Request types
- [ ] command execution approval com parsed safe display
- [ ] file-change e plan approval
- [ ] user input/questions e MCP elicitation
- [ ] resolution/reissue mapping para runtime primitive

## Grants e authority
- [ ] Integrar turn/session/always grants sem bypass de policy
- [ ] Scope grant por Thread/workspace/action/identity
- [ ] Restringir remote persistence conforme decisão
- [ ] Audit log sem secrets/raw hidden content

## Fault/security tests
- [ ] disconnect during request/response/effect
- [ ] duplicate/conflicting/stale answers
- [ ] controller takeover e observer mutation attempts
- [ ] hook denial/sandbox conflict permanece authoritative

## Validação
- [ ] E2E approval failover sem double execution
- [ ] Fake + shell runtime broker contract suite
- [ ] Focused checks/security review

## Specs e docs
- [ ] Controller/interaction state diagrams
- [ ] Atualizar SPECS/README/status

## Tarefas operacionais (humanas)
- [ ] (HUMAN) Aprovar controller election/reclaim policy — type: product-decision — blocking: stable failover
- [ ] (HUMAN) Aprovar remote `always` grants — type: product-decision — blocking: remote persistent option
