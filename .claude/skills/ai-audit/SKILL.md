---
name: ai-audit
description: "Two-phase instruction audit. Phase 1 (subagent) verifies every learnings.md `Escalated?` claim points to a rule that actually exists AND that every `Superseded by:` reference resolves to a real later entry or merged PR; fixes drift in either field. Phase 2 (main session) reads all instruction files (AGENTS.md, ai-docs/*, .claude/skills/**, .claude/agents/**, hooks in settings.json) against official Claude Code docs and proposes fixes for inconsistencies, dead references, format violations, and refactor candidates."
disable-model-invocation: true
argument-hint: "[scope: 'phase1' | 'phase2' | omit for both]"
allowed-tools: Bash(rg *) Bash(grep *) Bash(find *) Bash(realpath *) Bash(jq *) Bash(awk *) Bash(shellcheck *) Bash(wc *) Bash(basename *) Bash(git branch *) Bash(git status *) Bash(git checkout *) Bash(git rev-parse *) Bash(git diff *) Bash(git add *) Bash(git commit *)
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
4. If the subagent emitted any `🌱 Stale-validation` flags, surface them to the user as a **signal for `/improve`** — NOT an auto-fix. Each flag identifies a `Kind: validation` entry that is > 30 days old AND `Escalated? no` AND has ≥1 instruction-file commit since its validation date. The audit does not promote the pattern (promotion is a `self-improve` Carrot-pass Step 2b decision); it only surfaces the candidate so the user can decide whether to invoke `/improve` in a follow-up turn. Per AGENTS.md § Learning Log threshold line, a single `🌱` flag (alongside the ≥3-correction / ≥2-validation thresholds) is sufficient to justify an `/improve` invocation.

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

#### K. Skill-directory layout (SKILL.md + supporting files + scripts/)

