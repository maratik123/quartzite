# Design: CI-failed skills — `/pr-ci-failed` + `/master-ci-failed`

**Issue:** #357
**Spec:** `ai-docs/plans/2026-05-15-ci-failed-skills.spec.md` (round-2 amended)
**Date:** 2026-05-15

## Approach

Two new instruction-only skills under `.claude/skills/`, deliberately authored as a paired set: each is a one-round CI-failure response skill, structured as a near-clone of `/pr-commented` SKILL.md (the closest-shape template at 22,339 chars). The fork happens at the *fix-landing* boundary — `/pr-ci-failed` pushes onto the existing feature branch; `/master-ci-failed` creates a fresh feature branch + new PR. Everything before that boundary (identify → fetch logs → classify → reproduce → diagnose → self-review) is shared shape, expressed twice (each file is self-contained — no third "shared" file).

**Why two separate skills, not one parameterised skill.**
- Preconditions diverge sharply (`master` is a bail trigger in one and a bail-on-violation in the other).
- Allowed-tools lists differ (`/master-ci-failed` needs `gh pr create` + `git push -u origin *`; `/pr-ci-failed` does not).
- Skill-selection ergonomics: the user's mental model at invocation is "I'm on a PR branch and CI failed" vs. "master CI went red". Forcing a single `/ci-failed` with a `--master` flag pushes that decision into the wrong layer.
- The 40k-char cap is per-file; two ~18k files leave more headroom than one ~32k file with branching paragraphs.

**Why mirror `/pr-commented` (not `/bugfix`, not a new shape).**
- `/pr-commented` already encodes the closest-shape pieces this task needs: a YAML frontmatter with `disable-model-invocation: true`, a commit-authorisation banner pre-authorising commit + push, the Variant A compaction-recovery callout with the canonical cross-link, a Preconditions table, a numbered Workflow, an Edge cases table, an Anti-patterns list, a Gate checklist — and it already enforces the `learnings.md` boundary and the AXIOM 2 read.
- `/bugfix` is a poor template here: it owns a *trace-driven* workflow (Step 1 reproduce-and-trace → Step 2 root-cause confirmation → Step 3 failing test before fix) with explicit user-confirmation checkpoints. CI failures arrive with logs already in hand — no trace step, no user-confirmation gate for divergence-vs-expected.

**Rejected alternatives.**

1. *Single `/ci-failed` skill with `$ARGUMENTS=pr|master` mode.* Rejected — see "Why two separate skills" above; spec already pre-resolved this in *Key decisions*.
2. *A sister `ci-classifier` Opus agent owning log triage.* Rejected for v1 (spec § *Deferred* item 3). Inline classification in the skill body is sufficient until observed misclassification justifies the cost.
3. *Auto-invoke `/master-ci-failed` from `/pr-merged` when merge commit CI turns red.* Deferred to a separate issue (spec § *Deferred* item 1 + Open question 3). v1 has the user manually re-invoke; design records the wiring decision below.
4. *Watch mode (loop `gh pr checks` until green).* Deferred to a separate issue (spec § *Deferred* item 2). v1 re-poll is documented as optional Step 9 inside `/pr-ci-failed`.

### Resolutions to the four open questions

**Q1 — Bugfix delegation hand-off shape.**
**Decision:** `/bugfix` owns the full Trace → Root cause → Failing test → Fix → self-review → push loop after either CI-failed skill delegates. The calling skill exits cleanly at the delegation point with a one-line summary to the user (`Delegated to /bugfix for <class> regression; re-invoke /pr-ci-failed (or /master-ci-failed) after /bugfix completes if further CI rounds surface.`). Rationale: `/bugfix` already mandates `## Step 6.5 Self-review` (loop max 3, same semantics as `/task` Step 10) and writes its own trace artefact as durable state. Splitting ownership — calling skill keeps the push step — would force `/bugfix` to skip its own Step 7 cleanup and create an awkward two-skill push gate. The CI-fix self-review axiom is satisfied because `/bugfix` Step 6.5 enforces it.
**Audit-trail check:** `/bugfix`'s commit message format is workflow-agnostic; if traceability to "this fix originated from CI run R" matters, the user adds the run URL to the `/bugfix` trace's Reporter field at Step 1 — the trace artefact is retained inside the `/bugfix` flow until Step 7. Open follow-up (not in v1 scope): consider extending `/bugfix` Step 1 to auto-record the parent-skill's CI run URL when invoked from a CI-failed skill — track as `learnings.md` candidate if it actually causes audit-trail gaps in practice.

