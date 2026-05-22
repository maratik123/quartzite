# Reframe instruction-file size-cap AXIOM (advisory + widened scope)

**Source:** issue #383
**Date:** 2026-05-22
**Tracked in:** #383

## Scope

1. **Rewrite the size-cap AXIOM block in `AGENTS.md` (currently ~lines 60–72)** so the three-band table conveys *advisory severities* rather than blockers (vocabulary aligned with `/ai-audit` Sub-check 9's severity column):
   - `≥ 40,000 chars` → **`major`** (was: `STOP. Plan extraction / dedup before the next commit`).
   - `35,000–39,999 chars` → **`minor`** (band unchanged; explicit `minor` severity label added; current text "Proactive extraction pass; do not let the next `/task` push it over 40k" stays as the right-column action verb).
   - `< 35,000 chars` → `OK` (unchanged).
   - The new wording must NOT imply a separate mechanical gate or a per-commit blocker; sizing work belongs to `/ai-audit`'s next pass.
2. **Drop the trailing sentence** *"Until `scripts/check-instruction-file-sizes.sh` lands as a pre-commit / CI gate, any `/task` whose work touches an instruction file should run this command before commit."* from the AGENTS.md AXIOM block.
3. **Widen the audited file set** in the AGENTS.md "Applies to" prose AND in the `wc -c` quick-scan command:
   - Replace `.claude/skills/**/SKILL.md` with `.claude/skills/**/*.md` (now covers `reference.md` siblings + every other in-skill markdown).
   - Add `.claude/rules/*.md` (rules surface introduced by PR #521; now load-bearing instruction surface). Flat glob — `.claude/rules/` currently has no subdirectories; recursive `**` would mislead readers into expecting one.
4. **Sweep the AGENTS.md Propagation-Rule row** (~line 220, the `ai-docs/skill-size-exemptions.md` entry of the Size-exemption-index group) and **`## Agent Docs` row** (~line 253, the `ai-docs/skill-size-exemptions.md` row) to drop the trailing "deferred `scripts/check-instruction-file-sizes.sh` (#383)" forward pointers.
5. **Update `.claude/skills/ai-audit/reference.md` Sub-check 9** (~lines 126, 135–155):
   - Widen the `wc -c` command's file set to match the AGENTS.md change (add `.claude/skills/**/*.md` superset + `.claude/rules/*.md`). Shell-glob form is acceptable here per the existing note tying back to Pattern 4.
   - Reframe the `≥ 40,000` row's severity column from `blocker` to `major` (advisory, not gating; `35,000–39,999` stays `minor`). The AGENTS.md AXIOM block uses the same `major` / `minor` labels — vocabulary unified across both surfaces.
   - Drop the trailing sentence *"The mechanical pre-commit gate is planned in #383; until that lands, Sub-check 9 fires per-`/ai-audit`-run."* — replace with a plain restatement that Sub-check 9 is the only enforcement surface for the size-cap rule (no forward pointer to a future gate).
6. **Update `ai-docs/skill-size-exemptions.md` (~lines 22, 110)** to drop the "Deferred `scripts/check-instruction-file-sizes.sh` (#383)" bullets / forward pointers. The "Consumers" list collapses to `/ai-audit` Checklist K item 1 as the sole consumer.
7. **Run the Propagation-Rule sweep** across `.claude/agents/` / `.claude/skills/` / `.claude/rules/` (using `grep -rn` against those directories as search-tree roots for `scripts/check-instruction-file-sizes` and `40,000 chars` / `35,000 chars` cap mentions) and apply the same scope-widening + advisory-warning reframe to any sister mention discovered. Note the asymmetry the AGENTS.md / Sub-check 9 prose uses: `.claude/skills/**/*.md` (recursive — each skill is a directory) vs `.claude/rules/*.md` (flat — no subdirectories present). Specifically check `ai-docs/agent-writing-style.md § 8. 40k char-cap on instruction files` (per Sub-check 9's "rule-of-truth" pointer); update its covered-set enumeration to match the new globs.

## Out of scope

- Adding any new shell scripts or CI gates (explicitly rejected by user direction 2026-05-22).
- Actually shrinking files currently above 35k / 40k chars — that is `/ai-audit`'s job once its widened scan starts firing against the new file set.
- Refactoring the AXIOM into a non-AXIOM (the blockquote / table format stays; only the severity wording and covered-set change).
- Repointing the rule-of-truth from `ai-docs/agent-writing-style.md § 8` to a new location.

## Deferred

- `scripts/check-instruction-file-sizes.sh` precommit/CI gate | why: superseded by user direction — `/ai-audit` Sub-check 9 is the only enforcement surface | separate issue needed? **No** — this spec retires the deferred-script forward pointers; the row in `ai-docs/deferred/ci-docs-workflow.md` should be dropped by `/triage` after merge (NOT by this PR, per Boundary rule on `_inbox.md` editing).

## Key decisions

| Question | Decision |
|---|---|
| Should the `≥ 40,000` row stay as a blocker in any surface? | **No.** All surfaces reframe to advisory (`major` severity label in AGENTS.md prose AND in `/ai-audit` Sub-check 9 — vocabulary unified). No surface keeps blocker semantics. |
| What's the wording for the 35k band's name? | `minor` severity in AGENTS.md prose AND in Sub-check 9 (unified vocabulary). The action verb in AGENTS.md's right column (`"Proactive extraction pass; do not let the next `/task` push it over 40k"`) is unchanged; only the band gets the explicit `minor` label. |
| Why `major` / `minor` (not `hard warning` / `soft warning`)? | Round-2 user direction (2026-05-22): align the AGENTS.md severity vocabulary with `/ai-audit` Sub-check 9's severity column, which already uses `minor` / `major` / `blocker`. Using the same vocabulary across both surfaces removes a translation step when an audit finding references the rule. |
| Glob form vs explicit paths? | Glob form is acceptable both in AGENTS.md's quick-scan command and Sub-check 9's `wc -c` invocation — Pattern 4's explicit-path requirement applies to fail-loud bullet lists, NOT shell commands consuming the set. The existing Sub-check 9 note already memorialises this; no new exemption needed. |
| Why recursive `**` for skills but flat `*` for rules? | Round-3 user direction (2026-05-22): match each glob to the actual directory shape. `.claude/skills/` is nested (each skill is a directory with `SKILL.md` + optional `reference.md` siblings), so `.claude/skills/**/*.md` is accurate. `.claude/rules/` currently holds only `ast-index.md` with no subdirectories, so `.claude/rules/*.md` is accurate; a recursive `**` glob would mislead future readers into expecting a nested structure that doesn't exist. The flat glob would still match any future top-level rule file. Documented here so future Propagation-Rule sweeps don't "fix" the asymmetry. |
| Should the `ai-docs/skill-size-exemptions.md` Consumers list keep two bullets? | **No.** Collapse to a single bullet (`/ai-audit` Checklist K item 1). The "Once the mechanical pre-commit gate lands..." bullet at lines 110–114 is deleted entirely. |
| Does `ai-docs/deferred/ci-docs-workflow.md`'s #383 row need cleanup in this PR? | **No.** Per AGENTS.md Boundary rule (`_inbox.md` and deferred files are written only by `/task` Step 12 / `/triage`), this PR does NOT hand-edit `_inbox.md` or the thematic deferred file. Drop happens via `/triage` after merge. |

## Technical constraints

- AGENTS.md is itself bounded by the 40,000-char `major` threshold; the AXIOM edit must NOT increase total file size meaningfully. Net change should be near-zero (deleted "Until ... pre-commit gate" sentence offsets added `major` / `minor` labels + the two new globs).
- The Propagation Rule fires on every edit to an instruction file — the sweep MUST cover every file that currently references either `scripts/check-instruction-file-sizes` or the old `.claude/skills/**/SKILL.md` glob, and apply the corresponding wording change in the same PR.
- Sub-check 9's `wc -c` command must remain a single invocation (multi-line continuation is fine) so the audit can shell out once and parse one stream.
- The `/ai-audit` design lives in `.claude/skills/ai-audit/reference.md`; touching it requires only the Propagation-Rule sister-file check (no separate sync-group entry covers `reference.md` siblings — verify on edit).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `grep -rn 'scripts/check-instruction-file-sizes' AGENTS.md CLAUDE.md .claude/ ai-docs/` returns **zero** hits after the PR's diff (sweep is exhaustive). |
| AC2 | AGENTS.md size-cap AXIOM block (~lines 60–72) contains the severity labels `major` (against the `≥ 40,000` row) and `minor` (against the `35,000–39,999` row) — vocabulary unified with `/ai-audit` Sub-check 9's severity column; the word `STOP` is removed from this AXIOM block; the trailing "Until `scripts/check-instruction-file-sizes.sh` lands..." sentence is removed. The strings `hard warning` and `soft warning` do NOT appear in the AGENTS.md AXIOM block. |
| AC3 | AGENTS.md "Applies to" prose AND the `wc -c` quick-scan command BOTH list `.claude/skills/**/*.md` (NOT `.claude/skills/**/SKILL.md`) AND `.claude/rules/*.md` (flat — NOT `.claude/rules/**/*.md`). |
| AC4 | `.claude/skills/ai-audit/reference.md` Sub-check 9 table lists `major` (NOT `blocker`) in the `≥ 40,000` severity row; the `wc -c` command in the same sub-check includes the two scope additions (`.claude/skills/**/*.md` superset + `.claude/rules/*.md`); the trailing "mechanical pre-commit gate is planned in #383..." sentence is removed. |
| AC5 | `ai-docs/skill-size-exemptions.md` no longer references `scripts/check-instruction-file-sizes.sh` (the "Deferred..." bullet at lines 110–114 is removed AND the line-22 consumers bullet is removed); the "Consumers" list collapses to a single `/ai-audit` Checklist K item 1 bullet. |
| AC6 | Propagation-Rule sweep is verifiable: `grep -rn '.claude/skills/\*\*/SKILL.md' AGENTS.md .claude/agents/ .claude/skills/ .claude/rules/ ai-docs/agent-writing-style.md` returns no hits *in size-cap contexts* (other contexts like `.claude/skills/**/SKILL.md` referenced for compaction-recovery are unaffected); `ai-docs/agent-writing-style.md § 8` (if it enumerates the covered set verbatim) reflects the widened globs — recursive `.claude/skills/**/*.md` and flat `.claude/rules/*.md`. |
| AC7 | `cargo build` is NOT required (docs-only change); `actionlint` is NOT required (no workflow changes); a `wc -c AGENTS.md` post-edit reading confirms the file remains < 40,000 chars (AXIOM self-conformance). |
| AC8 | `gh pr view <PR-N>` body cites issue #383 and summarises the three reframe pillars (drop script reference, advisory warnings, widened file set). |

## Open questions

None. The issue body specifies the file targets, line references, and wording reframes concretely enough to proceed to design.

