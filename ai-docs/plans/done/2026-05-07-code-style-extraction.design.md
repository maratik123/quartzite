# Design: code-style-extraction

**Issue:** #124
**Spec:** [`2026-05-07-code-style-extraction.spec.md`](2026-05-07-code-style-extraction.spec.md)
**Date:** 2026-05-07

## Approach

Pure documentation refactor: extract the AGENTS.md `## Code Style` body (lines
52–141, 90 lines) into a new `ai-docs/code-style.md` reference file shaped
after `ai-docs/doc-convention.md`. AGENTS.md keeps a short bulleted index
(~10 lines, one per cluster) that links to the cluster anchors in
`code-style.md`. Propagate references across `.claude/agents/`,
`.claude/skills/`, `ai-docs/learnings.md`, `ai-docs/plans/` per the spec
grep. Update the escalation taxonomy in AGENTS.md and `self-improve.md` to
recognise `code-style` as a new escalation target alongside the existing
`doc-convention`.

**Why this shape over alternatives.**

- *Leaner one-`##`-per-rule layout (no `## Scope` or meta-sections).* Rejected
  by the spec's "Key decisions" table — mirroring `doc-convention.md`'s shape
  preserves cross-file navigation muscle memory and `## Scope` /
  `## Lints …` / `## Behavioural enforcement` are genuinely useful for the
  Code Style rule set (the rule set has both clippy-backed and
  reviewer-judgement rules, and clarifying which is which is the explicit
  point of the meta-sections).
- *Dense one-paragraph "Mandatory rules in brief" form in AGENTS.md (the
  pattern used for the doc-convention pointer at AGENTS.md:60).* Rejected by
  the spec — the rule set is 90 lines across ~10 logical clusters; one
  paragraph cannot index it readably. Bulleted index gives the reader an
  O(1) jump to the cluster they need.
- *Keep AGENTS.md `## Code Style` intact and add `code-style.md` as a duplicate
  expanded reference.* Rejected — duplication invites drift; the
  Propagation Rule already exists because the workspace has been bitten by
  this. The rule body must live in exactly one place.
- *Migrate `## Dependency Versions` into `code-style.md` too.* Rejected by
  spec — it covers Cargo.toml mechanics, not source style; folding it in
  would dilute the cluster taxonomy.

## Anchor and link strategy

GitHub / rustdoc Markdown auto-anchors lowercase the heading, replace
spaces with `-`, and strip most punctuation. The 10 cluster headings
chosen below produce the following anchors (verified by the GitHub
slugifier rules, which are what `gh` and our agents see; `rg`-grepable
verbatim):

| `##` heading text | Anchor (auto-generated) |
|---|---|
| `## Scope` | `#scope` |
| `## Source files` | `#source-files` |
| `## Linter posture` | `#linter-posture` |
| `## Rust idioms` | `#rust-idioms` |
| `## Library safety idioms` | `#library-safety-idioms` |
| `## Documentation` | `#documentation` |
| `## Error types` | `#error-types` |
| `## Tracing` | `#tracing` |
| `` ## `#[inline]` and the `_Simple._` doc tag `` | `#inline-and-the-_simple_-doc-tag` |
| `## Generic-fn split for binary size` | `#generic-fn-split-for-binary-size` |
| `## File size` | `#file-size` |
| `## Lints that mechanically enforce parts of this convention` | `#lints-that-mechanically-enforce-parts-of-this-convention` |
| `## Behavioural enforcement (what lints cannot check)` | `#behavioural-enforcement-what-lints-cannot-check` |

**Tricky case — the `#[inline]` heading.** Backticks are stripped from the
slug, and `[`, `]`, `.` are also stripped. `#[inline]` and `_Simple._` reduce
to `inline` and `_simple_`. Underscores **are preserved** by GitHub's
slugifier. The heading text contains `` ` `` `#[inline]` `` ` `` `and the` ``
`` ` `` `_Simple._` `` ` `` `doc tag`, which slugifies to
`inline-and-the-_simple_-doc-tag`. We will write the bulleted-index link in
AGENTS.md as:

