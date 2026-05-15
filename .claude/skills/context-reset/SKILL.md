---
name: context-reset
description: "Handoff protocol for large tasks (>=5 subtasks) AND compaction-recovery re-entry. Prevents context degradation and compaction-related quality loss."
when_to_use: "Activate automatically after completing 3rd subtask when total >= 5, when a summary/compaction block appears at the top of context, or when noticing yourself rushing, simplifying, or skipping steps."
allowed-tools: Bash(cargo build)
---

> **⚡ Compaction recovery check — read FIRST on every invocation.**
> If you are re-entering `/context-reset` after auto-compaction (a
> summary/compaction block appears at the top of context, or workflow
> context feels thin), STOP before any tool call and:
>
> 1. Identify the **parent workflow** (`/task`, `/code-review`, or
>    `/pr-commented`) whose handoff `/context-reset` is performing. The
>    parent's identity is recorded as the active progress file's
>    `parent_skill:` field (or `current_step:` mentions a `/task` /
>    `/code-review` / `/pr-commented` step name).
> 2. Run the parent skill's own compaction-recovery callout against its
>    durable-state file (see `/task`, `/code-review`, or `/pr-commented`
>    SKILL.md for the parent's variant). `/context-reset`'s body is the
>    shared handoff + re-prime action, not a separate durable surface.
>    (The parent's Variant-A callout ends with a "See `/context-reset`
>    § Compaction recovery (re-entry)" cross-link — that is a *reading*
>    link to the canonical rationale, **not** an instruction to re-run
>    this Variant-C callout. The chain terminates at the parent's active
>    subtask.)
> 3. After the parent's callout routes you to the active subtask, follow
>    `/context-reset`'s **Handoff protocol** below for the actual handoff
>    or re-prime action.
>
> This skill carries the canonical rationale below (§ **Compaction
> recovery (re-entry)**) for the other skills to cross-link to.

## Handoff protocol

When triggered (N=3, M>=5 OR compaction detected):

1. `cargo build` — ensure code compiles
2. Update `ai-docs/plans/YYYY-MM-DD-name.progress.md` (format below)
3. Launch: `Agent(subagent_type="general-purpose", prompt="Read ai-docs/plans/YYYY-MM-DD-name.progress.md and continue")`
4. Remaining subtasks — one per Agent call, not batched
5. Do NOT continue in current context

## Compaction recovery (re-entry)

Canonical rationale for the compaction-recovery callout that every code-side orchestrator SKILL.md (`/task`, `/code-review`, `/pr-commented`, `/bugfix`, `/interview`, `/context-reset`) places at the top of its body. Every Variant-A and Variant-B callout cross-links to this section by the exact h2 heading `## Compaction recovery (re-entry)`; this is the singular destination.

### Why a callout at all

Claude Code's per-skill truncation after auto-compaction keeps the **start** of `SKILL.md` and drops the rest. The compaction-recovery callout therefore lives at the very top of each code-side orchestrator SKILL.md so it survives even when the rest of the body is dropped at the per-skill 5,000-token cap. The user explicitly chose **heuristic self-detection over a `SessionStart|compact` hook** — the callout is the only mechanism; correctness rests on its wording.

### The Full-read-on-re-entry invariant

Every re-entry path (compaction recovery or otherwise) MUST re-read the durable-state file end-to-end before any tool calls, then re-enter the skill from the **top of its body** (preambles included). The recorded `current_step` is a hint for the human reader and a cross-check, NEVER an instruction to skip the read or jump straight to that step.

The invariant matters because:

- `/task` Step 1 is the interview phase (not the active-state probe — that lives in the `⚡ First` preamble above Step 1).
- `/code-review` Step 1 is "Determine branch" (not the RESUME probe).
- `/pr-commented` Step 1 is "Open / extend progress file" (preconditions run *above* the numbered steps).
- `/bugfix` Step 1 is "Reproduce and Trace" — re-running it on a confirmed trace re-asks the user to confirm what's already confirmed (Variant B carries an explicit skip rule for this case).
- `/interview` Step 1 is "Detect entry mode" — round counter lives in `.state.md`, not in Step 1.

Re-entering literally "from Step 1" would skip the active-state probes in three of six skills and re-trace in `/bugfix`. The wording "**re-enter the skill from the top of its body**" covers all six skills uniformly.

### Variant taxonomy

Three callout variants address the three structurally-different probe shapes:

- **Variant A** — `/task`, `/code-review`, `/pr-commented`. Probe lives in a `⚡ First` preamble that globs / greps for the active artefact. The callout routes via the probe (the preamble glob) then re-enters from the top of the body.
- **Variant B** — `/bugfix`, `/interview`. Probe is a fixed-glob (`ai-docs/bugfix/trace-*.md` or `<spec_path>.state.md`); when a single in-flight artefact exists the callout reads it and applies a per-skill resume rule (skip-Step-1 for `/bugfix` on a confirmed trace; resume from the recorded `round:` for `/interview`).
- **Variant C** — `/context-reset` itself. No own durable surface; routes to whichever parent skill (`/task` / `/code-review` / `/pr-commented`) is active. The callout above this section is Variant C.

The variant-distinguishing phrases that the char-cap / variant-identity audit greps for are kept in the design doc only (`ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md` § *Risks* row "Per-skill callout variants drift over time"), so the audit's expected hit-count of 1 per skill is preserved.

### Why the callout cross-links here

Concentrating the rationale in one place keeps the per-skill callouts short (each is ~25 lines). The cross-link `See .claude/skills/context-reset/SKILL.md § Compaction recovery (re-entry)` resolves to this exact anchor in every Variant-A and Variant-B callout. Variant C punts to the parent's callout instead of cross-linking back to this section (the chain terminates at the parent's active subtask).

## Checkpoint handoff: 1 Agent = 1 subtask

- Do NOT ask "continue?" between subtasks — just proceed
- Each Agent = 1 subtask, ending with `cargo build`
- Update progress.md after each Agent

## `.progress.md` format (canonical)

The full format spec lives in the shared-templates directory: **[`ai-docs/templates/progress-format.md`](../../../ai-docs/templates/progress-format.md)** (template, required vs optional fields, lifecycle by field, lifecycle by process, exemptions). Used by `/task`, `/code-review`, `/pr-commented`, `/bugfix`, and the `review-findings` / `self-review` agents. Required fields now include `**current_step:**`, `**last_passed_gate:**`, and a `## Decisions log` section in addition to the original `**Branch:**` / `**base_commit:**` / `**Last build:**`.

## Rules

1. Progress file = `ai-docs/plans/*.progress.md`. Updated at each checkpoint.
2. On context reset: pass file path in Agent prompt: `"Read ai-docs/plans/YYYY-MM-DD-name.progress.md and continue"`
3. `cargo build` BEFORE handoff — don't pass broken code
4. Maximum 3 handoffs per task. If more needed — the task is too large, decompose.
