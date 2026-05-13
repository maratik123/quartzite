# Shrink `ai-docs/context.md` below 35k-char speed bump

**Source:** issue #328
**Date:** 2026-05-13
**Tracked in:** #328

## Scope

1. Reduce `ai-docs/context.md` from its current **37,139 chars** (217 lines) to **≤ 30,000 chars** (~14% headroom under the 35,000-char project-side speed bump from the AGENTS.md § *Build & Test* "instruction-file size" AXIOM landed in PR #327). Verified on disk via `wc -c ai-docs/context.md` before and after.
2. Preserve every architecturally-relevant decision, key entity definition, and cross-cutting plan entry from the current `ai-docs/context.md` — wording may change, semantics may not. "Preserve in intent" means: a reader looking up "what does the project say about X?" finds the same answer after the PR, either inline in `context.md` or via a one-click anchored link from `context.md` to an `ai-docs/*.md` reference page.
3. Apply the three shrink mechanisms named in the issue body (design agent allocates work across them to hit the target):
   - **(a) Extract verbose subsections into `ai-docs/<topic>.md` reference pages** with anchored links from `context.md` — following the precedent set by PR #324 (`AGENTS.md § Workflow → ai-docs/workflow.md`, `AGENTS.md § Corrections Log → ai-docs/corrections-log.md`).
   - **(b) Collapse near-verbatim repetitions** across the Maintenance plans list and the Key Decisions table — where the same point is stated in both places, keep one canonical statement.
   - **(c) Tighten the Maintenance plans list** to one-line summaries per entry (currently ~3–5 lines each), letting the linked spec carry the detail. The current list has ~30 entries; one-line summaries with `[spec](plans/done/<file>)` links are the issue body's explicitly-suggested mechanism.
4. Re-verify the size on disk after edits via `wc -c ai-docs/context.md`.

## Out of scope

- Editing instruction files other than `ai-docs/context.md` and the **new or existing** `ai-docs/*.md` files that absorb extracted content. (Other instruction files are touched only as required by the Propagation Rule — e.g., updating `ai-docs/plans/INDEX.md` if the inbound `#maintenance-plans-cross-cutting` anchor reference would otherwise break.)
- Editing `ai-docs/learnings.md` in the same conversation turn as `ai-docs/context.md` implementation edits (Corrections Log Boundary Rule 2). Existing entries' `Escalated?` / `Superseded by:` fields are out of scope for this PR.
- Restructuring the H2 section ordering of `ai-docs/context.md`. The current sequence — `## Purpose` → `## Crate Layout` → `## Concept Mapping` → `## Out of Scope` → `## Core Architecture` → `## Key Design Decisions` → `## Plans (Implementation Order)` → `## Open Questions` — stays so existing memory of "look in `## Key Design Decisions`" continues to resolve. Sections may be **shortened** or have their content **moved out**, but the H2 skeleton stays.
- Editing the `CLAUDE.md` `@AGENTS.md` import line (`context.md` is not imported by `CLAUDE.md` today; flagged here per issue body in case future edits add such an import).
- Re-running `/improve` or `/ai-audit` as part of this task.
- Changes to any Rust source file — this is documentation hygiene only.

## Deferred

- Migration of *other* `ai-docs/{code-style,doc-convention,agent-writing-style,corrections-log}.md` files that the AGENTS.md size AXIOM also targets but which have not yet crossed the 35k speed bump. | independent perf chore; not blocking this one | yes, separate chore-issue per file that crosses 35k.
- A `scripts/check-instruction-file-sizes.sh` precommit / CI gate that would mechanically prevent regression across all instruction files at once. | nice-to-have automation, named in the AGENTS.md AXIOM as "Until ... lands"; not required by this task | yes, separate tooling-issue (this is the AXIOM-named follow-up).

## Key decisions

| Question | Decision |
|---|---|
| Target char count | **≤ 30,000 chars** (~14% headroom under the 35,000-char project-side speed bump). Issue-stated. |
| Current heaviest sections | `## Key Design Decisions` table (~17,813 chars, ~48% of file mass, 59 rows) and `## Plans (Implementation Order)` § Maintenance plans (~7,718 chars, ~30 entries × 3–5 lines). These two together account for ~69% of file mass. The introductory matter (`## Purpose` through `## Core Architecture`) is ~8,501 chars and is mostly architecturally-load-bearing — least likely to be a shrink target. |
| Default mechanism for Maintenance plans | Compress each entry to a one-line summary that retains the spec slug + `[spec](plans/done/<file>)` link. Per-entry one-line shape: `**<slug>** — <one-line summary>. [spec](...)`. The verbose multi-line description moves into the linked spec only if it isn't already there; otherwise it is dropped because the spec is the canonical source. Expected save: ~4–5k chars. |
| Default mechanism for Key Decisions table | Design-agent's call between (a) in-place row condensing — keep table in `context.md`, shorten each row's "Decision" cell to the architectural takeaway and drop implementation-detail clauses; (b) extraction to a new `ai-docs/key-decisions.md` reference page with an anchored link from `context.md`; or (c) judgment-call mix per row (architecturally-load-bearing rows stay condensed inline; archival implementation-note rows extract). Sensible default: **(c)** — mirrors PR #324's mixed approach (some content stayed AXIOM-in-place, some extracted to reference pages). Design agent documents the per-row partition. |
| Inbound anchor stability | `ai-docs/plans/INDEX.md` line 115 links to `../context.md#maintenance-plans-cross-cutting`. This anchor MUST remain valid after the PR — either by keeping the heading text `### Maintenance plans (cross-cutting)` in `context.md` or by updating `INDEX.md` in the same PR. Design agent re-runs `grep -rn "context.md#" .claude/ ai-docs/ AGENTS.md CLAUDE.md` once before commit; any anchor whose target was renamed or removed gets the source link updated in the same PR (Propagation Rule). |
| Extraction destinations: new vs existing reference pages | Design-agent's call, following PR #324 precedent. New reference pages may be created when no existing file is topically appropriate (likely candidates: `ai-docs/maintenance-plans.md` if compression alone is insufficient; `ai-docs/key-decisions.md` if extraction is chosen over in-place condensing). Anchors use the existing link format `[ai-docs/<file>.md → <Heading>](<file>.md#<anchor>)`. |
| Repetition-dedup criterion (mechanism b) | A point counts as "redundantly stated" when (i) it appears in the `## Key Design Decisions` table AND in the same-PR-related Maintenance plans entry, OR (ii) it appears in `## Crate Layout` long-form *and* repeats in a Maintenance plans entry, AND in either case the wording is verbatim or near-verbatim. The canonical statement stays in the table / `## Crate Layout`; the Maintenance plans copy is dropped from the one-line summary. |
| Anchor stability for restructured headings | If a heading is renamed during shrink (e.g., `### Maintenance plans (cross-cutting)` → `### Maintenance plans`), every inbound link in `.claude/`, `ai-docs/`, `AGENTS.md`, `CLAUDE.md` is updated in the same PR. Default: do not rename headings that already have known inbound anchors. |
| Verification of "no architectural content lost" | Design agent supplies a concrete verification procedure (expected shape: enumerate the `## Key Design Decisions` rows + Maintenance-plans entries before-and-after via `grep -nE` for the row-header and slug-name patterns, plus a per-extracted-section confirmation that the moved text is reachable via the link). Exact procedure is design-phase output. |
| Cross-cutting reformatting | Out of scope. Only the three named shrink mechanisms are applied. No "while we're here" cleanups of unrelated sections. |
| New learning entries in this turn | None permitted (Corrections Log Boundary Rule 2). Existing working-tree changes to `learnings.md` from prior turns MAY be staged with the implementation commit per AGENTS.md *Workflow*. |

## Technical constraints

- The AGENTS.md § *Build & Test* "instruction-file size" AXIOM sets a **35,000-char project-side speed bump** and a **40,000-char harness-enforced soft cap**. Target ≤ 30,000 chars gives ~14% headroom under the speed bump and ~25% headroom under the soft cap, matching the headroom ratio PR #324 used for AGENTS.md (~20% under 40k).
- `ai-docs/context.md` is read on demand (not on every invocation per `AGENTS.md § Project`), so the shrink is preventative — it keeps the file from reaching the harness cap as the project grows. Same threshold applies regardless of load frequency per the AXIOM enumeration.
- **Wording of architecturally-load-bearing statements must not weaken.** Specifically:
  - Crate names, trait names, attribute names, and type names (`ObjectBase`, `AsObject`, `Object`, `ObjectExt`, `Value`, `Signal`, `MetaObject`, `WidgetBase`, etc.) stay verbatim — these are grep targets.
  - Decision wording for any row in `## Key Design Decisions` that is the *only* place a design choice is documented must preserve the takeaway (the table is the canonical reference for cross-cutting decisions that don't have a dedicated spec).
- The Propagation Rule fires on this PR for any cross-file edit: `grep -rn "context.md" .claude/ ai-docs/ AGENTS.md CLAUDE.md` before commit; any reference that no longer resolves (renamed heading, moved content) gets updated in the same PR.
- Per the AGENTS.md § *Workflow* axiom 1, the implementation work runs on a feature branch (`chore/2026-05-13-shrink-context-md` or similar), never on local `master`.
- After extraction, every architecturally-relevant decision must remain reachable in **one click** from `context.md` — extracted content lives in `ai-docs/*.md` with stable anchors, and the `context.md` line that summarises the rule includes the anchored link. The "one-click reachable" requirement matches the PR #324 `## Code Style → ai-docs/code-style.md` pattern.
- `ai-docs/agent-writing-style.md` patterns apply to any new content written to `context.md` or extracted into reference pages.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `wc -c ai-docs/context.md` reports a value **≤ 30,000** (strictly: `[ "$(wc -c < ai-docs/context.md)" -le 30000 ]`). |
| AC2 | The H2 section sequence of `ai-docs/context.md` after the PR matches before the PR: `## Purpose`, `## Crate Layout`, `## Concept Mapping`, `## Out of Scope`, `## Core Architecture`, `## Key Design Decisions`, `## Plans (Implementation Order)`, `## Open Questions`. Verified by `grep -nE '^## ' ai-docs/context.md` diff being empty (heading text and order both preserved). |
| AC3 | Every crate listed in `## Crate Layout` at the start of the PR is still listed (by crate name) in `## Crate Layout` after the PR. Verified by `grep -nE '^\| \`quartzite' ai-docs/context.md` before / after — the set of crate names is identical. (Per-crate purpose blurbs may be tightened but the table row must remain.) |
| AC4 | Every Key Design Decisions table row present at the start of the PR is either (a) still present in `ai-docs/context.md` with the architectural takeaway preserved, or (b) extracted to a referenced `ai-docs/*.md` file with `ai-docs/context.md` retaining a one-line summary plus anchored link. Verified by enumerating the row "Question" cells before / after via `grep -nE '^\| ' ai-docs/context.md` (in the `## Key Design Decisions` block) plus a manual cross-check of any extracted-into reference page. |
| AC5 | Every Maintenance plans entry slug (`cleanup-progress-issue-derive`, `shrink-agents-md`, `project-docs`, …) present at the start of the PR is either (a) still present in `ai-docs/context.md` § Maintenance plans as a one-line summary with a working `[spec](plans/done/<file>)` link, or (b) relocated to a referenced `ai-docs/*.md` file with the link preserved. Verified by enumerating slugs before / after. |
| AC6 | Every Crate-level plan entry (the numbered list in `## Plans (Implementation Order)`, items 1–13 at start of PR) is still present in `ai-docs/context.md` with its ✅ status marker intact. |
| AC7 | The `### Maintenance plans (cross-cutting)` heading exists in `ai-docs/context.md` after the PR (so the inbound anchor `context.md#maintenance-plans-cross-cutting` from `ai-docs/plans/INDEX.md` line 115 still resolves), OR `INDEX.md` is updated in the same PR to the new anchor. Verified: `grep -nE '^### Maintenance plans' ai-docs/context.md` matches AND `grep -rn 'context.md#' .claude/ ai-docs/ AGENTS.md CLAUDE.md` shows every link still resolves to a real heading. |
| AC8 | `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo build`, and `cargo test` all pass — i.e., the shrink touched no Rust code. (Trivially true if only `ai-docs/*.md` changed; AC enforces the "no scope drift" boundary.) |
| AC9 | No **new** learning entry is appended to `ai-docs/learnings.md` in the same conversation turn as the `ai-docs/context.md` implementation edits (Corrections Log Boundary Rule 2). Pre-existing working-tree changes to `ai-docs/learnings.md` from prior turns MAY be staged with the implementation commit per AGENTS.md *Workflow*. |
| AC10 | Propagation Rule observed: for every quoted text, slug, or anchor that this PR moved out of `ai-docs/context.md`, `grep -rn` across `.claude/agents/`, `.claude/skills/`, `ai-docs/`, `AGENTS.md`, and `CLAUDE.md` was run and any matches were updated in the same PR. |

## Open questions

_(None — design-phase decisions deferred to design agent are recorded in `## Key decisions` as defaults with documented alternatives.)_
