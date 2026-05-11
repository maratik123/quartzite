# Process-Improvements Plan

**Status:** revision 3 (interview-derived; subagent-reviewed three times; umbrella issues filed; awaiting plan-file PR merge before `/task` picks up Issue A1)
**Started:** 2026-05-10
**Tracked in:** umbrella issues — A1 = #202 · A2 = #203 · B = #204 · C = #205 (all filed 2026-05-10 with `enhancement` label; A2/B/C also carry `blocked` until the previous issue lands)
**Style reference:** [`ai-docs/agent-writing-style.md`](../agent-writing-style.md)
**Authoring branch:** `chore/process-improvements-plan` (per AGENTS.md AXIOM 1 — feature branch created before any file edit).
**Author:** main session, derived from a structured interview with the project owner across five rounds (4 questions per round, all answered via AskUserQuestion structured input), then refined three times via opus-subagent clean-sheet review.

This is a **meta-plan** in the same shape as
[`2026-05-08-instruction-file-rewrite.md`](2026-05-08-instruction-file-rewrite.md):
no single GitHub issue, multiple umbrella issues to be opened from it.

## Revision history

| Rev | Date | What changed |
|-----|------|--------------|
| draft | 2026-05-10 | Initial draft from interview rounds 1–5. |
| rev 1 | 2026-05-10 | Applied opus subagent review #1: split Issue A → A1 + A2; fixed row terminology; bulk `gh issue list`; opus subagent for `/triage`; backfill seed; AGENTS.md axiom for `_inbox.md`. |
| rev 2 | 2026-05-10 | Applied opus subagent review #2: corrected spec count (55 not 109); corrected umbrella count (four not three); enumerated ≥ 5 spec shapes; corrected false `Tracked`-only-in-header claim; wrote actual axiom prose with action table; deferred `_inbox.md` creation from A1 to A2; fixed AC5 verification; reconciled AC8 with Risk row (dropped mtime); listed un-borrowed `/improve` patterns; added 4 risk rows; added `/bugfix` and web-UI exemptions; locked open-question answers (backfill scope + no pre-sort + defer file to A2 + threshold default 3). |
| rev 3 | 2026-05-10 | Applied opus subagent review #3: dropped `_inbox.md` from 5 → 4 columns (`Item \| Source \| Section \| Tracked`) to restore cell-4-Tracked invariant; carved `_inbox.md` out of Issue C's bridge sweep (drain step is the canonical handler); added backfill dedupe by `Source` link against existing thematic-file rows (no duplicate harvest of the 22 pre-2026-05-04 specs); rewrote A2 ACs to use synthetic fixtures in `tests/fixtures/` (deterministic at PR-merge time, no future-event dependency); replaced `wc -l ≥ 100` with structural per-section coverage check; promoted issue-label decision (`enhancement`) from risk-row to locked-in decision; added 6th spec shape (`- **<Term>** — <explanation>`). Subagent verdict: ITERATE → GO after these edits. |

## Goal

Close five observed gaps in the project's deferred-work tracking process so
that `/next` and `/task` see all eligible work, the index files don't drift
from `gh issue` state, untracked rows get promoted to issues in a controlled
batched flow (not ad-hoc), the `widget-backlog.md` lifecycle is folded into
the same machinery, and the 33 specs completed since the last manual
extraction get backfilled.

**Out of scope.** Visual surface for the maintainer (an HTML dashboard,
quartzite-powered designer, etc.) is **explicitly punted** to the v1
quartzite UI-designer track. This plan ships text-only changes.

## Background — what was found

### Three overlapping registries
Project backlog state currently lives in three places that drift:

| Registry | What it holds | Edited by | Read by |
|---|---|---|---|
| GitHub Issues | Tracking issues; `blocked` label; closed/open state. **64 total issues** as of 2026-05-10 (verified). | Humans (web UI), `/interview` (one issue per task), `/triage` (batched promotion — Issue B onward). | `/next` (issues + labels via `gh api`). |
| `ai-docs/plans/INDEX.md` | Active / done / deferred plans + dependency tree. | `/task` Step 12. | `/next` (full read). |
| `ai-docs/deferred/*.md` (8 thematic files + `widget-backlog.md`) | Per-spec follow-ups extracted from completed plans' *Out of scope* / *Deferred* / *Open questions* sections. **50 tracked references; 24 distinct issue numbers** (verified via `rg '\| #[0-9]+' ai-docs/deferred/`). | Manual one-off extraction passes (commits `a8f23d5`, `0304e8e`, `8bd8c26`). | Nothing — `/ai-audit` reads `deferred-items.md` index but no skill or agent reads the thematic files. |

### Actual row format (per-file)

The **8 thematic files** (`signals-slots.md`, `properties.md`, `macros-codegen.md`, `object-tree.md`, `threading-runtime.md`, `future-crates.md`, `ci-docs-workflow.md`, `python.md`) use a **4-column** markdown table:

```
| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| `BlockingQueued` connection type … | [core-types spec](…) |        | #48     |
| Auto-merge                          | [github-workflow spec](…) |   | —       |
```

Cell 4 (`Tracked`) holds either `#N` (issue ref), `—` (un-reviewed), or — under this plan — `untracked` (reviewed and declined for promotion).

The new `_inbox.md` (created in Issue A2) uses a **4-column** schema designed to keep the `Tracked` column at cell 4 across all files: `| Item | Source | Section | Tracked |`. The `Section` cell takes one of `out-of-scope` / `deferred` / `open-question` to record which heading the parser pulled the row from. `Status` is intentionally **not** included — it would always be blank initially and would shift `Tracked` to cell 5, breaking the invariant.

