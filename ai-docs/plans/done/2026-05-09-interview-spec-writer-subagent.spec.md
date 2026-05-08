# Extract `/interview` spec-drafting into `spec-writer` opus subagent

**Source:** issue #188
**Date:** 2026-05-09
**Tracked in:** #188

## Scope

1. **New agent file** — `.claude/agents/spec-writer.md`:
   - Frontmatter: `model: opus`; tools `Read, Write, Edit, Bash`.
   - System prompt embeds: input contract, YAML output schema, AGENTS.md
     preflight rule, the full Rule-5 substring blacklist (mirrored from
     AGENTS.md and labelled as such), the spec-format template, round-cap
     and per-round-questions-cap awareness rules.

2. **Subagent input contract** (passed in every invocation prompt):
   - Issue ref + full body verbatim from `gh issue view <N>`
   - `round`: integer in `1..=round_cap`
   - `round_cap`: integer (default 4)
   - `questions_per_round_cap`: integer (default 3)
   - Prior Q&A history (canonical list — passed every call, not relied on
     via agent memory)
   - Current spec draft path (may not yet exist on round 1)

3. **Subagent output**:
   - **Side effect:** writes spec to disk at the spec path; spec follows the
     existing format (`# Task name`, `**Source:**`, `**Date:**`,
     `**Tracked in:**`, `## Scope`, `## Out of scope`, `## Deferred`,
     `## Key decisions`, `## Technical constraints`,
     `## Acceptance Criteria`, `## Open questions`).
   - **Mandatory final response block** — YAML, must come last in the
     agent's response:

     ```yaml
     ---
     status: ready | ask | unresolvable
     round: <N>
     questions:                  # required iff status == ask, length ≤ questions_per_round_cap
       - question: "..."
         header: "..."           # ≤ 12 chars (AskUserQuestion-shaped)
         options:
           - { label: "...", description: "..." }
     reason:                     # required iff status == unresolvable
       category: cap_reached | logically_unresolvable | external_dependency | empty_scope | user_loop
       detail: "..."
       suggested_action: defer_to_deferred | abort | extend_cap | request_external_info
     ---
     ```

4. **Hard rules embedded in agent definition**:
   - AGENTS.md preflight: read AGENTS.md before drafting questions; apply
     pre-resolved rules silently (do not ask).
   - Rule-5 substring blacklist: mirrored from AGENTS.md `/interview` skill
     Rule 5; the agent must not emit a question containing any blacklisted
     substring.
   - `questions` list length is a hard upper bound at
     `questions_per_round_cap`.
   - When `round == round_cap`, status MUST be `ready` or `unresolvable`,
     never `ask`.
   - Optimization target (verbatim in the system prompt):
     > Produce the smallest spec sufficient for the design agent to return a
     > `GO` verdict on the first design-review pass. Ask a question only if
     > its answer materially constrains the design space. Apply AGENTS.md
     > defaults silently. Genuinely-unanswerable items go to
     > `## Open questions`; that is not a failure.

5. **Orchestrator rewrite** — `.claude/skills/interview/SKILL.md` becomes a
   thin orchestrator. Existing scope-extraction / question-content content
   moves into the agent definition. Orchestrator responsibilities:
   - Detect entry mode (issue ref / free-text / empty) and load the issue
     body via `gh issue view`.
   - Compute spec path and state-file path
     (`<spec_path>.state.md`).
   - Round 1 — invoke `Agent(subagent_type="general-purpose", model="opus", ...)`
     with the agent's instruction file referenced (`.claude/agents/spec-writer.md`)
     and the input-contract fields in the prompt. Capture the returned
     `agentId`.
   - Rounds 2..cap — invoke `SendMessage(to=agentId, ...)` with the
     updated input-contract fields (round, full prior_qa, current spec
     path). The agent re-derives context from the prompt every call (per
     hard rule).
   - Parse the YAML status block from the agent's response.
   - **On `status: ask`** — surface all questions in a single
     `AskUserQuestion` tool call (tool supports up to 4 questions per call,
     so 1–3 fit cleanly). Append (Q, user_answer) pairs to state file;
     advance to round N+1.
   - **On `status: ready`** — confirm the spec with the user; on approval
     post the cross-link comment on the tracking issue (Step 7 of the
     existing skill); delete the state file; exit.
   - **On `status: unresolvable`** — surface the reason via
     `AskUserQuestion` with the agent's `suggested_action` first, plus the
     other applicable actions:
     - `defer_to_deferred` → move spec draft (if any) to
       `ai-docs/plans/deferred/`, update `INDEX.md` (status `🟡 spec-only`),
       delete the state file, exit.
     - `abort` → delete partial spec draft, delete the state file, exit.
     - `extend_cap` → bump `round_cap += 1`; SendMessage round=N+1,
       cap=cap+1; resume the loop.
     - `request_external_info` → prompt the user for additional context
       paste; SendMessage round=N+1 with the appended context; resume the
       loop.
   - YAML parse failure recovery: on malformed YAML, SendMessage to the
     agent: "Re-emit only the YAML status block, exact schema." Cap retries
     at 1; on second failure, surface to the user as an
     `unresolvable: logically_unresolvable` (orchestrator-injected).

