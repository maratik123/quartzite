# Design: Fix cleanup-progress.sh issue-number derivation

**Issue:** #325
**Date:** 2026-05-13

## Approach

### Chosen solution

Replace line 47 of `.claude/skills/pr-merged/scripts/cleanup-progress.sh` with a two-step derivation:

1. Inject a new step between the existing `PR_NUM` lookup and the spec grep that fetches the merged PR body via `gh pr view` and extracts the **first** issue number referenced by a GitHub closing-issue keyword (`Closes` / `Closed` / `Fixes` / `Fixed` / `Resolves` / `Resolved` — case-insensitive; cross-repo `owner/repo#N` form out of scope).
2. Use the derived `ISSUE_NUM` as the spec-lookup key in the existing `grep -l "Tracked in:.*#${ISSUE_NUM}\b" …` call. The PR-number grep is **dropped entirely** (spec's option A) — see "Rejected alternatives" for why retaining it as a fallback is net negative.

Each miss case (no closing-keyword line in the PR body; no spec matches the derived issue number) emits a one-line stderr warning naming the merged branch and the relevant identifier (PR number, issue number), then continues to the `/pr-comments` fallback path. The script's exit-0 contract is preserved (workflow step 4 — `git branch -d` — runs regardless of step 3's outcome).

### Exact replacement for the line-47 block

The current line 47 (one line) becomes a multi-line block (locations of new lines below shown relative to the existing structure):

```bash
# After line 45 (`fi` closing the empty-PR_NUM guard), before line 47's grep:
ISSUE_NUM=$(gh pr view "${PR_NUM}" --json body --jq '.body' 2>/dev/null \
  | grep -oiE '(Close[sd]?|Fix(es|ed)?|Resolve[sd]?) #[0-9]+' \
  | head -n1 \
  | grep -oE '[0-9]+$')

if [ -z "${ISSUE_NUM}" ]; then
  printf 'pr-merged: PR #%s body has no Closes/Fixes/Resolves #N line; skipping spec-driven progress-file cleanup for %s.\n' "${PR_NUM}" "${PREV_BRANCH}" >&2
else
  SPEC_PATH=$(grep -lE "^\*\*Tracked in:\*\* #${ISSUE_NUM}\b" ai-docs/plans/done/*.spec.md ai-docs/plans/*.spec.md 2>/dev/null | head -n1)
  if [ -n "${SPEC_PATH}" ]; then
    SPEC_BASE=$(basename "${SPEC_PATH}" .spec.md)
    rm -f "ai-docs/plans/${SPEC_BASE}.progress.md"
  else
    printf 'pr-merged: no /task spec matches Tracked in: #%s (derived from PR #%s, branch %s); skipping spec-driven progress-file cleanup.\n' "${ISSUE_NUM}" "${PR_NUM}" "${PREV_BRANCH}" >&2
  fi
fi
```

The existing `rm -f "ai-docs/pr-comments/pr-${PR_NUM}.progress.md"` and `rmdir ai-docs/pr-comments 2>/dev/null || true` lines remain unchanged below this block — they operate on `${PR_NUM}` (correct) and form the `/pr-commented`-PR fallback path.

### Regex form & justification

`grep -oiE '(Close[sd]?|Fix(es|ed)?|Resolve[sd]?) #[0-9]+'`:

