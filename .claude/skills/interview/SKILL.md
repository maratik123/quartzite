---
name: interview
description: "Requirements interview with the product owner. Output: spec saved to ai-docs/plans/, cross-linked with a tracking GitHub issue. Run before design or coding when task scope is unclear."
disable-model-invocation: true
argument-hint: "[issue-number]"
allowed-tools: Bash(gh issue view *) Bash(gh issue list *) Bash(gh issue create *) Bash(gh issue comment *)
---

Requirements interview with the product owner.
Output: refined task spec saved as `ai-docs/plans/YYYY-MM-DD-name.spec.md`, bidirectionally linked with a tracking GitHub issue.

> **MUST run before:** code investigation, design agent, or writing code.
> For the full workflow use `/task` (accepts either a description or a GitHub issue number) — it includes interview as steps 2-4.

## Rules

1. **Max 3 questions** per round.
2. **Max 4 rounds** total.
3. Don't ask about the obvious — focus on edge cases, technical constraints, scope ambiguities.

## Workflow

### Step 1: Detect entry mode

Inspect `$ARGUMENTS`:

- **Issue ref** — matches `^#?\d+$`: load it via `gh issue view <N>` (description + comments). Record `tracking_issue = <N>`.
- **Free text / empty**: use as the task description, or ask "What do you want to plan?" if empty. `tracking_issue` is unset for now — Step 5 resolves it.

### Step 2: Extract scope

Extract ALL scope items as a numbered list.

### Step 3: Confirm scope

Show list to user: in scope / out of scope / deferred.

### Step 4: Question rounds (max 4)

- Round 1: scope confirmation, key decisions
- Round 2: edge cases, API backward compatibility
- Round 3: technical constraints, trade-offs
- Round 4 (if needed): final clarifications

### Step 5: Resolve tracking issue

Every spec **must** carry a `**Tracked in:**` field referencing an open GitHub issue (`#N` shorthand).

- **Issue ref mode** (`tracking_issue` already set): use that issue. Skip the search.
- **Free text / interview mode**:
  1. Search existing open issues:
     ```bash
     gh issue list --state open --search "<keyword>"
     ```
     Use 1–3 keywords from the task scope (crate names, feature names, key types).
  2. **Candidate exists** → present to user: "I found #N: '<title>' — track this spec there?"
     - User confirms → use that issue.
     - User says no → continue.
  3. **No suitable issue** → propose a new one:
     - Title: matches the planned spec name (e.g. `feat(<crate>): <short description>`)
     - Body: brief background + scope items distilled from Steps 2–4
     - Show proposed title and body, ask user to confirm before running `gh issue create ...`
     - Capture the new issue number from the create output as `tracking_issue`.

> **Skip Step 5 only if the user explicitly states "no tracking issue".** Note the reason in the spec header (`**Tracked in:** none — <reason>`) and skip Step 7.

### Step 6: Save the spec

Write `ai-docs/plans/YYYY-MM-DD-name.spec.md` using the format below.

### Step 7: Cross-link the issue

Post a comment on the tracking issue pointing to the spec path:

```bash
gh issue comment <N> --body "Spec: \`ai-docs/plans/YYYY-MM-DD-name.spec.md\`"
```

This closes the loop: the spec references the issue via `**Tracked in:**`, and the issue references the spec file via the comment.

## Spec format

```markdown
# [Task name]

**Source:** user description | issue #<N>
**Date:** [YYYY-MM-DD]
**Tracked in:** #<N>

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

`**Source:**` is `issue #<N>` for issue-ref mode, `user description` otherwise. `**Tracked in:**` always uses the `#<N>` shorthand (no full URL).

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
- Saving the spec without `**Tracked in:**` (unless user explicitly opted out)
- Skipping the cross-link comment on the tracking issue
