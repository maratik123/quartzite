# Harness taxonomy unify — Tools / Subagents / Skills / Hooks

**Source:** user description
**Date:** 2026-05-23
**Tracked in:** #547

## Scope

Unify Claude-Code-harness terminology across this project's instruction-file corpus by:

1. **Adopting the canonical naming from [`ai-docs/claude-tools-hierarchy.md`](../claude-tools-hierarchy.md) (already in-tree, 9,505 chars) as the single source of truth** for four naming axes:
   - **Tool** (capitalised, code-style for the name) — `Read`, `Write`, `Edit`, `Bash`, `Agent`, `Skill`, `AskUserQuestion`, `ScheduleWakeup`, `ToolSearch`, `Grep`, `Glob`, and the deferred set (`CronCreate`, `EnterPlanMode`, `LSP`, `Monitor`, `NotebookEdit`, `PushNotification`, `RemoteTrigger`, `TaskCreate`/…/`TaskUpdate`, `WebFetch`, `WebSearch`).
   - **Subagent** — the thing spawned via the `Agent` Tool (`Explore`, `general-purpose`, `Plan`, `design`, `design-review`, `learnings-escalation-audit`, `review-findings`, `self-improve`, `self-review`, `spec-writer`, `triage-runner`, …). NOT "agent" for the spawned thing.
   - **Skill** — the thing invoked via the `Skill` Tool or typed `/<name>` (`interview`, `task`, `bugfix`, `code-review`, …). NOT "slash command" as a generic term.
   - **Hook** — references spell the event name (`SessionStart` / `PreToolUse` / `PostToolUse`) and the matcher (`Bash` / `Write|Edit`) where relevant.

2. **Sweeping every in-scope instruction file** for terminology that diverges from these axes; rewriting in place. Prose-only sweep — on-disk filenames stay unchanged for the naming-axis sweep. (Clash-rename incidental fix, below, is the explicit exception.)

3. **De-duplicating shared taxonomy explanations** by pointing siblings at `ai-docs/claude-tools-hierarchy.md` instead of re-explaining the hierarchy locally. Each touched file keeps a one-line on-demand pointer to `claude-tools-hierarchy.md` in place of any verbose paragraph it currently carries.

4. **Stabilising the anchor surface of `claude-tools-hierarchy.md`** so cross-file deep-links survive future edits AND **documenting the embedded `verify` Skill in §3a**:
   - Add an explicit `## Stable anchors` section to `claude-tools-hierarchy.md` listing every anchor other files deep-link to (initial set: `#1-tools`, `#2-subagents`, `#3-skills`, `#4-hooks`, `#mental-model`, `#stable-anchors`).
   - Beside each load-bearing heading, add an HTML comment `<!-- stable-anchor: #x -->` (matching the GitHub-flavoured-markdown slug for that heading) so editors who touch the file see the marker before renaming the heading.
   - Future PRs that edit `claude-tools-hierarchy.md` MUST preserve every listed anchor OR update every inbound deep-link in the same PR (this becomes a Propagation-Rule row — see AC10).
   - **§3a embedded-Skills table** gains a NEW row for the embedded `verify` Skill, with the upstream description from the live system-reminder Skills list (verbatim per AC21). §3a embedded-Skills count goes 11 → 12; any preamble citing the count is updated. This addition is unconditional and lands in the same PR as the §3c `verify` → `verify-change` rename (AC18) — see *Files touched* for the co-location.

5. **Codifying a clash-rename policy** for project-defined names vs. embedded (Anthropic-shipped or marketplace-plugin) names across all four taxonomy axes (Tool / Subagent / Skill / Hook):
   - A NEW AXIOM lands in `AGENTS.md` near the existing "Do not refer to a skill as an 'agent' or vice versa" sentence. The AXIOM forbids project-defined names from colliding with embedded names; on detected clash, the **project-defined** name is the rename candidate, never the embedded one.
   - `/ai-audit` gains a new checklist item (**Checklist O** — see AC16 / AC17) that scans every project-defined Tool / Subagent / Skill / Hook name against the embedded inventory enumerated in `ai-docs/claude-tools-hierarchy.md` §§1a + 1b + 2a + 3a + 3b. Any match → fail-loud finding.
   - The first incidental fix this PR applies under the new policy: rename the project's `verify` Skill (today `.claude/skills/verify/SKILL.md`) to a non-clashing name. **Target name:** `verify-change` (rationale in *Key decisions*). Inbound references audited & updated in this PR (none currently in `AGENTS.md` / `CLAUDE.md`; the four hits in `ai-docs/templates/progress-format.md`, `.claude/agents/self-review.md`, `ai-docs/plans/INDEX.md` archived-spec mention, and `ai-docs/claude-tools-hierarchy.md` §3c — see *Files touched*). `ai-docs/learnings.md` entries that mention the old name are NOT rewritten (Boundary rule 1, APPEND-ONLY).