**Q2 — Coverage failure reproducer.**
**Decision (per-class table entry for `coverage`):**
```bash
timeout 600 xvfb-run -a cargo llvm-cov --workspace --lcov --output-path lcov.info --doctests
```
**Prerequisites recorded in the skill body** (so a user without the full coverage harness understands why it might not reproduce locally on first try):
- `cargo llvm-cov` installed (`cargo install cargo-llvm-cov`).
- `nightly` toolchain with `llvm-tools-preview` component.
- Linux GPU stack (`mesa-vulkan-drivers vulkan-tools xvfb libxkbcommon-x11-0 libfontconfig1-dev`) — on non-Linux hosts, fall back to `cargo llvm-cov --workspace --lcov --output-path lcov.info` without `xvfb-run` (loses xvfb_smoke coverage but covers the lint/test surface).
- Env vars `WGPU_BACKEND=vulkan WGPU_ADAPTER_NAME=llvmpipe LIBGL_ALWAYS_SOFTWARE=1` for parity with the workflow's lavapipe path.

If the local environment cannot run llvm-cov (missing toolchain, missing GPU stack, non-Linux without xvfb) — coverage falls back to the `other` class (pause-and-surface to user with the failing-step log excerpt). Source verified at `.github/workflows/coverage.yml:71`.

**Q3 — Auto-invocation of `/master-ci-failed` from `/pr-merged`.**
**Decision: deferred to a separate issue. Not wired in this PR.** Rationale: keeps the two skills loosely coupled in v1 — `/pr-merged` is near-stateless (cleanup-only); adding a CI-status probe inflates its scope and risks `/pr-merged` blocking on a slow `gh run list` call. Spec already records this under *Deferred*. Documentation deliverable in this design: the new `AGENTS.md` Agent-Docs row for `/master-ci-failed` notes "downstream of `/pr-merged` when the merge commit's CI subsequently turns red — invoked manually in v1" so the future auto-wire issue has a reference anchor.

**Q4 — "Latest failing master run" discovery for `/master-ci-failed`.**
**Decision:** Default = scan the **latest master commit only**. When `$ARGUMENTS` is empty:
1. `git rev-parse origin/master` → master HEAD SHA.
2. `gh run list --branch master --commit <sha> --status failure --json databaseId,name,headSha,event --limit 5` → enumerate failing runs on that commit.
3. If empty → bail with `No failing CI runs on master HEAD (<sha-short>). If you suspect an older still-red commit, re-invoke with the SHA as $ARGUMENTS.`
4. If non-empty → pick the most recent run (top of list) and proceed.
When `$ARGUMENTS` is set (a commit SHA), substitute that SHA in step 2; same bail wording adapted.

Rationale: scanning the last N master pushes broadens scope without a strong signal — the latest push is the case the user almost always wants. Older still-red commits are uncommon enough that the explicit `$ARGUMENTS` SHA is the right ergonomic escape hatch.

### File-size headroom verification

Per AGENTS.md `## Build & Test` 40k-char cap with 35k early-warning:

