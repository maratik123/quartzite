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

The full format spec lives in a dedicated reference file: **[progress-format.md](progress-format.md)** (template, required vs optional fields, lifecycle: created by `/task` or `/code-review`, extended by self-review and `/pr-commented`, gitignored, deleted by `/pr-merged`). Used by `/task`, `/code-review`, `/pr-commented`, and the `review-findings` / `self-review` agents.

## Rules

1. Progress file = `ai-docs/plans/*.progress.md`. Updated at each checkpoint.
2. On context reset: pass file path in Agent prompt: `"Read ai-docs/plans/YYYY-MM-DD-name.progress.md and continue"`
3. `cargo build` BEFORE handoff — don't pass broken code
4. Maximum 3 handoffs per task. If more needed — the task is too large, decompose.
