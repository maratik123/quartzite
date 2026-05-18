# Wire Quartzite Design System into Claude Code agent context

**Source:** issue #462
**Date:** 2026-05-18
**Tracked in:** #462

## Scope

Wire the `design-system/` folder into the Claude Code agent prompts as a **conditional** context source — loaded on demand for visual work, never auto-imported. Four files are touched:

1. **`AGENTS.md`** — add a short "Design system" pointer section (≤ 1 024 bytes added). Names entry points (`design-system/SKILL.md`, `design-system/README.md`) and supporting subfolders (`preview/`, `colors_and_type.css`, `ui_kits/widgets/`). Lists the four trigger conditions verbatim from AC1: work on `quartzite-style` (any `Style` impl, including `DefaultStyle`); `quartzite-widgets` paint paths, widget views, or any user-facing rendering; changes to `Palette` / `ColorRole` semantics or seeds; adding or modifying snapshot tests under `quartzite-style/tests/snapshots/`. Does not quote design-system contents.
2. **`.claude/agents/design.md`** — add one bullet under `## Read before designing` naming both `design-system/SKILL.md` and `design-system/README.md`, qualified as "only when the task is visual" (UI-touching `quartzite-widgets` paint logic, `quartzite-style`, or any user-facing surface). Not a blanket load.
3. **`.claude/agents/design-review.md`** — extend Step 2 of `## Workflow` (`**Read context**`) to require `design-system/README.md` § VISUAL FOUNDATIONS + `design-system/colors_and_type.css` whenever the design under review touches a widget, paint code, or `Palette` / `ColorRole`. Deviations from documented rules (outline width, radius, derivation formulas, focus overlay) flagged as `major` blockers in the verdict — same severity bar as the existing handoff-grouping check.
4. **`.claude/skills/design/SKILL.md`** — new file (~10 lines). Frontmatter (`name: design`, short description) + minimal body that instructs the agent to Read `design-system/SKILL.md`, then `design-system/README.md`, then explore `design-system/preview/` and `design-system/ui_kits/widgets/` as needed. Pointer only — no inlined content.

Implementation lands on the `design-system` branch (master already merged in at commit e931e65). PR base will be `design-system`, not `master`.

## Out of scope

- Editing `CLAUDE.md`. No `@design-system/SKILL.md` import added. Auto-imported context size stays where it is today (`AGENTS.md` only).
- Editing `design-system/` contents themselves — source of truth is already correct.
- Changing the YAML frontmatter of `design-system/SKILL.md` (the `user-invocable: true` flag stays).
- Modifying other `.claude/agents/*` (e.g. `spec-writer`, `triage-runner`). Those do not currently make visual decisions; future gap → follow-up issue.
- Per-PR review automation that checks visual-rule deviations programmatically. Candidate follow-up issue once AC4 has been exercised in practice.
- Inlining the 19 KB `design-system/README.md` into `AGENTS.md`, `CLAUDE.md`, or any agent prompt. Every touchpoint references the design-system folder by path.

## Deferred

- Per-PR automated check for visual-rule deviations | premature before AC4 has been exercised in practice | yes — separate follow-up issue once AC4 has stabilised
- Wiring `spec-writer` / `triage-runner` / other non-visual agents to the design system | none of those currently make visual decisions | yes — file a follow-up the first time a real gap surfaces

## Key decisions

