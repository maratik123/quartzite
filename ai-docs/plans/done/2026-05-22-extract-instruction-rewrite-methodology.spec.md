# Extract instruction-rewrite methodology into a permanent reference doc

**Source:** issue #531
**Date:** 2026-05-22
**Tracked in:** #531

## Scope

1. Create a NEW permanent reference doc at `ai-docs/instruction-file-validation.md` (name resolved at Round 1 — see Key decisions) that owns the durable dual-model instruction-file-clarity testing methodology, bias taxonomy, subagent prompt templates, and historical-replay protocol previously embedded in `ai-docs/plans/2026-05-08-instruction-file-rewrite.md`.
2. Move the seven methodology sections from the plan file into the new reference doc, verbatim where possible, structurally complete:
   | Source section | Source lines | Destination § in new doc |
   |---|---|---|
   | `## Probe composition` (Classes A/B/C/D + open-ended + ordering + totals) | 241–372 | Probe taxonomy |
   | `## Probe-surface coverage rules` (60% / ≤40% / heavy-section override + worked examples + coverage-map deliverable) | 374–461 | Coverage rules |
   | `## Rubric-based evaluation` (answer key shape, convergence table, reporting format, edge cases, what is NOT used) | 463–565 | Rubric framework |
   | `## Approval gates` + `## Iteration cap = 3` | 567–595 | Workflow gates |
   | `## Implementation hints` (branch naming, section inventory, link audit, probe heuristics, rubric heuristics, subagent spawn / prompt templates, dual-model report format, honest closing-comment template, decoupled probe-author flow) | 633–974 | Templates |
   | `## Methodology limitations` (8-bias taxonomy, honest-framing rule, Phase 2 future improvements, Phase 1 retrospective summary) | 976–1057 | Known biases & limitations |
   | `## Historical replay testing` (when-to-run, worktree protocol, subagent prompt, ground-truth comparison, cleanup, lessons, methodology-fit limitations, cost vs synthetic) | 1059–1244 | Historical replay testing |
