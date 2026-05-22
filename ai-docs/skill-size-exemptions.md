# Skill Size Exemptions

Audited list of `.claude/skills/*/SKILL.md` files that exceed the 200-line soft
target documented in `ai-docs/code-style.md § File size` but are **load-bearing**
— the residue is workflow-time material (cat-3) or per-round template scaffolding
(cat-2) that must remain in the SKILL body to fire on every invocation.

Each entry records the SKILL path, `wc -l` and `wc -c` at audit time
(post-strip; values are the file shape after PR #502 removed the inline
`<!-- size-exemption: ... -->` comments), the load-bearing reason paraphrased
from the original inline comment so the category vocabulary is preserved, and
the category map (cat-2 / cat-3 per the #498 spec § Scope vocabulary; cat-1
absent by definition — anything cat-1 would have been extracted).

**Consumers.**

- `/ai-audit` Checklist K item 1 (today) — reads this file for the exemption
  list, runs `wc -l` against the live tree, and emits a `minor` finding when
  (a) a SKILL > 200 lines is NOT in the index ("oversized + no exemption") or
  (b) a SKILL listed here drifts from its cited `wc -l` ("`<path>`: index
  cites X lines, live is Y lines").

**Entry-removal rule.** When a SKILL drops to ≤ 200 lines, **delete** its entry
from this file in the same commit that triggered the shrink. The index records
exemptions; the predicate "entry exists" is equivalent to "exemption applies".
A drift to ≤ 200 lines without an index update is itself a Checklist K item 1
finding.

---

## Active entries

### `.claude/skills/master-ci-failed/SKILL.md`

| Field | Value |
|---|---|
| **SKILL path** | `.claude/skills/master-ci-failed/SKILL.md` |
| **wc -l at audit time** | 373 |
| **wc -c at audit time** | 22,885 |
| **Load-bearing reason** | Load-bearing residue after extraction: compaction-recovery callout (cat-3) + Workflow Steps 0–9 narrative (cat-3) + Step 1 progress-file schema (cat-2 round-template scaffolding the skill writes into during each round) + Step 2 log-fetch / classification / reproducer tables (cat-3 workflow-time, consulted every Step-2 execution). |
| **Category map** | cat-2 round-template scaffolding (Step 1 progress-file schema) + cat-3 workflow narrative (Steps 0–9, compaction-recovery callout) + cat-3 workflow-time tables (Step 2 log-fetch / classification / reproducer). |

### `.claude/skills/pr-ci-failed/SKILL.md`

| Field | Value |
|---|---|
| **SKILL path** | `.claude/skills/pr-ci-failed/SKILL.md` |
| **wc -l at audit time** | 326 |
| **wc -c at audit time** | 22,320 |
| **Load-bearing reason** | Load-bearing residue after extraction: compaction-recovery callout (cat-3) + Workflow Steps 0–9 narrative (cat-3) + Step 1 round-section template (cat-2 round-template scaffolding) + Step 2 log-fetch / classification / reproducer + fallback bash (cat-3 workflow-time). |
| **Category map** | cat-2 round-template scaffolding (Step 1 round-section template) + cat-3 workflow narrative (Steps 0–9, compaction-recovery callout) + cat-3 workflow-time content (Step 2 log-fetch / classification / reproducer + fallback bash). |

### `.claude/skills/pr-commented/SKILL.md`

| Field | Value |
|---|---|
| **SKILL path** | `.claude/skills/pr-commented/SKILL.md` |
| **wc -l at audit time** | 305 |
| **wc -c at audit time** | 21,419 |
| **Load-bearing reason** | Load-bearing residue after extraction: compaction-recovery callout (cat-3) + Workflow Steps 0–7 narrative (cat-3) + Step 0 GraphQL + REST snapshot recipes (cat-3 workflow-time, every invocation) + Step 1 round-section template (cat-2 round-template scaffolding) + Step 2 classification table + pause-trigger list (cat-3 workflow-time, every Step-2 per-thread loop). |
| **Category map** | cat-2 round-template scaffolding (Step 1 round-section template) + cat-3 workflow narrative (Steps 0–7, compaction-recovery callout) + cat-3 workflow-time content (Step 0 GraphQL + REST snapshot recipes, Step 2 classification table + pause-trigger list). |

### `.claude/skills/bugfix/SKILL.md`

| Field | Value |
|---|---|
| **SKILL path** | `.claude/skills/bugfix/SKILL.md` |
| **wc -l at audit time** | 257 |
| **wc -c at audit time** | 14,095 |
| **Load-bearing reason** | No category-(1) content; workflow narrative Steps 2–7 (cat-3) + Step 1 trace-file template (cat-2 round-template scaffolding) + Step 6.5 self-review prompt block (cat-3) all load-bearing. |
| **Category map** | cat-2 round-template scaffolding (Step 1 trace-file template) + cat-3 workflow narrative (Steps 2–7) + cat-3 workflow-time content (Step 6.5 self-review prompt block). |

### `.claude/skills/interview/SKILL.md`

| Field | Value |
|---|---|
| **SKILL path** | `.claude/skills/interview/SKILL.md` |
| **wc -l at audit time** | 242 |
| **wc -c at audit time** | 13,882 |
| **Load-bearing reason** | No category-(1) content; Workflow Steps 1–4 narrative (cat-3) + State file YAML template (cat-2 round-template scaffolding) + Round/question caps table (cat-3) all load-bearing. |
| **Category map** | cat-2 round-template scaffolding (State file YAML template) + cat-3 workflow narrative (Steps 1–4) + cat-3 workflow-time tables (Round/question caps table). |

### `.claude/skills/task/SKILL.md`

| Field | Value |
|---|---|
| **SKILL path** | `.claude/skills/task/SKILL.md` |
| **wc -l at audit time** | 228 |
| **wc -c at audit time** | 26,707 |
| **Load-bearing reason** | Residue is cat-(3) workflow (four ⚡ entry-routing preambles fire before any tool call + Step 8 every-group handoff + Step 12 commit/PR sub-steps); cat-(1) candidates already moved to `reference.md` + `inbox-propagation.md` (category-4 prior extraction). |
| **Category map** | cat-3 workflow narrative (four ⚡ entry-routing preambles, Step 8 every-group handoff, Step 12 commit/PR sub-steps). |

---

## Notes

- **Supersedes #498 spec AC1.** The original "in-file `<!-- size-exemption: ... -->`
  comment" contract from PR #501 (closing issue #498) is replaced wholesale by:
  *"an entry in `ai-docs/skill-size-exemptions.md` OR the file is ≤ 200 lines"*.
  The #498 spec under `ai-docs/plans/done/` is **not** edited — the supersede
  claim is recorded forward here, not retroactively against done-tree history.
- **Entry-removal rule.** When a SKILL drops to ≤ 200 lines (e.g. via a further
  extraction pass), **delete** its entry from this file in the same commit that
  triggered the shrink. The index records exemptions; "entry exists" ⇔
  "exemption applies". A SKILL at ≤ 200 lines listed here is itself a
  Checklist K item 1 drift finding.