6. **State file** — `<spec_path>.state.md`:
   - Created on round 1; deleted on `ready` / `abort` / `defer_to_deferred`.
   - Stores: round counter, `round_cap`, `questions_per_round_cap`,
     `agentId`, the canonical Q&A history list.
   - Format: markdown header + a YAML fenced block. (Specific layout is a
     design-phase decision; the spec only requires that it be a single
     dedicated file and not embedded in the spec itself.)

7. **AGENTS.md updates**:
   - Add `spec-writer.md` to the **Agent Docs** table with one-line purpose.
   - Add `interview ↔ spec-writer ↔ AGENTS.md` to the **Propagation Rule**
     sync-groups list, with explicit annotation: "Rule-5 substring-blacklist
     edits MUST propagate to `spec-writer.md`'s embedded copy."

## Out of scope

- Changing `/task` skill orchestration. Steps 6 onwards (design, design
  review, implementation, verify, self-review, finalise) stay as-is. Only
  Steps 1–5 internals change.
- Changing the spec format template. The agent emits the same shape
  (`# Task name`, sections, AC table); only the author changes.
- Adding `/interview` CLI overrides for `round_cap` or
  `questions_per_round_cap`. Defaults stay as constants in the agent
  definition for this iteration; configurability deferred to a future
  iteration if needed.
- Migrating any other skill (`/code-review`, `/improve`, etc.) to the
  subagent pattern. This is the pilot.
- Automated unit / integration tests for the orchestrator or agent. No Rust
  test infrastructure exists for skills/agents in this repo; verification
  is structural + live-test only.

## Deferred

- Configurability of `round_cap` / `questions_per_round_cap` via
  `/interview` arguments — separate issue if/when there's evidence the
  defaults are wrong.
- Migration of other skills to the subagent pattern — separate
  per-skill issue once this pilot validates the approach.
- Cross-session resumability of an interrupted interview (e.g., agent
  process dies mid-interview). The spec covers normal-flow exit; recovery
  from agent-side failures is out of scope here.

## Key decisions