| File | Current size | Post-PR delta | Post-PR projected |
|---|---|---|---|
| `.claude/skills/pr-ci-failed/SKILL.md` | (new) | ~16–18k | ~16–18k |
| `.claude/skills/master-ci-failed/SKILL.md` | (new) | ~17–19k | ~17–19k |
| `AGENTS.md` | 35,529 | +2 Agent-Docs rows ≈ +800–1,100 chars | ~36,400–36,600 |

**AGENTS.md projection crosses the 35k early-warning AND the post-PR projection lands inside the 35k–40k danger band.** Two paths to choose between:

**Path A (preferred):** proceed with the two new rows; defer extraction to a follow-up issue. Reasoning: the 35k–40k band is a "proactive extraction pass" cue per AGENTS.md, not a hard stop, and 36,400 still gives ~3,600 chars of headroom (~3 average rows) before the 40k harness cap. The two new rows are themselves not extractable (they belong in `## Agent Docs`); extracting other rows requires a separate dedicated PR per the Boundary-rule-2 / Propagation-Rule precedent (instruction-file extractions are their own scope).

**Path B (alternative):** extract the verbose progress-file rows (`ai-docs/plans/*.progress.md`, `ai-docs/pr-comments/pr-<N>.progress.md`, `ai-docs/triage/...`, `ai-docs/bugfix/trace-*.md`) into a new `ai-docs/agent-docs-progress-files.md` reference page, leaving one-line stubs in `AGENTS.md`. This reclaims ~2,500–3,000 chars but enlarges the PR scope beyond CI-failed skills.

**Recommendation: Path A** — surface the early-warning to the user but proceed; track Path B as a follow-up extraction issue if a subsequent `/task` would push AGENTS.md over 38k.

### `gh` version preflight

Pre-task verification step in the implementing PR:

```bash
gh --version    # observed at design time: gh 2.81.0 (2026-05-01)
gh run view --help | grep -- '--log-failed'    # confirms flag presence
```

Local environment confirms `gh 2.81.0` supports `gh run view --log-failed`. Each new SKILL.md records the observed version + minimum-supported version in a one-line `## Pre-task verification` callout near the top (after the compaction-recovery callout). When `gh --version` < the recorded minimum, the skill body instructs the user to upgrade `gh` OR use the `gh api repos/:owner/:repo/actions/runs/<run-id>/logs` fallback (already documented in the spec).

Minimum-supported version recorded as `gh 2.4.0` (first stable release with `--log-failed` per the `gh` changelog) — pinned in the skill body so a future `gh` downgrade or fresh CI runner can still verify.

### Wiring with existing skills

