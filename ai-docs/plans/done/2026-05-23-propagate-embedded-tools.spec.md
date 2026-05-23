

# Propagate embedded tools to harnesses

**Source:** user description (free-text)
**Date:** 2026-05-23
**Tracked in:** #549

> **POST-MERGE AMENDMENT (PR #550 round 1).** Class C and Class D entries D3/D4 were **rolled back** after a `design-review` runtime probe on 2026-05-23 confirmed the harness elides `Agent` from a Subagent's runtime tool list even when declared in frontmatter `tools:` (corroborates `ai-docs/learnings.md` 2026-05-15 `self-improve` entry). Nested Subagent-from-Subagent spawn is unsupported. Affected ACs (AC6–AC11) are obsoleted by the rollback; AC1–AC5 (Classes A + B + D1 + D2), AC10 (entry C5 byte-for-byte), AC12 (stable anchors), AC13 (Interview-group no-op), AC14 (char budget), AC15 (Class E), AC16 (Class F) remain in force. `ai-docs/claude-tools-hierarchy.md` §2 now carries an explicit *Nested-spawn restriction* note documenting the harness behaviour. Class F decisions are moot in the rolled-back state (no rewrite → no retention review needed). Full revert details in PR #550 round-1 commit message.

## Problem statement

Several project-defined skills / subagents in `.claude/` perform work that the **embedded** Claude Code surface (Tools in `ai-docs/claude-tools-hierarchy.md` §1a/§1b, Subagents in §2a) already covers. The duplication has two costs:

1. **Behavioural drift.** Each in-line re-implementation can diverge from the embedded primitive's defaults (e.g., `Explore`'s tool-exclusion list, `claude-code-guide`'s domain-aware fetching of `code.claude.com` pages).
2. **Hierarchy-doc accuracy.** `ai-docs/claude-tools-hierarchy.md` §1a omits embedded tools the project's `.claude/` surface already relies on (`Grep`, `Glob` — both referenced in subagent `tools:` frontmatter at `spec-writer.md` and `design-review.md`).

## Scope

Six concrete classes of edit:

### Class A — Docs-fetch propagation (`claude-code-guide`)

Replace the orchestrator's inline `WebFetch` calls against `code.claude.com/docs/en/*` with one `Agent` Tool call spawning the embedded `claude-code-guide` subagent.

| # | Site | Today | After |
|---|------|-------|-------|
| A1 | `.claude/skills/ai-audit/SKILL.md` Phase 2 (line 66, lines 68–70, line 74) | "Fetch the three primary references via WebFetch … `code.claude.com/docs/en/{skills,sub-agents,hooks-guide}` … fetch additional pages from `code.claude.com` on demand" | "Spawn `claude-code-guide` via `Agent` with a `prompt` asking for the verbatim shape contracts for skill frontmatter / Subagent file structure / Hook event matchers, citing the three pages by URL. Re-use the returned text for the rest of the run." |
| A2 | `.claude/skills/ai-audit/SKILL.md` line 186 (anti-pattern note) | "Do **not** skip the `WebFetch` Tool call in Phase 2." | "Do **not** skip the `claude-code-guide` spawn in Phase 2." (Same prohibition, updated to point at the new primitive.) |

### Class B — Read-only code-search propagation (`Explore`)

Replace orchestrator-side read-only code-search guidance with an `Agent` spawn of the embedded `Explore` subagent, when the search is bounded and read-only (matches `Explore`'s contract per §2a).

| # | Site | Today | After |
|---|------|-------|-------|
| B1 | `.claude/skills/bugfix/SKILL.md` Step 1 (line 57: "Using only read-only tools (`rg`, `Read`, logs, test output), trace the actual execution path") | Orchestrator does the read-only trace inline | Orchestrator spawns `Explore` with a `prompt` instructing it to trace the execution path, draw the ASCII diagram per the existing template, and return both the diagram AND the file:line citations supporting each arrow. The `prompt` MUST include the verbatim `ast-index.md § Rules for subagents` block (per existing rule). |

`triage-runner` Phase 4.5 inline one-file `Read` — kept as-is. One bounded `Read` of a known path is not a search; spawning `Explore` for it would be heavier than the inline call.

### Class C — Nested-spawn rewrite (`design-review` + `spec-writer`)

Per round 2's answer ("Yes, rewrite both"), both subagents drop `Grep` / `Glob` from their `tools:` frontmatter and gain `Agent` (so they can spawn `Explore` for read-only search). `Read` is retained — both subagents read **known-path** files inline (`AGENTS.md`, the spec under review, source files named in the design doc); spawning `Explore` for a single known-path `Read` is the same over-spawn the Class B rationale rejects for `triage-runner` Phase 4.5. `Bash` is retained — `spec-writer` needs it for the existing Rule-5 mechanical-check block (`printf … | grep -iE …`), and both subagents may shell out for `gh` / `cargo` calls.

| # | Site | Today | After |
|---|------|-------|-------|
| C1 | `.claude/agents/design-review.md` `tools:` frontmatter (line 4) | `tools: Read, Grep, Glob, Bash` | `tools: Read, Bash, Agent` |
| C2 | `.claude/agents/design-review.md` body line 20 ("Every suspicion — **investigate via Read/grep**, don't guess") | Inline `Read/grep` directive | "Every suspicion — investigate via `Read` for known paths, or spawn `Explore` via `Agent` for code search across the workspace; don't guess." |
| C3 | `.claude/agents/spec-writer.md` `tools:` frontmatter (line 4) | `tools: Read, Write, Edit, Grep, Glob, Bash` | `tools: Read, Write, Edit, Bash, Agent` |
| C4 | `.claude/agents/spec-writer.md` body line 30 ("Use `Grep` against AGENTS.md for any rule that might affect the spec") | Inline `Grep` Tool directive | Rewritten to shell out via `Bash` (`grep -nE '<keyword>' AGENTS.md`). AGENTS.md is a known path; `Bash` `grep` is lighter than spawning `Explore` for a single-file regex search and parity with C5's existing pattern. |
| C5 | `.claude/agents/spec-writer.md` Rule-5 substring blacklist mechanical-check block (the `printf '%s\n' … \| grep -iE '...'` shell snippet) | Already uses `Bash` + `grep` (not the `Grep` Tool) | **Unchanged.** This is a `Bash` invocation, not a `Grep` Tool call; it survives the `tools:` rewrite untouched. |

Each Class B / Class C `Explore` spawn passes the verbatim `ast-index.md § Rules for subagents` block in the `prompt` (per the existing rule in `.claude/rules/ast-index.md`), and inlines the returned text into the orchestrator's working context the same way today's inline `Grep` / `Read` results are consumed (no temp-file persistence under `ai-docs/`).

### Class D — Hierarchy-doc gap-fill

Update `ai-docs/claude-tools-hierarchy.md` §1a to enumerate every embedded tool the project's `.claude/` surface depends on. Also reflect the Class C rewrite in §2b (the project-Subagent table) so the documented `tools:` columns for `design-review` and `spec-writer` match their post-rewrite frontmatter.

| # | Edit | Rationale |
|---|------|-----------|
| D1 | Add row for `Grep` to §1a — Origin 🟦, Purpose "Regex search across files", Parameters `pattern`, `path`, `glob`, `-i`, `-n`, `-l`, `-A`/`-B`/`-C`, `output_mode`, `type`, `head_limit`, `multiline` | Referenced by `spec-writer.md` and `design-review.md` `tools:` frontmatter pre-rewrite; documents the embedded tool regardless of the C1/C3 drops so other instruction files that still cite `Grep` resolve against an existing row |
| D2 | Add row for `Glob` to §1a — Origin 🟦, Purpose "Fast filename pattern matching", Parameters `pattern`, `path` | Same — `Glob` is in both subagents' `tools:` frontmatter pre-rewrite and is the project's standard filename-pattern tool |
| D3 | Update §2b row for `design-review` — `Tools` column from `Read`, `Grep`, `Glob`, `Bash` → `Read`, `Bash`, `Agent` | Documents the C1 rewrite; same-PR sync per Propagation Rule (Tools/Subagents anchor row in `AGENTS.md`) |
| D4 | Update §2b row for `spec-writer` — `Tools` column from `Read`, `Write`, `Edit`, `Grep`, `Glob`, `Bash` → `Read`, `Write`, `Edit`, `Bash`, `Agent` | Documents the C3 rewrite; same-PR sync per Propagation Rule |

All §1a additions land in §1a (loaded tools), preserving the existing column order. No row reordering. No stable-anchor changes (§1a's anchor is `#1-tools`, not per-row; §2b's anchor is `#2-subagents`, not per-row).

### Class E — Audit of remaining six project Subagents (audit-only, no behavioural rewrite this PR)

Per the round-3 user-directed scope expansion, audit the six project Subagents outside Classes A–C for further embedded-primitive adoption candidates. The deliverable is **a per-Subagent finding recorded inline in the spec below**, NOT a behavioural rewrite of any of the six in this PR (rewrites, if any, become follow-up PRs surfaced via Deferred or `/triage`).

Verdict glyphs (used in the table below):

- ✅ keep — no embedded-primitive candidate; inline path is correct
- 🟡 candidate — embedded primitive (`Explore` / `claude-code-guide` / `Agent`) could replace some inline path; follow-up PR appetite
- ⛔ inline mandatory — embedded primitive structurally unavailable to this Subagent class (e.g., `Agent` is unfulfillable from inside `self-improve` per the 2026-05-15 learnings entries; `self-improve` cannot spawn nested subagents)

| # | Subagent | Verdict | Finding |
|---|----------|---------|---------|
| E1 | `design` | ✅ keep | `tools:` line is unrestricted (no explicit `tools:` frontmatter — inherits full toolset). Body does NOT cite `Grep` / `Glob` Tools or `WebFetch` directly; investigation is shell-driven (`rg` / `grep` via `Bash`) and known-path `Read`. No `code.claude.com` fetches. No simulation of an embedded primitive. Inline path is correct for its decomposition / artefact-writing duty. |
| E2 | `self-review` | ✅ keep | No explicit `tools:` line. Body uses `rg` via `Bash` for marker-maintenance / unsafe-index / doc-section greps (e.g., line 58 `rg '\bunsafe\s*\{...'`, line 110 `rg '^\s*///\s*#\s*(...)'`). All are single-pattern shell-greps over a known-path file set (the diff under review) — same rationale that keeps the `spec-writer.md` line 30 `Bash` `grep` invocation inline (Class C entry C4) rather than spawning `Explore`. Diff-scoped review is not a workspace-wide code search. |
| E3 | `review-findings` | ✅ keep | No explicit `tools:` line. Body's `rg` patterns (line 37, 74, 88) mirror `self-review` — single-pattern shell-greps. Operates on the entire branch (no diff), which superficially looks like a workspace-wide code search, but the contract is to produce a findings table from direct file reads, not to discover symbol locations — `Explore`'s discovery-oriented contract is the wrong shape. Inline path is correct. |
| E4 | `triage-runner` | ✅ keep | No explicit `tools:` line. Phase 4.5 does one bounded one-file `Read` of a known path (already exempted from Class B per the existing Class B "kept as-is" note). `gh issue create/edit` via `Bash` is mutation, not search. No `WebFetch`, no `Grep`/`Glob` Tool calls. Inline path is correct. |
| E5 | `learnings-escalation-audit` | ✅ keep | No explicit `tools:` line. Body's `rg -n "<keyword>" AGENTS.md` (line 60) is a single-pattern shell-grep against a known path — same Class-C4 inline-`Bash`-grep rationale. The audit walks every entry of `learnings.md` and checks `Escalated?` targets resolve; the lookup pattern is shell-grep-against-known-paths, not search-by-symbol. Inline path is correct. |
| E6 | `self-improve` | ⛔ inline mandatory | The `Agent` primitive is **structurally unfulfillable** from inside `self-improve` per `ai-docs/learnings.md` (2026-05-15 entries: "`self-improve` silently degraded `/improve` Step 6" from PR #362 Commit C, and "`self-improve` subagent genuinely lacks the `Agent` primitive" P5 from PR #364). Step 6 explicitly uses the pause-and-surface pattern because the parent thread (which has `Agent`) dispatches reproducers, not the subagent. **Embedded-primitive propagation into `self-improve` is impossible by class-of-Subagent constraint.** Inline path is mandatory and well-justified. |

Audit-only deliverable: the table above lives in this spec; AC15 verifies it lands. Class E does NOT rewrite any of the six Subagents in this PR.

### Class F — `Bash` retention review for the rewritten Subagents

Per the round-3 user-directed scope expansion, walk the Subagents rewritten in Class C (`design-review`, `spec-writer`) plus every Class-E 🟡 candidate (none) and decide per-Subagent whether `Bash` can also be dropped from the `tools:` line after the `Grep`/`Glob` drop. The decision MUST be recorded inline with a per-Subagent justification line.

| # | Subagent | Decision | Justification |
|---|----------|----------|---------------|
| F1 | `design-review` | **Keep `Bash`** | `design-review` may shell out for `cargo check` / `cargo doc` style verifications and `gh pr view` calls when reviewing a design that depends on live state. Dropping `Bash` would force every such verification into the parent thread or into a nested `Agent` spawn — both heavier than the existing inline shell-out. No body line currently cites a `Bash` dependency that the post-rewrite spawn pattern would supersede; the retention is a conservative same-PR scope. (Future PR may revisit if a body audit finds `Bash` is dead.) |
| F2 | `spec-writer` | **Keep `Bash`** | `spec-writer` REQUIRES `Bash` for the Rule-5 mechanical-check `printf … \| grep -iE …` block (entry C5 — explicitly unchanged per AC10). It also needs `Bash` for the Class C4 rewrite (`grep -nE '<keyword>' AGENTS.md` against the pre-resolved-rules table) and for `gh issue view <N> --json …` lookups (already cited in the input contract). Drop = breakage. **Mandatory retention.** |
| F3 | (Class E 🟡 candidates) | n/a | Class E surfaced zero 🟡 candidates; nothing to walk here. If a future audit reclassifies any of the six, Class F repeats per-Subagent then. |

The retention decisions are recorded above; AC16 verifies the decisions land in the spec.

## Out of scope

- Renaming any project-defined Subagent / Skill / Hook (the Embedded-name clash AXIOM in `AGENTS.md § Propagation Rule` already covers naming conflicts; this task is the inverse — adopting embedded primitives where project ones duplicate them).
- Replacing `ast-index` usage with anything (`ast-index` is a marketplace plugin §3b, not a simulation candidate — it remains the project's preferred search primitive).
- **Behavioural rewrite of any Class-E-audited Subagent** (`design`, `self-review`, `review-findings`, `triage-runner`, `learnings-escalation-audit`, `self-improve`) in THIS PR. Class E is audit-only per the round-3 promotion; any 🟡 candidates surface as follow-up PRs. Class E's findings (table E1–E6) record zero 🟡 candidates — all six landed ✅ keep or ⛔ inline mandatory.
- Re-architecture of any Subagent's overall purpose (migrating to embedded `Plan` / `general-purpose` wholesale would be re-architecture, not propagation — out of scope by user's "Docs+search+gap" answer to Q1).
- Replacing `triage-runner` Phase 4.5's bounded one-file `Read` with `Explore` (too narrow for the spawn-cost trade-off; same rationale that keeps `Read` on `design-review` / `spec-writer` in Class C; reaffirmed by Class E entry E4).
- Touching `.claude/settings.local.json` (user-local state per `/ai-audit` Step 2.2).
- Re-fetching live `code.claude.com` content during this task to verify URLs — the three URLs in §A1 are the ones currently embedded in `/ai-audit`; if any 404 in practice, the `claude-code-guide` subagent surfaces that and the failure is treated as a docs-publisher change, not a regression of this task.

## Deferred

| What | Why | Separate issue needed? |
|------|-----|------------------------|
| Behavioural rewrite of any Class-E 🟡 candidate Subagent | Class E's table currently lists zero 🟡 candidates (all six landed ✅ keep or ⛔ inline mandatory). If a future audit reclassifies any to 🟡, the rewrite itself is its own PR per the user's round-3 promotion directive ("audit-only is acceptable for this PR"). | Conditional — only if a future Class E re-run flips a verdict to 🟡 |

## Key decisions

| Question | Decision |
|---|---|
| Q1 — Which classes of simulation are in-scope? | **All three:** Docs-fetch (Class A), read-only search (Class B), hierarchy-doc gap-fill (Class D) — per round 1 answer "Docs+search+gap". |
| Q2 — Replace, prefer, or audit-only? | **Replace** — each enumerated site is rewritten in this PR; no parallel-fallback retained. Per round 1 answer "Replace". |
| Q3 — Add missing rows (`Grep` / `Glob`) to `claude-tools-hierarchy.md` §1a? | **Yes, both** — per round 1 answer "Add both". Rows land in §1a; no anchor changes. |
| Q4 — Rewrite `design-review` and `spec-writer` to spawn `Explore` (commit to `Agent`-from-Subagent nested spawns)? | **Yes, both** — per round 2 answer "Yes, rewrite both". Encoded as Class C; both `tools:` lines drop `Grep`/`Glob` and add `Agent`. |
| Q5 — Promote the deferred "audit the remaining six Subagents" item into THIS PR? | **Yes, audit-only** — per round-3 user directive. Encoded as Class E. Findings recorded inline (table E1–E6); no behavioural rewrite of any of the six in this PR. All six landed ✅ keep (E1–E5) or ⛔ inline mandatory (E6 — `self-improve` lacks `Agent` per the 2026-05-15 learnings entries). |
| Q6 — Promote the deferred "`Bash` retention review for rewritten Subagents" item into THIS PR? | **Yes** — per round-3 user directive. Encoded as Class F. Decision recorded inline (table F1–F3) with per-Subagent justification. **Both rewritten Subagents keep `Bash`**: F1 (`design-review`) for `cargo` / `gh` shell-outs, F2 (`spec-writer`) for the Rule-5 `printf … \| grep -iE …` block (entry C5) and Class C4 (`grep -nE … AGENTS.md`) and `gh issue view`. |
| (Default applied silently — no question asked) | `Read` and `Bash` are RETAINED in both rewritten subagents' `tools:` lines. `Read` because each subagent reads known-path files inline (AGENTS.md / the spec / source files cited by the design doc) — spawning `Explore` for a single known-path `Read` is the same over-spawn rationale that keeps `triage-runner` Phase 4.5 inline. `Bash` because `spec-writer` needs the existing Rule-5 mechanical-check `Bash` snippet (entry C5) and both may need `gh` / `cargo` shell-outs. |
| (Default applied silently — no question asked) | `spec-writer.md` body line 30's `Grep`-against-AGENTS.md directive is rewritten as a `Bash` `grep` invocation (Class C entry C4), not an `Explore` spawn. AGENTS.md is a single known path; the existing Rule-5 mechanical-check block (entry C5) already shells out via `Bash`+`grep` against this same file — parity beats spawn-cost. |
| (Default applied silently — no question asked) | `/bugfix` Step 1's "Using only read-only tools" guidance line IS rewritten to spawn `Explore` (entry B1). Step 1's other inner steps (artefact creation, `current_step` tracking) are unchanged — `Explore`'s output feeds into the existing trace template. |
| (Default applied silently — no question asked) | `claude-code-guide` spawn output (Class A) is inlined into orchestrator context the same way today's `WebFetch` results are; no temp-file persistence under `ai-docs/`. If a future skill needs cross-phase re-use, that's a separate concern. |
| (Default applied silently — no question asked) | Class D row contents follow the existing column order (`Tool` / `Origin` / `Purpose` / `Parameters` for §1a; `Subagent` / `Purpose` / `Tools` for §2b) with the embedded glyph 🟦. No new columns introduced. |

## Technical constraints

- **AGENTS.md § Propagation Rule, Tools / Subagents / Skills / Hooks anchor row.** "Any edit that changes a Tool / Subagent / Skill / Hook contract OR renames a stable anchor in `claude-tools-hierarchy.md`" requires `ai-docs/claude-tools-hierarchy.md` to be updated in the same PR. Class D IS that update: D1/D2 document existing-but-missing tool rows; D3/D4 sync §2b `tools:` columns with the Class C frontmatter rewrite. Classes A and B do NOT change any tool contract (they consume existing contracts), so the rule does not impose further inbound-link rewrites for those.
- **AGENTS.md § Propagation Rule, Interview group row.** Class C edits `.claude/agents/spec-writer.md`; the row requires `.claude/skills/interview/SKILL.md` to be checked / updated in the same PR. The Class C edits to `spec-writer.md` are `tools:`-line + body-line-30 only; the Rule-5 substring blacklist mirrors live in `interview/SKILL.md` are unaffected (entry C5 leaves the blacklist block unchanged). Design records the explicit no-op verification.
- **AGENTS.md § Propagation Rule, sync groups for Class-E-audited Subagents — NO-OP this PR.** Class E is audit-only. The six audited Subagent files (`design`, `self-review`, `review-findings`, `triage-runner`, `learnings-escalation-audit`, `self-improve`) are NOT edited in this PR; therefore their sync-group rows (Review group for `self-review` / `review-findings`; Triage group for `triage-runner`; Learning-Log group for `self-improve` / `learnings-escalation-audit`; Task/Design group for `design`) do NOT fire. The Class E table is recorded inside THIS spec only — a docs artefact, not an instruction-file edit. **If a future follow-up PR acts on any 🟡 verdict (currently zero), that PR triggers the relevant sync-group rows and must enumerate them at design time.**
- **AGENTS.md § Embedded-name clash AXIOM.** Project-defined names MUST NOT clash with embedded names. This task is the dual direction (drop project simulations in favour of embedded primitives); no rename is triggered. `Explore`, `claude-code-guide`, `Agent`, `Grep`, `Glob` are all embedded; no project names collide with them post-rewrite.
- **`.claude/rules/ast-index.md` § Rules for subagents.** When the orchestrator spawns a Subagent for code search, the subagent does NOT inherit `ast-index.md`. The verbatim block MUST be passed in the `Agent` Tool's `prompt`. Class B's `Explore`-spawn `prompt` (and Class C's, when `design-review` / `spec-writer` spawn `Explore` post-rewrite) therefore embeds the verbatim block.
- **40 000-char per-instruction-file cap (AGENTS.md § Build & Test).** Post-edit, every touched instruction file MUST stay below 40 000 chars (35 000 = early-warning). The candidates (`ai-audit/SKILL.md`, `bugfix/SKILL.md`, `design-review.md`, `spec-writer.md`, `claude-tools-hierarchy.md`) are well under today; design verifies post-edit and AC7 grep-checks it.
- **AC-verification-grep contract.** Every AC in this spec MUST have a corresponding grep / shell check; the design records one `AC<N> verified by: <command>` line per AC.
- **`code.claude.com` URL stability.** The three URLs in Class A1 (`/docs/en/{skills,sub-agents,hooks-guide}`) are the ones currently embedded in `/ai-audit`; this task does not re-validate them. If any 404s in practice, `claude-code-guide` surfaces that and the failure is treated as a docs-publisher change, not a regression of this task.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/skills/ai-audit/SKILL.md` Phase 2 spawns `claude-code-guide` via the `Agent` Tool instead of calling `WebFetch` directly. Verifiable by: `grep -E 'WebFetch' .claude/skills/ai-audit/SKILL.md` returns no matches AND `grep -E 'claude-code-guide' .claude/skills/ai-audit/SKILL.md` returns ≥ 1 match. |
| AC2 | `.claude/skills/ai-audit/SKILL.md` anti-pattern note refers to the `claude-code-guide` spawn, not `WebFetch`. Verifiable by: `grep -F 'Do **not** skip the `claude-code-guide`' .claude/skills/ai-audit/SKILL.md` matches. |
| AC3 | `.claude/skills/bugfix/SKILL.md` Step 1 spawns `Explore` via the `Agent` Tool, and the spawn `prompt` includes the verbatim `ast-index.md § Rules for subagents` block. Verifiable by: `grep -E 'subagent_type.*Explore' .claude/skills/bugfix/SKILL.md` matches AND `grep -F 'ast-index search' .claude/skills/bugfix/SKILL.md` matches (the verbatim block contains this command). |
| AC4 | `ai-docs/claude-tools-hierarchy.md` §1a has a row for `Grep` with 🟦 origin and at least the `pattern` and `path` parameters listed. Verifiable by: `grep -E '^\\| `Grep` \\| 🟦' ai-docs/claude-tools-hierarchy.md` matches. |
| AC5 | `ai-docs/claude-tools-hierarchy.md` §1a has a row for `Glob` with 🟦 origin and at least the `pattern` and `path` parameters listed. Verifiable by: `grep -E '^\\| `Glob` \\| 🟦' ai-docs/claude-tools-hierarchy.md` matches. |
| AC6 | `.claude/agents/design-review.md` `tools:` frontmatter no longer contains `Grep` or `Glob`, and DOES contain `Agent`. Verifiable by: `head -10 .claude/agents/design-review.md \| grep -E '^tools:'` shows a line matching `tools: Read, Bash, Agent` (order tolerant) AND does NOT match `Grep` or `Glob`. |
| AC7 | `.claude/agents/spec-writer.md` `tools:` frontmatter no longer contains `Grep` or `Glob`, and DOES contain `Agent` (plus retained `Read`, `Write`, `Edit`, `Bash`). Verifiable by: `head -10 .claude/agents/spec-writer.md \| grep -E '^tools:'` shows `Read`, `Write`, `Edit`, `Bash`, `Agent` (order tolerant) AND does NOT match `Grep` or `Glob`. |
| AC8 | `.claude/agents/design-review.md` body no longer says "investigate via Read/grep"; it names `Explore` (via `Agent`) for code search. Verifiable by: `grep -F 'investigate via Read/grep' .claude/agents/design-review.md` returns no matches AND `grep -E 'spawn `Explore`' .claude/agents/design-review.md` matches. |
| AC9 | `.claude/agents/spec-writer.md` body line 30 no longer instructs use of the `Grep` Tool against AGENTS.md; it shells out via `Bash` `grep` instead. Verifiable by: `grep -F 'Use `Grep` against AGENTS.md' .claude/agents/spec-writer.md` returns no matches AND `grep -E 'grep -[nE].* AGENTS.md' .claude/agents/spec-writer.md` returns ≥ 1 match (the Class C4 rewrite plus the pre-existing C5 block). |
| AC10 | `.claude/agents/spec-writer.md` Rule-5 substring blacklist mechanical-check block (entry C5) is byte-for-byte unchanged. Verifiable by: `grep -F "printf '%s\n'" .claude/agents/spec-writer.md` matches (single backslash + `n` literal — what the file actually contains) AND `grep -F "backward.compat" .claude/agents/spec-writer.md` matches (the surrounding `grep -iE` literal blacklist token is preserved). |
| AC11 | `ai-docs/claude-tools-hierarchy.md` §2b rows for `design-review` and `spec-writer` reflect the post-rewrite `tools:` lines. Verifiable by: `grep -E '^\\| `design-review` \\|' ai-docs/claude-tools-hierarchy.md` shows `Read`, `Bash`, `Agent` and not `Grep`/`Glob` AND `grep -E '^\\| `spec-writer` \\|' ai-docs/claude-tools-hierarchy.md` shows `Read`, `Write`, `Edit`, `Bash`, `Agent` and not `Grep`/`Glob`. |
| AC12 | The Propagation Rule firing for this PR is honoured: stable anchors in `claude-tools-hierarchy.md` remain intact. Verifiable by: `grep -E 'stable-anchor: #(1-tools\|2-subagents\|3-skills\|4-hooks\|mental-model\|stable-anchors)' ai-docs/claude-tools-hierarchy.md` returns 6 matches. |
| AC13 | Interview-group Propagation-Rule no-op is recorded: editing `spec-writer.md` did NOT require an `interview/SKILL.md` change. The Rule-5 substring blacklist is **not mirrored** into `interview/SKILL.md` by design (it forbids inlining the blacklist there per its own anti-pattern note); the no-op holds iff `interview/SKILL.md` remains free of the blacklist tokens AND the file is untouched by this PR. Verifiable by: `! grep -qE 'backward.compat\|compat.shim\|deprecat\|legacy' .claude/skills/interview/SKILL.md` exits 0 (blacklist tokens stay absent) AND `git diff --name-only $(git merge-base HEAD master)..HEAD -- .claude/skills/interview/SKILL.md` produces no output (file untouched in this PR). |
| AC14 | Every touched instruction file stays below 40 000 chars. Verifiable by: `wc -c .claude/skills/ai-audit/SKILL.md .claude/skills/bugfix/SKILL.md .claude/agents/design-review.md .claude/agents/spec-writer.md ai-docs/claude-tools-hierarchy.md` — every line's char count < 40000. (Class E / Class F additions live in THIS spec only and do not touch the six audited Subagent files.) |
| AC15 | Class E's per-Subagent audit deliverable lands in this spec: a table with one row per audited Subagent (`design`, `self-review`, `review-findings`, `triage-runner`, `learnings-escalation-audit`, `self-improve`) carrying a verdict glyph (✅ keep / 🟡 candidate / ⛔ inline mandatory) and a justification cell. Verifiable by: `grep -cE '^\\| E[1-6] \\|' ai-docs/plans/2026-05-23-propagate-embedded-tools.spec.md` returns `6` AND every row carries one of ✅ / 🟡 / ⛔. The audit being entirely ✅ / ⛔ (zero 🟡) is an acceptable outcome per the round-3 promotion directive ("outcome may be 'keep on all'"). |
| AC16 | Class F's per-Subagent `Bash`-retention decision lands in this spec for both Class C subagents AND every Class E 🟡 candidate (currently zero). Verifiable by: `grep -cE '^\\| F[1-3] \\|' ai-docs/plans/2026-05-23-propagate-embedded-tools.spec.md` returns `3` AND rows F1 / F2 explicitly state "Keep `Bash`" with a justification line each. If Class E surfaces a future 🟡 (re-run scope), this AC re-fires with additional rows. |

## Open questions

_(none — all design-affecting questions resolved across rounds 1–3; Deferred holds one conditional follow-up that fires only if a future Class-E re-run flips any verdict to 🟡.)_
