# Design: `/triage` skill — batched promotion + `_inbox.md` drain + widget-backlog source

**Issue:** [#204](https://github.com/maratik123/quartzite/issues/204)
**Date:** 2026-05-11
**Spec:** [`2026-05-11-triage-skill.spec.md`](2026-05-11-triage-skill.spec.md)
**Branch:** `feat/2026-05-11-triage-skill` (already created per AGENTS.md AXIOM 1)

## Approach

### Chosen solution

Ship `/triage` as a **two-file prompt artefact** mirroring the `/improve` skill +
`self-improve` agent precedent, plus the four-edit sync-group propagation the
spec locks in:

1. **`.claude/skills/triage/SKILL.md`** — thin launcher with the spec's
   frontmatter verbatim. Body documents the trigger, the cell-iteration sweep,
   the `_inbox.md` drain, a *Bridge* placeholder Issue C will fill, and the
   run-output summary. The body launches `triage-runner` in a single line and
   delegates all behaviour. Same shape as `improve/SKILL.md`.
2. **`.claude/agents/triage-runner.md`** — `model: opus` subagent. Same five-
   stage shape as `self-improve.md` (read inputs → identify candidates → draft
   diffs → present batch → apply after confirmation), with a sixth `_inbox.md`
   drain phase carved out for per-entry handling, and the `deferred-items.md`
   row-count update + run-summary phase at the end. Mutation scope is strictly
   `ai-docs/deferred/**` + `gh issue create / edit` calls — every other write
   is forbidden.
3. **`AGENTS.md` *Agent Docs* table + *Propagation Rule* table** — two new
   rows. Adds the skill ↔ agent ↔ `next/SKILL.md` sync-group entries and
   documents both new files under *Agent Docs*.
4. **`.claude/skills/improve/SKILL.md`** — one-line cross-reference noting the
   shared batched-approval + threshold-trigger patterns and the divergence in
   mutation scope.

### Why this shape

- **Precedent.** `/improve` + `self-improve` are the canonical
  "human-supervised batched-mutation skill" pair in this repo. Mirroring the
  shape lets every contributor who has read `/improve` read `/triage` without
  context-switching. The spec's *Locked-in decisions* lock this mirror.
- **Disable-model-invocation.** `/triage` must not auto-fire — it mutates
  authoritative state (md files + gh issues). The frontmatter enforces this.
- **Opus model.** Spec calls for opus discipline (same as `/improve`); the
  decision is per-row irreversible (issue numbers, file rewrites), so the
  stronger model warrants the cost.
- **Subagent split.** The launcher / subagent split keeps the user-visible
  skill prompt small while letting the long agent file carry the workflow
  detail. `/improve` follows the same split.

### Rejected alternatives

- **Single-file skill** (skill body carries the full workflow, no subagent).
  Rejected: breaks the `/improve` mirror, makes the skill prompt huge,
  conflates user-visible documentation with agent-side execution detail.
- **Two skills** (`/triage-promote` for the sweep + `/triage-drain` for the
  inbox). Rejected: the spec locks a single `/triage` entry point; the
  batched `gh issue create` covers both flows in one bulk call, so splitting
  the skill would split the API call.
- **Per-entry approval across the sweep** (no batched table). Rejected: spec
  locks batched approval for the cell-iteration sweep; per-entry is reserved
  for `_inbox.md` drain (where every row has a destination question to
  resolve before promote/drop semantics apply).
- **Stand-alone Rust binary or shell script.** Rejected: spec lists this as a
  non-goal; `/triage` is pure prompt logic.

### Mirror map: `improve/self-improve` ↔ `triage/triage-runner`

| Aspect | `/improve` + `self-improve` | `/triage` + `triage-runner` | Divergence rationale |
|---|---|---|---|
| Skill frontmatter `disable-model-invocation` | `true` | `true` | identical |
| Skill frontmatter `argument-hint` | `"[optional context]"` | `"[N — override default threshold]"` | `/triage` accepts a numeric threshold override; `/improve` takes free-text context |
| Skill frontmatter `allowed-tools` | (none — relies on `.claude/settings.json` defaults) | enumerated narrowing: `Bash(gh issue create/edit/list/view *) Bash(gh api *) Bash(grep *) Bash(rg *) Read Edit` | `/triage` documents its intended subset for clarity per spec |
| Skill body shape | thin launcher; 8 numbered bullets describing what the subagent will do | thin launcher; sections *Trigger and threshold* → *Cell-iteration sweep* → *Inbox drain* → *Bridge (placeholder)* → *Run-output summary* — placeholder reserved so Issue C plugs in cleanly | `/triage`'s sections are richer because the skill body is also the user-facing reference for what each phase does |
| Agent frontmatter | `model: opus`, single-paragraph description | `model: opus`, single-paragraph description | identical |
| Agent workflow phases | Find patterns → Determine actions → Propose concrete changes → Escalate to hooks → Apply after confirmation → Eval (REQUIRED) | Branch check → Threshold gate → Read inputs / identify candidates → Draft titles/bodies → Bulk `gh issue list` dedupe → Present batch + collect approvals → Drain `_inbox.md` per-entry → Combined `gh issue create` pass (single bulk) → Update `deferred-items.md` + emit summary | `/triage` adds branch-check + threshold-gate preambles, defers all creates to a single contiguous Phase 7.5 pass to match the spec's "one bulk call" contract, and appends two terminal stages (drain + summary) because its mutation scope is broader; **omits eval gate** (see below) |
| Routing table inside agent | category × occurrence × current-rule-status decision matrix | row-type × `Tracked`-cell-value decision matrix | both files use action tables; the dimensions differ because the inputs differ |
| Anti-patterns section | yes — 5 bullets | yes — 5 bullets analogous to `/improve`'s | identical shape |
| Branch check before applying | `git branch --show-current`; switch if `master` | `git branch --show-current`; switch if `master` | identical — both follow AGENTS.md AXIOM 1 |

### Patterns intentionally NOT borrowed (with rationale)

1. **Post-application eval gate.** `/improve`'s Step 6 spawns a clean-context
   subagent to verify the rule landed. `/triage`'s mutations are user-in-loop
   per row: every approval IS the eval. There is no "did the silent rule
   stick" question to ask. **Where this surfaces in the design:** the agent's
   *Apply after approval* stage has no follow-up verification stage.

2. **Hook escalation at ≥3 occurrences.** `/improve` proposes hooks for
   repeated rule violations. `/triage` mutates *data* (rows + issues), not
   *rules*. A repeatedly-declined row is already marked `untracked` in cell 4
   and stops surfacing on subsequent runs — the file state IS the
   short-circuit. **Where this surfaces in the design:** the agent's decision
   matrix has no escalation column.

3. **`learnings.md`-style decision log.** `/improve` keeps an append-only log
   of corrections; `/triage`'s log is implicit in the md state itself (`#N`
   markers, `untracked` markers, `—` markers). Adding a separate decision log
   would duplicate state and invite drift. **Where this surfaces in the
   design:** the agent's *Run-output summary* phase emits a one-shot table of
   decisions for the current run, but does not persist it to a long-lived
   log.

