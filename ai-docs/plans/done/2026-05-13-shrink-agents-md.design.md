# Design: Shrink AGENTS.md below 40k-char performance threshold

**Issue:** #323
**Date:** 2026-05-13

## Approach

**Chosen solution.** Apply mechanism (b) — AXIOM-plus-redundant-prose dedup — first, across the three heavy sections (`## Workflow`, `## Corrections Log`, `## Propagation Rule`). On the per-AXIOM dedup table below, this saves ~6,400 chars by itself, which is **insufficient** to clear the 8,572-char gap needed to reach ≤ 32,000. We therefore also apply mechanism (a) — extraction — to the two narrative passages in `## Corrections Log` (the **Boundary rule 1 Exception** + the **Entry format** key-glossary blockquote) and to the `## Workflow` section's two long narrative bullets (the **PR review comment resolution** GraphQL recipe and the **PR body sync after every push** prose). These four passages combined are ~5,800 chars and move cleanly into existing `ai-docs/` reference pages with one-line summaries left in `AGENTS.md` (precedent: `## Code Style`). Mechanism (c) is applied lightly: the Propagation Rule's `Any other instruction file` catch-all row in the sync-group table is already terse; the prose `**Sync groups (canonical):**` list below it is the real redundancy (also a (b) candidate — see dedup table).

**Projected savings:**

| Source | Mechanism | Approx chars removed |
|---|---|---|
| Workflow AXIOM 1 prose paragraph + sub-bullet (lines 139–140) | (b) | ~1,250 |
| Workflow AXIOM 2 prose paragraph (line 169) | (b) | ~880 |
| Workflow "PR review comment resolution" bullet (lines 156–168) → `ai-docs/workflow.md` | (a) | ~1,500 (kept link + 2-line summary, ~150 chars) |
| Dependency Versions "Query the registry before pinning" prose + "Per source:" list (lines 117–123) | (b) | ~1,650 |
| Propagation Rule "When editing..." restatement + "**Sync groups (canonical):**" prose list (lines 217–227) | (b) | ~2,150 (kept the table only; the prose list duplicates table rows) |
| Corrections Log Boundary rule 1 + 2 narrative bodies → `ai-docs/corrections-log.md` | (a) | ~3,200 (kept AXIOM-shaped one-line summary + 1-line link per rule, ~600 chars) |
| Corrections Log "Entry format" — the inline glossary blockquote (lines 323–326) → `ai-docs/corrections-log.md` | (a) | ~1,400 (template block stays; glossary moves) |

Total projected removal: **~12,030 chars**. Buffer above the 8,572 minimum: ~3,460 chars. Final projected size: **~28,540 chars** (well below the 32,000 cap, with margin for future binding-rule additions).

**Rejected alternatives.**

- *Mechanism (b) only across all three heavy sections* — modeled above. Saves ~6,400 chars, falls 2,200 chars short of target. Rejected on numbers, not principle. The spec's "sensible default" said try (b) first; we honoured that and confirmed it doesn't reach.
- *Extract `## Corrections Log` wholesale into `ai-docs/corrections-log.md`, leaving only a stub.* Rejected: Boundary Rule 2's substring `**Read the two boundary rules below before you write**` and the entry-format template block are read mid-flow by every agent that appends to `learnings.md`. Inline retention of the AXIOM-shaped summaries keeps the binding text one click away; only the per-exception elaboration and the field-glossary blockquote (not the template itself) move out.
- *Extract `## Workflow` wholesale into `ai-docs/workflow.md`.* Rejected: the H2 skeleton must stay (spec, "Out of scope"), and AXIOM 1, AXIOM 2, and the bullet list of small rules (cargo build before commit, never `git add -A`, etc.) are the canonical lookup. Only the two long narrative passages (PR review comment resolution + PR body sync prose) extract cleanly. AXIOMs and their tables stay in `AGENTS.md`.
- *Rewrite AXIOMs to be shorter.* Forbidden by spec § Out of scope ("Rewording any AXIOM... is out of scope. Only relocation and dedup are permitted").
- *Drop the per-Corrections-Log entry-format template block (lines 316–327) to `corrections-log.md`.* Rejected: the template itself is the line-by-line shape `learnings.md` appenders type. Keeping it in `AGENTS.md` makes appending a one-screen action. The *glossary blockquote* immediately after it (which explains what each field token means in prose, lines 323–326) is the extractable part.

