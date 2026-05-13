# Fix cleanup-progress.sh issue-number derivation

**Source:** user description (free-text)
**Date:** 2026-05-13
**Tracked in:** #325

## Context

`/pr-merged` step 3 invokes `.claude/skills/pr-merged/scripts/cleanup-progress.sh <previous-branch>` to delete the merged task's local `.progress.md` files (gitignored agent artefacts). Line 47 of that script:

```bash
SPEC_PATH=$(grep -l "Tracked in:.*#${PR_NUM}\b" ai-docs/plans/done/*.spec.md ai-docs/plans/*.spec.md 2>/dev/null | head -n1)
```

greps for `Tracked in: #<PR_NUM>`, but the `/task` + `/interview` workflow writes `**Tracked in:** #<ISSUE_NUM>` into every spec. The two numbers are different (PR #324 closed issue #323; observed in this very bug report). As a result `SPEC_PATH` is empty, the `${SPEC_BASE}.progress.md` deletion is skipped, the script exits 0 silently, and the merged task's `.progress.md` lingers on disk until the user cleans it up manually. The branch-delete step (workflow step 4) is unaffected because it doesn't depend on the script.

Convention check (`grep -E "Tracked in:.*#[0-9]+" ai-docs/plans/**/*.spec.md`): every spec — past and present — uses the issue number. No spec uses the PR number. The current grep pattern has never matched and the bug has been latent in every `/task`-produced PR since `cleanup-progress.sh` was introduced (PR #300).

The docstring at lines 8–30 enumerates the derivation chain as "PR number → grep specs for `Tracked in: #N`" which matches the broken implementation, not the actual spec convention; it must be corrected alongside the code fix.

## Scope

1. Replace the PR-number grep on line 47 with a derivation that finds the spec via the **issue number** the merged PR closed. The mechanism (option A / B / C below, or a refinement) is a design decision; the spec's contract is only that the script reliably identifies the matching spec for any `/task`-produced PR.
2. Update the docstring at lines 8–30 to describe the actual derivation chain after the fix (the current docstring is wrong on point 2 of the "Derivation" enumeration).
3. Verify the fix by exercising the script against a realistic merged-PR scenario (mechanism — synthetic fixture vs. retroactive re-run against PR #324's reconstructed artefacts — design's choice).

## Out of scope

- Refactoring `cleanup-progress.sh` beyond what the fix requires (rewrite to a different language, extracting helpers, etc.).
- Adding a bash unit-test harness to the repo. No such harness exists today and introducing one is a separate tooling chore.
- Editing `.claude/skills/pr-merged/SKILL.md` — the workflow steps don't change; only the script's internal derivation does. (The SKILL.md's mention of "no matching `/task` spec" failure mode in step 3 stays accurate.)
- Adding a CI / repo-level gate that catches future "orphaned `.progress.md` on master" regressions. Useful, but a separate tooling task.
- Touching any other script under `.claude/skills/**/scripts/` or under `scripts/`.
- Retroactively cleaning up any `.progress.md` files left behind by previous `/task` PRs (those are local-only / gitignored; users can delete them at leisure, and the issue does not request a sweep).

## Deferred

- CI gate for "no orphaned `.progress.md` on master" | useful safety net but requires a separate design discussion and a separate workflow file | yes, separate issue if/when escalated.
- Bash unit-test harness for `.claude/skills/**/scripts/*.sh` | useful for future script changes but a workspace-wide tooling decision | yes, separate issue if/when escalated.

## Key decisions

| Question | Decision |
|---|---|
| Source of the spec-lookup key | Issue number, derived from the merged PR. The script must call `gh` (or otherwise inspect the merged PR) to obtain the issue number it closes, then grep `Tracked in:.*#${ISSUE_NUM}\b` in `ai-docs/plans/done/*.spec.md` and `ai-docs/plans/*.spec.md`. The PR-number grep on line 47 is the source of the bug; it must not be retained as the *primary* lookup. Design may choose to retain it as a *secondary* fallback (option C) only if a defensible case exists for a spec written with `Tracked in: #<PR>` — today no such spec exists. |
| Issue-number derivation | Parse the merged PR body for the GitHub closing-issue keywords (`Closes`, `Fixes`, `Resolves`, optionally `Close`, `Closed`, `Fix`, `Fixed`, `Resolve`, `Resolved`) followed by `#<N>` (cross-repo `owner/repo#N` form out of scope — this repo's `/task` workflow never produces those). Exact regex and case-sensitivity are design-phase details. |
| Failure handling when issue number cannot be derived | Script must still exit 0 (the workflow continues to step 4 — branch delete — regardless), but the case must be observable (stderr warning), not silent. The current "silent no-op" behaviour is the second-order bug that hid the primary regression for the script's entire lifetime. |
| Sanity-check / orphan-detection stderr warning | Design's call whether to emit a warning when `${PR_NUM}` is non-empty AND no spec was matched AND a `.progress.md` exists in `ai-docs/plans/` whose basename overlaps the merged branch's `<prefix>/<date>-<slug>` slug. The user flagged it as optional; the value is converting future regressions of this kind into visible signals. Design may include it, decline it (with rationale), or scope it to a follow-up. |
| Fix option (A / B / C from issue body) | Design's call. Constraints: must not be silent on miss (Key Decision row 3); must reflect that no current spec carries a `Tracked in: #<PR>` line (Key Decision row 1). |
| Verification approach | Design's call. Acceptable: synthetic-fixture run, retroactive re-run against PR #324's reconstructed artefacts, or a one-shot manual recipe documented in the design doc. No automated test harness is in scope. |

## Technical constraints

- `bash` script, `set -uo pipefail`. No new external tools beyond what `pr-merged` already allows (`gh`, standard POSIX utilities). `jq` is acceptable — already used elsewhere by `gh --jq`.
- Must remain idempotent — re-running on a branch whose progress file has already been deleted must still exit 0 and not error.
- `.claude/skills/pr-merged/SKILL.md`'s `allowed-tools:` directive enumerates the commands the skill may invoke; the script can invoke any subprocess (the script is run as one whitelisted command, `Bash(.claude/skills/pr-merged/scripts/cleanup-progress.sh *)`). No `allowed-tools` change is required to add a `gh pr view` call inside the script.
- Docstring at lines 8–30 must accurately describe the post-fix derivation chain. Mis-describing the lookup is what made this bug hard to spot during review of PR #300.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | For a merged PR whose body contains `Closes #<N>` (or `Fixes`/`Resolves` equivalents) referencing an issue number, and whose matching spec under `ai-docs/plans/done/*.spec.md` or `ai-docs/plans/*.spec.md` carries `**Tracked in:** #<N>`, `cleanup-progress.sh <branch>` deletes the matching `ai-docs/plans/<spec-base>.progress.md` file. |
| AC2 | The retroactive case — PR #324 (`feat/2026-05-13-shrink-agents-md`, closing issue #323, spec `ai-docs/plans/done/2026-05-13-shrink-agents-md.spec.md`) — is exercised end-to-end against the fixed script (with a reconstructed `2026-05-13-shrink-agents-md.progress.md` placeholder so the rm has a target). The placeholder is deleted; the script exits 0. Recipe documented in the design doc or in the PR description. |
| AC3 | When `${PR_NUM}` is non-empty AND no `Closes/Fixes/Resolves #N` is parseable from the merged PR body, the script emits a stderr message naming the branch and exits 0 (workflow continues). The exact message wording is design's call; "silent" is not acceptable. |
| AC4 | When `${PR_NUM}` is non-empty AND an issue number is derived AND no spec matches `Tracked in:.*#${ISSUE_NUM}\b`, the script emits a stderr message naming the issue number and exits 0 (workflow continues). Wording is design's call. |
| AC5 | The script's docstring (lines 8–30, or wherever it lands post-edit) accurately describes the post-fix derivation chain. A reader can predict the script's behaviour from the docstring alone. |
| AC6 | `shellcheck .claude/skills/pr-merged/scripts/cleanup-progress.sh` is clean (no errors or warnings). If `shellcheck` is not currently installed on developer machines, manual inspection against `shellcheck.net` once is acceptable; the AC is "no shellcheck-detectable issues", not "shellcheck must be present in CI". |
| AC7 | Re-running the script on the same `<previous-branch>` a second time (after step 1's `rm -f` already deleted the progress file) exits 0 with no error (idempotency preserved). |
| AC8 | `cargo build`, `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all clean (sanity gate; no Rust changes expected, but the project gates still run on the feature branch). |

## Open questions

- **Should the sanity-net stderr warning also fire when `${PR_NUM}` IS empty but a `.progress.md` whose slug matches the branch exists?** The current `${PR_NUM}` empty path prints `pr-merged: no merged PR found for <branch>; skipping progress-file cleanup.` and exits 0. Extending the orphan-check to that path would catch the "branch merged outside `gh`" case as well. Design's call — recorded here as a candidate refinement, not an AC.
- **Should the issue body keyword list include the conjugated forms (`Closed`, `Fixed`, `Resolved`)?** GitHub's auto-close parser accepts them, but `/task` PR bodies in this repo consistently use the bare imperative (`Closes #N`). Design may choose to over-cover (cheap) or stay minimal (simpler regex).
