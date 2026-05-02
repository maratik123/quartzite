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

Context from user (if any): $ARGUMENTS