### In-scope file corpus

Every file Claude loads as instructions per the AGENTS.md 40k-char AXIOM, plus the canonical taxonomy doc itself:

- `AGENTS.md`
- `CLAUDE.md`
- `.claude/skills/**/*.md` (every SKILL.md under the 15 project skills; supporting `reference.md` files inherited via `Read`)
- `.claude/agents/**.md` (all 8 project subagent definitions — **prose inside the files only; filenames unchanged for the naming-axis sweep**)
- `.claude/rules/*.md` (currently just `ast-index.md`)
- `ai-docs/code-style.md`, `ai-docs/doc-convention.md`, `ai-docs/context.md`, `ai-docs/agent-writing-style.md`, `ai-docs/corrections-log.md`
- `ai-docs/claude-tools-hierarchy.md` (anchor stabilisation + §3a + §3c row updates after the `verify` rename + minor edits if reference doc itself drifts from the four axes)

### Files touched by the clash-rename incidental fix (`verify` → `verify-change`)

- `.claude/skills/verify/` directory → `.claude/skills/verify-change/`
- `.claude/skills/verify-change/SKILL.md` (frontmatter `name: verify` → `name: verify-change`)
- `ai-docs/claude-tools-hierarchy.md` — TWO co-located edits in §3a + §3c:
  - **§3a (Embedded Skills table):** add a NEW row for the embedded `verify` Skill with the upstream description from the live system-reminder Skills list (quoted verbatim in AC21). Today §3a lists 11 embedded Skills (code-review, run, review, security-review, init, update-config, keybindings-help, fewer-permission-prompts, loop, schedule, claude-api); after this PR it lists 12. Any introductory sentence in §3a that cites the count (e.g. "11 embedded skills") is updated to "12" in the same edit. No "project override of embedded `verify`" annotation is added on the §3a side — once the project's Skill is renamed to `verify-change` (AC18), the clash is gone and §3a stands alone.
  - **§3c (Project Skills) row:** update the row from `verify` → `verify-change` AND remove the "(note: this project also overrides it — see §3c)" parenthetical that the §3a row currently lacks anyway, plus the §3c-side "(note: this project's `verify` overrides the embedded `verify` Skill — see §3a)" or equivalent parenthetical (live wording verified at design time). After the rename, the project Skill name no longer clashes with the embedded name, so no cross-reference parenthetical is required in either direction.
- `.claude/agents/self-review.md` — the `/verify` mention in the progress-file format paragraph
- `ai-docs/templates/progress-format.md` — `/verify` exemption lines in body + § Exemptions
- `ai-docs/plans/INDEX.md` — historical mention in the sonnet-skill-reentry-protocol row (acceptable to update since INDEX.md is not append-only; design phase confirms)

Inbound references explicitly NOT rewritten:

- `ai-docs/learnings.md` — APPEND-ONLY (Boundary rule 1).
- `ai-docs/plans/done/**` — archived plans, historical record.

### Pre-resolved by the user (do NOT re-ask)

Round-1 pre-confirmed:

- All four naming axes (Tool / Subagent / Skill / Hook) apply.
- "agent" lowercase is permitted only inside compound brand-style proper nouns ("Anthropic Agent SDK", "Claude Agent SDK") — every other lowercase "agent" referring to the spawned-thing concept becomes "subagent".
- Per-skill voice / dual-model-readability AXIOMs / fail-loud table formatting / sync-group distinction MUST survive the rewrite.
- AGENTS.md's "skill vs agent distinction matters for spawning" sentence MUST be preserved verbatim at the AGENTS.md site (not moved).

Round-2 confirmed (3 answers):

- **Prose only** — on-disk filenames under `.claude/agents/*.md` stay unchanged; no `.claude/settings.json` edits for the naming-axis sweep; no `Agent` Tool dispatch risk; no Propagation-Rule row updates for filename renames. Sweep human-readable prose only. (Round-3 clash-rename incidental fix is a NAMED exception, not a general loosening.)
- **Consistency only** — net char-count delta is reported in the PR body for transparency but does NOT gate AC8; AC9 (no in-scope file crosses 40k AND no in-scope file currently in the 35k–40k warning band grows) is the load-bearing size guard. Consistency + de-dup is the primary success criterion.
- **Freeze + mark** — `claude-tools-hierarchy.md` gains an explicit `## Stable anchors` section + per-heading `<!-- stable-anchor: #x -->` comments; this PR seeds the contract. Future edits MUST preserve the listed anchors OR update every inbound deep-link in the same PR.

