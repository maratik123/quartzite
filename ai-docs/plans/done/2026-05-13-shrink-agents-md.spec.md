# Shrink AGENTS.md below 40k-char performance threshold

**Source:** user description (free-text)
**Date:** 2026-05-13
**Tracked in:** #323

## Scope

1. Reduce `AGENTS.md` from its current **40,572 chars** to **≤ 32,000 chars** (~20% headroom under the 40,000-char perf threshold). Verified on disk via `wc -c AGENTS.md` before and after.
2. Preserve every binding rule in `AGENTS.md` verbatim in intent (wording may change; semantics may not):
   - Every `**AXIOM ...**` blockquote.
   - Every `**MUST**`, `**NEVER**`, `**DENY**`, `**ASK**` directive (the "fail-loud" verbs per `ai-docs/agent-writing-style.md`).
   - Every action table (`If you see X → do Y`).
   - The full Propagation Rule sync-group lookup table (the canonical lookup; agents grep here to resolve sync groups).
3. Apply the three shrink mechanisms named by the user (design agent allocates work across them to hit the target):
   - **(a) Extract verbose subsections into `ai-docs/` reference pages** (anchored links from `AGENTS.md`, following the precedent set by `## Code Style` → `ai-docs/code-style.md`). New reference pages may be created if no existing file is topically appropriate.
   - **(b) Collapse AXIOM-plus-redundant-prose duplications** — several axioms in `AGENTS.md` restate the same rule once as an AXIOM block (with table) and again as a prose paragraph immediately below. Where the prose paragraph is a verbatim or near-verbatim restatement of the AXIOM, the prose paragraph may be deleted; the AXIOM block (with its decision table) is the canonical rule.
   - **(c) Tighten tables that have placeholder / catch-all rows** (e.g., the Propagation Rule table's `Any other instruction file` row may be folded into the procedure paragraph instead).
4. Re-verify the size on disk after edits via `wc -c AGENTS.md`.

## Out of scope

- Editing instruction files other than `AGENTS.md` and the **new or existing** `ai-docs/*.md` files that absorb extracted content. (Other instruction files are touched only as required by the Propagation Rule, e.g., updating cross-references that pointed to a now-moved AGENTS.md section.)
- Editing `ai-docs/learnings.md` in the same turn as AGENTS.md edits (Corrections Log Boundary Rule 2). Existing entries' `Escalated?` / `Superseded by:` fields are also out of scope for this PR.
- Rewording any AXIOM, MUST, NEVER, DENY, or ASK directive. Only relocation (extraction into a referenced page) and dedup (AXIOM-vs-redundant-prose) are permitted.
- Restructuring the H2 section ordering of `AGENTS.md` (the `## Project / ## Permissions / ## Build & Test / ...` sequence). Sections may be shortened or their content moved out, but the H2 skeleton stays so existing memory of "look in `## Workflow`" continues to resolve.
- Re-running `/improve` or `/ai-audit` as part of this task.
- Editing the `CLAUDE.md` `@AGENTS.md` import line.

## Deferred

- Migration of *other* `.claude/skills/**` and `.claude/agents/**` files that are also approaching their own perf thresholds — track separately when measured. | independent perf chore; not blocking this one | yes, separate chore-issue per file that crosses 40k.
- A `scripts/check-agents-md-size.sh` precommit / CI gate that would mechanically prevent regression past the threshold. | nice-to-have automation, not required by this task | yes, separate tooling-issue.

## Key decisions

| Question | Decision |
|---|---|
| Target char count | **≤ 32,000 chars** (~20% headroom under the 40,000-char perf threshold). User-confirmed in round 1. |
| Top-3 heaviest sections to target first | `## Workflow` (8,281 chars), `## Corrections Log` (7,076 chars), `## Propagation Rule` (6,132 chars) — these three alone account for ~52% of file mass; shaving them is the highest-leverage path to the target. Design agent chooses the actual cut allocation across the three. |
| Per-section cut allocation strategy | Sensible default: prioritise mechanism (b) — collapse AXIOM-plus-redundant-prose duplications — across all three heavy sections first, then apply mechanism (a) — section extraction — if (b) alone is insufficient to hit ≤ 32k. Design agent may override and document the alternative. |
| Extraction destinations: new vs existing reference pages | Design agent's call, following the precedent of `## Code Style → ai-docs/code-style.md`. New reference pages may be created when no existing file is topically appropriate (likely candidates: `ai-docs/workflow.md` for the Workflow section's narrative, `ai-docs/corrections-log.md` for the Boundary Rule narratives). Anchors must match the link format already in use (`[ai-docs/<file>.md → <Heading>](ai-docs/<file>.md#<anchor>)`). |
| AXIOM-vs-prose dedup criterion | An AXIOM-block + adjacent prose paragraph qualifies as redundant **when** the prose contains no rule, exemption, mechanism, or example absent from the AXIOM block's heading text + decision table. Otherwise the prose stays (or is moved into the AXIOM's decision table). Design agent applies this criterion per occurrence. |
| Propagation Rule sync-group table | Stays in `AGENTS.md` (it is the canonical lookup; agents grep here). The **"Sync groups (canonical):"** prose list below the table, however, restates information already in the table and is a candidate for collapse or extraction (design's call). |
| Anchor stability for files that link into AGENTS.md | One known external anchor reference: `ai-docs/code-style.md` line 57 → `AGENTS.md#api-naming`, targeting `## API Naming` (not in the heavy-extraction set). Design agent re-runs `grep -rn "AGENTS.md#" .claude/ ai-docs/` once before commit; any anchor whose target H2 was renamed or removed gets the source link updated in the same PR (Propagation Rule). |
| Verification of "no binding rule lost" | Design agent supplies a concrete verification procedure (expected shape: `grep -nE 'AXIOM\|\*\*MUST\*\*\|\*\*NEVER\*\*\|\*\*DENY\*\*\|\*\*ASK\*\*' AGENTS.md` before-and-after, plus a per-extracted-section confirmation that the moved text is reachable via the link). Exact procedure is design-phase output, not spec-phase. |
| Cross-cutting reformatting | Out of scope. Only the three named shrink mechanisms are in scope. No "while we're here" cleanups of unrelated sections. |

## Technical constraints

- `AGENTS.md` is the source of truth that every agent reads on every invocation; **wording of binding rules must not weaken**. Specifically:
  - "MUST" stays "MUST"; "NEVER" stays "NEVER"; "DENY" stays "DENY"; "ASK" stays "ASK". Synonyms ("should", "may want to", "consider") are forbidden as replacements.
  - AXIOM blockquotes retain the `> **AXIOM — <one-line summary>.**` header + their decision table.
  - The Propagation Rule's per-file sync-group table stays in `AGENTS.md`; only the prose around it is a shrink candidate.
- The `ai-docs/agent-writing-style.md` patterns apply to any new content written to `AGENTS.md` or extracted into reference pages: AXIOM blockquote at top of section, fail-loud verbs, action tables, explicit file lists (no globs in binding-rule context).
- After extraction, every binding rule must remain reachable in **one click** from `AGENTS.md` — i.e., extracted content lives in `ai-docs/*.md` with stable anchors, and the `AGENTS.md` line that summarises the rule includes the anchored link. The "one-click reachable" requirement matches the existing `## Code Style` pattern.
- The Propagation Rule fires on this PR: edits to `AGENTS.md` require `grep -rn` for every changed keyword and corresponding updates to any `.claude/agents/**`, `.claude/skills/**`, or `ai-docs/**` file that referenced the same rule by quoted text or anchor.
- Corrections Log Boundary Rule 2: the implementation turn's commits MUST NOT include changes to `ai-docs/learnings.md` (no new learning entry written in the same turn as instruction-file edits, except via `/improve` or `/ai-audit`).
- The `CLAUDE.md` file imports `AGENTS.md` via `@AGENTS.md`; the import line is preserved as-is. Any content previously inlined into `AGENTS.md` and now extracted to `ai-docs/` is reachable from `AGENTS.md` via a link, so the `CLAUDE.md` reader path is preserved.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `wc -c AGENTS.md` reports a value **≤ 32,000** (strictly: `[ "$(wc -c < AGENTS.md)" -le 32000 ]`). |
| AC2 | Every AXIOM blockquote present in `AGENTS.md` at the start of the PR is either (a) still present in `AGENTS.md` verbatim in intent, or (b) extracted to a referenced `ai-docs/*.md` file with the AXIOM blockquote format preserved; in case (b), `AGENTS.md` retains a one-line summary plus anchored link. Verified by enumerating AXIOMs before/after via `grep -nE '^\s*> \*\*AXIOM'`. |
| AC3 | Every MUST / NEVER / DENY / ASK directive (matched case-sensitively via `grep -nE '\*\*(MUST\|NEVER\|DENY\|ASK)\*\*'`) is preserved verbatim or extracted with an anchored link from `AGENTS.md`. No directive is reworded to a softer synonym. |
| AC4 | The Propagation Rule sync-group lookup table remains in `AGENTS.md` (`grep -nE '^> \| If you edit\.\.\. \| You MUST also check / update\.\.\. \|' AGENTS.md` finds the table header). |
| AC5 | Every external file that linked into `AGENTS.md` via an anchor (`grep -rn 'AGENTS.md#' .claude/ ai-docs/`) either still resolves (target H2 unchanged) or has been updated in the same PR to the new location. |
| AC6 | `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo build`, and `cargo test` all pass — i.e., the shrink touched no Rust code and broke nothing. (Trivially true if only `AGENTS.md` and `ai-docs/*.md` changed; AC enforces the "no scope drift" boundary.) |
| AC7 | No **new** learning entry is appended to `ai-docs/learnings.md` in the same conversation turn as the `AGENTS.md` implementation edits (Corrections Log Boundary Rule 2 protects edit-turn pairing, not commit-pairing). Pre-existing working-tree changes to `ai-docs/learnings.md` from prior turns MAY be staged with the implementation commit per AGENTS.md *Workflow* — they are part of the task deliverable and must appear in the PR diff. |
| AC8 | Propagation Rule observed: for every quoted text or anchor that this PR moved out of `AGENTS.md`, `grep -rn` across `.claude/agents/`, `.claude/skills/`, and `ai-docs/` was run and any matches were updated in the same PR. |

## Open questions

_(None — all design-affecting questions resolved. Per-section cut allocation is a design-phase decision, not an open question.)_
