# Design: `/task` Step 12 propagation + `_inbox.md` creation + one-shot deduped backfill

**Issue:** [#203](https://github.com/maratik123/quartzite/issues/203) (umbrella A2 of `ai-docs/plans/2026-05-10-process-improvements.md`)
**Spec:** [`2026-05-10-task-step12-inbox-backfill.spec.md`](2026-05-10-task-step12-inbox-backfill.spec.md)
**Date:** 2026-05-11
**Branch:** `feat/2026-05-10-task-step12-inbox-backfill`
**Base commit:** A1 merged at `6701bd8`

## Approach

A2 is prompt-only work. There is no Rust code, no shell script, no separate binary. Every behaviour ships as edits to `.claude/skills/task/SKILL.md`, one new markdown file (`ai-docs/deferred/_inbox.md`), a one-line edit to `AGENTS.md`, and a directory of committed `*.spec.md` fixtures.

The audit appendix at the end of this document (`Appendix A`) walks all 111 files under `ai-docs/plans/done/` (56 specs, 55 designs — note: the spec body cites *55 specs / 54 designs* as the meta-plan rev-3 snapshot; the live count is one higher in each because `done/` grew between meta-plan rev 3 and design phase) and enumerates every shape encountered across the three section headings *Out of scope* / *Deferred* / *Open questions*. The audit drives every design-affecting decision below; it is not advisory text.

**Decision-gate output:** **option (a) — multi-shape parser with warn-and-skip fallback.** Rationale recorded in *Approach → Decision gate* below; per-shape parser rules in *Approach → Parser specification*; per-spec skip log in `Appendix A.3`.

### Decision gate

The spec's *Preconditions* required choosing one of:

- **(a)** multi-shape parser with warn-and-skip fallback;
- **(b)** canonical shape locked forward + per-spec skip list for non-conforming specs.

**Chosen: (a).** Three reasons:

1. **The corpus is already heterogeneous and per-section-heterogeneous.** Out of 121 emittable spec/design sections (`Appendix A.1`), four shapes account for >95 % (PLAINBULLET 81, PIPEBULLET3 20, PIPEBULLET2 13, BOLDBULLET 4) and two specs even mix PIPEBULLET2 and PLAINBULLET inside the *same* spec across different sections (e.g. `2026-05-01-runtime.spec.md` — *Out of scope* PLAINBULLET, *Deferred* PIPEBULLET2). Forcing a single canonical shape forward (option b) would require either (i) rewriting 93 historical spec sections to the canonical shape — out of scope for A2 and politically wrong because `done/` is supposed to be immutable history; or (ii) accepting that the backfill is silent for 93 sections — defeats the spec's *AC3* ("emit-or-skip structurally per section").
2. **The shape distribution is bounded and small.** Six well-defined shapes cover the entire corpus; one of them (`NONE` sentinel) requires zero parser work. Six rules in skill-prompt logic is not heavyweight.
3. **Future-proofing is cheap.** When a new shape appears in a forward spec, the parser emits a warning per the spec's *Out of scope* row 4 of the AGENTS.md AXIOM action table. The user reformats the spec or appends a rule. No data loss; no Step-12 block.

### Parser specification

The parser walks a single spec or design file, locates each of the three target headings, parses the section body per the rules below, and emits one row per item to `_inbox.md`. The parser MUST anchor on `^## <Heading>$` (exact h2 match with trailing whitespace tolerated) — substring matching on bare heading text is forbidden (the *Technical constraints* rule about column-header anchoring applies analogously to heading anchoring here).

For each section body found, the parser classifies its shape using the following ordered rules. The first rule that matches the entire body wins. Mixed-shape sections (e.g. a body containing both BOLDBULLET and PLAINBULLET lines, as in `2026-05-10-object-property-serialization-layer.design.md` *Open questions*) are handled by running per-line classification within the body — each line is parsed by its own rule.

**Shape rules:**

1. **NONE sentinel.** Body matches one of:
   - `_None._` / `None.` / `None` (literal, optionally underscored as markdown emphasis)
   - `(none — …)` / `(none -- …)` / `_(none)_` (parenthesised sentinel with optional gloss)
   - `None — …` / `None - …` (the word `None` followed by an em-dash or hyphen and prose, where the prose is metadata not a list)
   - `None at spec time.` / `None blocking …` (boilerplate close to the above)

   Matcher: collapse whitespace, lower-case, strip wrapping `_` and `()`, then test against the regex `^none\b.*$` with `len(collapsed) < 250` AND no `- ` bullet lines AND no `|` table pipes. **Emit zero rows; emit zero warnings.** This is the only case where the parser is allowed to be silent.

2. **TABLE.** Section body's first non-blank line starts with `|` and is a markdown table header (followed by a `|---|...|` separator line). Per-row extraction:
   - Skip the header row and the separator row.
   - For each remaining row `| C1 | C2 | C3? | ... |`:
     - `Item` = `C1` (verbatim, including markdown formatting like backticks and bold).
     - If C2 contains text and is not the empty string, append ` — <C2>` to `Item` (so the inbox row carries the rationale; matches how thematic-file rows carry rationale today).
     - Subsequent columns are ignored at backfill time (`/triage` re-reads the source spec for full context when promoting).
   - Regex anchor: `^\|\s*[^|]+\s*\|` for header detection; `^\|` for row detection; terminate on first blank line or first `^## ` line.

3. **PIPEBULLET3.** Section body's bullet lines match `^- (.+?) \| (.+?) \| (.+?)$` — a bulleted list whose items contain three `|`-separated fields. Per-row extraction:
   - `Item` = field 1 + ` — ` + field 2 (the third field is metadata like "Separate issue needed?" and is dropped at backfill time).
   - Regex: `^- ([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*(.+?)\s*$`.

4. **PIPEBULLET2.** Section body's bullet lines match `^- (.+?) \| (.+?)$` — a bulleted list whose items contain two `|`-separated fields. Per-row extraction:
   - `Item` = field 1 + ` — ` + field 2.
   - Regex: `^- ([^|]+?)\s*\|\s*(.+?)\s*$`.

5. **BOLDBULLET.** Section body's bullet lines match `^- \*\*([^*]+)\*\*\s*[—-]\s*(.+)$` — a bulleted list whose items begin with a bolded leading term followed by an em-dash (`—`) or hyphen (`-`) and prose. Per-row extraction:
   - `Item` = `**<term>**` + ` — ` + `<prose>` (the leading bold survives into the inbox to preserve the term/explanation split).
   - Regex: `^- \*\*([^*]+)\*\*\s*[—-]\s*(.+)$`.

   This is the shape the spec calls out as "~5 spec sections per spec sample"; the audit confirms 4 pure BOLDBULLET sections plus 1 mixed (BOLDBULLET,PLAINBULLET).

6. **PLAINBULLET.** Section body's bullet lines match `^- (.+)$` and do NOT match any of rules 3 / 4 / 5 above (rule ordering enforces that pipe-bullet and bold-bullet shapes take precedence). Per-row extraction:
   - `Item` = the entire bullet text, verbatim.
   - Multi-line continuation: lines starting with 2+ spaces of indentation and no leading `- ` are joined into the preceding bullet with a single space. This handles the very common case visible in `2026-05-09-paint-style.spec.md` *Out of scope* (each bullet spans 2–3 wrapped lines).
   - Regex: `^- (.+(?:\n\s{2,}.+)*)$` (multi-line aware) — but in prompt-driven skill logic, the equivalent rule is: "read until the next `- ` or blank line or `^## `, joining wrapped continuations".

**Unrecognised body shape.** If a section has a non-blank body that fails every rule above (including NONE), the parser emits a one-line warning:

```
WARN: ai-docs/plans/done/<file>.spec.md :: <section heading> — unrecognised body shape, no rows emitted
```

The warning surfaces in Step 12 output; Step 12 completes; `_inbox.md` is unchanged for that section.

### Per-row mapping to `_inbox.md`

Each parsed row becomes a 4-cell `_inbox.md` row:

```
| <Item> | [<source-label>](<source-path>) | <section-key> | — |
```

Where:

- **`<Item>`** — the `Item` value produced by the matched shape rule above.
- **`<source-label>`** — derived from the source-spec filename: strip the `YYYY-MM-DD-` date prefix and the `.spec.md` / `.design.md` suffix; append ` spec` or ` design` accordingly. Examples: `2026-05-09-paint-style.spec.md` → `paint-style spec`; `2026-05-01-auto-connection.design.md` → `auto-connection design`. This matches the existing thematic-file convention (`grep -h 'plans/done' ai-docs/deferred/*.md | head` — every link is `[<slug> spec](...)` or `[<slug> design](...)`).
- **`<source-path>`** — `../plans/done/<filename>` (the path is relative to `ai-docs/deferred/_inbox.md`, which sits in `ai-docs/deferred/`, so `..` resolves to `ai-docs/`).
- **`<section-key>`** — one of the three literal tokens `out-of-scope` / `deferred` / `open-question` (note: singular `open-question`, not `open-questions`, to match the AGENTS.md A1-landed AXIOM table cell text). The mapping is hard-coded:
  - `## Out of scope` → `out-of-scope`
  - `## Deferred` → `deferred`
  - `## Open questions` → `open-question`
- **`—`** — the literal em-dash, identical to the un-triaged marker used in the 8 thematic files' `Tracked` column. Cell-4-`Tracked` invariant honoured.

If `<Item>` contains a literal `|` character, escape it as `\|` (markdown table convention; matches how existing thematic-file rows escape pipes inside item text, e.g. `signals-slots.md:9`).

### Backfill flow

The backfill is invoked once at A2 PR-merge time as a `/task`-Step-12-equivalent skill prompt. It is not a recurring run; it has no scheduled cadence; it is not its own slash-command. Concretely:

**Input.** The full set `done/*.spec.md` ∪ `done/*.design.md`, sorted by filename (which means sorted by the `YYYY-MM-DD-` date prefix because of the project's spec-naming convention, ties broken by alphabetical slug). This is the chronological order rule — *oldest first* (spec row 4 of *Key decisions*: "Backfill order? Chronological (oldest first)").

**Per-file processing.**

1. Read the file; locate the three target sections.
2. For each non-empty / non-NONE section, run the shape rules and produce zero or more candidate rows.
3. For each candidate row, apply the **dedupe check** (see below). If dedupe says skip, log the skip to `Appendix A.3` (this design doc) citing the matching thematic-file `Source` cell. If dedupe says keep, append the row to `_inbox.md`.
4. Sections that fail every shape rule emit one warning to stdout and contribute zero rows; the section is recorded as a `WARN` row in `Appendix A.2`.

**Dedupe rule.** The dedupe key is the **`Source` link path** — the second cell of the candidate `_inbox.md` row, considered only as a path string (label and surrounding markdown stripped). The check:

1. Build a set `H` = every relative path (`../plans/done/*.{spec,design}.md`) that appears as a `Source` cell in any of the 8 thematic files (`signals-slots.md`, `properties.md`, `macros-codegen.md`, `object-tree.md`, `threading-runtime.md`, `future-crates.md`, `ci-docs-workflow.md`, `python.md`). Construction: parse every `^|` row of every thematic file's tables, extract the second cell, strip surrounding `[label](path)` to keep just `path`. (Done once at backfill start; cached for the run.)
2. For each candidate row, **normalise the candidate's `Source` path** (see *Normalisation* below) and check membership in the normalised set `H`. If matched, **skip the candidate** (any row whose source spec/design has already been harvested) and append a skip-log entry to `Appendix A.3` listing the candidate's `Item`, the spec path, and the thematic-file path where the match was found.

The audit (`Appendix A.1`) shows 14 distinct source files already harvested into the 8 thematic files: 12 specs + 2 designs (`auto-connection.design.md`, `runtime.design.md`). The dedupe applies at *file* granularity — if any row of a candidate file's section appears in a thematic file, the whole *file* is treated as already-harvested, so all of its sections are skipped. This is the spec's intended reading: the prior manual extraction passes (`a8f23d5`, `0304e8e`, `8bd8c26`) were file-at-a-time sweeps, not row-at-a-time; re-harvesting one row from a known-already-harvested file would still produce duplicates the user has to drop. *(Per-row dedupe would be defensible but breaks no current data, just adds noise — `Appendix A.3` enumerates exactly the rows skipped under file-level dedupe so a reviewer can confirm none are obvious misses.)*

**Normalisation.** The dedupe is sensitive to trivial path differences; the spec required locking one normalisation. Rules:

- **Trailing slash:** strip exactly one trailing `/` if present (none expected for file paths, but cheap insurance).
- **Anchor hashes:** strip everything from `#` onwards. (No thematic-file row currently uses fragment anchors, but `_inbox.md`'s `Source` cell would gain one if a candidate row's Source label tried to point at a specific heading — pre-empted.)
- **Case:** preserve as-is. The corpus is uniform-lowercase (a manual constraint of the date-slug naming convention), so case normalisation has no observable effect; preserving avoids accidentally collapsing two distinct files if a future contributor introduces case-sensitive paths.
- **Leading `./`:** strip if present.
- **Whitespace:** strip leading/trailing.

**Chronological order.** Files are processed in `sorted(filenames)` order; within a single file, sections in fixed order (`Out of scope` → `Deferred` → `Open questions`) regardless of their order in the source spec. Within a section, rows preserve the order they appear in the source.

**Output.** The backfill is one single pass over the corpus that appends rows to `_inbox.md` and prints warnings to stdout. `_inbox.md` is initialised (before the first append) with its header prose and a blank table header row. The backfill is then run; rows are appended below the table header. After the backfill, the design-doc skip log (`Appendix A.3`) is regenerated to match the actual run output (the design-phase audit and the implementation-phase run should produce identical skip logs by construction; any divergence is a parser bug).

### Step 12 integration

The current `task/SKILL.md` Step 12 (lines 236–273) already handles spec/design move, INDEX.md update, ROADMAP.md regeneration, commit, push, and PR creation. The A2 changes insert one new sub-step between current sub-step 3 (move spec/design to done/) and current sub-step 4 (regenerate dependent artefacts). Specifically:

**Insert new sub-step 3.5:** *Parse the just-finalised spec — and its design if present — for inbox propagation.*

- Run the parser (per *Parser specification* above) against `ai-docs/plans/done/YYYY-MM-DD-name.spec.md`.
- Run the parser against `ai-docs/plans/done/YYYY-MM-DD-name.design.md` if that file exists.
- Apply the dedupe check using the live `H` set at run time (read every `^|` row of `ai-docs/deferred/{signals-slots,properties,macros-codegen,object-tree,threading-runtime,future-crates,ci-docs-workflow,python}.md` — `widget-backlog.md` is NOT in `H`; its rows are tracked via the `Notes` cell, not via thematic-file membership, and the dedupe is about avoiding *prior thematic-file harvest*, not about whether `widget-backlog.md` already mentions an item).
- Append non-dedupe-skipped rows to `ai-docs/deferred/_inbox.md`.
- Emit warnings to stdout for any unrecognised shape, citing the spec path and section heading.
- The Step 12 commit then stages `_inbox.md` alongside the existing artefacts.

**Sub-step 4 (existing artefact regeneration) is unchanged.** `ROADMAP.md` regeneration sits *after* the inbox append because both are stage-time outputs; they do not interact. INDEX.md update is in sub-step 3 and also unchanged.

**Sub-step 6 (existing staging list) gains one item.** Add `ai-docs/deferred/_inbox.md` to the staged paths.

**Step 12 Gate-checklist row update.** The current row at `task/SKILL.md:289`:

```
| Step 12 | Branch ≠ master? INDEX.md ✅? spec/design moved to done/? Auto-derived artefacts regenerated and staged (e.g. `ROADMAP.md` from `INDEX.md`)? `Cargo.lock` refreshed? PR body references the tracking issue (`Closes #N` or `Refs #N`)? PR created and URL posted? |
```

Becomes:

```
| Step 12 | Branch ≠ master? INDEX.md ✅? spec/design moved to done/? `_inbox.md` parsed and appended (or warning logged for unrecognised shape) and staged? Auto-derived artefacts regenerated and staged (e.g. `ROADMAP.md` from `INDEX.md`)? `Cargo.lock` refreshed? PR body references the tracking issue (`Closes #N` or `Refs #N`)? PR created and URL posted? |
```

The single inserted clause `` `_inbox.md` parsed and appended (or warning logged for unrecognised shape) and staged? `` covers AC5 (Step 12 Gate checklist row mentions `_inbox.md`).

**Forward-going behaviour (no design-doc edits at Step 12 run time).** Step 12 in forward use does NOT touch this design doc. The design-doc `Appendix A` audit is a one-shot artefact of A2; subsequent forward-going Step 12 runs emit to `_inbox.md` and to stdout warnings, nothing else. The skip-log section of `Appendix A` is also one-shot (it records the backfill, not future runs).

**Backfill is also a Step 12 invocation, structurally.** The backfill is "Step 12 applied to every historical file in chronological order, with the dedupe rule keyed against current thematic-file content". The skill-prompt logic shares the parser; only the iteration target differs (one spec vs. every spec/design). This matches the spec's *Locked-in decisions* row "Backfill executable shape? Pure skill-prompt logic + bash; reuses Step 12's parser. No separate executable, no Rust binary, no shell script."

### `_inbox.md` header text

The file lives at `ai-docs/deferred/_inbox.md`. Verbatim content (header + empty table) at A2 PR-merge time, before the backfill populates rows:

```markdown
# Inbox

Untriaged rows extracted from completed plans' *Out of scope* / *Deferred* /
*Open questions* sections. Every row here is awaiting classification by
`/triage` (Issue B, [#204](https://github.com/maratik123/quartzite/issues/204))
— do not hand-edit.

This file is the universal landing zone for both forward-going propagation
(`/task` Step 12 appends one row per spec section after merging a plan) and
the one-shot backfill that seeded it. `/triage` drains rows by sorting each
into a thematic file (`signals-slots.md`, `ci-docs-workflow.md`, etc.),
promoting to a GitHub issue, or dropping with the literal `untracked`
decline-marker token written into the `Tracked` cell.

**Write discipline.** Hand-edits to this file are FORBIDDEN per the
`AGENTS.md` AXIOM (*Workflow* section, anchor `_inbox.md`) — only `/task`
Step 12 and `/triage` may write here.

**Schema.** 4-column markdown table. `Section` records which spec heading
the row was pulled from (`out-of-scope` / `deferred` / `open-question`).
`Tracked` mirrors cell 4 of the 8 thematic files — initially `—`,
rewritten to `#N` on promotion or `untracked` on decline by `/triage`.

| Item | Source | Section | Tracked |
|------|--------|---------|---------|
```

The table body is empty at file-creation time and is then populated by the backfill run. Forward Step-12 runs append rows below the existing body. The header is never modified after the file is first created.

The prose deliberately echoes the wording of `signals-slots.md`'s top line ("Items extracted from completed plans") so a reader landing on `_inbox.md` recognises the family resemblance.

### AGENTS.md *Agent Docs* row suffix drop

Current line at `AGENTS.md:243`:

```markdown
| `ai-docs/deferred/_inbox.md` | triage queue — rows from completed specs awaiting `/triage` classification (writers: `/task` Step 12 and `/triage` only; introduced in Issue A2). |
```

After A2:

```markdown
| `ai-docs/deferred/_inbox.md` | triage queue — rows from completed specs awaiting `/triage` classification (writers: `/task` Step 12 and `/triage` only). |
```

The change is the deletion of the literal substring `; introduced in Issue A2`. The trailing period stays inside the parenthesis. No other edit to `AGENTS.md` in this PR (the AXIOM at `AGENTS.md:182-189` is left untouched; it was authored to reference `_inbox.md` *and* a `/triage` skill that doesn't yet exist, and that forward note becomes live without rewording when A2 lands).

The Propagation Rule (`AGENTS.md:194-211`) fires on any AGENTS.md edit, but this suffix-drop is a no-op for every sync-group sibling — no `.claude/skills/**` or `.claude/agents/**` file references the "introduced in Issue A2" phrase (verified by `grep -rn 'introduced in Issue A2' .claude/agents/ .claude/skills/ AGENTS.md` returning the single hit on `AGENTS.md:243`; after the edit, that returns zero hits).

### Synthetic fixtures

The fixtures live at `tests/fixtures/process-improvements/`. The directory is workspace-root relative (not under any crate's `tests/`); the spec is explicit that they are not Rust test inputs but skill-driven verification inputs. Each fixture is a committed `*.spec.md` file — a syntactically valid spec with all the standard headings present (title, **Tracked in:**, **Date:**, an `## Acceptance Criteria` row, and the three target headings) so the parser sees a realistic spec shape.

Fixtures (each item: filename — purpose):

1. `shape-1-plainbullet.spec.md` — one *Out of scope* section with three PLAINBULLET items; one of them is wrapped across two lines (continuation handling). Verifies rule 6.
2. `shape-2-pipebullet2.spec.md` — one *Deferred* section with three PIPEBULLET2 items (`- Item | Why`). Verifies rule 4.
3. `shape-3-pipebullet3.spec.md` — one *Deferred* section with three PIPEBULLET3 items (`- Item | Why | Separate issue?`). Verifies rule 3 (and confirms the third column is dropped).
4. `shape-4-table.spec.md` — one *Deferred* section as a 3-column markdown table (`| What | Why | Separate issue needed? |`) with three rows. Verifies rule 2.
5. `shape-5-none.spec.md` — *Out of scope* with body `None.`; *Deferred* with body `_None._`; *Open questions* with body `(none — all resolved during interview)`. Verifies rule 1 emits zero rows and zero warnings for all three sentinel forms.
6. `shape-6-boldbullet.spec.md` — one *Out of scope* section with three BOLDBULLET items (`- **<Term>** — <prose>`). Verifies rule 5.
7. `all-three-sections.spec.md` — one *Out of scope* PLAINBULLET item, one *Deferred* PIPEBULLET2 item, one *Open questions* PLAINBULLET item. AC2 fixture: a single Step-12 run on this file appends exactly three rows with `Section` cells `out-of-scope` / `deferred` / `open-question` in that order.
8. `mangled-section.spec.md` — *Deferred* section with deliberately broken content (e.g. a single line of free prose, no bullet, no table, no sentinel — `> blockquote-style note` or `Some unparseable narrative.`). AC4 fixture: parser emits one warning, zero rows, Step 12 still completes.
9. `mixed-shapes.spec.md` — one spec exercising per-section heterogeneity: *Out of scope* PLAINBULLET (2 items), *Deferred* PIPEBULLET2 (1 item), *Open questions* BOLDBULLET (1 item). Confirms the parser handles a single spec whose three sections each use a different shape — matches the dominant real-corpus pattern.

Nine fixtures total. The spec required ≥ 6 (one per shape) plus AC2's all-three-section fixture plus AC4's mangled-section fixture, total ≥ 8; one extra (`mixed-shapes.spec.md`) covers the per-section heterogeneity that the audit revealed is the norm and that no other single fixture exercises.

Each fixture has a `## Acceptance Criteria` row containing at least one synthetic AC (e.g. `| AC1 | Fixture exists and parses cleanly | n/a |`) so the fixture passes the same shape-checks a real spec would (no surprise from `interview` validation paths if a future smoke check runs).

All fixtures use the `Tracked in: none` convention (since they are not real tasks). Fixture filenames intentionally do NOT include a `YYYY-MM-DD-` date prefix — they are not date-sorted task specs, so omitting the prefix avoids polluting the chronological ordering used by the backfill skill-prompt logic if someone accidentally pointed the backfill at the fixtures directory.

### Decomposition note

The audit (Appendix A) is part of the design, executed during design phase. Implementation in Step 8 does NOT redo the audit; it consumes the audit's outputs (shape rules, per-file skip log, fixture list) as design output and produces only the prompt-text edits to `task/SKILL.md`, the new `_inbox.md`, the AGENTS.md suffix drop, the 9 fixture files, and (during the one-shot backfill at PR-merge time) the populated body of `_inbox.md`.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `ai-docs/deferred/_inbox.md` with the header text and empty table (no rows). | `ai-docs/deferred/_inbox.md` | — |
| 2 | Drop the `; introduced in Issue A2` suffix on `AGENTS.md:243`. | `AGENTS.md` | — |
| 3 | Create the 9 synthetic fixtures under `tests/fixtures/process-improvements/`. | `tests/fixtures/process-improvements/{shape-1-plainbullet,shape-2-pipebullet2,shape-3-pipebullet3,shape-4-table,shape-5-none,shape-6-boldbullet,all-three-sections,mangled-section,mixed-shapes}.spec.md` | — |
| 4 | Insert sub-step 3.5 (parser invocation) into `task/SKILL.md` Step 12; update the Step-12 Gate-checklist row; ensure sub-step 6 lists `_inbox.md` among staged files. | `.claude/skills/task/SKILL.md` | 1 |
| 5 | Document the parser shape rules and per-row mapping inline in `task/SKILL.md` Step 12 (or as a `### Inbox propagation` sub-section adjacent to Step 12) so the skill-prompt logic is self-contained for the LLM running `/task`. | `.claude/skills/task/SKILL.md` | 4 |
| 6 | Run the one-shot backfill as a `/task`-Step-12-equivalent skill prompt walking all 111 `done/*` files in chronological order. Populate `_inbox.md`'s body with ~93 rows (post-dedupe; exact count derives from `Appendix A.3`). Verify the actual run's skip log matches `Appendix A.3`; reconcile any divergence as a parser bug. | `ai-docs/deferred/_inbox.md` (body append only — header from task 1 untouched) | 1, 4, 5 |
| 7 | Verify all 7 ACs using the recipes in *AC verification recipes* below; record results in the PR body. | none (verification step) | 6 |

7 tasks; each atomic; dependencies form a single DAG with task 6 as the synthesis point.

## Risks

- **Parser shape drift in a forward spec.** A new spec adopts a seventh shape that no rule covers; Step 12 emits a warning, but the row is lost to `_inbox.md`. *Mitigation:* the warning is loud and surfaces during the failing `/task` run; `/triage` later sees the candidate is missing and prompts the user. Adding a seventh rule is an Edit to `task/SKILL.md` — trivial.

- **Dedupe false-positive at file granularity.** A spec already harvested into thematic files has a *new* section row added later (say, a `## Errata` appendix to a `done/` spec adds a follow-up *Deferred* bullet). File-level dedupe would skip the new row. *Mitigation:* `done/` specs are append-only by AGENTS.md convention — rare event; if it occurs the user runs the backfill *manually with row-level dedupe* as a one-off. Backfill is itself a one-shot, so the cost is bounded. Audit Appendix A.3 lists exactly which files are skipped under file-level dedupe; reviewer spot-checks plausibility.

- **`_inbox.md` row order is path-sorted, not git-history-sorted.** Two specs on the same date sort alphabetically by slug, not by actual merge order. *Mitigation:* meta-plan accepts this (`Backfill order? Chronological (oldest first) so _inbox.md's row order is stable and reviewable`); the slug-tiebreak is deterministic and stable, which is the underlying goal. No mitigation needed beyond stating the rule.

- **Cell-4-`Tracked` invariant accidentally broken.** A future edit to `_inbox.md` or to a thematic file shifts the `Tracked` column to cell 5. *Mitigation:* the spec lifts this to a *Technical constraint* and the design doc echoes it; AC6 verifies the header row text is exactly `| Item | Source | Section | Tracked |`. Reviewer enforces. (If desired, a future cosmetic PR could codify a grep gate, but YAGNI — A2 has no recurring cadence that could break the invariant.)

- **`/triage` (Issue B) doesn't exist at A2 PR-merge time.** `_inbox.md`'s header references `/triage` as the future drain mechanism. *Mitigation:* the reference is deliberate — meta-plan rev 2 locked it in. Header text says "*Issue B, [#204]*" so a reader landing on `_inbox.md` before B lands sees the upcoming-skill pointer with the tracking issue.

- **Backfill produces hundreds of rows; `/triage`'s first run is overwhelming.** Post-dedupe the audit predicts ~93 rows. *Mitigation:* meta-plan-rev-3 risk row covers this — default `/triage` threshold is 3, so the first run drains a subset, then exits; user can iterate. No A2-design mitigation needed.

- **`done/` corpus grows between design-phase audit and A2 PR-merge time.** A spec lands while A2 is in flight; the spec is not in `Appendix A`. *Mitigation:* the backfill iterates the live `done/` directory at run time, not the audit list. Any new file is parsed and its rows appended; the audit appendix becomes 1 row stale but the data is correct. (Design-phase audit was 56 specs / 55 designs on 2026-05-11; live count = same; minor drift acceptable.)

- **The audit conflated `(none — gloss)` and unrecognised body.** The NONE-sentinel rule has 4 sub-patterns and could be subtly wrong on novel phrasings. *Mitigation:* fixture 5 covers three of them (`None.`, `_None._`, `(none — gloss)`); a fourth ("None at spec time.") was observed in `2026-05-07-recursive-inline-annotations.spec.md`; rule 1 explicitly enumerates it. If the parser warns on a NONE-equivalent body, the user's response is to extend the rule — same as adding a new positive shape. No row loss either way; only warning-noise risk.

## Test Design

A2 ships no Rust code, so there is no `#[cfg(test)]` module to populate. All verification is skill-driven against the synthetic fixtures committed at `tests/fixtures/process-improvements/`. The *AC verification recipes* table below substitutes for the test-design block.

### AC verification recipes

| AC | Recipe |
|----|--------|
| **AC1** | Read this design doc (`ai-docs/plans/2026-05-10-task-step12-inbox-backfill.design.md`). Confirm: (i) `Appendix A.1` enumerates all 9 shapes (NONE / TABLE / PIPEBULLET3 / PIPEBULLET2 / BOLDBULLET / PLAINBULLET / BOLDBULLET+PLAINBULLET mixed / UNCLASSIFIED-collapsed-to-NONE / MISSING-section) with section counts and example spec+section. (ii) The *Approach → Decision gate* section names option (a) (multi-shape parser with warn-and-skip fallback) and lists the three rationale points. (iii) `Appendix A.3` enumerates 28 skipped sections (14 already-harvested files × ~2 sections each — exact count audit-derived) with the matching thematic-file path for each. |
| **AC2** | After A2 lands, run `/task` Step 12 against `tests/fixtures/process-improvements/all-three-sections.spec.md` in dry-run mode (do not commit). Inspect the proposed diff: `git diff ai-docs/deferred/_inbox.md` shows exactly three new rows appended at the bottom. The `Section` cell of each row is, in order, `out-of-scope`, `deferred`, `open-question`. The `Item` cell of each row matches the fixture's content (one PLAINBULLET item, one PIPEBULLET2 item, one PLAINBULLET item). |
| **AC3** | After A2 lands, the backfill has run once at PR-merge time. Read `Appendix A.3` (skip log) and `Appendix A.2` (emit log per-section). For *every* non-empty, non-NONE section row in `Appendix A.1`, confirm it appears in exactly one of: (i) `Appendix A.2` (with a `Section`-cell-labelled `_inbox.md` row), (ii) `Appendix A.3` (with a thematic-file `Source` cite), or (iii) a `WARN` row (with the parser-warning text). Reviewer spot-checks any section by opening the source spec and confirming the matching `_inbox.md` row, thematic-file row, or warning. |
| **AC4** | After A2 lands, run `/task` Step 12 against `tests/fixtures/process-improvements/mangled-section.spec.md` in dry-run mode. Confirm: (i) stdout contains the line `WARN: tests/fixtures/process-improvements/mangled-section.spec.md :: Deferred — unrecognised body shape, no rows emitted` (path and section match the fixture). (ii) `git diff ai-docs/deferred/_inbox.md` shows no row added for the mangled section. (iii) Step 12 returns success (no non-zero exit, no abort, all other sub-steps complete). |
| **AC5** | `grep -n 'Step 12' .claude/skills/task/SKILL.md` finds the Gate-checklist row; the row body contains the literal substring `` `_inbox.md` parsed and appended ``. |
| **AC6** | `ls -la ai-docs/deferred/_inbox.md` shows the file exists. `head -25 ai-docs/deferred/_inbox.md` shows the header prose specified in *_inbox.md header text* above, including the line beginning `# Inbox`, the `/triage` reference, and the literal table-header row `| Item | Source | Section | Tracked |` followed by a separator row. `grep -c '^|' ai-docs/deferred/_inbox.md` returns ≥ 2 (header row + separator row + N body rows, N ≈ 93). `grep -n '_inbox.md' AGENTS.md` shows the AXIOM rows (`AGENTS.md:182-190` unchanged from A1) and the *Agent Docs* row (`AGENTS.md:243` with the suffix dropped). |
| **AC7** | After the backfill has run, for each of the 14 already-harvested source files (the dedupe set `H`), `grep -F '<file-basename>' ai-docs/deferred/_inbox.md` should return 0 matches against the `Source` cell. Concretely: `for f in 2026-05-01-auto-connection.spec.md 2026-05-01-auto-connection.design.md 2026-05-01-core-types.spec.md 2026-05-01-github-workflow.spec.md 2026-05-01-macros.spec.md 2026-05-01-runtime.spec.md 2026-05-01-runtime.design.md 2026-05-02-code-quality-cleanup.spec.md 2026-05-02-docs-and-facade.spec.md 2026-05-02-examples-crate.spec.md 2026-05-02-inline-simple-fns.spec.md 2026-05-02-lookup-perf.spec.md 2026-05-02-public-api-docs.spec.md 2026-05-02-signals-blocked.spec.md; do count=$(grep -cF "$f" ai-docs/deferred/_inbox.md); echo "$f: $count rows"; done` — every count must be 0. `Appendix A.3` (skip log) enumerates the corresponding skipped sections; reviewer reads `Appendix A.3` and confirms each skipped row's thematic-file cite is real. |

### Manual smoke recipe (not an AC, but recommended before merge)

After running the backfill at A2 PR-merge time:

```bash
# 1. Row count sanity:
echo "Body rows:" $(awk '/^\| Item /{flag=1; getline; getline; next} /^\| / && flag {print}' ai-docs/deferred/_inbox.md | wc -l)
# Expect ~93 (matches Appendix A.2 emit-log count).

# 2. Section-cell distribution:
awk -F'|' '/^\| / && NR > 2 {gsub(/^ +| +$/, "", $4); print $4}' ai-docs/deferred/_inbox.md | sort | uniq -c
# Expect 3 categories: out-of-scope, deferred, open-question. Counts match Appendix A.1 distribution.

# 3. Tracked cell uniformity:
awk -F'|' '/^\| / && NR > 2 {gsub(/^ +| +$/, "", $5); print $5}' ai-docs/deferred/_inbox.md | sort -u
# Expect single value: `—`.

# 4. Source-cell duplicates flag:
awk -F'|' '/^\| / && NR > 2 {gsub(/^ +| +$/, "", $3); print $3}' ai-docs/deferred/_inbox.md | sort | uniq -c | sort -rn | head
# Many `Source`-cell repeats are expected (one spec contributes ≥ 1 row to several sections); a value > 6 might indicate parser over-emission and warrants review.
```

## Open questions

None blocking implementation. Two design-phase ambiguities the audit resolved definitively:

- *Are `done/*.design.md` files in scope for the backfill?* **Yes.** The audit shows 13 design files have emittable `Open questions` sections (`Appendix A.1`); 2 of them have already been harvested into `signals-slots.md`, `object-tree.md`, and `threading-runtime.md` (`auto-connection.design.md` and `runtime.design.md`), confirming the spec's hint and locking the scope.
- *Does `next/SKILL.md` need an explicit `!`-block for `_inbox.md`?* **No.** The A1 changes already taught `/next` to read every `ai-docs/deferred/*.md` row; `_inbox.md` glob-matches that pattern by construction. An explicit block adds prose without behaviour change. Deferred-row stayer in the meta-plan (`## Deferred` of A2 spec, row 2).

---

## Appendix A — Spec-shape audit

Audit run at design-phase start, 2026-05-11, against `ai-docs/plans/done/` at base commit `6701bd8` (A1's merge to master). Corpus snapshot: 56 spec files + 55 design files = 111 source files total. Per file × 3 target headings = 333 candidate (file, heading) pairs.

### A.1 — Shape enumeration table

Distribution of `(file, heading)` pairs by classified shape, after the NONE-equivalent collapse (UNCLASSIFIED entries empirically reduce to NONE-sentinel variations the parser handles via rule 1):

| Shape | Count | Example spec + section |
|-------|------:|------------------------|
| **MISSING** (file has no section with this heading) | 115 | `2026-05-01-auto-connection.design.md` :: *Out of scope* (design doc has no *Out of scope* heading at all) |
| **NONE** (sentinel: `None.`, `_None._`, `(none — …)`, `None blocking …`) | 97 | `2026-05-05-log-facade.spec.md` :: *Open questions* (body `_None._`) |
| **PLAINBULLET** (`- <prose>`, possibly wrapped) | 81 | `2026-05-01-widgets.spec.md` :: *Out of scope* (3 wrapped bullets) |
| **PIPEBULLET3** (`- <Item> \| <Why> \| <Sep-issue?>`) | 20 | `2026-05-05-log-facade.spec.md` :: *Deferred* (3-column bullet) |
| **PIPEBULLET2** (`- <Item> \| <Why>`) | 13 | `2026-05-01-core-types.spec.md` :: *Deferred* (2-column bullet) |
| **BOLDBULLET** (`- **<Term>** — <prose>`) | 4 | `2026-05-08-project-docs.spec.md` :: *Out of scope* (5 bolded-term bullets) |
| **TABLE** (markdown table, 3 columns typical) | 2 | `2026-05-10-object-property-serialization-layer.spec.md` :: *Deferred* (3-col table) |
| **BOLDBULLET,PLAINBULLET** (mixed-shape body — one entry) | 1 | `2026-05-10-object-property-serialization-layer.design.md` :: *Open questions* |

Total: 333 (96 + 81 + 20 + 13 + 4 + 2 + 1 = 217 emittable+NONE-counting (NONE counts among non-MISSING), and 115 MISSING; sum = 333). After dropping MISSING and NONE, **emittable** count = 121 sections across 76 distinct source files.

**Shape coverage by rule:**

- Rule 1 (NONE) handles 97 sections — silent emit.
- Rule 2 (TABLE) handles 2 sections — emits up to 5 rows per section (table is variable-length).
- Rule 3 (PIPEBULLET3) handles 20 sections.
- Rule 4 (PIPEBULLET2) handles 13 sections.
- Rule 5 (BOLDBULLET) handles 4 sections + 1 mixed (5 lines classified as BOLDBULLET via the per-line classifier).
- Rule 6 (PLAINBULLET) handles 81 sections + 1 mixed remainder.

**Mixed-shape per-line classification correctness.** The single mixed section (`2026-05-10-object-property-serialization-layer.design.md` *Open questions*) opens with three BOLDBULLET items and continues with PLAINBULLET. Per-line classification within the body, with rule 5 ordered before rule 6, emits the first 3 rows under rule 5 and the remainder under rule 6 — both shapes' rows go to `_inbox.md` as one section's emit-set. The Section cell is uniformly `open-question` regardless of line shape; this is correct (the Section cell records the source heading, not the row shape).

### A.2 — Emit/skip per-section log

Total emittable sections: **121** (76 distinct files × ~1.6 sections each on average).

Per the dedupe rule (*Approach → Backfill flow → Dedupe rule*), file-level dedupe against thematic-file `Source` cells. The set `H` contains 14 entries:

```
../plans/done/2026-05-01-auto-connection.design.md
../plans/done/2026-05-01-auto-connection.spec.md
../plans/done/2026-05-01-core-types.spec.md
../plans/done/2026-05-01-github-workflow.spec.md
../plans/done/2026-05-01-macros.spec.md
../plans/done/2026-05-01-runtime.design.md
../plans/done/2026-05-01-runtime.spec.md
../plans/done/2026-05-02-code-quality-cleanup.spec.md
../plans/done/2026-05-02-docs-and-facade.spec.md
../plans/done/2026-05-02-examples-crate.spec.md
../plans/done/2026-05-02-inline-simple-fns.spec.md
../plans/done/2026-05-02-lookup-perf.spec.md
../plans/done/2026-05-02-public-api-docs.spec.md
../plans/done/2026-05-02-signals-blocked.spec.md
```

Emittable sections split:

| Bucket | Count | Behaviour |
|--------|------:|-----------|
| Emit (Source NOT in `H`) | 93 | Append to `_inbox.md` |
| Skip — file is in `H` (already harvested) | 28 | Log to `A.3`; no `_inbox.md` row |

Emit roster (the 93 emitting sections), grouped by source file — each row produces one `_inbox.md` row per item, item count derives from the shape; total approximate row count = 93 × (1–6 items per section, median ~2) ≈ ~150 individual `_inbox.md` rows. (Exact row count is the implementation-phase output; the audit fixes the *section* count at 93.)

Files contributing emits (date-sorted; each contributes the listed sections; sections marked MISSING/NONE in the file are not listed):

```
2026-05-01-geometry-events.spec.md      :: Out of scope, Deferred
2026-05-01-widgets.spec.md              :: Out of scope, Deferred, Open questions
2026-05-01-widgets.design.md            :: Open questions
2026-05-01-runtime.design.md            :: [SKIPPED — see A.3]
2026-05-01-auto-connection.design.md    :: [SKIPPED — see A.3]
2026-05-03-connect-queued-codegen.spec.md          :: Out of scope
2026-05-03-enumflags2-property-flags.spec.md       :: Out of scope, Deferred
2026-05-03-graphics-stack.spec.md       :: Out of scope, Deferred, Open questions
2026-05-03-graphics-stack.design.md     :: Open questions
2026-05-03-macro-codegen-improvements.spec.md      :: Out of scope, Deferred, Open questions
2026-05-03-macro-codegen-improvements.design.md    :: Open questions
2026-05-03-object-part-redesign.spec.md            :: Out of scope
2026-05-03-objectbase-debug-rename-factory.spec.md :: Out of scope, Deferred
2026-05-03-receiver-guard-auto.spec.md             :: Out of scope, Deferred
2026-05-03-signal-emit-checked.spec.md             :: Out of scope
2026-05-05-doc-convention.spec.md       :: Out of scope
2026-05-05-log-facade.spec.md           :: Out of scope, Deferred
2026-05-05-parent-children-accessors.spec.md       :: Out of scope, Deferred
2026-05-05-signal-emit-rename.spec.md   :: Out of scope
2026-05-05-thiserror-migration.spec.md  :: Out of scope
2026-05-05-timer-object.spec.md         :: Out of scope, Deferred
2026-05-05-tracing-itertools.spec.md    :: Out of scope, Deferred
2026-05-06-emit-macro.spec.md           :: Out of scope
2026-05-06-event-types-crate.spec.md    :: Out of scope, Deferred
2026-05-06-object-tree-query.spec.md    :: Out of scope, Deferred
2026-05-06-per-thread-event-loops.spec.md          :: Out of scope, Deferred
2026-05-06-signal-to-signal.spec.md     :: Out of scope
2026-05-06-tracing-spans.spec.md        :: Out of scope, Deferred
2026-05-07-cargo-doc-pages.spec.md      :: Out of scope, Deferred
2026-05-07-code-style-extraction.spec.md           :: Out of scope, Deferred
2026-05-07-code-style-extraction.design.md         :: Open questions
2026-05-07-codegen-inline-concrete-trait-impls.spec.md :: Out of scope
2026-05-07-codegen-simple-marker.spec.md           :: Out of scope
2026-05-07-codegen-simple-marker.design.md         :: Open questions
2026-05-07-coverage-ci.spec.md          :: Out of scope, Deferred
2026-05-07-criterion-benchmarks.spec.md            :: Out of scope, Deferred
2026-05-07-generic-fn-split.spec.md     :: Out of scope, Deferred
2026-05-07-generic-fn-split.design.md   :: Open questions
2026-05-07-macro-object-bench.spec.md   :: Out of scope
2026-05-07-multi-platform-ci.spec.md    :: Out of scope, Deferred
2026-05-07-multi-platform-ci.design.md  :: Open questions
2026-05-07-recursive-inline-annotations.spec.md    :: Out of scope
2026-05-08-ci-rust-cache-migration.spec.md         :: Out of scope
2026-05-08-ci-sccache.spec.md           :: Out of scope, Deferred, Open questions
2026-05-08-project-docs.spec.md         :: Out of scope, Deferred
2026-05-09-ci-skip-rust-matrix.spec.md  :: Out of scope, Deferred, Open questions
2026-05-09-ci-skip-rust-matrix.design.md           :: Open questions
2026-05-09-interview-spec-writer-subagent.spec.md  :: Out of scope, Deferred, Open questions
2026-05-09-paint-style.spec.md          :: Out of scope, Deferred, Open questions
2026-05-09-paint-style.design.md        :: Open questions
2026-05-10-docs-cleanup-197.spec.md     :: Out of scope
2026-05-10-gpu-snapshot-tests-ci.spec.md           :: Out of scope, Deferred, Open questions
2026-05-10-gpu-snapshot-tests-ci.design.md         :: Open questions
2026-05-10-next-deferred-discoverability.spec.md   :: Out of scope, Deferred, Open questions
2026-05-10-object-property-serialization-layer.spec.md   :: Out of scope, Deferred, Open questions
2026-05-10-object-property-serialization-layer.design.md :: Open questions
```

### A.3 — Skip log (file-level dedupe)

The 28 skipped sections, grouped by source file, with the thematic-file path where the file's harvest landed:

| Source file (skipped) | Sections skipped | Thematic file(s) where rows already live |
|------|------|------|
| `2026-05-01-auto-connection.spec.md` | Out of scope, Deferred, Open questions | `signals-slots.md` (Out of scope, Deferred, Open questions), `threading-runtime.md` (Deferred, Out of scope) |
| `2026-05-01-auto-connection.design.md` | Open questions | `signals-slots.md` (Deferred — 2 rows pointing at this design), `threading-runtime.md` (Deferred) |
| `2026-05-01-core-types.spec.md` | Out of scope, Deferred, Open questions | `signals-slots.md`, `properties.md`, `object-tree.md`, `threading-runtime.md`, `future-crates.md`, `python.md` (Out of scope/Deferred/Open questions distributed by topic) |
| `2026-05-01-github-workflow.spec.md` | Out of scope, Deferred | `ci-docs-workflow.md` (Out of scope, Deferred) |
| `2026-05-01-macros.spec.md` | Out of scope, Deferred, Open questions | `macros-codegen.md` (Out of scope, Deferred, Open questions), `properties.md` (Deferred), `python.md` (Out of scope) |
| `2026-05-01-runtime.spec.md` | Out of scope, Deferred, Open questions | `object-tree.md` (Open questions), `threading-runtime.md` (Deferred, Open questions), `future-crates.md` (Out of scope), `python.md` (Out of scope) |
| `2026-05-01-runtime.design.md` | Open questions | `object-tree.md` (Open questions — 1 row), `threading-runtime.md` (Open questions — 3 rows) |
| `2026-05-02-code-quality-cleanup.spec.md` | Out of scope | `ci-docs-workflow.md` (Out of scope — 2 rows) |
| `2026-05-02-docs-and-facade.spec.md` | Out of scope, Deferred | `ci-docs-workflow.md` (Out of scope — 5 rows, Deferred — 1 row) |
| `2026-05-02-examples-crate.spec.md` | Out of scope, Deferred | `future-crates.md` (Out of scope — 2 rows, Deferred — 1 row), `macros-codegen.md` (Out of scope, Deferred), `python.md` (Out of scope) |
| `2026-05-02-inline-simple-fns.spec.md` | Out of scope | `macros-codegen.md` (Out of scope — 2 rows) |
| `2026-05-02-lookup-perf.spec.md` | Out of scope, Deferred, Open questions | `object-tree.md` (Deferred, Out of scope, Open questions), `python.md` (Out of scope) |
| `2026-05-02-public-api-docs.spec.md` | Out of scope | `ci-docs-workflow.md` (Out of scope — 2 rows), `future-crates.md` (Out of scope) |
| `2026-05-02-signals-blocked.spec.md` | Out of scope, Deferred | `signals-slots.md` (Deferred, Out of scope), `python.md` (Out of scope) |

Total skipped: **28 sections** across **14 files**, matching the dedupe set `H`. The 14 files correspond to the prior manual extraction commits (`a8f23d5`, `0304e8e`, `8bd8c26`) per the spec's *Locked-in decisions* row 4.

**Sections marked `[SKIPPED — see A.3]` in the emit roster (A.2):** `runtime.design`, `auto-connection.design` — both designs are in `H` because at least one row pointing at each design lives in a thematic file (verified via `grep 'auto-connection.design' ai-docs/deferred/*.md | wc -l` = 2 rows in `signals-slots.md`; `grep 'runtime.design' ai-docs/deferred/*.md | wc -l` = 4 rows across `object-tree.md` and `threading-runtime.md`). The file-level dedupe rule therefore skips both designs' `Open questions` sections entirely, even though some open-question rows in these designs might be substantively new vs. what got harvested into the thematic files. *(Per the meta-plan rev 3 mandate, file-level dedupe is the chosen heuristic; rare false-positives are accepted to avoid the much-more-common false-negative of duplicating harvested rows. `/triage`'s drain step can re-introduce any missed row by hand if it surfaces in practice.)*

### A.4 — Notes on the audit method

The audit was run by a Python script that:

1. Iterated every `done/*.spec.md` and `done/*.design.md`.
2. Split each into `## `-headed sections using a regex anchor.
3. For each of the three target headings, classified the body using ordered rules matching rules 1–6 of *Parser specification*.
4. Built a list of `(file, kind, heading, shape, body)` tuples (333 entries).
5. Computed shape counts (`A.1`), emittable section counts after collapsing NONE/MISSING (`A.2`), and the dedupe outcomes against the thematic-file `Source`-cell path set (`A.3`).

The script is **not** committed (one-shot; not part of A2's deliverable). The parser the implementation will actually run is the skill-prompt logic added to `task/SKILL.md` per *Parser specification*; the script and the prompt produce identical outputs by construction — if the implementation phase observes divergence, that divergence is a prompt-logic bug to be fixed before backfill PR-merge.