## Per-AXIOM dedup decisions

Applies the spec's AXIOM-vs-prose dedup criterion: prose qualifies as redundant **when it contains no rule, exemption, mechanism, or example absent from the AXIOM block's header + decision table**.

| AXIOM (file line) | Adjacent prose | Verdict | Rationale |
|---|---|---|---|
| `actionlint MUST pass before git add` (line 38) | None below — section moves to `Search:` line and ends. | **No action.** | Nothing to dedup. |
| `Pre-publish: clean breaks. No compat shims.` (line 52) | "Revisit this rule before the first `cargo publish`." (line 61) | **Keep.** | Carries unique deferred-action timing that the AXIOM block does not state. 60 chars — negligible. |
| `_unchecked means unsafe AND UB-on-failure. Period.` (line 65) | Three bullets at lines 74–79 ("Follow std", "_unchecked is reserved for unsafe fns only.", "Default safe + checked", "Prefer non-panicking APIs", "Other with-vs-without runtime variants") | **Keep all bullets.** | The bullets carry rules absent from the table: (i) the panicking-acceptable-only-when-fundamental-invariant-broken policy, (ii) the `try_*` default + optional panicking-wrapper rule, (iii) the "other variants" descriptive-suffix rule. None of these appear in the AXIOM's three-row table. Dedup would lose binding content. |
| `Query the live registry BEFORE writing any specific version string` (line 98) | "Query the registry before pinning." paragraph (line 117) + "Per source:" list (lines 119–123) | **Remove prose paragraph; remove "Per source:" list.** | The AXIOM block + its decision table already state (i) the Cargo `curl` query command, (ii) the gh-action releases query, (iii) the long-lived-doc `(verified current YYYY-MM-DD)` annotation rule, and (iv) the action-behaviour evidence-source requirement. Lines 117–123 restate items (i)–(iii) verbatim plus a "the cost is asymmetric" paragraph that adds rhetoric, not rule. The four bullets at lines 110–115 (`0.x` / `x` pinning, no `~` prefix, `cargo update` after edits) stay — they are pinning *syntax* rules, not registry-query rules. **Savings: ~1,650 chars.** |
| `AXIOM 1 — NEVER edit on local master when work is intended for a PR` (line 127) | "Never edit on local `master`..." paragraph + Recovery sub-bullet (lines 139–140) | **Remove paragraph + sub-bullet.** | The AXIOM's three-row table includes the **STOP**/recovery row verbatim ("`master` AND you've already made commits (recovery)" → exact same `git stash → git checkout -b → git reset --soft → restore --staged` sequence). Lines 139–140 say the same thing in prose. **Savings: ~1,250 chars.** |
| `AXIOM 2 — Read the PR body via gh pr view after EVERY git push` (line 171) | "PR body sync after every push." prose bullet (line 169) — appears *above* AXIOM 2 in the source because the bullet list precedes the AXIOM | **Remove the prose bullet.** | The AXIOM block + four-row table cover the unconditional read, the conditional edit, the `gh pr create` exception, and the upstream-issue-body protection. The bullet at line 169 restates all four. **Savings: ~880 chars.** |
| `ai-docs/deferred/_inbox.md is written ONLY by /task Step 12 and /triage` (line 183) | None — section ends. | **No action.** | Already AXIOM-only. |
| `Edits to one instruction file MUST propagate to its sync-group siblings` (line 195) | "When editing any instruction file..." (line 217) + "**Sync groups (canonical):**" list (lines 219–225) + collapsed-group note (line 227) | **Remove "When editing..." line; remove "Sync groups (canonical)" list; keep collapsed-group note.** | The sync-group **table** (lines 198–215) is the canonical lookup the spec requires preserved. The prose list at lines 219–225 enumerates the same groups in narrative form (Review group / Triage group / Interview group / Corrections-Log group / Task/Design group / Snapshot-helper group) — each line restates two adjacent table rows. Line 217's "When editing..." is a one-line restatement of the AXIOM. Line 227 (the `task` ↔ `task-issue` collapse note) carries unique historical context not in the table — keep. The "**Procedure:**" block (lines 229–232) is the *how* (the `grep -rn` recipe + the "rule exemptions to enforcement files" propagation) — keep; it is rule, not restatement. The "Do not refer to a skill as an 'agent'" line (line 234) is a separate terminology rule — keep. **Savings: ~2,150 chars.** |

## Extraction destinations (mechanism (a))

