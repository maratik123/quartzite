# Design: Extract `/interview` spec-drafting into `spec-writer` opus subagent

**Issue:** #188
**Date:** 2026-05-09

## Approach

Split the existing `/interview` skill into two cooperating instruction files: a thin **orchestrator** (`.claude/skills/interview/SKILL.md`) that owns flow control, state, and user I/O; and a new **subagent** (`.claude/agents/spec-writer.md`) that owns scope extraction, question drafting, AGENTS.md preflight, the Rule-5 substring blacklist, and spec writing. The subagent is invoked by the orchestrator via the `Agent` tool with `subagent_type="general-purpose"` and `model="opus"`, and the prompt instructs the subagent to "Read `.claude/agents/spec-writer.md` and follow it" — mirroring how `/task` invokes `design`, `design-review`, `self-review`, etc.

**Why this split.** The current 149-line `SKILL.md` mixes three concerns: (a) flow plumbing — entry mode, tracking-issue resolution, save + cross-link; (b) authoring policy — Rule 5 blacklist, scope-extraction prompts, anti-patterns; (c) the spec-format template. The subagent pattern — already proven for design / design-review / self-review — keeps the orchestrator small (≤ 200 lines, one job) and lets the spec-writer system prompt be tuned independently for spec quality without touching flow control. The orchestrator stops being a thinking loop and becomes a state machine.

**Lifecycle.** Round 1 invokes the agent fresh via `Agent(subagent_type="general-purpose", model="opus", prompt=...)` — capturing the returned `agentId`. Rounds 2..cap reuse that warm agent via `SendMessage(to=agentId, ...)` with the full updated input contract (round, prior Q&A, current spec path, current `round_cap`). The agent re-derives context from the prompt every call (per AC1's hard rule); warm reuse is purely an efficiency choice (preflight reads of AGENTS.md cache in the agent's own context).

**State persistence.** A single dedicated file `<spec_path>.state.md` carries the round counter, caps, `agentId`, and canonical Q&A history. Created on round 1; deleted on terminal exit (`ready` / `abort` / `defer_to_deferred`). Layout: a markdown header (one `#` heading + `**...**` metadata lines) plus a single fenced YAML block holding everything machine-readable. This mirrors the existing `<spec>.progress.md` shape and keeps the state file human-readable for debugging without forcing the orchestrator to parse free-form markdown.

**YAML parse-failure recovery.** On malformed YAML in the agent's response, the orchestrator issues exactly one `SendMessage` retry with prompt body `"Re-emit only the YAML status block, exact schema."` On a second consecutive parse failure, the orchestrator synthesises an `unresolvable: logically_unresolvable` block with `detail: "agent emitted unparseable YAML twice"` and surfaces it via `AskUserQuestion` per the normal unresolvable flow. This is a catastrophic-fallback path, not an expected branch.

**Rejected alternatives:**

1. **Inline the agent prompt inside `SKILL.md` (no separate file).** Rejected: the Rule-5 blacklist would still need to live somewhere and the file would balloon back over the soft 500-line limit. Worse, the orchestrator and spec-writer would share a single context window during invocation, losing the warm-agent caching benefit and forcing every round to re-read AGENTS.md.

2. **Spec-writer agent without warm-agent reuse (fresh `Agent` every round).** Rejected: every round would re-execute AGENTS.md preflight + scope-extraction reasoning from cold. The pattern in `/task` (design ↔ design-review iteration) already uses warm reuse for the same reason; consistency is cheap.

3. **State persisted inside the spec file as an HTML comment.** Rejected: the spec is a deliverable; mixing transient state with deliverable content is an anti-pattern, and on `ready` the cleanup step would have to surgically strip the comment block. A separate file deletes cleanly.

4. **Single `unresolvable` category instead of five.** Rejected by the spec — five categories enable distinct default `suggested_action`s. Collapsing them would force every unresolvable into the same generic prompt, defeating the UX win.

