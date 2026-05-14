# Corrections Log — reference

This page extracts the field-level glossary and exception bodies from [`AGENTS.md` § Corrections Log](../AGENTS.md#corrections-log). The boundary-rule AXIOM blockquotes themselves stay in AGENTS.md.

## Boundary rule 1 Exception

> **Exception — `Escalated?` and `Superseded by:` fields, agent-driven only.** The `Escalated?` line AND the optional `Superseded by:` line of an existing entry MAY be updated (or — for `Superseded by:` only — added when absent), **and only**:
>
> - by the `self-improve` agent (invoked via `/improve`), after the named instruction-file change has landed:
>   - update `Escalated?` to replace the prior value with the comma-separated list of targets actually modified; AND/OR
>   - add or update `Superseded by:` on a *prior* entry when the Commit A change reverses, refines, generalizes, subsumes, or withdraws that entry's rule. Write to the prior entry's `Superseded by:`, not the new entry. Reference format: `[ref] — [one-line reason]`, where `[ref]` is `YYYY-MM-DD` (date of a later entry; disambiguate by quoted slug when multiple entries share that date), `PR #N`, or both comma-separated.
> - by the `learnings-escalation-audit` agent (invoked via `/ai-audit` Phase 1):
>   - fix drift in `Escalated?` (target file no longer contains the rule, OR `Escalated? no` despite the rule existing in a target file);
>   - fix drift in `Superseded by:` (date reference doesn't match any later entry; `PR #N` reference is not a real merged PR);
>   - fix obvious typos within either value (e.g., `AGENTS,md` → `AGENTS.md`, `skillcode-review` → `skill:code-review`, missing comma between two targets, mistyped date in `Superseded by:`).
>
> All other lines of the entry (date, category, description, **What happened**, **Rule**) remain immutable. New learning entries are still append-only. Manual user edits to `Escalated?` / `Superseded by:` are NOT authorised by this exception — invoke `/ai-audit` or explicitly request the change.

## Boundary rule 2 Exception

> **Exception — `/improve` and `/ai-audit` workflows.** During a `/improve` run, the `self-improve` agent MAY (a) update the `Escalated?` field of the specific entries it just escalated, AND/OR (b) add or update the `Superseded by:` field of any prior entry whose rule the Commit A change reversed, refined, generalized, subsumed, or withdrew — both done **after** the instruction-file edit has been staged (separate commit on the same feature branch). During `/ai-audit` Phase 1, the `learnings-escalation-audit` agent MAY fix drift in either `Escalated?` or `Superseded by:` and edit the named target file to align them. These exceptions apply to **existing-entry `Escalated?` and `Superseded by:` updates only** — NEW learning entries STILL cannot be appended in the same turn as instruction-file edits (Rule 2's main protection — "I wrote a learning, therefore I'm authorised to escalate it" — stays intact).

> **Exception — in-flow learning capture during `/task` Steps 8–12.** A NEW learning entry MAY be appended to `ai-docs/learnings.md` in the same conversation turn as an instruction-file edit when **all** of these hold:
>
> 1. The running skill is `/task`, currently in Steps 8 (Implementation), 9 (Verify), 10 (Self-review), 11 (Fix), or 12 (Finalise) — **or any sub-skill (e.g., `/bugfix`, `/context-reset`) invoked from within that range**. The parent `/task` workflow owns the staging conflict; sub-skill detours inherit the carve-out's scope. The Boundary Rule 2 main-body protection against "I wrote a learning, therefore I'm authorised to escalate it" still applies — the sub-skill inheritance widens same-turn-write rights only, not escalation rights.
> 2. The entry's **content** documents an insight gained **during** the task being implemented — an observation, a non-obvious decision, a corner the implementer just learned exists. It does **not** pre-emptively escalate a rule that the task itself is already landing into AGENTS.md / a skill / an agent / `ai-docs/code-style.md` / `ai-docs/doc-convention.md`.
> 3. The entry is marked `Escalated? no`. The same-turn write resolves the **staging conflict** with AGENTS.md `## Workflow`'s pre-commit directive ("stage `learnings.md` together with the related code changes — learnings are part of the task deliverable and must be visible in the PR diff") — it does **not** authorise any project-level escalation in the same turn. Escalation always requires a separate `/improve` invocation per Rule 2's main body.
>
> The two failure modes Boundary Rule 2's main body protects against — "I wrote a learning, therefore I'm authorised to escalate it" and "the user asked me to record this, therefore I should also fix AGENTS.md" — both remain forbidden. This exception only lets the corpus-building action ("I noticed something useful while implementing the spec; let me record it before compaction") survive the in-flight `/task` workflow without an artificial turn split.
>
> Outside a `/task` flow (bare instruction-file edits, `/improve`, `/ai-audit`, `/code-review`, `/pr-commented`, `/triage`, ad-hoc rule changes), the main rule still applies — NEW learning entries are forbidden in the same turn as instruction-file edits. The carve-out is `/task`-specific because the staging conflict is `/task`-specific.

## Entry format — field glossary

> `Escalated?` records **project-level** persistence only — instruction files visible to every contributor (`AGENTS.md`, skills, agents, hooks, project `settings.json`, `ai-docs/doc-convention.md`, `ai-docs/code-style.md`). **User-local persistence does NOT count and is NOT a value of this field** — that includes the auto-memory store (`~/.claude/.../MEMORY.md`) and `settings.local.json`, both of which are private to one developer and don't help future readers. If a correction was saved only to user-local memory, mark `Escalated? no`; the entry remains a candidate for project-level escalation by `/improve`.
>
> `doc-convention` = the rule landed in `ai-docs/doc-convention.md`. Use only for documentation-style rules that genuinely belong in the workspace doc-convention reference rather than in AGENTS.md or a skill.
>
> `code-style` = the rule landed in `ai-docs/code-style.md`. Use only for code-style rules that genuinely belong in the workspace code-style reference rather than in AGENTS.md or a skill.
>
> `Superseded by:` records that the rule recorded above was later reversed, refined, generalized, subsumed, or withdrawn. The field is **optional** and absent from most entries. `[ref]` is one of: a `YYYY-MM-DD` date matching a later learnings entry (when multiple entries share that date, disambiguate by appending a quoted slug from the other entry's description — e.g., `2026-05-08 ("mutually exclusive markers")`); a `PR #N` reference to a merged PR that reversed the rule directly in instruction files; or both, comma-separated. The `[one-line reason]` is freeform — a short note explaining the nature of the supersession (reversed / refined / generalized / subsumed / withdrawn). Maintained by `self-improve` (via `/improve`) and `learnings-escalation-audit` (via `/ai-audit` Phase 1) under the same Boundary rule 1 Exception that authorises `Escalated?` updates.