Round-3 pre-confirmed (clash-rename scope):

- Option **(b)** — forward-looking policy in AGENTS.md AXIOM + `/ai-audit` enforcement; first incidental fix is the `verify` Skill rename.
- On clash, the project-defined name is the rename candidate; the embedded name is never renamed.
- Policy applies to all four axes (Tool / Subagent / Skill / Hook).

Round-2 folded-in defaults (O1, O2 resolved — see *Key decisions* and *Open questions* below).

## Out of scope

- Editing entries in `ai-docs/learnings.md` to rename historical "agent" → "subagent" OR `verify` → `verify-change` — Boundary rule 1 forbids it (file is append-only).
- Renaming on-disk filenames under `.claude/agents/` (e.g. `.claude/agents/design.md` → `.claude/agents/design.subagent.md`) — explicitly out per round-1 answer ("Prose only").
- Renaming the `.claude/agents/` directory itself — would break `Agent` Tool dispatch.
- Renaming the YAML `name:` field of subagent frontmatter (e.g. `name: spec-writer`) — that field is consumed by the harness's `Agent` Tool dispatch and is not human-facing terminology. (Exception: the `verify` Skill rename DOES change its frontmatter `name:` field from `verify` to `verify-change`; this is the harness-dispatch name and is the rename target by design.)
- Renaming `Bash` / `Grep` / `Glob` Tool invocations themselves — only the prose **referring** to them.
- Renaming the `## Agent Docs` heading in AGENTS.md or `ai-docs/agent-docs-index.md` (these refer to documentation pages, not to spawned subagents — `agent` is a legitimate compound-noun adjective there).
- Editing the project banner `# Rust OSS Agent Rules` in AGENTS.md line 1 — proper-noun compound; project title.
- Renaming `ai-docs/agent-writing-style.md` and `ai-docs/agent-docs-index.md` filenames — broad Propagation-Rule blast radius; defer to a separate refactor if ever desired.
- Editing `ai-docs/plans/done/**` (archived specs/designs are historical record; same Boundary-rule-style intent as `learnings.md`).
- Edits to `ROADMAP.md`, `ai-docs/deferred/_inbox.md` — orchestration-output files, not human-authored instruction prose. `ai-docs/plans/INDEX.md` updates only for the `verify` → `verify-change` rename row (see *Files touched*); no taxonomy sweep on INDEX.md prose.
- Moving AGENTS.md's "skill vs agent distinction matters for spawning" sentence into `claude-tools-hierarchy.md` — rule fires per-invocation, moving it costs a hop. Stays verbatim in AGENTS.md.
- Renaming any OTHER currently-clashing project-defined name beyond `verify`. Per Round 3 the AXIOM is forward-looking; the `verify` Skill is the only incidental fix this PR applies. If `/ai-audit` Checklist O surfaces additional clashes after merge, they get their own task.

## Deferred

- Renaming on-disk filenames under `.claude/agents/` to `*.subagent.md` — filename rename has broad blast radius (Propagation Rule sync-group references, `.claude/settings.json` references, plugin metadata). | separate issue needed: yes (only if a future need emerges) |
- Adding a CI lint that enforces the four axes (`grep -E '(the (Bash|Read|…) tool|slash command|sub-agent)\b'` returning empty in PR diff) — mechanical enforcement is a force-multiplier but distinct concern from the one-shot rewrite. | separate issue needed: yes |
- Adding a CI lint that enforces clash-rename policy (Checklist O mechanically, in pre-commit) — `/ai-audit` runs on demand; a pre-commit guard catches it earlier. | separate issue needed: yes (after at least one Checklist O false-positive / true-positive cycle informs the regex) |
- Renaming `ai-docs/agent-writing-style.md` → `ai-docs/subagent-writing-style.md` — the file documents writing-style rules for both Skills and Subagents and the filename is referenced from many places; rename has Propagation-Rule blast radius. | separate issue needed: only if future maintenance burden justifies it |

## Key decisions

