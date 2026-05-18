# Design: Wire Quartzite Design System into Claude Code agent context

**Issue:** #462
**Spec:** `ai-docs/plans/2026-05-18-wire-design-system-context.spec.md`
**Date:** 2026-05-18

## Approach

Wire `design-system/` into agent context as **pointer-only, conditional Read** across four prompt surfaces. The spec fully specifies the file list, the contents, the byte cap, and the verification plan; the design work is in sequencing, pre-extraction risk on `AGENTS.md`, and pinning the decomposition shape.

**Chosen shape:** `3 + 1` decomposition (Group A = three existing-surface edits, Group B terminal = one new-file creation). Rationale:

1. The three existing-file edits (`AGENTS.md`, `.claude/agents/design.md`, `.claude/agents/design-review.md`) form a coherent **wiring chunk** that exists in isolation: with only those landed, the conditional-Read trigger is fully described in the agent prompts. The new `.claude/skills/ui-design/SKILL.md` is a **discoverability convenience** (slash-command surface) — its absence does not break AC1–AC4.
2. Splitting into `3 + 1` keeps Group A's three subtasks atomic and independently committable, and isolates the new-file creation (Group B) so its slug-clash guard, frontmatter shape, and AC5 verification run against a clean diff.
3. The spec's alternative (merge the two agent edits → 3 terminal subtasks, M = 3) was rejected: the two agent files have **different rationales** (design.md = pre-design context priming for visual tasks; design-review.md = verdict-rubric extension with `major` severity). Merging them into one commit blurs the audit trail and complicates self-review.
4. `2 + 2` is forbidden by the handoff-grouping rule (non-terminal group must be exactly 3).

**Pre-extraction risk on AGENTS.md.** AGENTS.md is currently 35 793 bytes; adding up to 1 024 yields up to 36 817 bytes, past the proactive-extraction threshold of 35 000 documented in AGENTS.md § Build & Test (instruction-file size axiom) but still under the 40 000-byte hard cap. **Pre-extraction is NOT required** because the design caps the actual addition at ~970 bytes (see Subtask 1 budget below), which lands AGENTS.md at ~36 763 bytes — within an active budget of 4 207 (= 40 000 − 35 793). If the pointer section drafts above 970 bytes during implementation, the responsible course is to tighten wording before extracting any unrelated section — extraction is not part of this task's scope.

**Sync-group propagation.** Edits to `.claude/agents/design.md` and `.claude/agents/design-review.md` are members of the Task/Design sync group (per AGENTS.md § Propagation Rule). Verified at design time that the AC3/AC4 additions touch:

- design.md: one new bullet under `## Read before designing` (a pre-design context list).
- design-review.md: one extension to Step 2 of `## Workflow` (`**Read context**`) plus one new severity-rubric clause about visual-rule deviations.

Neither change modifies the design **artifact format**, the design-review **verdict format** (`GO` / `ITERATE` / `STOP`, the verdict table shape, severities `major` / `minor`), nor the handoff-grouping contract that `.claude/skills/task/SKILL.md` Step 8 and `.claude/skills/context-reset/SKILL.md` bind to. The spec's § Technical constraints "no co-edit" conclusion is re-verified and stands. The implementation MUST still run `grep -rn "Read before designing\|Read context" .claude/skills/ .claude/agents/` before commit to catch any drift introduced since this design was written.

**Agent Docs table row for the new skill.** Spec § Key decisions is silent on this; the design must decide. **Decision:** add a row for `.claude/skills/ui-design/SKILL.md` in AGENTS.md § Agent Docs, analogous to existing per-skill rows (`pr-commented`, `pr-ci-failed`, `master-ci-failed`, `triage`). Rationale: the row is the project's canonical "what skills exist" index; omitting the new skill creates discovery asymmetry. The row counts against the 1 024-byte cap for AC1 — see Subtask 1 byte budget below.

