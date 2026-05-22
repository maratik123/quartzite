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

> Moved to [`ai-docs/instruction-file-validation.md` § Probe taxonomy](../instruction-file-validation.md#probe-taxonomy).

---

> Moved to [`ai-docs/instruction-file-validation.md` § Coverage rules](../instruction-file-validation.md#coverage-rules).

---

> Moved to [`ai-docs/instruction-file-validation.md` § Rubric framework](../instruction-file-validation.md#rubric-framework).

---

> Moved to [`ai-docs/instruction-file-validation.md` § Workflow gates](../instruction-file-validation.md#workflow-gates).

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

> Moved to [`ai-docs/instruction-file-validation.md` § Templates](../instruction-file-validation.md#templates).

---

> Moved to [`ai-docs/instruction-file-validation.md` § Known biases & limitations](../instruction-file-validation.md#known-biases--limitations).

---

> Moved to [`ai-docs/instruction-file-validation.md` § Historical replay testing](../instruction-file-validation.md#historical-replay-testing).

---

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
