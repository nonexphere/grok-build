# C0 — SessionActor / leader command map (characterization)

## Known entry points (from codebase scan)

| Facade method | Likely existing authority | Notes |
|---|---|---|
| list_sessions | `JsonlStorageAdapter::list_sessions` / roster | storage + leader roster |
| read_session | `load_session` / summary.json | persistence |
| start_session | ACP new session / leader session create | SessionActor spawn |
| resume_session | load + rehydrate actor | registry one-actor |
| fork_session | session fork path | copy_session_data |
| archive_session | hide/archive summary | not full delete |
| start_turn | `SessionActor::handle_prompt` / prompt queue | foreground turn |
| steer_turn | interjection / steer | active turn |
| interrupt_turn | interrupt/cancel turn | |
| respond_interaction | permission/elicitation response | no second engine |
| replay | updates JSONL + live subscriptions | history epoch |

## Gate
No production composition may inject FakeRuntime. Fake remains unit/conformance only.