| Question | Decision |
|---|---|
| Canonical taxonomy doc | `ai-docs/claude-tools-hierarchy.md` (already in-tree at 9,505 chars). All other in-scope files link to it via on-demand pointer; none re-explain. |
| Naming axes — count | 4 (Tool, Subagent, Skill, Hook) — all in scope this PR. |
| "Tool" — code-style or prose? | Code-style backticked name (e.g. ``the `Bash` Tool``) — never lowercased prose "the bash tool". |
| "Subagent" — one word | One word, no hyphen ("subagent" not "sub-agent"). One known `sub-agent` occurrence (`.claude/skills/project-review/SKILL.md:97`) gets normalised. |
| "Slash command" — generic ban | Banned as a generic term for Skills. Use "the `interview` Skill" / "the `/interview` Skill" / "Skill `interview`". The literal token "slash command" survives only in deliberate phrases like "typed `/<name>`" where the syntactic point matters. |
| Hook references | Always carry the event name (`SessionStart` / `PreToolUse` / `PostToolUse`) AND the matcher (`Bash` / `Write\|Edit`) where applicable. |
| AGENTS.md "skill vs agent" sentence | Keep verbatim at the AGENTS.md site — it is the load-bearing pre-resolved rule the orchestrator cites. Not mirrored into `claude-tools-hierarchy.md` (round-1 "Consistency only" answer removes the size pressure that would have motivated the move; rule fires per-invocation; cheaper to keep in place). |
| File rename on disk (naming-axis sweep) | NO — prose-only sweep per round-1 answer. Filenames under `.claude/agents/*.md`, the `.claude/agents/` directory, and `ai-docs/agent-*.md` filenames stay unchanged. (Round-3 clash-rename `verify` → `verify-change` is a NAMED exception — see clash-rename row below.) |
| Size-win acceptance threshold | Consistency-only per round-1 answer. AC8 passes for ANY net delta as long as AC9's hard limits hold (no file crosses 40k; no warning-band file grows). Net delta reported in PR body for transparency. |
| `claude-tools-hierarchy.md` anchor stability | Freeze + mark per round-1 answer. Add `## Stable anchors` section listing `#1-tools` / `#2-subagents` / `#3-skills` / `#4-hooks` / `#mental-model` / `#stable-anchors`. Add `<!-- stable-anchor: #x -->` comment beside each load-bearing heading. Future edits MUST preserve them or update every inbound deep-link in the same PR. |
| Propagation Rule new sync-group row (resolves O1) | YES — add one new row reading: `\| Any edit that changes a Tool / Subagent / Skill / Hook contract OR renames a stable anchor in claude-tools-hierarchy.md \| Update ai-docs/claude-tools-hierarchy.md in the same PR (contract changes) AND every inbound deep-link to renamed anchors (anchor renames). \|`. The anchor-rename clause is the propagation pair to the "Freeze + mark" answer — without it, the marker comments would have no enforcement loop. The row's "Tool / Subagent / Skill / Hook contract" clause ALSO covers the clash-rename policy (any time a NEW embedded name enters the §§1a/1b/2a/3a/3b registry that may now clash with an existing project name). |
| "agent" lowercase carve-outs (resolves O2) | Whitelist (preserved as-is, NOT rewritten to "subagent"): (1) `# Rust OSS Agent Rules` AGENTS.md banner; (2) `## Agent Docs` heading in AGENTS.md + `ai-docs/agent-docs-index.md`; (3) the filenames `ai-docs/agent-docs-index.md` and `ai-docs/agent-writing-style.md` themselves and prose references to them; (4) compound proper nouns `Claude Agent SDK` / `Anthropic Agent SDK` and any equivalent external-product compound noun; (5) every `ai-docs/learnings.md` entry, historical or new (Boundary rule 1 makes it out-of-scope mechanically — listed here for completeness). Every other lowercase "agent" referring to the spawned-thing concept becomes "subagent". |
| Clash-rename policy AXIOM site | NEW AXIOM lands in `AGENTS.md` near the existing "Do not refer to a skill as an 'agent' or vice versa" sentence (currently the last line of the `## Propagation Rule` section / immediately before `## Communication`). Forward-looking — applies to all future project-defined names across the four axes. Spec-writer drafts the wording (see AC16); final wording is the design phase's call. **Draft AXIOM text:** ``> **AXIOM — Project-defined Tool / Subagent / Skill / Hook names MUST NOT clash with embedded (Anthropic-shipped or marketplace-plugin) names enumerated in `ai-docs/claude-tools-hierarchy.md` §§1a + 1b + 2a + 3a + 3b. On detected clash, the project-defined name is the rename candidate; the embedded name is never renamed.`` followed by a three-row fail-loud table per `agent-writing-style.md` (clash on add → choose non-clashing name; clash discovered post-add → rename project side same PR; new embedded name arrives that clashes with existing project name → project rename queued as separate task). |
| `/ai-audit` checklist letter for clash-scan | **Checklist O** — next available letter after Checklist N (last currently in `.claude/skills/ai-audit/reference.md` per `grep -n "^## Checklist" reference.md` returning A..N). Adding a new top-level letter rather than folding into Checklist K because (a) clash-detection is a different concern from skill-directory-layout (K), and (b) a top-level letter makes the corpus easier to grep for false-positive triage. Checklist O reads: scan every project-defined Tool / Subagent / Skill / Hook name (enumerated from `.claude/agents/*.md` frontmatter + `.claude/skills/*/SKILL.md` frontmatter + `.claude/settings.json` Hook event names) against the embedded inventory in `claude-tools-hierarchy.md` §§1a + 1b + 2a + 3a + 3b; any match → `major` finding with rename recommendation. |
| `verify` Skill rename target | **`verify-change`** — semantically closest to what the Skill does (`cargo test $ARGUMENTS` to verify a change passes tests). Rejected alternatives: `verify-impl` (less precise — the Skill verifies any change, not just impl code); `verify-feature` (too narrow — also used for refactors, tests, docs); `quartzite-verify` (no other project skill carries the `quartzite-` prefix; introduces inconsistency). The choice is reversible via Design Amendment if the design phase surfaces a stronger candidate. |
| Inbound-reference update strategy for `verify` rename | Mechanical: `grep -rn '\bverify\b' .claude/ ai-docs/` across the in-scope corpus, filter to references that refer to the project Skill (not `cargo test --verify` or `git commit --no-verify` or `verify derive-free`), update each. The 4 identified hits live in `.claude/agents/self-review.md`, `ai-docs/templates/progress-format.md`, `ai-docs/claude-tools-hierarchy.md` §3c, and `ai-docs/plans/INDEX.md`. AGENTS.md and CLAUDE.md have no Skill-`verify` references (verified by grep — the only `verify` tokens are `cargo build` verification prose and `git commit --no-verify`). |
| §3a embedded-`verify` row addition (round-4 hard scope) | Add a NEW row to `ai-docs/claude-tools-hierarchy.md` §3a (Embedded Skills table) for the embedded `verify` Skill. The Purpose column carries the upstream description verbatim from the live system-reminder Skills list: *"Verify that a code change actually does what it's supposed to by running the app and observing behavior. Use when asked to verify a PR, confirm a fix works, test a change manually, check that a feature works, or validate local changes before pushing."* The §3a embedded-Skills count goes 11 → 12; any in-tree text citing the count is updated. AC21 enforces this unconditionally. Brevity-for-line-wrap is a design-phase call (close paraphrase acceptable if and only if the table layout demands it). |
| §3a / §3c cross-reference removal after rename | Once AC18's rename lands (`verify` → `verify-change`), the §3c row drops any "this project also overrides embedded `verify` — see §3a" parenthetical AND the §3a row has no need for a "project override of embedded `verify`" annotation (because there is no longer a project Skill named `verify`). Both sides stand independent. AC21 + AC18 jointly carry this. |