- `-o` prints only the matched substring (so the trailing `head -n1 | grep -oE '[0-9]+$'` extracts the number cleanly).
- `-i` makes keywords case-insensitive (`closes`, `Closes`, `CLOSES` all match — GitHub's auto-close parser is case-insensitive; matching its behaviour avoids false negatives).
- `-E` enables ERE (alternation, `?`).
- `Close[sd]?` covers `Close` / `Closes` / `Closed`.
- `Fix(es|ed)?` covers `Fix` / `Fixes` / `Fixed`.
- `Resolve[sd]?` covers `Resolve` / `Resolves` / `Resolved`.
- ` #[0-9]+` — single literal space (GitHub's auto-close grammar requires exactly one space between the keyword and `#`; no tab variant). Trailing `\b` is **not** needed because `grep -o` only prints the match and `[0-9]+` is greedy — any trailing `.`, comma, newline, or word boundary already terminates the digit run.

This covers all conjugations the spec's Open Question #2 enumerates (`Closes`/`Fixes`/`Resolves` plus `Closed`/`Fixed`/`Resolved`; the bare-imperative forms `Close`/`Fix`/`Resolve` come for free from `[sd]?`/`(es|ed)?`). See "Open question resolution" below.

#### Spec-lookup grep — anchored to the literal field

The inner spec-grep is `grep -lE "^\*\*Tracked in:\*\* #${ISSUE_NUM}\b" …`:

- `^\*\*Tracked in:\*\* ` anchors the match to the literal `**Tracked in:** ` field at the **start of a line**, matching `/interview`'s exact convention. The four backslash-escaped asterisks are required because `-E` (ERE) treats `*` as a metacharacter; `\*` makes it literal.
- `#${ISSUE_NUM}\b` — the trailing `\b` (word boundary) terminates the digit run cleanly, so a search for `#32` does **not** false-match `**Tracked in:** #323`.
- `-l` (list-only) preserves the existing pipeline shape; `head -n1` after picks the first match.

**Why the anchor matters.** The inherited pre-fix pattern `Tracked in:.*#${ISSUE_NUM}\b` (with the loose `.*` and no line anchor) spans the entire line and matches **any** prose line that contains the literal substring `Tracked in:` **plus** `#<N>` anywhere downstream. AC4 verification on this very PR surfaced the false-positive: when the AC4 synthetic-fallback recipe moved the AC2 spec (`Tracked in: #323`) aside and re-ran the script, the loose grep falsely matched `ai-docs/plans/2026-05-13-cleanup-progress-issue-derive.spec.md` — because its *line 15* prose contains both `Tracked in:` (quoting the broken-grep substring) and `#323` (citing the example in flowing text). The spec's actual `**Tracked in:**` field on that line is `#325`, not `#323`. Anchoring to `^\*\*Tracked in:\*\* ` ensures the match is the literal field at line start (the convention `/interview` writes), not prose that happens to mention the substring.

The amendment carries from the line-47 replacement block above down through every code/docstring reference to the spec-lookup grep — the loose pattern is gone everywhere.

### Multi-close handling

`head -n1` picks the **first** closing-keyword line in the PR body. Justification:

- Every `/task`-produced PR in this repo closes exactly one issue (the tracking issue from `/interview`). The multi-close case is the exception, not the rule.
- For the multi-close case the repo has on record (PR #295 — `Closes #289` then `Closes #277`), the **first** closure is the primary task issue and matches a spec (`Tracked in: #289`), while subsequent closures are bundled satellite issues without their own spec. "First match" is the correct default.
- A future multi-close PR whose first-listed issue does not match a spec triggers the AC4 stderr warning, which surfaces the mismatch for the user. This is graceful degradation, not silent failure.

### Rejected alternatives

- **Option C (PR-number grep as secondary fallback).** Rejected: no spec in the repo has ever used `Tracked in: #<PR>` (verified by `grep -E "Tracked in:.*#[0-9]+" ai-docs/plans/**/*.spec.md` — every match is an issue number). The `/task` + `/interview` convention writes the issue number; the PR number is the secondary identifier. Retaining the PR grep would (a) be dead code (no spec to match), (b) mask future regressions where the PR body is malformed (the script would "succeed" by accidentally matching a spec whose issue number happened to equal the PR number — increasingly likely as N grows), and (c) complicate the docstring and the AC4 warning. Clean break.
- **Keep the inherited loose `Tracked in:.*#${ISSUE_NUM}\b` spec-lookup grep.** Rejected during this PR's AC4 verification: the loose `.*` regex spans an entire line and false-matches any prose line that contains the literal substring `Tracked in:` plus `#<N>` somewhere downstream — even when the spec's actual `**Tracked in:**` field carries a different issue number. The concrete case: while exercising the AC4 synthetic-fallback recipe (move `Tracked in: #323`'s spec aside, expect AC4 stderr), the loose grep falsely matched `ai-docs/plans/2026-05-13-cleanup-progress-issue-derive.spec.md` because *line 15* of this very spec contains both `Tracked in:` (quoting the broken-grep substring) and `#323` (citing the example in prose). The spec's actual `**Tracked in:**` field is `#325`, not `#323`. The fix — anchor to `^\*\*Tracked in:\*\* #${ISSUE_NUM}\b` (line-start + literal-field + word-boundary digit terminator) — is documented under § *Regex form & justification* → *Spec-lookup grep — anchored to the literal field*.
- **Iterate every `Closes #N` line, OR them all into one grep.** Rejected for YAGNI: the multi-close case is rare, and "first match" is correct for the only multi-close PR on record. If a future spec genuinely needs multi-issue cleanup, design it then.
- **Re-fetch the body and pipe through `jq` for line parsing instead of `grep -oiE`.** Rejected: `jq` is already used by the `gh --jq '.body'` call to get the body; a second `jq` invocation to do regex extraction adds a dependency on `jq` regex syntax for no benefit — `grep -oiE` is POSIX-portable and reads more directly.
- **Add a CI gate that catches orphan `.progress.md` files on `master`.** Out of scope (spec § *Out of scope*; spec § *Deferred*).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Replace line 47's PR-number grep with the two-step `gh pr view` → `grep -oiE` → `ISSUE_NUM` → spec grep block. Emit stderr warnings on both miss cases (AC3, AC4). | `.claude/skills/pr-merged/scripts/cleanup-progress.sh` | — |
| 2 | Rewrite the docstring (lines 8–30) to describe the post-fix derivation chain (Derivation steps 1–7, Failure modes updated for the two new stderr cases). | `.claude/skills/pr-merged/scripts/cleanup-progress.sh` | 1 |
| 3 | Run `shellcheck` on the modified script; resolve any new warnings/errors. Run the AC2 retroactive recipe end-to-end against PR #324 to verify (recipe in *Test Design* below). | `.claude/skills/pr-merged/scripts/cleanup-progress.sh` | 1, 2 |

Task 1 + Task 2 land in the same commit (they jointly fix the bug — orphan-docstring without orphan-code is meaningless and vice versa). Task 3 is verification, no code changes if shellcheck stays clean.

## Risks

- **`gh` CLI dependency.** Mitigation: the script already calls `gh pr list` at line 40, so adding `gh pr view` is no new dependency surface. If `gh pr view` fails (network, auth, deleted PR), the inner `grep -oiE` finds nothing, `ISSUE_NUM` is empty, the AC3 stderr warning fires, and the script exits 0. Failure is observable and non-blocking.
- **Multi-close parsing ambiguity.** The "first match" rule (above) is correct for the multi-close case on record (PR #295). A future PR with multi-close + non-spec-matching first closure triggers the AC4 stderr warning, which is the correct graceful-degradation behaviour. Documented in the new docstring.
- **PR body lacks a closing keyword.** Many merged PRs in this repo have no `Closes #N` line at all (PR #321, #320, #319, #322 …). The AC3 stderr warning fires and the script exits 0 — this is the documented behaviour for non-`/task` PRs and matches the SKILL.md's existing "manual PR without `/task`" path. No regression.
- **PR body uses cross-repo `owner/repo#N` form.** Out of scope per spec § *Key decisions* row "Issue-number derivation". The regex's literal `#` deliberately fails to match `owner/repo#N`; AC3 stderr warning fires; behaviour is graceful and documented.
- **PR body uses a non-ASCII digit / unicode-confusable.** Not a real risk — GitHub itself only renders ASCII `#[0-9]+` as a closure reference, and every `/task` PR is authored via `gh pr create` from this skill family.
- **`shellcheck` complains about the new `2>/dev/null` on the inner `gh pr view`.** Pre-empted: `shellcheck` warns about `2>/dev/null` swallowing genuine errors only in narrow contexts (e.g., before assigning to a variable that's then unguarded). The block guards `ISSUE_NUM` empty immediately after assignment, so the pattern is safe. Verified in Task 3.
- **Idempotency (AC7).** The second invocation finds `${SPEC_BASE}.progress.md` already absent; `rm -f` is silent on missing files; exit 0 preserved. No new code path mutates this contract.
- **The script keeps `set -uo pipefail`** — no `set -e`. The new `gh pr view | grep | head | grep` pipeline can have `head -n1` close early and the producing `grep -oiE` exit non-zero (SIGPIPE); `pipefail` would then propagate that as the assignment's exit code. Mitigation: the assignment is to a variable, not the script's exit status, so `pipefail` does not abort. `ISSUE_NUM` simply ends up empty in any pipeline-failure scenario, triggering the AC3 warning. No change to the exit-code contract.
- **SKILL.md prose drift (accepted tech debt).** `.claude/skills/pr-merged/SKILL.md` step 3's prose currently reads "**No matching `/task` spec** (manual PR without `/task`): skips the `/task`-progress-file deletion silently". After this fix the script emits a stderr warning before skipping, so the word `silently` is one word stale. Spec § *Out of scope* explicitly excludes SKILL.md edits ("The SKILL.md's mention of 'no matching `/task` spec' failure mode in step 3 stays accurate"); changing the word here would be a scope expansion contrary to the spec author's intent. **Mitigation:** the drift is one word; a future doc-cleanup pass may consolidate it with other SKILL.md prose touches. Logged as a follow-up doc chore (see § *Deferred follow-ups* below); no action in this PR.

## Deferred follow-ups

- **SKILL.md `silently` prose touch-up.** `.claude/skills/pr-merged/SKILL.md` step 3 will be one word stale after this PR (`silently` no longer matches the script's behaviour). Spec § *Out of scope* excludes SKILL.md edits in this PR; defer to a future doc-cleanup pass that may consolidate multiple SKILL.md prose drifts at once. No issue filed — the drift is too small to escalate on its own.

## Test Design

This is a `bash` script change, not Rust. No `#[cfg(test)]` module is added (spec § *Out of scope* explicitly excludes a bash unit-test harness). Verification is the AC2 end-to-end retroactive recipe.

### AC2 retroactive recipe (PR #324 — `feat/2026-05-13-shrink-agents-md`, closing issue #323, spec `ai-docs/plans/done/2026-05-13-shrink-agents-md.spec.md`)

Run from the repo root, **on the feature branch carrying the fix**:

```bash
# 1. Reconstruct the gitignored progress placeholder so the script has a target.
mkdir -p ai-docs/plans
printf '# placeholder for AC2 verification\n' > ai-docs/plans/2026-05-13-shrink-agents-md.progress.md
ls ai-docs/plans/2026-05-13-shrink-agents-md.progress.md   # confirms the placeholder exists

# 2. Drive the fixed script as the real workflow would (the branch already
# merged into master, so `gh pr list --state merged --head feat/...` returns
# PR #324 — no fixture needed).
bash .claude/skills/pr-merged/scripts/cleanup-progress.sh feat/2026-05-13-shrink-agents-md
echo "exit=$?"

# 3. Assert the placeholder is gone and the script exited 0.
test ! -e ai-docs/plans/2026-05-13-shrink-agents-md.progress.md \
  && echo "AC2 PASS: progress file deleted" \
  || echo "AC2 FAIL: progress file still present"
```

### AC7 idempotency (separate from AC4 — same matched-spec path, `rm -f` no-op)

Re-run the script against the same branch immediately after the AC2 run above. Run 1 deleted the placeholder; run 2 still finds the matching spec via the same `gh` + spec-grep path, but the `rm -f` becomes a no-op because the file is already gone. Both lookups succeed, so **no stderr warning fires** — the only difference from run 1 is that the file system is unchanged.

```bash
# Pre-condition: AC2 recipe above has completed; the progress file is absent.
bash .claude/skills/pr-merged/scripts/cleanup-progress.sh feat/2026-05-13-shrink-agents-md 2> /tmp/ac7.stderr
echo "exit=$?  # expect 0"

# Assert: exit 0, progress file still absent, no stderr emitted from the spec-driven path.
test ! -e ai-docs/plans/2026-05-13-shrink-agents-md.progress.md \
  && echo "AC7 PASS: file still absent (rm -f no-op)" \
  || echo "AC7 FAIL: file reappeared"

# Assert: no AC3/AC4 stderr warning fired (both `Closes #N` parse and spec grep succeeded —
# only the `rm -f` became a no-op, and `rm -f` is silent on missing files by design).
! grep -E 'no Closes/Fixes/Resolves|no /task spec matches' /tmp/ac7.stderr \
  && echo "AC7 PASS: no spurious stderr" \
  || echo "AC7 FAIL: unexpected stderr from spec-driven path"
```

Distinction from AC4: AC4 exercises the spec-grep miss path (different recipe — temporarily move the spec aside, see § *Synthetic miss-case fixtures* below). AC7 exercises the matched-spec happy-path on a second invocation where the progress file is already absent. The two are independent acceptance criteria.

### Synthetic miss-case fixtures (AC3, AC4)

Both miss cases are exercised against real merged PRs in the repo — no synthetic gh fixture is needed:

- **AC3 (no `Closes/Fixes/Resolves #N` in PR body).** PR #321 (recent merge with empty closing-keyword set). Run:
  ```bash
  # PR #321's head branch is `chore/improve-2026-05-13` per `gh pr view 321 --json headRefName`.
  bash .claude/skills/pr-merged/scripts/cleanup-progress.sh chore/improve-2026-05-13 2>&1 1>/dev/null | grep -F 'has no Closes/Fixes/Resolves #N line'
  ```
  Expect: stderr warning matches; exit 0.

- **AC4 (issue number derived, no matching spec).** Two ways to exercise this path:

  *Primary (real-PR cross-check) — PR #172 (`Closes #167`, no matching spec).* A sweep of merged PRs against `**Tracked in:** #<N>` lines in `ai-docs/plans/done/*.spec.md` + `ai-docs/plans/*.spec.md` identified PR #172 (`chore/2026-05-08-rewrite-agents-md`, body contains `Closes #167`) as a merged PR whose closing-keyword issue has **no** matching spec — issue #167 has no `**Tracked in:** #167` line anywhere in the plans tree. This is a non-destructive fixture: no spec is moved, no progress file is created or deleted, the real `gh pr view` path is exercised end-to-end.

  Sanity-check the fixture's preconditions, then drive the script:
  ```bash
  # Preconditions (run once; both should print the expected sentinel).
  gh pr view 172 --json headRefName --jq '.headRefName'
  # expect: chore/2026-05-08-rewrite-agents-md
  grep -E '^\*\*Tracked in:\*\* #167\b' ai-docs/plans/done/*.spec.md ai-docs/plans/*.spec.md 2>/dev/null \
    && echo 'AC4 FIXTURE INVALID: a spec matches #167 — fall back to the synthetic recipe below' \
    || echo 'AC4 fixture OK: no spec matches #167'

  # Drive the fixed script against PR #172's head branch; capture stderr.
  bash .claude/skills/pr-merged/scripts/cleanup-progress.sh "$(gh pr view 172 --json headRefName --jq .headRefName)" 2>&1 1>/dev/null \
    | grep -F 'no /task spec matches Tracked in: #167'
  echo "exit=$?  # expect 0 (grep found the AC4 warning)"
  ```
  Expect: stderr warning naming issue #167; script exits 0; no file system mutation. The fixture is stable for as long as PR #172 exists and no spec retroactively gains `**Tracked in:** #167` — both unlikely.

  *Fallback (synthetic move-spec-aside) — only when the primary fixture is invalidated.* If at some future point a spec carries `**Tracked in:** #167` (the sanity check above fires the "AC4 FIXTURE INVALID" branch) or PR #172 becomes inaccessible, use this synthetic recipe: temporarily move the AC2 matching spec aside so the same `gh` lookup that succeeds in AC2 now misses the spec grep.
  ```bash
  # Stash the spec, re-run AC2, expect AC4 stderr.
  mv ai-docs/plans/done/2026-05-13-shrink-agents-md.spec.md /tmp/ac4-stash.spec.md
  trap 'mv /tmp/ac4-stash.spec.md ai-docs/plans/done/2026-05-13-shrink-agents-md.spec.md' EXIT
  bash .claude/skills/pr-merged/scripts/cleanup-progress.sh feat/2026-05-13-shrink-agents-md 2>&1 1>/dev/null \
    | grep -F 'no /task spec matches Tracked in: #323'
  echo "exit=$?  # expect 0 (grep found the warning)"
  # Untrap and restore explicitly so the rest of the session sees the file back.
  trap - EXIT
  mv /tmp/ac4-stash.spec.md ai-docs/plans/done/2026-05-13-shrink-agents-md.spec.md
  ```
  Expect: stderr warning matches the AC4 wording for issue #323; exit 0; spec restored.

  Why this ordering: the real-PR fixture is non-destructive, exercises the full `gh pr view` path (the primary risk surface), and avoids the false-positive class that the pre-amendment loose-grep design hit during AC4 verification (a prose line in the AC2-moved-aside scenario mentioned the substring `Tracked in:` + `#323`, falsely matching this very spec). Anchoring the spec-lookup grep to `^\*\*Tracked in:\*\* ` closes that false-positive — and the primary recipe is now structured to exercise the path against a genuinely-no-spec issue rather than a temporarily-no-spec issue.

### AC6 — shellcheck

```bash
shellcheck .claude/skills/pr-merged/scripts/cleanup-progress.sh
echo "exit=$?  # expect 0"
```

Verified pre-fix: the current script is shellcheck-clean on v0.9.0. The new block uses only constructs already in the script (command substitution with `$()`, `2>/dev/null` redirect, `grep -oE`, parameterised `printf >&2`, `if [ -z … ] / [ -n … ]`). No new shellcheck-flagged idioms are introduced.

### AC8 — Rust gates (sanity)

No Rust files touched. Run the gates from `AGENTS.md → Build & Test` once at the end:

```bash
cargo build && cargo fmt -- --check && cargo clippy --workspace -- -D warnings && cargo test && RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features
```

All five must exit 0. No Rust regressions expected.

## Open question resolution

### Spec Open Question #1 — "sanity-net stderr warning when `${PR_NUM}` IS empty but a `.progress.md` whose slug matches the branch exists"

**Decision: decline.** Justification:

- The `${PR_NUM}`-empty path already prints `pr-merged: no merged PR found for <branch>; …` to stdout (line 43); the user sees the skip explicitly.
- Slug-overlap detection requires parsing the branch name (`<prefix>/<date>-<slug>`) and globbing `ai-docs/plans/<date>-<slug>.progress.md` — non-trivial bash, with edge cases (no-prefix branches, branches like `master`, irregular slugs).
- The orphan case the question targets is "branch merged outside `gh`" — extremely rare in this workflow (every PR in this repo is merged via `gh pr merge --merge` per AGENTS.md). The cost/benefit ratio is poor.
- The user explicitly flagged it as optional ("Design's call whether to emit a warning … The user flagged it as optional"). YAGNI applies.

If the orphan case is observed in the wild, escalate to a follow-up issue with a real example to design against.

### Spec Open Question #2 — keyword conjugation (`Closed` / `Fixed` / `Resolved`)

**Decision: include all conjugations.** The cost is one `?` and one `(es|ed)?` in the regex (zero readability cost — the regex is already a single line). The benefit is matching GitHub's actual auto-close parser, which is case-insensitive and accepts the past-tense forms. Over-covering is cheap; under-covering risks a future PR using `Closed #N` (technically valid GitHub syntax) tripping the AC3 warning unnecessarily.

The final regex is `(Close[sd]?|Fix(es|ed)?|Resolve[sd]?) #[0-9]+` with `grep -oiE`. See "Regex form & justification" above.

## Docstring rewrite (post-fix shape)

Replace lines 8–30 (the current `# Derivation:` and `# Failure modes:` blocks) with the following:

```text
# Derivation (PR linkage):
#   1. `gh pr list --state merged --head <branch>` -> merged PR number
#   2. `gh pr view <PR_NUM> --json body` -> PR body
#   3. `grep -oiE '(Close[sd]?|Fix(es|ed)?|Resolve[sd]?) #[0-9]+'` on the body
#      -> the first Closes/Fixes/Resolves #N reference. ISSUE_NUM is the
#      trailing digits. (Per /task + /interview convention every spec carries
#      `**Tracked in:** #<ISSUE_NUM>`, not #<PR_NUM>.)
#   4. `grep -lE "^\*\*Tracked in:\*\* #<ISSUE_NUM>\b"` in ai-docs/plans/done/
#      + ai-docs/plans/ -> matching /task spec file. The pattern is anchored
#      (line-start + literal `**Tracked in:** ` field + trailing word-boundary)
#      so prose lines that merely *mention* the substring `Tracked in:` and
#      `#<N>` do not false-match the actual spec convention.
#   5. `basename <spec>.spec.md` -> spec-base name
#   6. Delete ai-docs/plans/<spec-base>.progress.md (if it exists)
#   7. Delete ai-docs/pr-comments/pr-<PR_NUM>.progress.md (the fallback path
#      used by /pr-commented for PRs not produced by /task)
#   8. `rmdir ai-docs/pr-comments` -- opportunistic cleanup; non-fatal if the
#      directory still has unrelated files
#
# Failure modes (all exit 0 -- workflow step 4 proceeds regardless):
# - `PR_NUM` empty (branch merged outside `gh`, or PR is closed-not-merged):
#   prints a one-line note to stdout and exits 0 (nothing reliable to derive
#   paths from).
# - `ISSUE_NUM` empty (PR body has no Closes/Fixes/Resolves #N line — e.g.
#   manual non-/task PR, or `/task` PR whose body convention drifted):
#   prints a one-line warning to STDERR naming the PR and branch, skips the
#   /task-progress-file deletion, proceeds to the /pr-comments fallback path.
# - `SPEC_PATH` empty (issue number derived but no spec carries the literal
#   `**Tracked in:** #<ISSUE_NUM>` field at the start of a line -- e.g. PR
#   closes a satellite issue without its own /task spec): prints a one-line
#   warning to STDERR naming the issue number, skips the /task-progress-file
#   deletion, proceeds to the /pr-comments fallback path.
# - rm -f is silent on missing files (intentional — files may not exist;
#   idempotent re-runs are safe).
# - rmdir failing on non-empty directory is expected and ignored.
#
# Multi-close PRs (rare; e.g. `Closes #289` + `Closes #277` on PR #295):
# the FIRST closing-keyword match wins. By convention the first-listed
# issue is the primary /task tracking issue; satellite closures are
# bundled cleanup items without their own spec.
```

The two-line block immediately after (`# Deferred-task progress files … no merged PR to drive cleanup.`) stays unchanged — still accurate.

## Open questions

None remaining. The two spec-level open questions are resolved above; the answer to spec Open Question #1 ("decline") and Open Question #2 ("include conjugations") are committed in this design.
