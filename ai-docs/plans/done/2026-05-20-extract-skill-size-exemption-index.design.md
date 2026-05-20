# Design: Extract per-skill size-exemption comments to a dedicated index file

**Issue:** #502
**Spec:** [`ai-docs/plans/2026-05-20-extract-skill-size-exemption-index.spec.md`](2026-05-20-extract-skill-size-exemption-index.spec.md)
**Date:** 2026-05-20

## Approach

Land `ai-docs/skill-size-exemptions.md` as the new single source of truth for
the six "over-target-but-load-bearing" `SKILL.md` files, modelled exactly on
the existing `ai-docs/panic-index.md` / `ai-docs/unsafe-index.md` precedent
(preamble + per-entry `### <title>` blocks containing a `{Field | Value}` table
+ a terminal `## Notes` section). The six inline `<!-- size-exemption: ... -->`
HTML comments are stripped from the SKILL.md files in the same PR. `/ai-audit`
Checklist K item 1's grep recipe is rewritten to (a) read the index for the
exemption list and (b) compare each listed SKILL's cited `wc -l` against the
live `wc -l`, emitting a `minor` drift finding on mismatch — additive to the
existing "> 200 lines with no exemption" finding, both fire from the same K1
pass. AGENTS.md gets one `§ Agent Docs` row and one `§ Propagation Rule` row
for the new index; the supersede-#498-AC1 contract is documented forward in
the new file's `## Notes` section. The forward-pointer to the deferred
`scripts/check-instruction-file-sizes.sh` (issue #383 + the existing inbox
rows) lives ONLY in the new index file's `## Notes` and — as a one-line
purpose phrase — in the new AGENTS.md § Agent Docs row. **No edit to
`ai-docs/deferred/_inbox.md` is performed** (AGENTS.md AXIOM: that file is
written only by `/task` Step 12 and `/triage`); the existing inbox rows for
#383 continue to track the script's eventual landing without modification.

### Why this approach

1. **Precedent compatibility.** Two indices already follow this shape
   (`panic-index.md`, `unsafe-index.md`); a third one with the same schema
   minimises reader learning cost and lets future "audited list of source
   files" content (spec § Deferred row 2) reuse the pattern without inventing
   a third format. The shared shape — preamble → per-entry tables → `## Notes`
   — is already what readers expect from an `ai-docs/*-index.md` file.

2. **Per-invocation budget.** Each of the six inline comments costs ~250–400
   chars in the context budget for every skill invocation. Centralising them
   frees ~1.5–2.4 KB total across the six skills (most-used hot-path
   `pr-commented` / `pr-ci-failed` / `task` benefit most), at no cost — the
   audit gate that consumes the centralised list runs on demand, not at every
   skill spawn. The spec's framing is the right one: the comments are "verification
   gates" residue, not workflow content.

3. **Drift detection as audit-side back-stop.** Today, a SKILL.md author can
   edit a SKILL up or down and forget to update the inline comment's line
   count — the inline comment stays stale silently. Centralising into one
   file and adding a `wc -l`-recompute check in K1 turns the silent staleness
   into a `minor` audit finding the next time `/ai-audit` runs. This matches
   the spec's stated rationale and matches how `panic-index.md` /
   `unsafe-index.md` work today (they are read by reviewers, not by an
   automated gate, but the file shape supports both).

### Rejected alternatives

- **Keep inline comments + add an index that mirrors them.** Doubles the
  source of truth; introduces a third drift surface (inline vs index, on top
  of inline vs live `wc -l`). Rejected — the spec is explicit (Scope item 2)
  that the inline comments are stripped.

- **Index every SKILL.md (including ≤ 200 ones) as "exemption: none".** Adds
  ~7 entries with empty bodies and turns the "entry exists ⇒ exemption applies"
  predicate into "entry exists AND exemption-field non-empty". The spec rejects
  this in Key decisions row 7 (smaller index, simpler predicate, deletion-on-shrink
  rule). Rejected per spec.

