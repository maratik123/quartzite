# Design: AGENTS.md proactive extraction (clear 35k early-warning band)

**Issue:** #418
**Date:** 2026-05-16

## Approach

### Chosen solution

Two markdown-only commits on a single feature branch. **Commit 1 bundles both destination-file creations** (`agent-docs-index.md` + `api-naming.md`) **alongside the `§ Agent Docs` body collapse**, so the only AGENTS.md mutation in Commit 1 is a shrink. **Commit 2 carries the `§ API Naming` body collapse + the two new `§ Agent Docs` rows for the new reference pages.** AGENTS.md size is monotonically non-increasing at every commit boundary; new-file additions don't touch AGENTS.md byte count.

**Commit 1 — both destination files created + `§ Agent Docs` body collapse (spec § Scope step 1, expanded to fold the `api-naming.md` creation forward per round-2 design-review):**

1. **Create `ai-docs/agent-docs-index.md`** with the established extracted-reference-page shape:
   - Line 1: `# Agent Docs — verbose row index`
   - Line 3 (first paragraph, single sentence): cross-link back to `[AGENTS.md § Agent Docs](../AGENTS.md#agent-docs)` modelled verbatim on `ai-docs/dependency-versions.md` line 3 ("This page extracts the live-lookup table from [`AGENTS.md` § Dependency Versions](../AGENTS.md#dependency-versions). The AXIOM headline and pinning bullets stay in AGENTS.md.").
   - Single ATX `## Agent doc rows` heading housing the extracted verbose bodies. Inside, each row's verbose description sits under its own `### <path>` sub-heading (using inline-code path) — this matches the row identity used in the AGENTS.md table and gives readers a per-row anchor target without inventing new section names. Row bodies that are already one-line stubs in AGENTS.md (e.g., `ai-docs/context.md` → "Project context — read on demand") get a single `### ai-docs/context.md` sub-heading + the same prose carried through verbatim, so the index is uniform regardless of body length.
2. **Create `ai-docs/api-naming.md`** with the established shape:
   - Line 1: `# API Naming`
   - Line 3 (first paragraph, single sentence): cross-link back to `[AGENTS.md § API Naming](../AGENTS.md#api-naming)` — same shape as `ai-docs/workflow.md` line 3.
   - Sub-structure:
     - `## The _unchecked AXIOM` — houses the verbatim AXIOM blockquote (current AGENTS.md lines 78–85) including the "Your fn... | Suffix" table verbatim. (Heading text deliberately starts with "The" so the GFM slug is `#the-_unchecked-axiom` — a slug that does NOT start with a leading underscore. See § Risks → "Slugger rule for leading underscores" for the why.)
     - `## Naming rules` — houses the bullet-prose currently at AGENTS.md lines 87–92 verbatim (the four bullets following the AXIOM block).
   - Anchor slug verification — see § Risks for the slugger ruleset and the local verification recipe.
3. **Edit `AGENTS.md § Agent Docs` in the same commit:**
   - Keep the `## Agent Docs` heading and the table header (`| Path | Purpose |` + `|---|---|`).
   - For every existing row, replace column 2 with a `≤ 80-char` one-line description stub. **Target average ≤ 65 chars** (not just the 80-char hard cap) to close the budget arithmetic — see § Risks. The row's path (column 1) stays verbatim — that is the navigable enumeration `/next` and other consumers grep against.
   - The lone in-table cross-link to the destination is carried as an anchored prose link inside `ai-docs/agent-docs-index.md`'s first paragraph, NOT injected into the AGENTS.md table cells (keeps stubs short and avoids per-row link clutter).

