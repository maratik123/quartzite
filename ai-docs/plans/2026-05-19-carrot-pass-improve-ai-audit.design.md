# Design: Carrot Pass for `/improve` and `/ai-audit` — PR 1 (Phase 1)

**Issue:** #491
**Spec:** `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.spec.md`
**Date:** 2026-05-19

## Approach

PR 1 lands Phase 1 only — the schema migration plus the section rename plus the worked-example carve-out plus the Propagation-Rule row update plus the sync-group fan-out. It is the hard prerequisite for Phases 2–4 (PR 2) and for Phase 5 (PR 3).

The design centers on five mechanical, additive edits that respect Boundary Rule 1 (append-only `learnings.md`) and Boundary Rule 2 (no instruction-file fan-out triggered by a fresh learning entry). All edits are instruction-surface only; no Rust code changes. The merge contract still requires the four Rust gates (`cargo build` / `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt -- --check` / `RUSTDOCFLAGS="…" cargo doc --no-deps --workspace --all-features`) to pass, per AC18.

**Chosen approach.** Five-piece additive edit, committed as a single feature-branch commit (or split into 2 commits if the worked-example retro-add is staged separately for diff legibility — see Risks). All five pieces fan out in lock-step per the Propagation Rule, with the rename's reach enumerated by a programmatic `grep -rn -E 'Corrections.Log|corrections-log' AGENTS.md ai-docs/ .claude/` sweep before the commit (AC14).

The five pieces:

1. **`AGENTS.md` schema + section rename.**
   - Rename `## Corrections Log` → `## Learning Log` (line 279).
   - Add `**Kind:** correction | validation` row to the *Entry format* block (line 319 region), with a paragraph noting the default (`correction` when omitted) and that existing entries need NO rewrite.
   - Update the Propagation-Rule row at line 216 from *"Corrections Log section (Boundary rules 1 / 2, entry format, `Escalated?` semantics)"* to *"Learning Log section (Boundary rules 1 / 2, entry format incl. `Kind:`, `Escalated?` semantics, 🌱 verdict from `/ai-audit`)"*. **Decision (was O3):** update the existing row IN-PLACE rather than adding a second row. Sync-group membership (`self-improve.md` + `learnings-escalation-audit.md`) is unchanged; only the keyword set expands. A second row would create row-duplication confusion.
   - Update the internal cross-references that point at `ai-docs/corrections-log.md` (lines 251, 281, 292, 313, 315, 327) — wording stays substantively the same; the file-name reference `ai-docs/corrections-log.md` stays (file not renamed); only the prose phrase *"§ Corrections Log"* becomes *"§ Learning Log"* where it appears. Line 251 is the Agent Docs table row body (`Extracted § Corrections Log carve-outs + field glossary`) — caught by Subtask 5's grep sweep anyway, but listed explicitly here for exhaustiveness at design time.
   - Add a narrow named call-out under Boundary rule 1 documenting the worked-example carve-out: *"One-off carve-out — 2026-05-19 compaction-recovery-protocol entry retro-tagged `Kind: validation` per PR #491 Phase 1; recorded in that entry's `Superseded by:` line. Named, narrow, audit-traced; NOT a precedent for further bulk edits."* (AC13). **Decision (was O2):** the call-out is rendered as a one-paragraph text-form callout — a third indented paragraph under Boundary rule 1, consistent with the existing AXIOM-blockquote + indented Exception-paragraph shape at AGENTS.md ~283–292. NOT a table row, NOT a separate AXIOM block.
   - Trigger threshold line at line 331 (*"Run `/improve` when ≥3 unescalated entries accumulate"*) stays **unchanged** in PR 1. Phase 4 (PR 2) rewords it to cover both `Kind`s and the 🌱 flag. The Phase-4 sync-group lock-step with `improve/SKILL.md` is recorded in the spec; PR 1 does NOT touch the threshold.