- **Use YAML or TOML for the index.** The two precedent indices use Markdown
  tables; switching format would split the precedent family. Markdown also
  keeps the entries human-readable in PR diffs, which is the primary
  consumption surface. Rejected.

- **Detect drift via `wc -c` (char count) instead of `wc -l`.** Char drift is
  the actual budgetary concern (40k AXIOM is char-based), but line count is
  a stable proxy: editing a single word changes char count, not line count.
  Drift detection on lines correctly fires only when content structure shifts.
  The `wc -c` value is recorded in the entry for audit-time documentation per
  spec Key decisions row 6, but the K1 drift comparator uses `wc -l` only
  (spec § Open question row 3). Rejected by spec.

- **Add an `ai-docs/templates/skill-size-exemptions.md` template instead.**
  Templates live under `ai-docs/templates/` and are reusable scaffolding; this
  file is a registry, not a template. Wrong directory by AGENTS.md convention
  — registries / indices live directly under `ai-docs/`. Rejected.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **Create `ai-docs/skill-size-exemptions.md`** with the full per-entry schema. Preamble explains the file's purpose, its two consumers (`/ai-audit` Checklist K item 1 today; deferred `scripts/check-instruction-file-sizes.sh` (#383) later), and the entry-removal rule (drop the entry when a SKILL shrinks to ≤ 200 lines). Six per-entry blocks under `## Active entries` in descending `wc -l` order with **post-strip** `wc -l` / `wc -c` (the file shape this index references is the one that exists AFTER subtask 2 strips the comment + the blank line above it): `master-ci-failed` (373 lines, 22,885 chars), `pr-ci-failed` (326 lines, 22,320 chars), `pr-commented` (305 lines, 21,419 chars), `bugfix` (257 lines, 14,095 chars), `interview` (227 lines, 12,127 chars), `task` (219 lines, 25,002 chars). The post-strip `wc -c` values in the table below are computed-from-pre-strip arithmetic (pre-strip `wc -c` minus the comment-line + blank-line byte total, both pre-measured during design iteration 2), not measured live against post-strip files. Subtask 1 therefore has no execution dependency on subtask 2. Computation: each pre-strip `wc -c` minus (comment-line byte length incl. newline + 1 byte for the blank line above) = post-strip `wc -c`; per-file removed-byte totals are 372 / 309 / 406 / 229 / 219 / 291 respectively. Each entry is a `{Field \| Value}` table with the five fields: **SKILL path**, **wc -l at audit time** (post-strip), **wc -c at audit time** (post-strip), **load-bearing reason** (paraphrased verbatim from the existing inline comment to preserve category vocabulary), **category map** (e.g. `cat-2 round-template scaffolding + cat-3 workflow narrative + cat-3 workflow-time tables`). Terminal `## Notes` section: (a) the supersede-#498-AC1 contract — "an entry in `ai-docs/skill-size-exemptions.md` OR the file is ≤ 200 lines" replaces #498 spec AC1's "in-file comment" wording wholesale; (b) entry-removal rule when a SKILL drops to ≤ 200; (c) cross-reference to the inactive `scripts/check-instruction-file-sizes.sh` (#383) — this is the ONLY forward-pointer to #383 added by this PR (no `_inbox.md` edit). | **new** `ai-docs/skill-size-exemptions.md` | — |
| 2 | **Strip the six inline `<!-- size-exemption: ... -->` comments** + the blank line below each comment (so the file shrinks by exactly 2 lines: the comment + the trailing blank). Verified by spot-read: each affected SKILL has the shape `---` (frontmatter close, line 6 or 7) → blank → `<!-- size-exemption: ... -->` → blank → first body paragraph. Removing the `<!-- size-exemption: ... -->` line and the blank line immediately below it leaves the shape `---` → blank → first body paragraph (two-line shrink, frontmatter-close still followed by exactly one blank line — matches the in-tree convention used by all non-exempted SKILLs). | `.claude/skills/master-ci-failed/SKILL.md`, `.claude/skills/pr-ci-failed/SKILL.md`, `.claude/skills/pr-commented/SKILL.md`, `.claude/skills/bugfix/SKILL.md`, `.claude/skills/interview/SKILL.md`, `.claude/skills/task/SKILL.md` | 1 |
| 3 | **Update `/ai-audit` Checklist K item 1** (in `.claude/skills/ai-audit/reference.md § Checklist K`) to consume the new index file. Rewrite item 1's body in three parts: (a) Read `ai-docs/skill-size-exemptions.md` and parse the per-entry SKILL paths + their cited `wc -l`. (b) Run `wc -l .claude/skills/*/SKILL.md` against the live tree and identify all files > 200 lines that are NOT in the index — flag as the existing K1 "oversized + no exemption" finding (severity `minor`, same as today). (c) For each SKILL listed in the index, compare its cited `wc -l` against the live `wc -l` — flag any mismatch as a `minor` drift finding ("`<path>`: index cites X lines, live is Y lines"). Both finding categories are additive and fire from the same K1 pass. Cited audit-time line counts in the index → match against live. Severity unchanged: `minor`. The deferred-script forward-reference stays: "Mechanical pre-commit gate planned in #383; this audit-side back-stop fires in the meantime." | `.claude/skills/ai-audit/reference.md` | 1 |
| 4 | **Add AGENTS.md § Agent Docs row + § Propagation Rule row.** (a) `§ Agent Docs` (immediately after the `api-naming.md` row, before `ai-docs/templates/` — keeps `ai-docs/*-index.md` files clustered if a future row adds the symmetry follow-up for `panic-index.md` / `unsafe-index.md`): row text per spec AC4 — `\| \`ai-docs/skill-size-exemptions.md\` \| Audited list of \`.claude/skills/*/SKILL.md\` files exempted from the 200-line soft target; consumed by \`/ai-audit\` Checklist K item 1 + deferred \`scripts/check-instruction-file-sizes.sh\`. \|`. (b) `§ Propagation Rule` (insert just before the catch-all "Any other instruction file" row): row text — `\| \`ai-docs/skill-size-exemptions.md\` \| \`.claude/skills/ai-audit/reference.md\` (Checklist K item 1 anchor + cited \`wc -l\` numbers MUST stay synchronised; deferred \`scripts/check-instruction-file-sizes.sh\` (#383) reads the same index once landed) (Size-exemption-index group) \|`. The reciprocal direction (edits to `reference.md § Checklist K` re-check the index) is covered by the existing catch-all + the new row reading both directions. | `AGENTS.md` | 1 |
| 5 | **Final-sweep verification.** (a) `grep -rn "<!-- size-exemption:" .claude/skills/` returns zero matches (AC1). (b) `wc -l .claude/skills/{master-ci-failed,pr-ci-failed,pr-commented,bugfix,interview,task}/SKILL.md` returns 373, 326, 305, 257, 227, 219 respectively — confirms the per-file 2-line shrink (AC8); AND `wc -c` on the same six files returns 22,885 / 22,320 / 21,419 / 14,095 / 12,127 / 25,002 respectively, matching the post-strip char counts cited in `ai-docs/skill-size-exemptions.md` § Active entries (AC2 numeric back-stop — fails loud if either `wc -l` or `wc -c` drifts vs. the index on landing day). (c) Read top-of-file of each stripped SKILL and confirm the next meaningful content (commit-authorisation callout for `master-ci-failed`/`pr-ci-failed`/`pr-commented`/`task`; workflow narrative for `bugfix`/`interview`) is unchanged from pre-edit (AC7). (d) Cross-reference dry-run: read `ai-docs/skill-size-exemptions.md` and `realpath` the SKILL paths listed in each entry (AGENTS.md *Markdown link tracing* rule for new/moved markdown). (e) Re-read `.claude/skills/ai-audit/reference.md § Checklist K` and confirm the new K1 body grammars-correctly into the existing severity list (`minor` matches). (f) `grep -rn "skill-size-exemptions" .claude/ AGENTS.md ai-docs/` and confirm exactly the expected reference sites: the new index file, `.claude/skills/ai-audit/reference.md`, `AGENTS.md` (two new rows), the spec, this design (no orphans, no missing sites). (g) Demo-run validation per AC3: Trace the K1 drift-detection recipe by hand against a hypothetical `wc -l` of `<cited+1>` for one of the six SKILLs — read the index's cited value, mentally apply `live > cited` → emit drift finding with severity `minor`, body referencing both numbers and direction (grew / shrank). No actual SKILL edit, no working-tree mutation. AC3's drift-detection coverage is satisfied by tracing the recipe. | All seven affected files (read-only verification) | 1, 2, 3, 4 |

## Handoff plan

`M = 5`. Two groups (3 + 2). Non-terminal Group A is exactly 3 subtasks; terminal Group B is 2 subtasks (within `1..=3`).

- **Entry to Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) before starting subtask 1 — every-group handoff contract applies to the first group identically.
- **Group A:** subtasks 1–3 — land the new index file (subtask 1), strip the six inline comments (subtask 2), rewire Checklist K item 1 to read the index (subtask 3). These three subtasks are the load-bearing content edits; subtask 1 is the foundational artefact, subtask 2 establishes the new contract surface, subtask 3 wires the gate to the new artefact.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context. The handoff is mandatory regardless of how much budget Group A actually consumed (every-group handoff contract).
- **Group B:** subtasks 4–5 — terminal group (2 subtasks; within the `1..=3` range). AGENTS.md propagation (subtask 4) is content-light but rule-heavy (needs to read the existing § Agent Docs ordering + § Propagation Rule shape carefully); final-sweep verification (subtask 5) reads every affected file once and runs the AC1 + AC7 + AC8 grep / read commands. Both fit a fresh-context group with comfortable headroom.

