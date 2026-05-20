# Design: Enable model-side auto-invocation for /ui-design skill

**Issue:** #510
**Date:** 2026-05-20

## Approach

The change is instruction-file-only across two surfaces:

1. `.claude/skills/ui-design/SKILL.md` — flip the frontmatter from user-only to model-invocable, rewrite `description:` to embed the explicit trigger keywords the model uses for auto-pick, and add the 5th trigger bullet (`quartzite-paint-api`) to the prose body.
2. `AGENTS.md` § *Design system* — add the same 5th bullet so the workspace-level trigger list stays in sync with the SKILL.md body (the informal sync-group surface called out in the spec).

No code, no tests, no CI, no migration. Verification is grep + `wc -c`.

### Chosen form for the frontmatter flip: `disable-model-invocation: false`

The spec's Key-decisions row (line 33) left this open. Live state of `.claude/skills/*/SKILL.md`:

| Skill | Frontmatter line | Effective mode |
|---|---|---|
| `pr-ci-failed` | `disable-model-invocation: false` | model-invocable |
| `master-ci-failed` | `disable-model-invocation: false` | model-invocable |
| `bugfix`, `context-reset`, `interview` | _line absent_ | model-invocable (Claude Code default) |
| `ai-audit`, `code-review`, `improve`, `next`, `pr-commented`, `pr-merged`, `task`, `triage`, `verify`, `ui-design` (current) | `disable-model-invocation: true` | user-only |

Two patterns co-exist for "model-invocable": explicit `false` and absent line. **Choosing explicit `false`** because:

- It matches the only two skills in-tree that are *intentionally* model-invocable (`pr-ci-failed`, `master-ci-failed`). The "absent" skills (bugfix, context-reset, interview) inherited the default rather than declared it.
- The flip is the load-bearing intent of this whole task. Spelling it out makes the diff self-documenting and prevents an `/ai-audit` Checklist D inspector from having to infer "absence == model-invocable, on purpose".
- AC1 accepts either form; size delta is +1 char (`true` → `false`), well below any cap.

### Rejected alternatives

- **Drop the `disable-model-invocation:` line entirely.** Matches 3 skills' precedent (bugfix / context-reset / interview) but those are not the deliberate-model-invocable cluster. Spec leaves the choice to design; the deliberate cluster's pattern is the better precedent.
- **Inline visual rules into the SKILL.md body so the auto-invoked skill is self-contained.** Rejected — the spec's *Technical constraints* explicitly requires the skill stay pointer-only so it does not turn into a context-budget consumer on non-visual sessions. The body's Read instructions stay on-demand.
- **Edit `CLAUDE.md` § *Design system* in addition to AGENTS.md.** Rejected — CLAUDE.md is `@AGENTS.md`-import (spec line 23 confirms), so the edit propagates implicitly. Grep confirms `CLAUDE.md` has no second copy of the bullet list.

### Description-field wording strategy

The `description:` field is the model's auto-pick signal (per Claude Code docs cited in spec line 40). AC2 fixes the keyword set; design owns the prose. Strategy:

- One sentence stating the purpose (load design-system context for visual work).
- One clause naming the in-tree crate surfaces: `quartzite-style`, `quartzite-widgets`, `quartzite-paint-api`.
- One clause naming the in-tree symbol/concept surfaces: `Style impl`, `DefaultStyle`, `Palette`, `ColorRole`, `paint`, `snapshot`.
- Closing reminder that the skill is pointer-only (distinguishes from `design-system/SKILL.md` which is the actual visual-rules holder).

Draft (final wording the implementer should produce; design fixes the contract, not the byte-for-byte string):

> `Load Quartzite design-system context for visual / paint work — Style impl + DefaultStyle changes, Palette / ColorRole edits, paint paths in quartzite-style / quartzite-widgets / quartzite-paint-api, snapshot tests. Pointer-only: reads design-system/ on demand. Distinct from design-system/SKILL.md (name: quartzite-design), which is not slash-discoverable.`

Self-check vs AC2: `paint` ✓ (in `paint work`, `paint paths`), `Style impl` ✓, `DefaultStyle` ✓, `Palette` ✓, `ColorRole` ✓, `snapshot` ✓ (`snapshot tests`), `quartzite-style` ✓, `quartzite-widgets` ✓, `quartzite-paint-api` ✓. All 9 substrings present.

### 5th trigger bullet wording

Matching the existing 4 bullets' shape (`When working on …` / `When changing …` / `When adding or modifying …`):

> `- When working on quartzite-paint-api painter primitives, brush, pen, path, font, or color`

The bullet must appear in both AGENTS.md § *Design system* (after the 4 existing bullets) and the SKILL.md body trigger list (Subtask 1 expands the existing single-paragraph trigger sentence into a 5-bullet list to satisfy AC3 — see Subtask 1 below).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Update `.claude/skills/ui-design/SKILL.md`: flip frontmatter (`true` → `false`); rewrite `description:` field per the wording strategy above (AC1 + AC2); convert the body's single trigger sentence into a 5-bullet list including the new `quartzite-paint-api` bullet (AC3) | `.claude/skills/ui-design/SKILL.md` | — |
| 2 | Append the 5th `quartzite-paint-api` bullet to AGENTS.md § *Design system* trigger list (AC4); confirm AGENTS.md `wc -c` < 40 000 before staging (AC7); record pre/post `wc -c` delta in the commit message body | `AGENTS.md` | 1 (to keep the SKILL.md body bullet wording the source of truth for the AGENTS.md mirror) |
| 3 | Verification + commit: run all AC grep probes (AC1 / AC2 / AC6) and the size probe (AC7); confirm Checklist D wording untouched (AC8); confirm `name: ui-design` still present (AC5); stage explicitly (`git add .claude/skills/ui-design/SKILL.md AGENTS.md`); commit with size delta in the body | (no edits — verification only) | 1, 2 |