| Question | Decision |
|---|---|
| Auto-import via `CLAUDE.md` `@<file>` vs. conditional Read? | Conditional Read. `@` would unconditionally load `design-system/SKILL.md` (2 KB) plus its cascade-load of `design-system/README.md` (19 KB) on every session, pushing auto-imported context past 55 KB on tasks that need none of it. AGENTS.md is already ~35 KB. Conditional Read keeps zero-cost for unrelated work. |
| Inline visual rules into AGENTS.md? | No. Pointer-only. AGENTS.md adds ≤ 1 KB and names paths; agents Read the paths when the trigger conditions fire. Avoids duplicating the 19 KB README. |
| New slash command surface vs. reuse existing `design-system/SKILL.md`? | Add a new pointer at `.claude/skills/design/SKILL.md`. Claude Code's loader only discovers skills under `.claude/skills/<slug>/SKILL.md`, so the existing `design-system/SKILL.md` (with `user-invocable: true`) is not actually reachable as a slash command. The new file is a 10-line pointer that delegates to the canonical `design-system/SKILL.md`. |
| Slug for the new skill? | `design` (per the issue's expected file list — `.claude/skills/design/SKILL.md`). |
| Decomposition shape? | Recommend `3 + 1` — Group A = AGENTS.md + design.md + design-review.md (existing-agent-surface edits), Group B (terminal) = the new `.claude/skills/design/SKILL.md`. Design agent may instead merge the two agent edits into one subtask (3 terminal subtasks) and document the trade-off. `2 + 2` is not allowed under the handoff-grouping rule. |
| Severity bar for AC4 visual-rule deviation flagging? | `major` — same severity tier as the existing handoff-grouping checks in `.claude/agents/design-review.md`. |
| Branch / PR base? | Branch off `design-system`; PR base = `design-system`, not `master`. `master` has already been merged into `design-system` (commit e931e65). |

## Technical constraints

- **Byte budget (AC1):** `AGENTS.md` may grow by ≤ 1 024 bytes. Verify via `wc -c AGENTS.md` before and after, and `git diff --stat AGENTS.md`. AGENTS.md is currently 35 793 bytes; the 35 000-char early-warning threshold (from AGENTS.md § Build & Test instruction-file size axiom) is already passed, so any growth needs to stay tight.
- **No content duplication (AC6):** `wc -c` confirms only AGENTS.md grows, and by ≤ 1 KB. Grep confirms no `design-system/README.md` text appears inside `.claude/`.
- **`CLAUDE.md` unchanged (AC2):** `git diff CLAUDE.md` must be empty after the implementation.
- **No `@<file>` import added (NON-GOAL).** The pointer is read on demand via the agent's `Read` tool; never via `@`.
- **Behavioural sanity checks (AC7, AC8):** A non-visual top-level question ("next milestone in the roadmap?") must not trigger any Read under `design-system/`. A visual top-level question ("Slider widget visual contract?") must trigger Read of `AGENTS.md` pointer → then `design-system/SKILL.md` / `README.md`, and the eventual answer must cite the 1 px outline + no-radius + `Highlight` for pressed/checked + ×0.5 α disabled + additive 2 px focus rules.
- **Behavioural sanity check (AC3):** From a fresh `design` agent context, asking "What is the hover fill for `Button`?" on a UI-touching task must yield `Button.blend(Highlight, 0.25)` / `#BFDFFF` without priming.
- **Behavioural sanity check (AC4):** A synthetic design doc proposing 4 px radius on `Button` must surface a `major` issue row in the `design-review` verdict citing § VISUAL FOUNDATIONS.
- **Propagation Rule:** edits to `.claude/agents/design.md` and `.claude/agents/design-review.md` are in the Task/Design sync group (per AGENTS.md § Propagation Rule). Co-update sibling files in the same group only if the change to design.md / design-review.md is semantically connected to that group's contract; the AC1–AC5 edits here are localized additions to read-context bullets and one verdict-table rubric note — no co-edit of `.claude/skills/task/SKILL.md` / `.claude/skills/context-reset/SKILL.md` is implied. Confirm via grep at implementation time.
- **Slug uniqueness:** confirm `.claude/skills/design/` does not already exist before creating it; the existing skill list shows no `design` slug today, but design phase must re-verify.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `AGENTS.md` gains a "Design system" pointer section (one paragraph, ≤ 15 lines, ≤ 1 024 added bytes). Names `design-system/SKILL.md` and `design-system/README.md` plus supporting subfolders (`preview/`, `colors_and_type.css`, `ui_kits/widgets/`). Lists the four trigger conditions verbatim (any `Style` impl in `quartzite-style` including `DefaultStyle`; `quartzite-widgets` paint paths / widget views / user-facing rendering; `Palette` / `ColorRole` semantics or seeds; snapshot tests under `quartzite-style/tests/snapshots/`). Does not quote design-system contents. Verify: `grep -n 'design-system' AGENTS.md` returns the new section AND `git diff --stat AGENTS.md` shows ≤ 1 024 bytes added. |
| AC2 | `CLAUDE.md` unchanged. Verify: `git diff CLAUDE.md` is empty. |
| AC3 | `.claude/agents/design.md` lists `design-system/SKILL.md` and `design-system/README.md` under `## Read before designing`, qualified as "only when the task is visual" (touching `quartzite-widgets` paint logic, `quartzite-style`, or any user-facing surface). Verify behaviourally: fresh `design` agent on a UI task asked "hover fill for `Button`?" cites `Button.blend(Highlight, 0.25)` / `#BFDFFF` without priming. |
| AC4 | `.claude/agents/design-review.md` Step 2 `## Workflow` (`**Read context**`) requires reading `design-system/README.md` § VISUAL FOUNDATIONS + `design-system/colors_and_type.css` whenever the design touches a widget, paint code, or `Palette` / `ColorRole`. Deviations from documented rules (outline width, radius, derivation formulas, focus overlay) flagged `major` in the verdict — same severity bar as the existing handoff-grouping checks. Verify behaviourally: synthetic design proposing 4 px radius on `Button` yields a `major` row citing § VISUAL FOUNDATIONS. |
| AC5 | New file `.claude/skills/design/SKILL.md` exists. Frontmatter has `name: design` + short description. Body is ~10 lines instructing the agent to Read `design-system/SKILL.md`, then `design-system/README.md`, then explore `design-system/preview/` and `design-system/ui_kits/widgets/` as needed. Verify: slash-command dispatcher loads the new skill and returns its body. |
| AC6 | No content duplication. `wc -c AGENTS.md CLAUDE.md` confirms only AGENTS.md grew and by ≤ 1 KB. `grep -rn "VISUAL FOUNDATIONS\|Button.blend(Highlight" .claude/` returns no inlined design-system text. |
| AC7 | Sanity check — non-visual task. Top-level agent asked "What's the next milestone in the roadmap?" does not trigger any Read under `design-system/`. Verify behaviourally. |
| AC8 | Sanity check — visual task. Top-level agent asked "I want to add a new `Slider` widget — what's the visual contract I need to match?" Reads `AGENTS.md` pointer, then `design-system/SKILL.md` / `README.md`. Answer cites: 1 px outline, no radius, `Highlight` for pressed/checked, ×0.5 α disabled, additive 2 px focus. Verify behaviourally. |

## Open questions

- None. Issue body fully specifies scope, constraints, file list, verification plan, and even a decomposition recipe with explicit rationale. The 3 + 1 vs. merged-3 trade-off is a design-phase call (AGENTS.md § handoff-grouping rule), not a spec-affecting ambiguity.
