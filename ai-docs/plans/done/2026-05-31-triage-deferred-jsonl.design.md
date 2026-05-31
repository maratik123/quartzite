# Design: Triage tooling unblock + deferred-store migration to JSONL

**Issue:** #596
**Date:** 2026-05-31

## Approach

Two bodies of work in one PR (per spec Scope + Key-decisions "Phase boundary"): a
Phase-1 tooling/permission unblock and a Phase-2 markdown→JSONL migration of the
`ai-docs/deferred/**` store. Phase 1 / Phase 2 are handoff groupings only, not a
delivery boundary.

This is an instruction-file / data-file task — **no Rust source**, so no `cargo`
gate is engaged (AC9). The "code" being changed is markdown instruction files plus
a one-shot data conversion. There is no compiled test surface; verification is by
deterministic `jq`/`wc -l` reconciliation and `grep` invariants (see § Test Design).

### Storage-layout decisions (Open-questions resolved)

The spec hands three sub-decisions to this phase. Resolved:

1. **Per-theme `.jsonl` files, NOT a single `items.jsonl`.** Chosen because:
   - `/next` currently iterates file-by-file (`next/SKILL.md` lines 22–55, one
     `cat` block per file). Per-theme files keep that loop shape — each `cat`
     becomes a `jq` over the same per-theme path; the iteration structure and the
     per-file source-of-truth locality are preserved (minimal-diff, AC8-friendly).
   - `/triage` Phase 6 "sort" routes a drained row to a *user-chosen thematic
     file* via a numbered menu (`triage/SKILL.md` line 54). Per-theme files keep
     that menu 1:1 with physical files; a single discriminated file would force the
     menu to mean "set `.theme`" rather than "pick a file", a larger UX-shaped
     change the spec forbids ("phases / UX / prompts unchanged", AC5).
   - Per-theme line counts map directly onto the AC7 reconciliation table
     (`wc -l <theme>.jsonl` == index count), making AC7 a one-command check.
   - Rejected single-file: would collapse 8 reconciliation targets into one and
     require every `/next` and sort operation to carry a `theme`/`kind` filter.
   - File names mirror current basenames with `.jsonl`: `signals-slots.jsonl`,
     `properties.jsonl`, `macros-codegen.jsonl`, `object-tree.jsonl`,
     `threading-runtime.jsonl`, `future-crates.jsonl`, `ci-docs-workflow.jsonl`,
     `python.jsonl`, `widget-backlog.jsonl`.

2. **`_inbox.jsonl` kept physically separate** from the thematic store (spec
   default lean: yes). It is the one store written by both `/task` Step 12 and
   `/triage`; keeping it a distinct file makes the write-AXIOM boundary physically
   obvious and lets the drain step (`triage/SKILL.md` line 48) keep targeting one
   file. It is excluded from the `/next` sweep and from the AC7 thematic count
   (unchanged from today, where `_inbox.md` is excluded from both).

