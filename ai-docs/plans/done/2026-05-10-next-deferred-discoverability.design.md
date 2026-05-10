# Design: `/next` deferred-file discoverability + `_inbox.md` AGENTS.md governance

**Issue:** [#202](https://github.com/maratik123/quartzite/issues/202) (umbrella A1 of the four-issue process-improvements plan, [`ai-docs/plans/2026-05-10-process-improvements.md`](2026-05-10-process-improvements.md))
**Date:** 2026-05-10
**Spec:** [`2026-05-10-next-deferred-discoverability.spec.md`](2026-05-10-next-deferred-discoverability.spec.md)

## Approach

Prompt-only edits. Two files change: `.claude/skills/next/SKILL.md` (extend the prompt) and `AGENTS.md` (add an AXIOM in *Workflow*, add an *Agent Docs* row). No Rust code, no new files — `_inbox.md` is created by Issue A2 (#203); A1 only lands governance for it.

The meta-plan and the spec already pinned every architectural decision. This design doc nails down four file-level edits and the manual-verification recipe for each AC.

**Why prompt-only.** `/next` is a Claude Code skill (not a CLI tool); it has `disable-model-invocation: true` and is invoked manually. Its "logic" is the natural-language instructions an Opus reader follows on each invocation. There is no parser to write — the model reads the eight thematic files plus `widget-backlog.md` directly inside the prompt body, then applies the selection rules the prompt spells out. This matches the meta-plan's locked-in decision *"pure skill-prompt logic driven by an opus subagent. No Rust binary, no shell script."*

**Rejected alternatives.**

- **Rust pre-parser of `ai-docs/deferred/*.md` invoked by `/next`.** Rejected: the meta-plan locked "pure skill-prompt logic" and forbids new binaries / scripts in this track. Adding a parser also pulls in test infrastructure that A2 already plans; duplicating it here violates YAGNI.
- **Embed the deferred-file contents inline via fenced `!`-blocks (the same shape `/next` uses for `gh issue list` and `cat ai-docs/plans/INDEX.md`).** Considered. Decided **yes** — this is the consistent pattern with how `/next` already feeds context to the model and avoids any "go read these files" instruction the model could skip. The 9 files together are well under any context limit (the largest, `widget-backlog.md`, is ~130 lines).
- **Add a new `Tracked` column to `widget-backlog.md` so its schema matches the eight thematic files.** Rejected by spec *Out of scope*: schema migration is not part of A1. A1 only **reads** `widget-backlog.md`. Tracked refs in widget-backlog go in the existing `Notes` cell; the prompt instructs the reader to look for `tracked: #N` substring there.
- **Surface `🤔 undecided` and `📭 future` rows in *Candidates needing `/triage`* alongside `🟡 v2`.** Rejected by spec *Open questions*: only `🟡 v2` is "actionable backlog"; the other emojis (`✅` / `🤔` / `❌` / `📭`) are not.
- **Add the AXIOM in a new top-level section.** Rejected: the spec locks it inside *Workflow*, alongside AXIOM 1 and AXIOM 2, matching the existing pattern for binary process rules. (Note: the unnumbered "Pre-publish" axiom lives in *API Stability*, not *Workflow* — only AXIOM 1 / AXIOM 2 are the relevant Workflow neighbours.)

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Extend `/next` skill prompt: add 9 fenced `!`-blocks (one per deferred file) so the model sees their contents on every invocation | `.claude/skills/next/SKILL.md` | — |
| 2 | Add new selection-rule subsection to `/next` prompt explaining tracked vs. untracked classification, the *Candidates needing `/triage`* output section, and the `widget-backlog.md` parser-anchor caveat | `.claude/skills/next/SKILL.md` | 1 |
| 3 | Update *Output (both modes)* in `/next` prompt to enumerate the new *Candidates needing `/triage`* section as a third output bullet | `.claude/skills/next/SKILL.md` | 2 |
| 4 | Add the `_inbox.md` AXIOM (verbatim spec prose) to AGENTS.md *Workflow* section, placed after AXIOM 2 and before *Propagation Rule* (or in any spot inside *Workflow* that maintains chronological order with AXIOM 1 / 2) | `AGENTS.md` | — |
| 5 | Add the `_inbox.md` row to AGENTS.md *Agent Docs* table (verbatim spec prose, after `ai-docs/plans/deferred/` row) | `AGENTS.md` | — |
| 6 | Run AC verification recipes (manual): invoke `/next` on current data, confirm AC1 + AC2 outputs; run grep recipes for AC3 + AC4 | none (verification only) | 1, 2, 3, 4, 5 |

Six tasks, well under the seven-task limit. Tasks 1–3 are sub-edits of the same file but are conceptually independent (context-feed vs. selection-rule vs. output-shape); keeping them separate makes the implementation PR easier to review.

## Risks

- **Risk: model reading `widget-backlog.md` line 89 (`> spec. Tracked: TBD …`) misclassifies it as a tracked row.** Mitigation: spec *Technical constraints* mandates the prompt instruction *"anchor on column-header context, not bare substrings"*. Task 2 must include a literal example of the prose hit and the explicit instruction to ignore it. Verifiable by AC1 — if the prose hit produced a spurious "Candidates needing `/triage`" entry, the manual-run output would show "TBD" as a candidate, which AC1's spot-check inspection catches.
- **Risk: model double-recommends `#48` once as a `gh issue list` hit and again as a `signals-slots.md` row whose `Tracked` is `#48`.** Mitigation: Task 2's selection rule explicitly says "if `#N` is already in the candidate set, the deferred-file row appears (if at all) as a one-line supplement under that issue's recommendation, never as a second top-line entry". AC2 verifies on the live `#48` row.
- **Risk: forward references in the AXIOM and *Agent Docs* row (`/task` Step 12, `/triage`, `_inbox.md`) confuse readers because those targets do not exist yet.** Mitigation: the spec *explicitly* allows the forward references and frames them as "deliberate forward notes, not stale pointers". The *Agent Docs* row carries the literal suffix "introduced in Issue A2" so a future reader sees the temporal context. The AXIOM body's table rows likewise reference behaviour that A2 / B will implement; the meta-plan's strict A1 → A2 → B → C sequence ensures the references resolve quickly.
- **Risk: Propagation Rule miss.** Mitigation: spec sync-group footprint says "no new sync-group entry created in A1". Verify with `grep -rn 'ai-docs/deferred/_inbox.md' .claude/ AGENTS.md` after the edits — must be **exactly 2** (the AXIOM first-line reference + the *Agent Docs* row in `AGENTS.md`). AC4 codifies this count. The narrower path-suffix grep is implementation-choice independent: it does **not** count `next/SKILL.md`'s new `cat ai-docs/deferred/<file>.md` reads, which are audited separately via the `wc -l` line-count delta on `next/SKILL.md` (≥ +9 lines for the nine context-feed `!`-blocks).
- **Risk: AGENTS.md line-count growth pushing some unrelated CI gate.** Mitigation: there is no AGENTS.md size gate in CI today (verified via `.github/workflows/`). Edit adds ~14 lines (axiom + table) plus 1 line (Agent-Docs row); negligible.
- **Risk: `learnings.md` accidentally edited in the same PR.** Mitigation: AGENTS.md *Boundary rule 2* forbids it. The spec acknowledges this; the implementation agent must not append a learning entry in the same turn as the AGENTS.md edits.

## Test Design

A1 is prompt-only — there is no Rust code path to unit-test. Verification is **manual**, performed once after the edits land on the feature branch and again as a smoke check before merge. Each AC has a recipe.

### AC1 — `/next` surfaces ≥ 1 untracked row in *Candidates needing `/triage`*

- **Verification command:** invoke `/next` (default mode) on the A1 branch.
- **Expected output:** a section titled `## Candidates needing /triage` (or a clearly-titled subsection — exact heading shape is implementation-author's choice but must match the spec's literal "Candidates needing `/triage`") containing **at least one** of:
  - `Auto-merge` (from `ci-docs-workflow.md`, `Tracked: —`)
  - `removing rstest` (from `ci-docs-workflow.md`, `Tracked: —`)
  - `Full wildcard re-exports` (from `ci-docs-workflow.md`, `Tracked: —`)
  - `Future features (extension, 8k_pages, etc.)` (from `ci-docs-workflow.md`, `Tracked: —`)
  - `Separate EXAMPLES.md` (from `ci-docs-workflow.md`, `Tracked: —`)
- **Pass criterion:** at least one of the above row titles appears verbatim (or with minor word-order variation) in the section.
- **Fail criterion:** the section is missing entirely; or the section exists but lists none of the above (the prompt failed to read `ci-docs-workflow.md`); or the section spuriously contains the line-89 prose hit (`> spec. Tracked: TBD …`) treated as a row.

### AC2 — `/next` does not double-rank `Tracked: #48`

- **Verification command:** invoke `/next` (default mode) on the A1 branch. Inspect the output for `#48`.
- **Expected output:** `#48` appears **at most once as a primary recommendation or runner-up**. `signals-slots.md` mentions `#48` rows three times (lines 9, 11, 20 — under both *Deferred* and *Out of scope*); they may appear as supplements **under** the `#48` recommendation/runner-up, but never as a separate top-line entry.
- **Pass criterion:** zero double-recommendations of `#48`. (The model may legitimately decide `#48` is not the best pick at all — that also counts as pass; the rule prevents *double* listing, not the listing itself.)
- **Fail criterion:** `#48` appears as both (a) the recommendation/runner-up, and (b) a separately-titled top-line entry sourced from `signals-slots.md`.

### AC3 — AGENTS.md governance present

- **Verification commands:**
  ```bash
  grep -A 12 'AXIOM.*_inbox.md' AGENTS.md
  grep '_inbox.md' AGENTS.md | wc -l
  ```
- **Expected outputs:**
  - First grep: 13+ matched lines starting with the AXIOM header and including all four action-table rows from the spec.
  - Second grep: ≥ 2 (one in the AXIOM block, one in the *Agent Docs* row; possibly more if the AXIOM body cites the path in multiple cells).
- **Pass criterion:** both checks satisfied.

### AC4 — sync-group propagation: `_inbox.md` referenced exactly where introduced

The verification splits into two independent checks: (a) the **governance footprint** — that `ai-docs/deferred/_inbox.md` appears in the two and only two places this issue introduces (AXIOM first-line reference + *Agent Docs* row); and (b) the **context-feed footprint** — that `next/SKILL.md` actually grew by the nine new `!`-blocks. The two checks are deliberately decoupled so AC4 stays implementation-choice independent of how Edit 1's nine `cat`s are spelled.

- **Verification commands:**
  ```bash
  # (a) Governance footprint — must be exactly 2 (AXIOM first line + Agent Docs row)
  grep -rn 'ai-docs/deferred/_inbox.md' .claude/ AGENTS.md | wc -l

  # (b) Context-feed footprint — next/SKILL.md grew by ≥ 9 lines vs. baseline
  git diff --numstat master -- .claude/skills/next/SKILL.md
  ```
- **Expected outputs:**
  - (a) The `grep` count is **exactly 2**: one hit on the AXIOM's first line in `AGENTS.md`, one hit on the *Agent Docs* row in `AGENTS.md`. Zero hits inside `.claude/` because A1 does not create a `/triage` skill or any other consumer file referencing `_inbox.md` by path.
  - (b) The `git diff --numstat` "added lines" column for `next/SKILL.md` is **≥ +9** — covering the nine new `!`-blocks (one `cat ai-docs/deferred/<file>.md` line each, ignoring blanks and fences which only inflate the count). In practice the delta is much larger (closer to +30) because each `!`-block has 3 lines (opener, command, closer) plus surrounding heading and selection-rule text from Edits 2–3; +9 is the floor that proves the context-feed landed at all.
- **Pass criteria:** check (a) returns `2`, **and** check (b) returns added-lines ≥ 9 for `next/SKILL.md`.
- **Fail criteria:**
  - Check (a) returns `< 2`: the AXIOM or *Agent Docs* row was dropped or path-suffix doesn't match.
  - Check (a) returns `> 2`: `_inbox.md` was cited beyond the two governance spots — likely an unintended reference that needs justification or removal. (The AXIOM body's table rows mention `_inbox.md` *the filename* in prose multiple times, but only the AXIOM **first line** carries the full path `ai-docs/deferred/_inbox.md`; bare-`_inbox.md` mentions are not counted by this grep, by design.)
  - Check (b) added-lines < 9: Edit 1 was not fully applied — at least one of the nine deferred files is missing from the context-feed.

### Manual-verification scratchpad

The implementation agent should capture the `/next` output for AC1 and AC2 as a short blockquote in the PR body so reviewers can spot-check without having to re-run the skill. Format:

```
### AC1 verification (captured 2026-05-10 …)
<paste of the *Candidates needing /triage* section from /next output>

### AC2 verification (captured 2026-05-10 …)
<paste of the relevant lines mentioning #48>
```

### Fixtures / helpers needed

None — the deferred files on the A1 branch ARE the fixtures. The spec's *Technical constraints* mandate verification on current-data state.

## Edit specifications

This section nails down the exact placement and shape of each edit so the implementation agent has no ambiguity.

### Edit 1: `.claude/skills/next/SKILL.md` — context-feed extension (Task 1)

**Insert after line 18** (after the `## Plan index` block that `cat`s `INDEX.md`) and before `## Task` (line 20):

```
## Deferred-file backlog (8 thematic files + widget-backlog)

```!
cat ai-docs/deferred/ci-docs-workflow.md
```

```!
cat ai-docs/deferred/future-crates.md
```

```!
cat ai-docs/deferred/macros-codegen.md
```

```!
cat ai-docs/deferred/object-tree.md
```

```!
cat ai-docs/deferred/properties.md
```

```!
cat ai-docs/deferred/python.md
```

```!
cat ai-docs/deferred/signals-slots.md
```

```!
cat ai-docs/deferred/threading-runtime.md
```

```!
cat ai-docs/deferred/widget-backlog.md
```
```

(Nine separate `!`-blocks, one per file — kept separate so a missing file fails loudly rather than silently truncating a globbed read.)

### Edit 2: `.claude/skills/next/SKILL.md` — selection-rule subsection (Task 2)

**Insert a new subsection inside `## Task`, after the existing `### Blocked-issues label` subsection** (after current line 53), before `### Output (both modes)` (current line 56). Title: `### Deferred-file rows (8 thematic + widget-backlog)`.

The subsection text follows Pattern 5 from `ai-docs/agent-writing-style.md` (numbered enumeration of triggers) and must include, at minimum:

1. Definition of "tracked" vs. "untracked" rows for each schema:
   - **8 thematic files** (`signals-slots.md`, `properties.md`, `macros-codegen.md`, `object-tree.md`, `threading-runtime.md`, `future-crates.md`, `ci-docs-workflow.md`, `python.md`) — column 4 (`Tracked`): `#N` ⇒ tracked, `—` ⇒ untracked.
   - **`widget-backlog.md`** — `Status` column emoji `🟡 v2` ⇒ untracked candidate; `Notes` cell containing literal `tracked: #N` ⇒ tracked. Other emojis (`✅` / `🤔` / `❌` / `📭`) ⇒ skip (not in the candidate set at all).
2. The double-recommendation guard: "If `#N` is already in the `gh issue list` candidate set, the deferred-file row is **not** re-listed as a separate item — at most one supplementary one-liner under that issue's recommendation cites the deferred row."
3. The `widget-backlog.md` line-89 anchor caveat: "The string `Tracked` appears once as prose in `widget-backlog.md` (`> spec. Tracked: TBD …`). **Do not** treat this as a row. Anchor classification on column-header context — only rows inside an actual table count."
4. The new output section: "Untracked rows surface in a section titled **Candidates needing `/triage`** in the output. They are **never** the top-line recommendation — only listed for situational awareness, with a one-sentence suggestion per row to run `/triage` first (note: `/triage` ships in Issue B / #204; until then, the section is informational and the user can act on a candidate manually via `/interview`)."

### Edit 3: `.claude/skills/next/SKILL.md` — output shape (Task 3)

**Modify the `### Output (both modes)` section** (current lines 56–59) to add a third bullet:

> - **Candidates needing `/triage` (informational):** any untracked rows from the deferred files. Title each row with the row's `Item`-cell text and cite the source file. **Items in this section are never the top-line recommendation or a runner-up** — they are listed for situational awareness only. End the section with a one-sentence reminder that `/triage` ships in Issue B (#204) and until then the user can act on a candidate manually via `/interview`.

### Edit 4: `AGENTS.md` — `_inbox.md` AXIOM (Task 4)

**Insert after line 180** (the closing line of AXIOM 2's body) and before line 182 (`## Propagation Rule`). Place the verbatim AXIOM body from spec *Scope* item 2 — including the four-row action table — preceded by one blank line.

The verbatim prose (locked by the spec, do **not** rephrase):

```markdown
> **AXIOM — `ai-docs/deferred/_inbox.md` is written ONLY by `/task` Step 12 and `/triage`.**
> Hand-edits to `_inbox.md` defeat the propagation contract that Issue A2 sets up — they hide rows from the parser and conflict with future Step-12 appends.
>
> | If you see... | Action |
> |---|---|
> | A row in `_inbox.md` you want to move to a thematic file | Run `/triage`; let it sort the row |
> | A row in `_inbox.md` you want to drop | Run `/triage`; mark "drop" during the drain step |
> | A row missing from `_inbox.md` for a freshly-merged spec | Re-run `/task` Step 12 manually (or wait for the next merged spec to trigger it) |
> | An entry whose source-spec section shape was unrecognised by the parser | Step 12 emits a warning; resolve by reformatting the source spec OR by adding the shape to the parser's allow-list (Issue A2 design phase) |
```

(No "deliberate forward note" prose follows the AXIOM in `AGENTS.md` itself — the meta-plan and spec carry that explanation; the AXIOM stays bare per Pattern 1's "rule-only blockquote" shape.)

### Edit 5: `AGENTS.md` — *Agent Docs* row (Task 5)

**Insert into the *Agent Docs* table** (currently lines 222–235), as a new row after the `ai-docs/plans/deferred/` row (current line 232) and before `ai-docs/bugfix/trace-*.md` (current line 233):

```markdown
| `ai-docs/deferred/_inbox.md` | triage queue — rows from completed specs awaiting `/triage` classification (writers: `/task` Step 12 and `/triage` only; introduced in Issue A2). |
```

(2-column row matching the rest of the table.)

## Open questions

None blocking design. The spec's *Open questions* section noted one residual ambiguity (whether `widget-backlog.md`'s 5 emoji statuses should all surface or only `🟡 v2`); the spec itself resolved that to `🟡 v2` only. The design adopts the spec's resolution.

If the implementation agent discovers during AC1 verification that `🟡 v2` produces too few candidates to be meaningful, the design phase can revisit via Design Amendment — but on current data the 8 thematic files alone already supply ≥ 5 untracked rows (the AC1 spot-check list), so `widget-backlog.md` need not be relied on.
