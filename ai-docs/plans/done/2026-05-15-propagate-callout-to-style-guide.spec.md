# Propagate compaction-recovery callout pattern to `ai-docs/agent-writing-style.md`

**Source:** issue #358
**Date:** 2026-05-15
**Tracked in:** #358

## Context

### Why the callout pattern exists (load-bearing numeric facts)

The Compaction-recovery callout pattern exists because Sonnet-mode
sessions in this harness are auto-compacted, and Opus-mode sessions are
not. Concretely:

| Fact | Value | Consequence |
|---|---|---|
| Sonnet base-model context window | up to 1M tokens | Model can in principle hold a very long session. |
| Claude Code harness session cap when Sonnet is active | 200k tokens total = 180k input + 20k output | Harness imposes a tighter budget than the base model. |
| Auto chat-compaction trigger | input approaching the 180k input ceiling | This is the mechanism that emits the "Conversation compacted" marker the Compaction-recovery callout self-detects. |
| Opus-mode sessions | NOT auto-compacted in the same way | Opus-mode skills do not need a callout. |

The 1M / 200k / 180k / 20k numbers are the motivation for the entire
pattern: a Sonnet skill that runs past the 180k input ceiling will be
compacted mid-flow, lose intermediate reasoning, and re-enter the skill
without context — the callout is the mechanism that lets the skill
detect the marker and recover from its durable-state artefact. None of
this applies to Opus, which is why Pattern 7 is a code-side (dual-model
/ Sonnet) concern only.

### What was introduced

PR #349 (resolution of #348, merged 2026-05-14) introduced a new fail-loud
rule pattern — the top-of-file **"⚡ Compaction recovery check — read
FIRST on every invocation"** callout — and applied it to six code-side
SKILL.md files in three per-skill variants (A: glob-driven probe path
discovery; B: fixed-glob single artefact; C: parent-routing). The
canonical cross-link target is the singular h2 `## Compaction recovery
(re-entry)` in `.claude/skills/context-reset/SKILL.md`.

`ai-docs/agent-writing-style.md` is the style reference for binary rules
in `AGENTS.md` / `.claude/skills/**` / `.claude/agents/**` /
`ai-docs/code-style.md` / `ai-docs/doc-convention.md` (per `AGENTS.md §
Agent Docs`). It did not receive a propagated update during PR #349 — no
pattern entry was added, no citation, no `## Out of scope` review. The
variant taxonomy currently lives only inside the archival design doc at
`ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md`,
which is non-normative. This issue closes the propagation gap before
issue #357 (`/pr-failed` skill) would re-derive the same taxonomy from
the archival source.

