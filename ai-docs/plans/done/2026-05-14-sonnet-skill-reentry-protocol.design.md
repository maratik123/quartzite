# Design: Sonnet skill re-entry protocol after auto-compaction

**Issue:** #348
**Spec:** `ai-docs/plans/2026-05-14-sonnet-skill-reentry-protocol.spec.md`
**Date:** 2026-05-14
**Revision:** Round 3+4 — Round 3 amended to reconcile design with three Round-2 spec amendments (terminology rename of the re-entry invariant, AC1 per-variant-group identity, AC9 active-task-probe routing). Round 4 — pure terminology-cleanup pass (user-authorised exception to the 3-round cap): the spec landed a 4th amendment renaming three lingering "restart-Step-1" references (spec lines 126, 184, 215) to the **Full-read-on-re-entry invariant** wording. No design contract change — only an audit-trail row appended to *Spec-amendment history*. Round 2 had amended this design in-place to resolve design-review's 4 blockers + 1 major; Round 3 kept those resolutions intact and only renamed invariant-related prose to match the amended spec; Round 4 leaves all variant bodies, decomposition, ACs, and verification recipes untouched.

## Approach

### Summary

A heuristic, prompt-only re-entry protocol layered on top of the existing
`.progress.md` discipline. Three structural changes ship in one PR:

1. **Schema extension** of `ai-docs/templates/progress-format.md` with five
   new header fields (`current_step`, `decisions_log`, `last_passed_gate`,
   `parent_skill`, `entry_args`). Markdown-section format preserved — no
   format pivot, every current consumer keeps reading the file. (The fifth
   field, `entry_args`, was added in Round-2 amendment to resolve Major 5:
   `/task` has three preambles, and re-entry without `$ARGUMENTS` requires
   a recorded entry reference to route correctly.)
2. **Top-of-file "Compaction recovery check" callout** added to six
   code-side orchestrator SKILL.md files. Body is identical in *intent* but
   carries two **per-skill variants** (Round-2 amendment to resolve
   Blockers 1, 2, 3, 4): a *glob-based* variant for `/task`,
   `/code-review`, `/pr-commented` (probe-driven path discovery), and a
   *fixed-path* variant for `/bugfix`, `/interview`, `/context-reset`.
   Encodes the **Full-read-on-re-entry invariant** explicitly — re-read
   the durable-state file end-to-end, then re-enter the skill from the
   top of its body (preambles included). NOT "restart from Step 1" —
   that wording mis-routed `/task` into the interview phase per Blocker 1.
   (Spec terminology rename, Round-2 amendment 1 — see *Spec-amendment
   history* above.)
3. **Workflow-steps-on-top reorder** of every code-side SKILL.md
   (truncation preserves the file's *start*). Reference material — gate
   checklists, anti-patterns, edge cases, examples — moves below the
   steps. For `/task` only: extract reference material to a sibling
   `reference.md` to land the SKILL.md under the 5,000-token
   (~20,000-char) target so it no longer risks per-skill truncation.

A `/bugfix` trace file absorbs the same four fields inline (plus
`entry_args` when relevant). Two near-stateless skills (`/verify`,
`/pr-merged`) get a one-line waiver.

### Spec-amendment history

Three Round-2-resolution amendments landed on the spec after design-review
returned **GO with 3 Notes** at the end of Round 2. The user chose Path A
(amend spec) to reconcile spec wording with the variant model this design
already embodied. The amendments are mechanical — they update terminology
and acceptance-criteria wording so the spec matches the design's three-
variant model. The design's substantive content (variant bodies, per-skill
mapping, decomposition, risk table) is unchanged; this section names the
amendments so a future reviewer can trace the design's contract back to
the amended spec.

