# Design: Design-time handoff grouping for /task Step 8

**Issue:** #371
**Spec:** `ai-docs/plans/2026-05-16-design-handoff-groups.spec.md`
**Date:** 2026-05-16

## Approach

### Problem framing

Today `/task` Step 8 fires the `/context-reset` handoff via a **runtime
counter**: "I just completed subtask N of M; is N == 3 and M ≥ 5? If
yes, hand off." The gate is binding (per `reference.md` § *N=3 of M≥5
handoff gate (rationale)*), but the trigger boundary is recomputed
per-turn from the in-progress subtask number against the design's M.
That re-derivation is the failure surface — compaction mid-Step-8 has
silently dropped the gate at least once (PR #339, Step 10 omitted).

This spec moves the **trigger boundary into the design document**.
The design agent partitions the Decomposition into consecutive groups
of ≤ 3 subtasks each, names `/context-reset` as the handoff between
groups, and Step 8 reads that pre-computed plan instead of recomputing
N == 3 every turn. The gate itself (binding rule, severities,
`/context-reset` as destination) does not change — only the storage of
the trigger boundary does.

### Chosen representation for the handoff plan: **Option 1 — separate `## Handoff plan` h2 section**

The spec's open question lists three candidate shapes. I'm picking
**Option 1**: a new `## Handoff plan` section directly under
`## Decomposition`, prose-form listing each group's subtask range and
the handoff between groups.

| Option | Pros | Cons |
|---|---|---|
| **1. Separate `## Handoff plan` section** (chosen) | Zero impact on existing Decomposition-table consumers (`design-review`, future tooling); handoff destination (`/context-reset`) named in explicit prose at every boundary; one place to grep for the plan; size grows only by ~5 lines per group regardless of subtask count; trivially absent for M ≤ 4 designs (no section emitted → design-review's M ≤ 4 auto-PASS branch fires cleanly) | One extra section in the doc structure; the reader cross-references `#` numbers between the two sections |
| 2. Extra `Group` column on the Decomposition table | Group label co-located with each subtask; no cross-reference needed | Every existing Decomposition-table consumer (design-review parser, future tooling) must learn the new column; handoff destination is implicit (where does `/context-reset` get named?); column adds noise for M ≤ 4 designs where grouping is meaningless |
| 3. Header rows inside the Decomposition table marking group boundaries | Visually clean | Markdown tables do not natively express header-row separators — the convention would be ad-hoc (e.g., a row of `|---|`) and brittle to parse; rejected |

**Key tiebreaker:** option 1 preserves the existing
`| # | Task | Files | Depends on |` table shape (per spec
*Out of scope* item 5 — "no column-shape change"), names the handoff
destination explicitly in prose, and disappears cleanly when M ≤ 4.
Option 2 implicitly fights the spec's "no column-shape change" line.
Option 3 is brittle to parse and visually under-defined.

### Contract changes — three files

1. **`.claude/agents/design.md`** receives:
   - A new Rule (in the existing `## Rules` block) stating: "If
     Decomposition has M ≥ 5 subtasks, emit a `## Handoff plan`
     section; for M ≤ 4 omit it."
   - A new section in the artifact-format template showing where
     `## Handoff plan` lives and the prose shape inside.
   - An inline **synthetic example** in the artifact-format section so
     future readers see a compliant grouping without needing to track
     down this task's design.

2. **`.claude/agents/design-review.md`** receives:
   - One new checklist row in the `## Workflow` step-3 checklist
     covering: existence (when M ≥ 5), per-group cap (≤ 3 non-terminal),
     terminal group sized 1..=3, numbering consecutive, handoff
     destination `/context-reset`.
   - Explicit severities documented in the same row: missing grouping
     on M ≥ 5 = `major`; non-terminal group > 3 = `major`;
     cosmetic issues = `minor`; M ≤ 4 auto-passes.

3. **`.claude/skills/task/SKILL.md` Step 8 + `.claude/skills/task/reference.md` § N=3 of M≥5 handoff gate (rationale)** receives:
   - SKILL Step 8 sub-step 5 (the existing `**N=3 of M≥5 handoff gate (binding, not optional)**` bullet) gets a one-clause addition: "Read the design's `## Handoff plan` for the canonical boundary; the runtime N==3 check is now a cross-check against the design, not the source of truth."
   - `reference.md` § *N=3 of M≥5 handoff gate (rationale)* gets a paragraph: "The design's `## Handoff plan` section is the source of truth for trigger boundaries. The binding rule itself is unchanged — what changes is that the boundary is recorded in the design, not re-derived per turn." The existing prose ("Skipping this gate has caused…") stays.

### What this is NOT

- The N=3, M≥5 thresholds do not change.
- `/context-reset`'s own protocol is untouched.
- No hook-level enforcement is added (gate stays rule-enforced).
- No new column in the Decomposition table.
- No promotion to an AGENTS.md AXIOM (deferred per spec).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add grouping requirement + new `## Handoff plan` section to the artifact-format template in `design.md`, plus a new Rule entry; include the inline synthetic example demonstrating an M=5 grouped design. **The Rule entry MUST enumerate all four AC1 wording sub-points explicitly** (per design-review note 1 — implementation contract, not just verification recipe): (a) when grouping is required — `M ≥ 5`; (b) maximum group size — `3 consecutive subtasks`; (c) handoff destination — `/context-reset` (named in prose at every boundary); (d) terminal-group sizing — `1..=3` (last group may be smaller than the cap). | `.claude/agents/design.md` | — |
| 2 | Add the handoff-grouping verification checklist row to `design-review.md`'s `## Workflow` step-3 checklist with the three severity assignments (missing-grouping=major, non-terminal>3=major, cosmetic=minor) and explicit M≤4 auto-pass branch. | `.claude/agents/design-review.md` | 1 |
| 3 | Update `task/SKILL.md` Step 8 sub-step 5 to cite the design's `## Handoff plan` as source-of-truth for the trigger boundary; update `task/reference.md` § *N=3 of M≥5 handoff gate (rationale)* with the matching paragraph. **Cite `reference.md` § Design Amendment recipe explicitly** (per design-review recommendation) — when the runtime cross-check disagrees with the design's `## Handoff plan`, the agent triggers Design Amendment, not a silent skip. | `.claude/skills/task/SKILL.md`, `.claude/skills/task/reference.md` | 1 |
| 4 | Run the cross-file consistency grep with the **canonical single keyword `Handoff plan`** (per design-review note 2 — drop the `handoff-grouping\|Handoff plan` OR pattern; `Handoff plan` is the section heading and natural read-out anchor, so it should appear verbatim in all 4 files): `grep -rn "Handoff plan" .claude/agents/design.md .claude/agents/design-review.md .claude/skills/task/SKILL.md .claude/skills/task/reference.md` — confirm AC4 (matching terminology across all 4 files; ≥ 1 hit per file). Record post-edit file sizes for AC6. | (no file edits; verification only) | 1, 2, 3 |

**M = 4.** Per the new rule being introduced, no `## Handoff plan` section is required for M ≤ 4 designs. This design intentionally omits one to demonstrate the M ≤ 4 compliance shape (AC5 second clause — "future readers see what compliance looks like for small specs too"). The synthetic example carrying the M ≥ 5 demonstrator lives inside the `design.md` edit itself (task #1), satisfying AC5's first clause.

> **Note on M = 4 vs M ≥ 5 for this spec's own design.** The spec body suggested M could reach 5 (4 file edits + verification + synthetic example). I collapsed the synthetic example into task #1 (it's a single content edit inside `design.md` that ships in the same commit as the grouping-rule addition; splitting it into its own subtask would be artificial). The verification step is real, atomic, and worth its own row. Result: M = 4.

## Handoff plan

Not applicable — this design has M = 4 (≤ 4), so per the new rule no
`## Handoff plan` section is required. This omission is itself the AC5
compliance demonstrator for M ≤ 4 designs: future readers grepping for
`## Handoff plan` in this design will find this paragraph explaining
why it's absent. The single implicit group (subtasks 1–4) does not
trip the N=3-of-M≥5 gate during Step 8 implementation.

If a future revision pushes M ≥ 5, the grouping would look like:

```
## Handoff plan

- Group A: subtasks 1–3. After subtask 3, spawn `/context-reset`.
- Group B: subtasks 4–5. (Terminal group, sized 1..=3.)
```

## Risks

| # | Risk | Mitigation |
|---|---|---|
| R1 | Drift between the three files — terminology used in `design.md` ("handoff plan", "handoff group") could diverge from `design-review.md`'s parser language or `task/SKILL.md`'s read-the-design clause. | Task #4 runs the cross-file grep explicitly; identical key phrase (`## Handoff plan`) is used as both the section heading in `design.md` and the parser anchor cited by `design-review.md` and `task/SKILL.md`. |
| R2 | Design-review parser ambiguity on edge cases — what about a design with `## Handoff plan` section but M ≤ 4 (extraneous)? What about M ≥ 5 with a `## Handoff plan` section whose groups don't sum to M? | Spec is explicit: M ≤ 4 grouping is **optional** (extraneous section is allowed, not flagged); the parser checks **only when M ≥ 5**. Numbering-mismatch (groups don't cover [1..M] contiguously) falls under "non-terminal group size != 3 OR terminal group not in 1..=3" — same `major` severity. The checklist row in task #2 spells this out. |
| R3 | Step 8 runtime counter and design's plan disagree (design says boundary after subtask 3; runtime says we just finished subtask 4 without firing). | The runtime check becomes a **cross-check** against the design (per task #3's wording). If they disagree, the agent surfaces the discrepancy — this is a Design Amendment trigger, not a silent skip. `reference.md`'s existing Design Amendment recipe covers it. |
| R4 | File-size budget — adding the new section + synthetic example to `design.md` (~3 KB → ~5 KB), checklist row to `design-review.md` (~4 KB → ~4.5 KB), Step 8 update to `task/SKILL.md` (~20 KB → ~20.5 KB), paragraph to `reference.md` (~27 KB → ~27.5 KB). | All well under the 35 KB warning and 40 KB hard cap. Task #4 records post-edit sizes for the AC6 audit. |
| R5 | Existing GO designs (already-implemented tasks under `ai-docs/plans/done/`) lack the new section but were valid at the time. | Retroactive — out of scope. The new rule applies to designs produced **after** this PR merges. No back-fill. |
| R6 | Propagation Rule fan-out: edits to all 3 files (`design.md`, `design-review.md`, `task/SKILL.md`) trigger this group's Propagation Rule. `reference.md` is part of the `task/SKILL.md` natural-extension surface, not a separate group. | Task #1, #2, #3 together cover the full sync group; task #4's grep is the propagation-completeness check. |

