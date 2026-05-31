# Triage tooling unblock + deferred-store migration to JSONL

**Source:** issue #596
**Date:** 2026-05-31
**Tracked in:** #596

## Scope

Two bodies of work against the `/triage` workflow and the deferred-item store, organised as Phase 1 (tooling/permission unblock) and Phase 2 (JSONL migration). **Both phases ship in a single PR** — the user explicitly overrode the issue's separate-PRs note (round-1 Q&A). Phase 1 / Phase 2 remain as logical groupings for the design phase's handoff plan, not as a delivery boundary.

### Phase 1 — Unblock triage tooling (tooling/permission)

1. Widen `.claude/settings.json` allow-list to cover the data tools the triage workflow needs but that are not currently allow-listed — at minimum `jq` (already trusted: used 9× inside `settings.json` hook commands). Add the others the workflow actually invokes (the design phase enumerates the exact set from the runner's reworked recipes; candidates named in the issue are `jq`, `awk`, `sort`).
2. Give the `triage-runner` subagent a `Write` capability (its front-matter `tools:` list currently lacks `Write`; its mutation scope already nominally includes `ai-docs/deferred/**` + `ai-docs/triage/**`).
3. Replace the bare-redirect umbrella-body recipe so no `>` file-redirect remains in the triage instructions. Current offending recipe (`triage-runner.md` Phase 7.5, around the `gh issue view <N> --json body --jq .body > /tmp/triage-umbrella-<N>.body.md` line, and its companion `gh issue edit … --body-file /tmp/…` + tmp cleanup): rework to
   - read the body into a variable,
   - `Write` it to `ai-docs/triage/umbrella-<N>.body.md` (inside the subagent's existing mutation scope; `ai-docs/triage/**` is gitignored),
   - `gh issue edit <N> --body-file ai-docs/triage/umbrella-<N>.body.md`,
   - `rm -f` the file.
   No `>` redirect anywhere in the reworked recipe.
4. Propagate the Phase-1 instruction edits across the **Triage group** per the AGENTS.md Propagation Rule: `.claude/skills/triage/SKILL.md`, `.claude/agents/triage-runner.md`, `.claude/skills/next/SKILL.md` — any of the three touched ⇒ all three checked/updated.

### Phase 2 — Migrate deferred store to canonical JSONL; drop markdown

5. Make `ai-docs/deferred/**` canonical **JSONL**: one JSON object per line. Thematic rows carry fields `{id, theme, item, source_label, source_path, section, status, tracked}`; widget-backlog rows fold in as `{kind:"widget", widget, emoji_status, notes, …}`. (Exact key set + the single-vs-per-theme file layout are design-phase decisions — see Open questions.)
6. **Delete** the markdown tables and `ai-docs/deferred-items.md` entirely. Decision (issue-recorded): JSONL-only, no generated markdown views.
7. Rewrite the read/write mechanics — phases, UX, and user prompts unchanged — in `/triage` (`triage/SKILL.md` + `triage-runner.md`), `/next` (`next/SKILL.md`), and `/task` Step 12 (`task/SKILL.md` + `task/inbox-propagation.md`'s per-row mapping). Each query becomes a baked-in `jq` one-liner. Reference query forms from the issue:
   - untracked candidates: `jq 'select(.tracked=="—")'`
   - per-theme counts: `jq -s 'group_by(.theme)|map({(.[0].theme):length})'`
   - dedupe by source; rewrite `tracked` to `#N` / `untracked`.
8. `/task` Step 12's output sink changes from a 4-cell markdown row to one appended JSON line. The **six spec-shape parse rules in `inbox-propagation.md` are unchanged** — only the emitted sink changes. The file-level dedupe against thematic `source_path`s is preserved.
9. Update **AGENTS.md**: the `_inbox` write AXIOM (JSONL is hand-edit-hostile — strengthens, not weakens, the no-hand-edit rule) and every deferred-store reference (`§ Workflow` AXIOM block, the two `§ Agent Docs` table rows for `ai-docs/deferred/_inbox.md` and the triage skill, plus any `.md`-filename references that now name `.jsonl`).
10. One-shot lossless markdown → JSONL conversion of the existing corpus (~1070 rows / 10 files). Verify by row-count reconciliation against the current `deferred-items.md` counts (see AC table). Every ragged-column / escaped-pipe / multi-issue-cell row maps to exactly one JSON line.

## Out of scope

- A markdown-parsing helper CLI — explicitly **rejected** in the issue as strictly dominated by the JSONL migration.
- Re-litigating the storage format. JSONL is decided (rationale recorded below; do not revisit YAML/TOML).
- Any change to the six `inbox-propagation.md` spec-shape parse rules.
- Any change to triage phase structure, UX, or user-facing prompts (mechanics only).
- A `/triage --backfill-design-link` pass or any other deferred sub-feature mentioned in the runner.
- Reducing AGENTS.md below the 35,000-char early-warning cap — deferred to a dedicated `/ai-audit` session (user decision, 2026-05-31).

## Deferred
- (none — both phases are in scope; only the cross-phase PR boundary is a delivery constraint, not deferral)

## Key decisions

| Question | Decision |
|---|---|
| Storage format | **JSONL** (one JSON object per line). Append is line-local (1-line blast radius); single-field mutation is 1-line; clean diffs/merges; no YAML type coercion ("Norway problem", `#`→comment) — critical since `tracked` values include `—`, `#60 (closed)`, emoji, leading-backtick; `jq` already used 9× in `settings.json` hooks; `wc -l` gives a cheap count. JSON ⊂ YAML, so JSONL does not foreclose YAML tooling later. Rejected: YAML (coercion danger; readability advantage moot once markdown views are dropped), TOML (same multi-line-record drawback). |
| Markdown views | Dropped. No generated markdown alongside the JSONL; `deferred-items.md` removed. |
| Helper CLI | Rejected (strictly dominated). |
| Parse rules | The six `inbox-propagation.md` shape rules are unchanged; only the output sink (markdown row → JSON line) changes. |
| Phase boundary | **Single PR** for both phases (round-1 Q&A; overrides the issue's separate-PRs note). Phase 1 / Phase 2 are design-phase handoff groupings, not a delivery boundary. |

## Technical constraints

- **Contracts that any change MUST preserve:**
  - `_inbox` (`_inbox.jsonl` after Phase 2) is written ONLY by `/task` Step 12 and `/triage` (AGENTS.md AXIOM) — no hand-edits. JSONL reinforces this (hand-edit-hostile).
  - `/task` Step 12's six spec-shape parse rules + file-level dedupe against thematic `source_path`s.
  - `/next` must still surface untracked candidates in its *Candidates needing `/triage`* section, with output identical before/after the migration.
  - Triage-group Propagation Rule: any edit to `triage/SKILL.md` / `triage-runner.md` / `next/SKILL.md` touches all three.
- **No `>` file-redirect** in any triage instruction after Phase 1 (the static-analysis permission-prompt trigger).
- AGENTS.md instruction-file size: AGENTS.md is **already over** the 35,000-char early-warning cap at baseline (38,296 chars) — a pre-existing condition independent of this task. Bringing it back under the cap is **OUT OF SCOPE** here, deferred to a future `/ai-audit` pass. This task's AGENTS.md edits should simply avoid gratuitous growth (the `.md`→`.jsonl` renames are roughly net-neutral) but are **NOT gated on a char target**.
- Each modified workflow file under `.github/workflows/` (if any) must pass `actionlint` before `git add` — not expected to be touched here, but flagged per AGENTS.md.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | **Phase 1:** `triage-runner` can run the data tools added to the allow-list (`jq` at minimum) and `Write` without permission prompts (`triage-runner` front-matter `tools:` includes `Write`; allow-list widened in `.claude/settings.json`). |
| AC2 | **Phase 1:** No bare `>` file-redirect remains anywhere in the triage instructions (`grep -rn '>' .claude/skills/triage/SKILL.md .claude/agents/triage-runner.md` shows no file-redirect in a command recipe); the umbrella-body recipe uses `Write` to `ai-docs/triage/umbrella-<N>.body.md` → `gh issue edit --body-file` → `rm -f`. |
| AC3 | **Phase 1:** Triage-group propagation complete — the three files are mutually consistent on the reworked recipe + tooling references. |
| AC4 | **Phase 2:** `ai-docs/deferred/**` is canonical JSONL; the markdown tables and `ai-docs/deferred-items.md` are removed from the tree. |
| AC5 | **Phase 2:** `/triage` (SKILL + runner), `/next`, and `/task` Step 12 read/write the JSONL store via baked-in `jq` one-liners; phases / UX / prompts are unchanged. |
| AC6 | **Phase 2:** AGENTS.md `_inbox` write AXIOM and all deferred-store references are updated to JSONL. |
| AC7 | **Phase 2 — lossless reconciliation:** per-theme JSON line counts match the current `deferred-items.md` table — Signals & Slots 30, Properties 23, Macros & Codegen 44, Object Tree 10, Threading & Runtime 59, Future Crates 90, CI/Docs/Workflow 358, Python 6, plus widget-backlog rows. Every ragged-column / escaped-pipe / multi-issue-cell markdown row maps to exactly one JSON line. |
| AC8 | **No regression:** `/next` "Candidates needing `/triage`" output is identical before and after the migration. |
| AC9 | The PR is instruction-file / data-file only (no Rust source change); `cargo` gates unaffected. |

## Open questions

The issue explicitly hands these two storage-layout sub-decisions to the `/task` design phase (not spec-blocking — defensible defaults exist):

- **Single `items.jsonl` (discriminated by `theme` / `kind`) vs. per-theme `.jsonl` files.** Default lean: per-theme files preserve the current one-file-per-theme locality and keep `/next`'s per-file iteration loop close to its current shape; the design phase chooses.
- **Keep `_inbox.jsonl` physically separate** from the thematic store, to make the write-AXIOM boundary (only `/task` Step 12 + `/triage` may write `_inbox`) physically obvious. Default lean: yes, keep separate.
- Exact JSON key set per row kind (thematic vs `kind:"widget"`) — design phase finalises against the lossless-reconciliation requirement (AC7).