## Technical constraints

- **40k-char AXIOM (AGENTS.md § Build & Test):** No in-scope file may cross 40k chars. Files currently between 35k–40k chars (AGENTS.md at 36,808; `.claude/agents/triage-runner.md` at 37,916) MUST NOT grow — they may shrink or hold. AC9 makes this a hard pass gate. The new AGENTS.md AXIOM (clash-rename policy) MUST be terse enough to keep AGENTS.md ≤ 36,808.
- **Boundary rule 1 (AGENTS.md § Learning Log):** `ai-docs/learnings.md` is APPEND-ONLY — even though it contains many lowercase "agent" references AND a future entry may mention the old `verify` name, NONE are rewritten. (`learnings.md` is therefore not in the in-scope corpus above.)
- **Boundary rule 2 (AGENTS.md § Learning Log):** Writing instruction-file edits in this PR does NOT authorise a `learnings.md` entry in the same turn for those edits. Standard workflow.
- **Propagation Rule:** Every sync-group row in `AGENTS.md § Propagation Rule` that mentions a touched file MUST receive co-edits in the same PR. Specifically — the Review group, Interview group, Triage group, Task/Design group, Spec-Amendment group, Learning-Log group, Size-exemption-index group, and the `.claude/rules/<file>.md` row are all in scope. One NEW row gets added (see Key decisions / AC10). The new row's "Tool / Subagent / Skill / Hook contract" clause is what triggers on a future embedded-vs-project name clash.
- **Instruction-file refactor, not Rust API change:** AGENTS.md § *API Stability* governs Rust source code, not prose terminology. Renames here are clean prose substitutions — no parallel alias terms, no transitional dual spellings, no "rename in next PR" placeholders. The new spelling replaces the old in one PR. The `verify` → `verify-change` Skill folder rename follows the same clean-break model.
- **Dual-model readability (AGENTS.md § Build & Test → 40k AXIOM; `ai-docs/agent-writing-style.md`):** Every fail-loud pattern (warning-band table, AXIOM block, three-row decision table, code-style filenames in monospace) MUST be preserved structurally; only the **terminology** inside is renamed. The new clash-rename AXIOM in AGENTS.md MUST follow the same fail-loud pattern (three-row table per `agent-writing-style.md` § *Patterns*).
- **Per-skill voice:** Each skill / subagent file's tone (imperative, table-of-three-options, "STOP" hard rules) is preserved; only nouns shift.
- **On-demand pointer pattern:** When extracting taxonomy explanation to `claude-tools-hierarchy.md`, the residual pointer in the sibling file uses the existing project convention (e.g. ``> Read [`ai-docs/claude-tools-hierarchy.md`](ai-docs/claude-tools-hierarchy.md) on demand for the canonical Tools / Subagents / Skills / Hooks taxonomy.``).
- **Anchor stability marker pattern:** In `claude-tools-hierarchy.md`, each load-bearing heading gains a sibling line `<!-- stable-anchor: #<slug> -->` immediately above (or beside) the heading, where `<slug>` matches the GitHub-flavoured-markdown slug. The `## Stable anchors` section lists every such slug as the single index editors consult before renaming a heading.
- **`grep` sweep is mandatory:** After every per-file edit, the implementer runs the substring blacklist (see *Acceptance Criteria* below) against the file to verify residual hits are zero.
- **Clash-rename mechanical scan:** The `/ai-audit` Checklist O scan MUST be implementable as a deterministic command — e.g. `comm -12 <(sort project-names.txt) <(sort embedded-names.txt)` returning empty. Design phase decides the exact recipe.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Every in-scope file uses **Tool** (capitalised, backticked name) when referring to harness Tools. Substring blacklist `grep -niE '\bthe (bash\|read\|write\|edit\|agent\|skill\|grep\|glob) tool\b' <in-scope-files>` returns empty across all in-scope files. |
| AC2 | Every in-scope file uses **Subagent** (one word, no hyphen) for the thing spawned via the `Agent` Tool. `grep -nE '\bsub-agent\b' <in-scope-files>` returns empty. |
| AC3 | "agent" lowercase referring to the spawned-thing concept is rewritten to "subagent" in every in-scope file, except for the carve-outs documented in *Key decisions* (banner / `## Agent Docs` heading / filename references / `Claude Agent SDK` / `Anthropic Agent SDK` / `ai-docs/learnings.md`). |
| AC4 | Every in-scope file uses **Skill** (capitalised, backticked or `/`-prefixed name) for the thing invoked via the `Skill` Tool or `/<name>`. `grep -niE '\bslash command\b' <in-scope-files>` returns empty except where the literal token survives in deliberate phrases about syntax (e.g. "typed `/<name>`"). |
| AC5 | Every Hook reference in the in-scope corpus spells the event name (`SessionStart` / `PreToolUse` / `PostToolUse`) and, where applicable, the matcher (`Bash` / `Write\|Edit`). No in-scope file refers to "the X hook" without naming the event. |
| AC6 | `ai-docs/claude-tools-hierarchy.md` is the single source of truth: each in-scope file that previously re-explained any of the four taxonomies now carries a one-line pointer to `claude-tools-hierarchy.md` instead, using the existing on-demand pointer pattern. |
| AC7 | `AGENTS.md` "Do not refer to a skill as an 'agent' or vice versa — the distinction matters for spawning" sentence is preserved verbatim at its current AGENTS.md site. NOT mirrored into `claude-tools-hierarchy.md`. |
| AC8 | **Consistency pass criterion (round-1 answer "Consistency only"):** AC1–AC7 all pass. Net char-count delta across the in-scope corpus is computed and reported in the PR body for transparency; the delta itself does NOT gate AC8 (any delta passes, positive or negative). |
| AC9 | **Hard size guard:** No in-scope file crosses the 40k-char hard cap. Every in-scope file currently in the 35k–40k warning band (AGENTS.md at 36,808; `.claude/agents/triage-runner.md` at 37,916; verify on PR open via `wc -c` and report) ends at ≤ its pre-PR char count. Files below 35k may grow as long as they stay strictly below 35k. |
| AC10 | `AGENTS.md § Propagation Rule` table receives one new row reading: `\| Any edit that changes a Tool / Subagent / Skill / Hook contract OR renames a stable anchor in claude-tools-hierarchy.md \| Update ai-docs/claude-tools-hierarchy.md in the same PR (contract changes) AND every inbound deep-link to renamed anchors (anchor renames). \|`. |
| AC11 | `ai-docs/claude-tools-hierarchy.md` gains: (a) an explicit `## Stable anchors` section listing every anchor other files deep-link to (initial set `#1-tools` / `#2-subagents` / `#3-skills` / `#4-hooks` / `#mental-model` / `#stable-anchors`); (b) a `<!-- stable-anchor: #<slug> -->` HTML comment beside each load-bearing heading whose slug appears in (a). |
| AC12 | `cargo doc --no-deps --workspace --all-features` (under `RUSTDOCFLAGS="-D warnings -D missing-docs"`) is clean. (Sanity check — this PR is instruction-files only, but any embedded code examples in `code-style.md` / `doc-convention.md` may have residual doc-test impact.) |
| AC13 | `actionlint .github/workflows/<file>.yml` passes for every workflow file IF any workflow file is touched — likely zero in this PR; declared so review-time grep doesn't miss it. |
| AC14 | Every Propagation-Rule sync-group whose anchor file is touched by this PR receives co-edits in the same PR (Review / Interview / Triage / Task-Design / Spec-Amendment / Learning-Log / Size-exemption-index / `.claude/rules/<file>.md` rows). |
| AC15 | Spec-writer Rule-5 substring blacklist in `.claude/agents/spec-writer.md` (and its mirror in `.claude/skills/interview/SKILL.md`) is NOT extended by this PR — terminology is style, not a forbidden-question class. Existing blacklist entries that contain the lowercase word "agent" in their prose (e.g. row commentary, not the regex itself) are subject to the AC3 sweep like any other in-scope file. |
| AC16 | **Clash-rename AXIOM lands in AGENTS.md.** A new AXIOM block — sited near the existing "Do not refer to a skill as an 'agent' or vice versa" sentence — forbids project-defined Tool / Subagent / Skill / Hook names from clashing with embedded (Anthropic-shipped or marketplace-plugin) names enumerated in `ai-docs/claude-tools-hierarchy.md` §§1a + 1b + 2a + 3a + 3b. The AXIOM specifies that on detected clash the project-defined name is the rename candidate, never the embedded one. Block follows the fail-loud three-row table pattern from `ai-docs/agent-writing-style.md`. |
| AC17 | **`/ai-audit` Checklist O added.** `.claude/skills/ai-audit/reference.md` gains a new `## Checklist O — Embedded-name clash scan` section. SKILL.md's collapsed Checklist table gains the matching row. The recipe scans every project-defined Tool / Subagent / Skill / Hook name (enumerated from `.claude/agents/*.md` frontmatter `name:` field + `.claude/skills/*/SKILL.md` frontmatter `name:` field + `.claude/settings.json` Hook event names) against the embedded inventory in `claude-tools-hierarchy.md` §§1a + 1b + 2a + 3a + 3b. Match → `major` finding with rename recommendation. Checklist O is the next available letter after Checklist N (verified via `grep -n "^## Checklist" .claude/skills/ai-audit/reference.md` at design time). |
| AC18 | **`verify` Skill rename complete.** The folder `.claude/skills/verify/` is renamed to `.claude/skills/verify-change/`; the SKILL.md frontmatter `name:` field is `verify-change`; every inbound reference enumerated in *Files touched* is updated to the new name; `grep -rn '\b/verify\b\|skills/verify\b\|skill:\s*verify\b' .claude/ ai-docs/ AGENTS.md CLAUDE.md` returns only intentional residuals (`ai-docs/learnings.md` historical entries; `ai-docs/plans/done/**` archived plans; the literal `git commit --no-verify` / `cargo` `verify` prose mentions). No broken Markdown anchor links result. `ai-docs/claude-tools-hierarchy.md` §3c row reads `verify-change` (without any "project override of embedded `verify`" parenthetical); §3a embedded-Skills row for `verify` is verified present per AC21. |
| AC19 | **Clash-scan baseline passes after the rename.** Running the Checklist O recipe (per AC17) against the post-rename tree returns zero clashes — i.e. no remaining project-defined Tool / Subagent / Skill / Hook name matches any embedded name enumerated in `claude-tools-hierarchy.md` §§1a + 1b + 2a + 3a + 3b. This serves as the "first incidental fix landed clean" gate. |
| AC20 | **`ai-docs/skill-size-exemptions.md` / Checklist K drift updated for the rename.** If `.claude/skills/verify/SKILL.md` was previously cited in `ai-docs/skill-size-exemptions.md` (live check at design time — likely NO since the file is 11 lines), the index entry is updated to `.claude/skills/verify-change/SKILL.md`. Otherwise this AC is a no-op declared for completeness. |
| AC21 | **Embedded `verify` Skill documented in `claude-tools-hierarchy.md` §3a.** `ai-docs/claude-tools-hierarchy.md` §3a (Embedded Skills table) gains a NEW row with `verify` in the Skill column and the upstream description as the Purpose column. Verbatim upstream text: *"Verify that a code change actually does what it's supposed to by running the app and observing behavior. Use when asked to verify a PR, confirm a fix works, test a change manually, check that a feature works, or validate local changes before pushing."* A close paraphrase is acceptable iff the table layout demands brevity (design-phase call) — paraphrase MUST preserve the verb "verify", the triggers ("verify a PR", "confirm a fix works", "test a change manually", "validate local changes before pushing"), and the observed-behavior clause. Any preamble / introductory sentence in §3a citing the embedded-Skills count is updated to the new total (`11 → 12`). No "project override" cross-reference parenthetical appears on the §3a side — AC18's rename eliminates the clash. This AC is unconditional: the §3a row MUST exist post-PR regardless of the live §3a baseline at design time. |