The literal string `Tracked` appears as the column header in those tables. **One prose hit** exists in `widget-backlog.md:89` (`> spec. Tracked: TBD (file an issue when first item-view need surfaces).`) — the parser **must anchor on column-header context, not the bare substring**, or scrub that prose hit before parsing.

`widget-backlog.md` itself uses a **different** schema — `| Widget | Status | Notes |` with 5 emoji statuses (`✅` first-pass / `🟡` v2 / `🤔` undecided / `❌` dropped / `📭` future) and **no `Tracked` column**. Tracked-issue refs, when added, go into the `Notes` cell (e.g. `Notes: tracked: #N — needs button group`).

### The gaps

1. **Discoverability.** `/next` reads only `gh issue list` + `INDEX.md`. It cannot see any row in `ai-docs/deferred/*.md`. So a row whose `Tracked` cell holds `—` (e.g. *Auto-merge* in `ci-docs-workflow.md`) is invisible to recommendations even though it is genuinely actionable.

2. **Untracked rows can't enter `/task` by issue-number.** Rows with `—` in `Tracked` (no gh issue) have no way to be picked up via `/task <N>`. They have to be re-discovered manually as free text, then `/interview` resolves a tracking issue. There is no curated list of "ready-but-untracked" candidates.

3. **Status drift.** Several rows still have `—` in `Tracked` for work that is genuinely planned (e.g., Auto-merge, removing `rstest`, future features `extension`/`8k_pages`, full wildcard re-exports, EXAMPLES.md). Conversely, several rows pointing at closed issues remain — `ci-docs-workflow.md` cites `#60` for "Contributing guide / roadmap" / "Additional facade features" but **#60 is CLOSED** (verified via `gh issue view 60`). The aggregator has no maintenance loop.

4. **Widget-backlog has no triage gate.** The file says "file an issue when an item moves to in-progress" — purely manual, easy to miss, and the moment-of-pick is also the worst time to interrupt for issue creation.

5. **No propagation from new completed specs.** When a spec lands in `done/`, its own *Out of scope* / *Deferred* / *Open questions* sections never propagate into the aggregator. Counts (verified):
   - Total `done/*.spec.md` = **55**.
   - `done/*.spec.md` dated ≥ 2026-05-04 (the day after the last manual extraction commit `0304e8e` on 2026-05-03) = **33**.
   - Those 33 specs each have at least one of *Out of scope* / *Deferred* / *Open questions* sections that never reached the aggregator.
   - Issue A2 includes a one-shot backfill over all 55 specs (and additionally over `done/*.design.md` if the audit confirms designs also carry deferred items historically — see Issue A2 *Preconditions*), with **dedupe by `Source` link** so rows already extracted by the prior manual passes (covering the older 22 specs) are not re-emitted.

### Why "black squares" matters here

The project owner's framing — *"we're designing a GUI lib but proposing the user looks at black squares"* — is **literal**, not rhetorical: `quartzite-renderer`'s `Painter` methods are stubs (per `INDEX.md` "Suggested next steps" #3), so the gpu-snapshot tests committed in [#192](https://github.com/maratik123/quartzite/issues/192) produce blank PNGs.

The v1 DoD includes a UI designer, dogfooded so the framework hits its own missing pixels. **That work is a separate track**, not part of this plan. Visual surface for maintainer-side backlog browsing is consciously deferred to inherit from the v1 designer once it ships.

## Locked-in decisions

Captured during the round-by-round interview (rounds 1–5) and refined across three opus-subagent reviews. Each row below is non-negotiable for this plan unless a future review surfaces a contradiction.