2. **`ai-docs/corrections-log.md` field glossary + carve-out trail + section-name references.**
   - Add a `Kind:` paragraph to the *Entry format — field glossary* section (matches the AGENTS.md addition), naming the two values + the default-when-omitted rule + the bi-directional supersession convention (validation disconfirmed → new `Kind: correction` whose `Superseded by:` references the original validation — per spec Key Decisions row *Validation supersession semantics*).
   - Under *Boundary rule 1 Exception* (or its referenced section, per AC13), add a named-carve-out paragraph mirroring the AGENTS.md call-out. Includes the entry date, slug, the commit hash placeholder (filled at commit time), and a one-line rationale.
   - Update the two `[`AGENTS.md` § Corrections Log](../AGENTS.md#corrections-log)` references at lines 3 and 45 to `[`AGENTS.md` § Learning Log](../AGENTS.md#learning-log)`. The H1 `# Corrections Log — reference` at line 1 becomes `# Learning Log — reference`. The phrase "corrections log" in body prose is updated to "learning log" where it refers to the section by name; passages referring to the artefact category (e.g., "the corrections log records corrected behaviour") may stay or be updated per editorial taste — design defers to the most-local readable wording at edit time.
   - **File name stays `corrections-log.md`.** Renaming the file would force git-side churn across every cross-reference and risk breaking anchor links from the three `pr-*-failed` SKILL files. The section header rename is sufficient per the spec.

3. **`.claude/agents/self-improve.md` section-name references + entry-format reference.**
   - Step 1 (Inputs / Patterns): line 16 reference *"full corrections log"* → *"full learning log"*; section-name references in Step 5 backfill rules (line 88) *"AGENTS.md § Corrections Log → Boundary rule 1 → Exception"* → *"AGENTS.md § Learning Log → Boundary rule 1 → Exception"*; line 108 *"Corrections-Log sync-group sister file"* → *"Learning-Log sync-group sister file"*.
   - **No Carrot pass added here in PR 1.** That is Phase 2 / PR 2. The agent file is touched in PR 1 ONLY for the rename + (optionally) a forward-pointer comment noting Phase 2 will add the Carrot pass. The forward-pointer is OPTIONAL — recommend skipping per YAGNI.

4. **`.claude/agents/learnings-escalation-audit.md` section-name references.**
   - Frontmatter `description` (line 3) *"AGENTS.md § Corrections Log Boundary rule 1 Exception"* → *"AGENTS.md § Learning Log Boundary rule 1 Exception"*.
   - Body references at lines 17 (*"full corrections log"*), 24 (*"Per AGENTS.md "Corrections Log":"*), 96 (*"AGENTS.md § Corrections Log → Boundary rule 1 → Exception"*), 137 (*"AGENTS.md § Corrections Log Boundary rule 2 Exception"*) — all updated.
   - **No `🌱` verdict added here in PR 1.** That is Phase 3 / PR 2.

5. **Worked-example carve-out commit (Boundary-Rule-1 named exception).**
   - Append `**Kind:** validation` line to the 2026-05-19 *compaction-recovery protocol in skill files works* entry between the `**Rule:**` line (1303) and the `**Escalated?**` line (1305). **Decision (was O1):** `**Kind:**` is placed BETWEEN `**Rule:**` and `**Escalated?**`. Rationale: this groups content fields (What happened, Rule, Kind) before metadata fields (Escalated, Superseded by). The entry-format definition added to AGENTS.md (piece 1) and to `ai-docs/corrections-log.md` field glossary (piece 2) MUST reflect this ordering: *date heading → **What happened:** → **Rule:** → **Kind:** → **Escalated?** → **Superseded by:** (optional)*.
   - Insert a new `**Superseded by:** PR #<N> — Phase 1 worked-example retro-add of `Kind: validation`; named Boundary-Rule-1 carve-out (Q1 resolution).` line after the `Escalated?` line, matching the field-glossary's `Superseded by:` placement rule (after `Escalated?`).
   - The PR number `<N>` is filled at commit time; the commit hash referenced in the carve-out call-out (pieces 1 + 2) is back-filled in a follow-up amend or — preferred — written as `PR #<N>` (forward-referencing the merged PR) so no amend is needed. **Recommend `PR #<N>`** — date+slug supersession references the carve-out itself is satisfied by the PR-ref.

### Rejected alternatives