## Risks

- **R1 — Drift between cited line count and post-commit line count.** The
  index records `wc -l` numbers that are accurate at the moment subtask 2
  commits. If subtask 3 (which edits `reference.md`, not any SKILL.md) somehow
  triggers an unintended SKILL edit, the cited numbers would already be stale
  on landing day. **Mitigation:** subtask 5's verification step (b) re-runs
  `wc -l` against the index's cited numbers before commit and fails loud on
  mismatch. AC3's "zero findings on master" demo is the post-merge confirmation.

- **R2 — Wrong supersede semantics for the #498 spec.** The spec is explicit
  that the #498 spec file under `ai-docs/plans/done/` is NOT edited — the
  supersede claim lives forward in `ai-docs/skill-size-exemptions.md § Notes`
  only. A naive reading of the spec could lead to a "fix" of #498's AC1
  wording in the done tree. **Mitigation:** subtask 1's `## Notes` text uses
  the spec § Key decisions row 4 wording verbatim ("forward, not retroactively");
  subtask 5 final-sweep step (f) `grep -rn "in-file comment"` confirms no
  stale references survive without touching `ai-docs/plans/done/`.

- **R3 — Checklist K item 1 grammar drift.** The current K1 body is a
  single-paragraph rule. The rewrite splits into 3 parts (parse index +
  detect "oversized no-exemption" + detect drift). If the rewrite changes
  severity wording, the existing `minor` severity at item 1 could drift to
  `major` for one branch and `minor` for another. **Mitigation:** spec AC3
  fixes both branches at `minor`; subtask 3 carries that constraint into the
  rewrite explicitly. Subtask 5 step (e) reads the rewritten K1 body and
  confirms the severity word appears exactly once per branch.

