# Handoff C1-F — Independent test review (GLM)

| Field | Value |
|---|---|
| Agent role | **review** |
| Model | `glm-5.2` |
| When | After C1-D lands and diff is stable |
| Capability | read-only preferred; may run tests if execute allowed |

## Must verify

- Named gates non-vacuous (`run-rust-test-gate.sh`)
- Real adapter tests are not FakeRuntime-only for production claims
- RED/GREEN evidence exists for behavior changes
- No canned-success mocks for SessionActor path

## Output

`.llms/execution/app-server-mcp-tower-corrective/reviews/c1/test-review.md`