**Slug clash.** Verified at design time: `ls .claude/skills/` shows no `design` directory. Subtask 4 includes a defensive `test -e .claude/skills/ui-design && exit 1` pre-check (or equivalent) to guard against a race where the directory was created between design and implementation.

**Rejected alternatives:**

- **`@design-system/SKILL.md` auto-import in `CLAUDE.md`.** Rejected by spec § Key decisions: unconditionally loads ~21 KB on every session including non-visual tasks; AGENTS.md is already at 35 793 bytes and conditional Read keeps zero-cost for unrelated work. AC2 forbids any `CLAUDE.md` edit.
- **Inline `VISUAL FOUNDATIONS` content into AGENTS.md.** Rejected by spec § Out of scope and AC6 (no content duplication; grep gate). Pointer-only.
- **New shared "visual rules" doc under `ai-docs/`.** Rejected — `design-system/README.md` already IS that doc. Creating a parallel `ai-docs/visual-rules.md` would duplicate content and require a separate keep-in-sync rule.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add **Design system** pointer section to `AGENTS.md` (after `## Project`, before `## Permissions`), and add the new `.claude/skills/ui-design/SKILL.md` row to § Agent Docs table. Sum of both additions ≤ 1 024 bytes. | `AGENTS.md` | — |
| 2 | Add one bullet under `## Read before designing` in `.claude/agents/design.md` naming `design-system/SKILL.md` + `design-system/README.md`, qualified "only when the task is visual" with the three trigger surfaces (UI-touching `quartzite-widgets` paint logic, `quartzite-style`, any user-facing surface). | `.claude/agents/design.md` | — |
| 3 | Extend Step 2 (`**Read context**`) of `## Workflow` in `.claude/agents/design-review.md` to require reading `design-system/README.md` § VISUAL FOUNDATIONS + `design-system/colors_and_type.css` when the design touches a widget, paint code, or `Palette` / `ColorRole`; add severity-rubric clause: deviations from documented rules (outline width, radius, derivation formulas, focus overlay) = `major` (same tier as the handoff-grouping check already in Step 3). | `.claude/agents/design-review.md` | — |
| 4 | Create new pointer skill `.claude/skills/ui-design/SKILL.md` (~10 lines). Frontmatter `name: ui-design` + short description per AC5. Body instructs Read `design-system/SKILL.md` → `design-system/README.md` → explore `design-system/preview/` and `design-system/ui_kits/widgets/` as needed. Pre-check: confirm `.claude/skills/ui-design/` does not exist. | `.claude/skills/ui-design/SKILL.md` (new) | 1, 2, 3 |

**Subtask 1 — byte budget breakdown.** Pointer section (heading + ≤ 15-line paragraph naming entry points + four trigger conditions verbatim from AC1): target ≤ 850 bytes. New § Agent Docs row (one table line): target ≤ 120 bytes. Combined target: ≤ 970 bytes (with ~54-byte headroom against the 1 024 cap). If the draft overflows, prefer trimming prose over dropping the four trigger conditions (which are AC-bound verbatim).

**Atomicity check.** Each subtask produces a self-consistent diff: Subtask 1 leaves AGENTS.md internally coherent (pointer + table row both reference paths that exist today). Subtasks 2 and 3 are pure additions to existing agent prompts. Subtask 4 is a new file. Subtask 4 depends on 1–3 only for the conceptual ordering (the pointer skill should not exist before the prompts that justify it) — there is no compile-time or runtime dependency.

## Handoff plan

`M = 4` total subtasks. Two groups, `3 + 1`:

- **Group A:** subtasks 1–3 — the three existing-surface prompt edits (AGENTS.md pointer + Agent Docs row; design.md read-before bullet; design-review.md read-context extension + severity rubric). Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at the entry into Group A — every group fans out under the every-group handoff contract, including the first.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context. The handoff carries forward: progress-file delta from Subtasks 1–3 (`current_step`, `last_passed_gate`, tail of `## Decisions log`), branch state, base_commit invariance.
- **Group B:** subtask 4 — terminal group (1 subtask; within the `1..=3` range). Create the new `.claude/skills/ui-design/SKILL.md` pointer file. Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at the entry into Group B.