Two new reference pages are created. Existing pages absorb nothing additional; the precedent of `## Code Style → ai-docs/code-style.md` is followed verbatim.

| Source passage | Destination | New file? |
|---|---|---|
| `## Workflow` "PR review comment resolution" bullet body (the GraphQL recipe, lines 156–168) | `ai-docs/workflow.md` § *PR review comment resolution* | **NEW** |
| `## Corrections Log` Boundary rule 1 *Exception* body (lines 281–291) + Boundary rule 2 *Exception* body (line 312) | `ai-docs/corrections-log.md` § *Boundary rule 1 Exception (Escalated? and Superseded by: agent-driven updates)* and § *Boundary rule 2 Exception (/improve and /ai-audit workflows)* | **NEW** |
| `## Corrections Log` *Entry format* — the inline glossary blockquote at lines 323–326 (`Escalated?` semantics + `doc-convention` + `code-style` + `Superseded by:` semantics) | `ai-docs/corrections-log.md` § *Entry format — field glossary* | (same NEW file) |

**Anchor / link format.** Same as the `## Code Style` precedent already in `AGENTS.md`:

```markdown
See [`ai-docs/<file>.md` → <Heading>](ai-docs/<file>.md#<github-flavoured-anchor>).
```

**Inline summary shape (kept in `AGENTS.md`).** The AXIOM blockquote and its decision table (Boundary Rule 1's blockquote, Boundary Rule 2's blockquote) stay in `AGENTS.md` verbatim. Only the *Exception* sub-block of each rule moves out, replaced by the line:

```markdown
> **Exception — see [`ai-docs/corrections-log.md` → Boundary rule 1 Exception](ai-docs/corrections-log.md#boundary-rule-1-exception).** `Escalated?` and `Superseded by:` MAY be updated in-place by the `self-improve` and `learnings-escalation-audit` agents; manual user edits are not authorised.
```

This preserves the binding intent ("Exception exists; only these two agents may edit those two fields") inline, while the full carve-out (`self-improve` bullet, `learnings-escalation-audit` bullet, the four sub-points for each) moves to the reference page.

**Anchor stability for existing external references.**

`grep -rn "AGENTS.md#" .claude/ ai-docs/` finds exactly one external anchored link:

- `ai-docs/code-style.md` line 57 → `AGENTS.md#api-naming`, targeting `## API Naming`.

`## API Naming` is **not in the heavy-extraction set** and is **untouched by this design**. The anchor remains valid. No update to `code-style.md` line 57 is required.

The verification command from spec AC5 (`grep -rn 'AGENTS.md#' .claude/ ai-docs/`) is re-run pre-commit to confirm no new anchored references appeared (e.g., from intervening commits on `master`); if any new one resolves to a removed/renamed H2, it is updated in the same PR per the Propagation Rule.

## Decomposition