- **Rename the file `ai-docs/corrections-log.md` → `ai-docs/learning-log.md`.** Rejected. Three downstream SKILL files (`pr-commented`, `pr-ci-failed`, `master-ci-failed`) anchor-link into `corrections-log.md#forbidden-reasoning-…`. A file rename multiplies the fan-out diff and risks breaking anchors. The spec explicitly says *"rename the section, propagate to cross-references"* — the file name is not in the fan-out target list.
- **Bulk retro-tag every existing entry with `Kind: correction`.** Rejected. Spec § *Out of scope* explicitly forbids backfill; default-when-omitted handles legacy entries. Bulk edit also violates Boundary Rule 1.
- **Add the Carrot pass to `self-improve.md` in PR 1.** Rejected. The spec's PR-slicing decision keeps Phase 2 (pass), Phase 3 (verdict + checklists), Phase 4 (threshold) as a co-dependent stacked PR 2. Mixing them into PR 1 couples the schema migration with the workflow change and bloats the diff. PR 1 lands the schema + rename ONLY.
- **Defer the worked-example carve-out to PR 2.** Rejected. The spec's AC12 + AC13 explicitly land the carve-out in Phase 1 to demonstrate the new schema. The retro-add IS the schema's adoption signal.
- **Split PR 1 into "rename" + "schema" sub-PRs.** Rejected. The schema (Kind:) is what the rename enables; splitting introduces an intermediate state where the section is renamed but no `Kind` field exists, confusing future readers.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | `AGENTS.md` § Corrections Log → § Learning Log rename + add `Kind:` to entry format + add Boundary-Rule-1 named carve-out call-out + update Propagation-Rule row (line 216) + update internal `§ Corrections Log` prose references. **NO threshold-line change** (deferred to Phase 4 / PR 2). | `AGENTS.md` | — |
| 2 | `ai-docs/corrections-log.md` H1 + section-name prose rename + add `Kind:` paragraph to field glossary + add Boundary-Rule-1 carve-out paragraph (mirrors AGENTS.md) + document the bi-directional `Superseded by:` convention for disconfirmed validations + update `#corrections-log` anchor refs (lines 3, 45) → `#learning-log`. | `ai-docs/corrections-log.md` | 1 |
| 3 | `.claude/agents/self-improve.md` + `.claude/agents/learnings-escalation-audit.md` + `ai-docs/agent-docs-index.md` + `.claude/skills/ai-audit/SKILL.md` rename-only references (line-by-line per § Approach pieces 3 + 4 + the prose-only fan-out targets in the file-touch map); NO behavioural change. The latter two files are mandatory fan-out per the Propagation Rule — included in PR 1 to keep the rename atomic. | `.claude/agents/self-improve.md`, `.claude/agents/learnings-escalation-audit.md`, `ai-docs/agent-docs-index.md`, `.claude/skills/ai-audit/SKILL.md` | 1 |
| 4 | Worked-example carve-out: append `**Kind:** validation` line + new `**Superseded by:** PR #<N> — …` line to the 2026-05-19 *compaction-recovery protocol in skill files works* entry in `ai-docs/learnings.md`. **Boundary-Rule-1 named exception**; this is the ONLY entry edited. | `ai-docs/learnings.md` | 1, 2 |
| 5 | Verification sweep: run `grep -rn -E 'Corrections.Log\|corrections.log\|Correction.Log' AGENTS.md ai-docs/ .claude/` and confirm every remaining hit is either (a) the file-name reference `ai-docs/corrections-log.md` (intentionally preserved) OR (b) an updated prose reference. Run `wc -c` on AGENTS.md to confirm < 40,000 chars (35,000 early-warning band acknowledged). Run the four Rust gates (AC18). | (read-only verification; no edits) | 1, 2, 3, 4 |

Total: 5 subtasks. Below the 7-task split-into-multiple-issues threshold; within tolerance for a single PR.

## Handoff plan

The design defines **M = 5 subtasks**, grouped into two consecutive groups of 3 + 2 (per `.claude/agents/design.md` § Rules → handoff-grouping: non-terminal groups MUST be exactly 3; terminal group within `1..=3`).

- **Group A:** subtasks 1–3 — initial implementation chunk (AGENTS.md schema + rename + Propagation-Rule row; `corrections-log.md` glossary + carve-out trail; rename-only edits to both Corrections-Log sync-group agents). Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at the start of Group A (per the every-group handoff contract).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — terminal group (2 subtasks; within the 1..=3 range). The worked-example carve-out (Boundary-Rule-1 named exception) plus the verification sweep + char-count + Rust gates.

## File-touch map