Boundary sizes: Group A = 3 (non-terminal, matches the required cap exactly). Group B = 1 (terminal, within `1..=3`). No group is `2`, no group exceeds `3`, no `2 + 2` split.

## Risks

- **AGENTS.md byte cap breach (AC1).** Cap is 1 024 bytes added. Even with the ~970-byte target, prose drift during writing can push it over. **Mitigation:** Subtask 1 acceptance gate includes `git diff --stat AGENTS.md` and `wc -c AGENTS.md` (before/after) verification before commit; if the diff exceeds 1 024 bytes, tighten the pointer paragraph before commit — do not pre-extract unrelated sections (out of scope).
- **AGENTS.md past the 35 000-byte proactive-extraction threshold.** The 35 000-byte threshold is already crossed; the new addition will push AGENTS.md to ~36 763 bytes. **Mitigation:** this is acceptable per the size axiom (the hard cap is 40 000, the warning is 35 000 — the addition keeps room for one full `/task` cycle of headroom). The follow-up extraction work (deferred-issue candidate) is **not** part of this task's scope.
- **Sync-group co-edit miss.** AC3/AC4 additions touch agent files in the Task/Design sync group. **Mitigation:** Subtasks 2 and 3 must run `grep -rn "Read before designing\|Read context\|design-system" .claude/skills/task/SKILL.md .claude/skills/context-reset/SKILL.md .claude/agents/design.md .claude/agents/design-review.md` before commit; the additions are localized to read-context bullets, so the grep should show only the just-added strings. After Subtask 4 lands the new `.claude/skills/ui-design/SKILL.md`, **widen the grep for future audits** to include that path as well: `grep -rn "Read before designing\|Read context\|design-system" .claude/skills/task/SKILL.md .claude/skills/context-reset/SKILL.md .claude/agents/design.md .claude/agents/design-review.md .claude/skills/ui-design/SKILL.md`. Re-verified at design time: no Task/Design sync sibling reads design-system today, and the additions do not alter the handoff-grouping / verdict-format / artifact-format contracts that the sync group enforces.
- **Slug clash with `.claude/skills/ui-design/`.** Verified at design time that the directory does not exist; race possible between design and implementation. **Mitigation:** Subtask 4 first action is `test ! -e .claude/skills/ui-design` (or equivalent), failing fast if present.
- **AC4 behavioural test (4 px radius on Button → `major` issue).** Severity rubric must integrate cleanly with the existing `major` / `minor` semantics in design-review.md § Verdict format and § Rules. **Mitigation:** Subtask 3 wording **lifts the same `major` / `minor` phrasing structure literally** from the existing handoff-grouping severity clause in Step 3 of the workflow — same anchor wording, same tier, no new severity concept introduced. Implementation must read Step 3's existing clause first and mirror its sentence shape so the verdict-rubric stays single-shape.
- **AC6 grep gate false positive.** The pointer paragraph in AGENTS.md may use phrases like `outline width` or `radius` that resemble inlined design-system content. **Mitigation:** Subtask 1 uses **pointer paths only** (`design-system/SKILL.md`, `design-system/README.md`, `design-system/colors_and_type.css`, `design-system/preview/`, `design-system/ui_kits/widgets/`) and lists the four trigger conditions from AC1 — none of which contain the literal strings the AC6 grep is keyed on (`VISUAL FOUNDATIONS`, `Button.blend(Highlight`). Run the AC6 grep before commit on Subtask 1.
- **`design-system/SKILL.md` `name: quartzite-design` vs. new skill `name: ui-design`.** The existing `design-system/SKILL.md` declares `name: quartzite-design`. The new `.claude/skills/ui-design/SKILL.md` declares `name: ui-design` (per AC5 + spec § Key decisions). **Mitigation:** the two names are distinct; no collision. The new skill is a *pointer* — its body delegates to the existing skill by Read, not by `name:` alias. Document this in the new skill's frontmatter description so a future reader does not assume they are the same skill.
- **No `@<file>` import drift.** A future contributor may "helpfully" add `@design-system/SKILL.md` to `CLAUDE.md`. **Mitigation:** AC2 + spec § Out of scope make the non-import explicit; not a design-phase concern beyond ensuring the implementation does not introduce it.

