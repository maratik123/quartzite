---
name: interview
description: "Requirements interview with the product owner. Output: spec saved to ai-docs/plans/, cross-linked with a tracking GitHub issue. Invoked by `/task` for Steps 1–5, or run standalone for spec-only work that defers implementation."
argument-hint: "[issue-number | task description]"
allowed-tools: Bash(gh issue view *) Bash(gh issue list *) Bash(gh issue create *) Bash(gh issue comment *)
---

Orchestrator for the spec-drafting interview. Drives the round loop, surfaces the subagent's questions to the user, and applies the user's answers — but does **not** draft the spec itself. Spec drafting and question generation live in `.claude/agents/spec-writer.md` (subagent on `model: opus`).

> **MUST run before:** code investigation, design agent, or writing code.
> Run standalone when you want a spec without committing to implementation (defer it to `ai-docs/plans/deferred/` afterward).
> For the full task workflow use `/task` — it delegates Steps 1–5 to this skill, then continues with design → implementation → PR.

## Architecture

Two pieces:

1. **This file** (orchestrator) — plumbing only. Detects entry mode, manages state, runs the round loop, parses the subagent's YAML status block, surfaces questions via `AskUserQuestion`, executes action handlers on `unresolvable`, posts the cross-link comment on `ready`.
2. **`.claude/agents/spec-writer.md`** (subagent, `model: opus`) — owns scope extraction, question drafting, AGENTS.md preflight, the Rule-5 substring blacklist, the optimization-target enforcement, and the spec write itself.

## Round / question caps

| Constant | Value | Where |
|---|---|---|
| `round_cap` | 4 | Hard-coded in this skill; passed to subagent every invocation |
| `questions_per_round_cap` | 3 | Hard-coded in this skill; passed to subagent every invocation |