Variant assignments (verbatim from PR #349 *Per-skill mapping*):

| Variant | Probe shape | Skills using it |
|---|---|---|
| A | Glob-discovered active-state probe in the skill's preamble | `/task`, `/code-review`, `/pr-commented` |
| B | Fixed-glob single in-flight artefact | `/bugfix`, `/interview` |
| C | Inherited — parent skill owns the probe | `/context-reset` (the cross-link target itself) |

Each variant's distinguishing surface phrase (used by the subtask-13
grep audit in PR #349):

- Variant A: `"Locate the durable-state file via this skill's active-state probe"`
- Variant B: `"If exactly one in-flight artefact exists"`
- Variant C: `"Identify the **parent workflow**"`

The shared invariant phrase (Variants A and B): `"re-enter this skill
from the top of its body"`. Variant C uses the equivalent phrasing `"Run
the parent skill's own compaction-recovery callout"`.

## Scope

1. Add a new pattern entry **`### 7. Compaction recovery callout`** to
   `ai-docs/agent-writing-style.md § Patterns` documenting Variants A /
   B / C — when each fires, the per-variant distinguishing phrase, where
   the cross-link target h2 lives
   (`.claude/skills/context-reset/SKILL.md § Compaction recovery
   (re-entry)`), and a one-line pointer to the archival design doc as
   the source of the locked variant bodies.
2. Update `AGENTS.md § Propagation Rule` Procedure step 1 grep recipe
   to include `ai-docs/agent-writing-style.md` in the scan paths. After
   this change, the recipe is:
   `grep -rn "<keyword>" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md`.
3. Review `ai-docs/agent-writing-style.md § Out of scope` (the "Files
   for Opus-only readers" enumeration). The six skills carrying the new
   callout are all code-side dual-model skills, so no audience-boundary
   shift is expected; the AC requires the review pass, not a guaranteed
   edit.
4. Append one in-flow learning entry to `ai-docs/learnings.md`
   recording the propagation miss and the new Propagation-Rule grep
   recipe (per AGENTS.md *Boundary rule 2 — `/task` Steps 8–12
   exception*; entry marked `Escalated? no` because the project-level
   escalation IS the AGENTS.md edit happening in the same flow).

## Out of scope

1. Rewriting the callouts themselves in any of the 6 SKILL.md files —
   PR #349's subtask-13 variant-identity audit confirmed all six are
   correctly shaped.
2. Adding a 4th variant — three cover every code-side skill today.
3. Adding the optional one-line citation footer
   (`> Per ai-docs/agent-writing-style.md § Pattern 7, variant X`) to
   the 6 SKILL.md callout bodies. Issue #358 marks this as optional
   and AC5 explicitly bars edits to the SKILL.md callout bodies. If
   later desired, it becomes a separate trivial follow-up.
4. Mechanising the propagation via a CI gate — separate concern,
   parallel to the deferred `scripts/check-instruction-file-sizes.sh`.
5. Any change to the cross-link target h2 in
   `.claude/skills/context-reset/SKILL.md` (it already exists).
6. Any change to the archival design doc at
   `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md`
   — it is archival and remains read-only as the historical record.

## Deferred

- Optional citation-footer fan-out across the 6 SKILL.md callouts | low
  leverage, not required by this issue | separate follow-up issue if
  ever wanted
- CI / pre-commit gate that runs the Propagation-Rule grep
  automatically | engineering concern, parallel to
  `scripts/check-instruction-file-sizes.sh` | separate issue when both
  size-gate and propagation-gate are scoped together
- Generalising the three callout variants into a reusable include
  template (PR #349's *Risks* table flags this as a long-term mitigation
  for variant drift) | premature — wait until variant taxonomy stabilises
  through ≥1 more skill onboarding | separate issue post-`/pr-failed`

## Key decisions

| Question | Decision |
|---|---|
| Where in `## Patterns` does the new entry land? | At the end as **`### 7. Compaction recovery callout`**, after the existing six patterns; the file's existing numbering stays stable. |
| Should the pattern entry inline each Variant body verbatim? | **No.** The full Variant A/B/C bodies live in the archival design doc and the six SKILL.md files; duplicating them here would create a 4th drift surface. The style-guide entry documents the *shape* (when each fires, distinguishing phrase, cross-link anchor), names the per-skill assignments, and cites the archival design doc + a live SKILL.md sample per variant. |
| Should the SKILL.md files gain a citation footer? | **No** (issue marks optional; AC5 bars callout-body edits). Listed in *Deferred*. |
| Is the dependency on issue #357 a blocker for this spec? | No — it is a soft ordering preference from the issue author. This issue can ship independently; #357 picks it up when it next runs. |
| Where does the learning entry go in the same flow? | Appended to `ai-docs/learnings.md` per AGENTS.md *Boundary rule 2 — `/task` Steps 8–12 exception*. `Escalated? AGENTS.md, doc-convention` (both files are edited in this same `/task` flow — `doc-convention` because `agent-writing-style.md` is the dual-model style guide listed alongside `doc-convention.md` in AGENTS.md *Agent Docs*; this PR's edit to it is the in-flow escalation). |
| Citation in the PR body | Per `ai-docs/agent-writing-style.md § Citation in PRs`, the PR body cites Pattern 7 (the new entry) — meta-citation, since this PR IS the introduction of Pattern 7. |

## Technical constraints

- **File-size cap.** `wc -c ai-docs/agent-writing-style.md` ≤ 35,000
  chars after the edit (currently ~5,459 chars; ample headroom).
- **AGENTS.md size.** `wc -c AGENTS.md` after editing the grep recipe
  stays under the 35,000-char early-warning (currently ~33,917 chars).
  The grep-recipe edit adds ≤ ~50 chars (one path append) — comfortably
  within the headroom; no extraction pass needed.
- **AGENTS.md *Propagation Rule* table fires.** Editing
  `ai-docs/agent-writing-style.md` is not yet a row in the Propagation
  Rule table, but AGENTS.md is being edited in the same PR (the grep
  recipe), so the Procedure step 1 grep MUST run before commit; any
  match in `.claude/agents/`, `.claude/skills/`, or AGENTS.md for the
  changed terminology (`Pattern 7`, `compaction recovery callout`,
  `agent-writing-style.md` mentions) must be inspected.
- **Boundary rule 2 — `/task` Steps 8–12 exception** applies. The
  learning entry appended to `ai-docs/learnings.md` lands in the same
  conversation turn as the instruction-file edits IFF this work happens
  inside `/task` Steps 8–12 (the expected execution mode). Outside that
  flow, the learning entry would have to land in a separate turn — that
  case is unlikely here and out of scope for the spec.
- **No code change.** Documentation / instruction-file PR only; no Rust
  build / clippy / fmt / doc / test gate fires substantively (they all
  still run in CI as no-ops on doc-only diffs).
- **`actionlint` gate.** No workflow file is edited, so `actionlint` is
  not in scope.
- **No new artefacts requiring `.gitignore` rules.**

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ai-docs/agent-writing-style.md § Patterns` gains a new entry `### 7. Compaction recovery callout` documenting Variants A / B / C — for each variant, the entry names: (a) when it fires (which skill shape it covers — preamble-glob vs fixed-glob vs parent-routing); (b) the per-variant distinguishing surface phrase verbatim (the three phrases from the Context section above); (c) the per-variant skill assignments (Variant A: `/task` / `/code-review` / `/pr-commented`; Variant B: `/bugfix` / `/interview`; Variant C: `/context-reset`); (d) the canonical cross-link target `.claude/skills/context-reset/SKILL.md § Compaction recovery (re-entry)`; (e) a pointer to the archival design doc at `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md` as the source of the locked variant bodies. |
| AC2 | `AGENTS.md § Propagation Rule` Procedure step 1 grep recipe includes `ai-docs/agent-writing-style.md` in the scan paths. Verifiable: `grep -n "ai-docs/agent-writing-style.md" AGENTS.md` returns ≥ 1 hit inside the Procedure section. |
| AC3 | `ai-docs/agent-writing-style.md § Out of scope` reviewed; either left unchanged (with an explicit no-shift note in the PR body) or updated if any boundary line shifted since 2026-05-14. The six skills carrying the new callout are not in the "Opus-only readers" enumeration, so the expected outcome is "reviewed, no change". _Rationale: Opus-mode sessions are not auto-compacted (see Context § "Why the callout pattern exists") — therefore Opus-mode skills correctly do not require a callout and remain correctly enumerated under `## Out of scope`. The review pass confirms that none of the six dual-model code-side skills carrying the new callout crossed into the Opus-only enumeration since 2026-05-14._ |
| AC4 | `wc -c ai-docs/agent-writing-style.md` ≤ 35,000 chars. `wc -c AGENTS.md` ≤ 35,000 chars (early-warning gate). |
| AC5 | No edits to the six SKILL.md files' callout bodies. Verifiable: `git diff master -- .claude/skills/{task,code-review,pr-commented,bugfix,interview,context-reset}/SKILL.md` is empty (or touches only material outside the Compaction-recovery callout block, which is unlikely for this PR's scope). |
| AC6 | `ai-docs/learnings.md` gains one new dated entry (`### 2026-05-15 — process — [short description]`) recording the propagation miss and the new Propagation-Rule grep recipe. Entry follows the AGENTS.md *Entry format* exactly: includes `**What happened:**`, `**Rule:**`, `**Escalated?**` fields. `**Escalated?**` value reflects the files actually edited in this PR (expected: `AGENTS.md, doc-convention` — `doc-convention` is the AGENTS.md-side label for edits to the dual-model style guide `ai-docs/agent-writing-style.md`). |
| AC7 | PR body cites `ai-docs/agent-writing-style.md § Pattern 7` per the style guide's own `## Citation in PRs` section — meta-citation since this PR introduces Pattern 7. |
| AC8 | `grep -rn "Pattern 7\|compaction recovery callout" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md` after the edit returns the expected set: 1 hit in `agent-writing-style.md` (the new pattern heading) and the grep itself documented in AGENTS.md — no stale references in `.claude/agents/` or `.claude/skills/` that would imply Pattern 7 was the wrong number. |

## Open questions

None blocking design.

The design agent may decide:

- Exact wording of the new Pattern 7 entry — the spec fixes the
  required content (Variants A/B/C, distinguishing phrases, skill
  assignments, cross-link, archival pointer); the prose around that is
  the design agent's call.
- Whether the new entry includes a small inline example block (one
  trimmed Variant-A callout snippet, marked "see SKILL.md for the full
  body"), or just bullet-summarises each variant. Style guide already
  uses the "Concrete do/not examples" pattern (Pattern 6), so a single
  representative snippet is consistent — design agent's choice.
- Exact placement of the AGENTS.md grep-recipe edit (inline in the
  existing sentence vs. a new bullet) — trivial.
- Whether the AGENTS.md *Propagation Rule* table also gains a new row
  for `ai-docs/agent-writing-style.md` (e.g., "if you add a new fail-
  loud pattern entry → also update each downstream skill / agent
  carrying that pattern"). Listed here because the issue body does not
  explicitly require it; design agent may treat it as in-scope-by-
  implication or punt to a follow-up.