Each task ends with `AGENTS.md` self-consistent (no broken internal link, no half-extracted passage). Atomic order matters: extractions land before in-section dedup, because the extraction tasks remove larger spans that the dedup tasks would otherwise have to grep around.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `ai-docs/workflow.md` with the *PR review comment resolution* section (verbatim copy of lines 156–168 of `AGENTS.md`, reformatted into a top-level page with `# Workflow` H1 + `## PR review comment resolution` H2; preserve the GraphQL recipe verbatim incl. the four-step list and the `gh api graphql` code blocks; MUST / NEVER directives stay verbatim). Add a brief page intro: *"This page extracts long narrative passages from [`AGENTS.md` § Workflow](../AGENTS.md#workflow). The AXIOMs and short bullet rules stay in AGENTS.md."* | `ai-docs/workflow.md` (new) | — |
| 2 | Create `ai-docs/corrections-log.md` with three sections: (a) `## Boundary rule 1 Exception` — verbatim copy of the *Exception* sub-block of Boundary rule 1 (lines 281–291), preserving all bullets and the "All other lines..." trailing sentence; (b) `## Boundary rule 2 Exception` — verbatim copy of the *Exception — `/improve` and `/ai-audit` workflows.* paragraph (line 312); (c) `## Entry format — field glossary` — copy lines 323–326 from AGENTS.md (the `Escalated?` semantics + `doc-convention` + `code-style` + `Superseded by:` reference format) and **convert them from code-fenced text to real markdown blockquotes**: in `AGENTS.md` they sit inside the `Entry format` ``` fence (opened line 316, closed line 327) so their leading `>` is literal fence-content; in `ai-docs/corrections-log.md` they must render as actual blockquotes, so write each line outside any code fence with a real `>` markdown blockquote prefix. Page intro: *"This page extracts the field-level glossary and exception bodies from [`AGENTS.md` § Corrections Log](../AGENTS.md#corrections-log). The boundary-rule AXIOM blockquotes themselves stay in AGENTS.md."* | `ai-docs/corrections-log.md` (new) | — |
| 3 | In `AGENTS.md`, replace the *PR review comment resolution* bullet (lines 156–168) with a 3-line bullet: bullet title + one-line summary + link line, citing `ai-docs/workflow.md#pr-review-comment-resolution`. Verify no AXIOM, MUST, NEVER, DENY, or ASK directive was in the removed body that needs reinstating in the summary (audit: the removed body contains one `**not**`, two `**Mechanics**` formattings, no MUST/NEVER/DENY/ASK directives — they live in the surrounding bullets, not this one). | `AGENTS.md` | 1 |
| 4 | In `AGENTS.md` `## Corrections Log`, replace the Boundary rule 1 *Exception* sub-blockquote (lines 281–291) with a one-line *Exception* summary + link; replace the Boundary rule 2 *Exception* paragraph (line 312) with a one-line *Exception* summary + link; remove the four `>`-prefixed glossary lines (323–326) from inside the *Entry format* code fence and close the fence after line 322 (the `**Superseded by:** ...` template row); then add, immediately after the closed fence, a one-line link to `ai-docs/corrections-log.md#entry-format-field-glossary`. Note this is **not** a "delete a blockquote" edit — those lines are code-fence content (their `>` is literal text inside the fence), so the mechanical action is shortening the fence body and moving the fence's closing ``` upward. Boundary rule 1 and 2 AXIOM blockquotes themselves (the headers + the prose body listing what NEVER applies, the file-list block in Boundary 2) stay verbatim — only the *Exception* sub-block moves. The entry-format **code block template** (lines 316–322 + the relocated closing fence) stays in `AGENTS.md`. | `AGENTS.md` | 2 |
| 5 | In `AGENTS.md` `## Workflow`, delete the prose paragraph + Recovery sub-bullet at lines 139–140 (AXIOM 1 dedup); delete the *PR body sync after every push.* bullet at line 169 (AXIOM 2 dedup). Verify the AXIOM 1 table row "`master` AND you've already made commits (recovery)" and the AXIOM 2 four-row table both cover everything removed (already audited above). | `AGENTS.md` | 4 |
| 6 | In `AGENTS.md` `## Dependency Versions`, delete the "Query the registry before pinning." paragraph (line 117) and the "Per source:" list (lines 119–123) — AXIOM dedup. The four pinning-syntax bullets at lines 110–115 and the surrounding "When adding or editing dependencies" intro stay. In `AGENTS.md` `## Propagation Rule`, delete (i) the "When editing any instruction file..." line (line 217), (ii) the **Sync groups (canonical):** prose list (lines 219–225). Keep the collapsed-group note (line 227), the **Procedure:** block (lines 229–232), and the "Do not refer to a skill as an 'agent'..." line (line 234). | `AGENTS.md` | 5 |
| 7 | Verification + propagation: (a) **Pre-edit snapshot** — before any of Tasks 1–6 land their commits, capture `grep -nE '\*\*(MUST\|NEVER\|DENY\|ASK)\*\*' AGENTS.md > /tmp/directives-before.txt` and the AC8 propagation grep into `/tmp/propagation-before.txt`. Tasks 1–6 may, in practice, be implemented inside a single session — in that case, snapshot from `git show master:AGENTS.md` rather than the live file. (b) Run the AC verification suite (Test Design section below). (c) Re-run the AC8 propagation `grep -rn` for the keywords removed/relocated across `.claude/agents/`, `.claude/skills/`, `ai-docs/`. The pre-edit grep is **expected to find** five named cross-references documented in AC8 — verify each one's post-edit state matches the per-ref decision recorded there (UPDATE refs now point at the new location; KEEP refs still resolve to a present span in AGENTS.md). (d) Update any *additional* source file whose anchored link or quoted phrase now points at a removed location (none anticipated beyond the AC8-enumerated refs — design phase audited `code-style.md:57` and it targets the untouched `## API Naming`). (e) Run `wc -c AGENTS.md` and confirm `≤ 32000`. | `AGENTS.md`, `.claude/skills/task/SKILL.md`, `.claude/skills/pr-commented/SKILL.md`, possibly any other `.claude/**` or `ai-docs/**` file flagged by the grep | 6 |

