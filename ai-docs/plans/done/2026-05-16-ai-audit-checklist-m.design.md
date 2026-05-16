# Design: Extend /ai-audit Phase 2 with Checklist M — agent-writing-style.md conformance

**Issue:** #369
**Spec:** `ai-docs/plans/2026-05-16-ai-audit-checklist-m.spec.md`
**Date:** 2026-05-16

## Approach

### Summary

Doc-only change in three files plus one demonstrator audit run:

1. **Add `#### M. agent-writing-style.md conformance` block** to `.claude/skills/ai-audit/SKILL.md § Step 2.3`, after the existing Checklist L block (alphabetical slot). The block contains 8 numbered sub-checks (1–7 mapping to Patterns 1–7 in the style guide; sub-check 8 covers the Anti-patterns table). Each sub-check names: the detection mechanism (a `rg` / `grep -E` / `awk` recipe shape or a text-equality probe), the severity (from the existing `blocker` / `major` / `minor` / `nit` vocabulary), and a one-line rationale.
2. **Add `## Enforcement` section** to `ai-docs/agent-writing-style.md` between `## Citation in PRs` and `## Out of scope`. Names `/ai-audit § Checklist M` as the audit venue. Links back to `.claude/skills/ai-audit/SKILL.md#m-agent-writing-stylemd-conformance` (heading slug auto-derived from the literal `#### M. agent-writing-style.md conformance`).
3. **AC6 — no AGENTS.md amendment.** The existing line-212 row already names `ai-docs/agent-writing-style.md` as a Propagation Rule source-of-edits row (forward direction: edits to the style guide fan out to downstream consumers). Checklist M is the **reverse** direction — it audits downstream consumers against the style guide. AC4's reciprocal link from the style guide (`## Enforcement` → `/ai-audit § Checklist M`) is the discoverability surface a reader actually needs; amending AGENTS.md to add a redundant pointer would consume ~150-200 of the 225-char headroom for no new information. AC6 is satisfied by recording "no amendment — existing row already names the enforcement surface via Propagation Rule fan-out and AC4's reciprocal link" in the PR body.
4. **AC5 demonstrator** — run `/ai-audit phase2` against the post-merge corpus. Either zero findings (clean baseline) or ≥ 1 finding (real drift detected). Either outcome satisfies AC5. No Step 2.6 verification additions are needed: Checklist M's findings are text-shape findings the existing Step 2.6 procedure already re-verifies via the same `rg` recipes the checklist uses to detect them.

### Why this design

- **Slot M after L (alphabetical).** Checklists A–L follow alphabetical ordering. M is the next letter; no risk of misordering or skipping.
- **One Checklist-M block, eight numbered sub-checks (1–7 + Anti-patterns).** Matches the structure of Checklist K (numbered sub-items 1–3) and Checklist L (single table). Readers can cross-reference "Pattern N" in the style guide with "Checklist M sub-check N" in the audit one-to-one.
- **`rg` / `grep -E` / `awk` only.** The existing `/ai-audit` frontmatter `allowed-tools` already lists `Bash(rg *) Bash(grep *) Bash(awk *)` — no new permission needed. (Confirmed by reading the SKILL.md frontmatter at line 6.)
- **Pattern 7 audit by text-equality against the archival doc.** The locked Variants A/B/C bodies live in `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md`. The audit reads three variant-distinguishing phrases (Variant A: `"Locate the durable-state file via this skill's active-state probe"`; Variant B: `"If exactly one in-flight artefact exists"`; Variant C: `"Identify the **parent workflow**"`) and verifies each callout-carrying skill file contains exactly one. This is `rg -c` against three fixed strings — no parser needed. Source-of-truth is the archival doc per the spec's § Key decisions; reading it as a side-input is consistent with existing `/ai-audit` Phase-2 read policy (Step 2.2 reads `done/` only on demand, but this is a single named file referenced by exact path).
- **No Step 2.6 verification additions.** Step 2.6 already names the recipe "Re-run any `find`/`grep` checks from Step 2.3 that detected violations — confirm zero remaining." Checklist M's detection IS the re-verification recipe — each finding is "this `rg` returned N hits"; the fix is "edit until the same `rg` returns 0 (or N - delta)". Identity between detect + re-verify is a feature, not a gap.
- **No commit-staging additions.** Step 3 stages files "actually edited"; an audit run that touches no files commits no files. The block-level `cat <<'EOF'` in Step 3 is already wildcard-shaped ("`<any other files actually edited>`").

