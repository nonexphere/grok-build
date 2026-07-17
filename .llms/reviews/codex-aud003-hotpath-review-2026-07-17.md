# AUD-003 Wire Fidelity Hot-Path Review — 2026-07-17

**Auditor:** read-only review harness (Grok Build review subagent)  
**Repo:** `/home/guilherme/github/grok-goblin`  
**Mode:** read-only; no product code modified.  
**Scope:**
- `crates/codegen/xai-grok-sampling-types/src/conversation.rs`
  - `build_responses_input`
  - `try_build_responses_input`
  - `conversation_items_to_responses_input`
  - `FunctionCall` / `OpaqueWire` handling
  - tests `create_response_from_request_*`, `try_build_responses_input_errs_on_opaque*`
- `crates/codegen/xai-grok-sampler/src/client.rs` (~1930–1980) `From ConversationRequest` usage

**Execution note:** No shell-execution tool was available in this harness, so `cargo test` / `cargo check` were **not executed**. All evidence below is static source + test-definition evidence cited with `file:line`.

---

## 1. Executive verdict

| Criterion | Verdict | Evidence |
|-----------|---------|----------|
| **A. No silent `OpaqueWire` drop on production resend path** | **PASS** | Production resend goes through `(&req).into()` → `build_responses_input` → `try_build_responses_input` → `conversation_items_to_responses_input`. Unknown `OpaqueWire` returns `Err`; `build_responses_input` panics with the explicit `AUD-003 fail-loud` message. No filter/remove path exists. |
| **B. No double function-call emission** | **PASS** | `conversation_items_to_responses_input` detects any `FunctionCall` sibling and suppresses re-emission from `AssistantItem::tool_calls`. `response_to_conversation_items` dual-writes FC to the preceding assistant's `tool_calls` **and** as a sibling; resend uses siblings only, so only one FC is emitted. |
| **C. Tests drive `(&req).into() -> CreateResponse`** | **PASS** | Both `create_response_from_request_fails_loud_on_opaque_wire` and `create_response_from_request_single_fc_sibling_order` call `let _create: rs::CreateResponse = (&req).into();` / `let create: rs::CreateResponse = (&req).into();`. `try_build_responses_input_errs_on_opaque_no_silent_filter` exercises the fallible helper directly. |

**Overall: PASS** — the three acceptance criteria are met by source inspection.

**Residual risk:** one fidelity gap remains (see §4) and the test suite was not executed in this harness.

---

## 2. Detailed evidence

### 2.1 Production resend path is `(&req).into()`

`client.rs`:
- `conversation_stream_responses` at `client.rs:1934`:
  ```rust
  let responses_request: rs::CreateResponse = (&request).into();
  ```
- `conversation_responses` at `client.rs:1975`:
  ```rust
  let responses_request: rs::CreateResponse = (&request).into();
  ```

These are the only two non-streaming/streaming production entry points that convert a `ConversationRequest` to a Responses API request in the sampler client (confirmed by grep for `CreateResponse` in `client.rs`).

### 2.2 `OpaqueWire` cannot be silently dropped

Chain in `conversation.rs`:
- `From<&ConversationRequest> for rs::CreateResponse` (`conversation.rs:2547`) calls `build_responses_input(req)`.
- `build_responses_input` (`conversation.rs:2621`) unwraps `try_build_responses_input` and panics on error:
  ```rust
  fn build_responses_input(req: &ConversationRequest) -> rs::InputParam {
      match try_build_responses_input(req) {
          Ok(param) => param,
          Err(e) => panic!(
              "responses input convert failed (AUD-003 fail-loud, no silent drop): {e}"
          ),
      }
  }
  ```
- `try_build_responses_input` (`conversation.rs:2634`) calls `conversation_items_to_responses_input(&req.items)`.
- `conversation_items_to_responses_input` (`conversation.rs:2903`) handles `OpaqueWire` at `:2922`:
  ```rust
  ConversationItem::OpaqueWire(o) => {
      if o.type_name == "mcp_call" {
          if let Ok(mcp) = serde_json::from_value::<rs::MCPToolCall>(o.payload.clone()) {
              out.push(rs::InputItem::Item(rs::Item::McpCall(mcp)));
              continue;
          }
      }
      return Err(format!(
          "cannot resend opaque wire item type={} (not mapped to Responses input)",
          o.type_name
      ));
  }
  ```

There is no `_ => {}` or `.ok()` that would discard an unmapped `OpaqueWire`. The only outcomes are:
1. Known MCP payload → recovered as `InputItem::Item(McpCall)`.
2. Unknown payload → `Err` propagated and then panicked by `build_responses_input`.

Capture side also preserves unknown items: `response_to_conversation_items` (`conversation.rs:2060`) catches all unhandled `rs::OutputItem` variants in the `other` arm at `:2192` and stores them as `ConversationItem::OpaqueWire` with a warning.

### 2.3 No double function-call emission