**Scope check:** 7 tasks — at the threshold the design agent rules cite (`> 7 tasks → propose split`). The 7th is verification-only, not implementation. Implementation steps are 6. Acceptable without split.

## Risks

| Risk | Mitigation |
|---|---|
| **Anchor mismatch on extraction.** A new GitHub-flavoured-markdown anchor (e.g., `#boundary-rule-1-exception`) may not match the H2 GitHub auto-generates. | After Task 1 and Task 2, render the new pages locally (or `gh markdown-preview` on a scratch branch) to confirm the auto-generated anchor exactly equals the link target. Worst-case fallback: rename the H2 to force the anchor (still verbatim semantics; only the heading text changes). |
| **External link breakage.** `ai-docs/code-style.md:57` points at `AGENTS.md#api-naming` — if `## API Naming` is accidentally renamed, the link breaks. | `## API Naming` is explicitly NOT in scope (heavy-extraction set is Workflow / Corrections Log / Propagation Rule). Task 7's `grep -rn "AGENTS.md#" .claude/ ai-docs/` confirms it; if `code-style.md:57` no longer resolves to a present H2, fix in the same PR per Propagation Rule. |
| **Agent grep miss.** Agents that grep `AGENTS.md` for the strings "PR review comment resolution", "Boundary rule 1 Exception", "Boundary rule 2 Exception", "Sync groups", or "Query the registry before pinning" may no longer find them inline. | `grep -rln` for each of these phrases across `.claude/agents/`, `.claude/skills/`, `ai-docs/` before commit. The wider pre-edit grep identified **five** named cross-references that this PR must audit; the per-ref decision is set at design time and verified by Task 7. **Per-ref decisions:** (1) `.claude/skills/task/SKILL.md:269` cites "the GraphQL recipe in AGENTS.md *Workflow* → 'PR review comment resolution' verbatim" — **UPDATE** to "the GraphQL recipe in `ai-docs/workflow.md` → 'PR review comment resolution'" because the recipe body moves to that page; (2) `.claude/skills/pr-commented/SKILL.md:183` cites "verbatim per AGENTS.md 'PR review comment resolution'" — **UPDATE** to "verbatim per `ai-docs/workflow.md` → 'PR review comment resolution'" because the recipe body moves; (3) `.claude/skills/ai-audit/SKILL.md:162` references "AGENTS.md *Boundary rule 1 Exception*" — **KEEP**: the AXIOM blockquote and the one-line *Exception* summary + link stay in `AGENTS.md`; the canonical entry for the *Exception* name remains there, with the full carve-out one click away in `ai-docs/corrections-log.md`. The reference resolves to a present, named anchor in AGENTS.md. (4) `.claude/skills/ai-audit/SKILL.md:163` references "AGENTS.md *Boundary rule 2 Exception*" — **KEEP** for the same reason as (3); both Exception summaries remain in AGENTS.md with forward links. (5) `.claude/agents/learnings-escalation-audit.md:3` "Authorised by AGENTS.md § Corrections Log Boundary rule 1 Exception" — **KEEP**: the authorising summary stays inline in AGENTS.md (the one-line *Exception* sub-blockquote that replaces the full carve-out), so this reference still resolves to a present sentence in AGENTS.md that itself links forward to `ai-docs/corrections-log.md` for the full bullets. Additionally, `.claude/skills/ai-audit/SKILL.md` lines 158–163 reference "AGENTS.md § Corrections Log's *Entry format* block" — the *Entry format* H3 heading and **code-block template** stay in AGENTS.md; only the glossary blockquote (which sat under the code-block) moves. The H3 anchor is preserved, so this reference also stays valid. Verify all six call sites in Task 7. |
| **Boundary Rule 2 violation in the implementation turn.** Boundary Rule 2 prohibits editing `learnings.md` in the same conversation turn as `AGENTS.md` edits. | Implementation turn writes only `AGENTS.md`, `ai-docs/workflow.md`, `ai-docs/corrections-log.md`. Per spec AC7, **pre-existing** working-tree changes to `learnings.md` (from prior turns) MAY be staged with the implementation commit — they are not a *same-turn write*, they are a same-PR file. Confirm at commit time via `git status` that `ai-docs/learnings.md` is either unmodified or unchanged relative to the prior-turn state. |
| **Char-count target miss.** Projections are estimates; actual removal may be lower if dedup boundaries are conservative. | Buffer of ~3,460 chars above the 32,000 cap. If Task 7's `wc -c` reports >32,000, the recovery is to extract one more passage (candidate: the Workflow `_inbox.md` AXIOM's prose narrative at lines 184–191 — currently ~500 chars of explanatory rows; can collapse to one-line summary + link to a future `ai-docs/triage.md`). Not in the default plan because the buffer is comfortable. |
| **AC4 regression: Propagation table header text shifts.** The verifier `grep -nE '^> \| If you edit\.\.\. \| You MUST also check / update\.\.\. \|' AGENTS.md` is character-sensitive. | The table header line (line 198) is NOT touched in Task 6. Task 6 only removes lines *below* the table (217–225). Verify exact byte equality of the header line in Task 7. |
| **CLAUDE.md import path breakage.** `CLAUDE.md` imports `@AGENTS.md`. Spec § Out of scope forbids editing the import line. | No task in the decomposition modifies `CLAUDE.md`. Verify in Task 7 that `git status` shows no `CLAUDE.md` change. |