### Resolved open questions

#### Q1 — AC6 amendment scope (does AGENTS.md `## Propagation Rule` need a new row?)

**Decision: No amendment.** Recorded explicitly in AC6 as "no amendment — existing row already names the enforcement surface via Propagation Rule fan-out and AC4's reciprocal link".

Reasoning:

- **Existing row at AGENTS.md line 212** already names `ai-docs/agent-writing-style.md` as a Propagation Rule edit source: "After adding or amending a pattern, run `grep -rn "<pattern-keyword>" .claude/agents/ .claude/skills/` to find any file already half-using the pattern and reconcile." That row's job is the **forward** direction — when the style guide changes, sync the consumers.
- **Checklist M is the reverse direction.** It audits downstream consumers against an unchanged style guide. The two directions are complementary, not duplicative. Amending row 212 to mention `/ai-audit § Checklist M` would conflate them.
- **Discoverability is preserved by AC4.** The style guide's new `## Enforcement` section names the audit venue. A reader who lands on `agent-writing-style.md` sees the audit; a reader who lands on the AGENTS.md Propagation Rule row sees the fan-out grep. Both surfaces are reachable from their natural entry points.
- **Char-budget consequence.** AGENTS.md is at 39,775 / 40,000 hard cap (225-char headroom). A "see also `/ai-audit § Checklist M`" suffix on row 212 is ~120-180 chars depending on phrasing — possibly fits, but takes meaningful headroom for a redundant pointer. The Path A2 extraction follow-up is already pending from PR #363; adding a row now would consume headroom that future essential rules need more.

This resolves AC6 to a single record-keeping action in the PR body, not a file edit. AGENTS.md is untouched.

#### Q2 — Pattern 6 "non-trivial rule" detector

**Decision: Mechanical heuristic with `nit`-severity fallback.** The audit applies a two-part heuristic that the agent can mechanically check, then assigns severity `nit` to the result (Pattern 6 violations are an authoring style issue, not a correctness issue).

**Heuristic (sub-check 6 detection recipe):**

A paragraph-shaped fail-loud block (the body of a `> **AXIOM**` blockquote, or a `> ` block containing one fail-loud verb) is a "non-trivial rule" if **both** of these hold:

1. The body contains at least one of the words `instead`, `not`, `NOT`, `wrong`, `correct`, `right`, `bad`, `good`, `forbidden`, or any Pattern 2 fail-loud verb — i.e., the body itself already references a "negative shape" it is differentiating from.
2. The body does NOT contain a fenced code block or a two-column `| Do this | NOT this |`-shaped table within 8 lines after the rule statement — i.e., no concrete example accompanies the rule.

When both 1 AND 2 hold, the audit reports: "Pattern 6 shape violation in `<file>:<line>`: rule references a contrasting shape but no do/not example follows. Severity: `nit`."

**Why nit-severity:**

- Pattern 6 is the only Pattern where the style guide itself uses the qualifier "for non-trivial rules" — admitting the judgment call.
- The heuristic will produce noise: many rules legitimately differentiate without needing a code example (the words `not` / `instead` appear in ordinary English connective tissue).
- Downgrading to `nit` (per Step 2.5: "may apply autonomously if the fix is mechanical and obvious; otherwise ask") means the audit surfaces the candidate but never auto-edits. The user decides per-finding.
- A future refinement can raise severity if the heuristic proves accurate; lowering severity after the fact is a stronger signal change.

**Fallback if the heuristic produces too much noise on first run:** record the noise rate in the AC5 demonstrator run; if > 50% of Pattern 6 findings are false positives, the next `/improve` cycle can either tighten the heuristic (require a stronger trigger word like `instead` only) OR defer Pattern 6 to manual review with no auto-detection.

