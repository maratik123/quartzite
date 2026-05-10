# `/task` Step 12 propagation + `_inbox.md` creation + one-shot deduped backfill

**Source:** issue #203
**Date:** 2026-05-10
**Tracked in:** #203

This is umbrella issue **A2** of the four-issue process-improvements meta-plan
([`ai-docs/plans/2026-05-10-process-improvements.md`](2026-05-10-process-improvements.md)).
A2 ships second in the strict sequence **A1 (#202) → A2 (#203) → B (#204) → C (#205)**.
A1 merged to `master` at commit `6701bd8` and landed the AGENTS.md governance for
`_inbox.md`; A2 now creates the file, wires `/task` Step 12 to populate it forward-going,
and runs a one-shot deduped backfill over the historical `done/` corpus.

The meta-plan's *Locked-in decisions*, *Risks and mitigations*, and *Issue A2* sections
are the source of truth for every design-affecting choice; this spec lifts them so the
design phase has a self-contained brief.

## Scope

1. **Create `ai-docs/deferred/_inbox.md`.** New file with a 4-column markdown table
   `| Item | Source | Section | Tracked |` and a header that explains the role and points
   at the AGENTS.md AXIOM (already landed in A1 — `AGENTS.md:182-189`) and at the
   `/triage` skill (which ships in B / #204; the reference is a deliberate forward note).
   - `Section` cell takes one of `out-of-scope` / `deferred` / `open-question` recording
     which spec heading the parser pulled the row from.
   - `Tracked` cell holds `—` initially; same semantics as the 8 thematic files (cell 4 = `Tracked` invariant).
   - **No `Status` column** — adding one would shift `Tracked` to cell 5 and break the
     cell-4 invariant across the 9 files (decision locked in meta-plan rev 3).

2. **Extend `.claude/skills/task/SKILL.md` Step 12** to parse the spec's *Out of scope* /
   *Deferred* / *Open questions* sections after moving spec/design to `done/` and append
   each parsed row to `ai-docs/deferred/_inbox.md`.
   - Parser shape contract is decided by the design-phase audit (see *Preconditions*).
   - The `Section` cell records which heading the row came from (`out-of-scope` /
     `deferred` / `open-question`).
   - Stage `_inbox.md` in the same Step 12 commit alongside the spec/design move and
     `INDEX.md` update.
   - **Unrecognised section shapes emit a warning, never block Step 12.** The warning
     identifies the spec path and section heading; the user can fix the source spec or
     extend the parser later.

3. **Update Step 12 Gate checklist** (the `## Gate checklist` table at the bottom of
   `task/SKILL.md`, the `Step 12` row) to include an `_inbox.md` check — the row must
   confirm the parser ran and the resulting append (or warning) was staged.

4. **One-shot deduped backfill skill flow.** A repeatable procedure, expressed as skill-prompt
   logic (no separate executable, no Rust binary, no shell script), that reuses Step 12's
   parser to walk every `done/*.spec.md` (and `done/*.design.md` if the audit confirms
   designs historically carry deferred sections — `signals-slots.md` already shows at
   least one design-sourced row, so the audit will likely confirm yes) and seeds
   `_inbox.md`. Run **once** at A2 land.
   - **Dedupe rule.** For each parsed row, check whether its `Source` link already
     appears as the `Source` cell of any row in any of the 8 thematic files
     (`ai-docs/deferred/{ci-docs-workflow,future-crates,macros-codegen,object-tree,
     properties,python,signals-slots,threading-runtime}.md`). If yes, **skip** the row —
     it was already harvested by the prior manual extraction commits (`a8f23d5`,
     `0304e8e`, `8bd8c26`, covering ~22 pre-2026-05-04 specs). If no, append to
     `_inbox.md`.
   - **Skipped rows are logged to the design doc's audit log**, citing the matching
     thematic-file row, so a reviewer can spot-check any skip decision.
   - The backfill walks specs in chronological order (oldest first) so `_inbox.md`'s
     row order is stable and reviewable.

5. **Synthetic spec fixtures.** Create `tests/fixtures/process-improvements/` with one
   committed `*.spec.md` per parser-relevant shape (≥ 6 shapes — see *Preconditions* and
   *Technical constraints*) and one per AC scenario (mangled section, all-three-section
   spec, `None.` sentinel, etc.). Fixtures are checked-in regular `*.spec.md` files,
   used by the ACs below as deterministic inputs at PR-merge time. They do **not**
   depend on a future merged spec to verify any AC.

## Out of scope

- **`/triage` skill itself.** Ships in Issue B (#204). A2's `_inbox.md` header references
  `/triage` as a forward note — that reference becomes live when B lands.
- **md ↔ gh issues bridge / drift detection.** Ships in Issue C (#205).
- **`/next` reading `_inbox.md`.** A1 already taught `/next` to read every row of every
  `ai-docs/deferred/*.md` via the existing `!`-block pattern; once A2 creates `_inbox.md`,
  `/next` picks it up automatically. **Whether an additional explicit `!`-block in
  `next/SKILL.md` is warranted is a design-phase decision** (locked-in: defer to design;
  there is no scope-level requirement to edit `next/SKILL.md` in this issue).
- **AGENTS.md AXIOM for `_inbox.md`.** Already landed in A1; A2 does not modify it.
- **AGENTS.md *Agent Docs* row for `_inbox.md`.** Already landed in A1. A2's PR drops
  the *"introduced in Issue A2"* suffix from that row as part of normal sync-group
  propagation (the suffix is a deliberate forward note that becomes stale once `_inbox.md`
  exists).
- **Reshaping `widget-backlog.md` schema.** Tracked refs continue to live in the
  existing `Notes` column. A2 does not touch `widget-backlog.md`.
- **Renaming the `Tracked` column to `Issue`.** Noted in the meta-plan as a future
  cosmetic improvement; out of scope here.
- **Visual surface for the maintainer.** Punted to the v1 quartzite UI-designer track
  per the meta-plan.
- **Pre-sorting backfilled rows into thematic files.** Locked in meta-plan: every row
  goes to `_inbox.md`; `/triage` (Issue B) does all classification. Uniform code path
  with forward-going Step 12.
- **Eval gate / hook escalation / `learnings.md`-style decision log** for the backfill
  flow. Backfill is a one-shot operation; none of these patterns apply.

## Deferred

- Drop the *"introduced in Issue A2"* suffix from `AGENTS.md` *Agent Docs* row | The
  suffix becomes stale once `_inbox.md` exists | no separate issue — folded into A2's
  PR as a one-line edit
- Whether `next/SKILL.md` needs a dedicated `!`-block read for `_inbox.md` | The existing
  pattern reads every `ai-docs/deferred/*.md` so technically already covered; design
  phase decides if an explicit `!`-block improves clarity | no separate issue — design-phase
  decision in A2 itself
- Future cosmetic rename `Tracked` → `Issue` column header across all 9 deferred files |
  The "untracked" decline-marker token clashes semantically with column name "Tracked" |
  yes — a separate cosmetic PR (open if the friction surfaces in `/triage` v1)

## Key decisions

| Question | Decision |
|---|---|
| Which sections does Step 12 (and the backfill) walk? | *Out of scope* + *Deferred* + *Open questions* — locked in meta-plan rev 2. |
| Where do parsed rows go? | Always `_inbox.md`. **No pre-sort** into thematic files; `/triage` (Issue B) does all classification. Uniform code path. |
| `_inbox.md` schema? | 4-column table `\| Item \| Source \| Section \| Tracked \|`. `Section` ∈ {`out-of-scope`, `deferred`, `open-question`}. `Tracked` initially `—`. **No `Status` column** — would break cell-4 invariant. |
| Are designs (`done/*.design.md`) walked too? | **Audit decides** during A2 design phase. Hint: `signals-slots.md` shows ≥ 1 design-sourced row in current data, so audit will likely confirm yes. |
| Backfill dedupe key? | The row's `Source` link string. If it matches the `Source` cell of any row in any of the 8 thematic files, the row is skipped (and the skip is logged to the audit). |
| Backfill scope (which specs)? | All `done/*.spec.md` (and `done/*.design.md` if audit confirms). At A2 PR-merge time the corpus may have grown beyond the 55 verified at meta-plan rev 3; the procedure walks whatever exists. |
| What does the parser do with an unrecognised section shape? | **Emit a warning, do not error.** Step 12 still completes; the warning identifies the spec + section so the source can be reformatted or the parser extended. |
| AC verification strategy? | **Synthetic spec fixtures** at `tests/fixtures/process-improvements/`. ≥ 1 fixture per shape + 1 per AC scenario. Deterministic at PR-merge time, no future-event dependency. |
| Backfill executable shape? | **Pure skill-prompt logic** + bash; reuses Step 12's parser. No separate executable, no Rust binary, no shell script. |
| Backfill order? | Chronological (oldest first) so `_inbox.md` row order is stable and reviewable. |
| Where does the `_inbox.md` header live, and what does it say? | Top-of-file header (above the table) explaining the role, citing the AGENTS.md AXIOM (already landed in A1), and pointing at `/triage` as the future drain mechanism. Style: matches the prose-headers used by the 8 thematic files. |

## Technical constraints

- **A1 already merged to `master`** (commit `6701bd8`). The AGENTS.md AXIOM at
  `AGENTS.md:182-189` and the *Agent Docs* row at `AGENTS.md:243` exist on `master`. A2
  must not duplicate, contradict, or reword the AXIOM — only adjust the *Agent Docs*
  row's suffix per *Out of scope* above.
- **Spec-shape audit is a Preconditions / design-phase requirement.** At least 6 distinct
  shapes have been observed in current `done/*.spec.md` files; per-section heterogeneity
  within a single spec (e.g. *Deferred* table + *Out of scope* bullets) is the norm:
  1. **Bulleted list** `- Item | Why` (most common; e.g. `done/2026-05-01-widgets.spec.md` *Deferred*).
  2. **Bulleted list with three columns** `- Item | Why | Separate issue?` (e.g. `done/2026-05-05-log-facade.spec.md`).
  3. **Bulleted list, no `|` separators** — plain prose bullets (e.g. `done/2026-05-09-paint-style.spec.md` *Out of scope* / *Open questions*).
  4. **3-column markdown table** with `| What | Why | Separate issue needed? |` headers (e.g. `done/2026-05-10-object-property-serialization-layer.spec.md` *Deferred*).
  5. **Literal `None.` / `(none — …)` sentinel** (e.g. `done/2026-05-02-inline-simple-fns.spec.md`, `done/2026-05-05-doc-convention.spec.md`).
  6. **Bulleted list with bolded-leading term + em-dash** `- **<Term>** — <explanation>` (e.g. `done/2026-05-10-object-property-serialization-layer.spec.md` *Out of scope*, `done/2026-05-10-docs-cleanup-197.spec.md`, `done/2026-05-08-project-docs.spec.md`, `done/2026-05-08-ci-sccache.spec.md`, `done/2026-05-07-code-style-extraction.spec.md`). Parser extracts the bolded term as `Item` and the post-em-dash text as the explanation.

  The audit MUST enumerate every shape it finds (so the parser-spec, or the canonical-shape
  lock-in, is exhaustive). Decision-gate output is one of (a) multi-shape parser spec
  with warn-and-skip fallback, or (b) canonical shape locked forward + per-spec skip
  decisions for non-conforming pre-canonical specs. The decision is recorded in the
  design doc.
- **Backfill dedupe key uses the `Source` link string** — the markdown link target (URL
  or relative path), normalised so trivial differences (`/`-trailing, anchor hashes) do
  not produce false positives. Design phase locks the exact normalisation.
- **`/task` Step 12 is prompt-driven**, not Rust code. The "parser" is skill-prompt
  logic that the task subagent runs against the just-finalised spec. Inputs: the spec
  file path. Outputs: zero or more rows appended to `_inbox.md` + zero or more warnings.
- **Stage `_inbox.md` in the Step 12 commit.** This is the existing Step 12 contract
  pattern (also stages `INDEX.md` and `ROADMAP.md`); A2 extends it with one more file.
- **Cell-4-`Tracked` invariant** is a hard structural constraint: `_inbox.md`'s 4th
  column header MUST be exactly `Tracked` so it parallels the 8 thematic files. This
  is what lets `/triage` (Issue B) iterate `Tracked`-column values uniformly across all 9 files.
- **Parser MUST anchor on column-header context, not bare substrings**, to avoid the
  `widget-backlog.md:89` prose-hit class of false positive (the `widget-backlog.md`
  parser issue does not directly affect A2 — A2 reads `done/*.spec.md`, not
  `widget-backlog.md` — but the same constraint applies to any future cell-iteration
  the design phase introduces).
- **Synthetic fixtures live at `tests/fixtures/process-improvements/`** (NOT under
  `tests/` proper, since A2 ships no Rust test code — the fixtures are inputs for
  manual / skill-driven verification per the AC table). The directory is workspace-root
  relative.
- **Working branch** `feat/2026-05-10-task-step12-inbox-backfill` is already created (per
  AGENTS.md AXIOM 1 — feature branch before any file edit).
- **Forward references in the existing AGENTS.md AXIOM** (the row about "row missing
  from `_inbox.md` for a freshly-merged spec → re-run Step 12 manually") become live
  once Step 12 actually appends — A2 does not need to edit the AXIOM, only fulfil it.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | **Parser-shape audit complete.** A2's design doc contains a section enumerating every spec shape encountered across all `done/*.spec.md` (and `done/*.design.md` if in scope) under the *Out of scope* / *Deferred* / *Open questions* headings. The doc explicitly chooses between (a) multi-shape parser with warn-and-skip fallback or (b) canonical shape locked forward + per-spec skip list, and lists per-spec skip decisions for the chosen path. Verification: read design doc; verify enumeration covers ≥ 6 shapes from *Technical constraints* above; verify decision-gate output is one of the two listed options. |
| AC2 | **Step 12 against an all-three-sections fixture appends three correctly-labelled rows.** Running `/task` Step 12 against `tests/fixtures/process-improvements/all-three-sections.spec.md` (a fixture with one *Out of scope* bullet, one *Deferred* table row, one *Open questions* bullet) appends exactly three rows to `ai-docs/deferred/_inbox.md`, with `Section` cells equal to `out-of-scope`, `deferred`, and `open-question` respectively, in that order. Verification: dry-run; `git diff ai-docs/deferred/_inbox.md` shows three new rows with the expected `Section` values; row text matches the fixture content. |
| AC3 | **Backfill emits-or-skips structurally per section.** Running the backfill at A2 land emits, for **every** non-empty *Out of scope* / *Deferred* / *Open questions* section of every in-scope spec, either ≥ 1 row in `_inbox.md` OR a skip-log entry in the design doc citing the pre-existing thematic-file row whose `Source` already covered it. Sections that are literally empty (`None.` / `(none — …)` sentinels) are not required to emit a row. Verification: design doc contains a per-section emit-or-skip table; reviewer can spot-check any section by reading its source spec and confirming the corresponding `_inbox.md` row or thematic-file `Source` link exists. |
| AC4 | **Step 12 emits a warning (does not error) on unrecognised section shape.** Running `/task` Step 12 against `tests/fixtures/process-improvements/mangled-section.spec.md` (a fixture with a deliberately mangled *Deferred* section) produces a visible warning identifying the spec path and section heading. Step 12 completes; no error exit; `_inbox.md` is unchanged for that section. Verification: dry-run against the fixture; observe warning in output; `git diff` shows no row added for the mangled section. |
| AC5 | **Step 12 Gate checklist row mentions `_inbox.md`.** The `Step 12` row of the `## Gate checklist` table in `.claude/skills/task/SKILL.md` includes a check that `_inbox.md` was parsed and appended (or warning logged). Verification: `grep -A 1 'Step 12.*Gate' task/SKILL.md` shows an `_inbox.md` reference in the checklist row. |
| AC6 | **`_inbox.md` exists with proper header and schema.** The file `ai-docs/deferred/_inbox.md` exists in the A2 PR, with (i) a top-of-file header explaining the role and citing the AGENTS.md AXIOM and `/triage`, (ii) a 4-column markdown table with header row `\| Item \| Source \| Section \| Tracked \|` and a separator row, (iii) populated by the backfill at A2 land. The AGENTS.md AXIOM (landed in A1) now references a real file. Verification: `ls ai-docs/deferred/_inbox.md`; `head -20 ai-docs/deferred/_inbox.md` shows the header + table headers; `grep _inbox.md AGENTS.md` shows the A1-landed rows unchanged (modulo the *"introduced in Issue A2"* suffix drop). |
| AC7 | **Backfill skips rows whose `Source` link already appears in a thematic file.** Running the backfill produces an `_inbox.md` that contains **no** rows for source-spec sections already harvested by the prior manual extraction commits (`a8f23d5`, `0304e8e`, `8bd8c26` — covering ~22 pre-2026-05-04 specs). The design doc's audit log enumerates the skipped rows with the matching thematic-file row. Verification: at A2 land, `grep -F '<Source>' ai-docs/deferred/_inbox.md` returns 0 matches for any `Source` link that appears in `ai-docs/deferred/{ci-docs-workflow,future-crates,macros-codegen,object-tree,properties,python,signals-slots,threading-runtime}.md`; design doc's skip log enumerates the skipped rows. |

## Open questions

- **None blocking design.** The meta-plan resolved every design-affecting question across
  three opus-subagent reviews; A1 has merged so the upstream governance is in place.
  The single residual ambiguity — whether `done/*.design.md` is in the backfill scope —
  is structurally a design-phase precondition (the audit answers it definitively from
  the corpus, not from speculation), not a spec-level open question. If implementation
  reveals a counter-example to any *Locked-in decision* row, the design phase can revisit
  via Design Amendment.
