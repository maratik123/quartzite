---
name: improve
description: "Analyze corrections log (ai-docs/learnings.md), find repeating patterns, propose instruction updates and escalation to hooks."
disable-model-invocation: true
argument-hint: "[optional context]"
---

Launch the `self-improve` subagent.

The subagent reads `.claude/agents/self-improve.md` for full instructions.

The subagent will:
1. Read `ai-docs/learnings.md` for all correction records
2. Find repeating patterns (same mistake ≥2 times)
3. Propose concrete diffs to `AGENTS.md` or skill/agent files
4. For mistakes that repeat ≥3 times despite existing rules — propose escalation to hooks in `.claude/settings.json`
5. Apply changes after user confirmation
6. Run a targeted eval to verify the fix works. Reports PASS/FAIL.

Run when **≥3 unescalated correction entries**, **≥2 unescalated validation entries**, or a `🌱 Stale-validation` flag from `/ai-audit` accumulates. Mirror of the threshold line in `AGENTS.md § Learning Log` — keep both in sync per the Propagation Rule.

See also: `/triage` (`.claude/skills/triage/SKILL.md`) — same batched-approval and ≥3-unhandled threshold patterns; diverges in mutation scope (mutates `ai-docs/deferred/**` + `gh issue create/edit` rather than instruction files + `learnings.md`).

Context from user (if any): $ARGUMENTS