3. **JSON key set (resolved against AC7 lossless reconciliation).** Investigation
   showed three distinct row shapes in the live corpus, NOT two:

   **(a) Thematic rows** — the 8 thematic files AND the `## Topic-area follow-ups`
   section of `widget-backlog.md` (136 rows; same 4-column `Item|Source|Status|Tracked`
   schema — verified at `widget-backlog.md:152` onward). Keys:
   ```
   {"item": <str>, "source_label": <str>, "source_path": <str>,
    "status": <str>, "tracked": <str>}
   ```
   - `item` — verbatim cell-1 text including markdown formatting and any literal
     `|` (which appears as `\|` in markdown; **the JSON value stores the raw `|`,
     the `\|` escaping is a markdown-table artefact that does NOT survive into
     JSON** — JSON has no pipe-escaping, the byte is just `|`).
   - `source_label` / `source_path` — the two halves of cell 2. Cell 2 is either a
     markdown link `[label](path)` or a bare path (both forms exist — e.g.
     `future-crates.md:48` is bare `ai-docs/plans/done/…`). Split into label+path;
     for bare-path cells `source_label` = the path basename-derived label per the
     existing `inbox-propagation.md` label convention, OR `source_label: null` +
     keep the bare path in `source_path` (decision: **`source_label: null` for
     bare-path cells**, lossless and unambiguous on round-trip).
   - `status` — cell-3 verbatim (`""` / `✅ done` / etc.). Empty cell → `""`.
   - `tracked` — cell-4 verbatim string: `—`, `#48`, `#49 (closed)`, `untracked`,
     or a **multi-issue** string like `#45 (closed), #46 (closed), #47 (closed)`
     (verified `future-crates.md:9,46`). Stored as the **raw string**, NOT parsed
     into an array — preserves the exact "(closed)" annotations and comma layout
     the bridge sweep reads (`triage/SKILL.md` line 61–67). Empty cell-4 (open-
     questions rows whose 4th column is blank, e.g. `signals-slots.md:45`) → `""`.

   **(b) Widget rows** — `widget-backlog.md` sections 1–8 only (46 rows; the
   `Widget|Status|Notes` 3-column schema). Keys:
   ```
   {"kind": "widget", "widget": <str>, "emoji_status": <str>, "notes": <str>}
   ```
   - `widget` — cell-1 verbatim (e.g. `` `ProgressBar` ``).
   - `emoji_status` — cell-2 verbatim (`✅ first pass` / `🟡 v2` / `🤔 undecided`
     / `❌ dropped` / `📭 future`).
   - `notes` — cell-3 verbatim, **including the inline `tracked: #N — …` token**
     when present (the tracked-state lives inside Notes for widget rows, not a
     separate column — verified `widget-backlog.md:26,35–37`). `/triage` reads it
     by substring scan; keeping it inline preserves that contract verbatim.
   - Thematic rows carry NO `kind` key; the absence of `kind` discriminates
     thematic from widget. (`jq 'select(.kind != "widget")'`.)

   **(c) `_inbox.jsonl` rows** — the 4-column `Item|Source|Section|Tracked`
   schema. Keys (adds `section`, drops `status`):
   ```
   {"item": <str>, "source_label": <str>, "source_path": <str>,
    "section": <str>, "tracked": <str>}
   ```
   `section` ∈ {`out-of-scope`, `deferred`, `open-question`} (the existing
   `inbox-propagation.md` § per-row-mapping section-key tokens, unchanged).

   **Prose / non-row content is dropped on migration** (it is not data): file H1
   titles, `## Status legend`, `## Tracking`, `## Cross-references`, the
   `deferred-items.md` index. The JSONL files carry data rows only. Any prose a
   human needs (e.g. the widget status legend) is preserved by leaving it in the
   instruction-file commentary, NOT in the data store — but per spec AC4
   ("markdown tables … removed"), the legend prose is migration-incidental; we
   drop it and rely on `emoji_status` being self-describing alongside a one-line
   legend comment we add to `next/SKILL.md`'s classification block (already
   documents the emoji semantics at lines 108–109).

### `jq` query forms (baked into instructions, AC5)

- Untracked candidates (thematic): `jq -c 'select(.tracked=="—")' <theme>.jsonl`
- Untracked widget candidates: `jq -c 'select(.emoji_status=="🟡 v2")' widget-backlog.jsonl`
- Per-theme count (AC7): `wc -l <theme>.jsonl` (one row per line, exact).
- Tracked-ref harvest for the bridge sweep: `jq -rc 'select(.tracked|test("#[0-9]+")) | .tracked' <theme>.jsonl`
- Dedupe source set (file-level): `jq -rc '.source_path' <all-thematic>.jsonl | sort -u`
- Append (Step 12 / triage sort): the producer composes one JSON object and
  appends it as one line — `printf '%s\n' "$line" >> _inbox.jsonl` is a redirect;
  to stay clear of the no-`>` posture in *triage* instructions, the append uses
  the `Write`-equivalent read-modify-write **only inside triage**; `/task` Step 12
  is a `/task` instruction (not a *triage* instruction) and may use `>>` append —
  the no-`>` rule (AC2) is scoped to triage instructions only (spec Tech
  constraint: "No `>` file-redirect in any **triage** instruction"). See Risk R5.

### Phase-1 tooling specifics

- **allow-list (`.claude/settings.json`)** add `Bash(jq *)`, `Bash(awk *)`,
  `Bash(sort *)` to the `allow` array (lines 100–113). `jq` is already trusted
  (used 9× in hook commands); `awk`/`sort` are the other tools the reworked
  recipes + reconciliation invoke. `wc` is needed too (AC7 count) — add
  `Bash(wc *)`.