```markdown
- `#[inline]` and the `_Simple._` doc tag — [code-style.md#inline-and-the-_simple_-doc-tag](ai-docs/code-style.md#inline-and-the-_simple_-doc-tag)
```

**Verification step (Subtask 7).** Use `gh api repos/<O>/<R>/readme --jq
'.html'` is overkill; instead, after the doc lands on the branch, render the
file via GitHub's web view of the pushed branch and click each anchor
once, **or** rely on the simpler local check: `rg '^## ' ai-docs/code-style.md`
prints all heading texts; pipe through a slugifier in the head (lowercase,
replace ` ` → `-`, drop ``[]`.``) and `rg` each computed anchor against the
AGENTS.md bullet links. A wrong anchor is a silent broken link in the
rendered Markdown — must verify before merging.

## Verbatim-content guarantee (AC2)

To audit "every Code Style rule has its body present verbatim", the design
ships an explicit **line-range citation map** the implementer follows
without freelancing prose. Each AGENTS.md line range maps to exactly one
`code-style.md` cluster; the implementer copies the source lines, reflows
**only the header** (turn the leading `- **Title.**` bullet into `## Title`
with a body underneath), and otherwise pastes the rule body verbatim. After
extraction, the auditor diffs source-vs-target with whitespace tolerance:

```bash
# After Subtask 1 lands, run this to verify cluster N's prose round-trips.
# Strip leading '- ' / '## ' header marks; preserve everything else.
diff <(awk 'NR>=L1 && NR<=L2' AGENTS.md.original | sed 's/^- \*\*[^.]*\.\*\*//' | sed 's/^  //') \
     <(awk '/^## CLUSTER_NAME/,/^## /' ai-docs/code-style.md | tail -n +2 | head -n -1)
```

Mechanical check; no manual line-by-line diffing needed.

### Citation map (AGENTS.md → code-style.md cluster)

| Cluster `##` (in `code-style.md`) | AGENTS.md lines (current `master`) | Rule bullets |
|---|---|---|
| 1. Source files | 54, 55, 58 (the second sentence — "Always format via `cargo fmt`, never `rustfmt <file>` directly.") | `.rs`-only, max line 100, `cargo fmt` over `rustfmt <file>` |
| 2. Linter posture | 56 + the Rust Test Conventions line "No `#[allow(clippy::...)]` / `#[allow(dead_code)]` unless unavoidable." (lives in AGENTS.md `## Rust Test Conventions`, not Code Style — **NOT moved**, see "Out-of-cluster note" below) | strict clippy (line 56). The `#[allow]` policy stays in `## Rust Test Conventions` (out of scope per spec). |
| 3. Rust idioms | 57, 58 (first sentence — "Let chains … are valid in this codebase (edition 2024). Do not avoid them."), 126 (`Comparison helpers`) | Rust idioms over foreign ports + no GUI/UI-framework citations (line 57); let chains (line 58 first sentence); `.min`/`.max`/`.clamp`/`Option::or`/`Option::filter` (line 126) |
| 4. Library safety idioms | 127–129 | mutex `lock().ok()?` / `.unwrap_or_else`; safe primitives over raw pointers |
| 5. Documentation | 59, 60 | `#![deny(missing_docs)]` + `#![warn(clippy::undocumented_unsafe_blocks)]` per crate; one-line `///`; `# Examples` rule; pointer to `doc-convention.md` |
| 6. Error types | 130 | `thiserror` for new error types |
| 7. Tracing | 131–135 | `*_span!` guards; `debug_span!` vs `trace_span!`; `verbose-tracing` gate; sibling-consistency |
| 8. `#[inline]` and the `_Simple._` doc tag | 61–100 (the entire `#[inline]` block including the recursive definition table, marker-form decision tree, carve-out, codegen mirroring, marker maintenance) | full block |
| 9. Generic-fn split for binary size | 101–125 (entire block, including the `If the inner ends up simple, unwrap it.` paragraph at 125) | full block |
| 10. File size | 136–141 | 200–400 line target; soft 500/800; hard 1000/1500; exemptions; counter-rule; `too_many_lines` per-fn |

