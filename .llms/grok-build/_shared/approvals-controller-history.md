# Approvals, controller leases and history decisions

[provenance: handoff §13, review D-AP.1..6, existing runtime permission paths]

## Controller lease

States are UNOWNED, HELD, RELEASED and RESOLVED. A HELD lease contains
Interaction ID, controller connection ID, lease revision and expiry. Acquisition
uses compare-and-set on revision; only the holder may renew/respond. Disconnect
or expiry releases the lease, never the Interaction. Resolution is exactly once
and terminal. The bearer grants eligibility but is not itself the lease.

## Identity

Interaction ID is stable runtime identity and survives retry/reconnect. JSON-RPC
request ID correlates one delivery on one connection. A resent Interaction gets
a new request ID and the same Interaction ID. Idempotency key identifies a
response mutation and returns its original resolution on retry.

## Disconnect/default policy

Default `[PROPOSED]` headless policy is wait until Interaction deadline, then
explicit auto-deny. Auto-allow is forbidden. Another authenticated controller
may acquire after release. This default requires human approval before the
headless release epic; it does not block wire/scaffold freeze.

## History authority

Canonical session files remain MVP history authority. SQLite projection is
rebuildable indexing for replay/search after `30/v1-05`; it cannot authorize
execution or repair missing canonical events by invention. Divergence causes
rebuild or `history_unavailable`.

Cursor binds Tower instance, Session ID, history epoch, filters, snapshot
boundary and expiry. Example: cursor `(instance A, session S, epoch 4, seq 99)`
is invalid in instance B, epoch 5, or different filter. `eventSeq=99` without
the epoch is insufficient.

Named tests: `controller_lease_compare_and_set`,
`request_id_changes_while_interaction_id_survives`,
`disconnect_never_auto_allows`, `projection_rebuild_preserves_cursor_order`,
and `cursor_is_bound_to_instance_epoch_and_filters`.

## Human decisions

- `(HUMAN, product-decision, blocking: headless release policy)`: approve wait
  then auto-deny deadline/default duration.
- `(HUMAN, product-decision, blocking: compatibility adapter only)`: decide
  missing-jsonrpc behavior in the future Codex adapter.
- Safe defaults until those decisions: fail closed; native JSON-RPC strict.
