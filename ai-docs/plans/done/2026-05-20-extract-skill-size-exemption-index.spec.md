# Extract per-skill size-exemption comments to a dedicated index file

**Source:** issue #502
**Date:** 2026-05-20
**Tracked in:** #502

## Scope

Relocate the six inline `<!-- size-exemption: ... -->` HTML comments currently
sitting at the top of each over-target `SKILL.md` into a single index file at
`ai-docs/skill-size-exemptions.md`. The comments today exist purely for
verification gates (`/ai-audit` Sub-check K1, the deferred
`scripts/check-instruction-file-sizes.sh`, ad-hoc `grep -L` recipes); they have
zero behavioural effect on the skill loader but cost ~250–400 chars of
per-invocation context budget each. Centralising matches the existing
`ai-docs/panic-index.md` / `ai-docs/unsafe-index.md` precedent (facts about
source files audited by gates, gathered in one place).

In scope:

1. **Create** `ai-docs/skill-size-exemptions.md` — schema modelled on
   `panic-index.md` / `unsafe-index.md`:
   - Preamble explaining what the index is for + how it is consumed
     (`/ai-audit` Sub-check K1, deferred `scripts/check-instruction-file-sizes.sh`).
   - One per-entry section per exempted SKILL.md (six entries on landing day:
     `master-ci-failed`, `pr-ci-failed`, `pr-commented`, `bugfix`, `interview`,
     `task`) with a `{Field | Value}` table containing the SKILL path, `wc -l`
     line count at audit time, load-bearing reason (verbatim or
     paraphrased from the existing inline comment), and category map (cat-2 /
     cat-3 per the #498 spec § Scope vocabulary).
   - `## Notes` section explaining the supersede-relationship to #498 spec
     AC1's "in-file comment" wording, the new contract ("in-index-file row OR
     ≤ 200 lines"), and entry-removal rules (drop the entry when a SKILL
     shrinks below 200).
2. **Strip** the six existing `<!-- size-exemption: ... -->` comments from
   `master-ci-failed/SKILL.md`, `pr-ci-failed/SKILL.md`, `pr-commented/SKILL.md`,
   `bugfix/SKILL.md`, `interview/SKILL.md`, `task/SKILL.md`. Each file loses
   the comment line plus the blank line above it (~2 lines each).
3. **Update `/ai-audit` Checklist K item 1** (the "Oversized SKILL.md"
   sub-check, lives in `.claude/skills/ai-audit/reference.md § Checklist K`)
   to: (a) read the new `ai-docs/skill-size-exemptions.md` instead of
   `grep`-ing each SKILL.md for the inline comment; (b) detect **drift** by
   recomputing `wc -l` for each listed SKILL and comparing against the
   index's cited line count.
4. **Add an AGENTS.md § Agent Docs row** for the new index file (precedent:
   `panic-index.md` and `unsafe-index.md` are NOT listed there today — see
   Key decisions row 1; this spec resolves that inconsistency for the new
   index by adding it explicitly).
5. **Add an AGENTS.md § Propagation Rule row** binding the new index file
   to its consumers (the `/ai-audit` checklist + the deferred
   `scripts/check-instruction-file-sizes.sh` spec) so that future per-skill
   edits that flip a file's exemption status propagate to the index.
6. **Update the existing AGENTS.md / instruction-surface mention of `<!-- size-exemption: ... -->`**
   wherever the inline-comment contract is documented as the source of truth.
   The new contract — "an entry in `ai-docs/skill-size-exemptions.md` OR the
   file is ≤ 200 lines" — supersedes the inline-comment contract from #498
   spec AC1 wholesale.

## Out of scope