5. **Resolve the tracking issue inside the subagent instead of the orchestrator.** Rejected: tracking-issue resolution is interactive (`gh issue list`, asking the user to confirm a candidate, optionally `gh issue create` with confirmation). The orchestrator owns user I/O; pushing this into the subagent would require the subagent to drive `AskUserQuestion`, which conflicts with its single-responsibility role.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create the new agent definition file with frontmatter, system prompt (input contract, YAML output schema, AGENTS.md preflight, Rule-5 blacklist mirrored from AGENTS.md with the propagation-required HTML comment, spec-format template, round-cap-awareness rules, optimization-target verbatim quote, `AskUserQuestion` constraint note). | `.claude/agents/spec-writer.md` (new) | — |
| 2 | Rewrite `/interview` skill as a thin orchestrator: entry-mode detection, spec/state-file path computation, round-1 `Agent` invocation, rounds-2+ `SendMessage` invocation, YAML parsing, branching on `ready` / `ask` / `unresolvable` (with the four action handlers for unresolvable), state-file lifecycle, YAML-parse-failure recovery. Strips the scope-extraction / question-drafting / Rule-5 blacklist content (now owned by the agent). Tracking-issue resolution (Step 5) and cross-link comment (Step 7) stay in the orchestrator. | `.claude/skills/interview/SKILL.md` | 1 (orchestrator references the agent file by path) |
| 3 | Update `AGENTS.md`: append a row to **Agent Docs** for `.claude/agents/spec-writer.md`; append a sync group to **Propagation Rule § Sync groups (canonical)** — `interview ↔ spec-writer ↔ AGENTS.md` — with the explicit annotation "Rule-5 substring-blacklist edits MUST propagate to `spec-writer.md`'s embedded copy." Also append the corresponding row to the **Propagation Rule axiom table** mapping each of the three files to its sync siblings. | `AGENTS.md` | 1, 2 |
| 4 | Live verification: build the synthetic-issue corpus on this repo (one happy-path issue, one moderately-ambiguous issue, five unresolvable-category issues, one real backlog issue for the end-to-end run). Run `/interview` against each (AC4–AC6) and `/task` end-to-end against the real backlog issue (AC7). Close synthetic issues afterwards with a comment pointing back to the verification run. | (no source files; GitHub issues + spec/state file artefacts under `ai-docs/plans/`) | 1, 2, 3 |

Four atomic tasks, three of which are markdown-content authoring and one of which is verification. Below the 7-task threshold; no further splitting needed.

## Risks

- **Rule-5 drift between AGENTS.md and `spec-writer.md`.** The Propagation Rule sync-group entry (Task 3) is the primary mitigation. Secondary mitigation: the embedded copy in `spec-writer.md` carries an HTML comment `<!-- mirrored from AGENTS.md /interview Rule 5 — propagation-required -->` so a drift detected later by `/improve` or `learnings-escalation-audit` has a unique anchor string. Tertiary mitigation: the agent's system prompt instructs the agent to read AGENTS.md as the AGENTS.md preflight — if the canonical AGENTS.md copy ever supersedes a stale embedded copy mid-session, the agent sees the canonical version too.

- **Self-referential modification.** This task modifies the `/interview` skill that `/task` Steps 1–5 use. The current in-context `/interview` (running on this very design task) is the **old** monolithic version; the new subagent-based `/interview` only takes effect after merge. There is no migration path needed — it is a clean swap. Risk is bounded to: if the new orchestrator is broken, the next `/task` run breaks at Steps 1–5. Mitigation: AC4–AC7 live tests run before merge; specifically AC7 (end-to-end `/task <issue#>`) is a smoke test of exactly this failure mode.

- **`SendMessage` semantics.** The orchestrator depends on `SendMessage(to=agentId, ...)` preserving the warm agent's context across rounds. If `SendMessage` is unavailable in the harness or the captured `agentId` expires between rounds (e.g., long user-thinking pauses), the orchestrator falls back to a fresh `Agent` call with the full state-file contents pasted into the prompt. The agent definition is already required to re-derive everything from the prompt every call — so the cold fallback degrades performance, not correctness. Add a one-line note in the orchestrator's recovery section.

