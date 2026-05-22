# /next — UI-designer handoff queue section

**Source:** user description (free-text /task entry, 2026-05-23)
**Date:** 2026-05-23
**Tracked in:** #543

## Scope

1. Add a new informational section to `.claude/skills/next/SKILL.md` titled **"Candidates for UI-designer handoff"** that lists open GitHub issues carrying the `ui-design` label.
2. Update the `Selection rules` block in **both** Default mode and Small mode so issues carrying `ui-design` are filtered out of Recommendation / Runner-ups (same exclusion semantics as the existing `blocked` label).
3. Add a "UI-designer label" mini-section (parallel to the existing "Blocked-issues label" mini-section) that documents what the label means, who applies it, and the lifecycle (apply when an issue needs out-of-harness designer work; remove when the design-system assets land and the issue is unblocked).
4. Update the "Output (both modes)" block so the new section is listed after the existing "Candidates needing `/triage`" section, with the same "informational only — never top-line Recommendation or runner-up" caveat.
5. The new section's per-row format mirrors the existing informational style: issue number + link + title + a brief one-liner naming why the human designer is needed (cited verbatim or paraphrased from the issue title / first body line).

## Out of scope

- No Rust code changes — this is an instruction-file-only edit to `.claude/skills/next/SKILL.md`.
- No new label or label-management automation — the `ui-design` label already exists (created 2026-05-23, color `#E91E63`) and is applied manually by the orchestrator, just like `blocked`.
- No changes to `.claude/skills/triage/SKILL.md` or `.claude/agents/triage-runner.md` unless the design phase identifies a concrete sister-file edit (propagation rule: `next/SKILL.md` is in the Triage group; the rule requires a *check*, not a mechanical edit, when the changed keyword isn't shared).
- No changes to the 4 already-labelled umbrella issues (#539, #540, #541, #542) or their 40 blocked children — they remain as-is; the new section simply surfaces them.
- No changes to the existing `blocked`-label filter — this spec adds a parallel filter, not a replacement.
- No changes to `gh issue list` invocation in the frontmatter — the existing `--json number,title,labels,updatedAt` already exposes the labels array, so `ui-design` is mechanically detectable without a fetch change.

## Deferred

- Rolling the `ui-design` filter into a shared "non-actionable-by-Claude" filter alongside `blocked` | refactor into a single helper section once a third such label appears | no separate issue needed yet — re-evaluate when the next out-of-harness label lands.

## Key decisions

| Question | Decision |
|---|---|
| Where does the new section sit in the output order? | After "Candidates needing `/triage` (informational)" — both are informational and parallel; the UI-designer section is the later addition so it appends. |
| Section title? | **"Candidates for UI-designer handoff (informational)"** — mirrors the **"Candidates needing `/triage` (informational)"** wording. |
| Filter signal? | `labels` array contains `ui-design` (string match, same shape as the existing `blocked` filter). |
| Are `ui-design` issues excluded from Recommendation / Runner-ups? | Yes — same exclusion semantics as `blocked`. The work cannot proceed in this harness until the human designer hands back assets, so Claude shouldn't recommend it as a top-line item. |
| Does the section appear in both Default mode and Small mode? | Yes — consistent with "Candidates needing `/triage`", which also appears in both. |
| Per-row format? | `#N — <title> (<link>): <one-line rationale>`. Rationale defaults to "needs design-system visual spec / designer pass" unless the issue body's first line gives a more specific cue. |
| Reminder text at the end of the section? | One sentence: items in this section need an out-of-harness designer pass (Figma / `design-system/` folder); they unblock once the designer's PR lands and the `ui-design` label is removed. |
| What if no issues carry the label? | Section header still renders, with body "No issues currently labelled `ui-design`." (parallel to how `/triage` section would behave when empty — keeps the output schema stable). |

## Technical constraints

- File touched: `.claude/skills/next/SKILL.md` only (under workspace 40k-char instruction-file cap; current size well below threshold).
- The `.claude/skills/next/SKILL.md` frontmatter already declares `disable-model-invocation: true` and uses `gh issue list ... --json number,title,labels,updatedAt`; no frontmatter change needed.
- Propagation-rule sister files (Triage group: `.claude/skills/triage/SKILL.md`, `.claude/agents/triage-runner.md`): the design phase MUST grep both for any `ui-design` / `blocked`-label reference and decide whether a parallel edit is needed. Expected outcome: no edit required because the `ui-design` label is a `/next`-only concept (triage doesn't gate on it), but the grep must be performed and the outcome recorded.
- The new section's contract — informational, never recommended, never runner-up — must be expressed in the same language as the existing `/triage` section to avoid contract drift.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/skills/next/SKILL.md` contains a new section titled **"Candidates for UI-designer handoff (informational)"** in the "Output (both modes)" block, after the existing "Candidates needing `/triage` (informational)" item. |
| AC2 | The Default-mode `Selection rules` and Small-mode `Selection rules` each include a bullet "Skip GitHub issues carrying the `ui-design` label" parallel to the existing `blocked` skip bullet. |
| AC3 | A new mini-section "UI-designer label" appears parallel to the existing "Blocked-issues label" mini-section, documenting: label color (`#E91E63`), description ("Design-system designer pass / visual spec work required"), how it's applied (`gh issue edit <N> --add-label ui-design`), and when it's removed (when the design-system assets land and the issue can proceed). |
| AC4 | Running `/next` against the current repo state (4 issues labelled `ui-design`: #539, #540, #541, #542) lists all four in the new section and excludes them from the Recommendation and Runner-up slots. |
| AC5 | Running `/next` in **Small mode** against the current repo state also lists the same four issues in the new section and excludes them from Recommendation / Runner-ups. |
| AC6 | The contract sentence "Items in this section are never the top-line recommendation or a runner-up" appears in the new section's body, mirroring the wording of the existing `/triage` informational section. |
| AC7 | `.claude/skills/triage/SKILL.md` and `.claude/agents/triage-runner.md` were grep'd for `ui-design` references; design phase records the outcome (expected: no edit needed; if any reference found, propagate accordingly). |
| AC8 | When zero issues carry the `ui-design` label, the section still renders with a "No issues currently labelled `ui-design`." line — schema stability for `/next` consumers. |

## Open questions

- None blocking design. The section's contract is fully specified by analogy to the existing `/triage` informational section + `blocked` label filter.
