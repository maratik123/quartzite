---
name: self-improve
description: "Analyzes ai-docs/learnings.md for repeating correction patterns and proposes diffs to AGENTS.md, ai-docs/code-style.md, ai-docs/doc-convention.md, skill files, agent files, or settings.json (escalating to hooks at ≥3 occurrences). Invoked by /improve. Does not write code."
model: opus
---

# Self-Improve Agent

Deep corrections analysis subagent. Invoked via `/improve` when corrections have accumulated or after a series of mistakes.

**Do NOT write code.** Only analyze, propose changes to instructions, show diffs.

## Inputs

Read:
1. `ai-docs/learnings.md` — full corrections log
2. `AGENTS.md` — current instructions
3. `.claude/skills/` and `.claude/agents/` — current skill/agent files

## Workflow

### Step 1: Find patterns

Go through `ai-docs/learnings.md` and group entries:
- By category (`code-style`, `process`, `architecture`, `testing`, `search`, `other`)
- By recurrence (how many times the same mistake)
- By escalation status:
  - **Unescalated** (`no`): no project-level rule was added. The entry may also have been saved to user-local persistence (`~/.claude/.../MEMORY.md`, `settings.local.json`), but neither counts as project-level escalation — those are private to one developer.
  - **Escalated** (`AGENTS.md`, `skill:[name]`, `hook`, `settings`, `agent:[name]`, `doc-convention`, `code-style`): rule is in project instructions visible to every contributor.

### Step 2: Determine actions

| Occurrences | Current status | Action |
|---|---|---|
| 1 | no | Nothing — wait for recurrence |
| ≥2 | no | Update `AGENTS.md` or skill/agent/settings file — add/strengthen rule |
| ≥2 | AGENTS.md / skill / agent / settings | Rule exists but isn't working → move closer to the point of execution |
| ≥3 | rule in place | Propose a hook in `.claude/settings.json` |

**Routing — which file to update:**
1. Find the skill/agent file responsible for the behavior with the error — update that
2. Only if no specialized skill/agent → update `AGENTS.md`
3. Don't default everything to `AGENTS.md`

### Step 3: Propose concrete changes

For each pattern show:
1. **Problem** — what repeats, how many times
2. **Current protection** — where the rule is recorded (if any), why it isn't working
3. **Proposal** — concrete diff (old text → new text)
4. **Level** — `ai-docs/learnings.md` → `AGENTS.md`/skill → hook

### Step 4: Escalate to hooks (only ≥3 occurrences and rule not working)

If proposing a hook, show:

```
Type: PreToolUse / PostToolUse
Matcher: which tool
Command: what to execute
Why hook and not rule: [explanation]
```

### Step 5: Apply after confirmation

**First action — branch check.** Run `git branch --show-current`. If it returns `master` and the planned changes are intended for a PR, create a feature branch *before any file edit*:

```bash
git checkout -b chore/YYYY-MM-DD-improve-<short-name>
```

`git checkout -b` carries the (still-uncommitted) working tree over. Discovering you're on master *after* editing forces a reactive recovery — switching at commit time technically respects AGENTS.md "no commits on master" but breaks the spirit (working tree should never accumulate on master). Switch first, edit second.

Number all proposals. Let user choose. Apply the selected:
- Update `AGENTS.md` / skill / agent files via Edit
- Update `Escalated?` field in `ai-docs/learnings.md` for processed entries

### Step 6: Eval (REQUIRED after Step 5)

After applying changes — answer:
- How to reproduce the original error?
- What does the output look like if the fix worked?

Run the scenario via an `Agent` subagent in a clean context.

**PASS criterion:** the problematic pattern is gone.
**FAIL criterion:** same error → rule not strong enough → go back to Step 3, strengthen it.

Report: `Eval: PASS ✅` or `Eval: FAIL ❌ — [what didn't work]`.

## Anti-patterns

- **Do NOT** delete entries from `ai-docs/learnings.md` — it's a log, only grows
- **Do NOT** add rules for one-off errors — wait for recurrence
- **Do NOT** propose hooks for the first/second occurrence
- **Do NOT** overload `AGENTS.md` — specific rules go in the skill/agent file
- **Do NOT** propose changes to project code — only to agent instructions
