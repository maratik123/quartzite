---
name: context-reset
description: "Handoff protocol for large tasks (>=5 subtasks). Prevents context degradation and compaction-related quality loss."
when_to_use: "Activate automatically after completing 3rd subtask when total >= 5, when a summary/compaction block appears at the top of context, or when noticing yourself rushing, simplifying, or skipping steps."
allowed-tools: Bash(cargo build)
---

## Handoff protocol

When triggered (N=3, M>=5 OR compaction detected):

1. `cargo build` — ensure code compiles
2. Update `ai-docs/plans/YYYY-MM-DD-name.progress.md` (format below)
3. Launch: `Agent(subagent_type="general-purpose", prompt="Read ai-docs/plans/YYYY-MM-DD-name.progress.md and continue")`
4. Remaining subtasks — one per Agent call, not batched
5. Do NOT continue in current context

## Checkpoint handoff: 1 Agent = 1 subtask

- Do NOT ask "continue?" between subtasks — just proceed
- Each Agent = 1 subtask, ending with `cargo build`
- Update progress.md after each Agent

## `.progress.md` format (canonical)

This is the single source of truth for the progress-file format. `/task`, `/code-review`, and the `review-findings` / `self-review` agents all read and write it; the **required** fields below must be present in every progress file regardless of which workflow created it.

```markdown
# Progress: [task name] — ACTIVE
_Updated: YYYY-MM-DD HH:MM_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** [branch name]
**base_commit:** [git rev-parse HEAD output]
**Last build:** PASS / FAIL / not run

<!-- Optional, /task only — omit for /code-review: -->
**Issue:** [#number or URL]
**Spec:** ai-docs/plans/YYYY-MM-DD-name.spec.md

## Next action

**Do this immediately:** [one concrete sentence — file + what to do]

## Subtasks

- [x] 1. done thing
- [x] 2. done thing
- [ ] 3. current/next thing  ← CURRENT
- [ ] 4. pending

## Key discoveries (don't re-investigate)

- [finding]: [why it matters / what we decided]

## AC Status

| AC | Status |
|----|--------|
| AC1 | PASS / FAIL / NOT_TESTED |

## Files touched

- `src/foo.rs` — what changed
```

**Required fields** (read by `self-review` at handoff): `**Branch:**`, `**base_commit:**`, `**Last build:**`.
**Optional fields** (added by `/task` only): `**Issue:**`, `**Spec:**`.

## Rules

1. Progress file = `ai-docs/plans/*.progress.md`. Updated at each checkpoint.
2. On context reset: pass file path in Agent prompt: `"Read ai-docs/plans/YYYY-MM-DD-name.progress.md and continue"`
3. `cargo build` BEFORE handoff — don't pass broken code
4. Maximum 3 handoffs per task. If more needed — the task is too large, decompose.
