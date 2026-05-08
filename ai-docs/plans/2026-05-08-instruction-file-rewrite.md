# Instruction-File Rewrite Plan (v4)

**Status:** in progress (Phase 0 lands in PR #166)
**Started:** 2026-05-08
**Style reference:** [`ai-docs/agent-writing-style.md`](../agent-writing-style.md)
**Tracked in:** none (multi-PR workflow plan, no single GitHub issue)

## Goal

Rewrite the workspace's instruction files so that both Opus 4.7 and Sonnet 4.6
reading the same paragraph land on the same interpretation. Test the result
empirically with a dual-model comprehension probe per file.

Reusable: this plan is intended as a template for future dual-model rewrites
(e.g., Phase 2 procedural skills, or any new instruction-file family). Cite
sections of this doc when a future PR follows the same workflow.

## Scope

### In scope (Sonnet + Opus dual-readability)

| File | Lines |
|---|---|
| `AGENTS.md` | 220 |
| `ai-docs/code-style.md` | 409 |
| `ai-docs/doc-convention.md` | 429 |
| `.claude/agents/self-review.md` | 161 |
| `.claude/agents/review-findings.md` | 152 |

Total: ~1,371 lines across 5 files (Phase 1).

### Out of scope (Opus-only readers)

- Agents with `model: opus` frontmatter:
  - `.claude/agents/design.md`
  - `.claude/agents/design-review.md`
  - `.claude/agents/learnings-escalation-audit.md`
  - `.claude/agents/self-improve.md`
- Skills that run in Opus mode:
  - `.claude/skills/ai-audit/SKILL.md`
  - `.claude/skills/improve/SKILL.md`

For these readers, AXIOM/fail-loud styling is optional — the reader has the
context to disambiguate without it.

### Deferred (Phase 2, not committed)

Procedural skills with workflow steps but low binary-rule density:

- `.claude/skills/bugfix/SKILL.md`
- `.claude/skills/code-review/SKILL.md`
- `.claude/skills/task/SKILL.md`
- `.claude/skills/interview/SKILL.md`
- `.claude/skills/next/SKILL.md`
- `.claude/skills/pr-merged/SKILL.md`
- `.claude/skills/context-reset/SKILL.md`
- `.claude/skills/verify/SKILL.md`

No commitment to revisit after Phase 1 unless Phase 1 produces a specific
reason to.

---

## Phase 0 — Style spec (1 PR, complete in PR #166)

Land [`ai-docs/agent-writing-style.md`](../agent-writing-style.md) as the
citable reference for binary-rule writing style. After merge, all subsequent
PRs cite this doc as the standard.

---

## Phase 1 — Rewrite cycle (5 PRs, sequential)

### File order

| # | File | Why this order |
|---|---|---|
| 1 | `AGENTS.md` | Central reference; everything else cites it |
| 2 | `ai-docs/code-style.md` | Most rule-dense; multiple decision trees |
| 3 | `ai-docs/doc-convention.md` | Similar density to code-style |
| 4 | `.claude/agents/self-review.md` | Reads (1) and (2) — verifies they're effective |
| 5 | `.claude/agents/review-findings.md` | Same shape as (4) |

### Per-file workflow

```
1.  Branch off master:                  chore/<date>-rewrite-<filename>
2.  Read file end-to-end
3.  Build the SECTION INVENTORY:
      - list every top-level (##) heading
      - record line ranges and per-section share of total file lines
4.  Cross-reference ai-docs/learnings.md → list every misread event
    traceable to this file (quoted offending paragraph + line range)
5.  Identify failure-likely hot spots (nested conditionals, decision
    trees, carve-outs, cross-references) with line ranges
6.  Draft probes:
      - Class A (failure-targeted)  : 4-6 probes, drawn from step 4
      - Class B (failure-likely)    : 1-3 probes, drawn from step 5
    For each probe: question, full rubric, target section(s).
7.  Build the COVERAGE MAP — table showing which top-level section
    each probe targets. Verify the coverage rules (below) hold;
    if not, draft additional Class B probes until they do.
8.  ===> APPROVAL GATE 1:
        - probe set (A + B)
        - rubrics
        - coverage map (with section-share-aware override declarations
          where applicable)
      User accepts / amends / rejects BEFORE rewrite. <===
9.  Rewrite with the approved probe set + rubrics open in view.
    Each fail-loud edit is justified by which probe(s) it makes pass.
    A section with NO probe is OUT OF SCOPE — do not rewrite.
    Cross-reference touch-ups (link fixes, heading anchors) are allowed
    but no fail-loud restyling of unprobed sections.
10. Draft Class C probe — single literal-token control, anchored on a
    stable section of the rewritten file
11. ===> APPROVAL GATE 2: Class C probe (small gate) <===
12. Randomize all probes (A+B+C), spawn dual-model parallel test:
      - Subagent A: model="opus",   prompt="Read file. Answer probes 1..N."
      - Subagent B: model="sonnet", same prompt
13. Evaluate per the rubric (mechanical, per-probe CORRECT/WRONG)
14. If all probes CONVERGE on CORRECT → step 16
    Else → step 15
15. Revise the failing-probe sections, re-randomize order, re-run.
    Cap: 3 rounds. After round 3 if still failing → surface to user
    with diagnosis + proposed revision.
16. Semantic-preservation self-review (every old rule still in new file)
17. cargo build (sanity)
18. Stage explicit files, commit, push -u, gh pr create
19. Wait for user merge before next file
```

---

## Probe composition (per file)

Each comprehension test contains three classes of probe, randomly interleaved:

### Class A — Failure-targeted (4–6 probes)

Questions derived from documented misread events in `ai-docs/learnings.md`.
One probe per learnings entry that traces back to the file under rewrite.
These are the rules where ambiguity has *already* cost a session.

**Example (AGENTS.md):**

> The user says "add this to learnings: \<some rule\>". You write the entry.
> Are you authorised to also update AGENTS.md in the same turn? Yes / No, and
> why?

(Targets entry L13 — "on corrections, write to learnings only".)

### Class B — Failure-likely (1–3 probes)

Questions targeting complex/non-trivial logic in the file that *hasn't* failed
yet but has the structural shape that produces dual-model divergence: nested
conditionals, multi-axis decision trees, carve-out exemptions, cross-references
requiring synthesis.

**Example (code-style.md decision tree):**

> A method `fn foo(&self) -> u32 { self.field }` lives inside `impl<T> Trait
> for Foo<T>`. The body has no branches, no loops, and one trivial field
> access. Which marker does it carry, and which form (`/// _Simple._` /
> `// _Simple._` / `#[inline]`)? Why?

(Targets the carve-out boundary that's structurally tricky but hasn't failed
yet.)

### Class C — Control / A-B sanity (1 probe)

A read-the-file-literally question whose answer is a single explicit token
from the rewritten file. Serves as a control: both models should always
answer this identically. If they diverge on the control, the test setup
itself is broken (model didn't read, model misread the prompt) — not the
file.

**Example (AGENTS.md):**

> Per AGENTS.md § *Project*, what file should you read on demand for project
> purpose, entities, architecture, and design decisions? Quote the path
> verbatim.

(Answer: literally `ai-docs/context.md`. If models give different answers,
throw out the test run and re-spawn — file isn't to blame.)

### Random ordering

Each iteration shuffles probe order. Detects order-dependence (where a
model's answer to probe N is biased by what it saw in probe N-1). If round 2
with reshuffled order produces a *different* outcome than round 1 with the
same probes, that's diagnostic information about ordering effects in the
file's logical flow.

### Total per file

| Class | Count |
|---|---|
| A — Failure-targeted | 4–6 |
| B — Failure-likely | 1–3 |
| C — Control | 1 |
| **Total** | **6–10** |

---

## Probe-surface coverage rules

Probes must demonstrate the file is comprehensible end-to-end, not just at
known-bad spots.

### Hard requirements (checked at Gate 1)

| Rule | Threshold |
|---|---|
| Section coverage | At least **60% of top-level (`##`) sections** are targeted by at least one probe |
| Anti-clustering (default) | No more than **40% of probes** target a single section |
| Anti-clustering (heavy-section override) | If a section's line range is > 50% of the file (mechanical) OR is judged rule-dominant at Gate 1 (judgment-call), the per-section cap relaxes to `min(section_share + 10%, 70%)` |
| Class B placement | At least **one Class B probe** targets a section that has zero Class A probes |
| Class C placement | The control probe's anchor section must NOT be a section targeted by any Class A or B probe |

### Section-share-aware anti-clustering — two qualifying paths

**(a) Mechanical override** — section line range / total file line range > 50%.
Auto-detected during Section Inventory (step 3 above).

**(b) Judgment-call override** — section is ≤ 50% by lines but rule-dense to
the point that its share of the rules worth probing exceeds 50%. Invoking (b)
requires an explicit one-line justification at Gate 1 (e.g., "section is 30%
by lines but contains 4 of 5 candidate rules with documented misreads + 2 of
3 candidate failure-likely hot spots — declaring rule-dominant").

### Worked examples

**Example 1 — small variation (mechanical override)**

```
File: hypothetical-rule-doc.md (300 lines)
Section X: lines 100-280 = 180 lines = 60% of file → mechanical override

Cap on Section X:           min(60% + 10%, 70%) = 70%
Cap on every other section: 40%

If 8 probes total: up to 5 in Section X (62.5%), at least 3 spread elsewhere.
```

**Example 2 — code-style.md (judgment-call override)**

```
File: ai-docs/code-style.md (~409 lines)
Section "#[inline] and the _Simple._ doc tag": lines 116-238 = 30% by lines

NOT > 50% by lines → no mechanical override.

Justification at Gate 1: "Section is 30% by lines but contains the entire
marker-form decision tree (5 rows), the trait-method carve-out, the codegen
mirroring rule, and the marker-maintenance cascade. Of 5 candidate Class A
probes from learnings.md, 4 trace here. Of 3 candidate Class B hot spots,
2 are inside this section. Declaring rule-dominant."

Cap on this section (judgment-call):  60% (declared share)
If 8 probes total: up to 5 in this section, at least 3 spread elsewhere.
```

**Example 3 — flat file, no override**

```
File: hypothetical-flat.md (200 lines, 5 ## sections, ~40 lines each)
No section qualifies → default 40% cap applies.
If 8 probes total: max 3 per section.
```

### Coverage map deliverable at Gate 1

Surfaced as a table:

```
Section                               | Share | A probes  | B probes | C  | Cap | OK?
--------------------------------------|-------|-----------|----------|----|-----|----
Scope                                 | 3%    | -         | -        | -  | 40% | ✓
Source files                          | 2%    | -         | -        | -  | 40% | ✓
...
#[inline] and _Simple._ doc tag       | 30%   | A1,A2,A3,A4 | B1,B2  | -  | 60% | ✓ (judgment override)
...

Coverage:                7 / 11 sections targeted (64%) → PASS (≥ 60%)
Heaviest single section: 6/10 = 60% in #[inline] section → PASS (cap 60%)
Coverage rule:           PASS
```

The override declaration is part of Gate 1 — user sees the justification
text, agrees or pushes back.

---

## Rubric-based evaluation

Comparison is rubric-based, not prose-vs-prose. Each model's answer is
graded independently against the probe's pre-defined answer key. Wording
deviation between two correct answers is irrelevant.

### Probe answer key (per probe)

#### Class C (control) — exact-token match

```
Type: literal-token
Required value: <verbatim string from the file>

Comparison: case-sensitive substring search for the literal value
in each model's answer.
```

#### Class A and B — required-present / required-absent rubric

```
Type: prose-with-rubric

Required-present (each must appear in some form):
  1. <key concept 1>
  2. <key concept 2>

Required-absent (none of these may appear):
  1. <forbidden concept 1>
  2. <forbidden concept 2>

Verdict logic:
  - All required-present matched AND no required-absent matched → CORRECT
  - Anything else → WRONG
```

### Convergence verdict (across both models)

After both models produce answers and each is independently graded:

| Opus | Sonnet | Verdict |
|---|---|---|
| CORRECT | CORRECT | **CONVERGE** — file is clear on this rule |
| WRONG | WRONG | **DIVERGE** — rule is unclear to both models |
| CORRECT | WRONG | **DIVERGE** — rule reads differently to Sonnet |
| WRONG | CORRECT | **DIVERGE** — rule reads differently to Opus |

Both-CORRECT counts as CONVERGE *regardless of how their wording differs.*

### Reporting format

For each probe, after the round completes:

```
Probe N (Class A, derived from L13)

Question: <verbatim>

Opus answer: "<verbatim>"
  Rubric check:
    required-present 1: ✓ matched
    required-present 2: ✓ matched
    required-absent  1: ✓ none
    required-absent  2: ✓ none
  Verdict: CORRECT

Sonnet answer: "<verbatim>"
  Rubric check:
    required-present 1: ✓ matched
    required-present 2: ✓ matched
    required-absent  1: ✓ none
    required-absent  2: ✗ <forbidden concept> present
  Verdict: WRONG

Convergence: DIVERGE — Sonnet read the rule as admitting a carve-out
that does not exist. Source paragraph: <file>:<line range>.
Diagnosis: <text>. Proposed revision: <text>.
```

### Edge cases

| Case | Handling |
|---|---|
| Model gives correct answer + adds a wrong qualifier | Required-absent hit → WRONG |
| Both correct but verbose vs terse | Both CORRECT → CONVERGE |
| Model answers in list, other in prose | Format irrelevant — rubric scans concepts |
| Model partially answers (1/2 required-present) | WRONG — partial credit isn't enough |
| Model adds extra correct context not asked for | No required-absent hit → CORRECT |
| Rubric itself too lenient (catches false-CORRECT) | Caught at PR review or future learnings; tighten rubric next round |
| Rubric itself too strict (false-WRONG) | Caught when clearly-equivalent answer judged WRONG; amend rubric on the spot, re-evaluate |

### What is NOT used

- **No third LLM judge.** A judge subagent would itself have model-shape
  bias (an Opus judge would prefer Opus-shaped answers, vice versa) —
  defeating the dual-model neutrality. The driver evaluates against the
  rubric mechanically; the user signs off on the rubric.
- **No semantic-similarity scoring** (cosine/embedding distance). Too noisy
  at the granularity needed; would obscure substantive divergences.
- **No partial-credit grading.** Probes are binary CORRECT/WRONG.

---

## Approval gates

| Gate | When | Deliverables |
|---|---|---|
| Phase 0 PR | After style-spec draft | Approve / amend / reject `agent-writing-style.md` content |
| **Gate 1** (per file, before rewrite) | After step 7 of per-file workflow | Probes A+B, rubrics, coverage map (with override declarations) |
| **Gate 2** (per file, after rewrite) | After step 10 of per-file workflow | Class C probe + answer |
| Iteration-cap escalation (per file) | After round 3 of comprehension test fails | Diagnosis, divergent answers, proposed revision |
| Per-file PR | After self-review passes | Standard GitHub PR review |

---

## Iteration cap = 3

Per file, max 3 comprehension rounds. After round 3 if still diverging on
Class A or B, surface to user with:

- All three rounds' answers (Opus + Sonnet, side-by-side)
- The source paragraph
- Diagnosis: ordering effect (round 1 vs 2 vs 3 differed)? Persistent ambiguity
  (all rounds same wrong answer)?
- Proposed fourth revision

User decides: accept proposed revision, manually rewrite that section,
abandon fail-loud-ify of that section, or pause file pending judgment.

Class C divergence does NOT count against the cap — it's a setup-broken
signal, not a file-broken signal. Re-spawn subagents instead.

---

## Phase 2 — deferred

Procedural skills (`bugfix`, `code-review`, `task`, `interview`, `next`,
`pr-merged`, `context-reset`, `verify`) NOT in scope. No commitment to
revisit after Phase 1.

---

## Effort estimate

| Phase | Per-file effort | Total |
|---|---|---|
| Phase 0 (style spec) | 30–60 min single PR | ~1 hour |
| Phase 1 per file | 120–240 min (read, inventory, learnings cross-ref, probe drafting + rubrics + coverage map, Gate 1, rewrite, Gate 2, dual-model test, up to 3 iterations, self-review, fix, commit, PR) | ~10–20 hours across 5 files |
| **Total** | | **~11–21 hours of session time, spread across 6 PRs over multiple days** |

---

## Citation in future PRs

When a Phase 1 PR follows this workflow, cite this doc in its body:

> Per `ai-docs/plans/2026-05-08-instruction-file-rewrite.md` § Phase 1 file N,
> this PR rewrites `<file>` using probes drafted at Gate 1 (commit `<sha>`)
> and the section-share-aware coverage rule.

When a future rewrite plan reuses this template (e.g., Phase 2 procedural
skills, or a different instruction-file family), cite this doc as the source
template:

> Workflow patterned on `ai-docs/plans/2026-05-08-instruction-file-rewrite.md`,
> adapted for procedural skills as follows: \<deltas\>.

---

## Decision history

- v1 — initial sketch: probes after rewrite (test-after); 2 rounds; flat
  anti-clustering. Rejected: rewrite has no concrete success criterion.
- v2 — added 3 probe classes (A failure-targeted / B failure-likely / C
  control), but still test-after. Rejected: same flaw.
- v3 — test-driven (probes BEFORE rewrite); 2 approval gates per file;
  iteration cap = 3; random ordering. Rejected on anti-clustering rule:
  flat 40% cap doesn't work for files with one rule-dominant section.
- v4 — current. Adds section-share-aware anti-clustering (mechanical
  override > 50% by lines, judgment-call override for rule-dominant
  sections); adds probe-surface coverage requirement (≥ 60% sections
  targeted) and Class B/C placement constraints.
