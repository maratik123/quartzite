# Sonnet skill re-entry protocol after auto-compaction

**Source:** issue #348
**Date:** 2026-05-14
**Tracked in:** #348

## Problem (one-paragraph recap)

Code-side orchestrator skills (`/task`, `/code-review`, `/pr-commented`, `/bugfix`,
`/interview`, `/context-reset`) lose workflow context after auto-compaction on
Sonnet. CLAUDE.md / AGENTS.md re-inject from disk automatically, but `SKILL.md`
bodies are re-injected with a 5,000-token-per-skill cap and 25,000-token total
budget, truncating from the bottom and dropping oldest skills first. Concretely:
after a `/task` session detoured into `/bugfix` → `/verify` → `/context-reset`,
the agent on resume forgets which step it was on, what was decided earlier, and
what gate it just passed. Verified against Claude Code docs on 2026-05-14
(`/ai-audit` session).

## Scope

1. **Extend the canonical progress-file schema** at
   `ai-docs/templates/progress-format.md` to include four new fields:
   `current_step` (required), `decisions_log` (required, append-only one-liner
   per non-trivial decision), `last_passed_gate` (required: command +
   timestamp + commit SHA), `parent_skill` (optional, names the outer skill
   when a nested skill — e.g. `/bugfix` inside `/task` Step 8 — owns the
   current progress write).