## Decomposition

8 atomic tasks. The spec is dense enough that any larger collapse would hide
real edits; finer splits would not pay rent. Dependency order is `1 → 2 → 3
→ 4 → 5 → 6 → 7 → 8`.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create skill stub with frontmatter verbatim from the spec; body left as a TODO comment placeholder. | `.claude/skills/triage/SKILL.md` (new) | — |
| 2 | Create agent file with `model: opus` frontmatter and full workflow body (branch check → threshold gate → read inputs / identify candidates → draft → bulk `gh issue list` dedupe → present batch + collect approvals (no creates yet) → drain inbox per-entry → combined `gh issue create` Phase 7.5 (single bulk pass over union of approvals) → deferred-items.md update → summary; anti-patterns; routing table). | `.claude/agents/triage-runner.md` (new) | 1 |
| 3 | Fill the skill body — sections *Trigger and threshold* / *Cell-iteration sweep* / *`_inbox.md` drain* / *Bridge* placeholder / *Run-output summary*; one-line subagent launch. | `.claude/skills/triage/SKILL.md` | 1, 2 |
| 4 | Commit synthetic fixtures `widget-backlog-promotion.md` and `triage-decline.md` (and any helpers — see *Fixtures* below). | `tests/fixtures/process-improvements/widget-backlog-promotion.md` (new), `tests/fixtures/process-improvements/triage-decline.md` (new) | — (independent of 1–3; can land in same commit) |
| 5 | Update `AGENTS.md` *Agent Docs* table — add row for `triage/SKILL.md` + `triage-runner.md`. | `AGENTS.md` | 1, 2 |
| 6 | Update `AGENTS.md` *Propagation Rule* table + *Sync groups* prose — add `triage/SKILL.md` ↔ `triage-runner.md` ↔ `next/SKILL.md` group. | `AGENTS.md` | 5 |
| 7 | Add one-line cross-reference in `improve/SKILL.md` pointing at `/triage`, noting shared patterns + divergence. | `.claude/skills/improve/SKILL.md` | 1 |
| 8 | Run AC-verification recipes against the new files + fixtures; log results in progress doc. | (no file edits; manual verification) | 1–7 |

8 tasks ≤ 7 cap soft-line. Splitting tasks 5 + 6 (both edit `AGENTS.md`)
keeps the diffs small and lets reviewers read each table edit independently.

## Risks

- **AGENTS.md table-row insertion order.** *Agent Docs* and *Propagation
  Rule* are alphabetically/logically grouped; inserting in the wrong place
  breaks visual scan order. **Mitigation:** *Agent Docs* row sorts under the
  existing `spec-writer.md` row (the only `.claude/agents/` entry today);
  *Propagation Rule* row sorts after the existing Interview group.

- **Sync-group new entry creates a triangle `triage ↔ triage-runner ↔ next`
  but `/next` doesn't currently know it's in the group.** AGENTS.md
  Propagation Rule lists the trios as bi-directional, so an edit to
  `next/SKILL.md` (e.g. the *Candidates needing `/triage`* section text)
  must propagate. **Mitigation:** the Propagation Rule table row spells out
  all three directions (`triage/SKILL.md` → check `triage-runner.md` AND
  `next/SKILL.md`; `triage-runner.md` → check `triage/SKILL.md` AND
  `next/SKILL.md`; `next/SKILL.md` → check `triage/SKILL.md` AND
  `triage-runner.md`). This is heavier than the spec's "triage ↔
  triage-runner ↔ next" shorthand, but matches the Review group's existing
  three-row layout.