These constants are **not** configurable via skill arguments in this iteration. Future configurability is a separate concern (deferred — see issue #188 spec).

## State file

Path: `<spec_path>.state.md` — e.g. `ai-docs/plans/2026-05-09-name.spec.md` ↔ `ai-docs/plans/2026-05-09-name.spec.md.state.md`.

Created at the start of round 1; deleted on terminal exit (`ready` / `abort` / `defer_to_deferred`). Format: markdown header + a single fenced YAML block.

```markdown
# Interview state — <task name>

Transient handoff between rounds. Deleted on terminal exit.

```yaml
schema_version: 1
spec_path: ai-docs/plans/YYYY-MM-DD-name.spec.md
issue_ref: "#188"
round_cap: 4
questions_per_round_cap: 3
round: 2
agent_id: <captured from round-1 Agent invocation; null if cold-Agent path>
prior_qa:
  - round: 1
    question: "..."
    answer: "..."
  - round: 1
    question: "..."
    answer: "..."
```
```

## Workflow

### Step 1: Detect entry mode

Inspect `$ARGUMENTS`:

- **Issue ref** — matches `^#?\d+$`: load `gh issue view <N> --json title,body,comments` once. Record `tracking_issue = <N>`.
- **Free text / empty**: use as task description, or ask "What do you want to plan?" if empty. `tracking_issue` is unset until Step 5.

### Step 2: Compute paths and seed state

1. Derive a kebab-case spec slug from the issue title (or task description), ≤ 5 words.
2. `spec_path = ai-docs/plans/<TODAY>-<slug>.spec.md`
3. `state_path = <spec_path>.state.md`
4. Write the initial state file with `round: 1`, `prior_qa: []`, `agent_id: null`.

### Step 3: Round loop

For each round (1..=`round_cap`):

#### 3a. Invoke the subagent

**Round 1 — cold spawn (primary path):**

```
Agent(
  subagent_type="general-purpose",
  model="opus",
  prompt="""
    Read .claude/agents/spec-writer.md and follow it.

    issue_ref: <#N | "free-text">
    issue_body: |
      <verbatim from gh issue view, OR the user's free-text task description>
    round: 1
    round_cap: 4
    questions_per_round_cap: 3
    prior_qa: []
    spec_path: <spec_path>
  """
)
```

Capture the returned `agentId` into the state file's `agent_id`. If the harness does not return a usable `agentId`, leave it null — rounds 2+ will use the cold-spawn fallback.

**Rounds 2..cap — warm reuse if possible, cold fallback otherwise:**

- If `agent_id` is set in state: `SendMessage(to=agent_id, prompt="""<same fields with updated round + prior_qa>""")`. Capture the response.
- If `agent_id` is null OR the `SendMessage` call fails: cold spawn a fresh `Agent(model="opus", prompt=...)` with the full state in the prompt (the agent definition mandates re-derivation from prompt anyway). Update state file's `agent_id` from the new spawn (may again be null).

> The cold-spawn path is the **default contract**; warm reuse is an opportunistic optimization conditional on the harness returning a usable `agentId` and `SendMessage` succeeding.

#### 3b. Parse the YAML status block

The subagent's response ends with a fenced YAML block. Extract it; parse `status`, `round`, and `questions` / `reason` as applicable.

**On parse failure** (malformed YAML, missing required fields):

1. **One-shot retry** — `SendMessage(to=agent_id, ...)` (or fresh `Agent` if cold) with prompt: `"Re-emit only the YAML status block, exact schema. Your previous response did not contain a parseable status block at the end."` Parse again.
2. **On second failure** — orchestrator-injects a synthetic `unresolvable` and proceeds to 3d:
   ```yaml
   status: unresolvable
   round: <current>
   reason:
     category: logically_unresolvable
     detail: "Spec-writer subagent emitted unparseable YAML status twice."
     suggested_action: abort
   ```

#### 3c. Branch on status

- **`ready`** → go to **Step 4**.
- **`ask`** → go to **3d** (surface questions).
- **`unresolvable`** → go to **3e** (action chooser).

#### 3d. Surface questions to the user

Validate before forwarding:

- `len(questions) <= questions_per_round_cap` — if exceeded, send the agent a one-shot trim instruction (`"Trim to <cap> highest-leverage questions; emit only the YAML status block."`).
- Each `header` ≤ 12 chars; each `options` list has 2..=4 entries (`AskUserQuestion` constraints).
- No question contains a Rule-5 blacklisted substring (final defence — the subagent should have caught it). On violation: one-shot agent re-spawn with explicit instruction to re-read AGENTS.md and the Rule-5 blacklist.

Then call `AskUserQuestion(questions=[...])` with the entire list — the tool supports up to 4 questions per call, so 1..=3 fit cleanly. The user answers all in a single UI exchange.

Append each `(question, answer)` pair to state's `prior_qa` with `round: <current>`. Increment `round`. Loop to 3a.

#### 3e. Action chooser on `unresolvable`

Build an `AskUserQuestion` with:

- The agent's `reason.detail` as the question prose.
- Options: the agent's `suggested_action` first (recommended), plus the other applicable actions per the table below.

| Category | Actions to offer (recommended first) |
|---|---|
| `cap_reached` | `extend_cap` (recommended), `defer_to_deferred`, `abort` |
| `logically_unresolvable` | `defer_to_deferred` (recommended), `abort`, `request_external_info` |
| `external_dependency` | `request_external_info` (recommended), `defer_to_deferred`, `abort` |
| `empty_scope` | `abort` (recommended), `request_external_info`, `defer_to_deferred` |
| `user_loop` | `defer_to_deferred` (recommended), `abort`, `request_external_info` |

Execute the chosen action:

- **`extend_cap`** — bump `round_cap += 1` in state; loop to 3a with `round: <current> + 1`. The agent receives the new `round_cap` and may now `ask` if it has questions.
- **`defer_to_deferred`** — `mv <spec_path> ai-docs/plans/deferred/`; update `INDEX.md` (move row to **Deferred plans**, status `🟡 spec-only`); delete state file; exit. Skip Step 4.
- **`abort`** — delete `<spec_path>` (if exists); delete state file; exit. Skip Step 4.
- **`request_external_info`** — prompt the user via `AskUserQuestion` (single free-form question option) for the additional context; loop to 3a with `extra_context: <user paste>` injected into the next round's prompt.

### Step 4: Cross-link and exit (on `ready`)

1. Show the user the final spec at `<spec_path>` (last 80 lines if long).
2. Confirm — `AskUserQuestion`: "Approve and post cross-link comment?" / { Approve, Tweak first }.
3. On Approve:
   - Resolve the tracking issue if not already pinned (issue-ref mode = already pinned; free-text mode = run the issue-search / propose-new flow):
     ```bash
     gh issue list --state open --search "<keyword>"
     ```
     If a candidate exists, ask user. Otherwise propose a new issue (title from spec name; body from spec scope) and run `gh issue create` after user approval. Capture the number into the spec's `**Tracked in:**` field if it wasn't there.
   - Post the cross-link:
     ```bash
     gh issue comment <N> --body "Spec: \`<spec_path>\`"
     ```
   - Delete the state file.
4. Skill exits. `/task` (the caller) resumes at Step 6 (design agent).

> **Skip the tracking-issue resolution only if the user explicitly states "no tracking issue".** Note the reason in the spec header (`**Tracked in:** none — <reason>`) and skip the cross-link comment.

## Spec-only run

If the user wants to stop after the interview ("just draft the spec, defer the implementation"):

1. Move the spec to `ai-docs/plans/deferred/`
2. Update `INDEX.md` (move row to **Deferred plans**, status `🟡 spec-only`)
3. Delete the state file
4. Do NOT proceed to Step 6 of `/task`. The spec can be picked up later via `/task`'s deferred-plan-activation preamble.

## Anti-patterns

- Drafting questions yourself in the orchestrator. The subagent owns question authorship; you forward the questions verbatim.
- Mutating the spec yourself. The subagent owns spec writes; the orchestrator only reads it.
- Skipping the YAML status parse and inferring intent from prose. The status block is the contract; treat parse failure as a defect.
- Embedding the Rule-5 substring blacklist in this file. It lives in the agent definition; this orchestrator's only Rule-5 role is the validation gate at 3d (defence in depth).
- Forgetting to delete the state file on terminal exit. It's transient handoff; orphaned state files confuse subsequent runs.
- Saving the spec without `**Tracked in:**` (unless user explicitly opted out).
- Skipping the cross-link comment on the tracking issue.
- **Silently switching to implementation mid-interview.** If the subagent's first round suggests the task is trivially small (< ~20 lines, no design decisions), the agent should still emit `ready` with a complete spec; the orchestrator surfaces it normally and the user can choose to spec-only-defer if they want a one-shot edit instead.
