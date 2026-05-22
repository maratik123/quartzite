# Design: /next — UI-designer handoff queue section

**Issue:** #543
**Spec:** `ai-docs/plans/2026-05-23-next-ui-designer-queue.spec.md`
**Date:** 2026-05-23

## Approach

Pure instruction-file edit to `.claude/skills/next/SKILL.md`. Four mechanical insertions are required, every one of them parallel to an existing analogue in the same file:

1. A new Selection-rules skip bullet in **Default mode** parallel to the existing `blocked`-label skip bullet (line 71).
2. A new Selection-rules skip bullet in **Small mode** parallel to the existing `blocked`-label skip bullet (line 81).
3. A new mini-section **"UI-designer label"** parallel to the existing **"Blocked-issues label"** mini-section (lines 85–91), inserted immediately after it.
4. A new informational section **"Candidates for UI-designer handoff (informational)"** appended at end of file as a new bullet in the **Output (both modes)** block, after the existing "Candidates needing `/triage` (informational)" bullet (line 108).

**Why this design:**

- The spec already enumerates the four edit sites; no architectural choice remains. The design's job is to pin the exact insertion anchors, the exact text of each insert, and the propagation outcome.
- Wording reuse: every insert paraphrases an immediately adjacent analogue, so the change preserves the file's voice and minimises drift risk.
- AC8 schema-stability rule (section always renders; "No issues currently labelled `ui-design`." when empty) is encoded as a single conditional sentence inside the new informational section, mirroring how the analogue `/triage` section behaves implicitly.

**Rejected alternatives:**

- *Factor the `blocked` + `ui-design` skip-bullets into a shared "non-actionable-by-Claude" helper.* Rejected per spec § Deferred — re-evaluate when a third such label appears. YAGNI.
- *Change the `gh issue list --json …` frontmatter to add a label filter at fetch time.* Rejected per spec § Technical constraints + Out-of-scope item 6 — the labels array is already exposed, no fetch change needed. Filtering happens client-side in the skill body, same shape as the existing `blocked` filter.
- *Introduce a new label-management automation.* Rejected per spec § Out-of-scope item 2 — `ui-design` label was created manually and is applied manually.

## Propagation grep outcome (Triage group)

Per AGENTS.md § Propagation Rule (Triage group row), edits to `.claude/skills/next/SKILL.md` MUST be checked against `.claude/skills/triage/SKILL.md` and `.claude/agents/triage-runner.md`.

Grep performed:

```
grep -rn "ui-design" .claude/skills/triage/ .claude/agents/triage-runner.md .claude/skills/next/SKILL.md
```

Result: **zero matches** in all three files. The `ui-design` label is a `/next`-only concept; `/triage` does not gate on labels (it gates on `_inbox.md` rows). **No sister-file edit required.** AC7 is satisfied by recording this outcome in the design doc.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `ui-design`-label skip bullet to Default-mode Selection rules (after line 71) AND Small-mode Selection rules (after line 81). Add the new "UI-designer label" mini-section immediately after the "Blocked-issues label" mini-section (after line 91). Add the new "Candidates for UI-designer handoff (informational)" bullet to the "Output (both modes)" block (after line 108). Verify `wc -c` stays well under the 35k early-warning cap. | `.claude/skills/next/SKILL.md` | — |

(Single atomic edit — all four inserts target the same file and form one logically complete unit. Splitting into per-insert subtasks would add zero value and four context flushes.)

## Exact edit recipe

### Edit 1 — Default-mode Selection rules (after current line 71)

**Anchor (existing line 71):**

```
- Skip GitHub issues carrying the `blocked` label (see *Blocked-issues label* below) — body text like "Blocked by: #N" is not visible here, so the label is the canonical signal.
```

**Insert immediately AFTER that line:**

```
- Skip GitHub issues carrying the `ui-design` label (see *UI-designer label* below) — work cannot proceed in this harness until the human designer hands back assets.
```

### Edit 2 — Small-mode Selection rules (after current line 81)

**Anchor (existing line 81):**

```
- Skip GitHub issues carrying the `blocked` label (see *Blocked-issues label* below).
```

**Insert immediately AFTER that line:**

```
- Skip GitHub issues carrying the `ui-design` label (see *UI-designer label* below).
```

### Edit 3 — "UI-designer label" mini-section (after current line 91)

**Anchor (existing lines 85–91, the entire "Blocked-issues label" mini-section ends at line 91):**

```
### Blocked-issues label

This skill fetches issues via `gh issue list --json number,title,labels,updatedAt` — labels are visible, **issue bodies are not.** A "Blocked by: #N" line in an issue body therefore has no effect on `/next`. The convention is:

- After opening or triaging a new issue that depends on another open issue, run `gh issue edit <N> --add-label blocked` (creating the label first via `gh label create blocked` if the repo doesn't have it yet).
- When the blocking dependency is resolved, run `gh issue edit <N> --remove-label blocked`.
- `/next` filters out any issue whose `labels` array contains `blocked` in both default and small modes.
```

**Insert immediately AFTER line 91 (blank line then new mini-section):**

