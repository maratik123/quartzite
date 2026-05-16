# Design: Codify 40k char-cap AXIOM in agent-writing-style.md and /ai-audit Checklist M

**Issue:** #416
**Date:** 2026-05-16

## Goals

- Land the 40,000-char file-size cap rule (from `AGENTS.md § Build & Test`) as **Pattern 8** under `## Patterns` in `ai-docs/agent-writing-style.md`, fully self-conformant to existing patterns 1–7.
- Extend `.claude/skills/ai-audit/SKILL.md` Checklist M with two new sub-checks:
  - **Sub-check 9** — file-size AXIOM conformance (`wc -c` against the enumerated covered file set; severity ladder `minor` / `blocker`).
  - **Sub-check 10** — style-guide audit coverage map (dynamic ATX `## ` heading parse against an inline coverage map; emits `nit` on unmatched headings).
- Demonstrate end-to-end behaviour: `/ai-audit` against the post-merge corpus surfaces AGENTS.md = 39,960 chars as a `minor` Sub-check 9 finding (AC5).

## Non-goals

- No edit to `AGENTS.md § Build & Test`. The AXIOM stays where it is; Pattern 8 cites it as the source-of-truth via prose reference.
- No new propagation-rule row in AGENTS.md. The existing `agent-writing-style.md` propagation row (AGENTS.md § *Propagation Rule*) covers fan-out.
- No new pre-resolved-rule entry in `.claude/agents/spec-writer.md` (the char-cap is a structural property of files, not a question-time pre-resolved rule).
- No implementation of `scripts/check-instruction-file-sizes.sh` — tracked separately in #383. This task's audit-side check is the interim back-stop.
- No generalisation of Sub-check 10 to other style references (`code-style.md`, `doc-convention.md`, etc.).
- No Rust code changes; AC7 verifies workspace gates pass (`cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, doc gate) but the diff is markdown-only.

## Approach

The change is markdown-only and decomposes cleanly into two file edits plus verification. The pattern entry and the audit sub-checks are coupled (Sub-check 10's coverage map references the new Pattern 8 entry implicitly) but their bodies are independent — edits can be authored in either order, but landing Pattern 8 first keeps Checklist M's `nit` coverage-gap finding from misfiring against the new pattern during interim states.

### Pattern 8 wording shape (chosen)

Adopt the same shape as Patterns 1 and 2 — AXIOM blockquote (per Pattern 1) followed by an action table (per Pattern 1 / Pattern 3), then a bullet list enumerating the covered file set (per Pattern 4, explicit paths). The body cross-references `AGENTS.md § Build & Test` as the source-of-truth and PR #324 as the canonical extraction example.

**Self-conformance audit (the entry must not trip its own peers):**

| Pattern | How Pattern 8 satisfies it |
|---|---|
| 1 (AXIOM blockquote + action table) | Entry opens with `> **AXIOM — ...**` blockquote followed by an action table inside the same blockquote. |
| 2 (one fail-loud verb per paragraph) | Single `**STOP**` verb in the action table's right column for the `≥ 40,000` row. No second verb in any paragraph of the entry. |
| 3 (action verbs in right column) | Right column starts with `**STOP**` / "Proactive extraction pass" / "OK." — imperative forms, no narrative prose. |
| 4 (explicit file enumeration) | Each covered path appears as its own bullet; no `**` glob as the entire list. Per-bullet parenthetical `(any file under this directory)` is permitted by Pattern 4 itself. |
| 5 (numbered triggers with consistent connectors) | Pattern 8 does not use a numbered trigger list; N/A. |
| 6 (do/not examples) | Pattern 8 does not pair a "do X / not Y" contrast; N/A. |
| 7 (compaction recovery callout) | Pattern 8 is not a callout-carrying skill; N/A. |
| Anti-patterns | The entry does not duplicate any Anti-patterns row left-column text verbatim. |

### Sub-check 9 wording shape

Add row 9 to Checklist M's existing severity table. Detection mechanism: `wc -c` invoked against an enumerated covered file set (inline in Checklist M itself — not deferred to `scripts/check-instruction-file-sizes.sh` which doesn't exist yet). Three-band severity mirrors the AXIOM:

- `< 35,000` chars → no finding
- `35,000–39,999` chars → `minor` (early warning)
- `≥ 40,000` chars → `blocker` (AXIOM violation)

The detection command is a single inline `wc -c <enumerated paths>` invocation. The fail-loud bullet list in Pattern 8 uses explicit per-bullet paths (Pattern 4's requirement); the `wc -c` invocation itself, being a shell command, is permitted to use shell-glob expansion. Implementer copies the exact command into Sub-check 9's body verbatim:

```bash
wc -c AGENTS.md CLAUDE.md .claude/skills/*/SKILL.md .claude/agents/*.md \
      ai-docs/code-style.md ai-docs/doc-convention.md ai-docs/context.md \
      ai-docs/agent-writing-style.md ai-docs/corrections-log.md
```

The shell-glob form is acceptable inside the shell command because Pattern 4's explicit-path requirement applies to the fail-loud bullet list (where each path appears as its own bullet so a static reader can see the covered set), not to the shell command that consumes the set.

### Sub-check 10 wording shape

Sub-check 10 adds a coverage-map table inside its body listing each currently-known level-2 heading in `ai-docs/agent-writing-style.md` and either (a) the Checklist M sub-check that covers it, or (b) a "not-rule-bearing" exclusion reason. The matching algorithm at audit time:

1. Parse `## ` (ATX level-2, exactly two `#` followed by a space) headings from `ai-docs/agent-writing-style.md`. Skip lines inside fenced code blocks (track ```` ``` ```` and `~~~` fence state). Trim leading/trailing whitespace from the heading text. Case-sensitive match.
2. For each parsed heading, look it up in the inline coverage map. Match exact heading text (case-sensitive, post-trim).
3. Three outcomes:
   - **Mapped to a sub-check** (e.g., `## Patterns` → sub-checks 1–7 (which audit the shape of every entry under that heading, including the new Pattern 8 — Pattern 8's self-conformance is checked by sub-checks 1–4 since Pattern 8 follows Patterns 1, 2, 3, 4's shape; sub-check 9 itself is the file-size AXIOM check, not a topical anchor for Pattern 8); `## Anti-patterns` → sub-check 8) → no finding.
   - **Mapped to an exclusion** (e.g., `## Writing checklist`, `## Citation in PRs`, `## Enforcement`, `## Propagation rule for new patterns`, `## Out of scope`) → no finding.
   - **Unmapped** → emit `nit` finding: `audit coverage gap: § <heading>` with proposed action `add sub-check N+1 to /ai-audit Checklist M` (where N is the current max sub-check number).

The coverage map is inline in Checklist M (not in a separate config file) per the audit's self-containment design choice. When a new `## ` heading is added to `agent-writing-style.md`, Sub-check 10 fires at the next `/ai-audit` run, surfacing the gap; the operator either adds a corresponding sub-check or extends the exclusion list in the same /ai-audit follow-up.

### Q1 (deferred-script cross-reference) — adopt the spec's default

Pattern 8 carries one line cross-referencing `scripts/check-instruction-file-sizes.sh` and issue #383 as the planned mechanical gate: *"Mechanical pre-commit gate planned in #383; this audit-side back-stop fires in the meantime."* Cheap, signposts the symbiosis, doesn't lock in #383's scope. The user may revisit at design-review time.

### Rejected alternatives

- **Embed the AXIOM verbatim in `agent-writing-style.md` Pattern 8.** Rejected — duplicates the AGENTS.md body and creates a drift surface. Pattern 8 references AGENTS.md as source-of-truth with summary + threshold table + covered file set.
- **Make Sub-check 10 a static cross-reference table only (no dynamic parse).** Rejected — defeats the spec's stated benefit ("future style-guide additions auto-covered"). Static tables drift; dynamic parse against the live file does not.
- **Match Sub-check 10 by heading-text substring (e.g., "Pattern" → covered).** Rejected — fuzzy matching produces false positives. Exact match against the inline coverage map is unambiguous and lets the operator distinguish topical-coverage from explicit-exclusion.
- **Edit AGENTS.md to add a back-reference to Pattern 8.** Rejected — AGENTS.md is at 39,960 chars (40 below the hard cap); any AGENTS.md edit must pair with an extraction pass that drops it below 35,000 chars at every commit boundary. Out of scope for this task.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add Pattern 8 to `## Patterns` in `ai-docs/agent-writing-style.md`: AXIOM blockquote + action table + explicit covered-file bullet list + extraction-model citation of PR #324 + per-commit invariant note + one-line cross-reference to #383. Run the Propagation Rule grep (`grep -rn "40,000\|char-cap\|file size" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md`) and reconcile any half-uses or contradictions found. | `ai-docs/agent-writing-style.md` | — |
| 2 | Extend Checklist M in `.claude/skills/ai-audit/SKILL.md` with Sub-check 9 (file-size AXIOM conformance — three-band severity, inline `wc -c` recipe per the verbatim block in Approach → "Sub-check 9 wording shape", inline enumerated covered file set) and Sub-check 10 (style-guide audit coverage map — ATX `## ` heading parser recipe with fenced-code-block skip, inline coverage map enumerating current level-2 headings, exclusion list of non-rule-bearing meta-sections, `nit` finding shape for unmapped headings). **Prerequisite:** Implementer re-runs `grep -n '^## ' ai-docs/agent-writing-style.md` immediately before authoring the coverage map and uses the **live** heading set at implementation time — not the snapshot embedded in this design (which is correct for 2026-05-16's HEAD but could drift if another PR adds a `## ` heading between this design merging and subtask 2's commit). | `.claude/skills/ai-audit/SKILL.md` | 1 |
| 3 | Manually run `/ai-audit` Phase 2 Checklist M against the post-edit corpus (AC5 demonstrator). Verify Sub-check 9 emits a `minor` finding for AGENTS.md = 39,960 chars (path + count visible in the finding text). Verify Sub-check 10 emits no `nit` findings for the current `agent-writing-style.md` `## ` headings (all mapped or excluded). Record the demonstrator output inline in the PR description for the reviewer. | (no file edit — demonstrator run) | 1, 2 |
| 4 | Verify per-commit invariant (AC6): at each commit boundary on the feature branch, every covered file is `< 40,000` chars. Run `wc -c` on the covered file set at HEAD of each commit (not just merge-time HEAD). Verify workspace gates pass (AC7): `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` — expected no-op since the diff is markdown-only, but the gate run is procedurally required. | (no file edit — verification run) | 1, 2, 3 |

## Handoff plan

Per the every-group handoff contract (`.claude/skills/task/SKILL.md` Step 8 + `.claude/skills/task/reference.md` § *Every-group handoff (rationale)*), every group (including the first) enters via a `/context-reset` subagent. Maximum group size 3 (non-terminal must be exactly 3); terminal-group sizing 1..=3. M = 4 → two groups, 3 + 1.

- **Handoff into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task enters Group A with fresh context.
- **Group A:** subtasks 1–3 — initial implementation + demonstrator chunk (Pattern 8 edit, Checklist M edit, AC5 demonstrator run; 3 subtasks; equals the non-terminal maximum).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group B with fresh context.
- **Group B:** subtask 4 — terminal group (1 subtask; within the 1..=3 range). Per-commit invariant verification + workspace gate run.

Rationale for splitting between subtask 3 and subtask 4: M = 4 exceeds the non-terminal maximum group size of 3, so a split is mandatory. The natural seam is between subtask 3 and subtask 4 because subtask 3 is the AC5 demonstrator (running `/ai-audit` Phase 2 Checklist M, which itself loads the audited corpus); resuming with fresh context for the verification pass in Group B keeps the workspace-gate verification independent of any audit-run context drift.

## Risks

- **Risk: Pattern 8 entry trips Sub-check 1 (Pattern 1 audit) by missing an action table.** *Mitigation:* The action table is enumerated in the subtask 1 spec (three rows: `≥ 40,000` → STOP, `35,000–39,999` → proactive extraction, `< 35,000` → OK). Subtask 1 implementer reads Pattern 1 first and mirrors the shape.
- **Risk: Sub-check 10's heading parser misclassifies a fenced-code-block heading as a real `## ` heading.** *Mitigation:* The parser recipe in Sub-check 10 tracks fence state (\`\`\` and `~~~`); the recipe is included verbatim in Checklist M so the audit operator can validate it locally. Future-proofing: if the recipe proves brittle, a `/improve` follow-up tightens it; no schema change to Sub-check 10's finding format.
- **Risk: Sub-check 10 fires false-positive `nit`s on non-rule-bearing meta-sections.** *Mitigation:* The exclusion list (`## Writing checklist`, `## Citation in PRs`, `## Enforcement`, `## Propagation rule for new patterns`, `## Out of scope`) is enumerated inline in Sub-check 10's body. Subtask 2 implementer copies the current `## ` heading set verbatim into the coverage map + exclusion list.
- **Risk: Sub-check 9's `wc -c` enumeration drifts from the AGENTS.md AXIOM's covered file set.** *Mitigation:* Sub-check 9 enumerates the paths verbatim from the AGENTS.md AXIOM (subtask 2 implementer copies the bullet list from `AGENTS.md § Build & Test`). A future AGENTS.md edit that changes the covered file set MUST update Sub-check 9 in the same PR — captured as a propagation row in subtask 2's commit message.
- **Risk: AGENTS.md hits 40,000 chars before #383 lands.** *Mitigation:* Out of scope for this task. AGENTS.md is at 39,960 chars (40 below the hard cap); the early-warning band (Sub-check 9 = `minor`) is the demonstrator. Real mitigation is an extraction-driven AGENTS.md edit, which would be a follow-up issue triggered by Sub-check 9's first run.
- **Risk: AC5 demonstrator surfaces UNEXPECTED additional `minor` or `blocker` findings for files other than AGENTS.md.** *Mitigation:* If any other file is ≥ 35,000 chars at audit time (currently the next-largest is `ai-docs/context.md` at 33,852 — comfortably under), the demonstrator surfaces them honestly. This is the desired behaviour; the task does not gate on AGENTS.md being the only finding. Document any additional findings in the PR body alongside the AGENTS.md finding.

## Test Design

This task is markdown-only — no Rust code changes, no unit/integration test additions. The "test plan" is the verification battery in subtasks 3 and 4.

### Subtask 3 — AC5 demonstrator

- **Entry point:** Run `/ai-audit` with scope `phase2` against the post-subtask-2 corpus.
- **Scenarios:**
  - Happy path: Sub-check 9 emits exactly one `minor` finding for AGENTS.md = 39,960 chars; finding text contains `AGENTS.md` and the character count.
  - Sub-check 10 emits zero `nit` findings (all current `## ` headings in `agent-writing-style.md` are either mapped to a sub-check or excluded).
  - Edge case (informational): Sub-check 9 emits zero findings for any other instruction file (next-largest `ai-docs/context.md` at 33,852 chars is below the early-warning band).
- **Fixtures:** post-subtask-2 working tree (no separate fixtures needed; the corpus IS the fixture).

### Subtask 4 — AC6 + AC7 verification

- **Entry point:** Per-commit `wc -c` invocation on the covered file set; workspace gate commands.
- **Scenarios:**
  - At each commit boundary on the feature branch: every covered file is `< 40,000` chars. Iterate via `git rev-list <merge-base>..HEAD` then `git show <SHA>:<path> | wc -c` for each covered path.
  - `cargo fmt -- --check` exits 0 (no formatting changes).
  - `cargo clippy --workspace -- -D warnings` exits 0 (no new warnings).
  - `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` exits 0 (no doc-gate regressions).
- **Fixtures:** the feature branch HEAD and its commit history.

### Sub-check 9 / Sub-check 10 audit-side parser correctness

Sub-check 10's heading parser correctness is validated by subtask 3's demonstrator run (zero false positives expected against the current `agent-writing-style.md` corpus). No separate test fixture is added — the audit's own self-check IS the test. If a regression class emerges later (e.g., a future style-guide edit trips the parser), a `/improve` cycle tightens the recipe.

## Open questions

None outstanding from the spec. The spec's Q1 (deferred-script cross-reference) is resolved in this design by adopting the spec's default — Pattern 8 carries a one-line forward reference to issue #383. Design-review may flip this to "stay silent on #383" at no cost (the cross-reference is one line in Pattern 8's body).

## Files touched

| Path | Change |
|---|---|
| `ai-docs/agent-writing-style.md` | Add Pattern 8 entry under `## Patterns` (after Pattern 7). ~30–60 lines. Post-edit size ≤ ~12,000 chars (well under the 35k early warning band). |
| `.claude/skills/ai-audit/SKILL.md` | Extend Checklist M table with rows for Sub-check 9 + Sub-check 10. Add Sub-check 10's inline coverage map + exclusion list immediately after the Checklist M table. ~40–80 lines. Post-edit size ≤ ~24,000 chars (well under the 35k early warning band). |
| `ai-docs/plans/2026-05-16-ai-audit-charcap-axiom.spec.md` | (no change — spec is finalised) |
| `ai-docs/plans/2026-05-16-ai-audit-charcap-axiom.design.md` | (this file) |
| `ai-docs/plans/2026-05-16-ai-audit-charcap-axiom.progress.md` | Created/updated by `/task` Step 8+; local-only (gitignored). |
| `ai-docs/learnings.md` | Conditional — only if a new class of mistake emerges during implementation (per Boundary rule 2 Exception for in-flow `/task` Steps 8–12). No pre-emptive escalation. |
| `ai-docs/plans/INDEX.md` | Status row updated by `/task` Step 12 (in-progress → done after merge). |
