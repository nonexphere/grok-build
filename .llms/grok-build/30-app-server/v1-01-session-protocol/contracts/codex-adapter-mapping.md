# Isolated Codex compatibility mapping

This is the only native plan contract where the public Codex noun `thread` is
allowed. The experimental native protocol remains Session-based. Compatibility
adapter is not MVP and has no scaffold in this pass.
[provenance: handoff §13.0/§13.2, review D-SP.20/D-SP.25]

| Codex reference method/type | grok-oss native | Mapping rule |
|---|---|---|
| `thread/start` | `session/start` | rename ID/params; reject unsupported Codex-only fields |
| `thread/resume` | `session/resume` | preserve native Session ID |
| `thread/fork` | `session/fork` | new Session identity |
| `thread/read` | `session/read` | map snapshot nouns only |
| `thread/list` | `session/list` | cursor is adapter-owned; never reuse native cursor blindly |
| `thread/subscribe` | `session/subscribe` | map epoch/event sequence explicitly |
| Thread | Session | field-by-field adapter, no alias inside core types |
| Turn | Turn | status table required before adapter ships |
| ThreadItem | Item | unsupported item types surface adapter error, not silent drop |

Native JSON-RPC always requires `"jsonrpc":"2.0"`. Whether a future Codex
adapter accepts omitted `jsonrpc` is `(HUMAN, product-decision, blocking:
compatibility adapter only)`. Default is deny. This does not block native MVP.
