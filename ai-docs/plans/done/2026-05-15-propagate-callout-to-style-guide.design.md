# Design: Propagate compaction-recovery callout pattern to `ai-docs/agent-writing-style.md`

**Issue:** #358
**Date:** 2026-05-15

## Context

This is a documentation-only propagation task. PR #349 introduced a new fail-loud pattern — the top-of-file **"⚡ Compaction recovery check — read FIRST on every invocation"** callout — and applied it to six code-side SKILL.md files in three structural variants (A: glob-discovered probe, B: fixed-glob single artefact, C: parent-routing). It did NOT propagate the pattern to its style-guide home at `ai-docs/agent-writing-style.md`. This issue closes that gap.

**Why the pattern exists — load-bearing numeric facts** (verbatim from the spec's same-named Context subsection; reproduced here once so the implementer reads them without bouncing to the spec):

| Fact | Value | Consequence |
|---|---|---|
| Sonnet base-model context window | up to 1M tokens | Model can in principle hold a very long session. |
| Claude Code harness session cap when Sonnet is active | 200k tokens total = 180k input + 20k output | Harness imposes a tighter budget than the base model. |
| Auto chat-compaction trigger | input approaching the 180k input ceiling | This is the mechanism that emits the "Conversation compacted" marker the Compaction-recovery callout self-detects. |
| Opus-mode sessions | NOT auto-compacted in the same way | Opus-mode skills do not need a callout. |

These numbers are the motivation for the entire pattern: a Sonnet skill running past the 180k input ceiling is auto-compacted mid-flow, loses intermediate reasoning, and re-enters the skill without context — the callout is the mechanism that lets the skill detect the marker and recover from its durable-state artefact. The pattern is therefore a code-side (dual-model / Sonnet) concern only, and AC3's `## Out of scope` review pass is grounded in this fact (Opus-mode skills stay correctly enumerated as out of scope).

The Pattern 7 entry the implementer writes (Task 1) must surface the *motivation* (one sentence — "auto-compaction at the Sonnet 180k input ceiling can drop intermediate reasoning") and cite the spec's `## Context § Why the callout pattern exists` subsection as the load-bearing source. It must NOT inline the full 4-fact table — duplicating it across spec, design, and style guide creates a 3-way drift surface; the style guide stays the *shape* reference, the spec stays the *numeric facts* reference.

## Approach

Three coordinated edits in a single PR, plus an in-flow learning entry and a final verification pass:

1. **`ai-docs/agent-writing-style.md § Patterns`** — append a new `### 7. Compaction recovery callout` entry that documents the *shape* of each variant (when it fires, distinguishing surface phrase, skill assignments, cross-link anchor), inlines ONE trimmed Variant-A example, AND opens with a one-sentence motivation citing the spec's Context subsection. The entry does NOT duplicate the full Variant A/B/C bodies (those live in the six SKILL.md files and the archival design doc) and does NOT inline the 4-fact numeric table (that lives in the spec). Both would create drift surfaces.
2. **`AGENTS.md § Propagation Rule`** — (a) update the **Procedure:** step 1 grep recipe to add `ai-docs/agent-writing-style.md` to the scan paths; (b) mirror the same path in the AXIOM table's `AGENTS.md (rule add / exemption)` row so the table and the procedure agree; (c) add a new dedicated table row keying off `ai-docs/agent-writing-style.md` so the Propagation Rule fires explicitly the next time a new fail-loud pattern lands (resolves spec Open question 3).
3. **`ai-docs/agent-writing-style.md § Out of scope`** — review pass per AC3; expected outcome is "reviewed, no change" because the six skills carrying the new callout are dual-model code-side skills, not Opus-only readers. AC3's rationale now explicitly grounds in the Sonnet-180k-vs-Opus distinction from § Context.
4. **`ai-docs/learnings.md`** — append one new entry per AC6 documenting the propagation miss and the updated Propagation-Rule grep recipe. `Escalated? AGENTS.md, doc-convention` per the spec's *Key decisions* row (both files are edited in the same `/task` flow; `doc-convention` is the AGENTS.md-side label for the dual-model style guide).

**Open questions resolved (from spec § Open questions):**

- **Q1 (inline example snippet):** YES — inline ONE trimmed Variant-A snippet (~10–14 lines, marked "see SKILL.md for the full body"). Pattern 6 ("Concrete do/not examples") sets the precedent; a single representative variant gives readers an anchor without inflating the entry. Variant A is the natural choice (3 skills use it — the most numerous). Variants B and C get bullet-summarised with surface-phrase quotes.
- **Q2 (grep-recipe edit placement):** Inline edit at both the AGENTS.md *Propagation Rule* **Procedure:** step 1 sentence AND the AXIOM table's `AGENTS.md (rule add / exemption)` row. Single-line recipe; a new bullet would fragment the existing single-line command. Mirroring in two places prevents the table and the procedure from drifting.
- **Q3 (Propagation Rule table row for `ai-docs/agent-writing-style.md`):** YES — add a dedicated row. The existing "Any other instruction file → run the same grep" catch-all already covers the generic case, but a dedicated row makes the pattern-7-style propagation mechanical for the next contributor. The new row reads: "If you edit `ai-docs/agent-writing-style.md` (add / modify a numbered Pattern) — check every `.claude/skills/**` and `.claude/agents/**` file using that pattern's distinguishing surface phrase; propagate any wording / variant shift to all consumers." Adds ~ 220 chars; AGENTS.md stays at ~ 34,150 chars (under the 35,000 early-warning, comfortably under the 40,000 hard cap).

**Rejected alternatives:**

- **Inline all three variant bodies verbatim** — rejected by spec *Key decisions*. Creates a 4th drift surface; defeats the purpose of cross-linking to the archival doc + six live SKILL.md files.
- **Inline the 4-fact numeric table inside Pattern 7** — rejected. The numeric facts are the *spec's* load-bearing claim; replicating them in the style guide creates a 3-way drift surface (spec / design / style guide). The Pattern 7 motivation sentence cites the spec for the full table.
- **Add the citation footer to the 6 SKILL.md callouts** — rejected by spec *Out of scope* item 3 and AC5. Listed in *Deferred* for a future follow-up issue.
- **Generalise the three variants into a reusable include template** — rejected by spec *Deferred* (premature; wait until variant taxonomy stabilises through ≥1 more skill onboarding).
- **Add a new bullet to AGENTS.md Procedure step 1 for the new scan path** — rejected. The recipe is a single command line; splitting it into a bullet would obscure that step 1 IS a single command.
- **Skip the dedicated Propagation Rule row (rely on the catch-all)** — rejected because the catch-all is generic ("run the same grep"); a dedicated row names the *downstream consumers* of style-guide pattern changes (the SKILL.md callout bodies) and is therefore mechanically more actionable.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Append `### 7. Compaction recovery callout` entry to `## Patterns` (after Pattern 6, before `## Writing checklist`). The entry **MUST** include: (a) a one-sentence motivation citing the spec's `## Context § Why the callout pattern exists` subsection — "the callout exists because Sonnet-mode sessions auto-compact at the 180k input ceiling and risk losing intermediate reasoning mid-flow" — without inlining the full 4-fact numeric table; (b) Variant A / B / C named with when each fires (preamble-glob / fixed-glob / parent-routing); (c) the three distinguishing surface phrases verbatim; (d) per-variant skill assignments; (e) the cross-link `.claude/skills/context-reset/SKILL.md § Compaction recovery (re-entry)`; (f) a pointer to the archival design doc `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md`; (g) ONE trimmed Variant-A example snippet marked "see SKILL.md for the full body" | `ai-docs/agent-writing-style.md` | — |
| 2 | Review `## Out of scope` section per AC3; either no edit (expected) or a one-line clarification if any boundary line shifted; capture verdict for the PR body. Rationale grounded in the Sonnet-180k-vs-Opus distinction (§ Context): Opus-mode sessions are not auto-compacted, so Opus-mode skills stay correctly enumerated as out of scope and no shift is expected | `ai-docs/agent-writing-style.md` | 1 |
| 3 | Update `AGENTS.md § Propagation Rule` **Procedure:** step 1 grep recipe to include `ai-docs/agent-writing-style.md` (inline path append, no new bullet) | `AGENTS.md` | — |
| 4 | Mirror the same path in the AXIOM table's `AGENTS.md (rule add / exemption)` row so the table command matches Procedure step 1 | `AGENTS.md` | 3 |
| 5 | Add a new dedicated row to the AXIOM Propagation-Rule table for `ai-docs/agent-writing-style.md` (resolves spec Open Q3); placed **immediately before the `Any other instruction file` catch-all row** (the existing table rows are grouped by sync group, not alphabetised — the load-bearing constraint is "immediately before the catch-all", nothing else) | `AGENTS.md` | 4 |
| 6 | Append one new dated learning entry to `ai-docs/learnings.md` recording the propagation miss + the new grep recipe; `Escalated? AGENTS.md, doc-convention`; per AGENTS.md *Boundary rule 2 — `/task` Steps 8–12 exception* | `ai-docs/learnings.md` | 1, 3, 4, 5 |
| 7 | Verification pass: run AC4 size gates (`wc -c` ≤ 35,000 on both files), AC5 SKILL.md no-edit check, AC8 grep coverage; ensure PR body cites Pattern 7 (AC7) | (verification, no file edit) | 1, 2, 3, 4, 5, 6 |

**Atomicity note.** Tasks 1, 3, and 6 are independently committable (single-file, single-purpose). Tasks 4 and 5 are AGENTS.md follow-ons to Task 3 — they may share one commit with Task 3 if the agent prefers, but the design treats them as separate atomic units so a partial revert is straightforward. Task 7 produces no commits.

**Total: 7 atomic subtasks.** Within the design-agent rule limit (> 7 → propose splitting). Decomposition is unchanged from the round-1 design; the spec amendment only adds motivation context that is folded into § Context above and into Task 1's content requirements (single sentence + spec citation, no table duplication).

## Risks

- **Style-guide drift between Pattern 7 entry and live SKILL.md callouts.** Mitigation: the entry intentionally documents shape (surface phrases, anchor, assignments) rather than bodies; full bodies remain in the six SKILL.md files. The new dedicated Propagation Rule row (Task 5) names the downstream consumers so the next contributor sees the propagation contract.
- **Motivation-fact drift between spec, Pattern 7 entry, and any future re-derivation.** Mitigation: the 4-fact numeric table lives in the spec only; the style guide's Pattern 7 carries a one-sentence motivation + spec citation, not a duplicate table. Future updates to the numeric facts (e.g., harness window cap changes) update one place (the spec); the style-guide sentence stays accurate at the level of "auto-compaction at the input ceiling can drop intermediate reasoning".
- **AGENTS.md size cap.** Current 33,917 chars; the three AGENTS.md edits (Tasks 3, 4, 5) add an estimated ≤ 350 chars total, landing the file at ~ 34,267 chars — still under the 35,000-char early warning, well below the 40,000 hard cap. Mitigation: Task 7 verifies `wc -c AGENTS.md ≤ 35,000` after all edits.
- **`agent-writing-style.md` size cap.** Current 5,459 chars; Task 1 adds an estimated ~ 1,200 chars (heading + motivation sentence + Variants A/B/C summary + one inlined snippet + cross-link). Final size ~ 6,659 chars — ample headroom under both 35,000-char early warning and 40,000-char hard cap.
- **Boundary rule 2 violation risk.** The same conversation turn edits two instruction files (`AGENTS.md`, `ai-docs/agent-writing-style.md`) AND appends to `learnings.md`. Mitigation: spec *Key decisions* and *Technical constraints* explicitly invoke the `/task` Steps 8–12 in-flow exception. The learning entry's `Escalated?` field lists both files; entry content is an insight gained *during* this task (the propagation-gap surfacing), not a pre-emptive escalation. This is the canonical exception case.
- **Propagation Rule cascade.** Editing AGENTS.md triggers the Propagation Rule's own grep (Procedure step 1). Task 7's grep audit runs `grep -rn "Pattern 7\|compaction recovery callout" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md` (per AC8). Expected: 1 hit in `agent-writing-style.md` (the new heading), 1 hit in AGENTS.md (the new table row), 0 hits in `.claude/agents/` and `.claude/skills/` (since AC5 forbids SKILL.md callout-body edits). Any unexpected hit is a blocker.
- **AC7 PR-body meta-citation.** The style guide's `## Citation in PRs` section requires citing Pattern N when a PR adds a fail-loud section. This PR IS the introduction of Pattern 7, so the citation is meta — "Per `ai-docs/agent-writing-style.md` § Pattern 7, this PR introduces Pattern 7." Awkward but correct. Mitigation: the PR-body template in Task 7 carries the meta-citation verbatim.
- **`actionlint` not in scope.** No workflow file edited; AGENTS.md axiom does not fire. Mitigation: explicit Task 7 verification that `git status` shows no `.github/workflows/*.yml` changes.

## Test Design

**No code change.** This is an instruction-file PR; standard Rust gates (`cargo build`, `cargo test`, `cargo clippy --workspace -- -D warnings`, `cargo fmt -- --check`, `cargo doc`) run as no-ops on doc-only diffs. The "tests" are the AC verification gates:

- **AC1 verification** — `grep -n "^### 7\. Compaction recovery callout" ai-docs/agent-writing-style.md` returns exactly 1 hit. The entry's body is reviewed manually for required content: motivation sentence + spec citation, Variants A/B/C names, distinguishing phrases (verbatim), skill assignments, cross-link, archival-doc pointer, and ONE trimmed Variant-A snippet. The entry MUST NOT inline the full 4-fact numeric table.
- **AC2 verification** — `grep -n "ai-docs/agent-writing-style.md" AGENTS.md` returns ≥ 1 hit inside Procedure AND ≥ 1 hit inside the AXIOM table (existing `AGENTS.md (rule add / exemption)` row + the new Task-5 row).
- **AC3 verification** — manual review of `## Out of scope` (Task 2 captures the verdict in plain text for the PR body); expected: "reviewed, no change". The PR body's review note explicitly invokes the Sonnet-180k-vs-Opus rationale from § Context so a reviewer can verify the no-shift verdict is grounded.
- **AC4 verification** — `wc -c ai-docs/agent-writing-style.md AGENTS.md`; both ≤ 35,000 chars.
- **AC5 verification** — `git diff master -- .claude/skills/{task,code-review,pr-commented,bugfix,interview,context-reset}/SKILL.md` is empty.
- **AC6 verification** — `git diff master -- ai-docs/learnings.md` shows one new appended entry matching the AGENTS.md *Entry format*. Manual review confirms `**What happened:**`, `**Rule:**`, `**Escalated? AGENTS.md, doc-convention**`.
- **AC7 verification** — PR body (composed at Step 12 of `/task`) contains the meta-citation string `Per ai-docs/agent-writing-style.md § Pattern 7`. Manual visual check.
- **AC8 verification** — `grep -rn "Pattern 7\|compaction recovery callout" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md` returns the expected hit set (Pattern 7 heading in style guide; pattern references in AGENTS.md table row + Procedure; the existing `compaction recovery` mentions in `.claude/skills/context-reset/SKILL.md` are pre-existing and unrelated to Pattern 7 — the audit treats them as benign because they predate this PR). Additionally, the pre-existing `Compaction-recovery / re-entry fields` mention in `.claude/agents/review-findings.md` (already in the file before this PR; describes the extended `.progress.md` schema, not Pattern 7) is benign and treated with the same disposition as the `context-reset/SKILL.md` mentions — predates this PR, not a Pattern-7 reference, no action required.

**No `#[cfg(test)]` blocks involved.** No Rust test files modified.

## Open questions

None. All three spec-level open questions resolved in § Approach:

1. **Inline example snippet** — yes, one trimmed Variant-A snippet.
2. **AGENTS.md grep-recipe placement** — inline edit at Procedure step 1 + AXIOM table mirror.
3. **Propagation Rule table row for `ai-docs/agent-writing-style.md`** — yes, add a dedicated row placed immediately before the `Any other instruction file` catch-all (the table is grouped by sync group, not alphabetised).

The spec amendment (new `### Why the callout pattern exists (load-bearing numeric facts)` subsection) does not open new questions — it provides motivation that is folded into this design's § Context (full 4-fact table reproduced once for the implementer) and into Task 1's content requirements (one-sentence motivation + spec citation; no table duplication). Decomposition is unchanged.

If design-review surfaces a blocker on any of these, the design is iterated per the agent's *Iteration (feedback from review agent)* workflow.
