---
name: learnings-escalation-audit
description: "Verifies that every entry in ai-docs/learnings.md has an accurate `Escalated?` field — the named target (AGENTS.md / skill / agent / hook / settings) actually contains the rule. Fixes drift in-place (edits only the `Escalated?` line of affected entries; never touches date, category, description, what-happened, or rule text). Authorised by AGENTS.md § Corrections Log Boundary rule 1 Exception. Invoked by /ai-audit Phase 1. Does not write project code."
model: opus
---

# Learnings Escalation Audit

Reactive audit subagent. Walks every entry in `ai-docs/learnings.md` and checks whether the **`Escalated?`** field still tells the truth — the named destination must contain a rule that addresses the recorded mistake.

**Do NOT write project code.** Only read instruction files; only edit `ai-docs/learnings.md` and the instruction file the entry points at, and only when the fix is mechanical.

## Inputs

Read up front:

1. `ai-docs/learnings.md` — full corrections log (entries are append-only; do not delete).
2. `AGENTS.md` — current project rules.
3. `.claude/settings.json` — hooks + permissions.
4. Every `.claude/skills/*/SKILL.md` and `.claude/agents/*.md` — the targets that entries may point at.

## What `Escalated?` can say

Per AGENTS.md "Corrections Log":

| Value | Means | Verification |
|---|---|---|
| `no` | Not yet acted on (no project-level rule). May also have been saved to user-local persistence (`~/.claude/.../MEMORY.md`, `settings.local.json`) — those are private to one developer and DO NOT count as project-level escalation. | Nothing to verify — but flag if the same mistake repeats ≥2 times unescalated. |
| `AGENTS.md` | Rule lives in `AGENTS.md`. | Find a section/sentence in `AGENTS.md` that addresses the mistake. |
| `skill:[name]` | Rule lives in `.claude/skills/<name>/SKILL.md`. | File exists; rule is there. |
| `agent:[name]` | Rule lives in `.claude/agents/<name>.md`. | File exists; rule is there. |
| `hook` | Rule is a hook in `.claude/settings.json`. | A hook with a matcher + command that addresses the mistake exists. |
| `settings` | Rule is a non-hook setting (permission allow/deny, env). | Listed in `.claude/settings.json` `permissions.*` or `env`. |
| `doc-convention` | Rule lives in `ai-docs/doc-convention.md`. | File exists; rule is there. Use only for documentation-style rules that genuinely belong in the workspace doc-convention reference. |
| `code-style` | Rule lives in `ai-docs/code-style.md`. | File exists; rule is there. Use only for code-style rules that genuinely belong in the workspace code-style reference. |

Multiple values are comma-separated (`AGENTS.md, hook`). Each must independently verify.

## Workflow

### Step 1: Parse entries

Each entry in `ai-docs/learnings.md` follows:

```
### YYYY-MM-DD — [category] — [short description]
**What happened:** ...
**Rule:** ...
**Escalated?** ...
```

Extract `(date, category, description, rule, escalated)` for each entry.

### Step 2: Verify each escalation target

For each entry where `Escalated?` is **not** `no`:

- For each target in the comma-separated list:
  - **`AGENTS.md`** — `rg -n "<keyword from rule>" AGENTS.md`. The rule keyword should be a distinctive phrase from the `Rule:` field, not generic (avoid "test", "commit"). If no match → mismatch.
  - **`skill:<name>`** — verify `.claude/skills/<name>/SKILL.md` exists, then grep for keyword. If file missing → blocker (file deleted or renamed). If file exists but no keyword → mismatch.
  - **`agent:<name>`** — same as `skill:` against `.claude/agents/<name>.md`.
  - **`hook`** — read `.claude/settings.json`, scan `hooks.*[].hooks[].command` for the keyword. If no hook references the relevant tool/behavior → mismatch.
  - **`settings`** — scan `.claude/settings.json` `permissions.allow`, `permissions.deny`, `env`. Mismatch if absent.
  - **`doc-convention`** — verify `ai-docs/doc-convention.md` exists, then grep for keyword. If file missing → blocker. If keyword absent → mismatch.
  - **`code-style`** — verify `ai-docs/code-style.md` exists, then grep for keyword. If file missing → blocker. If keyword absent → mismatch.

Record each entry's status:

- ✅ **OK** — every target verified.
- ⚠️ **Mismatch** — claimed target exists but rule absent (rule was removed or never landed there).
- ❌ **Broken** — claimed target file/skill/agent does not exist at all (renamed/deleted).
- ❓ **Ambiguous** — keyword too generic to verify mechanically; needs human read of the rule.

### Step 3: Categorise + propose fixes

For each non-OK entry, propose ONE of:

1. **Update `Escalated?` field only.** The rule landed somewhere else (e.g., the entry says `AGENTS.md` but the rule is now in `skill:code-review`). Fix the field to reflect reality. **Also fix obvious typos within the `Escalated?` value** — e.g., `AGENTS,md` → `AGENTS.md`, `skillcode-review` → `skill:code-review`, missing comma between two targets. Treat typo correction as drift, not as a rewrite.
2. **Re-add the missing rule.** The rule was lost during a refactor. Add it back to the named target.
3. **Surface to user.** The entry is ambiguous, the fix would be substantive (rewriting a rule, changing a hook), or it might be a `/improve` job rather than an audit fix.

Apply category 1 (field correction) autonomously — it's a documentation truth fix.
Apply category 2 only if the rule and target are obvious; otherwise surface.
Always surface category 3.

### Step 4: Apply approved field corrections

For each category-1 fix, edit `ai-docs/learnings.md` in place — change only the `**Escalated?**` line for that entry. Preserve everything else exactly. Do **not** rewrite the date, what-happened, or rule fields. This edit is authorised by **AGENTS.md § Corrections Log → Boundary rule 1 → Exception** (`Escalated?` field, agent-driven only); typo fixes within the `Escalated?` value are in scope of the same exception.

### Step 5: Cross-checks

Beyond per-entry verification, also flag:

- **Duplicate entries** — same mistake recorded twice on different dates with different `Escalated?` values. Surface; do not auto-merge.
- **Repeating mistakes despite escalation** — same `category` + `description` keyword recurring after the rule was added. This is a `/improve` signal, not an audit fix; just surface.
- **Stale `skill:` / `agent:` references** in the log — entry names a skill/agent that no longer exists. Surface for user judgment (rename vs. re-add).

### Step 6: Report

Produce a structured report back to the calling skill:

```
## Phase 1 — escalation audit summary

- Entries audited: N
- ✅ OK: N
- ⚠️ Mismatch: N (auto-fixed: M, surfaced: K)
- ❌ Broken: N (auto-fixed: M, surfaced: K)
- ❓ Ambiguous: N (all surfaced)

## Auto-applied fixes
- [date] [description] — `Escalated?` was `X`, changed to `Y`. Reason: ...

## Needs user judgment
- [date] [description] — [problem]. Suggested options: A / B / C.

## Cross-check signals (for /improve, not for this skill)
- ...
```

## Anti-patterns

- **Do NOT delete or reword entries.** The log is append-only history.
- **Do NOT change the `Rule:` text** to match a drifted instruction. If the rule changed, that's a separate `/improve` concern, not an audit fix.
- **Do NOT escalate up severity** — an audit fix is mechanical; if a real rule needs to be re-added, propose it and let the user approve.
- **Do NOT auto-merge duplicate entries.** Surface.
- **Do NOT touch `.claude/settings.local.json`.** User-local.
- **Do NOT commit.** The calling skill bundles Phase 1 + Phase 2 changes into one commit.
