# Design: Update self-improve.md Step 6 contract — route clean-context eval to parent thread

**Issue:** #366
**Date:** 2026-05-16

## Approach

### Problem recap

`.claude/agents/self-improve.md` Step 6 ("Eval (REQUIRED after Step 5)") currently ends with:

```
Run the scenario via an `Agent` subagent in a clean context.
```

This directive is structurally unfulfillable by the `self-improve` agent class. Two append-only entries in `ai-docs/learnings.md` (2026-05-15) record the diagnosis:

1. **PR #362 Commit-C entry** (line 1219, *"`self-improve` subagent silently degraded `/improve` Step 6 from clean-context evals to same-context close-reads — must PAUSE-and-surface before substituting a primitive"*) codifies the general **pause-and-surface** rule: when a workflow step names a primitive the agent lacks, the agent MUST pause and surface to the parent BEFORE substituting any degraded alternative.
2. **PR #364 P5 finding** (line 1227, *"`self-improve` subagent genuinely lacks the `Agent` / subagent-dispatch primitive in its runtime tool exposure ... `self-improve.md` Step 6 contract is structurally unfulfillable by the subagent itself"*) falsifies the prior side-note hypothesis: the runtime tool exposure for `self-improve`'s agent class genuinely omits `Agent`, regardless of what its frontmatter `tools:` field implies.

Result: any `/improve` run that reaches Step 6 currently forces the subagent to either silently degrade (forbidden by entry 1) or stop mid-workflow with an ad-hoc surface. Both are bugs.

### Chosen solution

Rewrite Step 6 in `.claude/agents/self-improve.md` to:

1. Open by reaffirming the eval requirement is unchanged (PASS/FAIL on a clean-context reproducer run for each pattern applied in Step 5).
2. Codify the **structural absence** of `Agent` in `self-improve`'s runtime tool list, with inline citations to both 2026-05-15 learning entries (PR #362 Commit C and PR #364) as the authority. The wording stays generic (any named primitive the agent lacks) but uses `Agent` as the named case-in-point.
3. Specify the **pause-and-surface handoff**: the subagent MUST NOT spawn (or attempt to spawn) the reproducer in its own context. Instead, it composes a handoff message and yields control back to the parent thread.
4. Embed a fenced reproducer-prompt template (skeleton with named placeholders + one worked example) inside Step 6. The subagent fills placeholders per pattern from Step 5 and emits the assembled block verbatim. The number of numbered reproducers equals the number of patterns the subagent applied in Step 5 (no separate cap — bounded organically by Step 1 pattern-finding output).
5. Preserve the existing `PASS criterion:` / `FAIL criterion:` semantics and the `Eval: PASS ✅` / `Eval: FAIL ❌` report-line format (downstream consumers key on these strings — AC6).
6. Add a one-line **asymmetry note** (inline prose, not an HTML comment — markdown comments are invisible to grep-driven Propagation Rule checks in the future) noting that the Corrections-Log sync-group sister file `.claude/agents/learnings-escalation-audit.md` has no Step 6 equivalent, so no mirrored edit is needed for this change.

The new Step 6 reassigns *execution responsibility* (subagent → parent) but keeps *eval semantics* (clean-context reproducer per pattern, PASS/FAIL report) identical to today.

### Rejected alternatives

