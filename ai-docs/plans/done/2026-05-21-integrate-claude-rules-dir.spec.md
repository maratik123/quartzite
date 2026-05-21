# Integrate `.claude/rules/` directory into harnesses and instructions

**Source:** issue #522
**Date:** 2026-05-21
**Tracked in:** #522

## Context

PR #521 introduced `.claude/rules/` (first inhabitant: `.claude/rules/ast-index.md`)
as an on-demand instruction surface — files read only when an agent invokes the
relevant task, not auto-loaded per invocation. The directory is currently
invisible to the project's safety net: Propagation-Rule greps, `/ai-audit`
fail-loud-pattern sweep, the size-cap quick-scan, and `AGENTS.md § Agent Docs`
all enumerate file paths explicitly and omit `.claude/rules/`.

This spec covers the cross-cutting integration pass so future rule files (or
edits to the existing one) inherit the same conventions as `.claude/skills/`
and `.claude/agents/`.

## Scope

1. **Mechanical edits (no decision required):**
   1. `AGENTS.md` Propagation-Rule procedure grep — append `.claude/rules/` to
      the search roots in both occurrences (procedure-row body inside the table
      AND step 1 of the **Procedure:** numbered list).
   2. `AGENTS.md § Agent Docs` table — add a row for `.claude/rules/ast-index.md`
      pointing at the on-demand code-search rules and cross-referencing the
      `§ Build & Test` Search line that already cites it.
   3. Any other propagation row that mentions `.claude/agents/` AND
      `.claude/skills/` as a paired set — extend to include `.claude/rules/`
      where the rule's intent is "search every instruction directory".
2. **Decision-driven edits (Q-A — size cap):** rule files are exempt from the
   35k/40k char cap and NOT included in the `wc -c` quick-scan list at
   `AGENTS.md:62` / `:70` and `.claude/skills/ai-audit/reference.md:137`. No
   edit to these size-cap surfaces is needed beyond confirming the omission is
   intentional (record rationale in Key Decisions).
3. **Decision-driven edits (Q-B — Checklist M corpus):** add
   `.claude/rules/**/*.md` to the audited corpus list at
   `.claude/skills/ai-audit/reference.md:114`. The fail-loud-pattern sweep
   now runs against rule files.
4. **Decision-driven edits (Q-C — `Escalated?` enum):** introduce a new
   `rules:[name]` value in the `Escalated?` enum, alongside the existing
   `skill:[name]` / `agent:[name]` / `hook` / `settings` / `AGENTS.md` /
   `doc-convention` / `code-style`. Propagate to all four declaration sites
   per the Learning-Log group propagation row:
   - `AGENTS.md § Learning Log § Entry format` enum line
   - `.claude/agents/self-improve.md` (parse-site)
   - `.claude/agents/learnings-escalation-audit.md` (parse-site)
   - `ai-docs/corrections-log.md` field glossary
5. **Propagation Rule itself:** add a new propagation row in `AGENTS.md` for
   "edit to `.claude/rules/<file>.md` → run the standard grep" (the existing
   "Any other instruction file" fallback already covers this, but make it
   explicit since rule files are now a documented surface).
6. **Deferred-row entry:** append a row to `ai-docs/deferred/_inbox.md` for
   #383 (`scripts/check-instruction-file-sizes.sh`) noting the chosen Q-A
   semantics — the future script must NOT include `.claude/rules/` in its
   scan list.

## Out of scope

- Adopting additional rule files (this PR is purely the integration pass;
  new rule content lands in its own PR).
- Landing `scripts/check-instruction-file-sizes.sh` (#383) itself.
- Auto-loading rule files into the per-invocation prompt — would invalidate
  Q-A's rationale; explicit non-goal.
- Folding this work into PR #521 (rationale: blast-radius isolation + each
  PR keeps its ACs verifiable; the issue body's "Why not folded into PR #521"
  section captures this).

## Deferred

