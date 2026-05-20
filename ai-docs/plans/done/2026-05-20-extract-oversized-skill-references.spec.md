# Extract oversized SKILL.md references

**Source:** issue #498
**Date:** 2026-05-20
**Tracked in:** #498

## Scope

Reduce each of seven `.claude/skills/*/SKILL.md` files toward the 200-line target (or document an exemption) by extracting static reference content to a sibling `reference.md` (or similar pointer file), following the existing pattern already in place in `.claude/skills/task/` (`SKILL.md` + `reference.md` + `inbox-propagation.md`).

**Landing strategy:** One PR for all seven per-skill extractions (not seven separate PRs, contrary to the issue body's "Suggested approach"). Commit granularity within the PR is a design-agent choice.

In-scope files (current sizes from issue #498):

| Skill | Lines | Chars | Biggest section flagged |
|---|---|---|---|
| `.claude/skills/master-ci-failed/SKILL.md` | 403 | 26,951 | `## Decisions log` template (L133–352) |
| `.claude/skills/ai-audit/SKILL.md` | 369 | 33,312 | `## Phase 2 — Instruction audit` (L60–329) |
| `.claude/skills/pr-ci-failed/SKILL.md` | 358 | 25,982 | `## CI-fix cycle round M` template (L118–306) |
| `.claude/skills/pr-commented/SKILL.md` | 339 | 25,348 | `## Comment cycle round M` template (L116–287) |
| `.claude/skills/bugfix/SKILL.md` | 257 | 14,095 | Step-by-step workflow (multiple ~1–4k sections) |
| `.claude/skills/interview/SKILL.md` | 227 | 12,127 | `## Workflow` (~6,370 chars) |
| `.claude/skills/task/SKILL.md` | 219 | 25,002 | `## ⚡ Fourth` reconciliation preamble + subsequent steps (~17,469 chars) |

For each skill: read end-to-end, classify each section as one of:

1. **Static reference** loaded on-demand by the reader — extract to `reference.md` (or thematic sibling, e.g. `inbox-propagation.md`).
2. **Round-template scaffolding** the skill writes into during a round — KEEP in `SKILL.md` body (extraction would break the writeable template flow).
3. **Workflow narrative** executed once per invocation — KEEP in `SKILL.md` body.
4. **Already extracted** (e.g. `task/reference.md` exists) — verify pointer integrity, no further action.

Apply extraction only to category (1). Document any retained category-(2)/(3) section that pushes the file > 200 lines with an in-file exemption comment naming the load-bearing reason.

After every per-skill extraction:

- Verify the compaction-recovery callout at the top of `SKILL.md` (where present) still resolves all referenced paths.
- Trace at least one relative link via `realpath` per AGENTS.md *Markdown link tracing after generate/move*.
- Confirm the skill's gate-checklist / anti-patterns / re-invocation semantics remain in `SKILL.md` body (these are skim-on-entry content, not on-demand reference).

## Out of scope

- The `scripts/check-instruction-file-sizes.sh` pre-commit / CI gate (AGENTS.md mentions this as future work; tracked separately).
- AGENTS.md + `ai-docs/context.md` size squeeze — tracked in #497.
- Any change to skill workflow behaviour. Pure refactor; no semantic deltas.
- Re-extracting `.claude/agents/*.md` files (issue scope is `.claude/skills/*/SKILL.md` only).
- Adding new content to any SKILL.md — net-additive edits land in separate PRs.

## Deferred

- `scripts/check-instruction-file-sizes.sh` gate landing | needs its own design pass for soft/hard thresholds, CI placement, allow-list format | separate issue needed: yes (already implied by AGENTS.md "Until ... lands as a pre-commit / CI gate" wording).

## Key decisions

| Question | Decision |
|---|---|
| How should the seven per-skill extractions land? | **One PR** (round-1 answer). All seven skills are refactored within a single feature branch and a single PR. Commit granularity within that PR (one commit per skill vs bundled commits) is a design-agent choice based on diff size / reviewability. |
| Extraction target file name(s) | Primary sibling: `reference.md` (matches existing `task/reference.md` pattern). Thematic siblings allowed when the extracted material is a single coherent topic (precedent: `task/inbox-propagation.md`). Design agent chooses per skill. |
| 200-line target — hard or soft? | Soft. ACs accept either `≤ 200 lines` OR an in-file documented exemption naming the load-bearing reason (precedent: issue body itself names `master-ci-failed`'s Decisions log template as load-bearing). |
| Compaction-recovery callout — touchable? | No, do NOT move into `reference.md`. The callout fires before any tool call on re-entry and must remain inline. Verification (paths still resolve) is in scope; relocation is not. |
| Char-cap headroom | Each in-scope file is already < 35k chars (largest is `ai-audit` at 33,312). The 40k AXIOM gate is not the trigger; the 200-line target is. Extraction is housekeeping, not regression repair. |
| Round-template scaffolding (`## Decisions log` template in `master-ci-failed` / `pr-ci-failed` / `pr-commented`) | Keep inline by default. The template IS the section the skill writes into during each round; extracting it breaks the readable round-by-round flow. Design agent may revisit if a clean extraction exists. |

## Technical constraints

- **Propagation Rule (AGENTS.md):** edits to `.claude/skills/<skill>/SKILL.md` files in sync groups (Review / Interview / Triage / Task-Design / Spec-Amendment) propagate to sibling files. Specifically, `task/SKILL.md` is in the Task/Design group with `.claude/agents/design.md`, `.claude/agents/design-review.md`, and `.claude/skills/context-reset/SKILL.md`; `interview/SKILL.md` is in the Interview group with `.claude/agents/spec-writer.md`; the four `*-ci-failed` / `*-commented` / `task` family share the Spec-Amendment group recipe. Per-skill extraction must preserve any propagated rule text — extracting the rule to `reference.md` is acceptable provided the sync-group siblings continue to reference the same canonical wording.
- **Skill frontmatter** (`name:`, `description:`, `argument-hint:`, `allowed-tools:`, `model:` where present) MUST stay in `SKILL.md` (Claude Code skill loader requires it inline; extraction would break skill discovery).
- **Relative link integrity:** any `reference.md` extracted from `SKILL.md` must reference `../../../ai-docs/...` / `../../agents/...` correctly from its new directory depth (same depth as `SKILL.md`, so paths are unchanged in practice — but `realpath`-verify per AGENTS.md).
- **No behaviour change:** the seven skills' workflow remains byte-equivalent in semantics. A user invoking `/task`, `/pr-ci-failed`, `/pr-commented`, `/master-ci-failed`, `/bugfix`, `/interview`, `/ai-audit` must observe no functional difference.
- **Existing `task/reference.md` is the precedent.** The design phase should follow that file's structure (anchored headings the SKILL.md links to via `reference.md § <Heading>`).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Each of the 7 SKILL.md files is either `≤ 200 lines` (excluding frontmatter) OR carries an in-file comment naming the load-bearing reason for retained over-200 content. |
| AC2 | Every extracted reference file (e.g. `reference.md`, thematic siblings) lives in the same directory as its source `SKILL.md` and is reachable from `SKILL.md` via a relative link verified by `realpath`. |
| AC3 | Each skill's compaction-recovery callout (where present) continues to resolve all referenced paths after extraction; verified by re-reading the callout block. |
| AC4 | No regression in skill workflow: a representative dry-run of `/task`, `/pr-ci-failed`, `/pr-commented`, `/master-ci-failed`, `/bugfix`, `/interview`, `/ai-audit` (or equivalent re-read pass) confirms every workflow step remains discoverable from `SKILL.md` directly or via a single hop to the extracted file. |
| AC5 | Propagation Rule sync groups remain coherent: `grep -rn "<sync-group-keyword>"` across the moved content + sibling files shows no dangling reference. |
| AC6 | `wc -l` across the 7 in-scope files yields a total reduction matching the reported diff in the PR body (cited number must equal observed number — no "all files reduced" hand-wave). |
| AC7 | All seven per-skill extractions land in a single PR against `master`. No partial / per-skill follow-up PRs unless a documented blocker forces a split (recorded in the PR body if so). |

## Open questions

- Whether `interview/SKILL.md` (227 lines, 12,127 chars) and `bugfix/SKILL.md` (257 lines, 14,095 chars) genuinely benefit from extraction or are best left intact with documented exemptions. Both are already under the 14k-char marker and only marginally over the 200-line target; design agent decides per-skill whether net-positive readability is achievable.
- Whether the `ai-audit/SKILL.md` Phase 2 checklist (the largest single section at 27,878 chars) can be cleanly extracted to `reference.md` while keeping Phase 2 entry navigable from `SKILL.md`. The checklist IS the audit, but each sub-check (A–N + sub-checks 9–10) is independently referenceable. Design agent assesses.
