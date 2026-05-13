---
name: ai-audit
description: "Two-phase instruction audit. Phase 1 (subagent) verifies every learnings.md `Escalated?` claim points to a rule that actually exists AND that every `Superseded by:` reference resolves to a real later entry or merged PR; fixes drift in either field. Phase 2 (main session) reads all instruction files (AGENTS.md, ai-docs/*, .claude/skills/**, .claude/agents/**, hooks in settings.json) against official Claude Code docs and proposes fixes for inconsistencies, dead references, format violations, and refactor candidates."
disable-model-invocation: true
argument-hint: "[scope: 'phase1' | 'phase2' | omit for both]"
allowed-tools: Bash(rg *) Bash(grep *) Bash(find *) Bash(realpath *) Bash(jq *) Bash(git branch *) Bash(git status *) Bash(git checkout *) Bash(git rev-parse *) Bash(git diff *) Bash(git add *) Bash(git commit *)
---

# AI Audit

Compliance + structural audit of the project's instruction surface. Complements `/improve` (which detects new patterns from `learnings.md`) by checking that **already-recorded** rules are coherent, reachable, and live where they claim to live.

Steps execute **strictly in sequence**. Stop on any unrecoverable mismatch and surface to user.

## Scope argument

- `phase1` — only the escalation audit subagent
- `phase2` — only the main-session instruction audit
- omitted (default) — run Phase 1 then Phase 2

Argument received: `$ARGUMENTS`

---

## Pre-flight: branch check

Run `git branch --show-current`. If `master` and any change is anticipated:

```bash
git checkout -b chore/YYYY-MM-DD-ai-audit
```

Switch *before* any edit — uncommitted edits accumulating on master is the failure mode this guards.

---

## Phase 1 — Escalation audit (subagent)

Skip if `$ARGUMENTS` is `phase2`.

Spawn the subagent in a clean context. The subagent reads `.claude/agents/learnings-escalation-audit.md` for full instructions.

```
Agent(subagent_type="general-purpose", prompt="
  Read .claude/agents/learnings-escalation-audit.md and follow it exactly.
  Working directory: <CLAUDE_PROJECT_DIR>
  Report back: (a) entries audited, (b) mismatches found, (c) fixes applied, (d) entries that need user judgment.
")
```

After the subagent reports back:

1. Surface its summary to the user.
2. If the subagent left any entry as **needs user judgment** — present each one and ask how to resolve before continuing to Phase 2.
3. If the subagent applied edits to `ai-docs/learnings.md` or other files, do **not** auto-commit yet — Phase 2 may add more changes; bundle into one commit at the end.

---

## Phase 2 — Instruction audit (main session)

Skip if `$ARGUMENTS` is `phase1`.

### Step 2.1: Pull canonical Claude Code docs

Fetch the three primary references via WebFetch (cache them mentally for the rest of the run):

- `https://code.claude.com/docs/en/skills` — skill structure, frontmatter, allowed-tools
- `https://code.claude.com/docs/en/sub-agents` — agent file shape, when subagents fire
- `https://code.claude.com/docs/en/hooks-guide` — hook events, matchers, JSON I/O

For each fetch use a focused prompt like: *"Extract the canonical schema for skill frontmatter fields and which fields are required vs optional."*

If a referenced behavior in the codebase is unclear, fetch additional pages from `code.claude.com` (settings reference, MCP, slash commands) on demand — do not pre-fetch everything.

### Step 2.2: Inventory the instruction surface

Read every file in these locations (use Read, not grep — content matters, not just keywords):

- `AGENTS.md` and `CLAUDE.md`
- `ai-docs/context.md`, `ai-docs/doc-convention.md`, `ai-docs/learnings.md`, `ai-docs/deferred-items.md`
- `ai-docs/plans/INDEX.md` and any active `*.spec.md` / `*.design.md` / `*.progress.md` (skip `done/` and `deferred/` content; just note their existence)
- Every `.claude/skills/*/SKILL.md`
- Every `.claude/agents/*.md`
- `.claude/settings.json` (hooks + permissions)

Do **not** read `.claude/settings.local.json` content unless the user explicitly authorizes it — it is user-local state.

### Step 2.3: Run the checklist

For each item below, when a violation is found record: file path, line number (where applicable), the rule it conflicts with, the proposed fix.

#### A. Cross-reference integrity
- Every relative link (`../`, `./`, file references in prose) resolves to an existing file. Verify with `realpath` from the link's source directory or with `find`.
- Every `[text](file.md)` and bare `file.md` mentioned in instructions points to a file that exists.
- Every skill or agent named in another skill/agent (e.g., `code-review` references `review-findings`, `self-review`) actually has a matching file.

#### B. Conflicting / duplicated rules
- The same topic must not have contradictory guidance in two places (e.g., commit policy in AGENTS.md vs. in a skill).
- Verbatim-duplicated rules across files → consolidate to one canonical home + reference.
- Rule says "see `<other-file>`" — confirm the target file actually contains that rule.

#### C. Dead references
- Skills/agents named in `AGENTS.md` "Sync groups" must exist. (E.g., the AGENTS.md note about `task` ↔ `task-issue` collapse — verify no stale references remain.)
- Agent names referenced in skills must match a file under `.claude/agents/`.
- `ai-docs/plans/done/` references in agent checklists must still resolve.

#### D. Frontmatter conformance (skills)
Per the official docs:
- Every `SKILL.md` has YAML frontmatter with at minimum `name` and `description`.
- `description` should make trigger conditions clear (when to invoke).
- `disable-model-invocation: true` ↔ skill is user-only — verify intent matches.
- `argument-hint` style is consistent across skills.
- `allowed-tools` syntax matches `ToolName(pattern)` form documented at `code.claude.com/docs/en/skills`.