| File | Edit shape | Subtask |
|---|---|---|
| `AGENTS.md` | Section header rename; entry-format addition; carve-out call-out; Propagation-Rule row update; internal prose `§ Corrections Log` → `§ Learning Log` references | 1 |
| `ai-docs/corrections-log.md` | H1 rename; field-glossary `Kind:` paragraph; Boundary-Rule-1 carve-out paragraph; `Superseded by:` bi-directional convention paragraph; `#corrections-log` anchor refs → `#learning-log` | 2 |
| `.claude/agents/self-improve.md` | Section-name references at lines 16, 88, 108 (rename-only) | 3 |
| `.claude/agents/learnings-escalation-audit.md` | Frontmatter `description` + body references at lines 17, 24, 96, 137 (rename-only) | 3 |
| `ai-docs/learnings.md` | Append `**Kind:** validation` + new `**Superseded by:**` line to the 2026-05-19 *compaction-recovery protocol* entry ONLY. Boundary-Rule-1 named exception (AC13) | 4 |
| `ai-docs/agent-docs-index.md` | Line 19 `### ai-docs/corrections-log.md` + line 21 `*Corrections Log*` prose reference (file-name stays). **Mandatory** — include in PR 1 to keep the rename atomic per the Propagation Rule's fan-out discipline. | 3 |
| `ai-docs/context.md` | OPTIONAL — line 200 `corrections-log.md` reference is historical context (PR #324 narrative) and should NOT be rewritten (the rename happens AFTER PR #324). Leave as-is. | — |
| `ai-docs/agent-writing-style.md` | Line 173 `ai-docs/corrections-log.md` (covered-file-set bullet) — file name; stays. | — |
| `.claude/skills/ai-audit/SKILL.md` | Lines 156 (`#### L. Corrections-Log field coherence`), 158 (`AGENTS.md § Corrections Log's *Entry format*`), 171 + 195 (file-name `ai-docs/corrections-log.md` references), 261 (`AGENTS.md "Corrections Log" format`). **Mandatory — PR 1 scope: update prose references** (`Corrections-Log` → `Learning-Log`, `§ Corrections Log` → `§ Learning Log`). File-name `ai-docs/corrections-log.md` references stay. Include in PR 1 to keep the rename atomic per the Propagation Rule's fan-out discipline. The `Kind:` row addition to Checklist L is Phase 3 / PR 2. | 3 |

Total instruction-surface files edited in PR 1: **7 mandatory** (5 core + `ai-docs/agent-docs-index.md` + `.claude/skills/ai-audit/SKILL.md` for prose-only renames). The latter two are mandatory per the Propagation Rule's fan-out discipline; included in PR 1 to keep the rename atomic and to minimise grep-sweep noise in Phase 3 / PR 2.

**Files NOT touched in PR 1** (Phase 2/3/4 territory):

- `.claude/skills/improve/SKILL.md` — threshold restate is Phase 4.
- `.claude/agents/self-improve.md` Step 2 routing table + Step 6 inverted eval — Phase 2.
- `.claude/agents/learnings-escalation-audit.md` verdict set extension — Phase 3.
- `.claude/skills/ai-audit/SKILL.md` Checklist L row addition + Checklist M sub-check + Checklist N — Phase 3.

## Test Design

This task touches instruction-surface only. There are no Rust code changes and therefore no `#[cfg(test)]` modules to add or modify. The verification surface is mechanical and grep-based, per AC14 + AC18.

For each non-trivial subtask:

### Subtask 1 — AGENTS.md edits

- **Location:** N/A (instruction-file edit).
- **Verification:** `grep -n '## Learning Log' /home/syt/RustroverProjects/quartzite/AGENTS.md` returns exactly one hit. `grep -n '## Corrections Log' /home/syt/RustroverProjects/quartzite/AGENTS.md` returns zero hits. `grep -n 'Kind:' /home/syt/RustroverProjects/quartzite/AGENTS.md` returns ≥1 hit inside the entry-format block. `wc -c /home/syt/RustroverProjects/quartzite/AGENTS.md` returns < 40,000.
- **Scenarios:** happy path (section renamed, Kind added, char count under cap); edge case (char count crosses 40k — surface to user, propose extraction per Pattern 8).

### Subtask 2 — corrections-log.md edits

- **Location:** N/A.
- **Verification:** `grep -n '^# Learning Log' /home/syt/RustroverProjects/quartzite/ai-docs/corrections-log.md` returns one hit. `grep -n '#corrections-log\b' /home/syt/RustroverProjects/quartzite/ai-docs/corrections-log.md` returns zero hits. `grep -n '#learning-log\b' /home/syt/RustroverProjects/quartzite/ai-docs/corrections-log.md` returns ≥2 hits (the two updated cross-refs). `grep -n 'Kind:' /home/syt/RustroverProjects/quartzite/ai-docs/corrections-log.md` returns ≥1 hit inside the field-glossary section.
- **Scenarios:** happy path; edge (an anchor link inside the file was missed by the rename).

### Subtask 3 — agent file rename-only edits + optional fan-out (`agent-docs-index.md`, `ai-audit/SKILL.md`)

- **Verification:** for each edited file, `grep -n 'Corrections Log\|Corrections-Log' <file>` returns zero hits AFTER the edit, EXCEPT file-name references `ai-docs/corrections-log.md` (which stay).
- **Scenarios:** rename-only diff (no behavioural lines touched); the grep sweep at subtask 5 catches any miss.

### Subtask 4 — worked-example carve-out

- **Location:** `ai-docs/learnings.md`, the 2026-05-19 *compaction-recovery protocol in skill files works* entry (line 1299).
- **Verification:** the entry has a `**Kind:** validation` line AND a `**Superseded by:** PR #<N> — …` line. NO other entry in `learnings.md` has been edited (`git diff` shows exactly one entry-region modified, two new lines appended within it). `Escalated?` line remains `no` (the carve-out does NOT escalate the entry; that is Phase 3+4 territory after the verdict + threshold land).
- **Scenarios:** happy path (entry edited, no others touched); failure mode (rename caused a cascading edit elsewhere in `learnings.md`).

### Subtask 5 — verification sweep

- **Detection mechanism:**
  ```
  grep -rn -E 'Corrections.Log|corrections.log|Correction.Log' \
       /home/syt/RustroverProjects/quartzite/AGENTS.md \
       /home/syt/RustroverProjects/quartzite/ai-docs/ \
       /home/syt/RustroverProjects/quartzite/.claude/
  ```
  Every remaining hit must be one of:
  - the file-name `ai-docs/corrections-log.md` (intentionally preserved).
  - an explicit historical reference in `ai-docs/context.md` line 200 (PR #324 narrative — preserved).
  - a PR-history reference in `ai-docs/learnings.md` (historical entries — preserved per Boundary Rule 1).
- **Char-count gate:** `wc -c AGENTS.md` returns < 40,000. The estimated delta from PR 1 edits is +400 to +900 chars (rename: ~0; `Kind:` row: ~50; Propagation-Rule update: ~150; carve-out call-out: ~250–500). Starting from 36,598 chars, the post-edit count is projected at ≤ 37,500 — well under the 40,000 cap but above the 35,000 early-warning band. Acceptable per AGENTS.md § Build & Test (35k–39,999 is "proactive extraction pass; do not let next /task push over 40k") — Phase 2/PR 2's Carrot-pass additions to AGENTS.md must be sized accordingly.
- **Rust gates (AC18):** `cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. All four must pass. No Rust files are edited; gates should be green by default.

## Risks

- **Risk:** AGENTS.md crosses 40,000 chars due to the carve-out call-out length.
  **Mitigation:** measure before commit (`wc -c`). If close to cap, move the carve-out detail body into `ai-docs/corrections-log.md` § Boundary rule 1 Exception (already referenced from AGENTS.md) and keep AGENTS.md's call-out to one sentence + the cross-reference. Char-count projection (above) shows ≤ 37,500 — risk is low but real.

- **Risk:** A `Corrections Log` / `corrections-log` reference is missed in the rename, leaving silent drift between AGENTS.md and a downstream consumer.
  **Mitigation:** subtask 5's grep sweep is a hard gate. Every remaining hit must be classified (file-name / historical / fixable). The sweep runs against the full instruction surface, not just the touched files.

- **Risk:** The worked-example carve-out (subtask 4) is the FIRST authorised edit to an existing `learnings.md` entry's non-`Escalated?`-non-`Superseded by:` lines in the project's history. A future agent reading `learnings.md` may treat this as precedent for further bulk edits.
  **Mitigation:** the carve-out call-out in AGENTS.md AND in `corrections-log.md` explicitly names this entry as the named, narrow, audit-traced exception — NOT a precedent. AC13 binds. The carve-out's audit trail (entry's own `Superseded by:` line referencing the PR) is the durable record.

- **Risk:** The anchor `#learning-log` (post-rename) does not match the slug GitHub-Flavoured-Markdown will generate from `## Learning Log`.
  **Mitigation:** GFM slug for `## Learning Log` is `learning-log` (lowercase, spaces→hyphens, no other special chars). Verified by GFM slug rules; matches the manual references.

- **Risk:** The threshold-line update in AGENTS.md (line 331) is forgotten in PR 1 and Phase 4 (PR 2) tries to update an unrelated reference shape.
  **Mitigation:** the spec's PR-slicing decision is explicit — threshold-line update is Phase 4 / PR 2 territory. Design § Approach piece 1 calls this out: *"Trigger threshold line at line 331 stays unchanged in PR 1."* The PR 2 design (when it lands) will pick up the threshold edit + `improve/SKILL.md` body restate as a lock-step pair.

- **Risk:** The Propagation-Rule row update at AGENTS.md line 216 is too narrow — it names "entry format incl. `Kind:`, `Escalated?` semantics, 🌱 verdict from `/ai-audit`" but the 🌱 verdict does not yet exist in PR 1 (Phase 3 territory).
  **Mitigation:** the Propagation-Rule row update is a FORWARD-LOOKING fan-out trigger — naming the future shape so Phase 2/3/4 edits can be detected by the row's keyword. Acceptable per AC2 (*"names entry-format / `Kind:` / `🌱` semantics as triggers for fan-out"*). Alternative: include only `Kind:` in PR 1's row update and add `🌱` in PR 2 — but that doubles the AGENTS.md edit count. Recommend including both keywords in PR 1's row, with a parenthetical *"(🌱 verdict lands in Phase 3)"* for legibility.

- **Risk:** PR 1 lands but never triggers Phase 2/3/4 (PR 2) because the spec's PR slicing requires explicit PR-2 invocation.
  **Mitigation:** out of scope for this design — PR sequencing is the user's responsibility. PR 1's body should explicitly cite the follow-up PR (#491 Phase 2/3/4) so the dependency chain is visible.

- **Risk:** Subtask 3's mandatory fan-out touchpoints (`ai-docs/agent-docs-index.md`, `.claude/skills/ai-audit/SKILL.md` prose-only renames) are accidentally skipped in PR 1, leaving drift that the Phase 2/3/4 PR must also clean up.
  **Mitigation:** the file-touch map (above) marks both as **Mandatory** per the Propagation Rule's fan-out discipline (was design-review round 1 note #2; promoted from optional). The diff cost is ≤ 6 lines across the two files; Subtask 5's grep sweep catches any miss.

## Open questions

All notes from design-review round 1 (GO with notes) are resolved; ready for Step 8.

Resolution audit trail:

- **O1 — `Kind:` field position.** Resolved: placed BETWEEN `**Rule:**` and `**Escalated?**` (groups content fields before metadata fields). Recorded in § Approach piece 5.
- **O2 — Boundary-Rule-1 carve-out call-out shape.** Resolved: one-paragraph text-form callout — third indented paragraph under Boundary rule 1, consistent with existing AXIOM-blockquote + indented Exception-paragraph shape (AGENTS.md ~283–292). Recorded in § Approach piece 1.
- **O3 — Propagation-Rule row update strategy.** Resolved: update the existing row IN-PLACE. Sync-group membership unchanged (`self-improve.md` + `learnings-escalation-audit.md`); only the keyword set expands. Recorded in § Approach piece 1.
- **O4 — Phase 5 effort estimate.** Deferred to PR-2 design per spec PR-slicing decision; AC17 binds the recommendation to PR-2's design agent, not PR-1's. Not a PR-1 open question.

## Quality checklist self-verification

- **Completeness:** all 7 mandatory files listed (AGENTS.md, corrections-log.md, self-improve.md, learnings-escalation-audit.md, learnings.md, agent-docs-index.md, ai-audit/SKILL.md — the latter two promoted from optional to mandatory per design-review round 1 note #2 / Propagation Rule fan-out discipline). All 5 subtasks atomic. Fan-out is bounded and enumerated.
- **Correctness:** Boundary Rule 1 honoured (only one entry edited, via named carve-out, two new lines appended). Boundary Rule 2 honoured (the learnings.md edit is the worked-example carve-out — Boundary Rule 1 Exception authorises it, AND the carve-out is the FIRST learnings.md edit of its kind, NOT a pre-emptive escalation triggered by a fresh entry). Propagation Rule honoured (sync-group fan-out covered in subtask 3 + optional 3-extra). AC1–AC18 covered (AC3–AC11 + AC16–AC17 are Phase 2/3/4/5 territory — NOT in PR 1's scope).
- **Tests:** no Rust code touched; verification is grep + `wc -c` + Rust-gate checks per subtask 5.
- **Risks:** char-cap risk + rename-miss risk + carve-out precedent risk + forward-looking-row risk — all surfaced + mitigated.
- **Economy:** YAGNI — no Carrot-pass machinery added in PR 1; no Phase-4 threshold update; no Phase-3 verdict extension. PR 1 stays minimal-additive. Phase 2/3/4 design is the PR-2 design agent's territory.

## Phase 2–5 sketch (NOT part of PR 1 implementation)

For context only — the implementing agent should NOT touch any of these in PR 1.

- **PR 2 — Phases 2 + 3 + 4 (co-dependent).**
  - Phase 2: add Step 2 routing table to `.claude/agents/self-improve.md` Carrot pass + Step 6 inverted eval prompt + enumerate Carrot promotion verbs (*Default to* / *Prefer*) + Stick verbs (*MUST* / *NEVER*).
  - Phase 3: extend verdict set in `.claude/agents/learnings-escalation-audit.md` (add 🌱 Stale-validation, trigger: age > 30d AND `Escalated? no` AND ≥1 instruction-file commit since validation date). Add Checklist N to `.claude/skills/ai-audit/SKILL.md` § Step 2.3 (bidirectional `## Patterns` ↔ `Kind: validation`, severity `major`). Add `Kind:` row to Checklist L. Add cross-shape-verb sub-check to Checklist M (severity `major`). Document `🌱` verdict in `/ai-audit § Phase 1` orchestration prose.
  - Phase 4: AGENTS.md threshold line at line 331 rewritten — *"Run `/improve` when ≥3 unescalated correction entries, ≥2 unescalated validation entries, or a 🌱 Stale-validation flag from `/ai-audit` accumulates."* + `.claude/skills/improve/SKILL.md` body restate (sync-group lock-step).
  - **Worked example completion** (AC12): add a `## Patterns` block to the targeted skill (Inline Patterns home per Q3 — design-review at PR 2 time identifies the targeted skill; likely `.claude/skills/context-reset/SKILL.md` since the validation is about compaction recovery), back-linked to the 2026-05-19 entry. (The entry retro-tag itself lands in PR 1.)

- **PR 3 — Phase 5 (follow-up).**
  - Cross-feed with auto-memory: `self-improve` reads `~/.claude/projects/.../memory/feedback_*.md` as a companion signal. Design MUST specify (a) consent UX (explicit user approval) AND (b) project-side write guard (no auto-writes from auto-memory alone). Per spec AC16 + AC17.

## References

- Spec: `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.spec.md` (read in full during investigation).
- AGENTS.md § Corrections Log (current, lines 279–331) — the schema target.
- AGENTS.md § Propagation Rule (current, lines 200–232) — the fan-out trigger; row at line 216 is the update target.
- `ai-docs/corrections-log.md` — the extracted-reference target for the field glossary + carve-out trail.
- `.claude/agents/self-improve.md` (current, lines 16, 88, 108) — rename-only touchpoints in PR 1.
- `.claude/agents/learnings-escalation-audit.md` (current, lines 3, 17, 24, 96, 137) — rename-only touchpoints in PR 1.
- `ai-docs/learnings.md` 2026-05-19 *compaction-recovery protocol* entry (line 1299–1305) — the worked-example carve-out target.
- `ai-docs/agent-writing-style.md` § Patterns (current, lines 23–186) — the precedent for the `## Patterns` section convention adopted for promoted carrots in Phase 2 (out of PR 1 scope; cited for context).
- `ai-docs/agent-writing-style.md` § Pattern 8 (lines 146–185) — the 40k char-cap AXIOM the AGENTS.md edits must respect.
- `.claude/skills/ai-audit/SKILL.md` § Step 2.3 Checklists L + M (lines 156–186) — the integrity gates that grow in Phase 3 (out of PR 1 scope except for the prose-only rename pass; cited for context).