Per the [Claude Code skill-directory pattern](https://code.claude.com/docs/en/skills#add-supporting-files), a skill directory may contain SKILL.md plus supporting files (reference docs loaded on demand, scripts the skill executes, examples Claude can read). Audit checks:

1. **Oversized SKILL.md (reference material embedded in workflow).** Identify any `SKILL.md` > 200 lines. For each, scan for *reference content* sections (format specs, parser rules, lookup tables, long checklists, embedded templates) — material that is referenced once or twice in the workflow but loaded into context on every invocation. Propose extraction to a supporting file. Severity `minor`.

2. **Multi-consumer supporting files belong in `ai-docs/templates/`.** When a supporting file is referenced from **>1 skill or agent**, propose moving it from the owning skill's directory to `ai-docs/templates/<file>.md` (per AGENTS.md *Agent Docs*). Single-consumer supporting files stay inside the owning skill directory. Cross-references then point at `ai-docs/templates/` directly instead of routing through another skill's body. Severity `minor`.

3. **Inline-script extraction candidates.** Identify `SKILL.md` sections containing **self-contained `bash` blocks** — a complete, executable recipe with at most one or two `<placeholder>` substitutions, NOT orchestration guidance that Claude reconstructs dynamically per call. For each, propose extraction to `.claude/skills/<skill>/scripts/<descriptive-name>.sh`, invoked via the canonical `${CLAUDE_SKILL_DIR}/scripts/<name>.sh <args>` pattern. After extraction, narrow the skill's `allowed-tools` from per-command patterns to a single `Bash(.claude/skills/<skill>/scripts/<name>.sh *)` entry. Severity `minor`.

   **Counter-rule.** Bash snippets that are *orchestration guidance* — every call has different placeholder values that the agent constructs — are NOT script-extraction candidates. Forcing them into scripts requires the agent to call a helper for guidance it can express inline. Skip those.

#### L. Learning-Log field coherence

When AGENTS.md § Learning Log's *Entry format* block lists a field maintained by `/improve` and `/ai-audit` (currently `Escalated?` and `Superseded by:`), verify each field is covered in **all four** mandatory locations:

| Location | Required content |
|---|---|
| AGENTS.md *Boundary rule 1 Exception* | Explicit authorization to edit the field in-place |
| AGENTS.md *Boundary rule 2 Exception* | Explicit authorization for the field's edits to coexist with instruction-file edits in the same `/improve` / `/ai-audit` turn |
| `.claude/agents/self-improve.md` Step 5 (Commit B backfill) | Workflow describing when and how `/improve` writes the field |
| `.claude/agents/learnings-escalation-audit.md` Steps 2/3/4 | Verification recipe + Category-1 drift fixes (including typo fixes within the field's value) |

A field added to the entry format without parallel coverage in all four targets → `major` finding (rules diverge across the surface; one of the two gates fails silently). The historical proof point: F1 in this PR — Boundary rule 2 Exception text lagged behind Boundary rule 1 Exception after the `Superseded by:` field landed, leaving Boundary rule 2 under-describing the contract.

**Declared-schema fields (no `/improve`-time mutation).** Some Entry-format fields are *declared* by AGENTS.md and *parsed* by the two agents but are **never** mutated by `/improve` or `/ai-audit` after the entry is written (currently: `Kind:`). For each such field, verify coverage in this analogous 4-location list:

| Location | Required content |
|---|---|
| AGENTS.md `## Learning Log` *Entry format* block | Declaration of the field name, allowed values, and default-when-omitted semantics |
| `.claude/agents/self-improve.md` Step 5 (Commit B backfill site) | Mention of the field where the workflow branches on its value (e.g., Step 5 / Step 6 routing on `Kind:`) |
| `.claude/agents/learnings-escalation-audit.md` Steps 2/3/4 | Parse-site usage of the field (verdict routing, sweep predicates) |
| `ai-docs/corrections-log.md` field glossary | Declaration mirror — same allowed-values list, same default-when-omitted note |

The Exception-body locations used by the `Escalated?` / `Superseded by:` rows do not literally apply since declared-schema fields have no `/improve`-time mutation. A declared-schema field added to the entry format without parallel coverage in all four locations above → `major` finding (the field's allowed values or default semantics drift across the surface; the two agents disagree on how to interpret it).

#### M. `agent-writing-style.md` conformance

`ai-docs/agent-writing-style.md` is the canonical style reference for fail-loud rules in instruction files. Checklist M sweeps the audited corpus for drift against the 7 Patterns + Anti-patterns table. **Audited corpus** (named inline; do NOT defer to Step 2.2's inventory which omits some of these): `AGENTS.md` + every `.claude/skills/**/SKILL.md` + every `.claude/agents/**.md` + `ai-docs/code-style.md` + `ai-docs/doc-convention.md` + `ai-docs/agent-writing-style.md` + `ai-docs/corrections-log.md`.

| # | Sub-check | Detection mechanism | Severity |
|---|---|---|---|
| 1 | **Pattern 1 (AXIOM blockquote)** — every `> **AXIOM —`-prefixed block must be followed by an action table within the same blockquote. | For each match of `rg -n '^> \*\*AXIOM —'`, read the next 30 lines of the blockquote (lines starting with `> `). If no `> \|` table row appears, flag the AXIOM line. | `major` |
| 2 | **Pattern 2 (fail-loud verbs)** — at most one bold-uppercase verb per paragraph (`**NEVER**` / `**MUST**` / `**MUST NOT**` / `**FORBIDDEN**` / `**STOP**` / `**REJECT**` / `**REMOVE**` / `**REPLACE**` / `**DELETE**` / `**ALWAYS**`). | `awk` splits the file on blank lines into paragraph chunks. For each chunk, count matches of `\*\*(NEVER\|MUST\|MUST NOT\|FORBIDDEN\|STOP\|REJECT\|REMOVE\|REPLACE\|DELETE\|ALWAYS)\*\*`. If count > 1, flag the paragraph (file:start-line). | `minor` |
| 3 | **Pattern 3 (action tables)** — the right column of every `\| If you see... \| Action \|` (or analogous) table must start with an action verb (imperative form), NOT prose narrative. | For each `\| If you see` table row, extract the right-column cell. Heuristic: starts with one of `Run`, `Apply`, `Stop`, `Add`, `Remove`, `Edit`, `Confirm`, `Bail`, `STOP`, `**NEVER**`, `**MUST**`, etc. OR a backtick-quoted command. If the cell starts with prose narrative (e.g., `"This is..."`, `"Usually..."`), flag the row. | `minor` |
| 4 | **Pattern 4 (explicit file lists, never globs)** — fail-loud lists that enumerate files must spell out each path; no glob-as-the-entire-list. | For each fail-loud block (paragraph containing a Pattern 2 verb), scan immediate `- ` or `* ` bullet list. If the entire list reduces to one or two globs (`.claude/**`, `**/*.rs`) with no specific paths, flag. (Per-bullet parenthetical globs like `.claude/skills/** (any file under this directory)` are acceptable.) | `major` |
| 5 | **Pattern 5 (numbered enumeration of triggers)** — OR/AND connector placement must be consistent across items. | For each numbered list (`^1\.`, `^2\.`) inside a fail-loud block, check that EITHER every non-last item ends in `, OR` (or `, AND`), OR no items carry the connector. Mixed placement (some items connector-suffixed, some not) → flag. | `nit` |
| 6 | **Pattern 6 (do/not examples for non-trivial rules)** — paragraphs that articulate a contrast between two shapes must demonstrate both shapes. | **Tightened heuristic** (per design-review note 4 on issue #369 PR — `not`/`NOT` alone are too noisy, firing on every "do not" / "must not" / "is not" paragraph): trigger iff the paragraph contains **BOTH** (a) a Pattern 2 fail-loud verb AND (b) one of the stronger contrast markers `instead` / `wrong` / `correct` / `forbidden`. Then check if a fenced code block OR a two-column `\| Do this \| NOT this \|` table follows within 8 lines. If both triggers fire AND no example follows, flag the paragraph. (Words `not` / `NOT` / `right` / `bad` / `good` are NOT in the trigger list — they produce false positives at unacceptable scale.) | `nit` |
| 7 | **Pattern 7 (compaction recovery callout)** — every callout-carrying skill must carry exactly one of the three locked variant-distinguishing phrases. | **Drive off the live grep, NOT the style guide table.** For each `.claude/skills/*/SKILL.md` whose body contains the literal string `Compaction recovery check`, run `rg -F` against the three variant-distinguishing phrases (verbatim from the archival source-of-truth at `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md`): Variant A = `"Locate the durable-state file via this skill's active-state probe"`; Variant B = `"If exactly one in-flight artefact exists"`; Variant C = `"Identify the **parent workflow**"`. If a callout-carrying skill contains zero or > 1 of the phrases → flag (likely invented 4th variant OR Variant-A/B/C drift). Also flag any callout-carrying skill not enumerated in the style guide Pattern 7 table at `ai-docs/agent-writing-style.md` lines 119–121 (style guide drift; the table should grow when a new skill onboards the callout). | `major` |
| 8 | **Anti-patterns table audit** — no row of the Anti-patterns table (`ai-docs/agent-writing-style.md § Anti-patterns, lines 157–167`) should appear verbatim as a positive rule anywhere in the audited corpus. | For each anti-pattern row's left-column text (e.g., `"Every paragraph in caps"`, `"AXIOM blockquote without action table"`), grep the audited corpus for matches NOT inside the style guide itself. Flag matches. | `major` |
| 9 | **Pattern 8 (file-size AXIOM conformance)** — every covered instruction file must stay below the 40,000-char hard cap; the 35,000-char band is an early warning. Rule-of-truth: `ai-docs/agent-writing-style.md § 8. 40k char-cap on instruction files`; source AXIOM: `AGENTS.md § Build & Test`. | Run the verbatim `wc -c` invocation below against the covered file set, apply the three-band severity table. See § *Sub-check 9 — file-size AXIOM conformance* below for the recipe + severity bands. | see body |
| 10 | **Style-guide audit coverage map** — every `## ` (level-2) heading in `ai-docs/agent-writing-style.md` must map to either an existing Checklist M sub-check or to the explicit exclusion list of non-rule-bearing meta-sections. Unmapped headings produce `nit` "audit coverage gap" findings. | Parse ATX `## ` headings from the **live** `ai-docs/agent-writing-style.md` (re-grep at audit time; do NOT use a baked-in snapshot). Apply the inline coverage map below. See § *Sub-check 10 — style-guide audit coverage map* below for the parser recipe + map + finding format. | `nit` |
| 11 | **Cross-shape verbs** — carrot-shaped rules (entries in a `## Patterns` section) MUST NOT use stick verbs; stick-shaped rules (AGENTS.md AXIOM blockquotes or fail-loud bodies) MUST NOT use carrot verbs. The verb asymmetry IS the asymmetric-promotion contract — a wrong-shape verb either underweights a real obligation or locks in a brittle default as a hard rule. | (a) **Carrot block with stick verb:** for each `### N. <Name>` entry under a `## Patterns` section in the audited corpus, grep the entry body for `**MUST**` / `**NEVER**` / `**MUST NOT**` / `**FORBIDDEN**` — any match flags the entry. (b) **Stick block with carrot verb:** for each `> **AXIOM —` blockquote (and its action-table body) outside `## Patterns` sections, grep for `Default to` / `Prefer` — any match flags the blockquote. Both directions flagged at the same severity. The detection cross-checks the Kind shape (Patterns block ↔ Kind: validation entry; AXIOM block ↔ Kind: correction entry) against the verb pattern. | `major` |

After running Checklist M, surface findings using the same severity-driven apply-or-ask pattern as Checklists A–L (Step 2.5). Pattern 6 noise-management fallback: if AC5's demonstrator run shows > 50% false-positive rate on Pattern 6 findings, record the rate and tighten the heuristic in a follow-up `/improve` cycle (the heuristic itself is encoded here, not in a separate config file — design choice to keep the audit self-contained).

##### Sub-check 9 — file-size AXIOM conformance

Detection mechanism. Run this verbatim invocation:

```bash
wc -c AGENTS.md CLAUDE.md .claude/skills/*/SKILL.md .claude/agents/*.md \
      ai-docs/code-style.md ai-docs/doc-convention.md ai-docs/context.md \
      ai-docs/agent-writing-style.md ai-docs/corrections-log.md
```

Apply the three-band severity table to every reported size:

| Reported size (chars) | Finding | Severity |
|---|---|---|
| `< 35,000` | none | — |
| `35,000–39,999` | `<path>: <count> chars — early warning (≥ 35,000)` | `minor` |
| `≥ 40,000` | `<path>: <count> chars — AXIOM violation (≥ 40,000)` | `blocker` |

The covered file set is enumerated verbatim from `AGENTS.md § Build & Test` (the source-of-truth AXIOM) and restated in `ai-docs/agent-writing-style.md § 8. 40k char-cap on instruction files`. A future change to the covered file set MUST update Sub-check 9 in the same PR per the Propagation Rule.

Note: the shell-glob form (`.claude/skills/*/SKILL.md`, `.claude/agents/*.md`) is acceptable here because Pattern 4's explicit-path requirement applies to the *fail-loud bullet list* in Pattern 8 (so static readers see the covered set), not to the shell command that consumes the set.

This sub-check is the audit-side back-stop. The mechanical pre-commit gate is planned in #383; until that lands, Sub-check 9 fires per-`/ai-audit`-run.

##### Sub-check 10 — style-guide audit coverage map

Detection mechanism. The audit reads `ai-docs/agent-writing-style.md` at audit time and parses every ATX `## ` heading.

Parser strictness rules:

1. Match **ATX-style level-2 headings only** — exactly two `#` characters followed by exactly one space, then heading text.
2. **Skip lines inside fenced code blocks.** Track ` ``` ` and `~~~` fence state; a `## ` line inside an open fence is NOT a heading.
3. **Case-sensitive match.** `## Patterns` ≠ `## patterns`.
4. **Trim** leading/trailing whitespace from heading text before lookup.

Inline coverage map (live as of this commit; re-validate at audit time by re-running `grep -n '^## ' ai-docs/agent-writing-style.md` and reconciling against this map):

| `## ` heading | Maps to | Outcome |
|---|---|---|
| `## Patterns` | sub-checks 1–7 (audits the shape of every entry under this heading, including the new Pattern 8 via Patterns 1–4 self-conformance) | no finding |
| `## Anti-patterns` | sub-check 8 | no finding |
| `## Writing checklist` | excluded — meta-section (reader checklist, not a rule shape) | no finding |
| `## Citation in PRs` | excluded — meta-section (PR-author convention, not a rule shape) | no finding |
| `## Enforcement` | excluded — meta-section (cross-references the audit itself) | no finding |
| `## Propagation rule for new patterns` | excluded — meta-section (fan-out procedure, not a rule shape) | no finding |
| `## Out of scope` | excluded — meta-section (negative-space scoping, not a rule shape) | no finding |

Unmatched-heading rule. For every parsed `## ` heading NOT in the coverage map above, emit:

- **Finding text:** `audit coverage gap: § <heading>`
- **Proposed action:** `add sub-check N+1 to /ai-audit Checklist M` (where N is the current max sub-check number)
- **Severity:** `nit`

When a future PR adds a new `## ` heading to `agent-writing-style.md`, Sub-check 10 fires at the next `/ai-audit` run with the gap; the operator either adds a corresponding sub-check or extends the exclusion list in the same follow-up.

#### N. Bidirectional `## Patterns` ↔ `Kind: validation` coherence

The carrot-side analog of Checklist C (dead references). Every promoted-from-validation carrot must round-trip in both directions:

**Forward direction.** Every `### N. <Name>` entry under a `## Patterns` section in the audited corpus (`.claude/skills/**/SKILL.md`, `.claude/agents/**.md`, `AGENTS.md`) whose **body uses carrot verbs** (`Default to` / `Prefer`) MUST back-link to at least one `Kind: validation` entry in `ai-docs/learnings.md`. Detection recipe: for each `### N. <Name>` block, grep its body for `Default to` or `Prefer`; if found, also grep for a `learnings.md` back-link (path + date-slug citation) within the same block. Carrot-verb present AND no back-link → flag.

**Forward-sweep carrier-vs-template exemption.** Entries WITHOUT carrot verbs (template scaffolding, non-promoted prose, structural placeholders) are out of scope for the forward sweep — the audit greps for carrot-verb presence within each `### N. <Name>` block **before** requiring a back-link. The named exempt source is `ai-docs/agent-writing-style.md § Patterns` (template source, not a promoted-from-validation carrier). Other `## Patterns` sections that grow in future PRs are subject to the same carrot-verb-presence filter — no further per-file exemptions.

**Reverse direction.** Every `Kind: validation` entry in `ai-docs/learnings.md` whose `Escalated?` ≠ `no` MUST have a corresponding `## Patterns` block in each named target file (skill / agent / AGENTS.md). Detection recipe: parse each `Kind: validation` entry's `Escalated?` line; for each comma-separated target value, confirm the named file contains a `## Patterns` block AND that block contains an entry back-linking to this validation entry. Predicate gate: entries with `Escalated? no` are NOT subject to reverse-direction enforcement — only entries the operator has promoted (`Escalated? ≠ no`) require a paired `## Patterns` block.

Multi-target reverse direction. The `Escalated?` field may name multiple comma-separated targets (e.g., `skill:context-reset, AGENTS.md`). The reverse sweep iterates each value independently — a validation entry escalated to two targets must have a `## Patterns` block in BOTH files; missing in either flags.

| Direction | Trigger | Action |
|---|---|---|
| Forward | `### N. <Name>` block in `## Patterns` uses `Default to` / `Prefer` AND no `learnings.md` back-link in the same block | flag (severity `major`) |
| Forward (exemption) | `### N. <Name>` block has no carrot verb in its body | no flag (carrier-vs-template exemption) |
| Forward (named exempt source) | The audited file is `ai-docs/agent-writing-style.md` | no flag (template source) |
| Reverse | `Kind: validation` entry with `Escalated? ≠ no` AND named target file lacks a `## Patterns` block OR lacks a back-linking entry | flag (severity `major`) |
| Reverse (predicate gate) | `Kind: validation` entry with `Escalated? no` | no flag (not promoted; pattern block not required) |

Severity `major` — dead-reference class. The bidirectional shape mirrors Checklist C: every reference resolves AND every target has a back-reference.

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

Apply approved fixes via `Edit` / `Write`. Update `ai-docs/learnings.md` with a new entry per AGENTS.md "Learning Log" format **only if** the audit revealed a *new* class of mistake worth tracking — do not log routine cleanup.

### Step 2.6: Verify

After edits:

1. Re-run any `find`/`grep` checks from Step 2.3 that detected violations — confirm zero remaining.
2. If hooks were edited, eyeball the `.claude/settings.json` JSON validity: `jq . .claude/settings.json`.
3. If agent or skill files were edited, confirm their frontmatter still parses by reading the file back.
4. **Cross-reference re-verification (anchor-aware).** For every relative link the audit touched in any `.claude/agents/*.md` or `.claude/skills/**/SKILL.md`, confirm the target file exists AND the anchor (if present) matches a heading slug. Use this anchor-aware check rather than naive `realpath -m` (which mistakes `#anchor` for part of the path):

   ```bash
   for f in <changed-files>; do
     grep -oE '\(\.\./[./]*[^)#]+(#[^)]*)?\)' "$f" | sort -u | while read ref; do
       path_with_anchor=$(echo "$ref" | tr -d '()')
       path=${path_with_anchor%%#*}
       anchor=${path_with_anchor#*#}; [ "$anchor" = "$path_with_anchor" ] && anchor=""
       src_dir=$(dirname "$f")
       abs=$(realpath -m "$src_dir/$path")
       [ -e "$abs" ] || { echo "FILE MISSING: $f -> $path"; continue; }
       [ -z "$anchor" ] && continue
       # heading-slug match: lowercase, strip non-alnum-non-hyphen, spaces->hyphens
       awk '/^#{1,6}\s/{line=$0; gsub(/^#+\s+/,"",line); line=tolower(line); gsub(/[^a-z0-9 -]/,"",line); gsub(/ /,"-",line); gsub(/-+/,"-",line); sub(/^-+/,"",line); sub(/-+$/,"",line); print line}' "$abs" | grep -Fx "$anchor" >/dev/null || echo "ANCHOR MISSING: $f -> $path#$anchor"
     done
   done
   ```

5. **Script verification (when checklist K extracted any `scripts/*.sh`).** For every script the audit added or modified:
   - **`shellcheck`** the script if `shellcheck` is on `$PATH`; flag warnings/errors as `minor` post-extraction findings.
   - Confirm executable bit (`-rwx------` or broader). If `chmod +x` was forgotten, the script will fail at invocation time with `Permission denied`.
   - **Smoke-test the documented no-op path** when feasible: most extracted scripts have a documented exit-0 case (missing input, no-op match). Invoke against that input and verify expected exit code + status message (e.g., `cleanup-progress.sh nonexistent-branch-xyz` should print `pr-merged: no merged PR found for nonexistent-branch-xyz` and exit 0).

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
| Script extraction (K) | `shellcheck` clean, `+x` bit set, no-op smoke-test passes? |
| Commit | `jq .` passes on settings.json? frontmatter still valid on edited skills/agents? cross-references re-verified anchor-aware? |

## Anti-patterns

- Do **not** rewrite `learnings.md` history — it is append-only. Phase 1 may only correct the `Escalated?` field of an existing entry or add a *new* corrective entry; never delete or rephrase past entries.
- Do **not** invent rules. The audit finds compliance gaps in *existing* rules; new rules go through `/improve`.
- Do **not** skip the WebFetch step in Phase 2. The official docs are the source of truth for hook/skill/agent shapes — relying on memory is the failure mode this skill exists to prevent.
- Do **not** auto-resolve a blocker without surfacing it. Severity is a signal that human judgment is needed.
- Do **not** edit `.claude/settings.local.json` from this skill — it is user-local.
