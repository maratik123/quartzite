# Design: Reframe instruction-file size-cap AXIOM (advisory + widened scope)

**Issue:** #383
**Date:** 2026-05-22

## Approach

Docs-only edit pass across four files. The spec already specifies wording targets, file locations, and acceptance criteria; the design's job is to (a) confirm the line-locations the spec cites by reading the live tree, (b) order the edits so each commit leaves the corpus internally consistent, and (c) bind the Propagation-Rule sweep to a concrete grep so AC1 + AC6 are mechanically verifiable.

The reframe has three orthogonal pillars, applied uniformly to every surface that today carries the rule:

1. **Drop forward pointers to `scripts/check-instruction-file-sizes.sh` (#383).** The script is no longer planned (user direction 2026-05-22); every "Until X lands..." / "Deferred X (#383)" / "Mechanical pre-commit gate planned in #383" sentence is removed wherever it appears. AC1 verifies this exhaustively via `grep -rn 'scripts/check-instruction-file-sizes'` across `AGENTS.md CLAUDE.md .claude/ ai-docs/`.
2. **Advisory severity vocabulary.** The three-band table is rewritten so `≥ 40,000` carries severity label `major` (was `STOP` / `blocker`) and `35,000–39,999` carries `minor` (band unchanged; label made explicit). Vocabulary unified with `/ai-audit` Sub-check 9's severity column so an audit finding referencing the rule needs no translation.
3. **Widened covered set.** `.claude/skills/**/SKILL.md` → `.claude/skills/**/*.md` (recursive — now covers `reference.md` siblings) AND `.claude/rules/*.md` added (flat — `.claude/rules/` currently has no subdirectories; recursive `**` would mislead). Asymmetry is documented in the spec's Key Decisions table and propagated to every covered-set enumeration.

Live-tree investigation confirms the affected files and line ranges:

- `AGENTS.md` lines 61–70 (AXIOM block + quick-scan command), line 220 (Propagation-Rule Size-exemption-index row), line 253 (Agent Docs row).
- `.claude/skills/ai-audit/reference.md` lines 132–154 (Sub-check 9 body) — severity table at line 148 (`blocker` → `major`), `wc -c` command at lines 136–140 (file-set widening), trailing sentence at line 154 (forward-pointer drop).
- `ai-docs/skill-size-exemptions.md` line 22 (Consumers list second bullet) and lines 110–114 (Notes "Deferred" bullet) — both dropped; Consumers list collapses to a single `/ai-audit` Checklist K item 1 bullet.
- `ai-docs/agent-writing-style.md` § 8 — lines 151–152 ("Mechanical pre-commit gate planned in #383; this audit-side back-stop fires in the meantime."), line 159 (severity table row carries `STOP` / blocker semantics), line 167 (covered-set bullet `.claude/skills/**/SKILL.md`). Add a `.claude/rules/*.md` bullet to the covered-set list.

**Rejected alternatives:**

- *One-commit monolithic edit across all four files.* Rejected — the Propagation-Rule sweep step needs the AGENTS.md edits visible first so the `grep -rn` invocation against `.claude/rules/`, `.claude/skills/`, etc. discovers sister mentions seeded by the new wording. Two-commit split (anchor + sweep) makes the sweep diff smaller and easier to review.
- *Keep `blocker` in `/ai-audit` Sub-check 9 and use `hard warning` / `soft warning` in AGENTS.md.* Rejected explicitly by Key Decisions row 3 — unified `major` / `minor` vocabulary removes a translation step.
- *Drop the entire AXIOM blockquote and replace with a plain `## ` section.* Rejected by spec's "Out of scope" item 3 — the blockquote / table format stays; only severity wording and covered-set change.
- *Move the rule-of-truth from `ai-docs/agent-writing-style.md § 8` to a new home.* Rejected by spec's "Out of scope" item 4 — repointing is out of scope.
- *Hand-edit `ai-docs/deferred/ci-docs-workflow.md` and `_inbox.md` to drop the deferred-script rows.* Rejected by Boundary rule on `_inbox.md` editing (and by Key Decisions row 7) — drop happens via `/triage` after merge, NOT in this PR.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rewrite the AGENTS.md size-cap AXIOM block — severity labels (`major` / `minor`), drop the `STOP` word, drop the trailing "Until ... pre-commit gate" sentence, widen the "Applies to" prose AND the `wc -c` quick-scan command (`.claude/skills/**/*.md` superset + `.claude/rules/*.md` flat). | `AGENTS.md` (lines 61–70) | — |
| 2 | Sweep the two AGENTS.md downstream rows: drop the "deferred `scripts/check-instruction-file-sizes.sh` (#383) reads the same index once landed" forward pointer from the Size-exemption-index Propagation-Rule row (line 220), AND drop the trailing "+ deferred `scripts/check-instruction-file-sizes.sh`" from the Agent Docs row (line 253). | `AGENTS.md` (lines 220, 253) | 1 |
| 3 | Update `.claude/skills/ai-audit/reference.md` Sub-check 9 — widen the `wc -c` invocation's covered set to match AGENTS.md (recursive `.claude/skills/**/*.md` superset + flat `.claude/rules/*.md`), reframe the `≥ 40,000` severity column from `blocker` to `major`, replace the trailing "mechanical pre-commit gate is planned in #383..." sentence with a plain restatement that Sub-check 9 is the only enforcement surface (no forward pointer). | `.claude/skills/ai-audit/reference.md` (lines 132–154) | 1 |
| 4 | Update `ai-docs/skill-size-exemptions.md` — drop the line-22 Consumers bullet referencing the deferred script AND the lines 110–114 Notes bullet ("Deferred `scripts/check-instruction-file-sizes.sh` (#383)"). Consumers list collapses to a single `/ai-audit` Checklist K item 1 bullet. | `ai-docs/skill-size-exemptions.md` (lines 22, 110–114) | — |
| 5 | Run the Propagation-Rule sweep: `grep -rn 'scripts/check-instruction-file-sizes\|.claude/skills/\*\*/SKILL.md' .claude/agents/ .claude/skills/ .claude/rules/ ai-docs/agent-writing-style.md`. Apply the same advisory-reframe + scope-widening to `ai-docs/agent-writing-style.md § 8` (lines 151–152 drop forward pointer; line 159 severity row reframe; line 167 covered-set bullet widen to `.claude/skills/**/*.md`; add `.claude/rules/*.md` bullet). Verify no other sister mentions remain. | `ai-docs/agent-writing-style.md` (lines 146–185) plus any sister mentions discovered by the sweep | 1, 2, 3, 4 |
| 6 | Final verification pass — run AC1's exhaustive grep + AC6's sweep grep + `wc -c AGENTS.md` (must stay < 40,000 chars per AC7); confirm each AC1–AC6 evidence command returns the expected output. | (verification only — no file edits) | 5 |

**Note — Subtask 5 expected non-size-cap false-positives.** Expected non-size-cap false-positives from the narrow-glob grep `'.claude/skills/\*\*/SKILL.md'` (do NOT edit these — they're in unrelated contexts):

- `ai-docs/agent-writing-style.md:240` — Propagation-rule corpus enumeration (sister-file list)
- `.claude/skills/ai-audit/reference.md:114, :192` — Checklist M audited-corpus enumeration
- `.claude/skills/ai-audit/reference.md:223` + `.claude/skills/ai-audit/SKILL.md:139` — anchor-aware re-verification text

Only size-cap contexts (covered-set bullets, applies-to prose, `wc -c` invocations) get rewritten.

## Handoff plan

`M = 6` subtasks. Two groups: 3 + 3.

- **Group A:** subtasks 1–3 — anchor edits. AGENTS.md AXIOM block (1), AGENTS.md downstream rows (2), and `.claude/skills/ai-audit/reference.md` Sub-check 9 (3). Group A entry runs in its own `/context-reset` subagent per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — terminal group (3 subtasks; within the 1..=3 range). `ai-docs/skill-size-exemptions.md` (4), Propagation-Rule sweep including `ai-docs/agent-writing-style.md § 8` (5), and final AC verification (6).

## Risks

- **AGENTS.md crosses 40k chars after edits.** Mitigation: AGENTS.md is currently 36,046 chars (already in the 35–40k `minor` band). Net diff should be near-zero per the spec's technical constraint — the deleted "Until ... pre-commit gate" sentence (~120 chars) plus the dropped "deferred `scripts/check-instruction-file-sizes.sh` (#383) reads the same index once landed" forward pointer (~80 chars) plus the dropped "+ deferred `scripts/check-instruction-file-sizes.sh`" Agent Docs suffix (~50 chars) offset the added `major` / `minor` severity labels (~30 chars) and the two new glob entries in the quick-scan command (~30 chars). Subtask 6 verifies via `wc -c AGENTS.md`; if total exceeds 40k, Subtask 6 surfaces it as an AC7 blocker.
- **Sister mention missed by the Propagation-Rule sweep.** Mitigation: Subtask 5 runs a verbatim `grep -rn` invocation with two ORed patterns (`scripts/check-instruction-file-sizes` for forward-pointer leaks AND `.claude/skills/\*\*/SKILL.md` for stale-glob leaks). AC6 cites a derived grep as the verification command; if Subtask 5's sweep misses a hit, AC6 re-fires it.
- **Sub-check 9's `wc -c` command becomes multi-line and breaks single-invocation parsing.** Mitigation: spec § Technical constraints requires the `wc -c` command to remain a single invocation (multi-line continuation via `\` is fine). Subtask 3 preserves the existing continuation style.
- **`.claude/rules/` directory grows a subdirectory in the future and the flat glob misses files.** Mitigation: spec Key Decisions row 5 documents the deliberate asymmetry. A future PR adding subdirectories to `.claude/rules/` will trigger the Propagation Rule and widen the glob then.
- **`/ai-audit` Checklist M Pattern 8 audit fires against the new covered set on the next run and surfaces fresh `minor` / `major` findings for files now in scope.** Mitigation: that is the intended downstream effect per spec § Out of scope item 2 — file shrinks belong to `/ai-audit`, not this PR.

## Test Design

AC1's "zero hits" applies to the active instruction surface; historical surfaces (`ai-docs/plans/done/`, the spec/design pair for this PR, and `ai-docs/learnings.md`) retain forward-pointer mentions as part of the corrections record and are filtered out before the zero-hits assertion. The verification command's `grep -v '^ai-docs/(deferred|plans|learnings)'` filter encodes this.

Docs-only change; no `#[cfg(test)]` module or `tests/` file applies. Verification is mechanical greps + `wc -c` against the live tree, run as Subtask 6:

- **AC1 verification.** Entry point: `grep -rn 'scripts/check-instruction-file-sizes' AGENTS.md CLAUDE.md .claude/ ai-docs/`. Expected output: empty (zero hits). Excluded paths (acceptable hits): `ai-docs/deferred/_inbox.md`, `ai-docs/deferred/ci-docs-workflow.md`, `ai-docs/plans/done/`, `ai-docs/learnings.md`, and `ai-docs/plans/2026-05-22-reframe-size-cap-axiom.spec.md` (the spec itself + design — they document the removal). Subtask 6 filters via `grep -v '^ai-docs/\(deferred\|plans\|learnings\)'` (or equivalent) and confirms the filtered output is empty.
- **AC2 verification.** `grep -nE '^>.*(major|minor|STOP|hard warning|soft warning)' AGENTS.md | sed -n '/AXIOM — Every project instruction file/,/Quick scan:/p'` (or human read of lines 61–70). Expected: severity labels `major` (against `≥ 40,000`) and `minor` (against `35,000–39,999`) present; words `STOP`, `hard warning`, `soft warning`, and the trailing "Until `scripts/check-instruction-file-sizes.sh`..." sentence all absent.
- **AC3 verification.** `grep -n '.claude/skills/\*\*/\*.md\|.claude/rules/\*.md' AGENTS.md` over lines 61–70. Expected: BOTH `.claude/skills/**/*.md` AND `.claude/rules/*.md` appear in the "Applies to" prose AND in the `wc -c` quick-scan command. `.claude/skills/**/SKILL.md` (the old narrow glob) does NOT appear in this block.
- **AC4 verification.** Human read of `.claude/skills/ai-audit/reference.md` Sub-check 9 (lines 132–154). Expected: severity table lists `major` (not `blocker`) for the `≥ 40,000` row; `wc -c` command includes `.claude/skills/**/*.md` superset + `.claude/rules/*.md`; the trailing "mechanical pre-commit gate is planned in #383..." sentence is replaced with a plain restatement.
- **AC5 verification.** Human read of `ai-docs/skill-size-exemptions.md`. Expected: no references to `scripts/check-instruction-file-sizes.sh` anywhere; Consumers list collapses to a single `/ai-audit` Checklist K item 1 bullet; the "Deferred ... (#383)" bullet at the old lines 110–114 is gone.
- **AC6 verification.** `grep -rn '\.claude/skills/\*\*/SKILL\.md' AGENTS.md .claude/agents/ .claude/skills/ .claude/rules/ ai-docs/agent-writing-style.md`. Expected: any remaining hits are in *non-size-cap contexts* (compaction-recovery cross-links, audit-corpus enumeration in `reference.md`, etc.); no size-cap context retains the narrow glob. `ai-docs/agent-writing-style.md § 8` covered-set list reflects the widened globs.
- **AC7 verification.** `wc -c /home/syt/RustroverProjects/quartzite/AGENTS.md`. Expected: < 40,000 chars (AXIOM self-conformance).
- **AC8 verification.** Out of scope for design phase; lands in `/task` Step 12 (PR body authoring).

No fixtures or test helpers needed — every verification is a one-shot grep / `wc -c` against the live tree.

## Open questions

None. Spec § Open questions confirmed empty; all wording, line targets, and severity vocabulary specified concretely in spec § Scope + § Key decisions.