## Test Design

This is an instruction-file refactor — no Rust code changes, no
`cargo test` additions. Verification is by direct file inspection and
cross-file grep.

| AC | Verification recipe |
|---|---|
| **AC1** (`design.md` carries the rule) | `grep -n "Handoff plan\|handoff-grouping\|M ≥ 5\|/context-reset" .claude/agents/design.md` — expect (a) a Rule entry, (b) a section in artifact-format template, (c) a synthetic example. All four threshold elements present in the wording: when (M ≥ 5), group cap (≤ 3), destination (`/context-reset`), terminal-group sizing (1..=3). |
| **AC2** (`design-review.md` checklist row with severities) | `grep -n "Handoff plan\|major\|minor" .claude/agents/design-review.md` — expect a new bullet in the `## Workflow` step-3 checklist citing the three severities explicitly (missing-grouping=major; non-terminal>3=major; cosmetic=minor), and the M ≤ 4 auto-pass branch. |
| **AC3** (`task/SKILL.md` Step 8 + `reference.md` cite the plan as source of truth) | `grep -n "Handoff plan" .claude/skills/task/SKILL.md .claude/skills/task/reference.md` — expect at least one hit per file. The binding rule's existing prose (`reference.md` § N=3 of M≥5) stays in force; the new paragraph augments, not replaces. |
| **AC4** (Propagation Rule satisfied — consistent terminology) | `grep -rn "Handoff plan" .claude/agents/design.md .claude/agents/design-review.md .claude/skills/task/SKILL.md .claude/skills/task/reference.md` — all 4 files return ≥ 1 hit with identical spelling. Task #4 is this recipe. |
| **AC5** (demonstrator visible to future readers) | Inline synthetic example in `design.md`'s artifact-format section (task #1, mandatory) **plus** this design's own M ≤ 4 compliance demonstrator in its `## Handoff plan` section (above). Spec AC5's first clause is satisfied by the synthetic example; the second clause ("M ≤ 4 case so future readers see what compliance looks like for small specs too") by THIS design's `## Handoff plan` paragraph explaining the absence. |
| **AC6** (no instruction file crosses 40 000-char cap) | `wc -c .claude/agents/design.md .claude/agents/design-review.md .claude/skills/task/SKILL.md .claude/skills/task/reference.md` — every result < 40 000; preferred < 35 000. Pre-edit baseline already recorded above (`3 006 / 3 928 / 20 394 / 26 932`). Task #4 records the post-edit numbers. |

## Open questions

None. The spec's single open question (representation of the
handoff-grouping plan) is resolved above (Option 1 — separate
`## Handoff plan` section). All other Key Decisions in the spec are
pre-resolved and require no design-time choice.