- **`triage-runner` `Write` capability.** The front-matter has **no `tools:`
  line** (verified — it inherits the default tool set). The spec says to "give
  the subagent a `Write` capability". Two correct mechanisms: (i) the allow-list
  `Write(.claude/**)` / `Write(./**)` already permits writes repo-wide, so the
  *permission* exists; (ii) the subagent's declared mutation scope (front-matter
  `description` + body line 9) must explicitly name `Write` to
  `ai-docs/triage/umbrella-<N>.body.md`. Decision: **add an explicit
  `tools:` line** to `triage-runner.md` front-matter enumerating its tool set
  including `Write` (matches the pattern in `design-review.md`/`spec-writer.md`
  which DO declare `tools:`), AND update the mutation-scope prose (body line 9)
  to name the umbrella-body write. This makes AC1 verifiable by reading the
  front-matter, not by inferring from the global allow-list.
- **Phase 7.5 bare-`>` rework** (`triage-runner.md` lines 319, 330). Replace:
  - line 319 `gh issue view <N> --json body --jq .body > /tmp/triage-umbrella-<N>.body.md`
  - line 330 `--body-file /tmp/triage-umbrella-<N>.body.md` + tmp cleanup
  with: read body into a shell variable (`body=$(gh issue view <N> --json body --jq .body)`),
  `Write` it to `ai-docs/triage/umbrella-<N>.body.md`, `gh issue edit <N>
  --body-file ai-docs/triage/umbrella-<N>.body.md`, `rm -f` the file. No `>`
  anywhere in the recipe. (Note the `--jq` flag is `gh`'s own JSON extraction,
  not a shell pipe to `jq`, and not a `>` redirect — it stays.)

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **Phase-1 allow-list + `Write` capability.** Add `Bash(jq *)`, `Bash(awk *)`, `Bash(sort *)`, `Bash(wc *)` to `.claude/settings.json` `allow`. Add explicit `tools:` front-matter line to `triage-runner.md` including `Write`; update its mutation-scope prose (body ~line 9) to name `ai-docs/triage/umbrella-<N>.body.md`. (AC1) | `.claude/settings.json`, `.claude/agents/triage-runner.md` | — |
| 2 | **Phase-1 bare-`>` rework + gitignore.** Rework `triage-runner.md` Phase 7.5 recipe (lines ~319, ~330) to variable→`Write`→`gh issue edit --body-file`→`rm -f`; no `>` redirect remains. Widen `.gitignore` to cover `ai-docs/triage/**/umbrella-*.body.md` (currently only `*.progress.md` is ignored — verified leak). (AC2) | `.claude/agents/triage-runner.md`, `.gitignore` | 1 |
| 3 | **Phase-1 Triage-group propagation (recipe + tooling refs).** Reconcile `triage/SKILL.md` `allowed-tools` (add `Bash(jq *)` `Bash(awk *)` `Bash(sort *)` `Bash(wc *)` `Write`), and ensure the reworked recipe + tool references are mutually consistent across `triage/SKILL.md`, `triage-runner.md`, `next/SKILL.md`. (AC3) | `.claude/skills/triage/SKILL.md`, `.claude/agents/triage-runner.md`, `.claude/skills/next/SKILL.md` | 1, 2 |
| 4 | **Phase-2 one-shot lossless conversion + AC7 reconciliation.** Write a throwaway conversion (run-once, not committed as a tool) that reads each markdown file and emits the per-theme `.jsonl` per the § Approach key set, handling the three row shapes + ragged/escaped-pipe/multi-issue edge cases. Produce `signals-slots.jsonl` … `python.jsonl`, `widget-backlog.jsonl` (46 widget + 136 thematic rows), `_inbox.jsonl` (0 data rows today). Reconcile: `wc -l` per theme == index counts (30/23/44/10/59/90/358/6); widget-backlog == 46+136=182; assert every source `\|`/ragged/multi-issue row maps to exactly one line. (AC4 partial, AC7) | `ai-docs/deferred/*.jsonl` (new) | 3 |
| 5 | **Phase-2 delete markdown corpus.** `git rm` the 9 thematic+widget `.md` files, `_inbox.md`, and `ai-docs/deferred-items.md` after Task 4's `.jsonl` reconcile passes. (AC4) | `ai-docs/deferred/*.md`, `ai-docs/deferred-items.md` (deleted) | 4 |
| 6 | **Phase-2 rewrite read/write mechanics in `/next` + `/triage`.** Replace `/next`'s 9 `cat` blocks with per-theme `jq` reads producing identical *Candidates needing /triage* output (AC8). Rewrite `/triage` SKILL + runner sweep/drain/bridge to read `tracked`/`status`/`emoji_status`/`notes` from JSONL via baked-in `jq` one-liners; phases/UX/prompts unchanged (AC5). Propagate across Triage group. | `.claude/skills/next/SKILL.md`, `.claude/skills/triage/SKILL.md`, `.claude/agents/triage-runner.md` | 5 |
| 7 | **Phase-2 `/task` Step 12 + inbox-propagation sink swap.** Change Step 12 sub-step 5 + `inbox-propagation.md` § per-row-mapping so the emitted sink is one appended JSON line to `_inbox.jsonl` (keys per § Approach (c)); the six spec-shape parse rules + file-level dedupe (now over `.source_path` via `jq`) are **unchanged**. Update Step 12 sub-step 8 staging list (`_inbox.md`→`_inbox.jsonl`). (AC5) | `.claude/skills/task/SKILL.md`, `.claude/skills/task/inbox-propagation.md` | 5 |
| 8 | **Phase-2 AGENTS.md references (literal `.md`→`.jsonl` sweep).** Update the `_inbox` write AXIOM (`.md`→`.jsonl`; strengthen no-hand-edit rationale tersely: JSONL is hand-edit-hostile), the `§ Agent Docs` rows for `_inbox` + triage skill, and any `.md`-filename refs that now name `.jsonl`. **Done-gate:** every `_inbox.md`→`_inbox.jsonl` literal occurrence is swept — the `§ Workflow` `_inbox` write-AXIOM block (lines ~187–194) AND the `§ Agent Docs` `ai-docs/deferred/_inbox.md` row (~line 276) — verify via `grep -n '_inbox\.md' AGENTS.md` returning **zero** hits post-edit. **No `wc -c AGENTS.md` char-count gate** (size reduction is out of scope per the amended spec; see Risk R3). (AC6) | `AGENTS.md` | 5 |

