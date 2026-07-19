# Round-5 verification — resolution

Date: 2026-07-19  
Branch: `goblin-implement-epic-tree`  
Source review: `app-server-mcp-tower-verification-round-5-2026-07-19.md`

## Verdict

**All R5-01..R5-11 local findings addressed.** External items (TLS, live creds, npm) remain EXTERNAL and are not claimed DONE.

| Finding | Resolution |
|---|---|
| R5-01 | Product composition uses `ShellSessionActorRuntime::new`; echo factory is test-only |
| R5-02 | start/resume/idempotent share residency_result; hard spawn fails start; unsupported soft |
| R5-03 | claim_idempotency Won/Existing; speculative delete; cross-runtime concurrent test |
| R5-04 | unique epoch_{uuid}; rotate_history_epoch; fork new epoch |
| R5-05 | gap fixture; more only when remaining events |
| R5-06 | DEFAULT_MAX_SESSION_EVENTS circular buffer; min_retained expired cursor |
| R5-07 | evict_expired_sessions_and_interrupt |
| R5-08 | format_startup_token_status = "present" only |
| R5-09 | interaction_delivery_hub on SessionHandle; from_handle shares; ask_user_question dual-wait |
| R5-10 | STATUS/BLOCKERS/FINAL_REPORT/DECISIONS reconciled |
| R5-11 | intentional git checkpoint commit |

## Tests (sample)

- r5_runtime_correctness: 3 ok  
- r4_runtime_correctness: 4 ok  
- c6_respond_interaction: 10 ok  
- c7_conformance: 18 ok  
- mcp streamable_http r5_*: 2 ok  
- pager-bin r4_startup + composition initialize: ok  
