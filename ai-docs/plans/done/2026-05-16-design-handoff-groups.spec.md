# Design-time handoff grouping for /task Step 8

**Source:** user description (free-text)
**Date:** 2026-05-16
**Tracked in:** #371

## Scope

1. **`.claude/agents/design.md` — grouping requirement.** Extend the artifact format and Rules so that when the Decomposition table has **M ≥ 5** subtasks, the design MUST also carry an explicit per-group plan that partitions the subtasks into **consecutive** groups of **≤ 3 subtasks** each, naming the handoff boundary between groups (`/context-reset` is the canonical handoff destination). When **M ≤ 4**, no grouping is required (single implicit group; the N=3-of-M≥5 gate never fires).
2. **`.claude/agents/design-review.md` — verification checklist item.** Add a checklist row that mechanically verifies:
   - For M ≥ 5 designs: a handoff-grouping plan exists; every non-terminal group has exactly 3 subtasks; the terminal group has 1..=3 subtasks; consecutive numbering matches the Decomposition table; the handoff destination is `/context-reset`.
   - For M ≤ 4 designs: no grouping required (auto-PASS for this checklist row).
   - **Severity assignments:** missing grouping on an M ≥ 5 design → `major` (forces `/task` Step 8 to re-derive groups at runtime, recreating the silent-drift failure mode this spec eliminates). Any non-terminal group sized > 3 subtasks → `major` (would silently skip the binding gate at the right boundary). Cosmetic issues (group label naming, missing per-group rationale, ordering of the handoff-plan section relative to other sections) → `minor`.
3. **`.claude/skills/task/SKILL.md` Step 8 + `reference.md` § *N=3 of M≥5 handoff gate (rationale)* — execution contract.** Step 8 reads the design's handoff-grouping plan as the **source of truth** for when to fire the `/context-reset` handoff. The runtime counter ("just completed subtask N of M; is N == 3 and M ≥ 5?") becomes a deterministic lookup against the design's pre-computed group boundaries rather than an on-the-fly subtask count. The binding rule itself is unchanged; what changes is that the trigger is recorded in the design, not re-derived per turn.
4. **Propagation Rule fan-out.** Edits to all three files in (1)–(3) happen in the same PR per the AGENTS.md *Propagation Rule* Task/Design sync group (`task/SKILL.md` Steps 6–8 ↔ `design.md` ↔ `design-review.md`). The Propagation Rule already covers this group; this spec relies on it.
5. **Demonstrator design.** When the design agent runs for THIS spec (`/task` Step 6), the resulting `2026-05-16-design-handoff-groups.design.md` MUST itself comply with the new grouping rule (i.e., if its Decomposition lists M ≥ 5 subtasks, it carries a handoff-grouping plan). Additionally, `design.md` itself receives a minimal **inline synthetic example** in its artifact-format section so future readers see what a compliant grouping looks like without needing to read this task's design.

## Out of scope