(8 tasks; 1 over the 7-task soft limit. Splitting is NOT proposed because Tasks 1–3
and 4–8 are the two mandated phases of a single-PR deliverable per the spec's "Phase
boundary" decision — they cannot ship as separate issues. The count is a consequence
of the propagation fan-out, each task is atomic.)

## Handoff plan

Per `/task` Step 8 + `.claude/skills/task/reference.md` § *Every-group handoff
(rationale)*: a `/context-reset` handoff binds at the **start of every group**,
including the first. **(a)** Grouping is required for every M ≥ 1 (here M = 8).
**(b)** Non-terminal groups are exactly **3 consecutive subtasks**. **(c)** Handoff
destination at every boundary is **`/context-reset` per
`.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry)**. **(d)**
The terminal group size is within `1..=3`.

M = 8 ⇒ three groups (3 + 3 + 2):

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry) before starting subtask 1.
- **Group A:** subtasks 1–3 — Phase-1 tooling/permission unblock + Triage-group
  propagation (3 subtasks; non-terminal, exactly 3).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh
  context.
- **Group B:** subtasks 4–6 — JSONL conversion + reconciliation, markdown deletion,
  `/next` + `/triage` read/write rewrite (3 subtasks; non-terminal, exactly 3).
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry). Parent `/task` resumes in Group C with fresh
  context.
- **Group C:** subtasks 7–8 — `/task` Step 12 sink swap + AGENTS.md references/size
  (terminal group; 2 subtasks, within the 1..=3 range).

## Risks

- **R1 — Lossless conversion is the highest-risk subtask (AC7).** Three distinct
  row shapes (thematic / widget / `_inbox`), plus ragged columns (3-col rows where
  `Status` is omitted — verified `future-crates.md:14`, `macros-codegen.md:16–18`,
  86 such rows across files via pipe-count `4 pipes`), escaped pipes (`\|` inside
  `item` — verified in 7 files (object-tree, macros-codegen, threading-runtime,
  signals-slots, future-crates, ci-docs-workflow, properties); the worst is
  `ci-docs-workflow.md:93` with 8
  pipes), and multi-issue `tracked` cells (`future-crates.md:9,46`). *Mitigation:*
  (i) the converter splits each data row on **unescaped** `|` only (a markdown
  table cell boundary is `|` not preceded by `\`), then unescapes `\|`→`|` inside
  each cell; (ii) ragged rows are handled by positional fill — 4-col → all keys;
  3-col thematic → `status: ""` and the 3rd cell is `tracked`; widget 3-col →
  widget keys; (iii) reconcile `wc -l` against the exact index counts
  (30/23/44/10/59/90/358/6 verified by my `awk` data-row scan matching the index
  precisely) BEFORE Task 5 deletes anything; (iv) spot-check the 4 known-hard rows
  (8-pipe row, the two multi-issue rows, a bare-path source row) round-trip to one
  line each.
- **R2 — widget-backlog dual schema (verified).** `widget-backlog.md` is NOT a
  single-schema file: sections 1–8 (46 rows; count verified live during
  implementation — the design's earlier 47 was a stale section-accounting
  off-by-one) are `kind:"widget"`; the
  `## Topic-area follow-ups` section (136 rows, line 152+) is the standard 4-column
  thematic schema. The spec's `kind:"widget"` description covers only the first.
  *Mitigation:* the converter emits widget keys for rows under sections 1–8 and
  thematic keys (no `kind`) for `## Topic-area follow-ups` rows; AC7's "plus
  widget-backlog rows" reconciles to **46 + 136 = 182** total lines in
  `widget-backlog.jsonl`. `/next` + `/triage` classification (which already special-
  cases widget-backlog by emoji vs `tracked: #N` in Notes) must read `emoji_status`/
  `notes` for `kind:"widget"` rows and `tracked` for the no-`kind` rows in the same
  file.
  - **AC8 candidate-set-neutrality invariant (the explicit safety condition for the
    reclassification — DO NOT silently change the candidate set).** The 136
    `## Topic-area follow-ups` rows move from single-file `/next` handling (keyed on
    the `Status` emoji) to thematic handling (`kind`-absent, keyed on `tracked=="—"`).
    This is candidate-set-neutral ONLY because those 136 rows have an **empty `Status`
    column** AND **zero `—` values in their `Tracked` column** — verified live: 136
    data rows, `Status` non-empty count = 0, `Tracked` distribution is `#N` /
    `#N (closed)` / `untracked` / `tracked: #592` only, em-dash count = 0. So they
    surface as candidates under NEITHER the old `Status`-emoji rule NOR the new
    `tracked=="—"` rule, and the `/next` candidate set is provably unchanged (AC8). A
    future implementer who alters either condition (populates `Status`, or introduces
    a `—` into `Tracked`) breaks this invariant and MUST re-verify AC8 before merge.
