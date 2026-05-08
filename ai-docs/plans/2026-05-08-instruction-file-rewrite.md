# Instruction-File Rewrite Plan (v4)

**Status:** Phase 0 merged in PR #166; Phase 1 complete (issues #167, #168–#171, #174 — all closed); methodology shakedown. **Read [`## Methodology limitations`](#methodology-limitations) below before re-using this template** — the v4.2 framework is method-development, not strong validation.
**Started:** 2026-05-08
**Style reference:** [`ai-docs/agent-writing-style.md`](../agent-writing-style.md)
**Tracked in:** none (this meta-plan has no single GitHub issue; each Phase 1 file has its own — see table below)

## Goal

Rewrite the workspace's instruction files so that both Opus 4.7 and Sonnet 4.6
reading the same paragraph land on the same interpretation, and stress-test
that interpretation with structured probes. **What this empirically validates
is bounded — see [`## Methodology limitations`](#methodology-limitations).**

Reusable: this plan is intended as a template for future dual-model rewrites
(e.g., Phase 2 procedural skills, or any new instruction-file family). Cite
sections of this doc when a future PR follows the same workflow — and read
the limitations section before claiming a PASS as strong evidence of clarity.

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

## Phase 1 — Rewrite cycle (5 PRs, independent)

Each Phase 1 file has its own GitHub tracking issue and proceeds independently
of the other four — work can be parallelised across sessions without
inter-file sequencing. The numbered "order" below is a **recommended priority**
(highest leverage first), not a sequencing requirement.

### File order — recommended priority

| # | File | Issue | Recommended priority rationale |
|---|---|---|---|
| 1 | `AGENTS.md` | [#167](https://github.com/maratik123/quartzite/issues/167) | Central reference; everything else cites it |
| 2 | `ai-docs/code-style.md` | [#168](https://github.com/maratik123/quartzite/issues/168) | Most rule-dense; multiple decision trees; section-share-aware override expected |
| 3 | `ai-docs/doc-convention.md` | [#169](https://github.com/maratik123/quartzite/issues/169) | Similar density to code-style |
| 4 | `.claude/agents/self-review.md` | [#170](https://github.com/maratik123/quartzite/issues/170) | Reads (1)–(3) — verifies upstream rewrites are effective at agent-checklist depth |
| 5 | `.claude/agents/review-findings.md` | [#171](https://github.com/maratik123/quartzite/issues/171) | Mirrors (4); often updated in lockstep per Propagation Rule |

All five issues are labeled `blocked` until PR #166 (this Phase 0 PR) merges,
since the rewrite cites `ai-docs/agent-writing-style.md` (the style spec) and
this plan doc.

### Cross-reference link sanity (per file)

When a Phase 1 rewrite changes a section anchor, sibling files that link to
the changed anchor break. **Each per-file workflow includes a cross-reference
audit step**:

- Pre-rewrite: `grep -rn "<file>#" ai-docs/ .claude/skills/ .claude/agents/`
  to enumerate every link into this file's anchors
- Post-rewrite: re-grep, verify every old anchor still resolves OR update the
  linker file in the same PR (Propagation Rule applies)

If two files cross-link heavily, the rewrite PRs may opt to bundle a small
follow-up commit updating sibling anchors. That bundling is per-PR judgment;
no plan-level rule forces it.

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

    A section with NO probe is normally out of scope for restyling —
    EXCEPT when its current text directly contradicts
    `ai-docs/agent-writing-style.md` (e.g., uses globs as the entire
    fail-loud list, AXIOM blockquote without an action table, every
    paragraph in caps, negative-only rules without a positive shape, or
    other anti-patterns from that doc). In contradicting cases, fix the
    style violation to match the spec — but the fix is style-correction,
    not new re-emphasis. Cite the specific anti-pattern from
    `agent-writing-style.md` in the commit message.

    Cross-reference touch-ups (link fixes, heading anchors) are always
    allowed.
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
16. Semantic-preservation self-review — spawn an **Opus subagent**:

    ```
    Agent(
      description = "Semantic-preservation review (Opus)",
      subagent_type = "general-purpose",
      model = "opus",
      prompt = <see template in Implementation hints below>,
    )
    ```

    Opus is used here (not the default Sonnet self-review agent) because
    semantic-preservation review benefits from deeper reasoning across
    the diff: cross-rule interactions, subtle meaning shifts, nuance
    loss that Sonnet may miss. The subagent compares OLD
    (`git show <base_commit>:<path>`) vs NEW (HEAD on disk, including
    uncommitted edits) and returns a per-rule verdict: PRESERVED /
    WEAKENED / DROPPED.

    On any WEAKENED or DROPPED finding → revise the rewrite to restore
    the rule, then **re-run the dual-model comprehension test (step 12)**
    since substantive changes invalidate prior CONVERGE.
17. cargo build (sanity)
18. Stage explicit files, commit, push -u, gh pr create — link the file's
    GitHub issue (`Closes #N`)
19. (No inter-file sequencing — other Phase 1 files can be picked up in
    parallel by the same or different sessions)
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

## Implementation hints

Concrete patterns derived during the first execution of the per-file workflow
(issue #167, AGENTS.md rewrite). Future sessions applying this workflow to
other Phase 1 files (#168–#171) can imitate these for consistency.

### Branch naming

Pattern: `chore/<YYYY-MM-DD>-rewrite-<filename-kebab-case>` — drop the file
extension; convert dots and slashes to hyphens.

| File under rewrite | Branch name |
|---|---|
| `AGENTS.md` | `chore/2026-05-08-rewrite-agents-md` |
| `ai-docs/code-style.md` | `chore/2026-05-08-rewrite-code-style` |
| `ai-docs/doc-convention.md` | `chore/2026-05-08-rewrite-doc-convention` |
| `.claude/agents/self-review.md` | `chore/2026-05-08-rewrite-self-review` |
| `.claude/agents/review-findings.md` | `chore/2026-05-08-rewrite-review-findings` |

### Section inventory format

At workflow step 3, build a table with columns: `Section | Lines | Range | Share`.
Compute `Share = section_lines / total_file_lines`. Identify the largest
section — that's the candidate for the heavy-section override (mechanical >50%
or judgment-call rule-dominant) per the coverage rules.

### Cross-reference link audit

Pre-rewrite (after step 4) and post-rewrite (after step 17):

```bash
grep -rn '<file>#' ai-docs/ .claude/skills/ .claude/agents/
```

Pre-rewrite enumerates incoming anchor links. Post-rewrite verifies each
still resolves. Renaming or restructuring a `## Section` heading changes its
GitHub-style anchor (`#section-heading`); preserve the heading or update
siblings in the same PR.

### Probe drafting heuristics

**Class A (failure-targeted, 4–6 probes):**
- One per `learnings.md` misread event traceable to the file under rewrite
- Phrase as a realistic scenario asked as yes/no or what-do-you-do
- Include at least one detail that disambiguates the misread direction
  (e.g., explicitly offer (a)/(b)/(c) options to force a choice)

**Class B (failure-likely, 1–3 probes):**
- Target structural-complexity hot spots: nested conditionals, multi-axis
  decision trees, carve-out exemptions, cross-references requiring synthesis
- Place at least one Class B in a section with zero Class A probes —
  extends coverage beyond hindsight

**Class C (control, 1 probe):**
- Anchor on a stable, low-rule-density section (one that primarily points
  at another doc, or a short orientation paragraph)
- Required answer: a single literal token (file path, command, version)
- Verifies the test setup itself is intact

### Rubric authoring heuristics

For **Class A and B (prose-with-rubric):**

| Field | Items | Rules |
|---|---|---|
| Required-present | 2–4 | Each must be load-bearing (rule cannot be understood without it). Match leniently on synonyms (negation: `no` / `not authorised` / `forbidden` / `MUST NOT` all count). Match strictly on substantives (file paths, command names, specific triggers — verbatim). |
| Required-absent | 1–2 | Concepts that, if present, indicate the rule was misread in a known direction. Avoid common stop-words (don't required-absent `not` or `for example`). |
| Verdict | binary | ALL required-present matched AND NO required-absent matched → CORRECT. Anything else → WRONG. |

**Anti-rubrics — do NOT do:**

- Grade for exact wording (defeats dual-model neutrality)
- List required-present items that are synonyms of each other (graders end up checking the same concept twice)
- Required-absent on natural English phrases (`however`, `for example`)

For **Class C (literal-token):**

- Type: `literal-token`
- Required value: verbatim string from the rewritten file
- Comparison: case-sensitive substring search — model's answer must contain
  the literal value as a substring

### Subagent spawn

Spawn both models **in parallel** — single tool-use message with two `Agent`
calls:

```
Agent(
  description = "Comprehension test (Opus)",
  subagent_type = "general-purpose",
  model = "opus",
  prompt = <prompt template, see below>,
)

Agent(
  description = "Comprehension test (Sonnet)",
  subagent_type = "general-purpose",
  model = "sonnet",
  prompt = <IDENTICAL prompt>,
)
```

Both subagents receive identical prompts including the same shuffled probe
order — divergence is then attributable to the file's wording, not to setup
variance.

### Subagent prompt template

```
You are a comprehension-test subagent for the quartzite project's <FILE>
rewrite (issue #<N>). Your task is narrow and self-contained.

INSTRUCTIONS:

1. Read the file at <ABSOLUTE-PATH> — that file is the rewritten version
   under test.
2. Do NOT read any other file in the codebase.
3. Do NOT search for additional context, run grep, or open the web.
4. Answer each probe below INDEPENDENTLY, based ONLY on what's in the file.
5. Be brief: 1–4 sentences per answer. State the answer first, then briefly
   justify by referring to the relevant section.

OUTPUT FORMAT — for each probe, exactly this shape:

PROBE <N>:
<your answer>

PROBES (randomized order):

Probe 1: <verbatim Q>
Probe 2: <verbatim Q>
...

After answering all <N> probes, output a final line: `END OF PROBES`.
```

**Why each instruction is load-bearing:**

- "Do NOT read other files / grep / web search" — without this, subagents
  read sibling instruction files to "verify" cross-references and
  contaminate the test (model answers based on what *should* be true,
  not what the rewritten file says)
- "1–4 sentences" cap — keeps answers short enough for mechanical
  rubric-checking; longer answers risk burying the verdict-relevant token
- "State the answer first" — forces the verdict signal early; rubric
  matching becomes more reliable
- `END OF PROBES` sentinel — makes truncation detectable

### Probe order randomization

Per round, generate a fresh permutation of the probe set. Both subagents
get the **same** permutation in a given round — different orders across
models would introduce a confounding variable.

If round 2 is needed (some probes failed in round 1), shuffle again. If
round 2 with reshuffled order produces *different* convergence than round 1
on the same probes, that's diagnostic information about order-dependence in
the file's logical flow.

### Semantic-preservation subagent prompt template

Used at workflow step 16. Spawn with `model="opus"` — this review benefits
from deeper reasoning than the dual-model comprehension test does.

```
You are a semantic-preservation review subagent for the quartzite project's
<FILE> rewrite (issue #<N>). Your task is narrow and self-contained.

INSTRUCTIONS:

1. Read the OLD version of the file:
   `git show <BASE_COMMIT>:<RELATIVE-PATH>`
   This is the version before this PR's rewrite.
2. Read the NEW version of the file:
   `<ABSOLUTE-PATH>` (filesystem state — includes uncommitted edits)
   This is the rewritten version under review.
3. Do NOT read any other file (the rewrite is self-contained for this check).
4. Do NOT search for additional context, run grep on the wider tree, or
   open the web.

OBJECTIVE:

For every load-bearing rule present in OLD, verify whether it survives in
NEW. A "load-bearing rule" is anything an agent reading the file would be
expected to act on: prohibitions, requirements, command lists, file path
references, decision-tree branches, exemptions, edge cases.

OUTPUT FORMAT:

| # | Rule (from OLD) | OLD location | Verdict | NEW location |
|---|---|---|---|---|
| 1 | <one-line rule statement> | `<file>:<line>` | PRESERVED / WEAKENED / DROPPED | `<file>:<line>` or `—` |

Then a final summary block:

```
SEMANTIC PRESERVATION VERDICT: <PASS | FAIL>
- N rules in OLD
- M PRESERVED
- K WEAKENED  (must be 0 for PASS)
- L DROPPED   (must be 0 for PASS)
```

Definitions:

- PRESERVED: rule is unambiguously present in NEW with the same or stronger
  binding force (e.g., a paragraph rule promoted to AXIOM is PRESERVED).
- WEAKENED: rule is present but with reduced binding force (e.g., a
  "MUST NOT" softened to "should avoid", or a binary rule converted to a
  guideline). Or the rule lost an essential qualifier (e.g., an exemption
  was kept but its scope was widened/narrowed).
- DROPPED: rule has no equivalent in NEW.

Be skeptical. The rewrite was meant to clarify, not to soften. If a rule
appears in NEW but reads less binding than in OLD, mark it WEAKENED with a
specific reason.

After the table and summary, output a final line: `END OF REVIEW`.
```

**Why each instruction is load-bearing:**

- "Do NOT read other files" — prevents the subagent from cross-referencing
  sibling instruction files and judging based on their content rather than
  the file under review
- Forces a per-rule table — prevents narrative summaries that obscure
  whether specific rules survived
- Three-state verdict (PRESERVED / WEAKENED / DROPPED) is binary at the
  rule level — no partial credit; ambiguity defaults to flag-as-WEAKENED
- The summary's PASS/FAIL gate is explicit — N WEAKENED + L DROPPED must
  both be 0 for PASS; otherwise the parent agent must revise

### Reporting format for dual-model results

For all-CONVERGE rounds, a per-probe summary table is sufficient:

| Probe | Class | Topic | Opus | Sonnet | Verdict |
|---|---|---|---|---|---|
| A1 | failure-targeted | <topic> | CORRECT | CORRECT | ✅ CONVERGE |

For DIVERGE outcomes, add a per-probe detail block:

```
Probe N (Class X — derived from <source>)

Question: <verbatim>

Opus answer: "<verbatim>"
  Rubric check:
    required-present 1: ✓/✗ <reason>
    required-absent  1: ✓/✗ <reason>
  Verdict: CORRECT / WRONG

Sonnet answer: "<verbatim>"
  Rubric check: ...
  Verdict: ...

Convergence: DIVERGE — <one-sentence diagnosis>
Source paragraph: <file>:<line range>
Proposed revision: <text>
```

## Methodology limitations

This section was added after Phase 1 completed (5 files all PASS round 1 with
no iteration). Reflecting on the all-PASS rate honestly: it is **either**
evidence the files were already clear, **or** evidence the test was not
sensitive enough to surface ambiguity. The available evidence does not
distinguish between these. Future rewrites and re-runs should account for
the limitations below.

### What the v4.2 methodology validates

A PASS verdict in v4.2 means: *current methodology found no divergence under
isolation conditions with two correlated models, on probes drafted by the
parent agent that drove the workflow.* It does **not** mean: *the file is
unambiguous in production conditions.*

### Known limitations

| Limitation | What it means | Why it matters | Future improvement |
|---|---|---|---|
| **Context-isolation bias** | Subagent prompts strip context (`Do NOT read any other file`) | Real `self-review` / `review-findings` agents read AGENTS.md, the diff, spec, and design before reaching the rule under test — by which point thousands of tokens have shaped their interpretation. The test answers "is this file clear when read alone?", not "is this file clear under realistic context load?" | Add a context-loaded probe round per file: subagent reads file + sister files (the ones it cross-references) + a synthesised realistic diff that exercises the rule. Convergence on isolation but divergence on context = "rule is clear in the abstract but gets lost in workflow noise." See [`## Phase 2 / future improvements`](#phase-2--future-improvements) below. |
| **Model-correlation bias** | Opus 4.7 and Sonnet 4.6 share training distribution and inductive biases | Identical prompts to two correlated models produce convergent answers via shared bias, not file clarity. A truly independent test needs an out-of-family model (GPT-class, Gemini-class). | **No current access to out-of-family models** in this repo's tooling. Acknowledged limitation, not addressable via methodology change. When the option becomes available, add a third subagent run per file. Until then, weight convergence between Opus + Sonnet as **"two highly correlated reads agree"**, not as "file is unambiguous to readers in general." |
| **Probe-author selection bias** | The parent agent (driving the workflow) drafts the probes after reading the file | Risk that probes are unconsciously phrased to surface the rules I expect to be clear, not the rules at risk. The Gate 1 user-approval step partially counters this but doesn't eliminate it. | Decouple probe-authoring from rewrite-execution: a separate Opus subagent reads the file + learnings + style spec, drafts probes blind to whether the file is "expected" to pass. The rewrite-execution agent (parent) cannot influence probe wording. |
| **Closed-question bias** | Probes are mostly yes/no, multiple-choice, or specific-fact | Closed answers produce convergence easily because the answer space is narrow. Open-ended probes ("explain how rule X applies to scenario Y in your own words") would surface interpretation differences that closed forms hide. | Mix in 1–2 open-ended probes per file. Rubric for open-ended is necessarily looser ("must mention concept A; must not contradict concept B") but divergence on open-ended probes is a stronger ambiguity signal than divergence on yes/no. |
| **No post-validation feedback loop** | After a file passes, we don't track whether real `learnings.md` events keep happening on its rules | The actual test of clarity is "do new misread events on this file's rules occur after validation?" If yes, the test was a false-positive of clarity. Without surveillance, validation is one-shot and unverified. | Post-validation surveillance: monitor `learnings.md` for ~30 days after a file is closed as PASS. New entries targeting that file's rules → add to the file's issue as a re-open trigger. Codify in this plan as a "post-validation surveillance" rule before next use. |
| **Step 16 minimal sample** | Opus semantic-preservation gate (the v4.2 distinctive contribution) fired exactly once across Phase 1 (#174 only — others had empty diffs) | Calling v4.2 "validated across 5 files" overstates evidence. Step 16 has 1 data point. | Run Step 16 explicitly on every Phase 2 file even when the diff seems trivially-preserving. Track Opus self-review verdicts across runs to build a track record on whether Opus catches subtle weakening Sonnet would miss. |
| **All-PASS-round-1 rate is a smell** | 45/45 probe convergences across 5 files, no iteration triggered | When every test passes on round 1 with no iteration, it's either (a) files genuinely clean, or (b) test not sensitive enough. Priors should split between these; Phase 1's results don't update strongly toward (a). | Add the calibration probe (below). If models converge confidently on a calibration-probe wrong answer, the test setup itself is converging on biases — strong signal that other PASS results are suspect. |
| **No calibration probe** | No probe with intentionally-ambiguous file content | Without one, we cannot detect when convergence is the test's bias rather than the file's clarity. | Per file, include 1 calibration probe whose answer is **intentionally underspecified** in the file (a rule the file doesn't actually pin down). Both models converging on the same confident answer = setup itself is converging on training-data biases. Calibration failure = strong signal that PASS results from other probes are suspect. |

### Honest framing in PR / issue closing comments

When a file passes v4.2, the closing comment should say:

> The v4.2 methodology found no divergence on this file under the
> documented test conditions. This validates clarity-in-isolation against
> correlated readers, not clarity-in-production-context. See
> [`## Methodology limitations`](../plans/2026-05-08-instruction-file-rewrite.md#methodology-limitations).

It should NOT say "the file is solid" or "the file is unambiguous". The
Phase 1 closing comments (#168, #169, #170, #171) do use the stronger
phrasing — those wordings overstate what the methodology proves.

### Phase 2 / future improvements

When v4.2 is next applied (Phase 2 procedural skills, or a new instruction
file added later), the workflow should be amended to address the limitations
above. Concrete changes for next use:

1. **Two probe rounds per file** — isolation round (current) + context round
   (file + sister files + synthesised realistic diff).
2. **Decoupled probe-author** — separate Opus subagent drafts probes, parent
   agent runs the workflow.
3. **Mix in open-ended probes** — 1–2 per file, looser rubric.
4. **Add a calibration probe** — 1 per file, intentionally underspecified.
5. **Post-validation surveillance** — monitor `learnings.md` ~30 days
   post-PASS; new entries trigger re-open.
6. **Step 16 always fires** — even on empty diffs (returns trivial PASS but
   builds the run record).
7. **Honest framing** — closing comments distinguish "no divergence under
   test conditions" from "file is unambiguous in production".

Out-of-family model is **aspirational, not currently feasible** — no API
access in this repo's setup at present. When the tooling supports it, add
as an 8th change (third subagent run per file).

### Phase 1 retrospective summary

| Aspect | Verdict |
|---|---|
| Probe + rubric + structured-answer framework | ✅ Genuine improvement over no testing |
| Style-violation scan (step 9) | ✅ Mechanical, cheap, catches real anti-patterns |
| Heavy-section override mechanism | ✅ Worked correctly on `code-style.md` |
| Cross-reference link audit | ✅ Mechanical, would have caught real anchor breakage |
| Probe-design lessons (avoid symmetric multi-choice; confusion-traps in required-absent) | ✅ Concrete portable knowledge |
| Issue-tracking decoupled from PRs (closing comments as record) | ✅ Future readers can audit |
| Strong validation of file clarity | ❌ **Overstated** — see limitations table above |
| Step 16 (Opus self-review) cross-run track record | ❌ Only one data point (#174) |
| Detection of methodology biases | ❌ Not built in — discovered post-hoc |

Phase 1 was a **methodology shakedown** that produced good infrastructure
and concrete lessons. The infrastructure is reusable; the file-clarity
claims are weaker than the closing comments suggest.

## Decision history

- v1 — initial sketch: probes after rewrite (test-after); 2 rounds; flat
  anti-clustering. Rejected: rewrite has no concrete success criterion.
- v2 — added 3 probe classes (A failure-targeted / B failure-likely / C
  control), but still test-after. Rejected: same flaw.
- v3 — test-driven (probes BEFORE rewrite); 2 approval gates per file;
  iteration cap = 3; random ordering. Rejected on anti-clustering rule:
  flat 40% cap doesn't work for files with one rule-dominant section.
- v4 — adds section-share-aware anti-clustering (mechanical override > 50%
  by lines, judgment-call override for rule-dominant sections); adds
  probe-surface coverage requirement (≥ 60% sections targeted) and Class B/C
  placement constraints.
- v4.1 — each Phase 1 file gets its own GitHub tracking issue (#167–#171);
  Phase 1 PRs are now independent rather than sequential; the file order in
  the table is a recommended priority, not a sequencing requirement. Adds
  per-file cross-reference link-sanity audit step (pre-rewrite grep +
  post-rewrite re-grep) so anchor breakages are caught in the same PR.
- v4.2 — two refinements informed by the #167 execution:
  - Step 9 — unprobed sections are not strictly out-of-scope. They may be
    rewritten when the current text contradicts
    `ai-docs/agent-writing-style.md` (e.g., globs as the entire fail-loud
    list, AXIOM without action table, every paragraph in caps, negative-
    only rules without a positive shape). Style-correction is allowed;
    new re-emphasis without a probe is not.
  - Step 16 — semantic-preservation self-review now spawns an **Opus
    subagent** rather than running inline. The model upgrade addresses
    cross-rule reasoning depth that the parent agent (or a Sonnet
    subagent) may not consistently apply. New subsection in Implementation
    hints provides the prompt template.
- v4.3 — current. Methodology-limits retrospective added after Phase 1
  completed (5 files all PASS round 1, 0 iterations). Documents the gap
  between "what v4.2 actually validates" and "what the closing comments
  imply." Eight known limitations enumerated (context-isolation bias,
  model-correlation bias, probe-author selection bias, closed-question
  bias, no post-validation feedback loop, Step 16 minimal sample,
  all-PASS-round-1 smell, no calibration probe). Phase 2 / future
  improvements list 7 concrete addressable changes; out-of-family model
  noted as aspirational (no current tooling access). Status banner at
  the top of this doc and the Goal section reframed to direct readers
  at the limitations section before re-using the template. The v4.2
  framework remains useful infrastructure; the validation claims it
  produces are now appropriately bounded.
