# Squeeze `AGENTS.md` and `ai-docs/context.md` below the perf threshold

**Source:** issue #497
**Date:** 2026-05-20
**Tracked in:** #497

## Scope

1. Reduce `ai-docs/context.md` from its current **43,351 chars** to below the 40,000-char hard cap declared in the AGENTS.md *Build & Test* AXIOM ("Every project instruction file Claude loads per invocation MUST stay below 40,000 chars"). Verified on disk via `wc -c ai-docs/context.md` before and after.
2. Reduce `AGENTS.md` from its current **37,673 chars** to below the **35,000-char early-warning band** declared in the same AXIOM. Verified on disk via `wc -c AGENTS.md` before and after.
3. Apply the **PR #324 / PR #201 extraction model** (precedent: `2026-05-13-shrink-agents-md` for AGENTS.md; `2026-05-07-code-style-extraction` for the original `code-style.md` lift; `ai-docs/key-decisions.md` for the body of context.md's Key Design Decisions table). Specifically:
   - **(a) Extract verbose subsections into `ai-docs/<topic>.md` reference pages** with anchored links from the source file. New reference pages may be created when no existing file is topically appropriate.
   - **(b) Collapse AXIOM-plus-redundant-prose duplications** where the prose paragraph is a verbatim or near-verbatim restatement of the AXIOM block's heading + decision table.
   - **(c) Tighten tables that have placeholder / catch-all rows** (e.g., the Propagation Rule table's Task/Design group repeats the same group across four rows; condense to one anchor row + see-also).
4. Preserve every binding rule verbatim in intent (wording may change; semantics may not):
   - Every `**AXIOM ...**` blockquote.
   - Every `**MUST**` / `**NEVER**` / `**DENY**` / `**ASK**` directive (fail-loud verbs per `ai-docs/agent-writing-style.md`).
   - Every action table (`If you see X → do Y`).
   - The full Propagation Rule sync-group lookup table (the canonical lookup; agents grep here).
5. For each new reference page created, add a row to the AGENTS.md `## Agent Docs` table per the Propagation Rule.
6. Re-verify sizes on disk via `wc -c` after edits. The full instruction-file size scan (`wc -c AGENTS.md CLAUDE.md .claude/skills/**/SKILL.md .claude/agents/**.md ai-docs/{code-style,doc-convention,context,agent-writing-style,corrections-log}.md`) is run once before commit to confirm no sibling instruction file regressed.

### Primary extraction targets

| Source file | Subsection | Current size | Destination |
|---|---|---|---|
| `ai-docs/context.md` | `## Plans (Implementation Order)` → `### Maintenance plans (cross-cutting)` (L184–229) | ~15,900 chars | **NEW** `ai-docs/plans-summary.md` (analogous to `ai-docs/key-decisions.md`). Context.md retains the section heading + a thin pointer paragraph + anchored link. |
| `AGENTS.md` | `## Workflow` (~8,123 chars) | candidate | Design's call — condense compound bullets; some are canonical and stay. |
| `AGENTS.md` | `## Learning Log` (~6,580 chars) | candidate | Boundary-rule Exception bodies overlap with `ai-docs/corrections-log.md`; condense the Exception body to the rule statement + canonical pointer (the pointer already exists). |
| `AGENTS.md` | `## Propagation Rule` (~5,331 chars) | candidate | Task/Design group occupies 4 rows pointing to the same group; condensable to one anchor row + see-also rows. |

The design agent allocates the actual char-cut across these targets to hit the AC numbers. The `## Code Style` section (~4,545 chars) already follows the extraction pattern and is treated as optimal — no further cuts.

## Out of scope

- Editing instruction files other than `AGENTS.md`, `ai-docs/context.md`, and the new/existing `ai-docs/*.md` files that absorb extracted content. (Other instruction files are touched only as required by the Propagation Rule — e.g., updating cross-references that pointed to a now-moved AGENTS.md or context.md section.)
- Editing `ai-docs/learnings.md` in the same turn as instruction-file edits (Corrections Log Boundary Rule 2). Existing entries' `Escalated?` / `Superseded by:` fields are out of scope.
- Rewording any AXIOM, MUST, NEVER, DENY, or ASK directive. Only relocation (extraction into a referenced page) and dedup (AXIOM-vs-redundant-prose, repeated table rows) are permitted.
- Restructuring the H2 section ordering of either file. Sections may be shortened or have content moved out, but the H2 skeleton stays so existing memory of "look in `## Workflow`" / "look in `## Key Design Decisions`" continues to resolve.
- The **7 oversized `.claude/skills/*/SKILL.md` files** flagged by `/ai-audit` Sub-check K1 — tracked separately per the issue body.
- Re-running `/improve` or `/ai-audit` as part of this task.
- Editing the `CLAUDE.md` `@AGENTS.md` import line.

## Deferred

- A `scripts/check-instruction-file-sizes.sh` pre-commit / CI gate that would mechanically prevent regression past the 40k cap or the 35k early-warning band. | nice-to-have automation referenced by the AXIOM itself ("Until `scripts/check-instruction-file-sizes.sh` lands as a pre-commit / CI gate, …"); not required by this task | yes, separate tooling-issue.
- Shrinking the 7 oversized `.claude/skills/*/SKILL.md` files (`/ai-audit` Sub-check K1). | independent perf chore | yes, separate follow-up issue per the issue body's *Out of scope* note.

## Key decisions

| Question | Decision |
|---|---|
| `ai-docs/context.md` hard target | **< 35,000 chars** (early-warning band). The issue body says "< 40,000 target, ideally < 35,000"; pre-resolved to the stricter band so one full `/task` cycle of headroom is preserved (matches the AXIOM's stated rationale for the 35k early warning). |
| `AGENTS.md` hard target | **< 35,000 chars** (early-warning band). Issue body says "< 35,000 target"; AC pins to this band. |
| Primary extraction for context.md | New file `ai-docs/plans-summary.md` absorbing `### Maintenance plans (cross-cutting)` (~15,900 chars). Context.md retains the H3 heading + a one-line pointer paragraph + anchored link to the extracted file, analogous to how `## Key Design Decisions` keeps a one-line entry per row pointing into `ai-docs/key-decisions.md`. New file added to AGENTS.md `## Agent Docs` table. |
| Primary extraction candidates for AGENTS.md | Design agent's call among `## Workflow`, `## Learning Log`, `## Propagation Rule`. Sensible default: prioritise mechanism (b) (AXIOM-vs-redundant-prose dedup) and mechanism (c) (table-row collapse, Propagation Rule Task/Design group) before mechanism (a) (extraction into new reference pages), to minimise new files. |
| Extraction destinations: new vs existing reference pages | Design agent's call. Likely candidates: extend `ai-docs/workflow.md` (already exists, 1,484 chars) with extracted Workflow narrative; extend `ai-docs/corrections-log.md` (already exists, 13,621 chars) with extracted Boundary Rule Exception bodies. New files only when no existing file is topically appropriate. |
| Anchor format for extraction links | `[ai-docs/<file>.md → <Heading>](<file>.md#<anchor>)` from `AGENTS.md`, `[<file>.md → <Heading>](<file>.md#<anchor>)` from `ai-docs/context.md` (relative-path discipline per AGENTS.md *Workflow*). Format already in use across `AGENTS.md`. |
| AXIOM-vs-prose dedup criterion | An AXIOM block + adjacent prose paragraph qualifies as redundant **when** the prose contains no rule, exemption, mechanism, or example absent from the AXIOM block's heading text + decision table. Otherwise the prose stays (or is moved into the AXIOM's decision table). Same criterion the 2026-05-13 spec used. |
| Propagation Rule sync-group table | Stays in `AGENTS.md` (canonical lookup; agents grep here). Within the table, repeated rows for a single sync group (e.g., Task/Design listed 4 times) may be condensed to one anchor row plus see-also rows. The table-header shape stays so external lookups still resolve. |
| Anchor stability for files that link in | Before commit, run `grep -rn "AGENTS.md#\|context.md#" .claude/ ai-docs/` and update any source link whose target H2/H3 was renamed or removed in the same PR (Propagation Rule). Specifically guard the existing `AGENTS.md#api-naming` reference from `ai-docs/code-style.md` (out of extraction set, but still verify). |
| Verification of "no binding rule lost" | Design agent supplies the procedure (expected shape: `grep -nE 'AXIOM\|\*\*MUST\*\*\|\*\*NEVER\*\*\|\*\*DENY\*\*\|\*\*ASK\*\*' AGENTS.md` before-and-after enumeration, plus a per-extracted-section confirmation that the moved text is reachable via the link). |
| `ai-docs/plans/INDEX.md` | Out of scope. INDEX.md is a separate file (not part of context.md) and is not in the AXIOM's enumerated list. |
| Cross-cutting reformatting | Out of scope. Only the three named shrink mechanisms (a)/(b)/(c) are in scope. No "while we're here" cleanups of unrelated sections. |

## Technical constraints

- Both `AGENTS.md` and `ai-docs/context.md` are read on every agent invocation (`AGENTS.md` always, `context.md` on demand but routinely). Wording of binding rules MUST NOT weaken:
  - "MUST" stays "MUST"; "NEVER" stays "NEVER"; "DENY" stays "DENY"; "ASK" stays "ASK". Synonyms ("should", "may want to", "consider") are forbidden as replacements.
  - AXIOM blockquotes retain the `> **AXIOM — <one-line summary>.**` header + their decision table.
  - The Propagation Rule's per-file sync-group table stays in `AGENTS.md`; only the prose around it and intra-table row repetition are shrink candidates.
- The `ai-docs/agent-writing-style.md` patterns apply to any new content written to `AGENTS.md` / `context.md` or extracted into reference pages: AXIOM blockquote at top of section, fail-loud verbs, action tables, explicit file lists (no globs in binding-rule context).
- After extraction, every binding rule and every Key Design Decision row must remain reachable in **one click** from the source file — i.e., extracted content lives in `ai-docs/*.md` with stable anchors, and the source-file line that summarises the rule / decision includes the anchored link. Matches the existing `## Code Style` and `## Key Design Decisions` patterns.
- The Propagation Rule fires on this PR. Edits to `AGENTS.md` require `grep -rn` for every changed keyword and corresponding updates to any `.claude/agents/**`, `.claude/skills/**`, or `ai-docs/**` file that referenced the same rule by quoted text or anchor. Edits to `context.md` require the same grep against `.claude/`, `ai-docs/`, and any source file that referenced moved decision-row text.
- Corrections Log Boundary Rule 2: the implementation turn's commits MUST NOT include changes to `ai-docs/learnings.md` (no new learning entry written in the same turn as instruction-file edits, except via `/improve` or `/ai-audit`).
- `CLAUDE.md`'s `@AGENTS.md` import line is preserved as-is. Any content extracted to `ai-docs/` is reachable from the source file via a link, so the `CLAUDE.md` reader path is preserved.
- Branch protocol: per AGENTS.md *Workflow* AXIOM 1, create the feature branch (`chore/2026-05-20-squeeze-agents-context-size` or similar) **before** any file edit.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `wc -c ai-docs/context.md` reports a value **< 35,000** (strictly: `[ "$(wc -c < ai-docs/context.md)" -lt 35000 ]`). |
| AC2 | `wc -c AGENTS.md` reports a value **< 35,000** (strictly: `[ "$(wc -c < AGENTS.md)" -lt 35000 ]`). |
| AC3 | Every AXIOM blockquote present in `AGENTS.md` at the start of the PR is either (a) still present verbatim in intent, or (b) extracted to a referenced `ai-docs/*.md` file with the AXIOM blockquote format preserved; in case (b), `AGENTS.md` retains a one-line summary plus anchored link. Verified by enumerating AXIOMs before/after via `grep -nE '^\s*> \*\*AXIOM'`. |
| AC4 | Every MUST / NEVER / DENY / ASK directive (matched via `grep -nE '\*\*(MUST\|NEVER\|DENY\|ASK)\*\*' AGENTS.md ai-docs/context.md`) is preserved verbatim or extracted with an anchored link from the source file. No directive is reworded to a softer synonym. |
| AC5 | Every action table (`If you ... → do ...`) present in `AGENTS.md` / `ai-docs/context.md` at the start of the PR is preserved verbatim or relocated with the table intact and an anchored link from the source file. |
| AC6 | The Propagation Rule sync-group lookup table remains in `AGENTS.md` (`grep -nE '^> \| If you edit\.\.\. \| You MUST also check / update\.\.\. \|' AGENTS.md` finds the table header). |
| AC7 | Every new reference page created by this PR appears as a row in the `AGENTS.md` `## Agent Docs` table (Propagation Rule requirement). |
| AC8 | Every external file that linked into `AGENTS.md` or `ai-docs/context.md` via an anchor (`grep -rn 'AGENTS.md#\|context.md#' .claude/ ai-docs/`) either still resolves (target heading unchanged) or has been updated in the same PR to the new location. |
| AC9 | `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build`, and `cargo test` all pass — i.e., the shrink touched no Rust code and broke nothing. (Trivially true if only `.md` files changed; AC enforces the "no scope drift" boundary.) |
| AC10 | No **new** learning entry is appended to `ai-docs/learnings.md` in the same conversation turn as the instruction-file implementation edits (Corrections Log Boundary Rule 2 protects edit-turn pairing, not commit-pairing). Pre-existing working-tree changes to `ai-docs/learnings.md` from prior turns MAY be staged with the implementation commit per AGENTS.md *Workflow*. |
| AC11 | Propagation Rule observed: for every quoted text or anchor that this PR moved out of `AGENTS.md` or `ai-docs/context.md`, `grep -rn` across `.claude/agents/`, `.claude/skills/`, and `ai-docs/` was run and any matches were updated in the same PR. |
| AC12 | The full instruction-file size scan (`wc -c AGENTS.md CLAUDE.md .claude/skills/**/SKILL.md .claude/agents/**.md ai-docs/{code-style,doc-convention,context,agent-writing-style,corrections-log}.md`) is run once before commit; no sibling instruction file crossed 40,000 chars as collateral damage from extracted content landing there. |

## Open questions

_(None — all design-affecting questions resolved. Per-section cut allocation across the three AGENTS.md candidates is a design-phase decision, not an open question.)_