```

### UI-designer label

Issues that need an out-of-harness designer pass (Figma asset, visual spec, `design-system/` work) carry the `ui-design` label (color `#E91E63`, description "Design-system designer pass / visual spec work required"). Like `blocked`, the label is the canonical signal because issue bodies are not visible to this skill. The convention is:

- When an issue is identified as needing a human designer pass, run `gh issue edit <N> --add-label ui-design` (the label already exists in this repo; created 2026-05-23).
- When the design-system assets land and the issue can proceed in-harness, run `gh issue edit <N> --remove-label ui-design`.
- `/next` filters out any issue whose `labels` array contains `ui-design` from Recommendation and Runner-ups in both default and small modes, and surfaces them in the *Candidates for UI-designer handoff (informational)* section instead.
```

### Edit 4 — Output (both modes) new informational bullet (after current line 108)

**Anchor (existing line 108 — last bullet of file):**

```
- **Candidates needing `/triage` (informational):** any untracked rows from the deferred files. Title each row with the row's `Item`-cell text and cite the source file. **Items in this section are never the top-line recommendation or a runner-up** — they are listed for situational awareness only. End the section with a one-sentence reminder that `/triage` ships in Issue B (#204) and until then the user can act on a candidate manually via `/interview`.
```

**Insert immediately AFTER line 108 (new bullet on next line):**

```
- **Candidates for UI-designer handoff (informational):** any open GitHub issue whose `labels` array contains `ui-design`. Format each row as `#N — <title> (<link>): <one-line rationale>` — default rationale "needs design-system visual spec / designer pass" unless the issue body's first line gives a more specific cue. **Items in this section are never the top-line recommendation or a runner-up** — they are listed for situational awareness only. End the section with a one-sentence reminder that items here need an out-of-harness designer pass (Figma / `design-system/` folder) and unblock once the designer's PR lands and the `ui-design` label is removed. **Section always renders** — when zero issues carry the label, the body is the single line `No issues currently labelled \`ui-design\`.` (schema stability for `/next` consumers).
```

## Handoff plan

- **Group A:** subtask 1 — terminal group (1 subtask; within the 1..=3 range). No handoff between groups; the single group completes Step 8 in its own `/context-reset` subagent per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).

(Per `.claude/skills/task/SKILL.md` Step 8 + design.md rule (a): the `## Handoff plan` section is mandatory for every design including single-subtask designs; the single group is also terminal and runs in its own `/context-reset` subagent.)

## Risks

- **Drift risk between the new section and the existing `/triage` informational section if their contracts diverge later.** Mitigation: the design doc anchors wording on the `/triage` section ("Items in this section are never the top-line recommendation or a runner-up") so a future edit to one will naturally prompt a check of the other; both sit in the same "Output (both modes)" block, so a reviewer touching one sees the other.
- **AC8 schema-stability mis-interpretation.** Risk: implementer writes "renders only when non-empty". Mitigation: the recipe text spells out **Section always renders** verbatim and gives the empty-state body string verbatim.
- **Propagation grep result going stale.** Risk: a future PR adds a `ui-design` reference to `.claude/skills/triage/SKILL.md` / `.claude/agents/triage-runner.md`, but this design's "no edit needed" claim was only true at design time. Mitigation: the grep is re-run during implementation (AC7 wording is "design phase records the outcome") — if the implementer's re-grep returns hits, they propagate per AGENTS.md Triage-group row and amend the design via the Spec/Design Amendment recipe.
- **`wc -c` instruction-file cap.** Risk: four inserts push `.claude/skills/next/SKILL.md` over the 35k early-warning cap. Mitigation: current file is 108 lines / well under 10 KB; even with the four inserts (≈ 1 KB of added text) it remains far below the cap. Verify post-edit via `wc -c .claude/skills/next/SKILL.md` as part of the implementation subtask.
- **No test coverage.** Risk: instruction-file regressions are caught only by usage. Mitigation: AC4 / AC5 are validation gates — the `/task` workflow's Step 9 verification step is expected to literally run `/next` and `/next small` against the live repo and confirm the four labelled issues (#539, #540, #541, #542) appear in the new section.

## Test Design

No Rust tests apply — zero `.rs` files touched.

**Manual validation (covered by spec ACs):**

- **AC4 / AC5 — live `/next` invocation.** Run `/next` and `/next small` against the current repo state; confirm:
  - The four issues labelled `ui-design` (#539, #540, #541, #542) all appear in the new "Candidates for UI-designer handoff (informational)" section.
  - None of those four appears as Recommendation or in Runner-ups.
  - The reminder sentence about out-of-harness designer pass is present.
- **AC8 — empty-state rendering.** Cannot directly test in-repo (the repo currently has 4 labelled issues, not zero), but can be verified by visual inspection of Edit 4's recipe text: the empty-state body string `No issues currently labelled \`ui-design\`.` is present verbatim and the "Section always renders" rule is explicit.
- **AC7 — propagation grep re-run.** Implementer re-runs `grep -rn "ui-design" .claude/skills/triage/ .claude/agents/triage-runner.md` after the four inserts land; expected outcome is still zero matches (no sister-file edit needed).

## Open questions

- None. Spec § Open questions explicitly says "None blocking design"; all four edit sites and their exact wording are derivable from the spec + the file's existing structure.