3 atomic subtasks. Below the 7-task split threshold; no issue split needed.

## Handoff plan

`M = 3` (one group of 3).

- **Group A:** subtasks 1–3 — terminal group (3 subtasks; within the 1..=3 cap). All three subtasks touch instruction files in `.claude/skills/ui-design/` and `AGENTS.md` and must commit atomically so AC4's "AGENTS.md bullet list matches SKILL.md body bullet list" invariant holds at every commit boundary.
- **Handoff at entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). This is the every-group-handoff contract called out in `.claude/skills/task/SKILL.md` Step 8 + `.claude/agents/design.md` § Rules → handoff-grouping. Group A is also the terminal group; no further handoff after Group A — completion follows /task Steps 9–12 in the same context.

## Risks

| Risk | Mitigation |
|---|---|
| Implementer drops the `disable-model-invocation` line entirely instead of flipping it to `false`. | AC1 explicitly accepts both forms; design's chosen form is recorded above as `false` for in-tree parity, but the spec's open question allows either. No machine check fails either way. |
| Bullet wording in AGENTS.md and SKILL.md body drifts. | Subtask 2 depends on Subtask 1 so SKILL.md body is the source of truth and AGENTS.md copies its phrasing verbatim. Subtask 3's AC4 check (visual diff between the two lists) catches drift before commit. |
| AGENTS.md crosses the 35 000-char early-warning threshold (currently 35 306 — already in the warning band per `wc -c`). | Adding one ~80-char bullet pushes total to ~35 390. Still ≥ 4 600 chars under the 40 000 hard cap. AC7 fails loud if the delta unexpectedly inflates (e.g., if the implementer over-elaborates the new bullet). Subtask 2 records pre + post `wc -c` for an audit trail. |
| Auto-invocation by the model spams context on non-visual sessions. | Spec's *Technical constraints* fixes the mitigation contract: SKILL.md body stays pointer-only (`Read in order: 1. design-system/SKILL.md …`) — the body itself is ~1 KB; design-system content reads happen only when the body's instructions fire. No change required from this design. |
| `/ai-audit` Checklist D ("`disable-model-invocation: true` ↔ skill is user-only") regresses. | The rule wording is bidirectional ("↔") and survives both forms: after this change `disable-model-invocation: false` ↔ skill is model-invocable, which still matches intent. AC8 explicitly requires the Checklist D string be untouched. |
| Slash-invocation path (`/ui-design`) breaks. | The slash path is resolved from `.claude/skills/<name>/SKILL.md` by Claude Code regardless of the `disable-model-invocation` flag — the flag only governs the auto-pick side. AC5 verifies `name: ui-design` still resolves. No code wiring to break. |

## Test Design

This is an instruction-file-only change. There is no `*.rs` diff, no `cargo test` surface. Verification is mechanical and lives in Subtask 3:

- **AC1 probe:** `grep -E '^disable-model-invocation:\s*true$' .claude/skills/ui-design/SKILL.md` → expect zero hits.
- **AC2 probes (9 substrings, case-insensitive, on the `description:` line only):**
  ```
  desc=$(grep -E '^description:' .claude/skills/ui-design/SKILL.md)
  for kw in paint 'Style impl' DefaultStyle Palette ColorRole snapshot quartzite-style quartzite-widgets quartzite-paint-api; do
    echo "$desc" | grep -qi -- "$kw" && echo "OK $kw" || echo "MISSING $kw"
  done
  ```
- **AC3 probe:** SKILL.md body contains 5 bullets / sentences naming the 5 trigger surfaces; visual inspection + `grep -c '^- ' .claude/skills/ui-design/SKILL.md` after the body-list conversion.
- **AC4 probe:** `diff <(sed -n '/^## Design system/,/^## /p' AGENTS.md | grep '^> -')` against the SKILL.md body bullet list (manual comparison; both lists must enumerate the same 5 surfaces, paraphrasing allowed).
- **AC5 probe:** `grep -E '^name: ui-design$' .claude/skills/ui-design/SKILL.md` → expect 1 hit.
- **AC6 probe:** `grep -rn 'disable-model-invocation' .claude/skills/` returns the same lines as before the change EXCEPT the single `ui-design/SKILL.md` line flipping `true` → `false`. Capture pre-change output before Subtask 1; diff after Subtask 3.
- **AC7 probe:** `wc -c AGENTS.md` before + after; record delta in commit message body; assert post < 40 000.
- **AC8 probe:** `grep -n "disable-model-invocation: true.*skill is user-only" .claude/skills/ai-audit/reference.md` → expect 1 hit (line 31), unchanged.

All probes run inside Subtask 3 before `git commit`.

### Self-review scope

`/task` Step 10 will spawn `self-review` against the diff. Anticipated reviewer concerns:

- "Why explicit `false` over dropping the line?" — Approach § *Chosen form* documents the precedent.
- "Why two-file edit and not three (CLAUDE.md too)?" — Approach § *Rejected alternatives* documents `@AGENTS.md`-import.
- "Does this trigger a Propagation Rule sync-group lookup?" — Spec line 43 confirms ui-design is NOT in any Propagation Rule sync group. The AGENTS.md ↔ SKILL.md body sync is informal (spec line 35).

## Open questions

None. All spec open questions (lines 60–62) are about exact wording, which design owns:

- **Description-field wording** — fixed in Approach § *Description-field wording strategy* above; AC2 keyword set verified.
- **5th trigger bullet wording** — fixed in Approach § *5th trigger bullet wording* above; the same wording is reused by Subtask 1 (SKILL.md body) and Subtask 2 (AGENTS.md).