#### E. Frontmatter conformance (agents)
- Every `.claude/agents/*.md` starts with YAML frontmatter (`---`-delimited block at top of file). An agent without frontmatter is not a subagent — it's a stray document. Enumerate the directory at audit time rather than baking in a count.
- `name` field equals the file basename.
- `description` is one line and tells the orchestrator when to spawn this agent.

#### F. Hooks (`.claude/settings.json`)
- Each hook event name is one of the documented set (`SessionStart`, `PreToolUse`, `PostToolUse`, etc. — confirm against `hooks-guide`).
- Matchers are valid tool name patterns.
- Hook commands fail closed (`exit 2` for blocking) where intended; non-blocking informational hooks use stderr without `exit 2`.
- Timeouts are reasonable (≤30s default, longer only when the work demands it).
- Commands quote `$CLAUDE_PROJECT_DIR` and other env vars correctly — no shell injection footguns.

#### G. AGENTS.md "Propagation Rule" coherence
- Every "sync group" listed in AGENTS.md still has all listed members present and cross-referenced.
- Behaviors described in AGENTS.md and replicated in agent checklists agree (e.g., file-size hard/soft limits in `review-findings.md` match AGENTS.md).
- Exemptions in AGENTS.md (e.g., `examples/` and `benches/` no-test exemptions, trait-impl doc-convention exemption) appear in every enforcement file.

#### H. Documentation conformance pointers
- `ai-docs/doc-convention.md` is referenced by `review-findings.md` and `self-review.md`. Confirm the relative paths resolve.
- The canonical section order listed in AGENTS.md "Documentation Conventions" matches `doc-convention.md` exactly.

#### I. File-size & structure (instruction files)
- No `SKILL.md` or agent file exceeds ~500 lines without clear sectioning. Long files should split into a thin `SKILL.md` + reference file (the `improve` / `code-review` / `task` skills use this pattern).
- Each skill directory contains exactly one `SKILL.md` (no extra markdown unless intentional reference material).

#### J. Allow-list / permission consistency
- Tools used in skills' `allowed-tools` should be present (or coverable by) the `permissions.allow` list in `settings.json` — otherwise the user gets a prompt every time.
- Conversely: any allow-listed pattern that no skill actually uses is dead and should be reviewed.

### Step 2.4: Categorise findings

Group by severity:

- `blocker` — broken link, dead reference, frontmatter missing → instruction is unusable as written.
- `major` — contradicting rules, drifted exemption, hook with shell injection.
- `minor` — duplication, inconsistent style, unhelpful description.
- `nit` — wording, ordering, formatting.

Cap the report at 25 findings (same convention as `review-findings`); if more exist, list the 25 most severe and note the truncation.

### Step 2.5: Present + apply

Show the user a numbered list of findings with proposed fixes (concrete diffs for non-trivial edits). For each finding:

- `blocker` / `major`: ask user to confirm before applying.
- `minor` / `nit`: may apply autonomously if the fix is mechanical and obvious; otherwise ask.

Apply approved fixes via `Edit` / `Write`. Update `ai-docs/learnings.md` with a new entry per AGENTS.md "Corrections Log" format **only if** the audit revealed a *new* class of mistake worth tracking — do not log routine cleanup.

### Step 2.6: Verify

After edits:

1. Re-run any `find`/`grep` checks from Step 2.3 that detected violations — confirm zero remaining.
2. If hooks were edited, eyeball the `.claude/settings.json` JSON validity: `jq . .claude/settings.json`.
3. If agent or skill files were edited, confirm their frontmatter still parses by reading the file back.

---

## Step 3: Commit (if changes were made)

Stage explicitly:

```bash
git add .claude/skills/ai-audit/SKILL.md \
        .claude/agents/learnings-escalation-audit.md \
        <any other files actually edited>
git commit -m "$(cat <<'EOF'
chore(instructions): audit + fix [phase1|phase2|both]

[brief summary of fixes by category — A/B/C/... from the checklist]
EOF
)"
```

If `ai-docs/learnings.md` was modified by Phase 1 or Phase 2, stage it together — per AGENTS.md "Workflow", learnings entries are part of the deliverable and must be visible in the diff.

Per AGENTS.md, **never** `git add -A` / `git add .`.

---

## Gate checklist

| Before | Check |
|---|---|
| Phase 1 spawn | not on master OR no edits planned? |
| Phase 1 done | subagent reported every `needs user judgment` item? |
| Phase 2 fetch | all three docs successfully fetched? if a fetch failed, surface and ask whether to proceed without it |
| Phase 2 apply | `major` / `blocker` user-approved? |
| Commit | `jq .` passes on settings.json? frontmatter still valid on edited skills/agents? |

## Anti-patterns

- Do **not** rewrite `learnings.md` history — it is append-only. Phase 1 may only correct the `Escalated?` field of an existing entry or add a *new* corrective entry; never delete or rephrase past entries.
- Do **not** invent rules. The audit finds compliance gaps in *existing* rules; new rules go through `/improve`.
- Do **not** skip the WebFetch step in Phase 2. The official docs are the source of truth for hook/skill/agent shapes — relying on memory is the failure mode this skill exists to prevent.
- Do **not** auto-resolve a blocker without surfacing it. Severity is a signal that human judgment is needed.
- Do **not** edit `.claude/settings.local.json` from this skill — it is user-local.