- **`AskUserQuestion` per-call cap.** The tool supports up to 4 questions and up to 4 options per question. `questions_per_round_cap = 3` keeps the round inside the cap with one slot to spare for the orchestrator-injected questions on `unresolvable` (the orchestrator presents the chosen `suggested_action` plus the other applicable actions — at most 4 actions across all five categories). Verified mechanically in the agent's system prompt and in the orchestrator's unresolvable handler.

- **`gh issue` rate or auth.** Tracking-issue resolution and the cross-link comment depend on `gh` already authenticated. The current SKILL.md already relies on this and the user has the persistent rule "`gh auth` must be run by user"; no change needed. If `gh` fails, the orchestrator surfaces the error to the user (existing behaviour preserved).

- **State-file orphaning.** If the orchestrator crashes between writing the state file and reaching a terminal status, an orphan `<spec>.state.md` lingers. Mitigation: the orchestrator's round-1 logic checks for an existing state file with the same path and either resumes (if `agentId` is still live) or prompts the user "found stale state file at `<path>` — resume / discard / abort?". This is the same shape as the existing `<spec>.progress.md` resume protocol used by `/task`.

- **`agentId` capture format.** The exact wire format for `Agent` returning an ID is harness-specific. The orchestrator must store whatever string the harness returns and pass it back via `SendMessage(to=...)`; if the harness emits no usable ID, the orchestrator falls back to fresh-`Agent`-per-round. Documented as the cold fallback above.

## Test Design

**Verification is structural + live.** No Rust source is touched, so:

- **No `#[cfg(test)] mod tests`** — there is no Rust file to attach a test module to.
- **No `panic-index.md` change** — no production panic sites added.
- **No `actionlint`** — no `.github/workflows/*.yml` modified.
- **No `cargo build` / `cargo test` / `cargo clippy` / `cargo doc` test ACs** — those commands run as no-op sanity checks during the PR's CI but verify nothing about this change. They pass trivially.

The verification is split into **structural shape-checks** (Tasks 1–3) and **live runs** (Task 4).

### Structural shape-checks (deterministic, no harness invocation)

For Task 1 (`spec-writer.md` exists):

```bash
test -f .claude/agents/spec-writer.md
grep -q '^model: opus' .claude/agents/spec-writer.md
grep -q '^tools:.*Read.*Write.*Edit.*Bash' .claude/agents/spec-writer.md
grep -q 'mirrored from AGENTS.md /interview Rule 5' .claude/agents/spec-writer.md
grep -qE 'cap_reached|logically_unresolvable|external_dependency|empty_scope|user_loop' .claude/agents/spec-writer.md
grep -q 'Smallest spec sufficient' .claude/agents/spec-writer.md   # optimization-target verbatim
```

For Task 2 (orchestrator shape):

```bash
grep -qE 'Agent\(subagent_type="general-purpose", model="opus"' .claude/skills/interview/SKILL.md
grep -q 'SendMessage' .claude/skills/interview/SKILL.md
grep -q 'state.md' .claude/skills/interview/SKILL.md
# Negative checks — content moved to the agent must be GONE from the orchestrator:
! grep -qiE 'backward.compat|compat.shim|deprecat|keep.old' .claude/skills/interview/SKILL.md \
  || { echo "FAIL: Rule-5 blacklist still in SKILL.md — should be in spec-writer.md only"; exit 1; }
```

For Task 3 (AGENTS.md updates):

```bash
grep -q 'spec-writer.md' AGENTS.md
grep -qE 'interview.*spec-writer|spec-writer.*interview' AGENTS.md   # sync group line
grep -q 'Rule-5 substring-blacklist edits MUST propagate' AGENTS.md
```

These run by hand at the end of each task; not automated.

### Live runs (Task 4 — covers AC4–AC7)

**Fixture creation strategy.** Synthetic issues are opened on this repo (`maratik123/quartzite`) under a label `agent-test/spec-writer` for retrievability. Each issue body is authored to exercise exactly one branch. Issues are closed (not deleted) after each run with a comment `Closed: verification run on <commit-sha>`. The label allows post-merge cleanup via `gh issue list --label agent-test/spec-writer --state all`.