- Landing `scripts/check-instruction-file-sizes.sh` itself (deferred,
  tracked in #383 + the deferred-inbox rows). This task only **updates** the
  pre-merge design's reference target so the future gate reads the index.
- Auditing `.claude/agents/*.md` files for the same size pattern (separately
  deferred via #498 spec § Out of scope).
- Re-extracting more content from any SKILL.md (independent extraction
  work; #501 closed that work for now).
- Renaming, restructuring, or moving any of the six exempted SKILL.md files.
- Adding or removing entries from `ai-docs/panic-index.md` or
  `ai-docs/unsafe-index.md` (they are precedent, not part of this task).

## Deferred

- AGENTS.md § Agent Docs symmetry for `panic-index.md` + `unsafe-index.md`
  (already tracked as an open question in
  `ai-docs/deferred/_inbox.md`, unsafe-index source). This spec adds a row
  for `skill-size-exemptions.md` only; the symmetry question stays open. |
  cohesion with the new index might motivate adding panic / unsafe rows in
  a follow-up | separate issue not needed yet (already in the inbox).
- Generalising the audit-script-versus-instruction-surface tension (e.g.,
  any future "audited list of source files" should live in `ai-docs/` rather
  than as per-file inline comments). | design-pattern claim worth recording
  if a third index lands | separate issue likely no.

## Key decisions

| Question | Decision |
|---|---|
| Does the new index file go under `AGENTS.md § Agent Docs`? | YES — add a row. `panic-index.md` / `unsafe-index.md` not having rows is recorded as an open question in `_inbox.md`; this spec does not touch that question but does add the new file's row so the index is discoverable from AGENTS.md. The new index file is consumed by an active audit pathway (Checklist K), strengthening the case versus the more passive existing indices. |
| Drift-finding severity in Checklist K item 1? | `minor`. Issue body offered "nit or minor"; `minor` matches the existing K1 severity (the oversized-SKILL.md detection itself is `minor`) and treats stale audit metadata as a real audit-trail concern rather than cosmetic. |
| What counts as drift? | Any line-count mismatch between the index's cited `wc -l` value and the live `wc -l` value. The two demo conditions in AC3 (file shrunk → entry should be removed; file grew → reason may need updating) are sub-cases the finding text distinguishes by direction. A tolerance band is NOT introduced (zero churn cost: editor / triage runs the audit then bumps the index line count in the same commit that touched the SKILL). |
| Where does the supersede-#498-AC1 contract live? | `ai-docs/skill-size-exemptions.md § Notes` (canonical), with one-line forward references from (a) the new file's preamble and (b) the existing AGENTS.md-level reference to `<!-- size-exemption: ... -->` content (Scope item 6). #498 spec under `ai-docs/plans/done/` is NOT edited — its file lives in the done tree and the supersede claim is recorded forward, not retroactively. |
| Entry order in `skill-size-exemptions.md`? | Descending by `wc -l` at audit time (largest first). Mirrors the "biggest offenders first" reading order; provides natural drift-finding context (the file most likely to need re-extraction is at the top). |
| Per-entry schema (key fields)? | Five fields per entry: **SKILL path**, **wc -l at audit time** (numeric), **wc -c at audit time** (numeric, for 40k-char-cap context), **load-bearing reason** (verbatim or paraphrased from the existing inline comment), **category map** (cat-2 round-template scaffolding / cat-3 workflow narrative + workflow-time tables; cat-1 absent by definition). Five was chosen over a smaller schema because (a) the inline comments already encode this much, (b) `wc -c` gives the audit context for the 40k AXIOM without a second lookup, and (c) the category map is the reason the exemption is load-bearing (the audit needs it to validate the *justification*, not just the existence). |
| Should the index list SKILL files that are ≤ 200 lines? | NO. The index records EXEMPTIONS — files that are over the soft target but documented as load-bearing. A SKILL drifting below 200 lines causes its entry to be DELETED (per § Notes), not switched to a "no exemption needed" row. This keeps the index small and the "entry exists" predicate equivalent to "exemption applies". |
| How does Checklist K item 1 detect a NEWLY-oversized SKILL (one not in the index)? | Same mechanism as today — `wc -l .claude/skills/*/SKILL.md` filtered to > 200 lines, then `comm`/`grep -F` against the index entry paths. Files > 200 with no index entry → existing K1 finding (`minor`, "extract or document exemption"). The drift sub-check is additive: it does NOT replace the > 200-and-no-exemption finding; both fire from the same K1 pass. |

## Technical constraints

- The new file `ai-docs/skill-size-exemptions.md` must stay under the 40,000-char
  AGENTS.md AXIOM cap. Six entries plus preamble plus notes will land far
  below that; no extraction concern for the index itself.
- Six SKILL.md files (`master-ci-failed`, `pr-ci-failed`, `pr-commented`,
  `bugfix`, `interview`, `task`) lose their first `<!-- size-exemption: ... -->`
  comment line and the (typically blank) line immediately above it. The first
  Workflow heading or other top-of-file content shifts up by 2 lines per file.
- Pre-merge `wc -l` of the six files is the source of truth for the index's
  cited line-count column on landing day. Post-strip line counts (which will
  be 2 lines shorter per file) are what the index records.
- The supersede-#498-AC1 contract is documented forward (in the new index
  file + the AGENTS.md / instruction-surface mention). The #498 spec file
  under `ai-docs/plans/done/` is NOT edited — done-tree specs are historical
  artefacts.
- `/ai-audit` reference.md edit MUST follow Propagation Rule and the
  `.claude/skills/ai-audit/SKILL.md` row pointing at Checklist K stays
  consistent (the SKILL row references reference.md by anchor; if the anchor
  text shifts, the SKILL row updates in the same commit).
- The deferred `scripts/check-instruction-file-sizes.sh` design (tracked in
  #383 + `_inbox.md`) MUST be updated to name the new index file as its
  source of truth before that gate is implemented. This spec only commits
  to documenting the intention; the script itself stays deferred.

## Acceptance Criteria

| #   | Criterion |
|-----|-----------|
| AC1 | `grep -rn "<!-- size-exemption:" .claude/skills/` returns zero matches — all six inline markers stripped. |
| AC2 | `ai-docs/skill-size-exemptions.md` exists, contains six entries (one per `master-ci-failed`, `pr-ci-failed`, `pr-commented`, `bugfix`, `interview`, `task`), each with the five fields (SKILL path, `wc -l` at audit time, `wc -c` at audit time, load-bearing reason, category map per the #498 vocabulary). |
| AC3 | `/ai-audit` Checklist K item 1 (in `.claude/skills/ai-audit/reference.md § Checklist K`) reads `ai-docs/skill-size-exemptions.md` as its exemption source AND detects drift (any line-count mismatch between the index's cited `wc -l` and the live `wc -l`); severity = `minor`. A demo run on master surfaces zero findings; a demo run on a hand-edited SKILL (added or removed ≥ 1 line without updating the index) surfaces a drift finding. |
| AC4 | AGENTS.md § Agent Docs has a row for `ai-docs/skill-size-exemptions.md` pointing to the file with a one-line purpose ("audited list of `.claude/skills/*/SKILL.md` files exempted from the 200-line soft target; consumed by `/ai-audit` Checklist K item 1 + deferred `scripts/check-instruction-file-sizes.sh`"). |
| AC5 | AGENTS.md § Propagation Rule has a row mapping `ai-docs/skill-size-exemptions.md` ↔ `.claude/skills/ai-audit/reference.md` (and the deferred `scripts/check-instruction-file-sizes.sh` once it lands) so that any edit to the index re-checks the audit's anchor + cited line counts. |
| AC6 | The new index file's `## Notes` section explicitly supersedes #498 spec AC1's "in-file comment" wording with the new contract: "an entry in `ai-docs/skill-size-exemptions.md` OR the file is ≤ 200 lines". |
| AC7 | No skill workflow regression — the six SKILL.md files load and execute identically; only the stripped comment changes. Verify by reading the top of each SKILL.md and confirming the next meaningful content (Workflow heading, first paragraph, compaction-recovery callout) is the same as pre-edit. |
| AC8 | `wc -l` totals: each of the six SKILL.md files shrinks by 2 lines (1 comment line + 1 surrounding blank line) versus its pre-edit length. Document the per-file delta in the commit message. |
| AC9 | Self-review passes on the diff (`/task` Step 10). No `major` / `blocker` findings; `minor` / `nit` are folded back into the same branch. |

## Open questions

- **AGENTS.md § Agent Docs symmetry with `panic-index.md` / `unsafe-index.md`.**
  Adding only `skill-size-exemptions.md` (and not the older two) leaves the
  symmetry question that
  `ai-docs/deferred/_inbox.md` already tracks (unsafe-index spec, open-question
  row). This spec does not resolve that — the symmetry follow-up either adds
  both older rows in a later edit or removes the new row to keep all three
  consistent. Default chosen here: add the new row since the index is
  actively consumed by Checklist K (panic-index / unsafe-index are passive
  documentation).
- **Drift-finding output format.** The exact wording of the audit finding
  (e.g., does it print both numbers, suggest the fix, link to the SKILL?)
  is a presentation choice the design agent decides. The spec only fixes
  severity (`minor`) and trigger (any line-count mismatch).
- **Should the index also track `wc -c` drift?** The 40k-char-cap AXIOM is
  the actual budgetary concern; line count is a proxy. Spec records both
  numbers per entry (Key decisions row 6) but Checklist K item 1's drift
  detection compares line count only. Char drift detection is a possible
  follow-up if it adds signal; the chars value in the entry serves as
  audit-time documentation regardless.