2. **Add a top-of-file "Compaction recovery check" callout** to every
   code-side orchestrator SKILL.md, immediately after the front matter / lead
   paragraph and **before** any step instructions. The callout is the
   heuristic self-detection re-entry surface (no deterministic guard, no
   `SessionStart|compact` hook). Wording shared (DRY) across skills; each
   skill names its own progress-file path. The callout instructs the
   re-entering agent to **read the entire progress file top-to-bottom before
   any tool calls, then resume from Step 1** — never jump to the recorded
   `current_step` directly. (See *Key decisions* row "Re-invocation behavior
   on re-entry" for the rationale.)
3. **Reorder code-side SKILL.md bodies so workflow steps land at the top**
   (truncation keeps the start of the file). Reference material
   (anti-patterns, examples, troubleshooting, gate checklists) moves to the
   bottom of the same file. No content removal in this pass.
4. **Extract `/task` SKILL.md anti-patterns / troubleshooting / examples to
   `.claude/skills/task/reference.md`** (sibling, single-consumer reference
   file per the Claude Code supporting-files pattern). `/task` SKILL.md keeps
   a footer link. Target: `/task` SKILL.md ≤ 5,000 tokens (~20,000 chars) so
   it no longer faces truncation risk after compaction.
5. **Wire every code-side orchestrator to write progress at every step
   boundary** before further tool calls — `/task` (Steps 8–12), `/code-review`
   (per-phase), `/pr-commented` (per Step 1–7 within each round), `/bugfix`
   (per Step 1–7 in the trace file — see *Key decisions* row "`/bugfix` trace
   file vs parallel `.progress.md`"). `/interview` exempt (the in-flight spec
   at `<spec_path>` IS its durable state).
6. **Strengthen `/context-reset` to cover both triggers.** The SKILL.md body
   grows a second section so it explicitly covers: (a) the existing
   N=3-of-M≥5-subtasks auto-trigger, AND (b) compaction-recovery re-entry.
   Both sections share the same final action (write a handoff state and
   re-prime), with distinct preludes describing the trigger condition. The
   skill remains the single canonical handoff protocol.
7. **Apply the Propagation Rule** for every sync-group sibling touched by
   the SKILL.md edits. Specifically:
   - Review group: `code-review/SKILL.md` ↔ `review-findings.md` ↔
     `self-review.md` — both agents read the progress file and must learn
     the new schema fields.
   - Task/Design group: `task/SKILL.md` Steps 6–8 ↔ `design.md` ↔
     `design-review.md` — only if Steps 6–8 reorder changes their contract;
     otherwise these stay untouched (reordering is structural, not contractual).
   - Interview group: `interview/SKILL.md` ↔ `spec-writer.md` — `/interview`
     gets the compaction-recovery callout but is exempt from the
     `.progress.md` schema (spec serves that role); `spec-writer.md` is
     unaffected unless the orchestrator contract shifts.
   - `AGENTS.md` "Agent Docs" table row for `ai-docs/plans/*.progress.md`
     updated to reflect the extended schema's writer set if needed; the
     `ai-docs/bugfix/trace-*.md` row likewise updated to note the trace file
     now also carries `current_step` / `decisions_log` / `last_passed_gate`.

## Out of scope

- Model pinning on code-side orchestrators (must work on Sonnet; can also
  run on Opus).
- AGENTS.md slim-down for per-turn token savings (re-injected per session,
  not per turn; current 33,214 chars is under the 35k early-warning).
- Axiom-summary hook for CLAUDE.md (auto-re-injected per docs).
- `SessionStart|compact` re-entry hook (user explicitly chose heuristic
  self-detection over a hook).
- Text-side skills (`/triage`, `/ai-audit`, `/improve`, `/next`) and
  text-side subagent changes (user manually switches to Opus before
  invoking these).
- Changing `model: inherit` on `review-findings` / `self-review` — code-side
  subagents stay `inherit`.
- Text-heavy subagents (`design`, `design-review`, `spec-writer`,
  `self-improve`, `learnings-escalation-audit`, `triage-runner`) stay
  pinned `model: opus`.
- New SKILL.md content beyond the callout + reorder + (for `/task`)
  extraction + (for `/context-reset`) the compaction-recovery section.
  Authoring fresh workflow guidance beyond these is a follow-up.
- A parallel `.progress.md` for `/bugfix`. The existing trace file at
  `ai-docs/bugfix/trace-YYYY-MM-DD-<name>.md` absorbs the new schema
  fields; no second artefact per bug.

## Deferred

- Long-form anti-patterns catalogue for compaction-related failure modes
  (what NOT to do after re-entry) | depends on observation post-merge | likely
  separate `/improve` cycle; recorded in `_inbox.md` at Step 12.
- Generalising the "compaction recovery" callout into a reusable include /
  template once a second project adopts the pattern | premature now |
  separate issue if a second project ever shares this repo's skill layout.
- Token-budget telemetry / measurement scripts to confirm post-merge that
  `/task` SKILL.md actually stays under the 5,000-token target as it
  evolves | useful follow-up; not blocking | separate `tooling/` issue.

## Key decisions

| Question | Decision |
|---|---|
| Progress-file schema format — markdown sections vs YAML frontmatter vs table | **Extend the existing markdown-section format** at `ai-docs/templates/progress-format.md`. The four new fields (`current_step`, `decisions_log`, `last_passed_gate`, `parent_skill`) become named sections / header lines consistent with the existing template (`**Branch:**`, `**base_commit:**`, `**Last build:**`). No format pivot — every current consumer keeps working. |
| `decisions_log` write semantics | **Append-only**, one line per non-trivial decision, prefixed with the step or phase that made it (`Step 6: chose approach B because X`). Existing `## Key discoveries` section retained for read-time hints; `decisions_log` is the per-step audit trail. |
| `last_passed_gate` content | **Command + ISO-8601 timestamp + commit SHA** — `cargo clippy --workspace -- -D warnings \| 2026-05-14T18:42Z \| 549282b`. Lets the re-entrant agent see what was last green without re-running. |
| Re-entry wording — shared vs tailored | **Shared callout body**, each skill names its own progress-file path. DRY beats per-skill verbosity; the callout is mechanical and identical in intent across skills. The skill's existing top-paragraph context still tells the reader which skill they're in. |
| `/interview` exemption | **Keep.** The in-flight spec at `<spec_path>` IS the durable re-entry surface. Re-entry reads the spec and resumes from the next unanswered round via the existing `.state.md` (which is already round-by-round). No separate `.progress.md`. `/interview` gets the compaction-recovery callout pointing at `<spec_path>` (and the `.state.md` sibling) instead of `.progress.md`. |
| `/code-review` / `/pr-commented` round-by-round state | **Add `current_step` + `decisions_log` per round-section** (`## Self-Review (Round N)` / `## Comment cycle round M`). The existing round-section pattern already partitions state by round; the new fields nest inside the latest round's section. No top-level round counter duplication. |
| Nested-skill re-entry contract (`/task` → `/interview` / `/bugfix`) | **Parent owns its own progress file; nested skill owns its own durable artefact.** When `/task` invokes `/interview`, `/task`'s progress file records "Step <N> → /interview invoked at <time>" in `decisions_log`; `/interview` uses its existing spec + `.state.md`. When `/task` Step 8 invokes `/bugfix`, the `/bugfix` trace file records `parent_skill: /task` so re-entry into the nested skill knows the outer flow; `/task`'s own progress file records "Step 8 → /bugfix invoked at <time>" in `decisions_log`. On nested-skill completion, control returns to `/task`, which reads its own progress file and continues. |
| `/verify` and `/pr-merged` exemption | **1-line waiver in their SKILL.md** ("Near-stateless: no `.progress.md` discipline applies; re-entry consists of re-invoking the skill"). Costs ~50 bytes per file, gives visual consistency, prevents future "why doesn't `/verify` have a progress file?" confusion. |
| `/task` SKILL.md extraction target file | **`.claude/skills/task/reference.md`** — sibling file inside the skill's own directory, per the Claude Code [supporting-files pattern](https://code.claude.com/docs/en/skills#add-supporting-files). Not `ai-docs/templates/` (single consumer, not multi-consumer reference material). |
| 40k-char cap interaction | The reorder + extraction pass must NOT push any sibling SKILL.md over 35,000 chars (AGENTS.md early-warning). Quick `wc -c` scan after edits. |
| **Re-invocation behavior on re-entry** (Round 1, answer) | **Full re-read on re-entry** (interview Round-1 option label: "Restart Step 1" — the *option label* survives as audit trail; the *invariant name* is **Full-read-on-re-entry invariant** per *Technical constraints*; the routing per skill is via Variants A/B/C in the design's *Per-skill mapping*). On re-entry after compaction, the re-invoked skill always re-reads the progress file top-to-bottom before any tool calls and only then re-enters the skill from the **top of its body** (preambles + Step 1); never jump to the recorded `current_step` directly. Slower by one read, immune to stale `current_step` (a half-written boundary record cannot mislead the resume). The callout wording reflects this explicitly. |
| **`/bugfix` trace file vs parallel `.progress.md`** (Round 1, answer) | **Extend the existing trace file.** `ai-docs/bugfix/trace-YYYY-MM-DD-<name>.md` grows three new sections — `## current_step`, `## decisions_log`, `## last_passed_gate` — alongside the existing trace content. No parallel `.progress.md`. One artefact per bug; existing lifecycle preserved (created at Step 1, deleted on Step 7 after the fix lands). `parent_skill: /task` recorded inline when `/bugfix` is invoked from inside `/task` Step 8. |
| **`/context-reset` strengthening** (Round 1, answer) | **Add compaction-recovery section.** `/context-reset` SKILL.md grows a second section so the body explicitly covers both triggers: (a) the existing N=3-of-M≥5-subtasks auto-trigger, AND (b) compaction-recovery re-entry. The two sections share the same handoff + re-prime action; preludes differ to describe the trigger condition. `/context-reset` remains the single canonical handoff protocol — code-side skills' compaction-recovery callouts cross-link to it. |

## Technical constraints

- **Char-cap discipline** — every edited instruction file must stay
  under 35,000 chars (AGENTS.md early-warning) and well under 40,000 (harness
  cap). `/task` SKILL.md target ≤ 20,000 chars (~5,000 tokens) after extraction.
- **No machine-enforced re-entry guard.** Per user direction: heuristic
  self-detection only, no `SessionStart|compact` hook, no per-step Skill-tool
  re-invocation. The callout must be unambiguous enough that an agent
  emerging from compaction acts on it without a deterministic check.
- **Sonnet correctness.** Skills must work on Sonnet (can also run on Opus).
  Subagents pinned to Opus where they already are (`spec-writer`, `design`,
  `design-review`, etc.); inherited subagents (`review-findings`,
  `self-review`) stay `inherit`.
- **Bundled single PR** for all changes (user-directed in the issue).
- **Truncation order awareness.** Claude Code's per-skill truncation keeps the
  **start** of `SKILL.md`. Workflow steps must appear above reference /
  troubleshooting / examples in every code-side SKILL.md.
- **Propagation Rule** discipline (AGENTS.md) — every sync-group sibling
  touched in the same PR.
- **Pre-publish project.** Free to break instruction-file structure without
  compat shims. No "old layout / new layout" coexistence.
- **`/interview` exception** — its durable state is the in-flight spec at
  `<spec_path>` plus the `.state.md` sibling; no parallel `.progress.md`.
- **Full-read-on-re-entry invariant** — every re-entry path (compaction
  recovery or otherwise) MUST re-read the durable-state file end-to-end
  before any tool calls, then re-enter the skill from the top of its
  body (preambles included). The recorded `current_step` is a hint for
  the human reader and a cross-check, never an instruction to skip the
  read or to jump straight to that step. Per-skill callout variants
  (A/B/C — see design doc *Per-skill mapping*) route this re-entry
  correctly for each skill's preamble / step-1 / no-numbered-step shape.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Every code-side orchestrator SKILL.md (`/task`, `/interview`, `/bugfix`, `/code-review`, `/pr-commented`, `/context-reset`) carries a top-of-file "Compaction recovery check" callout, placed before any step instructions. Callout body is identical **within each variant group** (Variant A: `/task` / `/code-review` / `/pr-commented`; Variant B: `/bugfix` / `/interview`; Variant C: `/context-reset`); each instance names its own probe / durable-state file per the design's *Per-skill mapping* table. The callout explicitly tells the re-entering agent to re-read the durable-state file top-to-bottom and re-enter the skill from the **top of its body** (preambles + Step 1) — never jump to the recorded `current_step`. |
| AC2 | `.claude/skills/task/SKILL.md` is ≤ 5,000 tokens (~20,000 chars; verified by `wc -c`). Anti-patterns / troubleshooting / examples extracted to `.claude/skills/task/reference.md`. SKILL.md retains a footer link to `reference.md`. |
| AC3 | `ai-docs/templates/progress-format.md` schema documents `current_step` (required), `decisions_log` (required, append-only, one-liner per decision with step prefix), `last_passed_gate` (required: command + ISO-8601 timestamp + commit SHA), `parent_skill` (optional, names outer skill when this progress file is owned by a nested skill). Existing required fields (`**Branch:**`, `**base_commit:**`, `**Last build:**`) preserved. |
| AC4 | `/task` SKILL.md, `/code-review` SKILL.md, `/pr-commented` SKILL.md, and `/bugfix` SKILL.md each instruct the agent to write the new fields (`current_step`, `decisions_log`, `last_passed_gate`) at every step boundary, before further tool calls. Wording is consistent across skills. For `/bugfix`, the write target is the existing trace file `ai-docs/bugfix/trace-YYYY-MM-DD-<name>.md`; no parallel `.progress.md`. |
| AC5 | `/interview` SKILL.md's compaction-recovery callout points to `<spec_path>` and the `.state.md` sibling instead of `.progress.md`. No `.progress.md` is created by `/interview`. |
| AC6 | `/verify` SKILL.md and `/pr-merged` SKILL.md each carry a one-line waiver explicitly exempting the skill from `.progress.md` discipline ("Near-stateless: no `.progress.md` discipline applies; re-entry consists of re-invoking the skill"). |
| AC7 | `/context-reset` SKILL.md body explicitly covers BOTH triggers: (a) the existing N=3-of-M≥5-subtasks auto-trigger, AND (b) compaction-recovery re-entry. The two triggers are clearly distinguished (separate sections / headings); the handoff + re-prime action is shared. Code-side skills' compaction-recovery callouts cross-link to `/context-reset`. |
| AC8 | All sync-group siblings (per AGENTS.md Propagation Rule) updated: Review group (`code-review/SKILL.md` ↔ `review-findings.md` ↔ `self-review.md`) reflects the extended progress-file schema; Interview group (`interview/SKILL.md` ↔ `spec-writer.md`) reflects the spec-as-durable-state model; Task/Design group consulted but only touched if Steps 6–8 reorder altered the contract. AGENTS.md "Agent Docs" table rows for `ai-docs/plans/*.progress.md` and `ai-docs/bugfix/trace-*.md` updated to reflect the extended schema. |
| AC9 | A manual Sonnet `/task` run that triggers compaction can re-enter via the progress file: the agent re-reads the progress file top-to-bottom, re-enters the skill from the top of its body (preambles + Step 1, routed by the active-task probe), observes `current_step` and `decisions_log`, and resumes without losing the active step or earlier decisions. Verified by post-merge spot-check (recorded in `learnings.md`, not gated in CI). |
| AC10 | No code-side SKILL.md crosses 35,000 chars after the changes (AGENTS.md early-warning). `wc -c` scan at PR-create time, listed in the PR body. |
| AC11 | `.gitignore` covers any new local-only progress artefacts (current rule `/ai-docs/plans/**/*.progress.md` + `/ai-docs/pr-comments/` continues to cover the existing surface; `/bugfix` trace files remain in `ai-docs/bugfix/` under their existing tracking convention — they are deleted on Step 7 rather than gitignored). No new artefact paths require gitignore additions in this PR. |

## Open questions

(Items the design agent can defend on its own; user may revisit.)

- **Exact wording of the "Compaction recovery check" callout.** Design agent
  drafts; if a user-visible phrase becomes load-bearing post-merge, revisit.
  The callout MUST encode the Full-read-on-re-entry invariant explicitly
  (re-read the durable-state file end-to-end, then re-enter from the top
  of the skill body; preambles + Step 1; never skip-to-`current_step`).
- **Whether `decisions_log` lines should also be echoed to the user at each
  step boundary** (so the human running the task can spot drift in real time)
  or stay write-only. Design agent picks the lighter touch first; iterate
  post-merge if confusion observed.
- **Whether `last_passed_gate` is one field or one per gate** (build / clippy /
  fmt / doc). Design agent decides; one-field-per-gate is more verbose but
  unambiguous on re-entry. Defaults to the single-field "most recent passed
  gate" formulation in this spec.
- **Anti-patterns content for the new `.claude/skills/task/reference.md`.**
  Design agent owns what to include (e.g., "don't recall from memory after
  compaction", "don't update progress file from inside a subagent", "don't
  trust `current_step` without re-reading the file"). Concrete list to be
  finalised in the design doc; not blocking spec.
- **Section heading style for the `/bugfix` trace-file extensions.** Whether
  `## current_step` / `## decisions_log` / `## last_passed_gate` are headings,
  or inline `**Field:**` lines like the existing trace fields. Design agent
  picks the shape most consistent with the current trace template.
- **`/context-reset` heading style** for the two triggers — single SKILL.md
  with two sub-headings, or front-matter dispatch. Design agent decides.

## Notes for the design agent

- Issue #348 listed 9 "Open decisions for the spec / design phase". All 9
  are now resolved in *Key decisions* (items 1, 2, 5, 6, 7, 9 pre-resolved
  by Round-1 defaults; items 3, 4, 8 resolved by Round-1 user answers).
- The reorder for non-`/task` SKILL.md files is structural and small; the
  `/task` extraction is the only sizable refactor. Sequence the design so
  the small reorders land before the large extraction (incremental risk).
- The compaction-recovery callout is the only piece a user-visible Sonnet
  failure mode hinges on. Treat its wording as load-bearing.
- The Full-read-on-re-entry invariant is a hard requirement, not a design
  choice: every callout and every re-entry-aware step instruction must
  encode it (full re-read of the durable-state file → re-enter the skill
  from the top of its body, preambles + Step 1; never skip to
  `current_step`).