- **AGENTS.md *Workflow* section already carries the `_inbox.md` AXIOM
  (landed in A1).** The current axiom names `/triage` as a writer of a file
  that, as of A1, did not yet have a writer. With B landing, `/triage` IS
  the writer — the existing axiom prose remains correct, no edit needed.
  **Mitigation:** task 8's verification recipe greps for the axiom and
  confirms `/triage` is named as the second writer with no edit required.

- **`improve/SKILL.md` cross-reference creates a hidden second sync group
  (improve ↔ triage).** The spec calls for a one-line cross-reference, NOT a
  formal sync group. **Mitigation:** the one-liner in `improve/SKILL.md`
  flags shared patterns + divergence as informational; the *Propagation
  Rule* table does NOT add an `improve ↔ triage` row. Future maintainers
  read the cross-reference and update both files only when the shared
  pattern itself changes (not on every routine edit).

- **`triage-runner.md` agent file violates Boundary rule 2 if it cites
  learnings.md state.** The agent file describes mutating `ai-docs/deferred/`
  and gh issues — never `learnings.md`. **Mitigation:** anti-patterns list
  explicitly forbids `learnings.md` writes; mutation-scope contract is
  written into the frontmatter description.

- **Fixture `triage-decline.md` carries one thematic row AND one widget-
  backlog row in one file, which breaks the existing
  `tests/fixtures/process-improvements/` "one fixture per shape" pattern.**
  **Mitigation:** ship two-section fixture (one thematic-table section + one
  widget-backlog-table section) inside a single `.md` file; AC7's
  verification recipe greps the two sections independently. Alternative: two
  separate fixtures (`triage-decline-thematic.md` + `triage-decline-widget.
  md`). Design chooses single-file because AC7 verifies "both shapes
  declined in the same `/triage` run" — semantically one fixture, one run.

- **Concurrent-edit guard's content-snapshot check is order-sensitive across
  many files.** If the agent reads file A at session start, the user edits
  files B and C, then the agent rewrites file B, the snapshot for B is fresh
  (re-read immediately before rewrite) and the guard fires correctly. But if
  the agent batches reads + rewrites of the same file, the second rewrite
  sees the *first rewrite's* content as the new baseline. **Mitigation:**
  the agent's *Apply* phase re-reads each file once per rewrite, not once
  per session. The snapshot baseline is the most-recent on-disk content
  immediately before the next write.

- **`gh issue list --limit 500` is one call; pagination watchdog warns at
  ≥0.9× = 450 issues.** Current corpus is 64. **Mitigation:** if the
  response length is ≥450, the agent stops the run before any `gh issue
  create` and prints a one-line message recommending either `--limit 1000`
  or a pagination patch in a follow-up issue. Watchdog text is verbatim in
  the agent file.

- **`widget-backlog.md` parser may match the prose hit at line 89 (`> spec.
  Tracked: TBD …`).** **Mitigation:** the agent's *Identify candidates*
  phase anchors on column-header context — it must see a row inside an
  actual `| Widget | Status | Notes |` table; bare `Tracked:` substrings in
  prose are filtered. Same anchor rule `/next` already uses (per Issue A1).

## Test Design

`/triage` ships no Rust code, so the "unit test" layer is **AC-verification
recipes** that exercise the prompt artefacts against synthetic fixtures.
Every AC is mechanically verifiable.

### Fixtures

**Location.** `tests/fixtures/process-improvements/` (existing directory
from A2; new fixtures added here).

**Fixture 1 — `widget-backlog-promotion.md`** (AC4):
- **Purpose.** Single-file synthetic `widget-backlog.md` with three rows: one
  `🟡 v2` row (the promotion candidate), one `✅ first pass` row (must be
  skipped — not a candidate), one `📭 future` row (must be skipped). Plus
  the prose-hit line `Tracked: TBD (file an issue when first item-view need
  surfaces).` somewhere in the file (verifies the column-header anchor).
- **Schema.** Matches `widget-backlog.md` verbatim — `| Widget | Status |
  Notes |` 3-column table, with the existing status legend block above it.
- **Exercise.** Approve the `🟡 v2` row for promotion → verify `Notes` cell
  is rewritten to `tracked: #999 — <existing notes content>` (use a
  placeholder `#999` for the AC because the actual issue number depends on
  gh state at run time). Verify the prose hit is NOT proposed for promotion.
  Verify the `Status` cell is byte-identical pre/post.

**Fixture 2 — `triage-decline.md`** (AC7):
- **Purpose.** Two-section synthetic fixture:
  - Section A — a thematic-file shape `| Item | Source | Status | Tracked |`
    table with one row whose `Tracked` cell holds `—`.
  - Section B — a `widget-backlog`-shape `| Widget | Status | Notes |` table
    with one row whose `Status` is `🟡 v2`.
- **Exercise.** Run `/triage` against the fixture; decline both rows.
  - Section A row's `Tracked` cell ⇒ literal `untracked`.
  - Section B row's `Notes` cell ⇒ `untracked (declined 2026-05-11): <prev>`
    (where `<prev>` is the original cell contents).
- **Second-run check.** Re-run `/triage` on the mutated fixture and verify
  neither row appears as a candidate.