## Test Design

Verification is **behavioural** for ACs 3, 4, 7, 8 and **mechanical** (grep / wc / git diff) for ACs 1, 2, 5, 6. No Rust test code is touched by this task; this section enumerates the verification scripts each subtask owns at commit time.

**Subtask 1 — AGENTS.md** (AC1, AC2, AC6 partial):
- `wc -c AGENTS.md` — before / after delta ≤ 1 024 bytes.
- `git diff --stat AGENTS.md` — confirm the diff.
- `git diff CLAUDE.md` — empty (AC2).
- `grep -n 'design-system' AGENTS.md` — returns the new section + the new Agent Docs row.
- `grep -rn "VISUAL FOUNDATIONS\|Button.blend(Highlight" AGENTS.md` — empty (AC6).
- Visual scan: the four trigger conditions are present verbatim per AC1.

**Subtask 2 — design.md** (AC3):
- `grep -n 'design-system' .claude/agents/design.md` — returns the new bullet.
- `grep -n "Read before designing" .claude/agents/design.md` — anchor present, bullet directly under it.
- Behavioural (post-implementation, owned by the orchestrator): fresh `design` agent on a UI task asked "hover fill for `Button`?" cites `Button.blend(Highlight, 0.25)` / `#BFDFFF` without priming.

**Subtask 3 — design-review.md** (AC4):
- `grep -n 'design-system' .claude/agents/design-review.md` — returns the new Read-context extension.
- `grep -n "VISUAL FOUNDATIONS\|colors_and_type.css\|major" .claude/agents/design-review.md` — confirms the severity rubric clause aligns with the existing `major` tier.
- Behavioural (post-implementation): synthetic design proposing 4 px radius on `Button` yields a `major` row in the verdict citing § VISUAL FOUNDATIONS.

**Subtask 4 — new skill file** (AC5, AC6 final):
- `test -f .claude/skills/ui-design/SKILL.md` — file exists.
- `head -5 .claude/skills/ui-design/SKILL.md` — frontmatter has `name: ui-design` + short description.
- `wc -l .claude/skills/ui-design/SKILL.md` — body ~10 lines (loose check; AC5 says "~10 lines").
- `grep -rn "VISUAL FOUNDATIONS\|Button.blend(Highlight" .claude/` — empty (AC6 final gate; nothing inlined anywhere under `.claude/`).
- Behavioural (post-implementation, AC7 / AC8): non-visual top-level question triggers no Read under `design-system/`; visual top-level question Reads AGENTS.md pointer → `design-system/SKILL.md` / `README.md` and cites the 1 px outline + no-radius + `Highlight` for pressed/checked + ×0.5 α disabled + additive 2 px focus rules.

**Self-review per subtask.** Per AGENTS.md § Workflow AXIOM, every code-producing commit on this feature branch spawns `self-review` before `git push`. For docs-only edits the workspace AXIOM rates self-review optional, but the four subtasks here touch project-instruction surfaces that downstream agents read — treat self-review as required for each, with the diff scoped to the subtask's file.

## Open questions

None. The spec § Open questions notes "the 3 + 1 vs. merged-3 trade-off is a design-phase call" — this design pins `3 + 1` with the rationale above. No further questions for the product owner.