- Changing the gate's thresholds (`N = 3`, `M ≥ 5` stay). The handoff-grouping plan is a representation of the existing gate, not a replacement.
- Adding a 4th group size or making the cap configurable. Groups are strictly `≤ 3` consecutive subtasks (matching the gate's binding contract).
- Hook-level enforcement of the gate. The gate stays rule-enforced per the existing Step 8 contract; this spec only changes how the *trigger boundary* is recorded.
- Reworking `.claude/skills/context-reset/SKILL.md`'s own handoff protocol. `/context-reset` stays the canonical handoff destination; the protocol on the receiving side is untouched.
- Renaming or restructuring the existing Decomposition table columns (`| # | Task | Files | Depends on |`). The handoff-grouping plan is **additive** — added as a new structural element in the design doc, not as a column-shape change. (The exact representation — table column vs. a separate section vs. table header rows — is a Step 6 / Step 7 design decision; see **Open questions**.)

## Deferred

- Promoting the grouping rule into a project-wide AXIOM in AGENTS.md | additional propagation surface; the design / design-review agent files are sufficient surface for now | no separate issue needed.
- Tooling that lints a `*.design.md` for grouping compliance outside the design-review agent | second enforcement layer; review-agent enforcement is sufficient at the gate's current failure rate | no separate issue needed.

## Key decisions

| Question | Decision |
|---|---|
| Does the grouping plan replace the runtime N=3-of-M≥5 gate, or merely make the trigger deterministic? | Make deterministic. The binding rule in `reference.md` § *N=3 of M≥5 handoff gate (rationale)* stays the source of truth for the gate's *existence*; the design's grouping plan provides the *trigger boundary* per the design rather than per-turn re-derivation. |
| When M ≤ 4, is grouping forbidden, optional, or required? | **Optional.** Design agent MAY omit a grouping plan entirely for M ≤ 4 designs; design-review auto-passes the grouping checklist row when M ≤ 4. |
| What handoff destination is named in the plan? | `/context-reset`. The plan does not enumerate alternatives — `/context-reset` is the canonical handoff per the existing `reference.md` rationale and `.claude/skills/context-reset/SKILL.md`. |
| Severity for missing grouping on M ≥ 5? | `major`. Recreates the silent-drift failure mode this spec eliminates. |
| Severity for non-terminal group sized > 3? | `major`. Would silently skip the binding gate at the right boundary. |
| Severity for cosmetic grouping issues (labelling, ordering, prose)? | `minor`. |
| Does the synthetic example in `design.md` belong inline in the artifact-format section, or as a sibling reference file? | Inline. One synthetic example beside the artifact-format template; future readers see the shape without an extra hop. |
| Does the demonstrator design (this spec's own design.md) need an explicit grouping plan? | Only if its Decomposition has M ≥ 5. Same rule as every other design; no special-case. |

## Technical constraints

- **No threshold changes.** N = 3, M ≥ 5 are fixed by the existing gate. This spec does not propose alternatives.
- **Strict propagation.** Per the AGENTS.md Task/Design sync group, edits to any of the three files (`task/SKILL.md`, `design.md`, `design-review.md`) trigger edits to the other two in the same PR. All three are in scope.
- **No new dependencies.** Markdown-only edits to instruction files.
- **No code changes outside `.claude/` and (possibly) `ai-docs/`.** This is an instruction-file refactor.
- **`actionlint` does not apply.** No `.github/workflows/*.yml` files touched.
- **Build / test / clippy / doc gates** still run per `/task` Step 9 even on an instruction-only change, but should be no-ops (no Rust sources changed).
- **Instruction-file size budget** (AGENTS.md *Build & Test* axiom): every file edited must remain `< 40 000` chars, with `< 35 000` preferred. Pre-edit sizes: `.claude/agents/design.md` ~3.4 KB, `.claude/agents/design-review.md` ~5.9 KB, `.claude/skills/task/SKILL.md` ~28 KB, `.claude/skills/task/reference.md` ~28 KB. Headroom is comfortable; design should still budget the edits to favour reference-extraction over inline expansion where the addition is long-form prose.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/agents/design.md` carries the handoff-grouping requirement for M ≥ 5 decompositions — both as a Rule entry and as a structural element in the artifact-format section. The wording is unambiguous about (a) when grouping is required (M ≥ 5), (b) the maximum group size (3 consecutive subtasks), (c) the handoff destination (`/context-reset`), and (d) the requirement that the final group may be sized 1..=3. |
| AC2 | `.claude/agents/design-review.md` carries the verification checklist row with explicit severity assignments (major for missing grouping on M ≥ 5; major for non-terminal group sized > 3; minor for cosmetic issues). The row is integrated into the existing checklist, not appended as a postscript. |
| AC3 | `.claude/skills/task/SKILL.md` Step 8 (and `reference.md` § *N=3 of M≥5 handoff gate (rationale)* where appropriate) cites the design's handoff-grouping plan as the source of truth for handoff timing. The binding rule in `reference.md` stays in force; the change is that the trigger is read from the design rather than re-derived per turn. |
| AC4 | AGENTS.md Propagation Rule (Task/Design sync group: `task/SKILL.md` Steps 6–8 ↔ `design.md` ↔ `design-review.md`) is satisfied — all three files are edited in the same PR with consistent terminology. A grep for the new keyword (e.g., `handoff-grouping`) returns hits in all three files. |
| AC5 | A demonstrator grouped design is visible to future readers — either via (a) an inline synthetic example in `.claude/agents/design.md`'s artifact-format section, (b) the design produced for THIS spec being itself grouping-compliant (when its M ≥ 5), or **both**. The synthetic example in `design.md` is non-optional; the THIS-spec demonstrator is conditional on M ≥ 5. |
| AC6 | No instruction file edited by this task crosses the 40 000-char hard cap (AGENTS.md *Build & Test* axiom). Sizes recorded post-edit. |

## Open questions

- **Representation of the handoff-grouping plan in `design.md`'s artifact.** Three candidate shapes:
  1. New `## Handoff plan` h2 section directly under `## Decomposition`, prose-form: `Group A: subtasks 1–3. After subtask 3: spawn /context-reset. Group B: subtasks 4–6. ...`.
  2. New `Group` column on the existing Decomposition table: `| # | Group | Task | Files | Depends on |` with values like `A`, `A`, `A`, `B`, `B`, `B`, `C`.
  3. Header-row-style separators inside the Decomposition table marking group boundaries.

  The user did not pin a representation. Design (Step 6) will choose one and the design-review parser will follow. Pre-flagged here so the design agent treats the choice as load-bearing, not bikeshedding. Defensible default (if design agent must pick without further input): **option 1** — least invasive to existing Decomposition consumers, most explicit handoff-destination naming.
