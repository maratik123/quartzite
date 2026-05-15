# Update self-improve.md Step 6 contract — route clean-context eval to parent thread

**Source:** issue #366
**Date:** 2026-05-16
**Tracked in:** #366

## Scope

1. Update `.claude/agents/self-improve.md` Step 6 ("Eval (REQUIRED after Step 5)") to remove the directive "Run the scenario via an `Agent` subagent in a clean context", which is structurally unfulfillable: the `self-improve` agent class's runtime tool list does not contain the `Agent` primitive (the `Task*` family is queue management for in-flight subagents, not subagent spawning, and `AskUserQuestion` is also absent).
2. Replace the unfulfillable directive with the codified **pause-and-surface protocol**:
   - The subagent introspects its tool list and confirms `Agent` is absent.
   - The subagent halts Step 6 execution and surfaces a structured handoff message to the parent thread (which does have `Agent`).
   - The parent thread dispatches the clean-context eval reproducers and reports PASS/FAIL back into the conversation.
3. Embed a numbered reproducer-prompt template inside the new Step 6 body that the subagent emits verbatim for the parent to paste. The template carries: (a) the scenario to reproduce, (b) the expected fixed-output shape, (c) PASS/FAIL criteria, and (d) the count of reproducer scenarios (one numbered block per scenario the subagent identified during Step 5).
4. Cite the two authoritative learning-log entries inline in the new Step 6 prose:
   - 2026-05-15 *"`self-improve` silently degraded `/improve` Step 6"* (merged in PR #362 Commit C) — pause-and-surface rule.
   - 2026-05-15 *P5 finding* (merged in PR #364) — confirmed harness-level absence of the `Agent` primitive in the `self-improve` runtime tool set.
5. Propagation Rule check on `.claude/agents/learnings-escalation-audit.md` (sister file in the Corrections-Log sync group per AGENTS.md `## Propagation Rule`):
   - Confirmed: `learnings-escalation-audit.md` has no Step 6 equivalent, no `Agent` dispatch, no eval phase. The agent is a passive auditor (`/ai-audit` Phase 1).
   - **No propagation edit needed.** Document the asymmetry as a one-line note in `self-improve.md`'s new Step 6 body (or as a `<!-- propagation note -->` HTML comment) so future readers do not assume the contract needs mirroring.

## Out of scope

- Changing the substantive `/improve` workflow itself. Steps 1–5 (read `learnings.md` → find patterns → propose diffs → escalate to hooks at ≥3 occurrences → apply) remain unchanged.
- Changing what Step 6 *means* (it still requires a clean-context eval reproducer run with a PASS/FAIL report). Only the **execution-routing** changes: subagent → parent thread, rather than subagent → spawned `Agent`.
- Adding the `Agent` primitive to `self-improve`'s runtime tool exposure. That is a harness-level concern outside Claude Code project conventions.
- Hook-level enforcement of "always pause-and-surface". The pre-condition for a hook is ≥3 occurrences despite an existing rule; current state is two recorded occurrences (PR #362 Commit C, PR #363's working pattern) — threshold not met.
- Updating any other agent or skill file. Verified that no other Corrections-Log sync-group sibling has a Step 6 equivalent.
- Modifying the `description:` frontmatter field of `self-improve.md` unless the body change forces it for accuracy. Design may include a description tweak if directly implied by the new Step 6 wording.

## Deferred

- (none) | — | —

## Key decisions

| Question | Decision |
|---|---|
| Should the new Step 6 keep the same name ("Eval (REQUIRED after Step 5)")? | Yes — the eval requirement is unchanged; only the dispatch routing changes. |
| Should the pause-and-surface protocol be the agent's default for all primitives it lacks, or only `Agent`? | General — the 2026-05-15 Commit-C entry codifies the rule generically ("a workflow step names a specific execution primitive ... the agent MUST pause-and-surface BEFORE substituting"). The new Step 6 names `Agent` explicitly as the case-in-point and the absence the subagent will observe in practice, while keeping the rule generic enough to cover future primitive gaps. |
| Where does the reproducer-prompt template live — inline in Step 6, or as a separate appendix in the agent file? | Inline in Step 6 — single point of consumption, no risk of doc drift between the protocol and its template. |
| Should the template be a literal fenced markdown block the subagent copies verbatim, or a placeholder skeleton the subagent fills? | Skeleton with named placeholders (e.g., `<scenario>`, `<expected_output>`, `<PASS_criterion>`, `<FAIL_criterion>`) plus one worked example. The subagent fills the placeholders per scenario before emitting. |
| Does the contract need a "max reproducer count" cap? | No — the count equals the number of patterns the subagent applied in Step 5. Bounded organically by Step 1 pattern-finding output; no separate cap warranted. |
| How does the parent thread know it has been handed off to? | The surfaced message uses an unambiguous header (e.g., `## Step 6 handoff — clean-context eval reproducers`) plus an explicit one-line instruction directed at the parent thread. Design picks the exact wording. |
| Asymmetry note placement in `self-improve.md` | One-line inline note (or HTML comment) inside the new Step 6 body explaining that the Corrections-Log sync-group sister `learnings-escalation-audit.md` has no equivalent Step, so the propagation row does not require a mirrored edit. |

## Technical constraints

- Edit scope: a single file (`.claude/agents/self-improve.md`). No code changes, no test changes, no `Cargo.toml` touch.
- The `cargo build` / `cargo clippy` / `cargo fmt` / `actionlint` gates do not apply to this file. Pre-commit hook still runs and must pass.
- AGENTS.md file-size axiom: `.claude/agents/self-improve.md` is currently well below the 35 000-char early-warning threshold (≈4 000 chars / 114 lines). The Step 6 rewrite is replacing roughly the same body it removes plus the inline template + citations + asymmetry note; total expected delta is small. No extraction pass required, but design should confirm with `wc -c` after drafting.
- Corrections-Log sync group rule (AGENTS.md `## Propagation Rule` row at line 205): the row fires when `AGENTS.md § Corrections Log` is edited. This task does **not** edit `AGENTS.md § Corrections Log`. The reverse-direction check on the sister file `learnings-escalation-audit.md` has been performed during spec-writing (it has no Step 6 equivalent; no mirroring needed). The asymmetry is documented inline in the new Step 6.
- The rewritten Step 6 must preserve the existing `PASS criterion:` / `FAIL criterion:` semantics and the `Eval: PASS ✅` / `Eval: FAIL ❌` report-line format, since downstream consumers of `/improve` output already key on those strings.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/agents/self-improve.md` Step 6 body no longer contains the directive "Run the scenario via an `Agent` subagent in a clean context" (or any equivalent phrasing instructing the subagent to dispatch an `Agent` itself). |
| AC2 | The new Step 6 explicitly names `Agent` as a primitive the subagent's runtime tool list does NOT contain, and instructs the subagent to **pause-and-surface** to the parent thread rather than substitute a degraded primitive. |
| AC3 | The new Step 6 includes a numbered reproducer-prompt template (skeleton with named placeholders + one worked example) that the subagent fills per scenario before emitting verbatim to the parent thread. |
| AC4 | The new Step 6 cites both authoritative entries: 2026-05-15 *"`self-improve` silently degraded `/improve` Step 6"* (PR #362 Commit C) AND 2026-05-15 *P5 finding* (PR #364). |
| AC5 | The new Step 6 (or an adjacent note inside the file) documents the propagation-rule check on `.claude/agents/learnings-escalation-audit.md`, recording the asymmetry: the sister file has no Step 6 equivalent and therefore needs no mirrored edit. |
| AC6 | The rewritten Step 6 preserves the existing `Eval: PASS ✅` / `Eval: FAIL ❌` report-line format so downstream `/improve` consumers continue to parse the same strings. |
| AC7 | The next `/improve` invocation (post-merge) reaches Step 6, the subagent emits the templated handoff without silent degradation, and the parent thread successfully dispatches the reproducer scenarios — at least one clean-context eval returns PASS, reported back into the conversation. |

## Open questions

- (none — all design-affecting ambiguities resolved by the Key decisions table; remaining choices are presentation-level and within the design agent's purview)