- **R4 — AGENTS.md § Agent Docs ordering churn.** The table is alphabetised
  loosely (extracted reference docs first, then `templates/`, then `plans/`,
  then `deferred/`, then `bugfix/`, then `learnings.md`, then skill rows).
  Inserting the new row "between `api-naming.md` and `templates/`" preserves
  the loose grouping. **Mitigation:** subtask 4 cites the exact anchor row
  ("after `api-naming.md`, before `ai-docs/templates/`") so the agent does
  not need to re-derive an ordering rule.

- **R5 — Propagation Rule row over-fanning the audit anchor.** The new row
  binds `skill-size-exemptions.md` ↔ `ai-audit/reference.md`. If a future
  SKILL becomes exempt and the editor adds it to the index, that edit also
  re-runs the audit anchor check — adding noise on every index growth. **Mitigation:**
  the row's intent is "anchor + cited numbers stay synchronised" which IS the
  desired re-check (the audit must validate the new entry's `wc -l` against
  the live SKILL on the same PR). Acceptable noise; matches the spec's
  intent (AC5).

- **R6 — Deferred-script forward-reference becomes a phantom dependency.**
  The new index file references `scripts/check-instruction-file-sizes.sh`
  as a future consumer in two places (preamble + `## Notes`). If #383 never
  lands, those references stay phantoms. **Mitigation:** the spec's § Technical
  constraints already commits to "documenting the intention" only; the index
  file's text uses "deferred" / "planned" language explicitly, matching the
  existing `AGENTS.md` reference to the script (which has lived as a
  forward-pointer for multiple PRs without harm). No mitigation beyond
  matching tone.