#### Rejected alternatives for Q2

| Alternative | Rejected because |
|---|---|
| Length-based threshold ("paragraph > 200 chars triggers the check") | Length and "non-triviality" don't correlate — short rules can be subtle, long rules can be flat enumerations. Produces false positives on every long bullet list. |
| Verb-count threshold ("> 1 fail-loud verb triggers the check") | Already covered by Pattern 2 (which forbids > 1 verb per paragraph). Pattern 6 would catch the same paragraphs Pattern 2 already flags, double-counting findings. |
| Defer entirely to manual review (no auto-detection) | Spec § Open questions explicitly invites a heuristic and only authorises "downgrade severity to `nit`" as a noise-management fallback. Total deferral is a stronger move than asked for. |
| Substring-list grep for explicit `do this:` / `NOT this:` markers | Reverse direction — finds rules that already have examples, not rules missing examples. Wrong polarity. |

### Rejected alternatives (whole-design level)

| Alternative | Rejected because |
|---|---|
| Build a separate `/style-audit` skill | Spec § Out of scope explicitly forbids — `/ai-audit` already loads exactly this file set. |
| Hook-level pre-commit enforcement of Pattern shapes | Spec § Out of scope — no recurrence threshold met; zero historical violations recorded. |
| Auto-fix Pattern violations | Spec § Out of scope — existing Step 2.5 severity-driven apply-or-ask covers application policy unchanged. |
| Add Step 2.6 verification block specific to Checklist M | Identity between Checklist M's detect recipes and re-verify recipes makes a duplicated block redundant. Existing Step 2.6 "Re-run any `find`/`grep` checks from Step 2.3" already covers it. |
| Embed Pattern 7 locked bodies verbatim into Checklist M | Source-of-truth lives in the archival design doc. Embedding copies a second source-of-truth that could drift. Cite by path + grep three variant-distinguishing phrases (already short, already locked). |
| Amend AGENTS.md row 212 with a `/ai-audit § Checklist M` suffix | See Q1 — redundant pointer, consumes scarce headroom. |
| Hand-edit Step 3 commit-staging list to add Checklist M files | Step 3 already uses a wildcard placeholder for "any other files actually edited". No special-case needed. |

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `#### M. agent-writing-style.md conformance` block to `.claude/skills/ai-audit/SKILL.md` Step 2.3 (after Checklist L, before Step 2.4). **Preamble inlines the audited file set explicitly** (per design-review note 1 — Step 2.2's existing inventory does NOT enumerate `ai-docs/code-style.md` / `ai-docs/agent-writing-style.md` / `ai-docs/corrections-log.md`, so Checklist M names its own corpus inline without depending on a Step 2.2 expansion): `AGENTS.md` + every `.claude/skills/**/SKILL.md` + every `.claude/agents/**.md` + `ai-docs/code-style.md` + `ai-docs/doc-convention.md` + `ai-docs/agent-writing-style.md` + `ai-docs/corrections-log.md`. Contains 8 numbered sub-checks (1–7 mapping Patterns 1–7; 8 covers Anti-patterns table). Each sub-check states: detection mechanism (`rg`/`grep -E`/`awk` recipe shape OR text-equality probe), severity (`blocker`/`major`/`minor`/`nit` per spec § Key decisions defaults — Pattern 1 = `major`, Pattern 2 = `minor`, Pattern 3 = `minor`, Pattern 4 = `major`, Pattern 5 = `nit`, Pattern 6 = `nit` per Q2 above, Pattern 7 = `major`, Anti-patterns = `major`), one-line rationale. Pattern 7 sub-check **drives off a live grep** (per design-review note 2 — phrase the probe as *"for each `.claude/skills/*/SKILL.md` whose body contains `Compaction recovery check`, verify exactly one of {3 variant phrases} appears"* rather than against the style guide's static Pattern 7 enumeration; this auto-handles the 2 callout-carrying skills `/pr-ci-failed` and `/master-ci-failed` not yet listed in the style guide table), inlines the three variant-distinguishing phrases as `rg -F` patterns directly in SKILL.md (NOT as references to the archival doc — design § Why this design row 4 fallback), and cites the archival doc at `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md` by path for rationale only. Pattern 6 sub-check uses the **tightened heuristic** (per design-review note 4 — require BOTH a Pattern 2 fail-loud verb AND one of the stronger contrast markers `instead` / `wrong` / `correct` / `forbidden`; DROP `not` / `NOT` / `right` / `bad` / `good` from the trigger list — `not` alone fires on every "do not" / "must not" / "is not" paragraph and would bury AC5's signal). AC1, AC2, AC3. | `.claude/skills/ai-audit/SKILL.md` | — |
| 2 | Add `## Enforcement` section to `ai-docs/agent-writing-style.md` between `## Citation in PRs` and `## Out of scope`. Body: one short paragraph naming `/ai-audit § Checklist M` as the audit venue, linking to `.claude/skills/ai-audit/SKILL.md#m-agent-writing-stylemd-conformance` (auto-derived slug from the `#### M. agent-writing-style.md conformance` heading). **Verify the slug at commit-time** (per design-review note 3): before `git commit`, run the Step 2.6 anchor-aware recipe (or a minimal `awk` slug derivation) against the two changed files and confirm the link resolves; do NOT defer to the next `/ai-audit` run. AC4. | `ai-docs/agent-writing-style.md` | 1 (heading slug must exist before the cross-link can resolve) |
| 3 | AC5 demonstrator — run `/ai-audit phase2` against the post-merge corpus. **Surface the raw `rg` outputs** (per design-review recommendation — not just summary counts; a future reviewer must be able to re-derive findings independently). Record outcome verbatim in the PR body: either "AC5: no drift in current corpus — baseline clean" OR a list of ≥ 1 findings with file:line + severity + Pattern N + the raw `rg` output that produced the finding. No file edits unless the run surfaces findings the user approves to apply. AC5. | (audit run; no edits planned) | 1, 2 (audit needs Checklist M present to exercise it) |
| 4 | AC6 record-keeping. Add to PR body: "AC6: no amendment — existing row already names the enforcement surface via Propagation Rule fan-out and AC4's reciprocal link. AGENTS.md char count unchanged at 39,775; headroom preserved for the pending Path A2 extraction." No AGENTS.md edit. AC6. | (PR body only; no file edits) | — |

4 subtasks. Well under the 7-task split threshold; no shard candidates.

### Suggested commit boundary mapping

| Subtask | Suggested commit |
|---|---|
| 1 + 2 (bundle) | `docs(ai-audit): add Checklist M — agent-writing-style.md conformance + reciprocal Enforcement section` |
| 3 | (run, not commit — outcome recorded in PR body) |
| 4 | (PR-body record only — no commit) |

Subtasks 1 and 2 must ship in one commit so the cross-link in the style guide resolves to a heading that exists in the same diff. AGENTS.md *Propagation Rule* groups Checklist M's edit (subtask 1) and the reciprocal style-guide edit (subtask 2) under the implicit dependency the *Procedure* names ("apply the same change in every match") — single commit is the cleanest expression of that bundling.

## Risks

| Risk | Mitigation |
|---|---|
| **Heading slug for `#### M. agent-writing-style.md conformance` doesn't auto-derive cleanly.** GitHub's slug rules lowercase + replace spaces with hyphens + strip punctuation; the literal heading produces `m-agent-writing-stylemd-conformance` (the `.md` becomes `md` without a hyphen). | Spec already names the expected slug verbatim in AC4: `.claude/skills/ai-audit/SKILL.md#m-agent-writing-stylemd-conformance`. Subtask 2 uses that slug. Verification step: read both files after subtask 2 commit and confirm the link matches the heading slug rendering (the audit recipe in Step 2.6 sub-check 4 already does anchor-aware verification — it would catch the mismatch on the next `/ai-audit` run). |
| **Pattern 6 heuristic produces too many false positives on first run.** The heuristic catches any paragraph mentioning `instead`/`not`/`NOT`/etc. without a code block — many ordinary rules legitimately do this. | Severity is `nit` by design (Q2 above). User reviews findings; the noise stays in the report but never auto-applies. If > 50% false-positive rate emerges on the AC5 demonstrator, record the rate and tighten the heuristic in a follow-up. The fallback path is documented in the sub-check's rationale. |
| **Pattern 7 audit reads the archival doc; future archival doc edits could move the locked bodies.** | The archival doc at `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md` is referenced by exact path. AGENTS.md *Workflow* / `/triage` / `/improve` policy treats `done/` content as immutable history. The three variant-distinguishing phrases are short enough (a single sentence each) that even if a future PR rewrote the design doc, copying the locked phrases into Checklist M as inline strings is a one-line fix. Optional belt-and-braces: subtask 1's text-equality probe COULD inline the three phrases as `rg -F` patterns inside SKILL.md itself, removing the runtime dependency on the archival doc. Decision: cite the doc by path (lighter touch); inline the three short phrases as `rg -F` patterns. The phrases are themselves the source-of-truth surface that Checklist M cares about, not the design rationale around them. |
| **AC5 demonstrator surfaces a blocker that delays merge.** If `/ai-audit phase2` finds a real Pattern violation in the current corpus, the user must decide whether to fix it in this PR (in scope per AC3's mutation scope) or defer it. | Per spec AC5: "If ≥ 1 finding surfaces, the finding is recorded in the merge PR's body or a follow-up issue." User decides. The demonstrator is a discovery step, not a gate that blocks merge on findings — only on the audit running successfully. |
| **AGENTS.md headroom pressure unrelated to this PR.** Current 39,775 / 40,000 (225-char headroom) plus the pending Path A2 extraction follow-up from PR #363. This PR doesn't touch AGENTS.md, so it neither adds nor reduces the pressure — but Q1's resolution is implicitly load-bearing on that fact. | Recorded in subtask 4's PR-body line. If a future user wants AC6 amended retroactively, the Path A2 extraction must land first to free headroom. Not blocking this PR. |
| **`.claude/skills/ai-audit/SKILL.md` adds ~1.5-2.5k chars (per spec § Technical constraints), pushing it to ~18-19k chars.** | Well under the 35,000 / 40,000 caps and under Checklist-I's 500-line soft-limit. Not a risk; recorded for the PR body's char-count audit row. |
| **`ai-docs/agent-writing-style.md` adds ~300-500 chars, pushing it to ~9.6-9.7k chars.** | Same — well under any cap. |
| **Cross-link in `## Enforcement` could be a relative path requiring the `realpath` recipe at Step 2.6 sub-check 4.** | Subtask 2 uses the relative path `.claude/skills/ai-audit/SKILL.md#m-agent-writing-stylemd-conformance` (from `ai-docs/agent-writing-style.md` perspective: `../.claude/skills/ai-audit/SKILL.md#...`). The Step 2.6 anchor-aware re-verification (visible in the audit's own SKILL.md at lines 196-212) WILL exercise this cross-link on the next `/ai-audit` run — built-in safety net. |

## Test Design

Documentation-only PR — no Rust code, no `#[cfg(test)]` modules. "Tests" map to the audit's own grep / text-equality recipes plus the AC5 demonstrator. None gated by CI (per AC7).

### Per-AC verification plan

| AC | Verification |
|---|---|
| AC1 | `rg "^#### M\. agent-writing-style\.md conformance$" .claude/skills/ai-audit/SKILL.md` returns exactly 1. The block contains 8 numbered sub-checks. Each sub-check names a detection mechanism + severity. Spot-check: each Pattern N sub-check (1-7) cites the corresponding Pattern N in the style guide; sub-check 8 cites the Anti-patterns table. |
| AC2 | `rg "blocker\|major\|minor\|nit" .claude/skills/ai-audit/SKILL.md` returns hits inside the Checklist M block; no other severity label appears (e.g., no "critical", no "warning"). |
| AC3 | The mutation-scope sentence inside the Checklist M block names the same file set as Step 2.2 (verifiable by `diff <(grep -oE "AGENTS.md\|CLAUDE.md\|.claude/.+|ai-docs/.+\.md" <Checklist-M-block>) <(grep -oE … <Step-2.2-block>)`). |
| AC4 | `rg "^## Enforcement$" ai-docs/agent-writing-style.md` returns exactly 1. Section sits between `## Citation in PRs` and `## Out of scope` (verified by line-number ordering of the three section headings). Body contains the relative link `.claude/skills/ai-audit/SKILL.md#m-agent-writing-stylemd-conformance` (or the heading slug subtask 1 actually produced — must match). |
| AC5 | `/ai-audit phase2` invocation runs to completion (Phase 2 reports findings or "no drift"). The result is recorded verbatim in the PR body. If ≥ 1 finding surfaces, user decides fix-now vs defer-to-issue per spec § Out of scope mutation-scope rules. |
| AC6 | PR body contains the literal record "AC6: no amendment — existing row already names the enforcement surface via Propagation Rule fan-out and AC4's reciprocal link". `git diff AGENTS.md` returns empty (no edit). |
| AC7 | No `.github/workflows/*.yml` touched (`actionlint` n/a). No `.rs` touched (`cargo build`/`cargo test`/`cargo clippy` n/a). `git diff --name-only` against `master` returns ≤ 2 files: `.claude/skills/ai-audit/SKILL.md` and `ai-docs/agent-writing-style.md`. |

### Verification scripts (one-off, run during subtask 3 / commit-time)

```bash
# AC1 — Checklist M heading + 8 sub-checks
rg -c "^#### M\. agent-writing-style\.md conformance$" .claude/skills/ai-audit/SKILL.md  # expect 1
# Spot-count numbered sub-items within the Checklist M block (1. .. 8.):
awk '/^#### M\./,/^#### [A-Z]\./ { if (/^[0-9]+\./) print }' .claude/skills/ai-audit/SKILL.md | wc -l  # expect 8

# AC2 — severity labels from the existing vocabulary only
awk '/^#### M\./,/^#### [A-Z]\./' .claude/skills/ai-audit/SKILL.md | rg -oE '`(blocker|major|minor|nit)`' | sort -u  # expect at most 4 labels, no others

# AC4 — Enforcement section presence + ordering + link
rg -c "^## Enforcement$" ai-docs/agent-writing-style.md  # expect 1
grep -nE "^## (Citation in PRs|Enforcement|Out of scope)$" ai-docs/agent-writing-style.md  # expect Citation < Enforcement < Out of scope
rg -c "m-agent-writing-stylemd-conformance" ai-docs/agent-writing-style.md  # expect ≥ 1

# AC5 — demonstrator (manual run during subtask 3)
# /ai-audit phase2

# AC6 — AGENTS.md unchanged
git diff --quiet AGENTS.md && echo "AC6 satisfied" || echo "AC6 violated"
wc -c AGENTS.md  # expect 39,775 (unchanged)

# AC7 — diff scope
git diff --name-only master..HEAD | sort
# expect:
#   .claude/skills/ai-audit/SKILL.md
#   ai-docs/agent-writing-style.md
#   ai-docs/plans/2026-05-16-ai-audit-checklist-m.spec.md  (already merged in spec PR; may or may not appear)
#   ai-docs/plans/2026-05-16-ai-audit-checklist-m.design.md  (this design doc — yes, expected in the implementation PR)
```

## Open questions

None. All five open questions in the spec are resolved by this design:

- **AC6 amendment scope** → No amendment. See § Approach / Q1.
- **Pattern 2 verb set lock-in** → Take the current set as-is (spec already directs).
- **Pattern 6 "non-trivial rule" detector** → Mechanical heuristic with `nit` severity fallback. See § Approach / Q2.
- (The remaining two open questions cited in the spec body live in the prose, not in the § Open questions list — they are about archival-doc dependency and Anti-patterns scope, both addressed in the risk table.)