## Test Design

Verification is mechanical greps + `wc -c`. No Rust code is modified; `cargo` gates are required only as a guard against scope drift (AC6).

### AC1 — `wc -c AGENTS.md ≤ 32000`

- **Location:** shell.
- **Command:** `wc -c < /home/syt/RustroverProjects/quartzite/AGENTS.md`
- **Expected before:** `40572`
- **Expected after:** `≤ 32000` (target band: `28000`–`30000` based on projection).
- **Strict assertion:** `[ "$(wc -c < AGENTS.md)" -le 32000 ]`.

### AC2 — every AXIOM preserved verbatim-in-intent

- **Location:** shell, paired before/after greps.
- **Before-snapshot:** `grep -nE '^\s*> \*\*AXIOM' /home/syt/RustroverProjects/quartzite/AGENTS.md > /tmp/axioms-before.txt`
- **After-snapshot:** same command → `/tmp/axioms-after.txt`.
- **Expected before count:** **8** AXIOM lines (lines 38 / 52 / 65 / 98 / 127 / 171 / 183 / 195).
- **Expected after count:** **8** (no AXIOM moved out per the dedup table — all 8 stay in `AGENTS.md`).
- **Verbatim check:** `diff <(grep -E '^\s*> \*\*AXIOM' AGENTS.md) <(git show master:AGENTS.md | grep -E '^\s*> \*\*AXIOM')` — empty diff required.

### AC3 — every MUST / NEVER / DENY / ASK directive preserved verbatim

- **Before-snapshot:** `grep -nE '\*\*(MUST|NEVER|DENY|ASK)\*\*' AGENTS.md > /tmp/directives-before.txt`
- **Expected before count:** Run pre-implementation to record. Initial probe: 4 distinct lines matched at the `grep -cE` level (line count of lines containing at least one match); the actual directive count is higher because some lines contain multiples (e.g., the `actionlint` AXIOM table cell `**NEVER** bypass`). Pre-implementation snapshot is the authoritative baseline.
- **After-snapshot:** same command.
- **Required:** every directive present in `AGENTS.md` before is either (a) present in `AGENTS.md` after, or (b) present in the new `ai-docs/workflow.md` or `ai-docs/corrections-log.md` file with an `AGENTS.md`-side anchored summary link.
- **Mechanical check:** `cat /tmp/directives-before.txt /tmp/directives-after.txt ai-docs/workflow.md ai-docs/corrections-log.md | grep -cE '\*\*(MUST|NEVER|DENY|ASK)\*\*'` — after-side total ≥ before-side total.

### AC4 — Propagation sync-group table header preserved

- **Command:** `grep -nE '^> \| If you edit\.\.\. \| You MUST also check / update\.\.\. \|' /home/syt/RustroverProjects/quartzite/AGENTS.md`
- **Expected:** exactly one match (line 198 ± natural drift from removed lines above).

### AC5 — external anchored references still resolve

