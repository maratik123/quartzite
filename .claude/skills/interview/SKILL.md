---
name: interview
description: "Requirements interview with the product owner. Output: spec saved to ai-docs/plans/. Run before design or coding when task scope is unclear."
disable-model-invocation: true
argument-hint: "[issue-number]"
allowed-tools: Bash(gh issue view *)
---

Requirements interview with the product owner.
Output: refined task spec saved as `ai-docs/plans/YYYY-MM-DD-name.spec.md`.

> **MUST run before:** code investigation, design agent, or writing code.
> For the full workflow use `/task` (or `/task-issue` for an existing GitHub issue) — it includes interview as steps 2-4.

## Rules

1. **Max 3 questions** per round.
2. **Max 4 rounds** total.
3. Don't ask about the obvious — focus on edge cases, technical constraints, scope ambiguities.

## Workflow

1. Load the issue via `gh issue view $ARGUMENTS` — get description + comments.
2. Extract ALL scope items as a numbered list.
3. Show list to user: in scope / out of scope / deferred.
4. Question rounds (max 4):
   - Round 1: scope confirmation, key decisions
   - Round 2: edge cases, API backward compatibility
   - Round 3: technical constraints, trade-offs
   - Round 4 (if needed): final clarifications
5. Record the result → `ai-docs/plans/YYYY-MM-DD-name.spec.md`

## Spec format

```markdown
# [Task name]

**Issue:** [#number or URL]
**Date:** [YYYY-MM-DD]

## Scope
## Out of scope
## Deferred
- what | why | separate issue needed?

## Key decisions
| Question | Decision |
|---|---|

## Technical constraints

## Acceptance Criteria
| # | Criterion |
|---|-----------|
| AC1 | [specific, verifiable condition] |

## Open questions
```

## AC rules

- ✅ "Function returns `Err` if input is empty"
- ✅ "`parse()` returns correct value for valid UTF-8 input"
- ❌ "Test `foo_test` exists" — technical detail
- ❌ "`cargo test` passes green" — infrastructure requirement

## Anti-patterns

- 6+ questions at once
- Investigating code before the interview is done
- Stretching beyond 4 rounds
- Forgetting to save the spec before moving to design