- **Both skills downstream of `/task` Step 12** — user invokes after CI surfaces a failure on a PR / master commit.
- **`/pr-ci-failed`** integrates with `/task`'s progress file (`ai-docs/plans/<spec-base>.progress.md`) by appending `## CI-fix cycle round M` sections (symmetric with `/pr-commented`'s `## Comment cycle round M`). Fallback for non-`/task` PRs: `ai-docs/ci-fixes/pr-<N>.progress.md` (new gitignored directory).
- **`/master-ci-failed`** has no `/task` progress file at entry (no PR yet exists). Per-run progress file: `ai-docs/master-ci/<run-id>.progress.md` (new gitignored directory). The fresh PR's body includes a `**Tracked in:**` line referencing the master run-id; `/pr-merged`'s cleanup script (`scripts/cleanup-progress.sh`) discovers the file via that run-id reference after the PR merges.
  - **`/pr-merged` cleanup integration:** the existing `scripts/cleanup-progress.sh` derives the spec path from `Closes/Fixes/Resolves #N` in the PR body. `/master-ci-failed`'s fresh PR has no upstream tracking issue → no `Closes` line → cleanup script's `ISSUE_NUM` derivation will return empty and print the existing one-line warning to stderr. **Design choice:** extend the script in this PR to additionally probe for `**Tracked in run:** <run-id>` and delete `ai-docs/master-ci/<run-id>.progress.md` when found. This is a small, contained script change kept inside the same PR to keep the cleanup path coherent (AC-M6 names this expectation).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Author `.claude/skills/pr-ci-failed/SKILL.md` — full body following the `/pr-commented` template; YAML frontmatter with allowed-tools matching spec § *Technical constraints* row; commit-authorisation banner; Scope (In/Out); Variant A compaction callout (probe = `grep -l "Tracked in:.*#${PR_NUM}\b"`); Preconditions table; numbered Workflow (Step 0 identify-run → Step 1 progress-file open → Step 2 fetch + classify → Step 3 reproduce → Step 4 diagnose+fix → Step 5 self-review (loop cap 3) → Step 6 commit → Step 7 push + AXIOM-2 read → Step 8 optional re-poll); per-class reproducer table (incl. coverage command from Q2); Edge cases; Anti-patterns; Gate checklist; `## Pre-task verification` callout recording `gh --version` minimum (the callout is a MANDATORY body section in the design — satisfies AC6's optional-by-spec-wording in a stronger form, and acts as the AC2-enumerated section the design promotes from "optional" to "required") | `.claude/skills/pr-ci-failed/SKILL.md` (new) | — |
| 2 | Author `.claude/skills/master-ci-failed/SKILL.md` — same template shape as task 1, with the master-specific deltas: preconditions inverted (branch == master; no open PR on failing commit; latest-master-run discovery from Q4); fix-landing path = `git checkout -b fix/master-ci-<run-id>` + new PR via `gh pr create`; AXIOM-2 exemption noted (first push opens PR); deeper-regression escape hatch routes through `/bugfix` per Q1; new-PR body template with the four sections from spec § *Key decisions* incl. `**Tracked in run:** <run-id>` line; allowed-tools superset incl. `gh pr create` + `git push -u origin *`. **`gh run list --commit` sanity-check hint (per design-review note 3):** the Q4 discovery step uses `gh run list --branch master --commit <sha> --status failure --limit 5`. The `--commit` flag is a client-side filter in some `gh` versions; if the call returns zero rows but `gh run list --branch master --limit 1` returns the latest run, the SKILL body must surface a one-line hint: `gh --version may not support --commit filter; please upgrade or pass $ARGUMENTS run-id directly` — this is a SKILL-body wording addition, not a Decomposition change. The `## Pre-task verification` callout is MANDATORY here too (same rule as task 1) | `.claude/skills/master-ci-failed/SKILL.md` (new) | — |
| 3 | Extend `.gitignore` — add `/ai-docs/ci-fixes/` and `/ai-docs/master-ci/` directories (both gitignored, parallel with `/ai-docs/pr-comments/`) | `.gitignore` | — |
| 4 | Extend `.claude/skills/pr-merged/scripts/cleanup-progress.sh` — after the existing `Closes/Fixes/Resolves` path, add a secondary probe for `**Tracked in run:** <run-id>` in the PR body and delete `ai-docs/master-ci/<run-id>.progress.md` when found; also `rm -f ai-docs/ci-fixes/pr-${PR_NUM}.progress.md` parallel to the existing `pr-comments` fallback; opportunistic `rmdir ai-docs/ci-fixes ai-docs/master-ci 2>/dev/null \|\| true`; preserve existing exit-0 / non-fatal semantics. **Extend the top-of-file docstring (currently describes only the `Closes/Fixes/Resolves` derivation) with the new `**Tracked in run:**` secondary-probe path and its failure modes** (per design-review note 2 — keeps the script's prose interface in sync with the new derivation; future maintainers see both paths documented in the same shape as the existing `Closes/Fixes/Resolves` prose) | `.claude/skills/pr-merged/scripts/cleanup-progress.sh` | 3 |
| 5 | Add two new `## Agent Docs` rows in `AGENTS.md` — one per skill, immediately after the existing `/pr-commented` row, describing each skill, naming the gitignored progress-file path, calling out the `learnings.md` boundary, and noting "downstream of `/task` Step 12 (and `/pr-merged` for `/master-ci-failed`, manual invocation in v1)" | `AGENTS.md` | 1, 2 |
| 6 | Update `ai-docs/plans/INDEX.md` — add the new spec + design rows under the appropriate date section | `ai-docs/plans/INDEX.md` | 1, 2, 5 |
| 7 | Verification pass — run `wc -c` on both new SKILL.md files (< 20k each, hard-cap 40k); run `wc -c AGENTS.md` (verify < 40k, surface if ≥ 35k); run `actionlint` on any touched workflow (none expected in this PR — instruction-only); run a probe-fidelity check that the Variant A callout phrasing matches the locked body in `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md` | (no file edits — verification only) | 1, 2, 5 |

**Subtask count: 7.** Within the 7-subtask cap (design.md "If scope > 7 tasks in decomposition — propose splitting into multiple issues").

## Risks

- **AGENTS.md size growth crosses 35k early-warning.** Mitigation: Path A above (proceed; track Path B follow-up extraction issue). Surface to user at design-review time so the Path A vs Path B choice is explicit. **If user picks Path B**, add task 5.5 (extract progress-file rows) before task 5, and rescope task 5 accordingly.
- **Spec change between design and implementation.** Mitigation: spec is round-2 amended and stable; if a further round surfaces during implementation, return to `/task` Step 6 design re-invocation rather than patching in flight.
- **Allowed-tools list drift between SKILL.md frontmatter and actual `Bash(...)` calls in the body.** Mitigation: task 7 verification step enumerates every `Bash(...)` call in each SKILL.md body and confirms a matching entry in the frontmatter allowed-tools list. The spec already pins the exact lists.
- **Compaction-recovery callout body drift from the locked variant-A source.** Mitigation: the locked body lives in `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md` (treated as read-only history); tasks 1 + 2 copy verbatim from `/pr-commented`'s live body, swapping only the probe expression. Task 7 verification reads the callout from both new files and diffs against `/pr-commented` lines 28–54.
- **Cleanup-script change in task 4 has no unit test.** Mitigation: task 4 adds a bash-comment-block recipe documenting how to manually verify the new probe path: create a fake PR body locally, run the script's relevant grep against it, confirm the expected SHA-stripping output. The script is shell-only (already untested except by integration in `/pr-merged` itself) — matches existing convention.
- **Audit-trail gap on `/bugfix` delegation (Q1).** Mitigation: explicit one-liner in both SKILL.md bodies instructing the calling skill to surface the CI run URL to `/bugfix` at delegation (user manually carries it forward in v1; auto-record is a deferred enhancement noted in Q1).
- **`gh` version regression on a fresh runner.** Mitigation: each SKILL.md documents both the primary `--log-failed` path and the `gh api repos/:owner/:repo/actions/runs/<run-id>/logs` fallback. The minimum recorded `gh 2.4.0` is conservative enough that any user with `gh` newer than ~2023 will not hit the fallback.
- **`/master-ci-failed` race with concurrent push to master.** Mitigation: skill body documents the bail "If master HEAD has moved since the run was identified, surface `master moved ahead during fix; re-fetch and re-invoke`." Re-fetch happens at preconditions via `git fetch origin master`. No automatic rebase.
- **Multi-job failure where root causes diverge.** Spec already pre-resolved (handle first failing job per invocation, advise re-invocation). Mitigation: SKILL.md Workflow Step 0 documents this verbatim.

## Test Design

This is an instruction-file-only PR (no production Rust code). Test surface is **verification-style**, not unit/integration test-style:

- **File-size verification.** Per AC8: `wc -c .claude/skills/pr-ci-failed/SKILL.md .claude/skills/master-ci-failed/SKILL.md` — each < 20,000 chars (ideal) and < 40,000 chars (hard cap). `wc -c AGENTS.md` — surface to user if ≥ 35,000 chars. Scenario coverage: happy (both < 20k); near-limit (one between 20k–35k, surface warning to user); over-limit (any ≥ 40k → REJECT at self-review).
- **Allowed-tools fidelity verification.** For each new SKILL.md: enumerate `Bash(...)` calls in the body, intersect with the YAML frontmatter `allowed-tools` list, confirm no extras and no missing. Helper: `grep -oE 'Bash\(([a-z]+( -?[a-zA-Z]+)*)' <file>` against the frontmatter line.
- **AC-by-AC fidelity verification.** For each of AC1–AC11, AC-P1–AC-P4, AC-M1–AC-M6 (21 ACs), one verification step naming the location in the SKILL.md (section + line range) that satisfies it. Self-review Round 1 covers this.
- **Compaction-recovery callout drift verification.** `diff <(sed -n '28,54p' .claude/skills/pr-commented/SKILL.md) <(sed -n '<eq-range>' .claude/skills/pr-ci-failed/SKILL.md)` modulo the probe-expression line — the surrounding boilerplate (invariant phrase `"re-enter this skill from the top of its body"`, cross-link target) must match verbatim.
- **`/pr-merged` cleanup-script smoke test.** After task 4: create a temporary throwaway file `ai-docs/master-ci/test-run-12345.progress.md`, fabricate a PR body locally that contains `**Tracked in run:** 12345`, run the new probe block against the fabricated body, confirm the file is deleted and the script exits 0. Manual / one-off — recorded as a comment block in the script alongside the existing manual-verification recipe.
- **`gh --version` preflight.** Run `gh --version` and `gh run view --help | grep -- '--log-failed'`. Confirm both succeed.

**Test locations / fixtures: none.** No `#[cfg(test)]` module added; no Rust test file added; no `tests/` directory touched. The AGENTS.md "Any file with substantial logic (~50+ lines of non-trivial code) must have a `#[cfg(test)] mod tests` block" rule applies to Rust source files, not Markdown skill bodies or bash cleanup scripts.

## Open questions

None remaining — all four spec § *Open questions* are resolved above (Q1, Q2, Q3, Q4) and the AGENTS.md size-headroom choice (Path A vs Path B) is surfaced for the user / design-review agent to ratify during the design-review pass.

## Verdict request to `design-review`

This design is submitted for a GO / ITERATE / STOP verdict per the standard `/task` Step 7 contract.

- **GO** = proceed to implementation (task 1 onward).
- **GO-with-notes** = proceed; notes addressed during implementation, not in a re-design round.
- **ITERATE** = blockers itemised; design agent re-runs §Approach + §Decomposition only for the affected items, not a full rewrite.
- **STOP** = design fundamentally wrong; return to `/task` Step 6 with a re-scoped spec.

Specific items the reviewer is asked to ratify:

1. **AGENTS.md size — Path A vs Path B.** Proceeding with Path A (defer extraction) lands AGENTS.md at ~36,400 chars (3.6k headroom). Path B (extract progress-file rows) reclaims ~2,500–3,000 chars but enlarges PR scope. **Recommendation: Path A.**
2. **`/bugfix` hand-off shape (Q1).** Calling skill exits at delegation; `/bugfix` owns push. **Recommendation: confirmed, no audit-trail gap in v1.**
3. **Coverage reproducer (Q2).** The full `timeout 600 xvfb-run -a cargo llvm-cov --workspace --lcov --output-path lcov.info --doctests` command is recorded with documented prerequisites and a degradation path to the `other` class.
4. **Auto-invoke `/master-ci-failed` from `/pr-merged` (Q3).** Deferred; v1 is manual. **Recommendation: confirmed deferred.**
5. **Latest-failing-run discovery (Q4).** Scan latest master commit only; `$ARGUMENTS` SHA is the escape hatch. **Recommendation: confirmed.**
6. **Subtask count = 7.** Within the design.md cap. **Recommendation: confirmed atomic.**