- **R3 — AGENTS.md is already 38,296 chars, OVER the 35,000 early-warning cap and
  approaching the 40,000 harness hard cap — a PRE-EXISTING condition this task does
  NOT fix.** Per the amended spec (Out of scope + Tech constraint), reducing
  AGENTS.md below the early-warning cap is **OUT OF SCOPE** (deferred to a dedicated
  `/ai-audit` session, user decision 2026-05-31). There is **no char target and no
  done-gate on `wc -c AGENTS.md`** for Task 8. *Mitigation:* Task 8 must simply
  **avoid gratuitous growth** — the `.md`→`.jsonl` literal renames are roughly
  net-neutral; keep any strengthened no-hand-edit rationale terse (one clause, not a
  new paragraph). No "extract verbose sections if infeasible" contingency is required;
  do NOT perform an extraction pass under this task. (If the edit happens to nudge the
  count toward 40,000, note it for the future `/ai-audit` row, but it does not gate
  this PR.)
- **R4 — Triage-group propagation completeness (AC3).** Any touch to
  `triage/SKILL.md` / `triage-runner.md` / `next/SKILL.md` requires all three
  checked (AGENTS.md Propagation Rule). Both Phase 1 (Tasks 1–3) and Phase 2 (Task
  6) touch this group. *Mitigation:* after Tasks 3 and 6, run
  `grep -rn '\.md\b' .claude/skills/triage/SKILL.md .claude/agents/triage-runner.md
  .claude/skills/next/SKILL.md` to confirm no stale `deferred/*.md` reference
  survives, and confirm tool/recipe references are mutually consistent.