**Fixture 3 (optional, decided here as YES) — `triage-inbox-3rows.md`**
(AC5 + AC2):
- **Purpose.** Synthetic `_inbox.md` containing exactly 3 rows in the
  4-column `| Item | Source | Section | Tracked |` schema. Used to exercise
  the per-entry drain step's three actions (sort / promote / drop) — one
  row per action.
- **Schema.** Matches `_inbox.md` verbatim — header lines + the 4-col table.
- **Exercise.** Run `/triage` against the fixture; sort row 1 into
  `signals-slots.md`, promote row 2 (approve), drop row 3.
  - After run: 1 row remains in inbox? NO — sort removes row 1 from inbox
    and appends to signals-slots; promote removes row 2 from inbox and
    appends to user-chosen thematic file (with `#N` in cell 4); drop
    physically removes row 3. Final inbox count = 0.
- **Note.** Inbox-fixture testing requires a mock destination thematic file
  (since the fixture inbox references rows that don't exist in real
  thematic files). Solution: the fixture's `Source` cells point at the
  *fixture's own filename* via a relative link, and AC5's verification
  recipe accepts that the destination thematic file is a synthetic
  side-fixture or a temporary file created during the test.

**Fixture 4 (optional, decided here as NO)** — combined-run fixture. The
spec's AC2 calls for a 10-row-source status table; the existing 8 thematic
files + `widget-backlog.md` + `_inbox.md` on disk are sufficient to
demonstrate (no synthetic file needed). AC2's verification is a real run.

### AC verification recipes

Each recipe is a sequence of shell commands a reviewer (or a future
self-improve eval pass) can execute mechanically.

**AC1 — Skill + agent files exist with correct frontmatter.**
```bash
ls -la .claude/skills/triage/SKILL.md .claude/agents/triage-runner.md
head -10 .claude/skills/triage/SKILL.md     # verify frontmatter block exactly matches spec
head -5  .claude/agents/triage-runner.md
grep '^model:' .claude/agents/triage-runner.md   # expect "model: opus"
grep -c 'disable-model-invocation: true' .claude/skills/triage/SKILL.md   # expect 1
grep -c 'allowed-tools:' .claude/skills/triage/SKILL.md                    # expect 1
grep -E 'gh issue (create|edit|list|view)' .claude/skills/triage/SKILL.md  # expect 4 hits
```

**AC2 — `/triage` status table covers all 10 row sources.**
Manual run on current data. Output captured to a temp file; reviewer
greps for the 10 expected filenames:
```bash
for f in signals-slots properties macros-codegen object-tree threading-runtime future-crates ci-docs-workflow python widget-backlog _inbox; do
  grep -q "${f}.md" /tmp/triage-run.log || echo "MISSING: ${f}.md"
done
```
Expect zero "MISSING" output. (The actual `/triage` run produces a status
table the agent's *Run-output summary* phase emits — see *Agent body* §
Phase 8 below.)

**AC3 — Cell-iteration sweep proposes promotion, per-row approval, single
bulk dedupe.**
Manual run. Verification:
```bash
grep -c 'gh issue list' /tmp/triage-run.log   # expect 1 (exactly one bulk call)
grep -c 'Approve promotion' /tmp/triage-run.log   # expect N (one per proposed row)
```
Approval prompt count equals the count of `—`-cell rows proposed.

**AC4 — Widget-backlog `🟡 v2` promotion writes `tracked: #N — <prev>` into `Notes`.**
```bash
cp tests/fixtures/process-improvements/widget-backlog-promotion.md /tmp/wb-before.md
# Run /triage against the fixture; approve the 🟡 v2 row.
diff /tmp/wb-before.md tests/fixtures/process-improvements/widget-backlog-promotion.md
# Expect: only the 🟡 v2 row's Notes cell changed; Status cell byte-identical;
# table format intact.
```
Also verify the prose-hit line did NOT get rewritten.

**AC5 — `_inbox.md` drain per-entry, three actions.**
```bash
cp tests/fixtures/process-improvements/triage-inbox-3rows.md /tmp/inbox-before.md
# Run /triage; act: sort row 1, promote row 2 (approve), drop row 3.
diff /tmp/inbox-before.md tests/fixtures/process-improvements/triage-inbox-3rows.md
# Expect: 3 rows removed from inbox; sort and promote destinations gained 1 row each.
grep -c '^| ' /tmp/triage-prompts.log   # expect 3 (one per inbox row)
```

**AC6 — `deferred-items.md` row counts updated.**
After any `/triage` run that mutates ≥1 row:
```bash
git diff ai-docs/deferred-items.md   # expect count-column changes per affected file
for f in ai-docs/deferred/*.md; do
  expected=$(grep -c '^| ' "$f")   # rough count; agent's tally is the authoritative one
  echo "$f $expected"
done
```

**AC7 — Declined rows receive `untracked` / `untracked (declined YYYY-MM-DD): <prev>`.**
```bash
cp tests/fixtures/process-improvements/triage-decline.md /tmp/decline-before.md
# Run /triage; decline both rows. Note the run date.
diff /tmp/decline-before.md tests/fixtures/process-improvements/triage-decline.md
grep 'untracked' tests/fixtures/process-improvements/triage-decline.md   # expect 2 hits
grep 'untracked (declined 2026-05-11): ' tests/fixtures/process-improvements/triage-decline.md   # widget-backlog row
# Second run on mutated fixture:
# Run /triage again; expect neither row in the candidate list.
grep -c 'Approve promotion' /tmp/triage-second-run.log   # expect 0
```

