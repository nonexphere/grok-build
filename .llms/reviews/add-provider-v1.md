# Review: add-provider SKILL.md (v1)

## Verdict: PASS

Review method: direct adversarial review under `@create-skill`; no subagents were used,
as explicitly required by the user.

## Critical issues (must fix before use)

None.

## High-severity issues

None after correction.

## Medium-severity issues

- [M1, resolved] The initial draft described real implementation mistakes without an
  explicit evidence pointer. `SKILL.md` §Common Mistakes now cites the dated
  implementation review and finding ranges.
- [M2, resolved] The initial draft did not explicitly distinguish reproduced repository
  contracts from workflow enhancements. `SKILL.md` §Provenance now marks both.

## Low-severity / polish

- The provider skeleton intentionally contains `todo!()` placeholders because it is a
  structural template, not compilable generated code. The surrounding text explicitly
  requires replacement/removal and prevents it from being mistaken for completion.

## What's good (keep)

- Trigger and negative-trigger boundaries distinguish provider implementation from
  narrow bug fixes, review, custom API-key models, and architecture authoring.
- The workflow forces an end-to-end vertical slice before broad UI work.
- The checklist spans authorization, protocol, config, registry, login, storage,
  refresh, requests, models, CLI/TUI, compatibility, security, test layers, and evidence.
- Real observed failures are converted into mechanical prohibitions and release gates.
- Contract precedence, stop conditions, complementary skills, and severity/confidence
  conventions satisfy the universal teachings.
- The skill is followable: it includes a concrete module/trait/request-flow skeleton,
  executable verification categories, and real common mistakes.

## Fidelity check

- The request-time binding/token-manager/sampler rule faithfully reproduces D3/D4 and
  Phase 7 from `task.md`.
- Provider authorization fail-closed behavior faithfully reproduces D10/OQ/release gates.
- Storage crash consistency faithfully reproduces the refresh-race/fault requirements;
  it intentionally rejects the current two-file pseudo-atomic reference as a bug.
- Generic capability-driven CLI behavior faithfully reproduces G6/§4.3 and intentionally
  rejects the current provider-specific enum as a bug.
- Vertical-slice ordering and production-consumer search are documented enhancements.

## Absorption check

No previous `add-provider` skill existed in the global or project catalog. Nothing was
replaced or lost.

## Structural verification

- Frontmatter name matches `.agents/skills/add-provider`.
- Description contains what/when trigger keywords.
- Required sections and 3+ negative examples exist.
- Referenced checklist and template files exist and are linked.
- Detailed checklist/template content is decomposed without orphan files.
- The skill is project-scoped because it depends on Goblin-specific contracts and paths.