**Out-of-cluster note: line 58 splits.** AGENTS.md line 58 currently reads
"Let chains (...) are valid in this codebase (edition 2024). Do not avoid
them. Always format via `cargo fmt`, never `rustfmt <file>` directly." —
two distinct rules in one bullet. The citation map splits it: the
let-chains sentence goes to cluster 3 (Rust idioms), the `cargo fmt`
sentence to cluster 1 (Source files). This is the only structural seam
where source-line-1 maps to two clusters; the implementer must make the
split deliberately and the auditor diff above will catch any
prose-loss because both halves of the original sentence appear in the
target. **No rule wording change** — only re-bulleting.

**`#[allow(...)]` policy clarification.** The spec's cluster 2 description
mentions `#[allow]` policy, but the canonical wording for that rule
(`No #[allow(clippy::...)] / #[allow(dead_code)] unless unavoidable.`)
currently lives in `## Rust Test Conventions` at AGENTS.md:250, not in
`## Code Style`. The spec is explicit ("Body content is the verbatim rule
text from AGENTS.md's existing Code Style section — **no rule changes**").
**Decision: do not move it.** The `## Rust Test Conventions` section is out
of scope. The cluster 2 body in `code-style.md` will document only the
`-D warnings` rule (AGENTS.md:56). A short pointer ("Reviewer-judgement
allow-policy lives in `## Rust Test Conventions`") may be added if it
helps discoverability — flagged as a **nit** decision for the implementer.

## Order of edits (avoiding half-extracted state)

The PR opens new doc, propagates references that point at the new doc,
then trims AGENTS.md last — at no commit boundary should a reference
hang in the air pointing at a not-yet-existing anchor:

1. **Create** `ai-docs/code-style.md` with full body, all 10 clusters, the
   `## Lints` / `## Behavioural enforcement` meta-sections. AGENTS.md still
   carries the full original `## Code Style` section (lines 52–141)
   unchanged. Both files temporarily duplicate the rule prose — this is
   intentional.
2. **Verify** all relative links from `ai-docs/code-style.md` resolve
   (`../AGENTS.md`, `./doc-convention.md`); render and click through
   anchors locally.
3. **Update** AGENTS.md `Agent Docs` table (line ~209) to add the
   `ai-docs/code-style.md` row.
4. **Propagate** references across `.claude/agents/{review-findings,
   self-review}.md` and `ai-docs/plans/INDEX.md` line 89: every
   "AGENTS.md Code Style → X" citation gains the
   `ai-docs/code-style.md#anchor-X` qualifier (preserves rule-level
   specificity, see Risk #1). At this point the agents already point at
   `code-style.md` and AGENTS.md still has the full body — but agents and
   the trimmed AGENTS.md must agree, so:
5. **Trim** AGENTS.md `## Code Style` section (lines 52–141) to the
   bulleted index. Now AGENTS.md has only the index, agents point at
   `code-style.md`, and `code-style.md` is the canonical source.
6. **Update** the escalation taxonomy in AGENTS.md `## Corrections Log`
   and `.claude/agents/self-improve.md` (frontmatter + "Escalated" status
   list) per spec scope #6.
7. **Update** `ai-docs/plans/INDEX.md` Active plans table to add this
   spec as a row. (Step 12 from spec moves it to ✅ on completion — that's
   a separate post-merge edit, not part of this PR.)
8. **Sanity-check** `cargo build`, `cargo clippy --all-targets -- -D
   warnings`, `cargo fmt -- --check`, `cargo test`, `cargo build -p
   quartzite --no-default-features`, `RUSTDOCFLAGS="-D warnings -D
   missing-docs" cargo doc --no-deps --workspace`. Doc-only PR, so all
   should be no-ops; any failure indicates a rendering or link-resolution
   regression elsewhere.

The PR can be split into the commits listed in *Decomposition* below. The
key invariant: **the AGENTS.md trim (Subtask 5) is the LAST mutating step
before sanity checks.** If we trimmed AGENTS.md before propagating the
agents, the agents would briefly cite the AGENTS.md section header that
no longer carries the rule body — a stale citation. Order matters.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create branch `feat/2026-05-07-code-style-extraction`. Verify `git branch --show-current` returns the new branch before any Edit/Write. | (no files) | — |
| 2 | Create `ai-docs/code-style.md`: `## Scope` (mirrors `doc-convention.md` Scope shape — what the rules apply to), 10 cluster `##` headings with verbatim rule prose per the citation map, `## Lints that mechanically enforce parts of this convention`, `## Behavioural enforcement (what lints cannot check)`. No `## References`. | `ai-docs/code-style.md` (new) | 1 |
| 3 | Trace relative links from `ai-docs/code-style.md`: `../AGENTS.md` should resolve to repo-root `AGENTS.md`; `./doc-convention.md` should resolve to `ai-docs/doc-convention.md`. Verify by `realpath ai-docs/code-style.md/../../AGENTS.md` and `realpath ai-docs/code-style.md/../doc-convention.md`. Render `code-style.md` locally (or via GitHub web on the branch) and click each `## Lints` / `## Behavioural enforcement` anchor link. | (no edits, verification only) | 2 |
| 4 | Add `ai-docs/code-style.md` row to AGENTS.md `Agent Docs` table (line ~209). Purpose column: "Workspace code-style reference — read on demand". | `AGENTS.md` | 2 |
| 5 | Propagate references in `.claude/agents/`. In `review-findings.md` and `self-review.md`, replace each "see `AGENTS.md` Code Style → \<rule\>" with "see [`ai-docs/code-style.md` → \<rule\>](../../ai-docs/code-style.md#\<anchor\>)" preserving the rule-level specificity (no degradation to file-level pointer). For each match in INDEX.md (line 89) and any other in-scope hit, do the same. | `.claude/agents/review-findings.md`, `.claude/agents/self-review.md`, `ai-docs/plans/INDEX.md` | 2, 4 |
| 6 | Trim AGENTS.md `## Code Style` (lines 52–141) to the bulleted index: ~10 lines, one per cluster, each summarising the rule + linking the cluster anchor in `ai-docs/code-style.md`; closing line `see [\`ai-docs/code-style.md\`](ai-docs/code-style.md) for the canonical reference.` | `AGENTS.md` | 5 |
| 7 | Update escalation taxonomy: AGENTS.md `## Corrections Log` `Escalated?` enum (line ~231) — add `code-style` to the comma-separated list, add the `code-style` explanatory note alongside the existing `doc-convention` note. `.claude/agents/self-improve.md` frontmatter `description:` (line 3) — extend the list of escalation targets to include both `ai-docs/code-style.md` and `ai-docs/doc-convention.md`. `.claude/agents/self-improve.md` "Escalated" status list (line ~28) — add `code-style` AND `doc-convention` to the recognised set. | `AGENTS.md`, `.claude/agents/self-improve.md` | 6 |
| 8 | Add row to `ai-docs/plans/INDEX.md` Active plans table for this spec, status 🟢 ready (spec+design exist, no blockers). | `ai-docs/plans/INDEX.md` | 1 |
| 9 | Sanity-check: `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`, `cargo test`, `cargo build -p quartzite --no-default-features`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`. Re-run `rg 'AGENTS\.md.*Code Style' .claude/agents/ .claude/skills/ ai-docs/learnings.md ai-docs/plans/` and confirm the only remaining hits are this spec/design itself plus historical references in `ai-docs/plans/done/` (out of scope per AC5 — they cite the section header in commits/specs already merged). | (no edits, verification only) | 7, 8 |

Subtasks per-commit grouping suggestion:

- **Commit 1 (Subtask 1+2+3):** "docs: extract Code Style into `ai-docs/code-style.md`" — adds new file, no other edits. Bisectable: at this commit AGENTS.md is unchanged, agents still cite the AGENTS.md section, both files have the rule prose (deliberate temporary duplication).
- **Commit 2 (Subtask 4+5):** "docs: register code-style.md in Agent Docs and propagate agent citations" — table row + agent file edits.
- **Commit 3 (Subtask 6):** "docs: replace AGENTS.md Code Style body with bulleted index" — the trim.
- **Commit 4 (Subtask 7+8):** "docs: extend escalation taxonomy with code-style; index this plan" — taxonomy edits + INDEX.md.

Subtask 9 is the sanity gate before opening the PR; no commit. Total: 9
subtasks, 4 commits. Within the spec's "≥7 subtasks → consider splitting"
guideline, but each is ≤ 1 file's worth of edit and the order is
non-negotiable, so no split is warranted.

## Risks

- **Citation specificity loss when propagating references.** A naive
  propagation (`s/AGENTS.md Code Style/ai-docs\/code-style.md/`) drops
  rule-level specificity that `review-findings.md` and `self-review.md`
  rely on (e.g., `→ "Generic-fn split for binary size", "If the inner
  ends up simple, unwrap it" paragraph`). **Mitigation:** Subtask 5
  rewrites each citation to point at the **cluster anchor** in
  `code-style.md` so reviewer can still jump straight to the rule. The
  citation grammar becomes "see [`ai-docs/code-style.md` →
  `\<heading\>`](path#\<anchor\>)" — preserves the textual rule name
  *and* gains a clickable anchor. Verified by manual review of the
  diff against the original 6 hits in agents files (review-findings.md
  lines 64, 66, 68, 69, 71; self-review.md lines 74, 76, 78, 79, 81).

- **Anchor mismatch in AGENTS.md bulleted index.** GitHub auto-anchor
  rules differ slightly from rustdoc. The bulleted index uses anchor
  links; if the slugifier strips a character we predicted it would
  preserve, the link 404s silently. **Mitigation:** Subtask 3 includes a
  manual click-through; the special-case `#inline-and-the-_simple_-doc-tag`
  anchor is documented in the *Anchor and link strategy* section above,
  and a `rg '^## ' ai-docs/code-style.md` audit confirms the heading
  texts match what the bullets link to.

- **Out-of-spec hits in `ai-docs/deferred/macros-codegen.md`.** Subtask
  9's grep is scoped to the spec's directories
  (`.claude/agents/`, `.claude/skills/`, `ai-docs/learnings.md`,
  `ai-docs/plans/`). Two hits exist in `ai-docs/deferred/macros-codegen.md`
  (lines 19, 20) referencing "AGENTS.md Code Style → `#[inline]` and the
  `_Simple._` doc tag". **Decision: leave them.** The spec scope #5
  enumerates the directories — `ai-docs/deferred/` is intentionally not
  in the set, presumably because deferred items are revisited when
  unblocked and the citation update can land then. Flag as a follow-up
  in the PR body so the reviewer can opt in if desired (zero cost — same
  rewrite pattern as Subtask 5). **Add to "Open questions" below.**

- **Historical references in `ai-docs/plans/done/`.** AC5 says
  *"`rg ...` returns no hits, OR each remaining hit is intentionally
  pointing at the now-trimmed AGENTS.md section header (not at a rule
  body that moved)"*. The `done/` plan files are read-only history (they
  document what was done at the time of merge). They cite "AGENTS.md
  Code Style" as the rule's location *as of that PR's merge*. Rewriting
  them is not appropriate (would falsify the historical record).
  **Mitigation:** The PR body and Subtask 9 explicitly cite AC5's
  "intentionally pointing at the now-trimmed section header" branch
  for these — the historical citations now resolve to the bulleted
  index, which immediately links forward to `code-style.md`, so the
  navigation still works.

- **Doc-gate regression.** The doc gate `RUSTDOCFLAGS="-D warnings -D
  missing-docs" cargo doc --no-deps --workspace` is a sanity check.
  Doc-only PR, no `.rs` source touched, so it should pass. **If it
  fails:** investigate immediately — likely an unrelated drift caught
  by the gate run, not caused by this PR. **Do not** waive or skip the
  gate.

- **AGENTS.md self-reference loop.** The AGENTS.md bulleted index links
  to `ai-docs/code-style.md`; `code-style.md` links back to
  `../AGENTS.md` for the master rule list. No loop in *content* (the
  link is a navigation link, not a definition reference), but the
  reviewer will see two files cross-linking. **Mitigation:** clear
  framing — `code-style.md` opens with "This is the canonical reference
  for the workspace's Rust source-style rules. AGENTS.md carries the
  bulleted index summarising each cluster." Same pattern as
  `doc-convention.md` ↔ AGENTS.md today — no new shape.

## Test Design

This is a documentation-only PR. No `cargo test` cases are added. The
testing strategy is a sequence of mechanical checks per AC, executed
during Subtask 9.

### AC1 — Structural shape of `ai-docs/code-style.md`

- **Check:** `rg '^## ' ai-docs/code-style.md` lists the headings.
- **Expected:** `Scope`, then the 10 cluster `##` headings in spec order,
  then `Lints that mechanically enforce parts of this convention`, then
  `Behavioural enforcement (what lints cannot check)`. No `References`.
- **Failure:** missing or out-of-order heading.

### AC2 — Verbatim rule prose preserved

- **Check:** for each cluster N, run the citation-map diff documented in
  *Verbatim-content guarantee* above against AGENTS.md @ HEAD~N (the
  pre-trim AGENTS.md captured by `git show <commit-1>:AGENTS.md`).
- **Expected:** empty diff modulo header reformatting (`- **Title.**` →
  `## Title`) and intra-cluster blank-line normalisation.
- **Tooling:** the `awk` and `diff` snippet in the *Verbatim-content
  guarantee* section is the canonical implementation.

### AC3 — AGENTS.md bulleted index shape

- **Check:** `awk '/^## Code Style/,/^## /' AGENTS.md | head -n -1` extracts
  the section. Count bullets (`grep -c '^- '`).
- **Expected:** ~10 bullets, each one sentence + link, ending with the
  literal `see [\`ai-docs/code-style.md\`](ai-docs/code-style.md) for the
  canonical reference.` line.
- **Failure:** a bullet exceeds one sentence (the section regressed to
  inline rule prose); link target wrong.

### AC4 — Agent Docs table row

- **Check:** `rg '^\| \`ai-docs/code-style.md\`' AGENTS.md`.
- **Expected:** one row, purpose `Workspace code-style reference — read
  on demand`.

### AC5 — Propagation grep

- **Check:** `rg 'AGENTS\.md.*Code Style' .claude/agents/ .claude/skills/
  ai-docs/learnings.md ai-docs/plans/`.
- **Expected:** zero hits in `.claude/agents/`, `.claude/skills/`,
  `ai-docs/learnings.md`. Hits in `ai-docs/plans/done/` are accepted
  (historical record, AC5's "intentionally pointing at the now-trimmed
  section" branch). Hit in this spec/design file is accepted (it's the
  spec for this very rewrite). Hit in `ai-docs/plans/INDEX.md` line 89
  must be rewritten to point at `code-style.md`.

### AC6 — Markdown link integrity

- **Check (relative links from `code-style.md`):**
  ```bash
  realpath ai-docs/code-style.md/../../AGENTS.md            # → /…/AGENTS.md
  realpath ai-docs/code-style.md/../doc-convention.md       # → /…/ai-docs/doc-convention.md
  ```
  Both must resolve to existing files.
- **Check (anchor links from AGENTS.md to `code-style.md`):** `rg
  '\(ai-docs/code-style.md#' AGENTS.md` lists each link; for each anchor,
  `rg '^##.*\<computed-text\>' ai-docs/code-style.md` confirms a heading
  produces that anchor under GitHub's slugifier. The `#inline-and-the-_simple_-doc-tag` anchor is the only one with non-trivial slugification — verified once by hand.

### AC7 — Cargo sanity

- `cargo build`, `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt -- --check`, `cargo test`. All clean.

### AC8 — `--no-default-features` build

- `cargo build -p quartzite --no-default-features`. Clean.

### AC9 — Doc gate

- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps
  --workspace`. Clean. (Should be unaffected — no `.rs` change. If it
  fails, investigate; do not waive.)

### AC10 — `## Lints` and `## Behavioural enforcement` enumerations

- **`## Lints` enumeration must include at minimum:** `clippy::too_many_lines`
  (cluster 10), `clippy::undocumented_unsafe_blocks` (cluster 5),
  `clippy::missing_errors_doc` / `clippy::missing_panics_doc` /
  `clippy::missing_safety_doc` / `clippy::doc_markdown` (cluster 5
  cross-references; primary owner is `doc-convention.md`),
  `#![deny(missing_docs)]` (cluster 5), `-D warnings` (cluster 2).
  Each entry points at its owning `## …` section above.
- **`## Behavioural enforcement` enumeration must include:** marker-maintenance
  cascade (cluster 8), `_Simple._` decision tree (cluster 8), unwrap-simple-`inner`
  (cluster 9), no-foreign-framework citation (cluster 3), let-chain-encouragement
  (cluster 3), file-size soft-limit responsibility split (cluster 10),
  span-significance threshold (cluster 7), `thiserror`-vs-hand-rolled
  judgement (cluster 6), mutex `lock().unwrap()` justification (cluster 4).
  Each entry is a one-line pointer back into the cluster, not a
  re-statement of the rule.
- **Check:** read the two meta-sections; confirm the lists are
  cross-references (one bullet per item, each with a backlink to the
  owning `##` heading), not duplicate prose.

### AC11 — INDEX.md row

- **Check:** `rg 'code-style-extraction' ai-docs/plans/INDEX.md` returns
  one row in Active plans during this PR. Post-merge (Step 12, separate
  edit), the row moves to ✅ Completed plans.

### AC12 — Escalation taxonomy

- **AGENTS.md:** `rg 'code-style' AGENTS.md` finds the entry in the
  `Escalated?` enum (line ~231) and the explanatory note. The note
  follows the same format as the existing `doc-convention` note.
- **`self-improve.md` frontmatter (line 3):** the `description:` field
  mentions both `ai-docs/code-style.md` and `ai-docs/doc-convention.md`
  in its list of escalation targets the agent recognises.
- **`self-improve.md` "Escalated" status list (~line 28):** the comma-
  separated set in step 1 of *Find patterns* includes both `code-style`
  and `doc-convention`.

### Manual link click-through (AC6 backstop)

After all subtasks land but before opening the PR, render the doc tree
on the feature branch via `gh pr view` (once the PR is created) or
locally via a Markdown previewer. Click every link added by this PR at
least once. The slugifier check above is mechanical, but a click
confirms the rendered anchor actually scrolls to the right heading.

## Open questions

- **`ai-docs/deferred/macros-codegen.md` propagation.** Spec scope #5
  excludes `ai-docs/deferred/` from the propagation grep. Two hits exist
  there (lines 19, 20). Should those be rewritten in the same PR (zero
  cost, same pattern), or left for the deferred-item rework when the
  blocked work resumes? **Default:** leave them per spec; mention as a
  follow-up in the PR body. **Awaiting:** product-owner / architect
  preference. Resolvable inside the PR review.

- **Cluster 2 (`Linter posture`) — include the `#[allow(...)]` policy or
  not?** The spec table assigns `#[allow]` policy to cluster 2, but the
  canonical wording lives in AGENTS.md `## Rust Test Conventions`
  (line 250), not `## Code Style`. Spec is "no rule changes". **Default
  decision:** do not move the rule. Cluster 2 documents only `-D
  warnings`. Add a one-line cross-reference pointer to `## Rust Test
  Conventions` for discoverability. **Awaiting:** confirmation that
  this reading of the spec is correct. If the spec author intended the
  `#[allow]` rule to *also* live in `code-style.md`, that's a spec
  amendment and counts as a "rule change" — out of scope per the spec.