| Question | Decision |
|---|---|
| Spec-drafting model | `model: opus` regardless of parent session's model — pinned in agent frontmatter. Rationale: spec quality is load-bearing; opus is the most capable model available. |
| State persistence shape | Separate file `<spec_path>.state.md`, transient (deleted on `ready` / `abort` / `defer_to_deferred`). Rationale: atomic, clean separation from the spec, mirrors existing `<spec>.progress.md` pattern. |
| Agent lifecycle across rounds | Warm: `Agent` for round 1, `SendMessage` for rounds 2..cap, capturing `agentId` in state. Rationale: cheaper (preflight reading is cached in agent's context); agent definition still requires re-derivation from prompt for safety. |
| Rule-5 substring blacklist source-of-truth | Duplicated into `spec-writer.md` with annotation `<!-- mirrored from AGENTS.md /interview Rule 5 — propagation-required -->`. Rationale: subagent runs in isolation; can't reliably do dynamic-file-read inclusion of rules; sync-drift mitigated by Propagation Rule sync-group entry. |
| Optimization target | "Smallest spec sufficient for first-pass design GO." Embedded verbatim in agent system prompt. |
| Round / questions caps | 4 / 3 (matching current `/interview`). Hard-coded in agent system prompt and orchestrator constants for this iteration. |
| Multi-question rounds | Subagent emits 1..=`questions_per_round_cap` questions per `ask`; orchestrator forwards entire list to a single `AskUserQuestion` call. Rationale: matches current UX (single round of 1–3 questions answered together); avoids sequential per-question round trips. |
| Unresolvable categories | Five: `cap_reached`, `logically_unresolvable`, `external_dependency`, `empty_scope`, `user_loop`. Each maps to a default `suggested_action` the orchestrator presents first. |
| YAML parse-failure recovery | One-shot SendMessage retry asking the agent to re-emit only the status block. On second failure, orchestrator injects `unresolvable: logically_unresolvable` and surfaces to the user. |
| Verification approach | Structural (file existence + shape) + live tests on synthetic issues for each unresolvable category + one full `/task` end-to-end run. No automated test suite. |

## Technical constraints

- **No Rust source touched.** `cargo build` / `cargo test` / `cargo fmt` /
  `cargo clippy` / `cargo doc` will all be no-op sanity checks; they pass
  trivially. No `actionlint` gate (no workflow files). No `panic-index.md`
  update (no production panic sites added).
- **Propagation Rule fires.** Three files form the new sync group:
  `AGENTS.md` (Rule-5 source), `.claude/skills/interview/SKILL.md`
  (orchestrator), `.claude/agents/spec-writer.md` (subagent definition with
  the mirrored Rule-5 blacklist).
- **Self-referential touch.** This task modifies the very `/interview`
  skill that `/task` Steps 1–5 use. Sequence is fine: the **current**
  in-context `/interview` is drafting this spec right now; the new
  subagent-based `/interview` activates only after merge. No chicken-and-egg.
- **`AskUserQuestion` constraints.** The tool supports up to 4 questions
  per call and up to 4 options per question. The agent must emit options
  conforming to those caps (the agent definition will state this).
- **AGENTS.md axioms** (already in effect, restated for completeness):
  - Boundary rule 1: `ai-docs/learnings.md` is APPEND-ONLY (no edits to past
    entries during this task).
  - Boundary rule 2: writing to `learnings.md` does NOT trigger
    instruction-file edits in the same turn (this task DOES edit
    instruction files for an independent reason — Propagation Rule fires
    properly).
  - Workflow Axiom 1: never edit on local `master` for PR-targeted work;
    feature branch first.
- **File-size targets** (AGENTS.md *Code Style* — Source files; applies to
  all repo files including markdown). Orchestrator rewrite targets the
  current `SKILL.md`'s soft 500/800 budget. Agent definition file is new;
  same target applies.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/agents/spec-writer.md` exists. Frontmatter has `model: opus` and `tools: Read, Write, Edit, Bash`. System prompt embeds: input contract (issue body, `round`, `round_cap`, `questions_per_round_cap`, prior Q&A, spec path), YAML output schema (with all five `unresolvable` categories), AGENTS.md preflight rule, the Rule-5 substring blacklist (annotated as mirrored from AGENTS.md), the spec-format template reference, the round-cap-awareness rule, and the optimization-target verbatim quote. |
| AC2 | `.claude/skills/interview/SKILL.md` is rewritten as a thin orchestrator. It contains: entry-mode detection, spec-path / state-file-path computation, round-1 `Agent` invocation referencing `.claude/agents/spec-writer.md` with `model="opus"`, rounds-2+ `SendMessage` invocation, YAML-status parsing, branching on `ready` / `ask` / `unresolvable` with the four documented action handlers, state-file lifecycle (create round 1 / delete on terminal exit), and YAML-parse-failure recovery (one-shot SendMessage retry). It no longer contains the scope-extraction / question-drafting / Rule-5 blacklist content (those moved to the agent). |
| AC3 | `AGENTS.md` Agent Docs table contains a row for `.claude/agents/spec-writer.md` with a one-line purpose. The Propagation Rule "sync groups (canonical)" section contains a new sync group: `interview ↔ spec-writer ↔ AGENTS.md`, with an annotation that Rule-5 substring-blacklist edits must propagate to `spec-writer.md`. |
| AC4 | Live test — running `/interview` against a synthetic clear-scope issue (one-line, unambiguous) exits with `status: ready` in round 1. The orchestrator does not call `AskUserQuestion`; the spec is written; the cross-link comment is posted on the tracking issue. |
| AC5 | Live test — running `/interview` against a synthetic moderately-ambiguous issue triggers `status: ask` for at least round 1; the orchestrator surfaces 1–3 questions in a single `AskUserQuestion` call; user answers feed back into round 2 via `SendMessage`; the loop converges to `status: ready` within `round_cap` (4) rounds; spec on disk + cross-link posted. |
| AC6 | Live test — each of the five `unresolvable` categories (`cap_reached`, `logically_unresolvable`, `external_dependency`, `empty_scope`, `user_loop`) triggers correctly via a synthetic issue exercising it. For each, the orchestrator surfaces the reason via `AskUserQuestion` with the agent's `suggested_action` listed first. The chosen action executes correctly: `defer_to_deferred` moves the spec to `ai-docs/plans/deferred/` and updates `INDEX.md`; `abort` deletes the partial spec; `extend_cap` resumes the loop with `round_cap += 1`; `request_external_info` resumes with appended context. |
| AC7 | End-to-end `/task <issue#>` run completes successfully against a real, small, low-risk backlog issue. The new `/interview` produces a spec the design agent (Step 6) can consume and the design-review agent (Step 7) returns a `GO` verdict on the first pass (i.e., no spec-gap-driven `ITERATE` / `STOP`). |

## Open questions

- None. All design-affecting decisions are pinned in the issue body and the
  Key Decisions table above. Concrete YAML field shapes, state-file content
  layout, agent system-prompt wording, and recovery-from-agent-crash
  behaviour are design-detail; the design agent will resolve them without
  further user input.