**AC8 — Concurrent-edit content-snapshot guard.**
Manual scenario:
```bash
# 1. Start /triage; let it propose a row but pause before approval.
# 2. In another shell, edit the candidate row's Item text in the underlying file.
# 3. Approve the row in /triage; expect abort with diff identifying the file.
# 4. Touch the same file with `touch` (no content change) and approve another row;
#    expect /triage to proceed (mtime not in the check).
```

### What is NOT tested

- Live `gh issue create` against the real repo — too destructive for a test
  pass; covered by manual run during AC3.
- Pagination watchdog firing — would require ≥450 live issues; covered by
  documentation only (the watchdog text is in the agent file).
- Issue C's bridge logic — out of scope for this PR.

## Edit specifications

This section spells out the exact content shape (not byte-for-byte, but the
sections, action tables, and field labels) for each new or modified file.
The implementation agent writes the literal prose; the design phase locks
the structure.

### File 1 — `.claude/skills/triage/SKILL.md` (new)

**Frontmatter** (verbatim from spec):

```
---
name: triage
description: "Batched promotion of untracked rows to gh issues; drains _inbox.md; reconciles md ↔ gh issue divergence (bridge ships in Issue C). Default threshold ≥ 3 unhandled rows."
argument-hint: "[N — override default threshold]"
disable-model-invocation: true
allowed-tools: Bash(gh issue create *) Bash(gh issue edit *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh api *) Bash(grep *) Bash(rg *) Read Edit
---
```

**Body sections** (in order):

1. **Launch line** — one line: "Launch the `triage-runner` subagent. The
   subagent reads `.claude/agents/triage-runner.md` for full instructions."
   (Mirrors `improve/SKILL.md`'s launch line verbatim except the subagent
   name.)

2. **`## Trigger and threshold`** — three short paragraphs:
   - Default threshold: ≥3 unhandled rows across the 10 sources. Tunable via
     `/triage [N]`. Below threshold, the subagent exits with a brief status
     report; no approval prompt is opened.
   - What counts as "unhandled": rows with `Tracked` = `—` in the 8 thematic
     files + `_inbox.md`, plus `🟡 v2` rows in `widget-backlog.md`.
     `_inbox.md` rows count individually.
   - Manual invocation always proceeds regardless of threshold (the `[N]`
     argument explicitly raises or lowers the gate).

3. **`## Cell-iteration sweep`** — bulleted description of the sweep:
   - Walks the 8 thematic files + `widget-backlog.md` (NOT `_inbox.md` —
     carved out for the drain step).
   - For each candidate row, the subagent drafts a title + body from the
     row's `Item` cell text and the linked `Source` spec.
   - Single bulk `gh issue list --state all --json number,title --limit
     500` query upfront; dedupe proposed titles by exact match against
     existing open+closed issues.
   - Presents the full batch to the user as a table.
   - User approves a subset; batched `gh issue create` for the approved
     rows.
   - On approval: write `#N` into cell 4 (thematic files) or prepend
     `tracked: #N — ` to the `Notes` cell (widget-backlog).
   - On decline: write literal `untracked` into cell 4 (thematic) or
     `untracked (declined YYYY-MM-DD): <prev>` into `Notes` (widget-
     backlog). Implicit-by-decline, no separate write confirmation.

4. **`## _inbox.md drain`** — bulleted description:
   - `_inbox.md` rows are handled per-entry, not in the cell-iteration
     sweep.
   - One prompt per row with three actions: **sort** / **promote** / **drop**.
   - **Sort**: remove row from `_inbox.md`, append to a user-chosen
     thematic file; cell 4 in the destination stays `—` (the row remains
     untracked at the thematic-file level).
   - **Promote**: queue the row into the same batched `gh issue create`
     call as the sweep; on approval the row migrates to a user-chosen
     thematic file with `#N` in cell 4; on decline migrates with
     `untracked` in cell 4.
   - **Drop**: physically remove the row from `_inbox.md`. Reserved for
     legitimately-bad rows (wrong shape, duplicate, etc.). Distinct from
     the `untracked` decline-marker, which is for legitimate-and-declined
     rows.

5. **`## Bridge`** — placeholder section. Exact content:
   ```
   ## Bridge

   <!-- Issue C (#205) fills in this section with the md ↔ gh issue
   divergence-detection workflow. Until C lands, `/triage` ships the
   promotion + drain flows above; no drift detection runs. -->

   _Not yet implemented. See [Issue #205](https://github.com/maratik123/quartzite/issues/205)._
   ```
   This is a one-line forward-note inside an HTML comment plus a single
   user-visible italic line. Issue C replaces both with real prose, without
   restructuring the surrounding sections.

6. **`## Run-output summary`** — bulleted requirements for the run summary
   the subagent emits at the end of every `/triage` run:
   - Status table covering all 10 row sources with candidate counts.
   - List of issues created (with `#N` and one-line title each).
   - List of rows declined (file path + cell content).
   - List of inbox actions taken (sort / promote / drop, with destination).
   - Concurrent-edit aborts (if any), listing the affected files.
   - `deferred-items.md` row-count diff.

The skill body MUST stay under ~80 lines so the subagent's prompt budget is
not eaten by it. Detail belongs in `triage-runner.md`.