The label-and-close approach (not "issue body files passed via stdin") was chosen because:
- The orchestrator's entry path uses `gh issue view <N>` directly. Using real issues exercises the actual code path verbatim — no test-only stdin override needed.
- Closed-with-label issues remain queryable indefinitely; they document what was tested.
- The orchestrator's `gh issue comment` cross-link in AC4 has somewhere real to land — verifying AC4's "cross-link comment is posted" requirement end-to-end.

**Synthetic issue corpus (7 issues total):**

| Fixture | Issue body shape | Drives AC | Expected status |
|---|---|---|---|
| F1 happy | `"Add a one-line top-level rustdoc to crate `quartzite-paint-api/src/lib.rs` describing its public API surface."` | AC4 | round 1 → `ready` |
| F2 ambiguous | `"Speed up the runtime."` | AC5 | rounds 1..N → `ask` × ≥1, then `ready` within 4 rounds |
| F3 cap_reached | A genuinely-ambiguous issue with the user (tester) answering every clarifying question with `"defer that"` until the cap fires | AC6 | round 4 → `unresolvable: cap_reached` |
| F4 logically_unresolvable | `"Add a feature that depends on Rust stabilising specialisation."` | AC6 | round 1 → `unresolvable: logically_unresolvable` |
| F5 external_dependency | `"Migrate to wgpu 25.0 once it ships."` (wgpu 25 unreleased at run time) | AC6 | round 1 or 2 → `unresolvable: external_dependency` |
| F6 empty_scope | `"."` (intentionally empty body) | AC6 | round 1 → `unresolvable: empty_scope` |
| F7 user_loop | An issue where the tester deliberately gives contradictory answers across rounds (round 1 "yes do X", round 2 "no don't do X") | AC6 | rounds 2–3 → `unresolvable: user_loop` |
| F8 e2e | A real low-risk backlog issue (e.g. small docs typo, single-line refactor) selected at run time from `gh issue list --state open` | AC7 | full `/task` run reaches design-review GO on first pass |

For each unresolvable fixture (F3–F7), the tester verifies:
- The orchestrator surfaces the agent's `suggested_action` first via `AskUserQuestion`.
- The chosen action executes the documented effect: `defer_to_deferred` moves the spec draft to `ai-docs/plans/deferred/` and updates `INDEX.md` (status `🟡 spec-only`); `abort` deletes the partial spec; `extend_cap` resumes with `round_cap += 1`; `request_external_info` resumes with appended context.
- The state file `<spec>.state.md` is deleted on `ready` / `abort` / `defer_to_deferred` and retained on `extend_cap` / `request_external_info`.

**Test artefact handling.** Spec drafts produced for fixtures F1–F7 are deleted after the run unless they exercised `defer_to_deferred` (those go to `ai-docs/plans/deferred/` and are kept as documented evidence of the AC6 path). The F8 spec from the end-to-end run becomes a real PR — that's the AC7 deliverable.

### Round-cap-awareness rule (embedded in agent — verified by F3)

The agent's system prompt includes: "When `round == round_cap`, status MUST be `ready` or `unresolvable: cap_reached`. Emitting `ask` on the final round is a contract violation." F3 verifies this: the tester gives non-progressing answers; on round 4, the agent must not emit `ask`. The orchestrator additionally guards this — if the agent emits `ask` on `round == round_cap`, the orchestrator overrides to `unresolvable: cap_reached` and surfaces it. Both layers are necessary because hard rules in subagent prompts have historically failed (cf. `ai-docs/learnings.md` 2026-05-03 / 2026-05-05 on Rule 5).

## Open questions

None. All design-affecting decisions are pinned in the spec's Key Decisions table or resolved above (state-file layout = markdown header + fenced YAML block; YAML parse-failure recovery mechanics = one-shot `SendMessage` retry then orchestrator-injected `unresolvable: logically_unresolvable`; live-test fixture strategy = labelled synthetic issues on this repo, closed after testing; cold-`Agent` fallback for failed `SendMessage`).