## Open questions

None remaining — all round-1/round-2/round-3 ambiguities are resolved or folded in as defensible defaults:

- **O1 (Propagation Rule new row)** → folded-in default in *Key decisions* and AC10. **Rationale:** the round-2 "Freeze + mark" answer logically implies SOME enforcement loop for anchor stability — without a Propagation-Rule row, the `<!-- stable-anchor: #x -->` markers carry no mechanical follow-up. The proposed row text merges two triggers (contract change OR stable-anchor rename) under one column so the table doesn't gain two rows. The contract-change clause also catches future embedded-name additions that may clash with existing project names. Design Amendment may split or narrow it during implementation.
- **O2 ("agent" lowercase carve-outs)** → folded-in default in *Key decisions* with a 5-item whitelist; AC3 enforces. **Rationale:** (1) banner `# Rust OSS Agent Rules` is the project title — proper-noun compound; (2) `## Agent Docs` heading + `ai-docs/agent-docs-index.md` filename refer to documentation pages, where "Agent Docs" is a compound noun for "documentation about subagents/skills/etc.", not a synonym for "subagent"; (3) filenames `ai-docs/agent-docs-index.md` and `ai-docs/agent-writing-style.md` stay (rename has broad Propagation-Rule blast radius, deferred); (4) `Claude Agent SDK` / `Anthropic Agent SDK` are external product names; (5) `ai-docs/learnings.md` is APPEND-ONLY (Boundary rule 1). The whitelist is testable as part of AC3 via the substring blacklist + explicit exemption list.
- **O3 (move AGENTS.md "skill vs agent" sentence into hierarchy doc)** → resolved NO in *Key decisions* / AC7. Round-1/2 "Consistency only" removed the size pressure that would have motivated the move; AC9 still constrains AGENTS.md not to grow, which the per-file rewrite must respect via the de-dup pointer extraction elsewhere in AGENTS.md.
- **O4 (rename `ai-docs/agent-writing-style.md`)** → resolved NO in *Out of scope* / *Deferred*; round-2 "Prose only" answer makes this moot.
- **O5 (rename `.claude/agents/` directory)** → resolved NO in *Out of scope*; round-2 "Prose only" answer + harness-dispatch risk makes this moot.
- **O6 (`verify` Skill rename target name — reversible)** → resolved `verify-change` in *Key decisions* with rationale; the design phase MAY surface a stronger candidate via Design Amendment. The choice is reversible because the rename is mechanical (grep + sed across a finite reference set).
- **O7 (Checklist O letter choice)** → resolved `O` in *Key decisions* (next available letter after Checklist N verified by grep). New top-level letter chosen over folding into Checklist K because clash-detection is a distinct concern from skill-directory-layout.

The design agent may surface a Design Amendment if any of these folded-in defaults turn out to be wrong during implementation decomposition.