- **R5 — no-`>` scope is triage-only (AC2).** The spec constraint is "No `>`
  file-redirect in any **triage** instruction". `/task` Step 12's `_inbox.jsonl`
  append legitimately uses `>>`; that file is a `/task` instruction, not a triage
  instruction, so it is out of AC2 scope. *Mitigation:* the AC2 grep targets only
  `triage/SKILL.md` + `triage-runner.md` (spec AC2 names exactly those two files);
  do NOT widen it to `/task`.
- **R6 — bare-path vs markdown-link source cells.** Cell 2 has two forms; null-
  label for bare paths is lossless but the bridge's design-work classifier reads
  the source filename from either form (`triage/SKILL.md` line 101). *Mitigation:*
  the classifier consumes `source_path` (always present, both forms) — keep that
  the single source-of-truth field; `source_label` is display-only.

## Test Design

No Rust compile/test surface (AC9 — instruction/data only). Verification is
deterministic shell assertions, run during Tasks 4, 6, 7 and at PR self-review:

- **AC7 reconciliation (Task 4) — the gating test.**
  - Entry point: the produced `.jsonl` files.
  - Scenarios:
    - Happy: `for f in signals-slots properties macros-codegen object-tree
      threading-runtime future-crates ci-docs-workflow python; do echo "$f
      $(wc -l < ai-docs/deferred/$f.jsonl)"; done` matches 30/23/44/10/59/90/358/6.
    - Widget: `wc -l ai-docs/deferred/widget-backlog.jsonl` == 182 (46 widget +
      136 thematic); cross-check `jq -c 'select(.kind=="widget")' | wc -l` == 46 and
      `jq -c 'select(.kind!="widget")' | wc -l` == 136.
    - Validity: every line is valid JSON — `jq -e . <file> >/dev/null` per file
      exits 0 (catches unescaped quotes / broken splits).
    - Edge: the 8-pipe row (`ci-docs-workflow.md:93`), the two multi-issue
      `tracked` rows, a bare-path-source row, and an empty-`status` ragged row each
      produce exactly one line with the expected field values (manual spot-check).
  - Fixtures: the pre-deletion markdown files (Task 5 deletes only after this
    passes).
- **AC8 no-regression (Task 6).**
  - Entry point: `/next` *Candidates needing /triage* section output.
  - Scenario: capture `/next` candidate-section output against the markdown corpus
    (before Task 5) and against the JSONL corpus (after Task 6); assert byte-
    identical candidate set (same items, same source citations, same ordering).
  - Fixture: the candidate set is `tracked=="—"` thematic rows + `emoji_status=="🟡 v2"`
    widget rows — enumerate both before/after and diff.
- **AC2 no-redirect (Task 2/3).** `grep -rn '>' .claude/skills/triage/SKILL.md
  .claude/agents/triage-runner.md` shows no `>` file-redirect in a command recipe
  (the `gh --jq` and `2>/dev/null`/`>&2` forms, if any, are not file-redirects of
  command stdout to a path — inspect each hit).
- **AC6 reference sweep (Task 8).** `grep -n '_inbox\.md' AGENTS.md` returns **zero**
  hits after the edit — every `_inbox.md`→`_inbox.jsonl` literal in the `§ Workflow`
  write-AXIOM block (lines ~187–194) and the `§ Agent Docs` row (~line 276) is swept.
  **No `wc -c` char-count assertion** (size reduction is out of scope; see R3).
- **AC1 (Task 1).** `triage-runner.md` front-matter `tools:` includes `Write`;
  `.claude/settings.json` `allow` includes `jq`/`awk`/`sort`/`wc`.

## Open questions

All three spec-handed sub-decisions are resolved above (per-theme files;
`_inbox.jsonl` separate; the 3-shape key set). No open questions remain.

The former AGENTS.md size-pressure open question is **closed by the amended spec**:
instruction-file size reduction is OUT OF SCOPE (deferred to a future `/ai-audit`
session, user decision 2026-05-31). AGENTS.md being 38,296 chars (over the 35,000
early-warning cap) is a pre-existing condition this task neither fixes nor gates on.
Task 8 only sweeps the `_inbox.md`→`_inbox.jsonl` literals and avoids gratuitous
growth (see Risk R3).