### File 2 — `.claude/agents/triage-runner.md` (new)

**Frontmatter:**

```
---
name: triage-runner
description: "Batched promotion of untracked rows in ai-docs/deferred/*.md to gh issues; drains _inbox.md per-entry; rewrites declined rows with the untracked marker. Invoked by /triage. Mutation scope: ai-docs/deferred/** + gh issue create/edit only."
model: opus
---
```

**Body sections** (in order):

1. **`# Triage Runner Agent`** + one-paragraph description noting the agent
   is a deep batched-mutation subagent invoked by `/triage`; mutation scope
   is strictly `ai-docs/deferred/**` writes + `gh issue create/edit` calls;
   no code edits, no other instruction-file writes, no `learnings.md`
   writes.

2. **`## Inputs`** — read list:
   1. All 8 thematic files in `ai-docs/deferred/`.
   2. `ai-docs/deferred/widget-backlog.md`.
   3. `ai-docs/deferred/_inbox.md`.
   4. `ai-docs/deferred-items.md` (for end-of-run row-count update).
   5. Linked `Source` specs (read on demand for title/body drafting).

3. **`## Workflow`** — the 8 phases.

   - **`### Phase 1: Branch check`** — `git branch --show-current`; if
     `master`, halt and instruct the user to switch (per AGENTS.md AXIOM
     1). Identical shape to `self-improve.md` Step 5's branch check.

   - **`### Phase 2: Threshold gate`** — count candidates across all 10
     sources. Compare against `[N]` from `$ARGUMENTS` (default 3). If
     under threshold AND `[N]` was not explicitly raised, emit status
     summary and exit. Else proceed.

   - **`### Phase 3: Identify candidates`** — per-file rules in an action
     table:

     | Source | Candidate rule | Anchor |
     |---|---|---|
     | 8 thematic files | `Tracked` cell (column 4) = `—` | header `\| Item \| Source \| Status \| Tracked \|` required above the row |
     | `widget-backlog.md` | `Status` cell = `🟡 v2` | header `\| Widget \| Status \| Notes \|` required above the row; ignore bare `Tracked:` substrings in prose |
     | `_inbox.md` | `Tracked` cell (column 4) = `—` | header `\| Item \| Source \| Section \| Tracked \|` required above the row |

     `_inbox.md` candidates are tagged for the *drain phase*, NOT the
     batched sweep.

   - **`### Phase 4: Bulk `gh issue list` dedupe`** — exactly one call:
     `gh issue list --state all --json number,title --limit 500`.
     Pagination watchdog: if the response array has length ≥450, halt the
     run, print the watchdog message verbatim, exit. Otherwise build a
     local `{title → #N}` map. For each candidate, exact-title-match
     dedupe — if the proposed title already exists, the agent skips the
     `gh issue create` for that row and writes the existing `#N` into the
     destination cell (treating the match as an already-existing tracked
     row). Edge cases recorded in the run summary:
     - **Matched issue is closed.** Still treat as a match; write the
       closed `#N`. Issue C's bridge will flag the closed-state mismatch
       on a future run.
     - **Matched issue's title was rephrased after creation.** Out of
       reach of exact-match dedupe. Acceptable — the agent will propose a
       duplicate, and the user can decline it during approval. (The
       alternative — fuzzy matching — has too many false positives.)
     - **Title not matched but row's `Source` link already cites an
       issue.** Not a dedupe path — the row's `Tracked` cell would
       already hold `#N`, so the row is not a candidate in the first
       place.

   - **`### Phase 5: Draft titles and bodies`** — for each cell-iteration
     candidate:
     - **Title.** ≤70 chars; derived from the `Item` cell text, stripped
       of trailing `| Why …` continuations. The draft template is:
       ```
       <Item cell, trimmed>
       ```
     - **Body.** Markdown shape:
       ```
       Surfaced by `/triage` from [`<source path>`](<source path>).

       **Item:** <Item cell text>
       **Section:** <out-of-scope | deferred | open-question> (from Source spec section heading; for thematic-file rows, omit if not derivable)
       **Source spec:** [`<file>.spec.md`](<file>.spec.md)

       <one-paragraph context derived from the linked Source spec's surrounding text>
       ```

   - **`### Phase 6: Present batch and collect approvals (no creates yet)`**
     — the agent presents a table to the user: row-by-row title + source +
     destination cell. User approves a subset (per-row, but in a single
     table — the spec's "batched approval"). Phase 6 **collects** the
     approval-or-decline decision per row into an in-memory approval-queue
     and writes the decline-marker rewrites (`untracked` etc.) for declined
     rows immediately; **`gh issue create` calls are DEFERRED to Phase 7.5**
     so that approved cell-iteration rows and approved drain-promote rows
     (from Phase 7) share **one** contiguous sequential `gh issue create`
     pass, matching the spec's "one bulk call" contract.
     1. For each declined row, **immediately before** the rewrite, re-read
        the target file's content and confirm the candidate row's line
        still matches the start-of-session snapshot. **If mismatch:** abort
        that row's rewrite, print the diff, name the file, continue with
        the next row.
     2. On success: write `untracked` /
        `untracked (declined YYYY-MM-DD): <prev>` for declined rows per the
        action table below. Approved rows wait — their `#N` write happens
        in Phase 7.5 after the create.

        | Destination | Approval → write | Decline → write |
        |---|---|---|
        | 8 thematic files (cell 4) | `#N` | `untracked` |
        | `_inbox.md` (cell 4) | `#N` (then migrate row to user-chosen thematic file per drain rules) | `untracked` (then migrate per drain rules) |
        | `widget-backlog.md` (`Notes`) | prepend `tracked: #N — ` to existing notes | rewrite to `untracked (declined YYYY-MM-DD): <prev>` |

   - **`### Phase 7: Drain `_inbox.md`** — per-entry user prompt for every
     `_inbox.md` row tagged in Phase 3. UI shape (locked):

     For each row, present:
     ```
     Row N of M:
       Item:    <Item cell>
       Source:  <Source cell>
       Section: <Section cell>

     Action? (s)ort / (p)romote / (d)rop / (k)eep
     ```

     - **sort** — follow-up prompt: pick destination thematic file
       (numbered menu, 1–8). Append the row to that file with cell 4 =
       `—`; remove from `_inbox.md`.
     - **promote** — follow-up prompt: pick destination thematic file
       (numbered menu). Append the row to the **same** approval-queue
       collected by Phase 6 (Phase 6 deferred its creates for exactly
       this reason). All approved creates run together in Phase 7.5 as a
       single contiguous sequential `gh issue create` pass over the
       combined `(Phase 6 approved) + (Phase 7 promoted-and-approved)`
       union, sharing the dedupe map from Phase 4. On approval: migrate
       row to the destination thematic file with `#N` in cell 4 + remove
       from `_inbox.md`. On decline: migrate with `untracked` + remove.
     - **drop** — physically remove the row from `_inbox.md`. No
       migration.
     - **keep** — leave the row in `_inbox.md` unchanged (for rows the
       user wants to defer to a later `/triage` session).

   - **`### Phase 7.5: Combined `gh issue create` pass`** — the single
     "bulk call" the spec contracts for. Inputs: the approval-queue built
     by Phases 6 + 7 (union of cell-iteration sweep approvals and drain
     promotes). For each entry in the queue (sequential, in collection
     order):
     1. **Title-dedupe re-check** against the Phase-4 map (a freshly-
        approved title may collide with one that came back in the bulk
        `gh issue list`; if so, surface to user — accept the existing
        issue's `#N` or abort the create for that row).
     2. Run `gh issue create --title "<title>" --body "<body>"` and
        capture the returned `#N`.
     3. **Immediately before** the `#N` write to the target cell, re-read
        the target file's content and confirm the row's line still
        matches the start-of-session snapshot. **If mismatch:** abort
        the write, print the diff, name the file, continue with the next
        queue entry.
     4. On success: write `#N` per the action table — cell 4 for thematic
        files and `_inbox.md`; `tracked: #N — <prev>` in the `Notes` cell
        for `widget-backlog.md`. For `_inbox.md` drain-promote rows,
        migrate the row to its user-chosen thematic file (per Phase 7's
        sort sub-prompt) with `#N` in cell 4, and remove from `_inbox.md`.

   - **`### Phase 8: Update `deferred-items.md` and emit summary`** —
     re-count rows in every `ai-docs/deferred/*.md` file post-rewrite;
     rewrite the count column in `deferred-items.md` to match. Emit the
     run-output summary specified in the skill body (status table, issues
     created, rows declined, inbox actions, concurrent-edit aborts).
     Phase 8 is read-only after the count rewrite — no further rewrites
     of `ai-docs/deferred/*.md`, so the count is the snapshot baseline
     for any later concurrent-edit check.

4. **`## Anti-patterns`** — 6 bullets:
   - **Do NOT** write to any file outside `ai-docs/deferred/**` (excludes
     `ai-docs/learnings.md`, `AGENTS.md`, `.claude/**`).
   - **Do NOT** run multiple `gh issue list` calls per session — exactly
     one per run.
   - **Do NOT** silently overwrite a row if the content snapshot
     mismatches — abort with diff.
   - **Do NOT** auto-approve declined rows; the decline marker is
     implicit-by-decline (single user action), but the user's decline
     decision IS that action.
   - **Do NOT** route `_inbox.md` `—` rows through the cell-iteration
     sweep — drain is canonical.
   - **Do NOT** edit `widget-backlog.md`'s `Status` cell during promotion
     — only the `Notes` cell changes.

5. **`## Concurrent-edit guard`** — short prose section restating the
   content-snapshot rule from Phase 6 step 2, with an action table:

   | If the snapshot... | Action |
   |---|---|
   | matches the on-disk content immediately before write | proceed with rewrite |
   | does not match | **STOP** the rewrite for that row; print the unified diff between snapshot and current content; name the file; continue with the next row |
   | matches but mtime differs (file was touched, no content change) | proceed — mtime is not part of the check |

### File 3 — `AGENTS.md` *Agent Docs* table edit

**Insertion point.** Below the existing `spec-writer.md` row (line 246 of
`AGENTS.md` as of base commit).

**New row** (single addition to the table):
```
| `.claude/skills/triage/SKILL.md` + `.claude/agents/triage-runner.md` | `/triage` skill — batched promotion of `Tracked` = `—` rows in `ai-docs/deferred/*.md` (+ `🟡 v2` rows in `widget-backlog.md`) to gh issues; drains `_inbox.md` per-entry. Opus subagent; mutation scope strictly `ai-docs/deferred/**` + `gh issue create/edit`. |
```

### File 4 — `AGENTS.md` *Propagation Rule* table edit

**Insertion point.** Below the existing Interview group rows (`.claude/
agents/spec-writer.md` row, line 203).

**Three new rows** (mirroring the Review group's three-row pattern):
```
| `.claude/skills/triage/SKILL.md` | `.claude/agents/triage-runner.md` AND `.claude/skills/next/SKILL.md` (Triage group) |
| `.claude/agents/triage-runner.md` | `.claude/skills/triage/SKILL.md` AND `.claude/skills/next/SKILL.md` (Triage group) |
| `.claude/skills/next/SKILL.md` | `.claude/skills/triage/SKILL.md` AND `.claude/agents/triage-runner.md` (Triage group) |
```

**Also update the *Sync groups (canonical)* prose block** (currently lists
Review group + Interview group; add Triage group):
```
- **Triage group:** `.claude/skills/triage/SKILL.md` (workflow + threshold +
  drain UI) ↔ `.claude/agents/triage-runner.md` (deep subagent — `model: opus`,
  mutation scope `ai-docs/deferred/**` + `gh issue create/edit`) ↔
  `.claude/skills/next/SKILL.md` (consumer — surfaces `Candidates needing
  `/triage`` informational section).
```

### File 5 — `.claude/skills/improve/SKILL.md` one-line cross-reference

**Insertion point.** Immediately after the 6th list item, before the
`Context from user…` line — `improve/SKILL.md` is 20 lines total; the
numbered list spans lines 13–18, the blank line is 19 (or 20), and the
`Context` line is at the tail. Insert after the numbered list, before
the `Context` line (currently between approximately lines 18 and 21
depending on the blank-line shape).

**Inserted line** (one-line cross-reference, informational only, NOT a
formal sync group):
```
See also: `/triage` (`.claude/skills/triage/SKILL.md`) — same batched-approval and ≥3-unhandled threshold patterns; diverges in mutation scope (mutates `ai-docs/deferred/**` + `gh issue create/edit` rather than instruction files + `learnings.md`).
```

### Fixture 1 — `tests/fixtures/process-improvements/widget-backlog-promotion.md`

**Shape** (~30 lines):
- Header (`# Fixture: widget-backlog promotion (AC4)`)
- 1 short paragraph describing the fixture's purpose
- The status legend block copy-pasted from `widget-backlog.md`
- 1 table with 3 rows: `🟡 v2` (the candidate), `✅ first pass`
  (skipped), `📭 future` (skipped)
- The prose-hit line (`> spec. Tracked: TBD …`) inside a blockquote to
  test the column-header anchor
- `## Acceptance Criteria` block citing AC4 verification

### Fixture 2 — `tests/fixtures/process-improvements/triage-decline.md`

**Shape** (~30 lines):
- Header (`# Fixture: triage decline markers (AC7)`)
- 1 short paragraph describing the fixture's purpose
- `## Section A — thematic shape` with a `| Item | Source | Status |
  Tracked |` 4-col table; 1 row whose `Tracked` is `—`
- `## Section B — widget-backlog shape` with a `| Widget | Status | Notes
  |` 3-col table; 1 row whose `Status` is `🟡 v2`
- `## Acceptance Criteria` block citing AC7 verification (two-step: decline
  both → verify markers → re-run → verify neither re-proposed)

### Fixture 3 — `tests/fixtures/process-improvements/triage-inbox-3rows.md`

**Shape** (~25 lines):
- Header (`# Fixture: _inbox.md drain (AC5)`)
- 1 short paragraph describing the fixture's purpose
- The `_inbox.md` header block (copy-pasted from the real file's first
  10 lines)
- A 4-col table `| Item | Source | Section | Tracked |` with 3 rows, all
  `—` in cell 4, one each for `Section` ∈ {`out-of-scope`, `deferred`,
  `open-question`}
- `## Acceptance Criteria` block citing AC5 verification (sort row 1,
  promote row 2, drop row 3)

## Open questions

- **None blocking implementation.** The spec resolves every design-affecting
  decision; this design phase locks the remaining UX shape (drain prompt,
  fixture count, edit-spec insertion points). The implementation agent can
  proceed from this artefact without further interview.
- **Issue C extensibility check.** Will Issue C's *Bridge* section need to
  inject phases between Phase 6 (Present batch + apply) and Phase 7 (Drain)
  in `triage-runner.md`? Defensive answer: yes — drift detection probably
  runs *before* the cell-iteration sweep (so the user sees stale-tracked
  rows in the same batch as untracked candidates). Issue C's design phase
  decides; this PR leaves the *Bridge* section placeholder in the **skill
  body** (one HTML comment + one italic line) and does NOT add a Phase 6.5
  placeholder in the agent file. Issue C inserts its own phase.
- **`gh issue create` body length.** Long bodies fail the gh CLI's
  command-line size limit. The Phase 5 draft template above is short
  (~5 lines + a one-paragraph context blurb). If a long Source spec section
  produces a multi-paragraph context, the agent should truncate to the
  first paragraph and add a one-line link to the spec for full context.
  This is a soft rule recorded here for the implementation agent's
  attention; AC verification does not directly exercise it.