- **Command:** `grep -rn 'AGENTS.md#' /home/syt/RustroverProjects/quartzite/.claude/ /home/syt/RustroverProjects/quartzite/ai-docs/`
- **Expected before:** one match — `ai-docs/code-style.md:57` → `AGENTS.md#api-naming`.
- **Expected after:** same one match (since `## API Naming` is untouched). If a new external anchored reference appears in a `git diff master --` of this PR's instruction-file edits, validate its target H2 still exists in `AGENTS.md`.

### AC6 — Rust gates pass

- **Commands (run from repo root):**
  - `cargo build`
  - `cargo fmt -- --check`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test`
- **Expected:** all pass. Trivially true because no `.rs` file is touched, but enforces the no-scope-drift boundary.

### AC7 — no NEW `learnings.md` entry in same turn

- **Command:** `git diff master -- ai-docs/learnings.md | grep -cE '^\+###'`
- **Expected:** zero new `### YYYY-MM-DD` headers added by this PR. (Pre-existing diff lines for the stale-progress-file entry from a prior turn may appear; they are not new headers — they were added before this turn.)
- **Manual check:** confirm `git log --oneline ai-docs/learnings.md` shows the most recent commit touching `learnings.md` is **not** the implementation commit of this PR (the stale-progress-file change came in a prior turn and may be staged together but is a separately-authored entry per spec AC7).

### AC8 — propagation grep

- **Command:** `grep -rn "PR review comment resolution\|Boundary rule 1 Exception\|Boundary rule 2 Exception\|Sync groups (canonical)\|Query the registry before pinning" /home/syt/RustroverProjects/quartzite/.claude/ /home/syt/RustroverProjects/quartzite/ai-docs/`
- **Expected matches and per-ref decisions** (identified at design time — verify each post-edit):
  - `.claude/skills/task/SKILL.md:269` — "follow the GraphQL recipe in AGENTS.md *Workflow* → 'PR review comment resolution' verbatim" — **UPDATE** to reference `ai-docs/workflow.md` (the recipe body moved). Suggested replacement: "follow the GraphQL recipe in `ai-docs/workflow.md` → 'PR review comment resolution' verbatim".
  - `.claude/skills/pr-commented/SKILL.md:183` — "Mechanics — verbatim per AGENTS.md \"PR review comment resolution\"" — **UPDATE** to "Mechanics — verbatim per `ai-docs/workflow.md` → 'PR review comment resolution'" (the recipe body moved).
  - `.claude/skills/ai-audit/SKILL.md:162` — "AGENTS.md *Boundary rule 1 Exception*" — **KEEP**. The named anchor (the one-line *Exception* summary + link sentence) stays in AGENTS.md. Reference resolves.
  - `.claude/skills/ai-audit/SKILL.md:163` — "AGENTS.md *Boundary rule 2 Exception*" — **KEEP**. Same rationale as the line-162 ref.
  - `.claude/agents/learnings-escalation-audit.md:3` — "Authorised by AGENTS.md § Corrections Log Boundary rule 1 Exception" — **KEEP**. The authorising summary stays inline in AGENTS.md (replacing the removed Exception body); the carve-out details are one click away.
  - Any other match references the rule by name without anchored link — confirm it resolves to a still-present span in AGENTS.md (e.g., the AXIOM-shaped summary that replaced the extracted carve-out). The body moved; the H3/H4 headings and AXIOM headers stay.
- **Verification:** post-edit, re-grep; confirm every match either (a) resolves to a still-present span in AGENTS.md (KEEP refs) or (b) has been updated to its new target in the same PR (UPDATE refs).

### Test entry-point summary

This task has no functions or modules; "test entry point" = the four AC verification scripts above, runnable as a single shell block:

```bash
cd /home/syt/RustroverProjects/quartzite
echo "AC1: $(wc -c < AGENTS.md) chars"
diff <(grep -E '^\s*> \*\*AXIOM' AGENTS.md) <(git show master:AGENTS.md | grep -E '^\s*> \*\*AXIOM') && echo "AC2: AXIOMs identical" || echo "AC2: FAIL"
grep -nE '^> \| If you edit\.\.\. \| You MUST also check / update\.\.\. \|' AGENTS.md && echo "AC4: table header present" || echo "AC4: FAIL"
grep -rn 'AGENTS.md#' .claude/ ai-docs/
cargo build && cargo fmt -- --check && cargo clippy --workspace -- -D warnings && cargo test && echo "AC6: PASS"
```

## Open questions

_(None — all design-affecting questions were resolved in the spec. Per-AXIOM dedup, extraction destinations, and propagation grep targets are all specified above.)_