| Dimension | Decision |
|---|---|
| "Black squares" interpretation | Literal — vello/`Painter` stubs produce blank PNGs. Resolved by v1 UI designer track, **not** this plan. |
| Visual surface in this plan | **None.** Maintainer stays text-only until v1 designer ships. |
| Source of truth | Both md and gh issues editable; skill flags conflicts; **no silent overwrite either way**. |
| Bridge mechanism | Live `gh api` on read; **pure skill-prompt logic** driven by an opus subagent. No Rust binary, no shell script. Single bulk `gh issue list --state all --json number,state,title --limit 500` per `/triage` run; pagination watchdog warns at ≥ 0.9× limit. |
| Bridge cadence | Inside `/triage` only. Not on every push, not in CI, not on every `/task`. |
| Phase structure | **Four umbrella issues** (A1 / A2 / B / C — Issue A originally one, split during rev 1), no phases, hygiene-first ordering. |
| Step 12 propagation scope | Walk *Out of scope* + *Deferred* + *Open questions* sections. Append parsed rows to `ai-docs/deferred/_inbox.md`. |
| Backfill scope (one-shot) | Same sections as Step 12: *Out of scope* + *Deferred* + *Open questions*. Walks all 55 `done/*.spec.md` (plus `done/*.design.md` if audit confirms). **Dedupe:** rows whose `Source` link already appears in a thematic file are skipped (a prior manual extraction harvested them — commits `a8f23d5`, `0304e8e`, `8bd8c26`). Skipped rows are recorded in the design-phase audit log. |
| Backfill / Step 12 destination | Always `_inbox.md`. **No pre-sort** into thematic files; `/triage` does all classification. Uniform code path. |
| `_inbox.md` schema | 4-column table: `\| Item \| Source \| Section \| Tracked \|`. `Section` ∈ {`out-of-scope`, `deferred`, `open-question`}. `Tracked` initially holds `—`; same semantics as the thematic files (cell 4 = `Tracked`). |
| `/next` discoverability | Reads every md row; tracked rows piggyback on issues; untracked surface as "Candidates needing `/triage`" — never auto-recommended. |
| `widget-backlog.md` handling | Same triage rules as other deferred files; 🟡 v2 = untracked-candidate; tracked refs go in `Notes` cell (no schema change to the file). |
| Conflict types flagged | Closed-issue refs · `Tracked` cell holding `—` · status mismatch (`✅ done` ↔ OPEN). Issues without md rows = OK. |
| `/improve` parallel | `/triage` borrows the **batched-approval** and **threshold-trigger** patterns from `/improve`; **diverges** in mutation scope (md mutation + `gh issue create/edit`); **does not borrow** `/improve`'s eval gate, hook-escalation pattern (≥ 3 occurrences → hook), or `learnings.md`-style decision log. The reasons each pattern is omitted are documented in *Open implementation knobs* below. |
| `/triage` model | Opus subagent (mirrors `/improve`'s `model: opus` discipline). New file `.claude/agents/triage-runner.md` is part of Issue B. |
| `/triage` threshold | Tunable via skill arg (`/triage [N]`). Default ≥ 3 unhandled rows (matches `/improve`'s default). |
| Decline-marker token | Literal `untracked` written to the `Tracked` cell (cell 4 in thematic files and `_inbox.md`). In `widget-backlog.md`, prepend `untracked (declined YYYY-MM-DD): ` to the existing `Notes` cell. |
| `_inbox.md` creation | Created by **Issue A2** (alongside the backfill that populates it). A1 lands AGENTS.md governance only. |
| `_inbox.md` write discipline | New AGENTS.md AXIOM (full prose with action table — see Issue A1 deliverables). |
| Issue-creation discipline | "No ad-hoc gh issue creation" reads as: every issue must come from a controlled flow. Allowed flows: `/interview` (one issue per task), `/bugfix` (one issue per regression where no existing issue covers it), `/triage` (batched promotion). **Web-UI issue creation by the project owner is exempt** — the discipline rule constrains the agent, not the user. |
| Issue labels for umbrella PRs | Label each new umbrella issue with `enhancement` (existing label) at `gh issue create` time. Future plan may add a `process` label. |
| AC verification strategy | A2 / B / C verification uses **synthetic spec fixtures** committed at `tests/fixtures/`. Fixtures cover one example per parser-relevant spec shape and one per AC scenario. End-to-end ACs that genuinely require live gh-state run as documented manual scenarios. The plan does **not** wait for "the next merged spec" to verify any AC. |

## Plan: four umbrella issues

```
Issue A1 (`/next` discoverability)        ──► hygiene; ships first; verifiable on current data; AGENTS.md governance only
                  │
                  ▼
Issue A2 (`/task` Step 12 + backfill)     ──► creates `_inbox.md`, seeds it via deduped backfill, wires Step 12 forward-going
                  │
                  ▼
Issue B (`/triage` skill base)            ──► standalone batched promotion + inbox drain; opus subagent
                  │
                  ▼
Issue C (md ↔ issues bridge in `/triage`) ──► drift detection; per-`/triage` run; bulk `gh issue list`; `_inbox.md` carved out
```

A1 → A2 → B → C is strict. A1 contains no forward references to skills not yet created (`_inbox.md` and the `/triage`-referencing AGENTS.md row both ship in A2 and B respectively).

---

### Issue A1 — `/next` discoverability + AGENTS.md governance (hygiene-first)

**Goal.** Make backlog work visible to `/next`. Establish AGENTS.md governance for the forthcoming `_inbox.md`. Verifiable on current data without any new file.

**Deliverables.**
- `.claude/skills/next/SKILL.md` — extend the prompt to read every row from `ai-docs/deferred/*.md` (including `widget-backlog.md`).
  - Rows whose `Tracked` cell holds an issue ref (`#N`) rank as supplements to the matching open issue (no double-recommendation).
  - Rows whose `Tracked` cell holds `—` (or `widget-backlog.md` rows with status `🟡 v2`) appear in a new section titled *"Candidates needing `/triage`"*.
  - `/next` does **not** recommend a `Candidates` row as the chosen task — only mentions them and suggests running `/triage` first (note: `/triage` ships in Issue B; until then, candidates surface as informational).
- `AGENTS.md` *Workflow* section — new axiom in the existing `agent-writing-style.md` § Pattern 1 shape (multi-paragraph blockquote with action table). Proposed prose:

  > **AXIOM — `ai-docs/deferred/_inbox.md` is written ONLY by `/task` Step 12 and `/triage`.**
  > Hand-edits to `_inbox.md` defeat the propagation contract that Issue A2 sets up — they hide rows from the parser and conflict with future Step-12 appends.
  >
  > | If you see... | Action |
  > |---|---|
  > | A row in `_inbox.md` you want to move to a thematic file | Run `/triage`; let it sort the row |
  > | A row in `_inbox.md` you want to drop | Run `/triage`; mark "drop" during the drain step |
  > | A row missing from `_inbox.md` for a freshly-merged spec | Re-run `/task` Step 12 manually (or wait for the next merged spec to trigger it) |
  > | An entry whose source-spec section shape was unrecognised by the parser | Step 12 emits a warning; resolve by reformatting the source spec OR by adding the shape to the parser's allow-list (Issue A2 design phase) |
- `AGENTS.md` *Agent Docs* table — add row for `ai-docs/deferred/_inbox.md` with one-line purpose: *"triage queue — rows from completed specs awaiting `/triage` classification (writers: `/task` Step 12 and `/triage` only; introduced in Issue A2)."*. The "introduced in Issue A2" suffix is a deliberate forward note, not a stale pointer — it disappears when A2 lands.

**Acceptance criteria.**

| # | Criterion | Verification |
|---|---|---|
| AC1 | `/next` surfaces ≥ 1 untracked row when any `ai-docs/deferred/*.md` has `—` in the `Tracked` cell | Manual run on current data; output contains the *Candidates needing `/triage`* section listing at least one of: Auto-merge, removing `rstest`, full wildcard re-exports, EXAMPLES.md, future features `extension`/`8k_pages` |
| AC2 | `/next` does not double-rank a row whose `Tracked` cell holds `#N` referring to an already-listed open issue | Manual run; pick row with `Tracked: #48` (BlockingQueued); confirm only one mention in output |
| AC3 | AGENTS.md *Workflow* contains the `_inbox.md` axiom with action table; AGENTS.md *Agent Docs* table includes `_inbox.md` row with the "writers" clause | `grep -A 10 'AXIOM.*_inbox.md' AGENTS.md` shows axiom and table; `grep '_inbox.md' AGENTS.md` shows ≥ 2 hits (axiom + table row) |
| AC4 | Sync-group propagation: every reference to deferred files in `.claude/` and `AGENTS.md` is consistent with the new structure | `grep -rn 'ai-docs/deferred' .claude/ AGENTS.md` count = baseline (pre-PR) + new references. Expected new references: 1 in `next/SKILL.md` (read deferred), 1 in `AGENTS.md` *Workflow* axiom, 1 in `AGENTS.md` *Agent Docs* table. Reviewer confirms count matches expectation |

**Sync-group footprint.** `next/SKILL.md`, `AGENTS.md`. No new sync-group entries (the `/next` ↔ `/triage` informal coupling is documented in B's deliverables when `/triage` actually exists).

---

### Issue A2 — `/task` Step 12 propagation + `_inbox.md` creation + one-shot deduped backfill

**Goal.** Capture every newly-completed spec's deferred rows automatically. Backfill the 55 already-done specs at the same time (with `Source`-link dedupe so the older 22 manually-harvested specs don't double-emit). Create `_inbox.md` only now (when its writer exists).

**Preconditions (must complete before implementation).**

- **Spec-shape audit (REQUIRED).** Issue A2 design phase must begin with an audit of all `done/*.spec.md` *Out of scope* / *Deferred* / *Open questions* sections (and `done/*.design.md` to confirm whether designs historically carried deferred items — `signals-slots.md` shows at least one design-sourced row, so designs likely in scope). Sampling at draft time found at least **6 shapes**, and a single spec can mix shapes across its sections:
  1. **bulleted list** `- Item | Why` (most common; e.g. `done/2026-05-01-widgets.spec.md` *Deferred*).
  2. **bulleted list with three columns** `- Item | Why | Separate issue?` (e.g. `done/2026-05-05-log-facade.spec.md`).
  3. **bulleted list, no `|` separators** — plain prose bullets (e.g. `done/2026-05-09-paint-style.spec.md` *Out of scope*, *Open questions*).
  4. **3-column markdown table** (e.g. `done/2026-05-10-object-property-serialization-layer.spec.md` *Deferred*).
  5. **literal `None.` / `(none — …)` sentinel** (e.g. `done/2026-05-02-inline-simple-fns.spec.md`, `done/2026-05-05-doc-convention.spec.md`).
  6. **bulleted list with bolded-leading term + em-dash** `- **<Term>** — <explanation>` (e.g. `done/2026-05-10-object-property-serialization-layer.spec.md` *Out of scope*, `done/2026-05-10-docs-cleanup-197.spec.md`, `done/2026-05-08-project-docs.spec.md`, `done/2026-05-08-ci-sccache.spec.md`, `done/2026-05-07-code-style-extraction.spec.md`). Parser can extract the bolded term as `Item` and the post-em-dash text as the explanation.

  And the destination is `_inbox.md` (4-column schema above). **Per-section heterogeneity** within a single spec (e.g. *Deferred* table, *Out of scope* bullets) is the norm, not the exception.

- **Decision gate.** Audit must produce one of:
  - **(a)** A multi-shape parser specification with a "shape unrecognised — warn and skip" fallback. Lists every shape encountered and its mapping to the `_inbox.md` schema.
  - **(b)** A canonical *Deferred* / *Open questions* / *Out of scope* shape locked forward (e.g. "always a 3-column table with these headers"); pre-canonical specs are then either backfilled by hand or accepted as un-propagated (decided per-spec during the backfill pass below).
- The decision is recorded in the issue's design doc.

**Deliverables.**
- New file: `ai-docs/deferred/_inbox.md` — header explains the role and points at the AGENTS.md axiom + the `/triage` skill (which ships in Issue B and will be referenced here).
- `_inbox.md` schema lock: 4-column table `| Item | Source | Section | Tracked |` (per *Locked-in decisions*).
- `.claude/skills/task/SKILL.md` Step 12 — after moving spec/design to `done/`:
  - Parse the spec's *Out of scope* / *Deferred* / *Open questions* sections per the audit-derived parser contract.
  - Append each parsed row to `ai-docs/deferred/_inbox.md` with `Section` set to the heading the row came from.
  - Stage `_inbox.md` in the same commit.
  - Emit a warning when the spec's section shape is unrecognised; do not block Step 12.
- Update Step 12 *Gate checklist* row to require `_inbox.md` was checked / appended.
- **One-shot deduped backfill skill flow.** A repeatable procedure (skill prompt + bash, no separate executable) that reuses Step 12's parser to walk every `done/*.spec.md` (and `done/*.design.md` if the audit confirms) and seeds `_inbox.md`. Run once at Issue A2 land. **Dedupe rule:** for each parsed row, check whether the row's `Source` link already appears in any of the 8 thematic files; if yes, skip the row (the older 22 specs were harvested by commits `a8f23d5`, `0304e8e`, `8bd8c26` — re-harvesting would produce duplicates the user has to drop in `/triage`'s first run). The backfill is the same parser as Step 12, invoked across the historical corpus rather than on a single fresh spec — explicit reuse, not a separate codepath. Skipped rows are logged to the design doc with the matching thematic-file row for audit.
- **Synthetic spec fixtures.** New directory `tests/fixtures/process-improvements/` containing one fixture per parser-relevant spec shape (≥ 6 shapes) and one per AC scenario (mangled section, `None.` sentinel, all-three-section spec, etc.). Fixtures are checked-in committed-form `*.spec.md` files used by the ACs below.

**Acceptance criteria.**

| # | Criterion | Verification |
|---|---|---|
| AC1 | Parser-shape audit complete; design doc lists every encountered shape, decides between multi-shape parser and canonical-shape lock-in, and lists per-spec skip decisions | Read design doc; verify enumeration matches audit output |
| AC2 | `/task` Step 12 against `tests/fixtures/process-improvements/all-three-sections.spec.md` (one *Out of scope* bullet, one *Deferred* table row, one *Open questions* bullet) appends three rows to `_inbox.md` with correct `Section` values | Run Step 12 against the fixture; `git diff` of dry-run shows three rows; `Section` cells = `out-of-scope`, `deferred`, `open-question` respectively |
| AC3 | Backfill pass over all 55 `done/*.spec.md` (plus `done/*.design.md` if in scope per audit) emits, for **every** non-empty *Out of scope* / *Deferred* / *Open questions* section of every in-scope spec, either ≥ 1 row in `_inbox.md` OR a skip-log entry citing the pre-existing thematic-file row whose `Source` already covered it | Run backfill at A2 land; design doc lists per-section emit-or-skip decisions; reviewer can spot-check any section by looking up its source spec and confirming the inbox or thematic-file row exists |
| AC4 | Step 12 emits a warning (does not error) on unrecognised section shape | Run Step 12 against `tests/fixtures/process-improvements/mangled-section.spec.md` (a fixture with a deliberately mangled *Deferred* section); observe warning in output, no error exit |
| AC5 | Step 12 *Gate checklist* row includes `_inbox.md` check | Read `task/SKILL.md` |
| AC6 | `_inbox.md` exists at `ai-docs/deferred/_inbox.md` with header explaining role and writers; AGENTS.md axiom (landed in A1) now references a real file | `ls ai-docs/deferred/_inbox.md`; `head -20 ai-docs/deferred/_inbox.md` shows expected header; `grep _inbox.md AGENTS.md` matches the row landed in A1 |
| AC7 | Backfill skips rows whose `Source` link already has a corresponding row in a thematic file | Run backfill at A2 land; verify the 22 pre-2026-05-04 specs (e.g. `core-types`, `runtime`, `auto-connection`, `github-workflow`) do not generate duplicate items in `_inbox.md` for entries already in thematic files; design doc lists exactly which rows were skipped |

**Sync-group footprint.** `task/SKILL.md`. The AGENTS.md axiom landed in A1 already governs this issue's Step 12 changes — no new axiom needed here.

---

### Issue B — `/triage` skill (cadence + widget-backlog source) + opus subagent

**Goal.** A standalone batched-promotion workflow for untracked rows. Borrows `/improve`'s batched-approval and threshold-trigger patterns; runs as opus subagent. Does **not** borrow `/improve`'s eval gate, hook escalation, or `learnings.md` decision log — those omissions are intentional, see *Open implementation knobs* for rationale.

**Deliverables.**
- New file: `.claude/skills/triage/SKILL.md`. Frontmatter:
  ```
  ---
  name: triage
  description: "Batched promotion of untracked rows to gh issues; drains _inbox.md; reconciles md ↔ gh issue divergence (bridge ships in Issue C). Default threshold ≥ 3 unhandled rows."
  argument-hint: "[N — override default threshold]"
  disable-model-invocation: true
  allowed-tools: Bash(gh issue create *) Bash(gh issue edit *) Bash(gh issue list *) Bash(gh issue view *) Bash(gh api *) Bash(grep *) Bash(rg *) Read Edit
  ---
  ```
  (`.claude/settings.json` already allows `Bash(gh *)` globally — the per-skill `allowed-tools` is a deliberate narrowing for clarity, not a new permission.)
- New file: `.claude/agents/triage-runner.md`. Frontmatter `model: opus`. Mirrors `.claude/agents/self-improve.md` structure — agent reads md, proposes diffs, applies after user confirmation. Agent does **not** mutate code outside `ai-docs/deferred/**` and does not stage gh issue mutations until user approves.
- Triggers: manual (`/triage`); recommended threshold = skill arg `[N]` defaulting to 3 unhandled rows (matches `/improve`'s default).
- Walks every `ai-docs/deferred/*.md` (8 thematic + `widget-backlog.md` + `_inbox.md`).
- For untracked rows (the `Tracked` cell holds `—`, or `widget-backlog.md` rows with status `🟡 v2`):
  1. Drafts title + body per row from the row text + linked Source spec.
  2. Presents the full batch to the user.
  3. User approves a subset.
  4. Batched `gh issue create` for the approved rows only. **Single** `gh issue list --state all --json number,title --limit 500` query first to dedupe against any issue that already exists with a matching title.
  5. Writes the new `#N` back into the appropriate cell:
     - 8 thematic files and `_inbox.md` → the `Tracked` column (cell 4 in both).
     - `widget-backlog.md` → `Notes` cell (e.g. *"tracked: #N — original notes"*).
  6. Rows the user declines get the `Tracked` cell rewritten to literal `untracked` (in thematic files and `_inbox.md`), or `Notes` rewritten to `untracked (declined YYYY-MM-DD): <previous>` in `widget-backlog.md`. Implicit-by-decline write — single user action per row, no separate write-confirmation.
- Drains `_inbox.md`: per-entry user prompt; for each entry, asks the user to sort into a thematic file, promote to issue, or drop. Per-entry (not batched) because each decision is consequential. The drain step is the canonical handler for `_inbox.md` rows — they are **not** processed by the cell-iteration sweep above (carved out to avoid double-handling; see Issue C *Exception*).
- Updates `ai-docs/deferred-items.md` row counts at the end of each run.
- **Drift detection (closed-issue refs and status mismatch) is NOT in this issue** — that ships in Issue C.

**Acceptance criteria.**

| # | Criterion | Verification |
|---|---|---|
| AC1 | `/triage` skill exists at `.claude/skills/triage/SKILL.md` with frontmatter listed above; agent at `.claude/agents/triage-runner.md` (`model: opus`) | `ls`; read frontmatter; verify `model: opus` |
| AC2 | Running `/triage` reports a status table covering all 9 deferred files (8 thematic + `widget-backlog.md`) AND `_inbox.md` | Manual dry-run; output table contains 10 row-source labels |
| AC3 | Running `/triage` proposes promotion for rows with `—` in `Tracked` cell; requires per-row user approval; batches `gh issue create`; deduplicates against existing open issues by title via single bulk `gh issue list` call | Manual run on current data; verify approval prompt; verify only one `gh issue list` call in run log |
| AC4 | A `🟡 v2` row in `widget-backlog.md` promoted to an issue ⇒ `Notes` cell rewritten with the new `#N` reference; row's `Status` column unchanged; format intact | Run against `tests/fixtures/process-improvements/widget-backlog-promotion.md` fixture; `git diff` shows expected cell mutation |
| AC5 | `_inbox.md` rows (seeded by A2's backfill) sorted into thematic files OR promoted to issues OR dropped during a `/triage` run; per-entry user prompt; rows are **not** also processed by the cell-iteration sweep | Manual dry-run against the `_inbox.md` populated by A2's backfill; verify each inbox row triggers exactly one prompt (the drain step), not two |
| AC6 | `ai-docs/deferred-items.md` row counts reflect current state after `/triage` | `git diff ai-docs/deferred-items.md` shows count column changes matching actual row counts |
| AC7 | Declined rows receive a literal `untracked` token in the `Tracked` cell (or `untracked (declined YYYY-MM-DD): <prev>` rewrite in `Notes` for widget-backlog) so subsequent runs skip them | Manual scenario test |
| AC8 | `/triage` re-reads each md file immediately before its rewrite step; aborts with diff if the row text no longer matches the read snapshot | Manual scenario test (touch the file mid-session and confirm abort with diff output) |

**Sync-group footprint.** New files `triage/SKILL.md` and `triage-runner.md`. AGENTS.md gets one new row in *Agent Docs* table referencing both. `improve/SKILL.md` gets a one-line cross-reference. AGENTS.md *Propagation Rule* gets a new sync-group: `triage/SKILL.md` ↔ `triage-runner.md` ↔ `next/SKILL.md` (text in `/next` mentions `/triage`; mutual updates required).

---

### Issue C — md ↔ issues bridge inside `/triage`

**Goal.** Detect divergence between md state and `gh issue` state during `/triage` runs. Surface as conflicts; never silently overwrite.

**Deliverables.**
- Extend `.claude/skills/triage/SKILL.md` (after Issue B lands) with a *Bridge* section.
- **Bulk-fetch issue state** with one call: `gh issue list --state all --json number,state,title --limit 500`. Build a local `{number → {state, title}}` map. Then iterate `Tracked`-column values across all md files (cell 4 in the 8 thematic files; cell 4 in `_inbox.md`; the `Notes` cell in `widget-backlog.md`) and look up each ref in the map.
  - **Exception:** `_inbox.md` rows whose `Tracked` cell is `—` are **not** routed through the cell-iteration sweep — they go through Issue B's per-entry drain step instead, because `_inbox.md` rows carry an extra `Section` column that the drain step uses for classification. The bridge does still inspect `_inbox.md` rows whose `Tracked` cell holds `#N` (in case a previous `/triage` already promoted an inbox row).
  - Current data: 50 total tracked references, 24 distinct issue numbers (verified via `rg '\| #[0-9]+' ai-docs/deferred/`); 64 total open+closed issues exist.
  - Per-run cost stays at 1 API call regardless of growth.
  - **Pagination watchdog:** if the response length is ≥ 0.9× the limit (i.e. ≥ 450 issues returned), `/triage` warns the user and recommends raising `--limit` or implementing pagination.
- Conflict types reported (no silent overwrite — every conflict surfaces a diff and asks the user):
  1. **Stale tracked.** Row implies open / no `✅ done` marker, issue is CLOSED. (Worked example: `ci-docs-workflow.md` rows pointing at `#60` — currently CLOSED.)
  2. **Status mismatch.** Row says `✅ done` (in *Status* cell), issue is OPEN. Or row implies open and issue is closed-as-not-planned (different from completion).
  3. **Untracked candidate.** `Tracked` cell = `—`. Already handled by Issue B's promotion flow; surfaced here as a count for situational awareness, not as a conflict.
- Issues that exist in `gh` but have no md row anywhere are **explicitly allowed** — not flagged.
- For each detected conflict, the user picks one of: `update md` / `update issue` / `keep both` (with reason captured in run output).
- `update issue` decisions surface a `gh issue edit` diff preview to the user before running the command — the agent never silently rewrites issue body or state.
- Bridge runs as part of every `/triage` invocation. It is not its own command. Same batched-approval shape as the promotion flow in Issue B.

**Acceptance criteria.**

| # | Criterion | Verification |
|---|---|---|
| AC1 | `/triage` makes one bulk `gh issue list --state all --json number,state,title --limit 500` call per run; iterates `Tracked`-column values against the local map; `_inbox.md` rows with `Tracked` = `—` are excluded from the sweep | Manual dry-run on current data — run log shows exactly one `gh issue list` invocation; conflict report flags the `#60` references in `ci-docs-workflow.md`; `_inbox.md` `—` rows route to drain step only |
| AC2 | `/triage` reports md `✅ done` rows where the linked issue is OPEN, and vice versa | Run against `tests/fixtures/process-improvements/divergence-cases.md` fixture (synthetic divergence injected); verify both directions reported |
| AC3 | `/triage` does not propose creating an issue for an md row already linked to an OPEN issue | Manual scenario test |
| AC4 | An issue with no md row anywhere is silently accepted (not flagged) | Manual scenario test |
| AC5 | Per-row resolution (`update md` / `update issue` / `keep both`) recorded in run output | Manual run; verify decision log section |
| AC6 | `update issue` decisions show a diff preview before running `gh issue edit`; user must confirm | Manual scenario test |
| AC7 | Pagination watchdog: if `gh issue list` returns ≥ 0.9× the configured limit, `/triage` warns the user | Manual scenario test (set `--limit 60` against current 64-issue corpus and verify warning) |

**Sync-group footprint.** `triage/SKILL.md` and `triage-runner.md` only — both files get fattened during this issue.

## Sequencing

```
Issue A1 (`/next` discoverability)        ──► Issue A2 (Step 12 + `_inbox.md` + deduped backfill) ──► Issue B (`/triage` skill base) ──► Issue C (bridge in `/triage`)
```

Strict sequence. No parallel paths.

- **A1 → A2:** A1 lands AGENTS.md governance for `_inbox.md`; A2 creates the actual file.
- **A2 → B:** B's `/triage` drains `_inbox.md`; A2's backfill seeds it.
- **B → C:** C extends `triage/SKILL.md`.

The B+C collapse escape-hatch from rev 1 is **dropped** — both touch the same file but ship as distinct issues for clarity.

## Open implementation knobs (deferred to per-issue design phase)

- **Inbox drain UX.** Per-row prompt vs. batched table-of-decisions. Default proposal: per-row (each decision is consequential). Resolve in Issue B design.
- **Conflict resolution UI in `/triage`.** Per-conflict prompt vs. batched table. Resolve in Issue C design.
- **Backfill skip handling for unrecognised shapes.** What does the backfill do with specs whose section shape doesn't match the parser? Skip with a warning vs. block A2 land. Resolve in Issue A2 design — recommend warn-and-skip + audit log entry per skip.
- **`Tracked` column rename.** The "untracked" decline-marker token clashes semantically with the column name "Tracked" — a cell whose `Tracked` value is `untracked` reads awkwardly. **Out of scope for this plan** but consider for a future cosmetic PR: rename column to `Issue` (cell values: `#N` / `—` / `untracked`).
- **`/improve` patterns NOT borrowed (rationale).**
  - **Eval gate** (post-application "did the rule work?" check): `/triage`'s mutations are user-in-loop per row, not batched silent applications, so an eval gate is redundant. Each user approval IS the eval.
  - **Hook escalation at ≥ 3 occurrences:** `/improve` escalates *rules*; `/triage` mutates *data*. There's no "rule" to escalate after 3 triage runs — if a row is repeatedly declined, the row is already marked `untracked` and won't re-surface. If a parser pattern fails 3 times, that's an Issue A2 design-phase concern, not `/triage`-time.
  - **`learnings.md`-style decision log:** `/triage`'s decision log is implicit in the md file state (`#N` markers, `untracked` markers, `—` markers). Future enhancement: add date stamps to declined rows for audit purposes — out of scope for v1 of `/triage`.

## Non-goals

- Visual surface (HTML dashboard, quartzite UI, etc.).
- Source-of-truth migration to GitHub Issues with markdown auto-generated from issue queries (the user-chosen model is *both editable, conflicts flagged*, not unidirectional generation).
- CI gate that fails on drift. The user-chosen cadence is *inside `/triage` only*.
- Webhook-driven mirror.
- Rust binary or shell script for the bridge — pure skill-prompt logic, opus subagent.
- Folding `/improve` and `/triage` into a unified `/groom` skill.
- Touching `ai-docs/learnings.md` workflow. AGENTS.md *Boundary rule 2* prevents this plan from also editing learnings.md as part of the same PR.
- Reshaping `widget-backlog.md` schema to match the thematic files. Tracked refs go in the existing `Notes` column; no schema migration.
- Renaming the `Tracked` column to `Issue` (noted as future cosmetic improvement; out of scope for this plan).
- Adding a `Status` column to `_inbox.md` (rejected during rev 3 because it shifts `Tracked` to cell 5 and breaks the cell-4 invariant across the 8 thematic files; review state is captured by the `untracked` token in `Tracked` instead).

## Risks and mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| `gh api` rate-limiting during a large `/triage` run | low (after Issue C bulk-fetch fix) | Issue C uses one bulk `gh issue list` call (5000/h limit, well within budget). |
| `gh issue list --limit 500` truncates if project surpasses 500 issues | low today (64 live) | Pagination watchdog: warn at ≥ 0.9× limit (Issue C AC7). When triggered, user can raise limit or design phase implements pagination. |
| `_inbox.md` grows unbounded if `/triage` runs infrequently | low | Threshold trigger (default 3) reminds the user; `/next` *Candidates needing `/triage`* section keeps inbox visible. The new AGENTS.md axiom enforces write-discipline. |
| Step 12's auto-extraction misclassifies rows in specs that diverge from the standard heading shape | high (≥ 6 shapes verified, mix per-section) | Issue A2 *Preconditions* require a full audit of all 55 `done/*.spec.md` (and possibly `done/*.design.md`) section shapes before implementation. Parser emits warnings on unrecognised shapes; does not block Step 12. Synthetic fixtures cover each shape. |
| Backfill produces `_inbox.md` with hundreds of rows that overwhelm `/triage`'s first run | medium | **Dedupe by `Source` link** removes the older 22 specs' already-harvested rows (cuts ~50 rows). Backfill walks specs in chronological order (oldest first); `/triage` can be run iteratively, draining a subset per session. The default threshold (3) makes `/triage` re-trigger frequently. **No pre-sort to thematic files** — uniform code path, same as forward-going Step 12 behaviour. |
| Author of Step 12 declines to run `/triage` and `_inbox.md` becomes a dumping ground | low | Discipline enforced by the AGENTS.md axiom (Issue A1) and by `/next`'s *Candidates needing `/triage`* visibility. Mirrors how `learnings.md` discipline works today. |
| Conflict resolution in Issue C produces an `update issue` decision that overwrites a human's web-UI edit | medium | `update issue` decisions surface a diff preview to the user before any `gh issue edit`. Bridge is opt-in per row, never silent. AC6 enforces this. |
| `widget-backlog.md`'s schema deviation breaks parser | medium | Issue B design enumerates parser rules per file. Widget-backlog tracked refs go in `Notes` cell (no 4th column added). 🟡 v2 → untracked-candidate semantics applied without rewriting the file format. |
| Concurrent web-UI edits to deferred md files during a `/triage` session produce silent overwrites at commit time | low | `/triage` re-reads each file immediately before its rewrite step; aborts with diff if the row text no longer matches the read snapshot (AC8). Mtime is **not** part of the check (filesystem and editor behaviour make it unreliable; content-snapshot is stronger). |
| `triage-runner` agent file diverges from `self-improve.md`'s established opus discipline | low | Issue B design phase reads `self-improve.md` and explicitly enumerates similarities and differences. The plan already tracks the divergence (mutation scope) in *Locked-in decisions*. |
| Multi-PR ordering enforcement is honour-system only — B's PR can technically merge before A1 | low | No CI gate. Mitigation: cite the sequencing diagram in each umbrella issue body; reviewer enforces. AC chains across issues make out-of-order merges immediately visible (B AC5 references A2's backfill — won't pass without A2). |
| Anthropic deprecates Opus 4.7 mid-rollout | low | Both `/improve` and `/triage` use opus; if model is retired, both skills break in lockstep. Mitigation: keep `model: opus` in agent frontmatter abstract enough to swap to next-gen (the existing convention already does this — `.claude/agents/self-improve.md` says `model: opus` not `model: claude-opus-4-7`). |
| Backfill scope ambiguity: should designs be walked too? | medium | Issue A2 *Preconditions* explicitly include the audit of `done/*.design.md`. `signals-slots.md` shows at least one design-sourced row in current data, so the audit will likely confirm yes. |
| `_inbox.md` rows double-handled by both promotion sweep and drain step | resolved | Issue C *Exception* carves `_inbox.md` `—` rows out of the sweep; drain step is canonical. AC5 verifies one prompt per row, not two. |

## What this plan does NOT decide

- Concrete title / body wording for the four umbrella issues (drafted at filing time, with user approval per issue).
- Whether each umbrella issue is `/task`-ed sequentially or whether some are bundled.

## Approval gate

This plan exited revision-3 on 2026-05-10 and completed on 2026-05-11:

1. ✅ User approved the four-umbrella-issue shape (A1 / A2 / B / C).
2. ✅ User approved the locked-in decisions.
3. ✅ Umbrella issues filed: #202 (A1) · #203 (A2) · #204 (B) · #205 (C).
4. ✅ **Meta-plan complete: all four umbrella issues merged** (A1 #202, A2 #203, B #204, C #205). File moved to `done/` in the C PR's Step 12 commit.

## Cross-references

- Source interview: this conversation, 2026-05-10.
- Subagent review #1 (rev 1 trigger): opus subagent, clean-sheet review against the original draft. Verdict: ITERATE; 5 major findings + 8 minor + 6 concrete edits applied.
- Subagent review #2 (rev 2 trigger): opus subagent, clean-sheet re-review of revision 1. Verdict: ITERATE; 5 major findings + 13 minor + 5 new-issues-from-revision + 6 open questions. Major fixes (spec count 109→55, umbrella count three→four, ≥ 5 shapes not 3, false `Tracked` claim, axiom-prose underspec, A1 forward-references) and 4 user-decided open questions (backfill scope, no-pre-sort, defer `_inbox.md` to A2, threshold default 3) applied.
- Subagent review #3 (rev 3 trigger): opus subagent, clean-sheet re-review of revision 2. Verdict: ITERATE → GO after edits 1–3. Found 1 major (`_inbox.md` 5-col schema broke cell-4-Tracked invariant) + 3 minor (backfill duplication, 100-row threshold derivation, label-decision creep) + 6th spec shape missed. All 7 suggested edits applied + 3 user-decided open questions (4-col schema, dedupe by Source, fixtures-as-AC).
- Meta-plan precedent: [`2026-05-08-instruction-file-rewrite.md`](2026-05-08-instruction-file-rewrite.md).
- Mirrored skill (with documented divergences): [`.claude/skills/improve/SKILL.md`](../../.claude/skills/improve/SKILL.md), [`.claude/agents/self-improve.md`](../../.claude/agents/self-improve.md).
- Source-of-truth files surveyed: `gh issue list --state all`; `ai-docs/plans/INDEX.md`; `ai-docs/deferred-items.md`; `ai-docs/deferred/*.md` (8 thematic + `widget-backlog.md`); `.claude/skills/{next,task,interview,ai-audit,improve}/SKILL.md`; `.claude/agents/self-improve.md`.
- v1 quartzite UI-designer track (out of scope for this plan): tracked separately; surfaces in `INDEX.md` *Suggested next steps* #3 (real `Painter` impls) and is dependency-blocked by `paint-style` ✅ + `widgets` ✅ already shipped.