Capture path:
- `response_to_conversation_items` (`conversation.rs:2060`) handles `rs::OutputItem::FunctionCall` at `:2133`:
  - Creates a `ToolCall`.
  - Projects it onto the most recent `AssistantItem::tool_calls` (`:2155–2160`).
  - **Also** pushes a `ConversationItem::FunctionCall(tc)` sibling (`:2163`).

Resend path:
- `conversation_items_to_responses_input` (`conversation.rs:2903`) computes a global guard at `:2906`:
  ```rust
  let has_fc_siblings = items
      .iter()
      .any(|i| matches!(i, ConversationItem::FunctionCall(_)));
  ```
- When `has_fc_siblings` is true, the `ConversationItem::Assistant(a)` arm at `:2912` emits **only** the assistant text message; it does **not** iterate over `a.tool_calls`.
- `ConversationItem::FunctionCall(tc)` is handled at `:2823` and emits exactly one `rs::InputItem::Item(rs::Item::FunctionCall(...))` per sibling.

Therefore a response captured with FC siblings resends as `assistant(content) → function_call → assistant(content)` in the original order, and the projected `assistant.tool_calls` are not re-emitted. The test `create_response_from_request_single_fc_sibling_order` asserts exactly one FC in the serialized body (`:4766`).

### 2.4 Tests exercise `(&req).into()`

`conversation.rs` tests:
- `create_response_from_request_fails_loud_on_opaque_wire` (`:4661`):
  ```rust
  let req = ConversationRequest::from_items(items).with_model("m");
  let _create: rs::CreateResponse = (&req).into();
  ```
  Annotated: "Real production entry: `From` → `build_responses_input` (not helper alone)."
- `create_response_from_request_single_fc_sibling_order` (`:4696`):
  ```rust
  let req = ConversationRequest::from_items(items).with_model("m");
  let create: rs::CreateResponse = (&req).into();
  ```
- `try_build_responses_input_errs_on_opaque_no_silent_filter` (`:4771`) directly tests the fallible helper, confirming it returns `Err` (not `Ok` with filtered items).

---

## 3. Findings

### 3.1 Accepted / no issue

| Finding | Severity | Status |
|---------|----------|--------|
| `OpaqueWire` is either recovered (MCP) or fail-loud; never silently dropped on the resend path. | — | Accepted |
| `FunctionCall` siblings prevent double FC emission; resend order matches capture order. | — | Accepted |
| Tests target the production `(&req).into()` entry point, not only internal helpers. | — | Accepted |

### 3.2 Residual risk — global `has_fc_siblings` guard can over-suppress legacy `assistant.tool_calls`

**Severity:** Medium  
**Confidence:** Medium  
**Location:** `conversation.rs:2906–2912`

The guard is computed once over the entire conversation:
```rust
let has_fc_siblings = items
    .iter()
    .any(|i| matches!(i, ConversationItem::FunctionCall(_)));
```

If **any** `FunctionCall` sibling exists anywhere in the history, **every** `AssistantItem::tool_calls` array is treated as a UI projection and skipped. This is correct for a fully modern session (where `response_to_conversation_items` dual-writes siblings), but it can silently drop legitimate client-executable tool calls from:
- Legacy `chat_history.jsonl` rows loaded as `AssistantItem` with `tool_calls` but no `FunctionCall` siblings (the JSONL loader upgrades reasoning/backend-tool calls but does not synthesize `FunctionCall` siblings from `assistant.tool_calls`).
- Mixed histories where an old session is continued after the new sibling format was introduced.

This is **not** a double-FC bug and it is **not** an `OpaqueWire` drop, so it does not violate the stated acceptance criteria. It is, however, a wire-fidelity gap: a resend could lose function calls that the model previously made, breaking context/prefix-cache alignment. A per-assistant guard (skip `tool_calls` only for assistants that are immediately followed by their own `FunctionCall` siblings) would remove this risk without re-introducing double emission.

No test currently covers a mixed legacy/modern history.

### 3.3 Tests not executed

**Severity:** Process  
**Confidence:** N/A  

This harness has no shell-execution tool, so the relevant unit tests were not run. The acceptance criteria are proven by reading the test definitions and the production conversion code. A local `cargo test -p xai-grok-sampling-types create_response_from_request try_build_responses_input_errs_on_opaque` should be run before merge to confirm the assertions pass.

---

## 4. Residual risk summary

1. **Mixed-history FC loss** — the global `has_fc_siblings` optimization avoids double FC but may drop legacy `assistant.tool_calls` if any modern `FunctionCall` sibling exists in the same conversation. Recommend adding a regression test for a mixed legacy/modern history and, if it fails, narrowing the guard to per-assistant scope.
2. **Unexecuted tests** — runtime proof of the three acceptance criteria was not obtained in this harness; source proof only.

---

## 5. Conclusion

**PASS** for the scoped AUD-003 acceptance criteria:
- A. No silent `OpaqueWire` drop on production resend path.
- B. No double function-call emission.
- C. Tests drive `(&req).into() -> CreateResponse`.

The code should land only after the relevant test command is executed successfully and the mixed-history FC-loss risk is either accepted or mitigated.