Net AGENTS.md delta in Commit 1 ≈ **−3,930 chars** (Agent Docs body collapse only; the two new-file creations don't touch AGENTS.md). AGENTS.md ~39,960 → ~36,030.

**Commit 2 — `§ API Naming` body collapse + new `§ Agent Docs` rows (spec § Scope step 2):**

1. **Edit `AGENTS.md § API Naming`:**
   - Keep the `## API Naming` level-2 heading.
   - Replace the entire body (lines 78–92) with a single two-line stub:
     ```
     See [`ai-docs/api-naming.md` → The _unchecked AXIOM](ai-docs/api-naming.md#the-_unchecked-axiom) for the `_unchecked` AXIOM, the per-shape action table, and the rule bullets.
     ```
2. **Add two new rows to `AGENTS.md § Agent Docs`** in the same commit (per spec § Scope step 2 last bullet):
   - `| ai-docs/agent-docs-index.md | <≤ 80-char stub describing the index> |`
   - `| ai-docs/api-naming.md | <≤ 80-char stub describing the API-Naming reference> |`
   - Insertion point: immediately after the existing `ai-docs/workflow.md` row (alphabetical clustering with the other `ai-docs/<topic>.md` reference pages). The exact wording of the stubs is implementation-time; the design binds only the ≤ 80-char cap and the stable row order ("workflow.md, corrections-log.md, key-decisions.md, dependency-versions.md, …" → insert the two new ones at the natural alphabetical position).

Net AGENTS.md delta in Commit 2 ≈ **−2,210 chars** (API Naming body collapse minus the two new rows). AGENTS.md ~36,030 → ~33,820.

Monotonic-decreasing invariant intact: 39,960 → 36,030 → 33,820. AC5 (≤ 34,000) ✓ with ≥ 180-char safety margin (conservative; see § Risks "Headroom note" for actuals).

### Anchor slugs

GitHub's slugger rule is well-understood: lowercase the heading text, strip code-fence backticks and most punctuation, collapse interior whitespace to single hyphens. Underscores in source are preserved as-is in the slug. A leading `_` in the heading text would generate a slug that **starts** with `_`, which some markdown renderers don't allow as an HTML `id` value. To dodge that compatibility risk entirely, the design renames the destination heading from a hypothetical bare `## _unchecked AXIOM` to `## The _unchecked AXIOM` — the rendered slug becomes `#the-_unchecked-axiom`, which starts with a letter and is universally valid. The underscore stays preserved in the middle of the slug. Concrete slugs this design binds:

| Heading (in destination) | Rendered slug | Cross-link from AGENTS.md |
|---|---|---|
| `## Agent doc rows` (in `ai-docs/agent-docs-index.md`) | `#agent-doc-rows` | (none — index is reachable via the new `§ Agent Docs` row; per-row sub-headings `### ai-docs/<path>` give finer-grained targets if needed by future cross-references) |
| `## The _unchecked AXIOM` (in `ai-docs/api-naming.md`) | `#the-_unchecked-axiom` | `See [ai-docs/api-naming.md → The _unchecked AXIOM](ai-docs/api-naming.md#the-_unchecked-axiom)…` |
| `## Naming rules` (in `ai-docs/api-naming.md`) | `#naming-rules` | (optional secondary anchor for the rule bullets) |

The slugs are verified against GitHub's renderer **locally** before commit per § Risks → "Slugger rule for leading underscores".

### Rejected alternatives

1. **Single combined extraction file `ai-docs/agents-md-overflow.md`.** Rejected: violates the established `ai-docs/<topic>.md` convention (`workflow.md`, `corrections-log.md`, `dependency-versions.md`) where each file maps to one source section. Two destinations align with the spec's Q2 user guidance.
2. **Original three-commit / 2+2 grouping (Commit 1 = `agent-docs-index.md` + `§ Agent Docs` body collapse; Commit 2 = `api-naming.md` creation + `§ API Naming` collapse + two new rows; Group A = subtasks 1+2 / Group B = subtasks 3+4).** Rejected: violates `.claude/agents/design.md § Rules → handoff-grouping (b)`: non-terminal groups MUST be exactly 3. The original split made Group A size 2 (non-terminal, ≠ 3). The fix is to fold `api-naming.md` creation forward into Commit 1 — it's a new-file addition that doesn't touch AGENTS.md, so the monotonic-decreasing invariant still holds. Commit 2 then becomes a single-subtask commit (size 1, terminal — within `1..=3` ✓). See § Handoff plan for the corrected grouping.
3. **Three separate commits (split `api-naming.md` creation from the `§ API Naming` collapse).** Rejected: if `api-naming.md` lands in a commit BEFORE its referencing stub link, the stub doesn't yet exist to verify against — fine in isolation. But splitting it from the `§ API Naming` collapse adds a third commit boundary without any size-budget win: the `api-naming.md` creation doesn't change AGENTS.md, so folding it forward into Commit 1 (the present chosen solution) is strictly simpler.
4. **Whole extract of `§ Agent Docs` (move the entire table out).** Rejected: violates AC10 — `.claude/skills/next/SKILL.md` greps for `ai-docs/<path>` enumeration and would lose its index. Spec § Key decisions row 3 explicitly mandates partial extract.
5. **Partial extract of `§ API Naming` (keep the AXIOM blockquote in AGENTS.md, move the bullets).** Rejected: the AXIOM is the headline rule; splitting it from its enforcement bullets fragments the reader's first-pass scan. Spec § Key decisions row 4 mandates whole extract.
6. **Insert per-row sub-heading bodies as `#### <path>` (level-4) instead of `### <path>` (level-3).** Rejected: makes the `agent-docs-index.md` rendering visually flatter and less navigable; established convention in `ai-docs/key-decisions.md` uses `## <topic>` per row (level-2), but our index has an outer `## Agent doc rows` umbrella that calls for one more level of nesting, hence `### <path>`.
7. **Bare `## _unchecked AXIOM` heading (slug `#_unchecked-axiom`, leading underscore).** Rejected: some markdown renderers refuse HTML `id` attributes that start with `_`. The renamed `## The _unchecked AXIOM` (slug `#the-_unchecked-axiom`) preserves the human-readable underscore in the middle of the slug while guaranteeing a letter-leading slug.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `ai-docs/agent-docs-index.md` with first-paragraph cross-link back to `AGENTS.md § Agent Docs`, ATX `## Agent doc rows` umbrella heading, and one `### ai-docs/<path>` sub-heading per existing row carrying the verbose body verbatim (move + light restructure permitted). For rows whose current body is already a one-line stub, carry the prose through unchanged. | `ai-docs/agent-docs-index.md` (new) | — |
| 2 | Create `ai-docs/api-naming.md` with first-paragraph cross-link back to `AGENTS.md § API Naming`, `## The _unchecked AXIOM` heading housing the verbatim AXIOM blockquote + table, and `## Naming rules` heading housing the verbatim bullet-prose. **(Folded forward from old Commit 2 into Commit 1 — round-2 design-review fix; creates a new file, no AGENTS.md size delta.)** | `ai-docs/api-naming.md` (new) | 1 |
| 3 | Replace each existing `§ Agent Docs` row's column 2 in `AGENTS.md` with a `≤ 80-char` one-line description stub. Target average `≤ 65 chars` per row to close the AC5 budget. Heading + table header + all column-1 paths stay verbatim. Bundled with subtasks 1 + 2 into **Commit 1** so AGENTS.md's only mutation in this commit is a shrink. | `AGENTS.md` | 1 |
| 4 | Edit `AGENTS.md § API Naming`: keep the `## API Naming` heading, collapse the body (current lines 78–92) to a one-line stub-with-anchored-link to `ai-docs/api-naming.md#the-_unchecked-axiom`. **In the same commit**, add two new rows to `§ Agent Docs` for `ai-docs/agent-docs-index.md` and `ai-docs/api-naming.md` at the alphabetical insertion point among the other `ai-docs/<topic>.md` reference rows. | `AGENTS.md` | 2, 3 |

Commit boundaries:
- **Commit 1** = subtasks 1 + 2 + 3 (per spec § Scope step 1, plus `api-naming.md` creation folded forward per round-2 design-review). Net AGENTS.md delta ≈ −3,930 chars. AGENTS.md ~39,960 → ~36,030.
- **Commit 2** = subtask 4 (per spec § Scope step 2). Net AGENTS.md delta ≈ −2,210 chars. AGENTS.md ~36,030 → ~33,820.

## Handoff plan

Per `.claude/agents/design.md` § Rules → handoff-grouping, M = 4 (subtasks 1–4) requires a `## Handoff plan`. The Round-1 design's 2 + 2 split was rejected by design-review: non-terminal groups MUST be exactly 3. **Corrected grouping (round 2): 3 + 1.**

- **Group A (entry, non-terminal):** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` enters Group A with fresh context. Group A = subtasks 1, 2, 3 (3 consecutive subtasks; exactly the non-terminal cap of 3 ✓) — **Commit 1**: both destination files are created AND the `§ Agent Docs` body collapse lands in one commit. Net AGENTS.md delta ≈ −3,930 chars.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context and the measured `wc -c AGENTS.md` after Commit 1 as ground truth for the Commit 2 budget arithmetic.
- **Group B (terminal):** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Subtask 4 only (1 subtask; within the `1..=3` terminal-size range ✓) — **Commit 2**: `§ API Naming` body collapse + two new `§ Agent Docs` rows for the new reference pages. Net AGENTS.md delta ≈ −2,210 chars.

Group-size summary (mandatory four sub-points per `.claude/agents/design.md` § Rules → handoff-grouping):
- **(a) When grouping is required** — `every M ≥ 1`. This design's M = 4, so the section is mandatory; both groups are enumerated.
- **(b) Maximum group size** — `3 consecutive subtasks`. Group A (non-terminal) is exactly 3 ✓; Group B (terminal) is 1.
- **(c) Handoff destination** — `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Named in prose at every boundary: Group A entry, between Group A and Group B (i.e., between Commit 1 and Commit 2), and Group B entry.
- **(d) Terminal-group sizing** — `1..=3`. Group B (terminal) is size 1, within range ✓.

## Risks

- **Risk: 80-char stub cap is a hard max, not a budget target — naively writing ≤ 80-char stubs busts AC5.**
  Mitigation: target an average `≤ 65 chars` per row across the 25 existing rows. Arithmetic (live row count = 25; the live per-row path-column width averages ~31 chars, smaller than the 50-char conservative figure used here): 25 rows × (50 path + 3 ` | ` + 65 stub + 2 ` |`) = 25 × 120 = 3,000 chars after collapse, vs. the current ~6,930-char table body. Net `§ Agent Docs` reduction in Commit 1 ≈ **−3,930 chars**. AGENTS.md ~39,960 → ~36,030 after Commit 1. AC5 (≤ 34,000) is not yet satisfied at Commit 1 boundary — that's by design; Commit 2 lands the remaining ≈ −2,210 chars (API Naming collapse minus the two new rows). **AC5 is the binding gate** at branch-end; the 80-char cap is the per-row hard max. **Headroom note:** because the live per-row path width averages ~31 chars (not 50), the implementer's *actual* per-row budget for the stub is ~84 chars at the 65-char-average target, OR equivalently the 65-char-average target produces a per-row line ~19 chars shorter than the arithmetic predicts → projected branch-end AGENTS.md is closer to **~33,600 chars** than the conservative **~33,820 chars** the headline arithmetic states. Both pass AC5 (≤ 34,000).

- **Risk: Slugger rule for leading underscores — a heading starting with `_` would generate a slug starting with `_`, which some markdown renderers refuse as an HTML `id`.**
  Mitigation: design renames the destination heading from a hypothetical bare `## _unchecked AXIOM` to `## The _unchecked AXIOM`. The rendered GFM slug becomes `#the-_unchecked-axiom` — letter-leading, universally valid as an HTML `id`, and preserves the human-readable `_unchecked` in the middle. All references in this design (§ Approach, § Decomposition subtask 4, § File touch list, § Test Design) use the new heading + slug. Local pre-push verification: render `ai-docs/api-naming.md` with `pandoc -f gfm -t html ai-docs/api-naming.md | grep -oE 'id="[^"]+"'` (or any GFM-compatible renderer) and confirm the emitted `id="…"` matches the planned slug `the-_unchecked-axiom`. `/ai-audit` Phase 2 Step 2.6.4 is the canonical post-merge verifier per spec § Technical constraints, but local pre-push is the cheaper gate.

- **Risk: Per-commit ordering inversion — accidentally landing the `§ API Naming` collapse before `api-naming.md` exists.**
  Mitigation: explicit commit ordering bound by the decomposition (subtasks 1 + 2 + 3 → Commit 1, subtask 4 → Commit 2). `api-naming.md` is created in Commit 1 (subtask 2) — strictly BEFORE Commit 2's `§ API Naming` collapse that references it. The `/task` implementation step verifies `wc -c AGENTS.md` after each commit and the implementer should compare against the previous commit's number to confirm monotonic-decreasing — AC6 enforces this at PR time.

- **Risk: AC11 content-fidelity drift — "light restructure permitted" tempts the implementer to also tighten prose.**
  Mitigation: pre/post diff during self-review must show only formatting / cross-link prose changes — explicit per-spec AC11. The implementer should produce the destination file via copy-then-add-headings, NOT retype, to preserve binding semantics verbatim. `self-review` agent spawned per `/task` Step 10 is the catch-net.

- **Risk: Stable row ordering for the two new `§ Agent Docs` rows.**
  Mitigation: insert at the alphabetical position among other `ai-docs/<topic>.md` reference pages (after `workflow.md`, before `corrections-log.md` if alphabetical strictness is desired; OR clustered immediately after `workflow.md` to match the existing pattern where `workflow.md`, `corrections-log.md`, `key-decisions.md`, `dependency-versions.md` already appear in non-strict-alphabetical order). The design binds the **cluster location** ("among the other `ai-docs/<topic>.md` reference rows"), not the exact ordering within that cluster — implementer's discretion within the cluster.

- **Risk: AC10 propagation grep — a downstream skill might inline-quote `§ API Naming` rule text rather than cross-reference the heading.**
  Mitigation: pre-investigation showed `.claude/agents/review-findings.md:52` and `.claude/agents/self-review.md:71` cross-reference `AGENTS.md "API Naming"` by section heading (not by body content). The `## API Naming` heading STAYS in AGENTS.md per the stub policy, so these references remain valid. `.claude/skills/next/SKILL.md` greps for `ai-docs/<path>` enumeration which is preserved by the partial-extract policy. The AC10 grep is the explicit catch-net for any consumer the design phase missed.

## Test Design

Pure instruction-file / markdown-only PR. No Rust tests added. Verification is gate-based, not test-based:

- **AC1 / AC2 verification** — manual diff of destination-file content vs. source AGENTS.md lines. `git diff` between the AGENTS.md state at merge-base and the destination-file content at HEAD must show only formatting + cross-link prose changes (AC11).
- **AC3 verification** — `grep '| ai-docs/' AGENTS.md | wc -l` post-commit-2 returns N+2 where N is the count at merge-base. (Spot-check via the table-row enumeration.)
- **AC4 verification** — `grep -A1 '^## API Naming$' AGENTS.md` post-commit-2 returns the heading + the one-line stub-link line + a blank line; total body lines under `## API Naming` ≤ 3.
- **AC5 verification** — `wc -c AGENTS.md` post-commit-2 reports ≤ 34,000. Conservative projection ~33,820 (180-char margin); realistic projection ~33,600 (400-char margin given the live per-row path-column width is ~31 chars, not the 50-char headline figure).
- **AC6 verification** — per-commit iteration: `for sha in $(git rev-list <merge-base>..HEAD); do echo "$sha $(git show $sha:AGENTS.md | wc -c)"; done` confirms each value < 40,000 AND monotonically non-increasing. Expected sequence: 39,960 (merge-base) → ~36,030 (Commit 1) → ~33,820 (Commit 2).
- **AC7 verification** — `/ai-audit` Phase 2 Step 2.6.4 (or local equivalent: render the destination files, extract heading IDs, grep that every `[…](ai-docs/<topic>.md#anchor)` link introduced by the PR has a matching ID). For Commit 1, the only AGENTS.md→destination link introduced is the `§ Agent Docs` row(s) — none use anchored sub-section links at this point. For Commit 2, the AGENTS.md `§ API Naming` stub link `ai-docs/api-naming.md#the-_unchecked-axiom` MUST resolve to the heading `## The _unchecked AXIOM` created in Commit 1's subtask 2.
- **AC8 verification (pre-merge framing)** — AC8's binding evidence (post-merge `/ai-audit` Phase 2 Sub-check 9 returning empty) is produced by the **next** `/ai-audit` invocation AFTER this PR merges, not at PR-open time. To satisfy AC8 at PR-open time without waiting for post-merge: the PR body records (a) `wc -c AGENTS.md` at the `master`-merge-base (`39,960`) vs HEAD (`~33,820` projected); (b) a verbatim quote of the current `master` `/ai-audit` Sub-check 9 finding (`AGENTS.md: 39,960 chars — early warning`); (c) the predicted post-merge Sub-check 9 result (empty against AGENTS.md, since `~33,820 < 35,000` clears the early-warning band). The actual post-merge Sub-check 9 PASS-confirmation lands in the next `/ai-audit` invocation after merge. This decouples PR-open-time AC verification (predictable evidence) from post-merge demonstrator confirmation (binding evidence the next audit produces).
- **AC9 verification** — workspace gates: `cargo build`, `cargo test`, `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. Expected: all pass green; the PR introduces no Rust changes, so the gates are sanity checks confirming none slipped in.
- **AC10 verification** — `grep -rn "ai-docs/agent-docs-index\|ai-docs/api-naming\|§ Agent Docs\|§ API Naming\|_unchecked AXIOM\|the-_unchecked-axiom" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md` — every match either points at the new destination via anchored link OR is unaffected (heading-only reference). No downstream file should inline-quote a body line moved by this PR.
- **AC11 verification** — pre/post diff of each moved body, spot-checked by `self-review` during `/task` Step 10.

### File touch list

| Commit | File | Action |
|---|---|---|
| 1 | `ai-docs/agent-docs-index.md` | Create (subtask 1) |
| 1 | `ai-docs/api-naming.md` | Create (subtask 2) — heading `## The _unchecked AXIOM` (slug `#the-_unchecked-axiom`) + `## Naming rules` |
| 1 | `AGENTS.md` | Edit `§ Agent Docs` table column 2: replace verbose row bodies with ≤ 80-char stubs (subtask 3) |
| 2 | `AGENTS.md` | Edit `§ API Naming`: collapse body to one-line stub linking `ai-docs/api-naming.md#the-_unchecked-axiom`; add two new `§ Agent Docs` rows for `ai-docs/agent-docs-index.md` + `ai-docs/api-naming.md` (subtask 4) |

## Open questions

None. The spec resolved all open questions at draft time (Q1 extraction targets, Q2 destination filenames, Q3 commit decomposition); the design-time slugger and stub-budget calls are made in § Approach + § Risks above. The round-2 design-review fixes (3+1 handoff grouping, `#the-_unchecked-axiom` slug rename, AC8 pre-merge framing) are folded into the body.