| Alternative | Why rejected |
|---|---|
| **Leave Step 6 as-is; rely on the subagent to ad-hoc surface** | The 2026-05-15 PR-#362 entry shows the ad-hoc disposition the subagent picked was *silent degradation*, not surface. The rule must be encoded in the contract, not left as runtime judgment. |
| **Delete Step 6 entirely** | Spec out-of-scope #2 forbids changing what Step 6 means. The eval requirement is essential — eliminating it removes the safety net against rules-not-strong-enough drift. |
| **Add `Agent` to `self-improve`'s frontmatter `tools:` field** | The PR #364 P5 entry establishes the frontmatter is not authoritative for runtime tool exposure. Adding `Agent` to frontmatter does nothing if the harness gates it. Spec out-of-scope #3 explicitly excludes harness-level fixes. |
| **Propose a hook that forces pause-and-surface** | Hook threshold is ≥3 occurrences against an existing rule. Current state: two occurrences (PR #362, PR #363) with no rule yet in the contract. Adding the contract IS the rule; hook escalation can follow if it recurs. Spec out-of-scope #4 explicitly defers this. |
| **Place the reproducer template in a separate appendix at the file's bottom** | Spec Key Decision row 3 picked inline. Inline keeps the protocol and its template at the single point of consumption — no drift risk between two locations. |
| **Use an HTML comment (`<!-- propagation note: ... -->`) for the asymmetry note** | An HTML comment is invisible to the grep-driven Propagation Rule check in AGENTS.md procedure step 1 (`grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ AGENTS.md`). A future edit re-examining the Corrections-Log group would miss the rationale. Inline prose is greppable. |
| **Mirror an equivalent Step 6 contract into `learnings-escalation-audit.md` "for symmetry"** | `learnings-escalation-audit.md` is a passive auditor (`/ai-audit` Phase 1). Its workflow has no eval phase and no `Agent` dispatch — see Investigation below. Adding a stub Step 6 there would be cargo-cult propagation, not a real rule. |

### Investigation summary

Verified during design:

- `self-improve.md` body confirmed at 7,233 bytes (114 lines). Step 6 currently spans lines 94–105 (≈12 lines / ~500 chars). The rewrite adds the protocol prose (~10–15 lines), the citation paragraph (~3–4 lines), the template block (~30–40 lines fenced markdown), one worked example (~10–15 lines), and the asymmetry note (~1–2 lines). Estimated new Step 6 footprint: ~60–80 lines / ~3,000–4,500 chars. Estimated new file size after edit: ~10,000–11,500 chars — comfortably below the 35,000-char AGENTS.md early-warning threshold for instruction files (the per-file file-size axiom is governed by the same threshold across agent files).
- `learnings-escalation-audit.md` body confirmed at 9,665 bytes (138 lines). Its Step 6 ("Report") is a structured-output report step, not an eval phase. It has no `Agent` dispatch, no clean-context reproducer concept, and no execution-routing dependency on subagent-spawn primitives. **No propagation edit needed**; the asymmetry note in `self-improve.md` records this for future readers.
- AGENTS.md is at 39,775 chars (225 chars below the 40,000 hard cap). This PR does NOT touch AGENTS.md, satisfying the spec's Technical Constraints note.
- Both authoritative learning entries are already in master via merged PRs (#362, #364) — citations are stable references, not pending work.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rewrite Step 6 body in `self-improve.md`. Replace the directive at line 100 plus its surrounding context (lines 94–105). New body contains: (a) eval-requirement preamble unchanged; (b) named-primitive-absence statement citing PR #362 Commit-C entry + PR #364 P5 entry, including the verbatim phrase **"structurally unfulfillable"** (per design-review recommendation 3 — keeps the contract greppable against the P5 learning entry's wording); (c) pause-and-surface protocol prose (subagent introspects → confirms `Agent` absent → assembles handoff → yields); (d) fenced reproducer-prompt template skeleton with named placeholders (`<pattern_id>`, `<pattern_summary>`, `<original_error_repro>`, `<expected_fixed_output>`, `<PASS_criterion>`, `<FAIL_criterion>`); (e) one worked example using a real Step-5 pattern (e.g., a hypothetical "spec amendment requires design loop" reproducer) to anchor the skeleton; (f) handoff-header convention (`## Step 6 handoff — clean-context eval reproducers`) plus the explicit one-line parent-thread instruction; (g) preserved `Eval: PASS ✅` / `Eval: FAIL ❌` report-line format with explicit note that the parent thread (not the subagent) emits the final report into the conversation. | `.claude/agents/self-improve.md` | — |
| 2 | Add the propagation-rule asymmetry note as a one-line inline note inside the rewritten Step 6 (placement: immediately after the protocol prose, before the template — visible to a future grep, not buried in a comment). Wording: *"Propagation-rule asymmetry: the Corrections-Log sync-group sister file `.claude/agents/learnings-escalation-audit.md` has no Step 6 eval-phase equivalent (its workflow is a passive auditor; its `Step 6 — Report` is structured output, not a primitive-dispatch step), so this contract requires no mirrored edit there."* | `.claude/agents/self-improve.md` | 1 |
| 3 | Verification pass — `wc -c .claude/agents/self-improve.md` reports < 15,000 chars (sanity bound, raised from earlier 12,000 per design-review note 1; load-bearing limits remain AGENTS.md's 35k early-warning / 40k hard cap thresholds); `grep -n "Agent subagent in a clean context" .claude/agents/self-improve.md` returns no matches (AC1 mechanical); `grep -n "pause-and-surface\|Step 6 handoff" .claude/agents/self-improve.md` returns matches (AC2 / handoff header); `grep -n "PR #362\|PR #364\|2026-05-15" .claude/agents/self-improve.md` returns matches for both citations (AC4); `grep -n "Eval: PASS\|Eval: FAIL" .claude/agents/self-improve.md` returns matches (AC6); `grep -n "learnings-escalation-audit" .claude/agents/self-improve.md` returns the asymmetry-note match (AC5); `grep -n "structurally unfulfillable" .claude/agents/self-improve.md` returns at least one match (per design-review recommendation 3 — keeps the new Step 6 contract's prose greppable against the diagnosing P5 learnings entry's wording). Pre-commit hook runs and passes. | `.claude/agents/self-improve.md` | 1, 2 |

> Decomposition note: tasks 1 + 2 are tightly coupled (same Step 6 body) — the asymmetry note can land in the same Edit as the Step 6 rewrite, OR as a separate Edit; AC5 is verified by `grep -n "learnings-escalation-audit"` regardless of hunk shape (per design-review note 2). Tasks are listed separately for AC traceability, NOT to constrain the diff shape.

## Risks

- **AC7 — post-merge `/improve` Step 6 PASS verification depends on the NEXT `/improve` run reaching Step 6.** If no learning corrections accumulate to ≥3 entries before the next `/improve` invocation, AC7 cannot be exercised in the PR window. Mitigation: AC1–AC6 are mechanical, mergeable on grep-verifiable evidence. AC7 is a downstream behavioural acceptance that the PR enables but cannot force; record AC7 as a deferred verification gate in the PR body (post-merge, on next `/improve` run). The user controls the `/improve` cadence — design assumes AC7 is verified the next time `/improve` runs naturally.

- **Future drift between the Step 6 template and the actual `/improve` workflow.** If `/improve` itself grows additional reproducer requirements (e.g., per-pattern fixture data, multi-step reproducers), the template skeleton may go stale. Mitigation: keep the template's placeholders generic (`<pattern_id>`, `<expected_fixed_output>`) rather than mechanically enumerating every Step-1 pattern type. A change to `/improve` semantics should be a separate `/task` whose design phase includes a sweep of the Step 6 template.

- **Asymmetry-note rot.** If a future change adds an eval phase to `learnings-escalation-audit.md` (e.g., the audit acquires a clean-context verification step), the asymmetry note in `self-improve.md` becomes false. Mitigation: the note's exact wording cites the *current* shape ("passive auditor; structured output"). A future change that breaks the asymmetry will fail the AGENTS.md Propagation Rule grep step (the keyword "learnings-escalation-audit" in `self-improve.md` will be hit when re-running the procedure on the Corrections-Log group), prompting a sync edit.

- **Inline citation format drift.** The two 2026-05-15 entries are cited by date + slug + PR number. If `ai-docs/learnings.md` is renamed or the slug wording changes (Boundary rule 1 forbids editing existing entries, so the slug is stable in practice), the citations stay valid. The PR numbers are immutable. Low risk; no specific mitigation beyond using both date+slug AND PR# in each citation (belt-and-braces).

- **`description:` frontmatter mismatch.** The current frontmatter description does not name Step 6's eval-routing semantics. Spec out-of-scope #6 allows a frontmatter tweak only if directly implied by the new body. Decision: no frontmatter edit — the body change is about *who* runs the eval (subagent vs. parent), not *what* `self-improve` does. Existing description ("Analyzes ai-docs/learnings.md … escalating to hooks at ≥3 occurrences. Invoked by /improve. Does not write code.") remains accurate.

## Test Design

No Rust code under test; this is a doc-only edit. Verification is mechanical grep-driven and one downstream behavioural gate.

### Task 1 — Step 6 rewrite

- **Location:** `.claude/agents/self-improve.md` lines 94–105 (current Step 6) replaced and extended.
- **Entry point:** the file body itself; consumed by `/improve` when it spawns the `self-improve` agent.
- **Scenarios:**
  - *Happy path (AC1, AC2, AC4):* `grep -n "Run the scenario via an \`Agent\` subagent in a clean context" .claude/agents/self-improve.md` returns no matches (directive removed); `grep -n "pause-and-surface" .claude/agents/self-improve.md` returns at least one match (rule named); `grep -n "PR #362" .claude/agents/self-improve.md` and `grep -n "PR #364" .claude/agents/self-improve.md` each return at least one match (citations present); `grep -n "Agent" .claude/agents/self-improve.md` returns at least one match in a sentence stating Agent is absent.
  - *Template presence (AC3):* `grep -n "<pattern_id>\\|<expected_fixed_output>" .claude/agents/self-improve.md` returns matches for the named placeholders; one worked example is present (look for a concrete pattern name + filled placeholders).
  - *Eval format preserved (AC6):* `grep -n "Eval: PASS ✅" .claude/agents/self-improve.md` and `grep -n "Eval: FAIL ❌" .claude/agents/self-improve.md` each return at least one match.
  - *Edge case — handoff header convention:* `grep -n "Step 6 handoff" .claude/agents/self-improve.md` returns at least one match (header convention encoded).
- **Fixtures / helpers needed:** none — direct file grep.

### Task 2 — Asymmetry note

- **Location:** inside the rewritten Step 6, one-line note.
- **Scenarios:**
  - *Happy path (AC5):* `grep -n "learnings-escalation-audit" .claude/agents/self-improve.md` returns at least one match; the surrounding sentence names "Corrections-Log sync group" and "no Step 6 equivalent" (or equivalent prose).
  - *Greppable for future propagation checks:* the keyword "learnings-escalation-audit" is in plain prose (not inside an HTML comment) — `grep -v '<!--' .claude/agents/self-improve.md | grep -n "learnings-escalation-audit"` returns matches.

### Task 3 — Verification

- **Location:** the verification pass itself.
- **Entry point:** running the grep suite enumerated above + pre-commit hook + `wc -c` size check.
- **Scenarios:**
  - All AC1–AC6 greps pass.
  - File size < 15,000 chars (sanity bound per design-review note 1; load-bearing limit is AGENTS.md's 35,000-char early-warning threshold).
  - Pre-commit hook runs and passes (no `actionlint` / `cargo fmt` triggers — pure doc edit).
- **Fixtures / helpers needed:** none.

### AC7 — deferred behavioural gate

- **Location:** next `/improve` invocation, post-merge.
- **Entry point:** `/improve` reaches Step 6 with the rewritten contract loaded.
- **Scenarios:**
  - *Happy path:* the `self-improve` subagent introspects, confirms `Agent` absent, emits the templated handoff to the parent. The parent dispatches the reproducers via its own `Agent` primitive. At least one reproducer returns PASS, the parent reports `Eval: PASS ✅` into the conversation.
- **Recorded as deferred** in PR body; user runs `/improve` when corrections accumulate naturally.

## Open questions

- (none — all design-affecting ambiguities were resolved by the spec's Key decisions table; presentation-level choices in this design are within the design agent's purview per the spec's Open questions section)