| # | Spec section affected | Amendment |
|---|---|---|
| 1 | *Technical constraints* | Bullet renamed **Restart-Step-1 invariant** → **Full-read-on-re-entry invariant**. Body expanded to state "re-enter the skill from the top of its body (preambles included)" and to add: "Per-skill callout variants (A/B/C — see design doc *Per-skill mapping*) route this re-entry correctly for each skill's preamble / step-1 / no-numbered-step shape." This aligns the spec's invariant name with the design's *Per-skill mapping* + *Wording rationale* sections, which already used the "top-of-skill-body" framing throughout the variant bodies. |
| 2 | *Acceptance Criteria* — **AC1** | "Callout body is identical across skills (DRY)" → "Callout body is identical **within each variant group** (Variant A: `/task` / `/code-review` / `/pr-commented`; Variant B: `/bugfix` / `/interview`; Variant C: `/context-reset`); each instance names its own probe / durable-state file per the design's *Per-skill mapping* table." The trailing clause "restart from Step 1 — never jump to the recorded `current_step`" was replaced with "re-enter the skill from the **top of its body** (preambles + Step 1) — never jump to the recorded `current_step`". This brings AC1's identity claim and routing wording in line with the variant table below. |
| 3 | *Acceptance Criteria* — **AC9** | "restarts from Step 1" → "re-enters the skill from the top of its body (preambles + Step 1, routed by the active-task probe)". The active-task probe is the preamble's glob-and-validate sequence (Variant A) or fixed-glob sequence (Variant B); for Variant C the parent skill's probe applies. The phrase "active-task probe" in AC9 maps to the **Probe shape** column of the design's *Per-skill mapping* table — `⚡ First` preamble glob for `/task` / `/code-review`, PR-linkage derivation for `/pr-commented`, fixed-glob `ls ai-docs/bugfix/trace-*.md` for `/bugfix`, glob-then-pair on `*.state.md` for `/interview`, inherited parent probe for `/context-reset`. **Terminology note:** the design body and the locked Variant A callout use the established phrase "active-state probe" verbatim (load-bearing for subtask 13's grep audit). "Active-task probe" (spec AC9) and "active-state probe" (design body + Variant A) refer to the same mechanism — the active-task/state-discovery glob in the skill's preamble. The two surface strings name the same probe; no design edit is required (the orchestrator instruction explicitly forbids re-rewriting the locked Variant bodies). |
| 4 | *Key decisions* row "Re-invocation behavior on re-entry" (**spec line 126**); *Open questions* — "Exact wording of the callout" bullet (**spec line 184**); *Notes for the design agent* — Full-read-on-re-entry-invariant bullet (**spec line 215**) | **Terminology-cleanup pass (Round 4, user-authorised exception to the 3-round cap).** Three lingering references to "restart-Step-1" / "the restart-Step-1 invariant" in the spec — leftovers from Round 1 that survived the Round-2 amendment-1 rename — were renamed to align with the **Full-read-on-re-entry invariant** wording already in *Technical constraints*. Spec line 126 (*Key decisions* row): "Restart Step 1." → "Full re-read on re-entry" (with the Round-1 option label "Restart Step 1" preserved as audit-trail in a parenthetical and the body updated to "re-enter the skill from the **top of its body** (preambles + Step 1)"). Spec line 184 (*Open questions*): "The callout MUST encode the restart-Step-1 invariant" → "The callout MUST encode the Full-read-on-re-entry invariant" (with the explicit recipe appended: "re-read the durable-state file end-to-end, then re-enter from the top of the skill body; preambles + Step 1; never skip-to-`current_step`"). Spec line 215 (*Notes for the design agent*): "The restart-Step-1 invariant is a hard requirement" → "The Full-read-on-re-entry invariant is a hard requirement" (with the same recipe expanded inline). **Contract impact: none.** The variant bodies, *Per-skill mapping*, decomposition, ACs, risk table, and verification recipes all carried the correct wording already (Round-2 amendment 1 had renamed the invariant in *Technical constraints*; the design's *Wording rationale* + Variant A/B/C bodies already used "re-enter this skill from the top of its body" verbatim). This row exists for audit-trail completeness only — no current-state design reference contradicts the amended spec, and the historical references in this design (*Rejected alternatives* "Single global callout body ('restart from Step 1' version, Round-1)", *Wording rationale* "replaces the Round-1 'restart from Step 1' wording", Risk-table "renamed from Round-1 'Restart-Step-1 invariant' per Round-2 amendment 1", and Round-2-recommendations item 1) remain intact as documentation of the rejected Round-1 framing. |

Two design edits already landed BEFORE the spec amendments (in response to
Round-2 Notes 3 and (d)):

- **Variant C bullet 2** (the parent's cross-link back to `/context-reset`
  § Compaction recovery (re-entry)) gained a parenthetical clarifying that
  the link is a *reading* link to the canonical rationale, **not** an
  instruction to re-run Variant C — preventing a recursive callout chain.
- **Subtask 13** elevated "variant identity per skill" from risk-table
  mitigation prose to an explicit PR-body deliverable checklist (per-skill
  variant + variant-distinguishing phrase grep result).

### Why this design

The user explicitly chose **heuristic self-detection over a
`SessionStart|compact` hook**. The callout is the only mechanism;
correctness rests entirely on:

- **Truncation-order awareness.** Claude Code's per-skill truncation keeps
  the *start* of `SKILL.md`. If we put the callout at the top, it survives
  even when the rest of the body is dropped at the per-skill 5,000-token
  cap. The reorder is therefore not cosmetic — it's load-bearing.
- **The Full-read-on-re-entry invariant.** On re-entry the agent must
  re-read the progress file end-to-end before any tool calls and then
  re-enter the skill *at the top of the body* (preambles included), NOT
  at "Step 1". The invariant name matches the spec's *Technical
  constraints* bullet (renamed in Round-2 amendment 1 from "Restart-
  Step-1 invariant"). This matters because:
  - `/task` Step 1 is the interview phase (not the active-state probe —
    that lives in the `⚡ First` preamble above Step 1).
  - `/code-review` Step 1 is "Determine branch" (not the RESUME probe).
  - `/pr-commented` Step 1 is "Open / extend progress file" (preconditions
    run *above* the numbered steps).
  - `/bugfix` Step 1 is "Reproduce and Trace" — re-running it on a
    confirmed trace re-asks the user to confirm what's already confirmed.
  - `/interview` Step 1 is "Detect entry mode" — round counter lives in
    `.state.md`, not in Step 1.
  Restarting "from Step 1" literally would skip the active-state probes
  and/or re-do user-confirmed work. The correct invariant is **re-read the
  progress file then re-enter the skill body from the top** (which routes
  through the preamble or first probe-step as the skill defines).
- **The recorded `current_step` is a hint, never an instruction to skip
  the read.** This closes the half-written-boundary failure mode (the
  previous record could be torn or stale). Exception: for skills where a
  re-traceable user-confirmed gate exists earlier in the workflow
  (`/bugfix` trace confirmation, `/interview` round counter), the
  skill-body re-entry MAY consult `current_step` *after* the full re-read
  to avoid re-asking the user to confirm work already confirmed. The
  skill body itself encodes the logic — the callout merely routes
  control to the skill body's top.
- **One artefact per workflow.** `/bugfix` already owns a trace file; we
  extend it rather than create a parallel `.progress.md`. `/interview`
  already owns a spec + `.state.md`; same logic — its callout points at
  those, not at a fictitious progress file.

### Rejected alternatives

| Alternative | Rejected because |
|---|---|
| `SessionStart|compact` hook + per-step Skill-tool re-invocation | User explicitly chose heuristic over deterministic; out of scope per spec |
| Per-skill YAML/JSON header for `current_step` etc. | Format pivot breaks every current consumer (review-findings, self-review, `/pr-commented`); cost without benefit |
| Add `.progress.md` for `/bugfix` parallel to the trace file | Two artefacts per bug, conflicting lifecycles, both must be deleted at Step 7. Spec settles this: extend the trace. |
| Add `.progress.md` for `/interview` | Spec already settles: `<spec_path>` + `.state.md` are the durable surface |
| Skip the callout on `/verify` / `/pr-merged` entirely | Spec asks for the visible waiver — costs ~50 bytes, prevents future "why doesn't `/verify` have one?" confusion |
| Single global callout file `include`d from each skill | Claude Code skill loader doesn't include other files inline — each SKILL.md is a self-contained body. The DRY claim survives at *authoring* time (we copy the same wording) but not at runtime |
| Extract `/task` reference material to `ai-docs/templates/task-reference.md` | Single consumer — violates the "shared templates" convention in AGENTS.md *Agent Docs*. Belongs inside the skill directory as a supporting file |
| Conditional `current_step` write only on "significant" boundaries | Brittle to define; the rule "write at every step boundary" is mechanical and reliable. ~5 extra lines per progress file is cheap |
| **Single global callout body** ("restart from Step 1" version, Round-1) | **Rejected in Round 2.** "Step 1" mis-routes `/task` into the interview phase, mis-routes `/code-review` into "Determine branch" skipping the RESUME probe, makes `/bugfix` re-trace a user-confirmed trace, and is undefined for `/context-reset` (no numbered steps). Replaced by per-skill variants — see *Per-skill mapping* below. |
| **Force every skill's "first step" to be the active-state probe** (Round-1 attempt) | **Rejected in Round 2.** Three skills break this model intrinsically: `/task`'s probe is in the `⚡ First` preamble (Step 1 IS the interview); `/bugfix` Step 1 creates and blocks on the trace (not a probe); `/context-reset` has no numbered Steps at all. Adopting per-skill callout variants is cheaper than restructuring three skills' entry mode. |

### Per-skill mapping (compaction recovery callout → durable state + probe shape)

The mapping carries two columns Round-1 lacked: **probe shape** (glob vs
fixed path) and **callout variant** (which body wording each skill ships).

| Skill | Durable state surface | Probe shape | Callout variant |
|---|---|---|---|
| `/task` | `ai-docs/plans/<spec-base>.progress.md` | **Glob-discovered** — `⚡ First` preamble globs `ai-docs/plans/*.progress.md` then validates (stale-merge + branch-match) | **Variant A** (glob-based) |
| `/code-review` | `ai-docs/plans/YYYY-MM-DD-code-review.progress.md` | **Glob-discovered** — `⚡ First` preamble globs `ai-docs/plans/*.progress.md` (date is unknown to re-entering agent) | **Variant A** (glob-based) |
| `/pr-commented` | `ai-docs/plans/<spec-base>.progress.md` (derived via PR linkage in Step 1); fallback `ai-docs/pr-comments/pr-<N>.progress.md` | **PR-linkage-discovered** — Step 1 derives the path from the open PR number via `grep -l "Tracked in:.*#${PR_NUM}\b" ai-docs/plans/done/*.spec.md` | **Variant A** (glob-based — PR linkage is logically the same probe-then-read pattern) |
| `/bugfix` | `ai-docs/bugfix/trace-YYYY-MM-DD-<name>.md` (the trace file IS the progress file) | **Fixed-glob** — `ls ai-docs/bugfix/trace-*.md` returns exactly one in-flight trace per active bug | **Variant B** (fixed-path; explicit re-entry logic for confirmed-trace + Step ≥ 2) |
| `/interview` | `<spec_path>` + `<spec_path>.state.md` | **Glob-then-pair** — `ls ai-docs/plans/*.spec.md.state.md` then read both spec + state | **Variant B** (fixed-path-like; explicit "resume from round recorded in `.state.md`") |
| `/context-reset` | The active progress file for the **parent** workflow (`/task`, `/code-review`, or `/pr-commented`) | **Inherited** — no probe of its own; routes through whichever parent skill is active | **Variant C** (context-reset specific — no "Step", no "first step", points the agent back to the parent skill's callout) |

### Variant A — glob-based callout (`/task`, `/code-review`, `/pr-commented`)

```markdown
> **⚡ Compaction recovery check — read FIRST on every invocation.**
> If you are re-entering this skill after auto-compaction (a
> summary/compaction block appears at the top of context, or workflow
> context feels thin), STOP before any tool call and:
>
> 1. **Locate the durable-state file via this skill's active-state probe**
>    — run the preamble glob (`<PROBE_GLOB>`) and apply the validation it
>    documents (stale-merge, branch-match, or PR-linkage as the preamble
>    prescribes). The probe both finds the path AND decides whether to
>    RESUME, delete, park, or treat the situation as fresh.
> 2. Once the probe identifies the correct durable-state file
>    (`<DURABLE_STATE_PATH_DESCRIPTION>`), read it **top-to-bottom in one
>    pass** — every line, including older sections and the `## Decisions
>    log` section. Do not skim. The recorded `current_step` is a
>    cross-check, never an instruction to skip the read.
> 3. **Then re-enter this skill from the top of its body** — let the
>    preamble's probe / validation / RESUME sequence route control. Do
>    NOT jump to a numbered Step directly; the preamble owns the routing.
>    The probe will land you at the right next action without re-doing
>    completed work.
>
> If the probe finds no matching durable-state file (or returns a
> validated "no active task" result), this is a fresh invocation —
> proceed normally.
>
> See `.claude/skills/context-reset/SKILL.md` § **Compaction recovery
> (re-entry)** for the canonical handoff rationale.
```

Per-skill parameterisation of Variant A:

| Skill | `<PROBE_GLOB>` | `<DURABLE_STATE_PATH_DESCRIPTION>` |
|---|---|---|
| `/task` | `ls ai-docs/plans/*.progress.md 2>/dev/null` | "the matched `ai-docs/plans/<spec-base>.progress.md`" |
| `/code-review` | `ls ai-docs/plans/*.progress.md 2>/dev/null` | "the matched `ai-docs/plans/YYYY-MM-DD-code-review.progress.md`" |
| `/pr-commented` | `grep -l "Tracked in:.*#${PR_NUM}\b" ai-docs/plans/done/*.spec.md ai-docs/plans/*.spec.md` (where `PR_NUM` comes from `gh pr view --json number`) | "the matched `ai-docs/plans/<spec-base>.progress.md`" |

### Variant B — fixed-path callout (`/bugfix`, `/interview`)

```markdown
> **⚡ Compaction recovery check — read FIRST on every invocation.**
> If you are re-entering this skill after auto-compaction (a
> summary/compaction block appears at the top of context, or workflow
> context feels thin), STOP before any tool call and:
>
> 1. **Locate the durable-state file** — list `<FIXED_PATH_GLOB>`. If
>    exactly one in-flight artefact exists, that's the durable state. If
>    none exists, this is a fresh invocation. (Multiple matches: surface
>    to the user before continuing.)
> 2. Read it **top-to-bottom in one pass** — every line, including older
>    sections. Do not skim. The recorded `<RESUME_FIELD>` is a
>    cross-check, never an instruction to skip the read.
> 3. **Then re-enter this skill from the top of its body.** The body's
>    re-entry logic uses `<RESUME_FIELD>` (after the full read) to skip
>    user-confirmed checkpoints that need not be redone — `<RESUME_RULE>`.
>
> If `<FIXED_PATH_GLOB>` returns no matches, this is a fresh invocation —
> proceed normally.
>
> See `.claude/skills/context-reset/SKILL.md` § **Compaction recovery
> (re-entry)** for the canonical handoff rationale.
```

Per-skill parameterisation of Variant B:

| Skill | `<FIXED_PATH_GLOB>` | `<RESUME_FIELD>` | `<RESUME_RULE>` |
|---|---|---|---|
| `/bugfix` | `ls ai-docs/bugfix/trace-*.md 2>/dev/null` | `current_step` | "if the trace's `Confirmed by user: ✅ YES` line is present, do NOT re-run Step 1's reproduce-and-trace user-confirmation; resume from the step recorded in `current_step` (Step 2 Root Cause onward)" |
| `/interview` | `ls ai-docs/plans/*.spec.md.state.md 2>/dev/null` (then read both the matched `.state.md` AND its sibling `<spec_path>`) | `round` (from the `.state.md` YAML block) | "resume from the round recorded in `.state.md`'s `round:` field; do NOT restart at round 1, and do NOT re-create the state file" |

### Variant C — `/context-reset` self-callout

`/context-reset` has no numbered Steps and is the **cross-link
destination** for every other skill's callout. A recursive self-reference
adds noise; the callout here is short and explicitly punts to whichever
parent skill is active.

```markdown
> **⚡ Compaction recovery check — read FIRST on every invocation.**
> If you are re-entering `/context-reset` after auto-compaction (a
> summary/compaction block appears at the top of context, or workflow
> context feels thin), STOP before any tool call and:
>
> 1. Identify the **parent workflow** (`/task`, `/code-review`, or
>    `/pr-commented`) whose handoff `/context-reset` is performing. The
>    parent's identity is recorded as the active progress file's
>    `parent_skill:` field (or `current_step:` mentions a `/task` /
>    `/code-review` / `/pr-commented` step name).
> 2. Run the parent skill's own compaction-recovery callout against its
>    durable-state file (see `/task`, `/code-review`, or `/pr-commented`
>    SKILL.md for the parent's variant). `/context-reset`'s body is the
>    shared handoff + re-prime action, not a separate durable surface.
>    (The parent's Variant-A callout ends with a "See `/context-reset`
>    § Compaction recovery (re-entry)" cross-link — that is a *reading*
>    link to the canonical rationale, **not** an instruction to re-run
>    this Variant-C callout. The chain terminates at the parent's active
>    subtask.)
> 3. After the parent's callout routes you to the active subtask, follow
>    `/context-reset`'s **Handoff protocol** below for the actual handoff
>    or re-prime action.
>
> This skill carries the canonical rationale below (§ **Compaction
> recovery (re-entry)**) for the other skills to cross-link to.
```

### Shared "Compaction recovery (re-entry)" h2 in `/context-reset` SKILL.md

The cross-link target anchor (used by every Variant-A and Variant-B
callout) is the **singular** h2:

```markdown
## Compaction recovery (re-entry)
```

This h2 sits alongside the existing `## Handoff protocol` h2 (which
covers the N=3-of-M≥5 auto-trigger). Both h2 sections share the same
final action (write a handoff state + re-prime); they differ in their
trigger condition and prelude. **Singular h2 anchor — not two parallel
preludes.** The cross-link `See .claude/skills/context-reset/SKILL.md §
**Compaction recovery (re-entry)**` resolves to this exact anchor in
every callout.

#### Wording rationale (post-Round-2)

- "**Read FIRST on every invocation**" — heuristic trigger. Sonnet
  should treat the callout as the very first instruction even on a
  non-compacted invocation, where it becomes a cheap no-op (the
  existing preamble probe already does most of the same work).
- "**STOP before any tool call**" — prevents the "just one quick grep
  before loading state" failure mode. Every tool call is information
  the agent could have read from the durable-state file.
- "**top-to-bottom in one pass**" — counters the temptation to jump to
  the most-recent `## Self-Review (Round N)` or `## Comment cycle round
  M` section and skip everything before it.
- "**re-enter this skill from the top of its body**" — replaces the
  Round-1 "restart from Step 1" wording. The skill body's *top* is the
  preamble for `/task` / `/code-review` / `/pr-commented` (where the
  probe lives) and the first step for `/bugfix` / `/interview`. The
  wording covers all six skills uniformly without misrouting any of
  them. Round-1's "Step 1" was load-bearing for the option name, not a
  technical claim about skill structure — the design now reflects that.
  This is the **Full-read-on-re-entry invariant** named in the spec's
  *Technical constraints* (Round-2 amendment 1).
- "**cross-check, never an instruction to skip the read**" — restates
  the `current_step`-is-a-hint invariant. Even for Variant B where the
  body uses `current_step` to skip user-confirmed gates, the FULL READ
  happens first.
- "**See `.claude/skills/context-reset/SKILL.md` § Compaction recovery
  (re-entry)**" — cross-link required by AC7. Exact anchor named.
  Concentrates the rationale in one place; the callouts stay short.

### Schema extension (canonical, in `ai-docs/templates/progress-format.md`)

The new fields land as header lines in the same shape as the existing
`**Branch:**` / `**base_commit:**` / `**Last build:**`:

```markdown
**Branch:** [branch name]
**base_commit:** [git rev-parse HEAD output]
**Last build:** PASS / FAIL / not run
**current_step:** [Step name or "Step N: short label"]
**last_passed_gate:** [command | ISO-8601 timestamp | commit SHA]
**parent_skill:** [optional — outer skill name when this progress is owned by a nested skill]
**entry_args:** [original $ARGUMENTS at progress-file creation — bare issue ref, spec-base, free-text, or empty]

## Decisions log

- Step <N>: <one-line description of a non-trivial decision>
- Step <N>: <…>
```

- `current_step` (required): rewritten at every step-boundary write.
  Format: the step name or `Step N: short label`. `/code-review` writes
  its phase-name (`Phase 1 — review-findings`), `/pr-commented` writes
  `Round M Step N`, `/bugfix` writes `Step N` from the trace.
- `last_passed_gate` (required, single field): rewritten at every step
  boundary where a gate just passed. Format: `<command> | <ISO-8601 UTC
  timestamp> | <commit SHA from `git rev-parse HEAD`>`. Example:
  `cargo clippy --workspace -- -D warnings | 2026-05-14T18:42:11Z |
  549282b`. Single field (per Open-questions answer in spec). Each new
  pass overwrites the previous; no per-gate history kept. Rationale:
  re-entry needs "what was last green here" not "everything green so
  far".
- `parent_skill` (optional): present when the progress file is owned by
  a nested skill. `/bugfix` invoked from `/task` Step 8 writes
  `**parent_skill:** /task` in its trace. Empty / omitted when
  standalone.
- `entry_args` (**Round-2 addition** — required for `/task`, optional
  for the other consumers): records the original `$ARGUMENTS` at
  progress-file creation. Major-5 resolution: on re-entry after
  compaction, `$ARGUMENTS` is lost; `/task` must know whether the
  original entry was a bare issue number (`⚡ Third` preamble), a
  keyword like `activate`/`start`/`proceed` (`⚡ Second` preamble), an
  active-task probe path (`⚡ First` preamble), or fresh free text
  (Steps 1–5). Without `entry_args`, the re-entering agent has no way
  to disambiguate which preamble owns the routing. Format: free-text
  copy of `$ARGUMENTS`, single line. Empty / `(none)` for
  empty-argument invocations. Values like `#348` / `348` / `activate
  paint-style` / `add foo to bar` / `(none)` are all valid. **On
  re-entry, the active-state probe's behaviour is documented in the
  preamble** (see *`/task` re-entry routing* below).
- `decisions_log` (required, append-only): a `## Decisions log` section
  with one bullet per non-trivial decision, prefixed by the step that
  made it. Append-only — every step boundary that made a decision adds
  a new bullet; never edit older bullets. Distinct from the existing
  `## Key discoveries` section (which is a read-time hints box, not a
  per-step audit trail).

**Where each consumer writes which field:**

| Consumer | `current_step` | `decisions_log` | `last_passed_gate` | `parent_skill` | `entry_args` |
|---|---|---|---|---|---|
| `/task` Step 8 (progress-file creation) | initial value | initial entry | initial value (post-Step-8 cargo build) | only if `/task` itself was invoked from another skill (rare) | **required** — copy of `$ARGUMENTS` at Step 8 |
| `/task` Steps 9–12 | yes, per step | yes, per non-trivial decision | yes, after each gate-passing step | unchanged | unchanged (read-only after Step 8) |
| `/code-review` Phases 1–5 | yes, per phase | yes, per non-trivial decision | yes, after Step 4 gates | n/a | optional — branch name if non-default |
| `/pr-commented` per round | yes, per Step 1–7 within round | yes (inside the round section) | yes, after Step 4 gates | n/a | optional — PR number for traceability |
| `/bugfix` Steps 1–7 (writes into trace file) | yes, per step | yes | yes, after Step 6 gates | `/task` when invoked from `/task` Step 8 | optional |
| `/interview` | **n/a** — `.state.md` is its durable round counter | n/a | n/a | n/a | n/a (state.md captures `issue_ref` / `spec_path`) |

`review-findings` and `self-review` agents are *readers*: they don't
write the new fields themselves, but the skills that spawn them must
surface the new schema in the spawn prompt so the agents recognise the
layout. The agents' progress-file format example sections (in their
`.md` files) get updated to show the new header fields verbatim.

### `/task` re-entry routing with `entry_args` (Major-5 resolution)

`/task` has three preambles: `⚡ First` (active-task probe), `⚡ Second`
(deferred-plan keyword activation), `⚡ Third` (bare-issue activation).
After compaction `$ARGUMENTS` is gone — re-running the preambles in
order risks mis-firing `⚡ Second` or `⚡ Third` on a stale `$ARGUMENTS`
or no `$ARGUMENTS`.

**Resolution: documented routing rule, encoded in `/task` SKILL.md by
subtask 8.** The `⚡ First` preamble grows an explicit clause:

> **Re-entry-after-compaction case.** If `$ARGUMENTS` is empty (lost to
> compaction) AND the glob `ls ai-docs/plans/*.progress.md` finds a
> matching file with `entry_args:` recorded, treat the recorded
> `entry_args` as the canonical entry reference. The `⚡ Second` and
> `⚡ Third` preambles must NOT fire on the **lost-arguments path** —
> they require a positive match against the live `$ARGUMENTS`, which
> by definition is unavailable on re-entry. Only `⚡ First` (active-
> task probe) is allowed to route a lost-arguments re-entry. If the
> probe finds no matching progress file, the re-entry is "fresh";
> surface this to the user (do NOT proceed to Steps 1–5 silently,
> because the user's original task is unknown).

This is enforced by subtask 8's AC: "the `⚡ First` preamble of `/task`
SKILL.md documents the lost-arguments re-entry case AND `⚡ Second` /
`⚡ Third` preambles each include a guard sentence saying they only fire
when `$ARGUMENTS` is non-empty (otherwise fall through to `⚡ First` or
treat as fresh)."

The `entry_args` field is recorded once at Step 8 progress-file
creation and never overwritten (read-only after creation). Steps 9–12
do NOT touch it.

### `/bugfix` re-entry logic (Blocker-3 resolution)

The Round-1 design claimed `/bugfix` Step 1 was "the active-state
probe". That is **factually false**: Step 1 *creates* the trace artefact
and *blocks on user confirmation* (`Confirmed by user: ✅ YES`). On
re-entry with a confirmed trace, re-running Step 1 would re-ask the user
to confirm what's already confirmed.

**Resolution: explicit re-entry logic in `/bugfix` Step 1.** Subtask 5
encodes the following at the top of Step 1:

> **Re-entry-after-compaction case.** If a trace file exists at
> `ai-docs/bugfix/trace-*.md`, read it top-to-bottom (per the Variant-B
> callout's instruction). Then:
>
> | Trace state | Action |
> |---|---|
> | `Confirmed by user: ⏳ PENDING` (or missing) | Re-execute Step 1 normally — the trace was created but the user never confirmed. Re-show it and ask confirmation. |
> | `Confirmed by user: ✅ YES` AND `**current_step:**` ≥ Step 2 | **Skip Step 1**. Resume from the step recorded in `**current_step:**`. Do NOT re-trace, do NOT re-ask the user. |
> | `Confirmed by user: ✅ YES` but `**current_step:**` missing or blank | Treat as Step 1 just finished; resume at Step 2. |
> | Multiple matching trace files | Surface to user; do NOT auto-pick. |

The Variant-B callout's `<RESUME_RULE>` for `/bugfix` (table above)
points to this exact re-entry logic. The callout routes control to the
skill body; the body's re-entry logic then decides whether to skip
Step 1.

`/interview` gets the same shape via its `.state.md` `round:` field —
the callout points the agent at `.state.md`, the body's Step 1
("Detect entry mode") reads the round counter and routes accordingly.
No re-entry logic change needed in `/interview` itself beyond the
callout (Step 1 already handles a present `.state.md` via the existing
flow).

### `/bugfix` trace-file extension (heading vs `**Field:**`)

The existing trace template uses BOTH shapes:

- `Date: YYYY-MM-DD` and `Reporter: <quote>` — plain `Key: value` lines
- `## Actual behaviour` / `## Expected behaviour` / `## Root Cause` —
  h2 headings with multi-paragraph bodies
- `Confirmed by user: ✅ YES` — inline status line

The new fields fit the **header `**Field:**` shape** (analogous to the
`.progress.md` header fields and to the existing `Date:` / `Reporter:`
lines), with `## Decisions log` as an h2 section to match the existing
multi-paragraph pattern:

```markdown
# Bugfix Trace: <bug description>
Date: YYYY-MM-DD
Reporter: <quote from user message>

**current_step:** Step <N>: <label>
**last_passed_gate:** [command | ISO-8601 timestamp | commit SHA]
**parent_skill:** /task | (omitted)
**entry_args:** [optional — copy of `$ARGUMENTS` at trace creation]

## Actual behaviour
...
## Expected behaviour
...

## Confirmed by user: ⏳ PENDING / ✅ YES

## Decisions log

- Step <N>: ...

## Root Cause
...
```

Rationale: the trace's existing top-band is key-value, so the new
short-form fields belong there. The decisions log is a list that grows
across steps — same shape as `## Root Cause` / `## Self-Review (Round
N)`. Open question OQ-4 in the spec ("Section heading style for the
`/bugfix` trace-file extensions") is resolved by mirroring the existing
pattern: the trace already uses both shapes, and matching each new
field to its closest existing analogue minimises template churn.

### Sequencing within the PR

Per spec *Notes for the design agent*: the small reorders for
non-`/task` SKILL.md files land **before** the large `/task` extraction
(incremental risk). The decomposition orders subtasks accordingly:
schema first (1), shared callout variants drafted once (2), small skill
files updated (3–7), then the big `/task` extraction (8), then
propagation rule fan-out to agents (9–10), AGENTS.md table updates
(11), char-cap audit (12), final verification (13).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Extend the canonical `.progress.md` schema with `current_step`, `decisions_log`, `last_passed_gate`, `parent_skill`, `entry_args`. Add a "Lifecycle by field" sub-section enumerating which consumer writes which field. Update the inline example. Document `entry_args` as required for `/task` progress files (recorded at Step 8 creation, read-only thereafter). AC3. | `ai-docs/templates/progress-format.md` | — |
| 2 | Draft the three shared "Compaction recovery check" callout variants once in this design doc (already done above — Variants A, B, C). Lock the wording for AC1. No file changes in this subtask — the wording is a deliverable other subtasks copy verbatim per the table mapping each skill to its variant. | (design doc) | — |
| 3 | Apply Variant C callout + workflow-on-top reorder to `/context-reset` SKILL.md. Add the singular h2 anchor `## Compaction recovery (re-entry)` (distinct from existing `## Handoff protocol`). Two preludes share the bottom "Handoff protocol" + "Rules" sections as common action. Cross-link target anchor verified by `rg "^## Compaction recovery \\(re-entry\\)$" .claude/skills/context-reset/SKILL.md`. AC7. | `.claude/skills/context-reset/SKILL.md` | 2 |
| 4 | Apply Variant B callout + small reorder to `/interview` SKILL.md. Callout names `<spec_path>.state.md` glob and points the resume rule at the `round:` field. Step 1 ("Detect entry mode") already handles `.state.md` presence — no change to Step 1 itself, but verify the resume path matches the callout's claim. AC5. | `.claude/skills/interview/SKILL.md` | 2 |
| 5 | Apply Variant B callout + small reorder to `/bugfix` SKILL.md. Callout names the trace-file glob. **Step 1 grows the re-entry-after-compaction decision table** (per *`/bugfix` re-entry logic* in this design) — handles confirmed-trace, pending-trace, and missing-`current_step` cases. Add the four new schema fields to the trace template at Step 1 (header `**Field:**` lines + `## Decisions log` h2). Step instructions wire writes at each step boundary. Move "Anti-pattern: fix-break cycle" to the bottom (already at bottom). AC4 (bugfix half), AC11 (gitignore — no change needed). | `.claude/skills/bugfix/SKILL.md` | 1, 2 |
| 6 | Apply Variant A callout + reorder to `/code-review` SKILL.md. Callout's `<PROBE_GLOB>` is the existing `⚡ First` preamble's `ls ai-docs/plans/*.progress.md`. Wire per-phase writes of `current_step` / `decisions_log` / `last_passed_gate` to the progress file. Move "Gate checklist" + "Commit rules" to the bottom (already structured that way). AC4 (code-review half). | `.claude/skills/code-review/SKILL.md` | 1, 2 |
| 7 | Apply Variant A callout + reorder to `/pr-commented` SKILL.md. Callout's `<PROBE_GLOB>` is the existing `grep -l "Tracked in:.*#${PR_NUM}\\b"` derivation in Step 1. Wire per-Step (1–7) writes within each round to the existing `## Comment cycle round M` section (the new fields nest inside the latest round section, no top-level duplication). Move "Edge cases" / "Anti-patterns" / "Gate checklist" to the bottom (already there). AC4 (pr-commented half). | `.claude/skills/pr-commented/SKILL.md` | 1, 2 |
| 8 | Apply Variant A callout + reorder + reference extraction to `/task` SKILL.md. Callout's `<PROBE_GLOB>` is the existing `⚡ First` preamble's `ls ai-docs/plans/*.progress.md`. **Add the lost-arguments routing clause** (per *`/task` re-entry routing with `entry_args`* in this design) to the `⚡ First` preamble. **Add guard sentences** to `⚡ Second` and `⚡ Third` preambles stating they only fire when `$ARGUMENTS` is non-empty. **Record `entry_args` in the Step 8 progress-file creation template.** Move workflow steps (1–12 + Design Amendment) to the top right after the callout + commit-authorisation paragraph + entry-mode paragraph. Extract these reference blocks to `.claude/skills/task/reference.md`: (a) the validation-sequence details inside preambles 1–3 (the four-step validation in section 1; the bare-issue activation decision table in section 3); (b) the Design Amendment recipe; (c) the FORBIDDEN list; (d) the Gate checklist; (e) the per-step verbose narrative paragraphs (N=3 handoff gate rationale, local-FAIL investigation paragraph, panic-index sync detail). SKILL.md keeps a short cross-link footer ("Reference: `reference.md` — anti-patterns, gate checklist, Design Amendment recipe, validation procedures"). Wire writes of new schema fields at each Step 8–12 boundary. AC2, AC4 (task half). | `.claude/skills/task/SKILL.md`, `.claude/skills/task/reference.md` (new) | 1, 2 |
| 9 | Apply one-line `.progress.md` waiver to `/verify` and `/pr-merged` SKILL.md files: "Near-stateless: no `.progress.md` discipline applies; re-entry consists of re-invoking the skill." Placement: immediately after the front matter, before any step instructions (consistent with where the callout lands in the orchestrator skills — visible at the top of a truncated file). AC6. | `.claude/skills/verify/SKILL.md`, `.claude/skills/pr-merged/SKILL.md` | 2 |
| 10 | Propagation Rule — Review group. Update `review-findings.md` and `self-review.md` progress-file format example blocks to show the five new header fields (including `entry_args`). Update the "What you do/don't check" sections to note that the new fields exist (no review responsibility on them, but the format example must match the canonical template). AC8 (Review group). | `.claude/agents/review-findings.md`, `.claude/agents/self-review.md` | 1 |
| 11 | Propagation Rule — Interview group. Update `spec-writer.md` to note (in its read-list) that `/interview` now carries a compaction-recovery callout but the spec-writer subagent does NOT write a `.progress.md`; its durable state remains the in-flight spec + `.state.md`. Minimal: one-line addition under "Read before drafting". AC8 (Interview group). | `.claude/agents/spec-writer.md` | 4 |
| 12 | AGENTS.md "Agent Docs" table updates: the `ai-docs/plans/*.progress.md` row mentions the extended schema (`current_step` / `decisions_log` / `last_passed_gate` / `parent_skill` / `entry_args` writers); the `ai-docs/bugfix/trace-*.md` row notes the trace now carries the same fields inline. AC8 (AGENTS.md half). | `AGENTS.md` | 1, 5 |
| 13 | Char-cap audit + variant-identity audit. Run `wc -c` against every code-side SKILL.md and `reference.md`. **Required PR-body deliverables** (explicit checklist, not buried in audit script comments): (a) char counts per file with 35k-cap and 20k-target pass/fail flags; (b) per-skill variant identity from the *Per-skill mapping* table — `/task` = A, `/code-review` = A, `/pr-commented` = A, `/bugfix` = B, `/interview` = B, `/context-reset` = C — each row paired with the variant-distinguishing phrase grep result (Variant A: `"Locate the durable-state file via this skill's active-state probe"`; Variant B: `"If exactly one in-flight artefact exists"`; Variant C: `"Identify the **parent workflow**"`). Verify every file ≤ 35,000 chars (AC10) and `/task` SKILL.md ≤ ~20,000 chars (AC2). Verify the cross-link anchor `## Compaction recovery (re-entry)` exists in `/context-reset` SKILL.md exactly once. If `/task` SKILL.md is still over 20,000 chars after subtask 8, extract more reference material into `reference.md` (priority order: Step 12 inbox-propagation narrative, then Step 9 panic-index detail, then Step 11 review-fix narrative). | (audit only — re-edits if needed) | 3–9 |

13 subtasks. The spec's "Notes for the design agent" warned that >7
subtasks should prompt the design agent to consider splitting into
multiple issues. Here, however, all 13 subtasks are bundled into one PR
by explicit user direction in the spec's *Technical constraints*
("Bundled single PR for all changes (user-directed in the issue)"). The
bundling is a hard requirement, not a designer choice — splitting is
forbidden by spec. Each subtask is independently small (the largest,
#8, is the `/task` extraction that the spec also calls out as "the only
sizable refactor"). The 13-task count reflects the breadth of the
Propagation Rule fan-out (6 orchestrator skills × 1 callout + 2 waiver
skills + 3 sync-group agents + 2 schema/AGENTS tables + 1 audit), not a
single piece of work that would benefit from sharding across PRs.

The design groups subtasks 3–9 (the per-file callout / reorder passes)
so each can ship as a single atomic commit. Subtask 8 is the only one
that may need to span two commits (extract first, then trim — `cargo
build` between them is a no-op since no Rust files change). Subtask 13
is an audit gate, not a normal subtask; if it fails, the failing
subtask gets re-opened and a follow-up commit lands the extra
extraction.

### Suggested commit boundary mapping

| Subtask range | Suggested commit |
|---|---|
| 1 | `docs(progress): extend canonical schema with current_step / decisions_log / last_passed_gate / parent_skill / entry_args` |
| 3 | `docs(skills): /context-reset — compaction-recovery section + variant-C callout + reorder` |
| 4 | `docs(skills): /interview — variant-B callout` |
| 5 | `docs(skills): /bugfix — extend trace schema + variant-B callout + Step-1 re-entry logic + per-step writes` |
| 6 | `docs(skills): /code-review — variant-A callout + per-phase progress writes + reorder` |
| 7 | `docs(skills): /pr-commented — variant-A callout + per-step progress writes` |
| 8 | `docs(skills): /task — variant-A callout, entry_args routing, reorder, extract reference.md` (possibly two commits if extract / trim split) |
| 9 | `docs(skills): /verify + /pr-merged — near-stateless waivers` |
| 10 | `docs(agents): review-findings + self-review — recognise extended progress schema` |
| 11 | `docs(agents): spec-writer — note /interview callout (no schema change)` |
| 12 | `docs(agents-md): Agent Docs table — extended schema; trace carries same fields` |
| 13 | `docs(skills): char-cap audit (no edits if pass; trim if fail)` |

Each commit is self-contained per the AGENTS.md *Workflow* rule "stage
files explicitly by name". No commit straddles two Propagation Rule
sync groups (a finer split would be over-engineered for a documentation
PR; this granularity matches the existing PR-per-skill cadence).

## Risks

| Risk | Mitigation |
|---|---|
| **Callout wording drifts across the three variants.** A future PR edits one variant, the others stay stale; OR a per-skill variant drifts away from the locked template in this design. | Subtask 2 locks the three variants in this design doc; each subtask's PR-review prompt cross-references this section. Subtask 13's char-cap audit also greps for the variant-specific anchor phrases (e.g. "Locate the durable-state file via this skill's active-state probe" for Variant A, "if exactly one in-flight artefact exists" for Variant B) and counts hits to flag drift. **Long-term mitigation** deferred to *Open questions* — generalising into a reusable include once the variant taxonomy stabilises. |
| **Per-skill callout variants drift over time** (Round-2-added risk). | Char-cap audit (subtask 13) records both the file size AND the variant identity per skill; PR body lists each skill's variant alongside its char count. Propagation Rule fires whenever any callout edits; the sync-group siblings in this design are explicit (Variant A: 3 skills; Variant B: 2 skills; Variant C: 1 skill). A future audit can `rg` for the variant-distinguishing phrases in each file. |
| **Blocker-1 misrouting** (Round-1 failure mode): callout's "Step 1 of this skill performs the active-state lookup" mis-routes `/task` / `/code-review` / `/pr-commented` into the interview / branch-determination / progress-file-derivation step, skipping the active-state probe in the preamble. | **Resolved by Round-2 amendment.** Callout body Variant A explicitly routes "via this skill's active-state probe" (the preamble glob) and instructs "re-enter this skill from the top of its body" — covering preambles uniformly. Variant B uses a fixed glob and explicit resume rule. Variant C punts to the parent skill. The "Step 1" wording is removed everywhere. |
| **`/task` SKILL.md still > 20,000 chars after subtask 8.** | Subtask 13 audit catches it; extraction priority list (Step 12 inbox-propagation narrative → panic-index detail → review-fix narrative) is pre-ranked so the trim is mechanical. |
| **Schema extension breaks `review-findings` / `self-review` parse expectations.** | Subtask 10 updates both agent files in the same PR (Propagation Rule). The new fields are *additions*, not renames of existing ones; existing reads of `**Branch:**` / `**base_commit:**` / `**Last build:**` keep working. |
| **`/bugfix` trace template doubles in size.** | The four new lines add ~150 chars; trace files are already short, no risk of crossing the 1000-line file-size soft limit. |
| **Full-read-on-re-entry invariant is non-obvious — agent may "optimise" by jumping to `current_step`.** (Spec *Technical constraints* — renamed from Round-1 "Restart-Step-1 invariant" per Round-2 amendment 1.) | The callout body says this explicitly twice (sub-bullet 2 + "cross-check, never an instruction to skip the read"). Reinforced by the cross-link to `/context-reset` § Compaction recovery (re-entry). If the failure mode persists post-merge, the wording becomes load-bearing under the spec's open question and gets a `/improve` cycle. |
| **`/interview` callout points to two paths.** Sonnet may read only one. | Callout body Variant B for `/interview` enumerates both `<spec_path>` and `<spec_path>.state.md` with a single "read both" instruction via the `<FIXED_PATH_GLOB>` ("then read both the matched `.state.md` AND its sibling `<spec_path>`"). The state file is short (single fenced YAML block per spec); the spec is the long one. |
| **`/pr-commented` round-by-round nesting confuses re-entry — which round's `current_step` wins?** | The per-round section pattern (`## Comment cycle round M`) is already round-indexed. The new fields nest **inside the latest round section**, not at the top of the file. Top-level fields stay `/task`'s; round-specific fields live under the latest `## Comment cycle round M`. Documented in subtask 7. |
| **Char-cap regression on next `/task`.** Future SKILL.md edits push `/task` back over 20 KB. | A `scripts/check-instruction-file-sizes.sh` was already named in AGENTS.md as a future pre-commit gate. Not introduced here (out of scope per spec deferred section). The 35 KB early-warning still fires before the 40 KB harness cap. |
| **Existing `.progress.md` files on disk lack the new required fields.** Pre-publish, no downstream users, but the user's own active `/task` session might be running on a pre-PR progress file. | Spec is pre-publish (AGENTS.md *API Stability*) — no migration shim. If the user has an in-flight progress file, the new fields are absent; the re-entering agent treats their absence as "fresh `/task`, no recorded next step" and follows the existing `⚡ First` validation sequence (which IS the correct behaviour). No data loss, no broken flow. **`entry_args` absent on a pre-PR progress file**: subtask 8 documents that an absent `entry_args` triggers the user-prompt fallback ("Cannot disambiguate original entry mode; surface to user before proceeding") rather than silently proceeding to Steps 1–5. |
| **`/context-reset` is invoked as both subagent handoff (existing N=3-of-M≥5) AND as direct re-entry surface (new).** | Subtask 3 adds the singular h2 `## Compaction recovery (re-entry)` alongside the existing `## Handoff protocol`. The two preludes share the bottom "Handoff protocol" + "Rules" sections as the common action. Variant-C callout at the top routes to the parent skill, avoiding recursive self-reference noise. |
| **Major-5 mis-routing on `/task` re-entry without `$ARGUMENTS`.** | Resolved by `entry_args` field + `⚡ First` lost-arguments clause + `⚡ Second`/`⚡ Third` guard sentences (subtask 8 ACs). |
| **Blocker-3 — `/bugfix` re-traces a user-confirmed trace.** | Resolved by Step 1 re-entry decision table (subtask 5) — confirmed-trace + `current_step` ≥ Step 2 skips Step 1 entirely. |
| **Blocker-4 — `/context-reset` callout undefined.** | Resolved by Variant-C callout (no "Step 1" reference) + singular h2 cross-link anchor `## Compaction recovery (re-entry)`. |

## Test Design

This is a documentation / instruction-file PR — no Rust code, no
`#[cfg(test)]` modules. "Tests" map to manual verification gates and
automated text checks. None of the acceptance criteria are auto-gated
by the existing CI surface (Rust build / clippy / fmt / doc /
actionlint); verification is text-grep and post-merge spot-check.

### Per-AC verification plan

| AC | Verification |
|---|---|
| AC1 | `rg "Compaction recovery check" .claude/skills/{task,interview,bugfix,code-review,pr-commented,context-reset}/SKILL.md` returns 6 hits, one per file. Manual diff: callout body is identical **within each variant group** (spec AC1 wording, Round-2 amendment 2) per the *Per-skill mapping* table — Variant A across `/task` / `/code-review` / `/pr-commented`, Variant B across `/bugfix` / `/interview`, Variant C for `/context-reset`. Each instance names its own probe / durable-state file per the per-skill parameterisation tables under each Variant section. Variant-distinguishing phrases verified: Variant A uses "Locate the durable-state file via this skill's active-state probe"; Variant B uses "If exactly one in-flight artefact exists"; Variant C uses "Identify the **parent workflow**". The phrase "re-enter this skill from the top of its body" (the Full-read-on-re-entry invariant's surface wording) appears in Variants A and B; Variant C uses "Run the parent skill's own compaction-recovery callout" — both forms encode the same invariant. |
| AC2 | `wc -c .claude/skills/task/SKILL.md` ≤ 20,000. `wc -c .claude/skills/task/reference.md` exists. SKILL.md contains the line `Reference: .claude/skills/task/reference.md` near the bottom. |
| AC3 | `rg "^\\*\\*current_step:\\*\\*" ai-docs/templates/progress-format.md` returns ≥ 1; same for `last_passed_gate`, `parent_skill`, `entry_args`. `rg "## Decisions log" ai-docs/templates/progress-format.md` returns ≥ 1. Existing fields (`**Branch:**`, `**base_commit:**`, `**Last build:**`) still present. |
| AC4 | `rg "current_step" .claude/skills/{task,code-review,pr-commented,bugfix}/SKILL.md` returns hits in all four. `/bugfix` trace template (inside SKILL.md Step 1) carries the four new fields. No new `.progress.md` artefact created by `/bugfix`. `/task` SKILL.md `⚡ First` preamble carries the lost-arguments clause; `⚡ Second` and `⚡ Third` carry the `$ARGUMENTS` guard sentences (`rg "non-empty" .claude/skills/task/SKILL.md`). |
| AC5 | `/interview` callout text mentions `<spec_path>` and `.state.md` (not `.progress.md`). `rg "\\.progress\\.md" .claude/skills/interview/SKILL.md` returns zero hits. |
| AC6 | `rg "Near-stateless" .claude/skills/{verify,pr-merged}/SKILL.md` returns one hit per file. |
| AC7 | `/context-reset` SKILL.md contains the singular h2 `## Compaction recovery (re-entry)` (`rg "^## Compaction recovery \\(re-entry\\)$" .claude/skills/context-reset/SKILL.md` returns exactly 1). Each code-side orchestrator callout body contains `See .claude/skills/context-reset/SKILL.md § **Compaction recovery (re-entry)**`. |
| AC8 | Review group: `rg "current_step" .claude/agents/{review-findings,self-review}.md` returns hits; `entry_args` also present in both files' format example. Interview group: `spec-writer.md` mentions the callout in `/interview`. AGENTS.md "Agent Docs" table rows for `ai-docs/plans/*.progress.md` and `ai-docs/bugfix/trace-*.md` mention extended schema (including `entry_args`). |
| AC9 | Manual post-merge spot-check — user runs a Sonnet `/task` that triggers compaction; verifies full read of the progress file, then re-entry from the top of the skill body, routed by the **active-task probe** (spec AC9 wording, Round-2 amendment 3 — concretely the `⚡ First` preamble glob for `/task`; NOT via "Step 1", which is the interview phase). Outcome recorded in `ai-docs/learnings.md`. Not gated in CI. (Subtask 13 is the closest equivalent gate: char-cap audit + presence check + variant-identity audit.) **Terminology bridge for future audits:** AC9's "active-task probe" is the spec-side surface string for the same mechanism the design body + locked Variant A body + subtask 13 grep audit call "active-state probe" — see *Spec-amendment history* row 3 Terminology note. Do NOT add a second `rg` on "active-task probe" to the audit recipe: the locked Variant A body uses "active-state probe" verbatim and changing it would break subtask 13's grep audit. |
| AC10 | `wc -c .claude/skills/*/SKILL.md` — every file < 35,000 chars. Listed verbatim in the PR body. |
| AC11 | `git status` after subtasks 1–13 shows no new untracked files outside `ai-docs/plans/` and `.claude/skills/task/reference.md`. `.gitignore` not modified. (The spec confirms no new artefact paths require gitignore additions.) |

### Verification scripts (one-off, run during subtask 13)

A short audit recipe to run at the end of subtask 13, included verbatim
in the PR body:

```bash
# AC2 + AC10 char-cap audit
wc -c .claude/skills/{task,interview,bugfix,code-review,pr-commented,context-reset,verify,pr-merged}/SKILL.md .claude/skills/task/reference.md

# AC1 callout presence
rg -c "Compaction recovery check" .claude/skills/{task,interview,bugfix,code-review,pr-commented,context-reset}/SKILL.md

# AC1 variant-distinguishing phrases
rg -c "Locate the durable-state file via this skill's active-state probe" .claude/skills/{task,code-review,pr-commented}/SKILL.md  # Variant A — expect 1 each
rg -c "If exactly one in-flight artefact exists" .claude/skills/{bugfix,interview}/SKILL.md                                         # Variant B — expect 1 each
rg -c "Identify the \\*\\*parent workflow\\*\\*" .claude/skills/context-reset/SKILL.md                                              # Variant C — expect 1

# AC1 invariant phrase (Variants A & B)
rg -c "re-enter this skill from the top of its body" .claude/skills/{task,interview,bugfix,code-review,pr-commented}/SKILL.md

# AC3 schema fields
rg "^\\*\\*current_step:\\*\\*|^\\*\\*last_passed_gate:\\*\\*|^\\*\\*parent_skill:\\*\\*|^\\*\\*entry_args:\\*\\*|^## Decisions log$" ai-docs/templates/progress-format.md

# AC4 /task entry_args routing
rg "non-empty" .claude/skills/task/SKILL.md  # expect hits in ⚡ Second and ⚡ Third guard sentences

# AC5 /interview no progress.md
rg -c "\\.progress\\.md" .claude/skills/interview/SKILL.md  # expect 0

# AC6 waivers
rg -c "Near-stateless" .claude/skills/{verify,pr-merged}/SKILL.md

# AC7 /context-reset singular h2 + cross-link
rg -c "^## Compaction recovery \\(re-entry\\)$" .claude/skills/context-reset/SKILL.md  # expect 1
rg -c "Compaction recovery \\(re-entry\\)" .claude/skills/{task,interview,bugfix,code-review,pr-commented,context-reset}/SKILL.md  # expect hits in all 6
```

## Open questions

The spec lists five open questions ("Open questions" section); all five
are either resolved by this design or explicitly defended below.

- **Exact wording of the "Compaction recovery check" callout** —
  *Resolved by this design.* See *Approach → Variant A / B / C*.
  Locked across three variants per the per-skill mapping. If a
  user-visible phrase becomes load-bearing post-merge, the wording
  revisits via a `/improve` cycle.
- **Whether `decisions_log` lines should also be echoed to the user at
  each step boundary** — *Picked the lighter touch.* The skills do not
  echo decision-log entries to stdout at step boundaries. Rationale:
  most decisions are routine (which subtask is next; which gate just
  passed) and echoing them clutters the user's view. The skills DO
  surface non-routine decisions (Design Amendment trigger, `/bugfix`
  invocation, scope drift) to the user via the existing per-step
  surface — those are already user-visible. If observation post-merge
  shows drift goes unnoticed, the echo can be added in a follow-up.
- **Whether `last_passed_gate` is one field or one per gate** —
  *Single field.* Resolved per the spec's default and *Approach →
  Schema extension*. Each new pass overwrites the previous. Rationale:
  re-entry needs "what was last green here", not a historical
  timeline. Per-gate history is recoverable from `git log` via `cargo`
  invocations in commit messages.
- **Anti-patterns content for the new `.claude/skills/task/reference.md`** —
  *Resolved by this design.* The extracted content is reference
  material that already exists inside `/task` SKILL.md (validation
  procedures, Design Amendment recipe, FORBIDDEN list, gate
  checklist). No new anti-patterns content is authored in this PR —
  that's an explicit *Deferred* item in the spec ("Long-form
  anti-patterns catalogue for compaction-related failure modes is a
  follow-up").
- **Section heading style for the `/bugfix` trace-file extensions** —
  *Resolved by this design.* See *Approach → `/bugfix` trace-file
  extension*. Header `**Field:**` lines for the four short fields
  (including `entry_args`); `## Decisions log` h2 section for the
  appendable list. Matches the existing trace template shape.
- **`/context-reset` heading style for the two triggers** — *Resolved
  by this design.* Singular h2 `## Compaction recovery (re-entry)`
  (the cross-link anchor used by every other skill's callout) alongside
  the existing `## Handoff protocol` h2. Both preludes share the
  bottom "Handoff protocol" + "Rules" sections. Single SKILL.md, not
  front-matter dispatch. The existing SKILL.md is short (1.8 KB) —
  splitting into two files is premature.

No questions remain unresolved at the design phase.

### Round-2 recommendations (for design-review verification)

Round 2 resolved 4 blockers and 1 major. No GO-with-notes items are
emitted this round — every blocker was amended in-place. The following
recommendations carry forward as **explicit verification targets** for
the design-review:

1. **Variant-A callout body** does NOT contain the phrase "Step 1 of
   this skill" or "restart from Step 1" (Blocker 1).
2. **Variant-A callout body** routes through "this skill's active-state
   probe" — the preamble — explicitly, NOT through a numbered Step
   (Blocker 2).
3. **Variant-B callout body for `/bugfix`** carries the resume rule
   "if `Confirmed by user: ✅ YES` … resume from the step recorded in
   `current_step`" — re-tracing is forbidden on a confirmed trace
   (Blocker 3). Subtask 5 wires this into Step 1 itself, not just the
   callout.
4. **Variant-C callout body for `/context-reset`** carries NO "Step 1"
   reference (Blocker 4). The cross-link anchor `## Compaction recovery
   (re-entry)` is singular and verified by subtask 13's audit.
5. **`/task` SKILL.md `⚡ First` preamble** documents the
   lost-arguments re-entry case; `⚡ Second` and `⚡ Third` preambles
   each carry a guard sentence requiring non-empty `$ARGUMENTS` to
   fire (Major 5). The `entry_args` schema field is recorded at Step 8
   progress-file creation per subtask 1 + 8.

The 13-subtask decomposition, bundling justification, schema extension
shape, `/bugfix` trace shape, and sequencing rationale all carry
forward from Round 1 unchanged — Round-1 review explicitly confirmed
those.