## Test Design

Instruction-surface task — no Rust tests. The "tests" are mechanical
verification commands run during subtask 5:

- **AC1 verification:** `grep -rn "<!-- size-exemption:" .claude/skills/`
  returns exit-code 1 (zero matches). Location: subtask 5 step (a).
- **AC2 verification:** `wc -l ai-docs/skill-size-exemptions.md` reports a
  non-empty file; `grep -c "^### " ai-docs/skill-size-exemptions.md`
  reports exactly 6 (one per-entry `### <title>` heading). Location:
  subtask 5 step (a) reading.
- **AC3 verification (positive — zero findings on master):** Hand-trace
  Checklist K item 1's recipe against `ai-docs/skill-size-exemptions.md`
  + the six live SKILL files and confirm `wc -l` matches index numbers
  exactly. Location: subtask 5 step (g).
- **AC3 verification (negative — drift on a hypothetical `<cited+1>` SKILL):**
  Trace the K1 drift-detection recipe by hand against a hypothetical `wc -l`
  of `<cited+1>` for one of the six SKILLs — read the index's cited value,
  mentally apply `live > cited` → emit drift finding with severity `minor`,
  body referencing both numbers and direction (grew / shrank). No working-tree
  mutation. Location: subtask 5 step (g).
- **AC7 verification:** Read `head -15` of each of the six stripped SKILLs
  pre- and post-edit; confirm the first body line (after frontmatter close
  and one blank line) is identical. Location: subtask 5 step (c).
- **AC8 verification:** `wc -l .claude/skills/{master-ci-failed,pr-ci-failed,pr-commented,bugfix,interview,task}/SKILL.md`
  reports exactly 373, 326, 305, 257, 227, 219 (pre-edit was 375, 328, 307,
  259, 229, 221 — each shrunk by 2). `wc -c` on the same six reports
  22,885 / 22,320 / 21,419 / 14,095 / 12,127 / 25,002 (pre-edit was
  23,257 / 22,629 / 21,825 / 14,324 / 12,346 / 25,293 — shrunk by
  372 / 309 / 406 / 229 / 219 / 291 bytes respectively). Both `wc -l` and
  `wc -c` must match the post-strip values cited in
  `ai-docs/skill-size-exemptions.md § Active entries`. Location: subtask 5
  step (b).
- **AC9 verification:** Self-review at `/task` Step 10 is the unconditional
  gate; spec AC9 names it explicitly. No design-side test needed.

No fixtures or helpers needed; all checks are direct grep / wc / Read calls
against the working tree.

## Open questions

None — the spec § Key decisions table resolves every contested point, and
spec § Open questions row 1 (Agent Docs symmetry for `panic-index.md` /
`unsafe-index.md`) is explicitly deferred to the inbox follow-up rather
than design-time. Spec § Open questions rows 2 (drift-finding output
format) and 3 (`wc -c` drift) are presentation-side and tracked-follow-up
respectively; both resolved here at the design level — subtask 3 specifies
the finding wording (`"<path>: index cites X lines, live is Y lines"`) and
subtask 1 records `wc -c` per entry for audit-time documentation without
adding it to K1's drift comparator.
