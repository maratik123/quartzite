# Progress: code-style-extraction

**Branch:** feat/2026-05-07-code-style-extraction
**base_commit:** b0fb996f3c006745a031b858f5fea3c3f81d30f7
**Issue:** #124
**Spec:** ai-docs/plans/2026-05-07-code-style-extraction.spec.md
**Last build:** (not yet run — Subtask 9)

## Subtasks

| # | Task | Files | Status |
|---|------|-------|--------|
| 1 | Create branch `feat/2026-05-07-code-style-extraction` | (no files) | ✅ done |
| 2 | Create `ai-docs/code-style.md` with full body | `ai-docs/code-style.md` (new) | ⬜ open |
| 3 | Verify relative links + anchor click-through | (verification only) | ⬜ open |
| 4 | Add `code-style.md` row to AGENTS.md `Agent Docs` table | `AGENTS.md` | ⬜ open |
| 5 | Propagate references in `.claude/agents/` and INDEX.md:89 | `.claude/agents/review-findings.md`, `.claude/agents/self-review.md`, `ai-docs/plans/INDEX.md` | ⬜ open |
| 6 | Trim AGENTS.md `## Code Style` to bulleted index | `AGENTS.md` | ⬜ open |
| 7 | Update escalation taxonomy (AGENTS.md + self-improve.md) | `AGENTS.md`, `.claude/agents/self-improve.md` | ⬜ open |
| 8 | Add row to INDEX.md Active plans | `ai-docs/plans/INDEX.md` | ⬜ open |
| 9 | Sanity gate (cargo build/clippy/fmt/test/no-default-features/doc-gate; propagation re-grep) | (verification only) | ⬜ open |

## Commit grouping

- **Commit 1 (Subtask 2+3):** docs: extract Code Style into `ai-docs/code-style.md`
- **Commit 2 (Subtask 4+5):** docs: register code-style.md in Agent Docs and propagate agent citations
- **Commit 3 (Subtask 6):** docs: replace AGENTS.md Code Style body with bulleted index
- **Commit 4 (Subtask 7+8):** docs: extend escalation taxonomy with code-style; index this plan

## Files touched

- `ai-docs/code-style.md` (new)
- `AGENTS.md`
- `.claude/agents/review-findings.md`
- `.claude/agents/self-review.md`
- `.claude/agents/self-improve.md`
- `ai-docs/plans/INDEX.md`
- `ai-docs/plans/2026-05-07-code-style-extraction.{spec,design,progress}.md` (already on branch as untracked)

## Notes from design review (incorporated)

- Capture pre-trim AGENTS.md as a checkpoint before Subtask 6 (use `git show <commit-1>:AGENTS.md` or save as `AGENTS.md.original`) for AC2 verbatim audit.
- For long clusters (8 and 9) where the `awk`/`diff` recipe is fragile, fall back to `git diff <commit-1>:AGENTS.md ai-docs/code-style.md` with manual inspection.
- INDEX.md:89 rewrite must preserve rule-name specificity (not just a file-level pointer).
- After Subtask 7, run `rg 'doc-convention' .claude/agents/self-improve.md AGENTS.md` as a consistency check.
- GitHub web rendering for anchor click-through verification, not a local Markdown previewer (slug rules can disagree).

## Next action

Start Subtask 2: read AGENTS.md lines 52-141 in detail, draft `ai-docs/code-style.md` per the citation map, write the file. Capture `AGENTS.md` as a checkpoint reference before any trim (i.e., before Subtask 6).