- `scripts/check-instruction-file-sizes.sh` (#383) | not landed yet; must NOT
  include `.claude/rules/` in scan list (Q-A semantics) | already tracked under #383
- Cross-rule-file propagation conventions (when 2+ rule files exist and one
  changes a shared term) | premature — only one rule file exists today | revisit when 2nd rule file lands

## Key decisions

| Question | Decision |
|---|---|
| Q-A — Size cap for rule files? | **No cap.** Rule files are on-demand (read only when an agent invokes a search task), not auto-loaded into the per-invocation prompt. The 35k/40k AXIOM exists to bound per-invocation cost; rule files don't pay that cost. NOT added to the `wc -c` quick-scan at `AGENTS.md:70` or the Checklist K recipe at `reference.md:137`. |
| Q-B — Should `/ai-audit` Checklist M sweep rule files? | **Yes.** Rule files contain mandatory rules (ALWAYS / NEVER), so style drift here is the same hazard as drift in a `SKILL.md`. Add `.claude/rules/**/*.md` to the audited corpus list at `reference.md:114`. |
| Q-C — `Escalated?` enum value for rule-file escalations? | **Add `rules:[name]`.** Mirrors the existing per-surface convention (`skill:[name]`, `agent:[name]`). Reusing `AGENTS.md` would lose surface granularity. Propagate to all four declaration sites per the Learning-Log group propagation row. |
| Mechanical (1)/(2)/(3) edits — apply regardless of Q-A/B/C? | **Yes.** No decision dependency. |
| Where does the new `.claude/rules/*` Agent Docs row sit in the table? | Adjacent to existing `.claude/agents/spec-writer.md` row family; group entry by `.claude/` prefix. |
| Should the Propagation Rule grow a dedicated row for `.claude/rules/*`, or rely on the "Any other instruction file" fallback? | **Add a dedicated row** for explicitness — once the directory is documented in `§ Agent Docs`, the propagation table should mention it directly so the next-agent doesn't need to read the fallback rule. |

## Technical constraints

- All edits in a single PR titled
  `chore(.claude/rules): integrate rules dir into instructions + harnesses`.
- Edits to `AGENTS.md` trigger the existing AGENTS.md propagation row; run
  the standard `grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/
  AGENTS.md ai-docs/agent-writing-style.md` (now with `.claude/rules/`
  appended) to confirm no lingering references.
- Edits to `.claude/agents/self-improve.md` AND
  `.claude/agents/learnings-escalation-audit.md` AND
  `ai-docs/corrections-log.md` are mandatory whenever the `Escalated?` enum
  changes — Learning-Log group propagation row.
- The audited corpus at `.claude/skills/ai-audit/reference.md:114` is named
  inline (not glob-derived); the edit literally adds
  `+ .claude/rules/**/*.md` to the bullet enumeration.
- Existing `.claude/rules/ast-index.md` was authored to Patterns 1–7 already
  — Checklist M run after this spec lands MUST find zero new findings
  against it (AC4 below).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `grep -rn '\.claude/rules/' AGENTS.md` returns at minimum: (a) the Propagation-Rule procedure grep with `.claude/rules/` appended; (b) the new `§ Agent Docs` row for `.claude/rules/ast-index.md`; (c) the dedicated Propagation-Rule table row for edits to `.claude/rules/<file>.md`. |
| AC2 | `grep -n '\.claude/rules/' .claude/skills/ai-audit/reference.md` returns the Checklist M audited-corpus entry `.claude/rules/**/*.md` at the existing corpus list around line 114. |
| AC3 | The `Escalated?` enum line in `AGENTS.md § Learning Log § Entry format` includes `rules:[name]`. The same value appears in: (a) `.claude/agents/self-improve.md`, (b) `.claude/agents/learnings-escalation-audit.md`, (c) `ai-docs/corrections-log.md` field glossary. Verify with `grep -n 'rules:\[name\]' AGENTS.md .claude/agents/self-improve.md .claude/agents/learnings-escalation-audit.md ai-docs/corrections-log.md`. |
| AC4 | `/ai-audit` Phase 2 Checklist M run on master after merge surfaces zero new findings introduced by adding `.claude/rules/**/*.md` to the corpus. (Existing `.claude/rules/ast-index.md` was authored to Patterns 1–7; run is a sanity check.) |
| AC5 | A demonstration edit to `.claude/rules/ast-index.md` that mutates a shared term (e.g. `ast-index update`) is caught by `grep -rn "ast-index update" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md .claude/rules/` against any sister file that mentions the same string. (Demonstrated during design review; no commit needed.) |
| AC6 | `ai-docs/deferred/_inbox.md` contains a row referencing #383 noting that `scripts/check-instruction-file-sizes.sh` MUST NOT include `.claude/rules/` in its scan list (Q-A semantics carried forward). |
| AC7 | All standard CI gates pass on the PR: `cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, doc gate, `actionlint` (no workflow files touched), instruction-file size scan (target files all `< 35,000` chars). |
| AC8 | PR-self-review (per `/task` Step 10) returns APPROVE before push. |

## Open questions

None. The issue body's three open decisions (Q-A / Q-B / Q-C) all carry
explicit Option-1 recommendations from the user, justified inline. Taking
the recommended option silently and recording rationale in *Key decisions*
matches the spec-writer optimization target — Option-2 alternatives are
preserved in *Key decisions* and can be revisited via Spec Amendment if
design surfaces unforeseen drawbacks.
