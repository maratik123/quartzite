# `/next` deferred-file discoverability + `_inbox.md` AGENTS.md governance

**Source:** issue #202
**Date:** 2026-05-10
**Tracked in:** #202

This is umbrella issue **A1** of the four-issue process-improvements plan
([`ai-docs/plans/2026-05-10-process-improvements.md`](2026-05-10-process-improvements.md)).
A1 ships first, hygiene-first, and is verifiable on **current data** without any new file
being created in this issue. `_inbox.md` itself is created later (in A2) — A1 lands the
AGENTS.md governance for it ahead of time.

Strict sequence: **A1 → A2 (#203) → B (#204) → C (#205)**. No parallel paths.

## Scope

1. **`.claude/skills/next/SKILL.md` — extend the prompt to read every row from `ai-docs/deferred/*.md`** (the 8 thematic files plus `widget-backlog.md`), in addition to the existing `gh issue list` + `INDEX.md` reads.
   - **Tracked rows** (whose `Tracked` cell holds an issue ref `#N` in the 8 thematic files, or whose `Notes` cell contains a `tracked: #N` reference in `widget-backlog.md`): rank as *supplements* to the matching open issue. They must not produce a second top-line recommendation if the issue is already in the candidate set — no double-recommendation.
   - **Untracked rows** (whose `Tracked` cell holds `—` in the thematic files, or whose `Status` cell is `🟡 v2` in `widget-backlog.md`): surface in a new section titled *"Candidates needing `/triage`"* in the skill's output. These rows are **never** chosen as the recommended task — only listed for situational awareness, with a suggestion to run `/triage` first.
   - `/triage` itself does not exist yet (it ships in Issue B / #204). Until then, the *Candidates needing `/triage`* section is informational only — the user can surface a candidate manually via `/interview` if they want to act on one.
2. **`AGENTS.md` — new AXIOM in the *Workflow* section** (multi-paragraph blockquote + action table per `ai-docs/agent-writing-style.md` Pattern 1). The proposed prose is locked in the meta-plan; the spec adopts it verbatim:

   > **AXIOM — `ai-docs/deferred/_inbox.md` is written ONLY by `/task` Step 12 and `/triage`.**
   > Hand-edits to `_inbox.md` defeat the propagation contract that Issue A2 sets up — they hide rows from the parser and conflict with future Step-12 appends.
   >
   > | If you see... | Action |
   > |---|---|
   > | A row in `_inbox.md` you want to move to a thematic file | Run `/triage`; let it sort the row |
   > | A row in `_inbox.md` you want to drop | Run `/triage`; mark "drop" during the drain step |
   > | A row missing from `_inbox.md` for a freshly-merged spec | Re-run `/task` Step 12 manually (or wait for the next merged spec to trigger it) |
   > | An entry whose source-spec section shape was unrecognised by the parser | Step 12 emits a warning; resolve by reformatting the source spec OR by adding the shape to the parser's allow-list (Issue A2 design phase) |

   The axiom intentionally references `_inbox.md`, `/task` Step 12, and `/triage` even though they don't yet exist — A2 / B fulfil the references. This is a deliberate forward note, not a stale pointer.
3. **`AGENTS.md` *Agent Docs* table — new row** for `ai-docs/deferred/_inbox.md` with the one-line purpose:

   > *"triage queue — rows from completed specs awaiting `/triage` classification (writers: `/task` Step 12 and `/triage` only; introduced in Issue A2)."*

   The "introduced in Issue A2" suffix is a deliberate forward note that disappears when A2 lands.

## Out of scope

- **Creating `ai-docs/deferred/_inbox.md`.** That file is created by Issue A2 (#203) alongside the backfill that populates it. A1 lands AGENTS.md governance only.
- **`/triage` skill.** Ships in Issue B (#204). A1's reference to `/triage` is a forward note; the *Candidates needing `/triage`* section A1 adds to `/next` is informational until B lands.
- **`/task` Step 12 propagation.** Ships in Issue A2 (#203).
- **md ↔ gh issues bridge / drift detection.** Ships in Issue C (#205).
- **Visual surface for the maintainer.** Punted to the v1 quartzite UI-designer track per the meta-plan.
- **Reshaping `widget-backlog.md` schema** to add a `Tracked` column. Tracked refs go in the existing `Notes` cell; A1 only *reads* the file, no schema migration.
- **Renaming the `Tracked` column to `Issue`.** Noted in the meta-plan as a future cosmetic improvement; out of scope here.

## Deferred

- New informal sync-group `next/SKILL.md` ↔ `triage/SKILL.md` (mutual cross-reference) | documented in the *Sync-group footprint* of B; A1 does not create this group because `/triage` does not yet exist | no separate issue needed — folded into B (#204)
- Eventual removal of the "introduced in Issue A2" forward note from the *Agent Docs* row | once A2 lands, the suffix becomes stale | no separate issue needed — A2's PR (#203) will drop the suffix as part of its normal sync-group propagation

## Key decisions

| Question | Decision |
|---|---|
| Should A1 create `_inbox.md` itself? | **No.** Meta-plan locked: A1 = governance only; A2 creates the file. Forward references in the AXIOM and *Agent Docs* row are intentional. |
| Output placement for untracked rows in `/next`? | New section titled *"Candidates needing `/triage`"*, surfaced after the recommendation + runner-ups, as informational-only (never the top-line recommendation). |
| How does `/next` distinguish tracked vs. untracked rows in `widget-backlog.md`? | Status emoji `🟡 v2` ⇒ untracked candidate. `Notes` cell containing `tracked: #N` ⇒ tracked. Other status emojis (`✅` / `🤔` / `❌` / `📭`) are skipped — they are not "candidates" in the discoverability sense. |
| How does `/next` avoid double-recommendation when a row's `Tracked` is `#N` AND that `#N` is already in `gh issue list`? | Match by issue number; if the issue is already in the candidate set, the deferred-file row is *not* re-listed as a separate item — it appears (if at all) as a one-line supplement under that issue's recommendation. |
| Style template for the new AGENTS.md AXIOM? | `ai-docs/agent-writing-style.md` Pattern 1 — multi-paragraph blockquote + action table. Verbatim prose locked in meta-plan §A1 deliverables (and reproduced in *Scope* item 2 above). |
| Where does the AXIOM go in AGENTS.md? | Inside the existing **Workflow** section (alongside AXIOM 1 / AXIOM 2). Same blockquote shape as those. |
| Format of the *Agent Docs* table row | `\| `ai-docs/deferred/_inbox.md` \| triage queue — rows from completed specs awaiting `/triage` classification (writers: `/task` Step 12 and `/triage` only; introduced in Issue A2). \|` — column count matches the existing table. |
| Sync-group footprint | `next/SKILL.md` (changed prompt logic) and `AGENTS.md` (new AXIOM + new *Agent Docs* row). No new sync-group entry created in A1 — the future `/triage` ↔ `/next` group lives in B. |

## Technical constraints

- **Verifiable on current data.** AC verification must use the `ai-docs/deferred/*.md` files as they exist on the A1 PR's branch, not on hypothetical future state. The current data already contains:
  - Multiple `Tracked: —` rows (e.g., *Auto-merge* in `ci-docs-workflow.md`, *removing rstest* in `ci-docs-workflow.md`, *full wildcard re-exports* / *Future features* / *EXAMPLES.md* across other thematic files).
  - At least one tracked row whose `#N` is also an open issue (e.g., `Tracked: #48` for *BlockingQueued connection type*) — usable for AC2 double-recommendation check.
- **`widget-backlog.md` parser corner case.** A prose hit on the literal word "Tracked" exists at `widget-backlog.md:89` (`> spec. Tracked: TBD (file an issue when first item-view need surfaces).`). The `/next` prompt must anchor on column-header context (or scrub the prose hit) — bare-substring matching mis-classifies the row. The skill is prompt-driven (no Rust code), so the constraint is "the prompt instruction must explicitly tell the agent to anchor on the table-header context, not bare substrings".
- **`/next` is `disable-model-invocation: true`** (per current frontmatter) — it is invoked manually, not by the model. The skill output is the model's free-form response composed from the skill's instructions and the embedded `gh issue list` / file reads. No code changes; only prompt and instruction changes.
- **AGENTS.md style compliance.** The AXIOM follows Pattern 1 from `ai-docs/agent-writing-style.md` (multi-paragraph blockquote + action table). The *Agent Docs* row matches the existing 2-column table.
- **No CI gate for cross-issue ordering.** A1 → A2 → B → C is honour-system. Reviewer enforces. AC chains across issues make out-of-order merges visible (A2's backfill ACs reference `_inbox.md`'s axiom landed in A1, etc.) but A1 itself does not block on any future issue.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `/next` surfaces ≥ 1 untracked row in a new *Candidates needing `/triage`* section when run on current data. The output must explicitly list at least one of: *Auto-merge*, *removing `rstest`*, *full wildcard re-exports*, *EXAMPLES.md*, *Future features (`extension`, `8k_pages`)*. Verification: manual run on current `master` data after the A1 changes; capture the output and confirm the section exists with at least one of those rows present. |
| AC2 | `/next` does not double-rank a deferred-file row whose `Tracked` cell holds `#N` referring to an already-listed open issue. Verification: pick a row with `Tracked: #48` (the `BlockingQueued connection type` row in `signals-slots.md`); confirm only one mention of `#48` in `/next`'s output (either as the recommendation/runner-up *or* as a supplement, never both). |
| AC3 | `AGENTS.md` *Workflow* section contains the `_inbox.md` AXIOM with the four-row action table; `AGENTS.md` *Agent Docs* table includes the `_inbox.md` row with the "writers: `/task` Step 12 and `/triage` only" clause. Verification: `grep -A 12 'AXIOM.*_inbox.md' AGENTS.md` shows the axiom + table; `grep '_inbox.md' AGENTS.md \| wc -l` returns ≥ 2 (axiom + Agent-Docs row at minimum). |
| AC4 | Sync-group propagation: the AGENTS.md governance for `_inbox.md` is wired in exactly the two places this issue introduces. Verification: `grep -rn 'ai-docs/deferred/_inbox.md' .claude/ AGENTS.md` count must be **exactly 2** (the AXIOM first-line reference + the *Agent Docs* row). The narrower path-suffix grep is implementation-choice independent: it does **not** count `next/SKILL.md`'s new `cat ai-docs/deferred/<file>.md` reads (those are a separate design-level concern, audited via `wc -l < .claude/skills/next/SKILL.md` change vs. baseline ≥ +9). Pass: count == 2; otherwise fail. |

## Open questions

- **None blocking design.** The meta-plan resolved every design-affecting question across three opus-subagent reviews. The single residual ambiguity — whether `widget-backlog.md`'s 5 emoji statuses should *all* surface to `/next` or only `🟡 v2` — is resolved in Key Decisions above (only `🟡 v2`); the others (`✅ first-pass` / `🤔 undecided` / `❌ dropped` / `📭 future`) are not actionable backlog and don't belong in the *Candidates needing `/triage`* list. If implementation reveals a counter-example, the design phase can revisit via Design Amendment.