3. Edit-in-place `ai-docs/plans/2026-05-08-instruction-file-rewrite.md` — strip the seven sections moved out; rewrite `## Citation in future PRs` (lines 616–631) to point future readers at the new reference doc for methodology and at the archived plan for historical decision context only. Internal self-references inside the plan (currently lines 620, 628, 1012) that cite `ai-docs/plans/2026-05-08-instruction-file-rewrite.md` get rewritten to point within `done/` and / or to the new reference doc as appropriate.
4. `git mv ai-docs/plans/2026-05-08-instruction-file-rewrite.md ai-docs/plans/done/2026-05-08-instruction-file-rewrite.md` after the in-place strip lands.
5. Add an `## Agent Docs` table row in `AGENTS.md` for the new reference doc.
6. Add a verbose-body section for the new reference doc in `ai-docs/agent-docs-index.md` matching the existing pattern (heading `### ai-docs/instruction-file-validation.md` followed by 1–2 sentence description).
7. Update stale path citations to `ai-docs/plans/2026-05-08-instruction-file-rewrite.md` (no `done/`) in `ai-docs/plans/done/2026-05-10-process-improvements.md` lines 11 and 380 — rewrite both to the post-move path `ai-docs/plans/done/2026-05-08-instruction-file-rewrite.md` (or the corresponding relative link from the `done/` directory; both ends now sit in `done/`, so the relative link reduces to a same-directory filename).
8. The rename spec `2026-05-22-rename-code-review-to-project-review.spec.md` already merged (PR #532, commit cd904d0) and now lives at `ai-docs/plans/done/2026-05-22-rename-code-review-to-project-review.spec.md`. Its `--exclude=2026-05-08-instruction-file-rewrite.md` carve-out becomes redundant once the move lands (the `--exclude-dir=done` covers it) BUT the carve-out is inside a frozen `done/` artefact — `done/**` artefacts are not retroactively rewritten per the project's archival convention used by this very task. Leave it; AC verification adjusts the `--exclude-dir=done` exclusion list to ensure the grep still returns zero.

## Out of scope

- Phase 2 of the source rewrite plan itself (rewriting procedural skills under the methodology) — explicit deferral remains.
- Editing skills / agents that could later cite the new reference doc (`task`, `project-review`, `context-reset`, `spec-writer`, `triage-runner`). Listed as future candidates only.
- Re-running v5 validation on Phase 1 files — handled by `/improve` or `/ai-audit` if they naturally fire.
- Adding an `ai-docs/plans/INDEX.md` row — meta-plans don't get INDEX rows (precedent: `process-improvements.md`).
- Rewriting `done/**` artefacts to update the rename spec's `--exclude=` list — `done/` is archival.

## Deferred

- Wiring the new reference doc into `/task` / `/project-review` / `/context-reset` / `spec-writer` / `triage-runner` skill / agent files as a load-on-demand reference for `instruction file rewrite` triggers | low priority; methodology is only consumed when a contributor opts in via the source plan's Phase 2 follow-up | yes, separate issue when Phase 2 lands

## Key decisions

| Question | Decision |
|---|---|
| AC3 of the issue body proposes an `ai-docs/skill-size-exemptions.md` entry if the new doc exceeds 200 lines. Does that index apply to `ai-docs/*.md`? | **No.** `ai-docs/skill-size-exemptions.md` line 1–4 explicitly scope the index to `.claude/skills/*/SKILL.md`, and `/ai-audit` Checklist K item 1 reads it for that purpose only. The new doc therefore does NOT need an exemption row regardless of its size. The 200-line soft target referenced in the issue body is from `ai-docs/code-style.md` § File size which targets `.rs` files (`200–400 lines per .rs file excluding #[cfg(test)]`). No equivalent rule exists for `ai-docs/*.md`; sibling reference docs are 270 (`agent-writing-style.md`), 477 (`code-style.md`), 615 (`doc-convention.md`) lines. The new doc will land at ~900–1,000 lines (extracted material — line count estimated from source line ranges) — within the project precedent for dense reference material. No exemption needed; no line-count AC. |
| The methodology bodies cite specific PR numbers, dates, and historical context (e.g. "v4.4 replay batch validated 6/6", PR #149 `document_features` placement pilot). Extract verbatim or generalise? | **Extract verbatim.** The methodology sections reference historical PRs as worked examples / calibration evidence; rewriting them as abstract pattern descriptions destroys the load-bearing concreteness that makes the methodology testable. The new doc inherits these citations unchanged. The historical-narrative carve-out / `done/**`-is-frozen rule does NOT apply because the new doc is forward-living reference material — but the cited PR numbers themselves are historical facts that remain accurate. |
| Should the new reference doc carry its own Propagation-Rule sync-group row? | **No.** The new doc is read-on-demand reference (cat-3 documentation), not part of any enforce-each-other group with sister files. Per the issue body's explicit guidance: "No Propagation-Rule sync-group row needed". |
| Where do the internal self-references inside the surviving plan file (currently lines 620, 628, 1012 — citing `ai-docs/plans/2026-05-08-instruction-file-rewrite.md` and `../plans/2026-05-08-instruction-file-rewrite.md#methodology-limitations`) point after the move? | The plan moves to `done/`. The Citation section is rewritten — references to the methodology sections point at the new reference doc; references to historical decision context point at the same file via in-document anchors (relative paths drop the `ai-docs/plans/` prefix since both citations and target are now in `done/`). Line 1012's `#methodology-limitations` anchor moves with the section, so the link becomes a cross-link to the new doc's `Known biases & limitations` section. |
| Doc name | **`ai-docs/instruction-file-validation.md`** — resolved Round 1. Name leads with the artefact under test (instruction files) rather than the methodology (validation / testing / comprehension); pattern matches sibling docs that name the artefact first (`agent-writing-style.md`, `code-style.md`, `doc-convention.md`, `api-naming.md`). |

## Technical constraints

- Pre-publish project — no API stability concern; this task touches no Rust code.
- The new doc + the AGENTS.md row addition + the agent-docs-index.md section must pass the `wc -c` 35,000-char early-warning check enumerated by the *35k/40k* AXIOM. AGENTS.md is currently at 35,807 chars (already over the early warning); adding ~150–200 chars for one table row does not regress the warning state, but the design phase confirms the addition does not push AGENTS.md past 40,000.
- The new doc lives under `ai-docs/` (workspace convention: durable methodology, not skill / agent / rule); it does NOT get an `ai-docs/skill-size-exemptions.md` entry (see Key decisions).
- `cargo build` + `cargo clippy --workspace --all-targets -- -D warnings` must still pass (baseline sanity — no code touched).
- `actionlint` not exercised — no workflow files touched.
- Self-review agent runs on the full diff before `git push` per `/task` Step 10.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | New file `ai-docs/instruction-file-validation.md` exists, containing all seven sections enumerated in Scope item 2 — structural-completeness check passes: every section in the *Scope of extraction* table appears in the new doc; nothing dropped silently. |
| AC2 | `AGENTS.md` § *Agent Docs* table has one new row: `\| ai-docs/instruction-file-validation.md \| Dual-model instruction-file-clarity test methodology + bias taxonomy + subagent prompt templates. Read on demand. \|`. |
| AC3 | `ai-docs/agent-docs-index.md` has one new section heading `### ai-docs/instruction-file-validation.md` with a 1–2 sentence verbose body matching the existing pattern. |
| AC4 | `ai-docs/plans/done/2026-05-10-process-improvements.md` lines 11 and 380 reference the post-move plan path `2026-05-08-instruction-file-rewrite.md` (relative from `done/`, no leading `../plans/`). |
| AC5 | `ai-docs/plans/2026-05-08-instruction-file-rewrite.md` is stripped of the seven methodology sections per Scope item 2, the `## Citation in future PRs` section is rewritten to point at the new reference doc for methodology and at the plan file for historical decision context only, and the file is moved to `ai-docs/plans/done/2026-05-08-instruction-file-rewrite.md` via `git mv`. |
| AC6 | `grep -rn '2026-05-08-instruction-file-rewrite' . --exclude-dir=.git` returns only: the plan file at its new `done/` location; updated path citations in `done/2026-05-10-process-improvements.md`; internal self-references inside the plan (now under `done/`); the carve-out reference inside `done/2026-05-22-rename-code-review-to-project-review.spec.md` + `.design.md` (frozen `done/` artefacts — not retroactively rewritten). NO stale `ai-docs/plans/2026-05-08-instruction-file-rewrite.md` (without `done/`) outside historical-narrative carve-outs. |
| AC7 | `wc -c AGENTS.md CLAUDE.md .claude/skills/**/*.md .claude/agents/**.md .claude/rules/*.md ai-docs/{code-style,doc-convention,context,agent-writing-style,corrections-log}.md` — every enumerated file remains below 40,000 chars (AGENTS.md AXIOM hard cap). The 35,000-char early-warning state of AGENTS.md does not regress; the design phase confirms the table-row addition lands below 40,000 chars. |
| AC8 | Markdown relative-link trace per `AGENTS.md` § *Workflow*: pick one relative link inside the new reference doc and verify it resolves via `realpath`. |
| AC9 | `cargo build` + `cargo clippy --workspace --all-targets -- -D warnings` pass (baseline sanity — no code touched). |
| AC10 | `self-review` agent runs on the diff before `git push` per `/task` Step 10; verdict is APPROVE (or REJECT-fix-loop terminates with APPROVE within 3 iterations). |

## Open questions

(none — doc name resolved to `instruction-file-validation.md` in Round 1; the line-count and exemption concerns from the issue body's AC3 are resolved silently per the Key decisions table.)
