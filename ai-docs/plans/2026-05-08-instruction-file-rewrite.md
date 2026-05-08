# Instruction-File Rewrite Plan (v4)

**Status:** Phase 0 merged in PR #166; Phase 1 complete (issues #167, #168–#171, #174 — all closed); v4.4 replay batch validated 6/6; v5 retrofit bakes lessons into prescriptive workflow.

**Workflow version: v5.** Phase 2 / new-file applications follow v5 by default. v4.x retrospective sections ([`## Methodology limitations`](#methodology-limitations), [`## Historical replay testing`](#historical-replay-testing-complement-to-v42)) remain as the rationale for v5's design — read them when applying the workflow to understand WHY each step is structured the way it is.
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
6.  Draft probes — Classes A, B, D (Class C is drafted later, in step 10,
    because it anchors on a stable section of the *rewritten* file and
    cannot be drafted before the rewrite exists):
      - Class A (failure-targeted)  : 4-6 probes, drawn from step 4
      - Class B (failure-likely)    : 1-3 probes, drawn from step 5
      - Class D (calibration)       : 1 probe — anchored on a topic the
        file mentions but does NOT actually pin down (a deliberate
        non-coverage). Class D **always runs** as 1 probe per file. Its
        verdict logic has three outcomes (handled in step 13), and one
        of those outcomes (both models confidently converge on the same
        answer) flags the test setup as SETUP-SUSPECT — but Class D
        itself is unconditionally part of every test run.
      - **Open-ended requirement:** at least 1-2 of the A+B probes must be
        open-ended ("explain in your own words how rule X applies to Y") —
        not yes/no, not multiple-choice. Closed forms produce convergence
        too easily; open-ended forms surface interpretation differences that
        closed forms hide.
    For each probe: question, full rubric, target section(s).
7.  Build the COVERAGE MAP — table showing which top-level section
    each probe targets. Verify the coverage rules (below) hold;
    if not, draft additional Class B probes until they do.
8.  ===> APPROVAL GATE 1:
        - probe set (A + B + D)
        - rubrics
        - open-ended probes flagged (at least 1-2 must be present)
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
12. Randomize all probes (A+B+C+D), spawn dual-model parallel test:
      - Subagent A: model="opus",   prompt="Read file. Answer probes 1..N."
      - Subagent B: model="sonnet", same prompt
13. Evaluate per the rubric (mechanical, per-probe CORRECT/WRONG).
    **Class D verdict logic** has three outcomes:
      - both models hedge / acknowledge the file's silence → ✅ healthy
      - both models confidently converge on the same answer → ⚠️ SETUP-SUSPECT — flag the entire run; the test bias may have masked real ambiguity in A/B/C; surface to user
      - models diverge (one says X, other says Y) → 🟡 expected (rule is genuinely ambiguous; either is defensible)
    Class D failure does NOT count against the iteration cap — it's
    setup signal, not file-clarity signal.
14. **Branch on convergence:**
      - All probes (A+B+C) CONVERGE on CORRECT AND Class D didn't
        trigger SETUP-SUSPECT → step 16
      - Otherwise → step 15
15. Revise the failing-probe sections, re-randomize order, **loop back
    to step 12**. Cap: 3 rounds. After round 3 if still failing → surface
    to user with diagnosis + proposed revision.
16. **Historical replay** — if `learnings.md` has documented misread
    events on this file's rules, run **at least one replay case** (per
    § Historical replay testing below) before declaring PASS. Connects
    test signal to ground truth. If no documented misreads exist for
    this file (e.g., a brand-new instruction file), skip with a one-line
    note in the closing comment — replay is not feasible without ground
    truth.

    If replay produces a false negative on the documented misread → the
    v5 PASS verdict is suspect; surface to user, sharpen the relevant
    rule, **loop back to step 12** for dual-model + step 16 replay until
    both pass.
17. Semantic-preservation self-review — spawn an **Opus subagent**:

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

    **v5: always fire, even on empty diff.** When the rewrite step (9)
    produced no edits, the Opus subagent returns trivial PRESERVED on
    all rules — that's still a valuable run-record entry. Skipping
    this step on empty diff (as Phase 1 did for #168–#171) leaves it
    with a sample size of 1 — not enough to build a track record on
    whether Opus catches subtle weakening Sonnet would miss.
18. cargo build (sanity)
19. Stage explicit files, commit, push -u, gh pr create — link the file's
    GitHub issue (`Closes #N`).
    **v5: closing comment uses honest framing template** (see Implementation
    hints) — distinguishes "no divergence under test conditions" from "file
    is unambiguous in production". If replay (step 16) was run, include
    its results as positive evidence; if not run, note explicitly.
20. (No inter-file sequencing — other Phase 1 files can be picked up in
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

### Class D — Calibration / setup-bias (1 probe) — v5

A probe whose answer is **intentionally underspecified** in the file — a rule
the file does NOT actually pin down, where any of several answers could
reasonably be defended. If both models confidently converge on the *same*
answer to a deliberately-ambiguous question, the test setup itself is
converging (training-data biases, prompt-shape biases) — strong signal that
the v5 PASS verdicts on this run are suspect.

**Why:** v4.2's all-PASS-round-1 result across 5 files is either evidence the
files were genuinely clear OR evidence the test isn't sensitive enough.
Without a calibration probe we can't distinguish. Class D distinguishes.

**Example (AGENTS.md):**

> Per AGENTS.md, when a fn has both a `_Simple._` doc tag AND a `# Panics`
> doc section, what's the recommended order of the two within the doc
> comment, and why?

(File doesn't actually specify this — `_Simple._` ordering vs `#`-headings
isn't pinned down. Reasonable answers include "_Simple._ before all `#`
headings", "_Simple._ after `# Examples`", or "no fixed rule". If both
Opus and Sonnet confidently say the same thing without hedging, that's
training-data convergence, not file clarity.)

**Verdict logic:**
- Both models hedge / acknowledge the file doesn't specify → ✅ healthy (file is honest about what it does and doesn't pin down; setup is reading correctly)
- Both models confidently converge on the same answer → ⚠️ SETUP-SUSPECT — flag the run; the test bias may have masked real ambiguity in A/B/C
- Models diverge → 🟡 expected (the rule is genuinely ambiguous; either is defensible)

Class D failure does NOT count against the iteration cap — it's a setup
signal, not a file-clarity signal. But it should be surfaced to the user
who decides whether to trust the run's A/B/C verdicts.

### Open-ended probe requirement — v5

In v5, **at least 1–2 of the Class A + Class B probes must be open-ended**:

> "Explain in your own words how rule X applies in scenario Y"

NOT yes/no, NOT multiple-choice, NOT specific-fact. Closed forms produce
convergence too easily because the answer space is narrow. Open-ended forms
require the model to articulate the rule's substance, surfacing
interpretation differences that closed forms hide.

**Rubric for open-ended probes** is necessarily looser:

- Required-present: load-bearing concepts that must appear (any phrasing)
- Required-absent: concepts that, if present, indicate misread
- Verdict: same binary CORRECT / WRONG, but the grader (parent agent)
  applies semantic judgment rather than substring matching for required-
  present concepts. State the judgment criterion at probe-draft time
  (Gate 1 approval).

Divergence on open-ended probes is a much stronger ambiguity signal than
divergence on yes/no.

### Random ordering

Each iteration shuffles probe order. Detects order-dependence (where a
model's answer to probe N is biased by what it saw in probe N-1). If round 2
with reshuffled order produces a *different* outcome than round 1 with the
same probes, that's diagnostic information about ordering effects in the
file's logical flow.

### Total per file (v5)

| Class | Count | Notes |
|---|---|---|
| A — Failure-targeted | 4–6 | At least 1 must be open-ended (counted toward A+B 1–2 open-ended quota) |
| B — Failure-likely | 1–3 | At least 1 of A+B must be open-ended (combined quota) |
| C — Control | 1 | Literal-token sanity check (unchanged from v4.2) |
| D — Calibration | 1 | Intentionally-ambiguous answer (NEW in v5; surfaces setup-bias) |
| **Total** | **7–11** | |

**Open-ended quota:** at least 1–2 probes from A+B combined must be open-
ended (state-the-rule-in-your-own-words style). Closed forms
(yes/no, multiple-choice, specific-fact) for the remaining A/B probes are
fine but should not be the entire test.

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

### Class D calibration probe template (v5)

A calibration probe is a question whose answer is intentionally NOT pinned
down by the file under test. Both models converging confidently = test
setup is biased.

**Drafting heuristic:**
1. Read the file
2. Identify a topic the file mentions but doesn't pin down (e.g., ordering between two optional sections; severity assignment for an edge case; default behaviour where the file is silent)
3. Phrase as a question that could plausibly have multiple defensible answers
4. Document the verdict logic explicitly: "if both models confidently say X, that's training-data convergence not file clarity"

**Anti-patterns for Class D:**
- Drafting a probe whose answer IS in the file but obscure (that's a Class A/B difficulty probe, not calibration)
- Drafting a probe whose answer is genuinely random/unpredictable (no one model would converge confidently — defeats the purpose)
- Tying the calibration to a topic the file is silent on by oversight (it should be a deliberate non-coverage; otherwise calibration becomes a backdoor coverage test)

**Verdict:**
- Both models hedge or acknowledge the file's silence → ✅ healthy
- Both models confidently converge on same answer → ⚠️ SETUP-SUSPECT
- Models diverge (one says X, other says Y) → 🟡 expected

Class D failure does NOT count against iteration cap (it's setup signal).

### Open-ended probe template (v5)

```
Probe (Class A or B, open-ended):

Question: "Explain in your own words how <RULE> applies when <SCENARIO>.
What does the agent do, and why?"

Rubric (looser than closed-form):
  Type: prose-with-semantic-judgment
  Required-present (load-bearing concepts, any phrasing):
    1. <concept 1 — load-bearing>
    2. <concept 2 — load-bearing>
  Required-absent (concepts that indicate misread):
    1. <forbidden concept>
  Grader: parent agent applies semantic judgment for required-present
    (substring matching is too strict for open-ended). State the
    judgment criterion at probe-draft time (Gate 1).
  Verdict: same binary CORRECT / WRONG.
```

### Honest closing-comment template (v5)

When a file passes v5 and you write the issue closing comment, use this
language template instead of "the file is solid" or "the file is unambiguous":

```
The v5 methodology found no divergence on this file under the documented
test conditions:
- Step 12 dual-model comprehension test: <X>/<N> probes CONVERGE on round 1
- Step 12.5 historical replay: <Y>/<Z> documented misread cases caught
  by both models (or: not feasible — no documented misreads on this file's
  rules)
- Step 16 Opus semantic-preservation: <PRESERVED count>/<total> rules

This validates clarity-in-isolation against correlated readers and (where
replay was run) rule-fires-on-real-input on a sample of N documented cases.
It does NOT prove the file is unambiguous in production conditions.

See § Methodology limitations in the plan doc for the limits this
methodology has not addressed.
```

### Decoupled probe-author flow (v5)

To address probe-author selection bias (per § Methodology limitations), v5
prescribes that a separate Opus subagent drafts the probe set BEFORE the
parent agent runs the workflow:

1. **Probe-drafting subagent** (model="opus"): reads file + learnings.md + style spec, drafts probes A/B/C/D + rubrics + coverage map, surfaces to user (Gate 1).
2. **Workflow-execution agent** (parent / current session): receives the approved probe set, runs steps 9–18 against it, cannot influence probe wording.

This breaks the I-drafted-probes-I-expected-to-pass loop. The trade-off is one extra subagent invocation upfront (~15-30 min).

For Phase 1 retrospective (where this didn't fire), the existing replay batch (PR #176) provides external validation against ground truth — partial but real check on parent-agent bias.

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

## Historical replay testing (complement to v4.2)

Phase 1 retrospective ([`## Methodology limitations`](#methodology-limitations))
identified that v4.2 synthetic-probe methodology validates clarity-in-isolation
against correlated readers, but doesn't measure whether rules actually fire on
real input. **Historical replay testing** addresses this directly: feed a real
historical buggy state to the current `self-review` / `review-findings` agent
and check whether it catches the documented misread.

Treat as a **complement** to v4.2, not a replacement. v4.2 = cheap clarity
stress-test for new files; replay = expensive but high-information-value deep
validation against documented misreads.

### When to run

- After a major instruction-file rewrite — confirms whether the rewrite
  preserved enforcement, not just clarity
- When `/improve` proposes escalating a rule that was previously v4.2-PASS —
  replay confirms whether the proposed escalation is necessary, or whether
  the existing rule was already sufficient and the misread was a one-off.
  **A `/improve`-proposed escalation on a v4.2-PASS rule is the canonical
  re-open trigger** for the corresponding closed Phase 1 issue.
- Periodic surveillance on a rotating set of historical cases

### Methodology

**Inputs per case:**
1. A `learnings.md` entry with file/line locations of the documented misread
2. The buggy commit SHA (state where the misread was present)
3. (Optional) historical context — spec/design from the PR

**Setup — git worktree:**

```bash
# 1. Create replay worktree at the buggy state
git worktree add /tmp/replay-<short-sha> <BUGGY_COMMIT>

# 2. (Variant A — recommended) Overlay CURRENT instruction files
cd /tmp/replay-<short-sha>
git checkout master -- AGENTS.md \
  ai-docs/code-style.md ai-docs/doc-convention.md ai-docs/agent-writing-style.md \
  .claude/agents/review-findings.md .claude/agents/self-review.md
```

**Variants:**

| Variant | Tests |
|---|---|
| **A — replay-on-current-rules** (overlay current instruction files) | Whether *current* rules would catch the misread. Higher-info test for retrospective. |
| **B — replay-on-historical-rules** (no overlay) | How the agent actually performed at the time. Baseline / regression check. |

**Subagent prompt** (review-findings, whole-codebase):

```
You are running as the review-findings agent on a quartzite codebase review.

CRITICAL paths:
- Codebase under review: /tmp/replay-<sha>
- Use ABSOLUTE paths starting with /tmp/replay-<sha>/ for ALL reads
- Do NOT read from the main checkout — different unrelated codebase

PROCEDURE:
1. Read /tmp/replay-<sha>/.claude/agents/review-findings.md and follow it
2. Read AGENTS.md, ai-docs/code-style.md, ai-docs/doc-convention.md from worktree
3. Read every *.spec.md and *.design.md in /tmp/replay-<sha>/ai-docs/plans/done/
4. Walk the source tree
5. Apply checklist mechanically. Output findings.
```

For `self-review` (diff-based), additionally specify the base_commit and HEAD.

**Dual-model:** spawn Opus + Sonnet in parallel, same as v4.2.

**Ground-truth comparison:**

| Subagent flagged the documented violation? | Outcome |
|---|---|
| Yes, correct location + severity | ✅ True positive — rule fires on real input |
| Yes, wrong severity | ⚠️ Partial — caught but undermarked |
| No (or different findings, missed the known violation) | ❌ False negative — **re-open trigger** for the corresponding v4.2-PASS Phase 1 issue |

**Cleanup:**

```bash
git worktree remove /tmp/replay-<sha> [--force]
```

### Methodology lessons (from the pilot run below)

- **Replay surfaces both ground-truth signal AND noise.** Current-rules-on-historical-code may flag historical code that was correct under historical rules. Frame ground truth as the test signal; surface additional findings separately — they're audit byproduct, not test failure.
- **Ground-truth labelling requires care.** Pull file/line and rule from the `learnings.md` entry + the fix-PR's diff + the escalation PR's instruction-file update.
- **Spec/design unavailability:** older PRs may not have these. `review-findings` (whole-codebase) avoids spec-construction; `self-review` (diff-based) needs a synthesised minimal spec.
- **Methodology divergence vs file-clarity divergence:** if Opus and Sonnet flag the violation but disagree on grouping or severity, that's a rule-wording question, not a file-clarity failure. Don't conflate.

### Cost vs v4.2 synthetic-probe rerun

Roughly the same cost (~1–2 hours per case for setup + dual-model run + analysis), but **much stronger evidence per case** because the test signal connects to documented ground truth rather than author-selected probes.

### Pilot run — PR #149 `document_features` placement

**Setup:**
- Buggy commit: `22459307bd737235bd0543177766e2f527a0d4e6` (master pre-PR-#149)
- Variant A — current instruction files overlaid on historical worktree
- Documented misreads (ground truth):
  - `src/lib.rs:9` — `document_features!()` BEFORE `//!` block (forbidden position A)
  - `quartzite-core/src/lib.rs:15` — `document_features!()` AFTER attribute block, no preceding `## Feature flags` heading (forbidden position B)

**Subagents:** Opus 4.7 and Sonnet 4.6 spawned in parallel via `general-purpose` `Agent` calls with `model` overrides.

**Results:** ✅ **TRUE POSITIVE** for both. Both caught both violations as `major`, citing the correct rule (`doc-convention.md` § Feature flags rendering / `review-findings.md` § Checklist § 6).

| Aspect | Opus | Sonnet |
|---|---|---|
| Ground truth caught? | ✅ both locations | ✅ both locations |
| Severity | `major` | `major` |
| Rule cited | `doc-convention.md` § Feature flags rendering | `doc-convention.md` § Feature flags rendering |
| Grouping | One combined finding (applied "group same pattern" rule) | Two separate findings (kept ungrouped) |

**Methodology divergence (interesting, not a failure):** Opus applied the `review-findings.md` § Rules "group same pattern across files into one finding" rule; Sonnet kept the findings separate. Both readings are defensible — this is a rule-wording question, not a file-clarity failure. Both models recognised both violations correctly.

**Additional findings (~10–15 per agent, out of scope for ground truth):** marker-form discrepancies in `ObjectExt` trait methods, tracing-span gaps, file-size limits, etc. These are **current-rules applied to historical code** — historical code predates current rule clarifications. For replay-test purposes these are noise. For codebase-quality purposes they may be interesting audit byproducts, but each needs separate adjudication.

**Implications for v4.2 verdicts:**
- The `review-findings.md` v4.2 PASS verdict (#171) is **consistent** with this replay — the rules in `review-findings.md` do fire on at least this one documented misread.
- The replay does not invalidate the v4.2 PASS — it provides positive evidence that the v4.2 PASS was warranted for this rule.
- For full retrospective coverage, replay 2–3 more cases targeting different rules (marker-mutex, mutex `.expect()`, `--workspace` clippy).

### Replay batch — completed runs

The two additional candidates from the originally-proposed batch were run.

#### Replay 2 — Marker-mutex co-occurrence (`cc382cd`)

**Setup:** worktree at `cc382cd` ("fix(widgets): name palette colors; add #[inline] to WidgetExt defaults") with current instruction files overlaid (Variant A). The fix commit `5622056` ("fix(style): drop redundant _Simple._ from fns that carry #[inline]") was the explicit correction; we replay the state just before that.

**Ground truth:** `quartzite-widgets/src/widget_ext.rs` had 21+ default trait methods carrying both `/// _Simple._` AND `#[inline]` simultaneously. Per the AXIOM in `code-style.md` § `#[inline]` and the `_Simple._` doc tag, these are mutually exclusive.

**Result:** ✅ **TRUE POSITIVE** for both subagents. Both grouped per the "same pattern across files" rule and emitted one `major` finding listing all locations.

| Subagent | Finding | Severity | Locations grouped |
|---|---|---|---|
| Opus | "26 trait default methods carry BOTH `/// _Simple._` AND `#[inline]`. Per AGENTS.md axiom..." | `major` | 26 location pairs |
| Sonnet | "every default method in `pub trait WidgetExt` carries BOTH... co-occurrence axiom violation" | `major` | 30 line numbers |

**Methodology convergence (vs PR #149):** in this case both Opus and Sonnet applied the grouping rule consistently — diverging on grouping that occurred in the PR #149 pilot (Opus grouped, Sonnet split) was case-specific, not a systemic methodology gap.

#### Replay 3 — Mutex `.expect()` substitution (`543bb4f^` = `1b80ccc`)

**Setup:** worktree at `1b80ccc` (parent of fix commit `543bb4f` "replace panicking mutex ops with poison recovery") with current instruction files overlaid (Variant A).

**Ground truth:** Multiple production files had `.lock().expect("...poisoned")` and `.lock().unwrap()` patterns on `Mutex` / `RwLock` / `Condvar` — `quartzite-runtime/src/timer.rs`, `connection_table.rs`, `thread_pool.rs`, `event_loop.rs`. Per AGENTS.md / `code-style.md` § Library safety idioms, these must be `.unwrap_or_else(|e| e.into_inner())`.

**Result:** ✅ **TRUE POSITIVE** for both subagents. Both flagged all violations as `major`, citing the substitution rule.

| Subagent | Grouping strategy | Findings | Severity |
|---|---|---|---|
| Opus | By lock type (Mutex / Condvar / RwLock) | 3 findings covering all locations | `major` |
| Sonnet | By file (connection_table / thread_pool / event_loop / timer) | 4 findings covering all locations | `major` |

**Methodology divergence (different from PR #149):** Opus and Sonnet again diverged on grouping strategy — Opus grouped by violation type, Sonnet grouped by file. Both readings are reasonable; both caught all violations. This confirms the grouping divergence observed in PR #149 is a recurring rule-wording question, not random variance.

#### Candidate 3 — `--workspace` clippy: NOT replayable via this methodology

The `--workspace` clippy rule is a **process-level rule** (specifies which shell command CI / agents should run), not a Rust-code-level rule that `review-findings` checks against the source tree. The "buggy state" was AGENTS.md saying `cargo clippy -- -D warnings` (without `--workspace`); `review-findings` doesn't review AGENTS.md content as code.

The rule is enforced directly by CI's `cargo clippy --workspace -- -D warnings` invocation in `.github/workflows/ci.yml` — process-level enforcement is the test, no agent layer needed.

**This is a methodology-fit limitation, not a test failure.** Some rules are agent-scope (rules `review-findings` / `self-review` apply to source code) and some are process-scope (rules CI / agents follow when running shell commands). Replay applies only to the former.

#### Combined replay results — all completed runs

| # | Replay case | Opus | Sonnet | Verdict |
|---|---|---|---|---|
| 1 | PR #149 — `document_features` placement | ✅ TRUE POSITIVE (`major`) | ✅ TRUE POSITIVE (`major`) | confirmed |
| 2 | `cc382cd` — marker-mutex co-occurrence | ✅ TRUE POSITIVE (`major`) | ✅ TRUE POSITIVE (`major`) | confirmed |
| 3 | `1b80ccc` — mutex `.expect()` | ✅ TRUE POSITIVE (`major`) | ✅ TRUE POSITIVE (`major`) | confirmed |
| — | `--workspace` clippy | n/a — methodology-fit limitation | n/a — methodology-fit limitation | not replayable |

**6 / 6 model-runs caught the documented misread.** All v4.2-PASS Phase 1 verdicts on the affected files (#167/#174 AGENTS.md, #168 code-style.md, #170 self-review.md, #171 review-findings.md) now have positive replay evidence supporting them. The retrospective concern (Phase 1's "100% PASS round 1 is a smell") is partially addressed — replay against ground truth confirms the rules do fire on real input for the cases tested.

#### Methodology lessons added by the batch

- **Grouping strategy divergence is recurring**, not random. Opus and Sonnet have different default groupings (by-pattern-type vs by-file), both consistent with the rule wording. Future probe-design and rule-wording work should expect this divergence and either tighten the grouping rule or accept both forms.
- **The methodology-fit limitation is real** but bounded: rules whose enforcement is in shell-command form (clippy invocations, CI workflow gates) aren't replayable. Document them as out-of-scope for replay.
- **Replay surfaces audit byproducts** — every run produced ~10–15 additional findings beyond ground truth (current rules applied to historical code). For replay-test purposes these are noise; for codebase-quality purposes they're either real ongoing issues or rules-that-postdate-the-historical-commit. Each needs separate adjudication.

### Future replay batch (if undertaken)

The completed batch covers high-frequency rules. Future replays could target:

| Candidate | Why | Tests rule |
|---|---|---|
| `clippy::doc_markdown` allowlist scope (PR #83 follow-up) | Earlier high-volume corrections | Backtick + allowlist scope rules |
| Trait-impl `// _Simple._` placement vs `///` | Separate from co-occurrence axiom | Marker-form by impl shape |
| `panic-index.md` updates after introducing production panic sites | New rule from #170 work | Panic-index sync rule |
| Bench files exempt from `#[cfg(test)]` | Carve-out exemption rule | Test coverage carve-out |

Each ~1–2 hours per case if rule-shaped (vs process-shaped).

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
- v4.3 — methodology-limits retrospective added after Phase 1 completed
  (5 files all PASS round 1, 0 iterations). Documents the gap between
  "what v4.2 actually validates" and "what the closing comments imply."
  Eight known limitations enumerated; seven addressable Phase-2
  improvements listed (out-of-family model aspirational, no current
  tooling access). Status banner + Goal section reframed to direct
  readers at the limitations section before re-using the template.
- v4.4 — adds **Historical replay testing** as a complement to
  v4.2/v4.3, addressing the strongest limitation (no connection to
  ground truth). Methodology section provides the worktree-based
  setup, two variants (current rules vs historical rules), subagent
  prompt template, ground-truth comparison table, and cleanup. **Three
  replay cases run** (PR #149 `document_features` placement, `cc382cd`
  marker-mutex co-occurrence, `1b80ccc` mutex `.expect()` substitution):
  6 / 6 model-runs caught the documented misread as `major`. All
  v4.2-PASS Phase 1 verdicts now have positive replay evidence
  supporting them. **One candidate (`--workspace` clippy) was
  documented as not replayable** via this methodology — process-level
  rules (shell-command invocations) aren't `review-findings`-shaped.
  **Methodology lessons identified:** Opus / Sonnet grouping-strategy
  divergence is recurring (by-pattern-type vs by-file) but both
  consistent with rule wording; replay always surfaces audit-byproduct
  findings beyond ground truth that need separate adjudication. The
  `/improve`-proposed-escalation-on-v4.2-PASS-rule mechanism documented
  as the canonical re-open trigger.
- v5 — current. **Retrofit:** bakes the v4.3 retrospective improvements
  and v4.4 replay methodology into the prescriptive workflow steps so
  future runs apply them by default. Workflow changes (per-file step
  list grew from 19 to 20 steps after integer renumbering — replay was
  inserted as a new step 16, bumping previous 16/17/18/19 to 17/18/19/20):
  - Step 6: Class A+B+D drafted (Class C still drafted at step 10
    after rewrite, since C anchors on a stable section of the rewritten
    file). Open-ended probe quota: at least 1–2 of A+B must be
    open-ended (not yes/no, not multiple-choice). Class D always runs
    as 1 probe per file — verdict logic (handled at step 13) has three
    outcomes; Class D is unconditionally part of every test run.
  - Step 12: Class D included in randomization
  - Step 13: explicit verdict logic for Class D — three outcomes
    (healthy hedging / SETUP-SUSPECT confident-converge / expected
    divergence). SETUP-SUSPECT does NOT count against iteration cap.
  - Step 16 (NEW): if `learnings.md` has documented misreads on
    this file's rules, run at least one historical replay case
    before declaring PASS
  - Step 17 (was 16): Opus semantic-preservation self-review now
    always fires — no empty-diff escape, builds track record over time
  - Step 19 (was 18): closing-comment template uses honest framing
    ("validates clarity-in-isolation against correlated readers,
    not clarity-in-production-context")

  Probe composition section gains Class D (calibration, 1 probe) and
  open-ended quota (1–2 from A+B). Total probes per file: 7–11
  (was 6–10).

  Implementation hints gain: Class D drafting heuristic, open-ended
  rubric template, honest closing-comment template, decoupled
  probe-author flow (separate Opus subagent drafts probes — addresses
  probe-author selection bias).

  v5 is the prescriptive form; v4.x retrospective sections (Methodology
  limitations, Historical replay testing) remain as the rationale for
  why the v5 workflow looks the way it does. Future Phase 2 / new-file
  applications follow v5.

  **Phase 1 retrofit:** issue closing comments on #167–#171 / #174
  receive follow-up comments adding (a) replay evidence from PR #176
  where applicable, (b) honest framing per v4.3, (c) link to v5 status.
  Done in same PR as the v5 workflow update.
