# Design: AC# rustdoc leak guard

**Issue:** #559
**Date:** 2026-05-27
**Revision:** round 2 (resolves design-review ITERATE — 1 major, 2 minor)

## Approach

Ship a new stand-alone script `scripts/check-ac-doc-leaks.sh` modelled structurally on `scripts/check-rustdoc-internal-refs.sh`, and wire it into the existing `docs` job of `.github/workflows/ci.yml` immediately after the precedent step. The script scans every `.rs` file under `**/src/**` (excluding `tests/`, `benches/`, `target/`, and `quartzite-test-helpers/src/`), matches lines that begin with a doc-comment marker (`///`, `//!`, or `#[doc = "..."]`) and contain a token of shape `\bAC[0-9]+[a-z]*\b`, drops any hit that the precedent's `#[cfg(test)]`-region heuristic classifies as test-region, then exits `1` if at least one published-surface hit remains.

### Why stand-alone (resolves Open Question 1)

The spec's defensible default is "stand-alone, decided by Design Subagent". Stand-alone wins on four legs:

1. **Self-contained failure message.** A reviewer reading a red CI step sees a single guard's name in the failure line, with a single rule cited in stderr. Sharing a script would force the failure message to describe two unrelated rules (Family A/B internal-refs vs AC# leaks), which dilutes the call-to-action.
2. **Independent evolution.** The precedent's Pattern A / Pattern B regexes already encode three distinct revisions (round-3 `design-system/`, round-5 `\b#[0-9]{1,4}\b`, round-7 contributor-tooling tokens). Adding a third pattern there would couple AC-token churn to internal-refs churn; future spec amendments to either rule would force re-review of the other.
3. **Lower review surface.** A new ~140-line script (estimated below) reviewed in isolation is cheaper than a +60-line diff threaded through the 231-line precedent.
4. **Shared infrastructure cost is near zero.** The two scripts share the brace-depth-tracking `awk` helper and the sibling-file detection regex. Code-duplication risk is mitigated by the *Risks* section's drift entry plus a brace-tracking smoke test in CI (the precedent's own startup smoke check serves the same purpose for its sibling regex). A `lib/` directory of sourced helpers was considered and rejected as YAGNI — there are only two consumers and the duplicated block is ~50 lines of awk plus ~15 lines of bash.

### Rejected alternatives

- **(b) Extend `scripts/check-rustdoc-internal-refs.sh` with an additional pattern** — rejected per legs 1, 2, and 3 above.
- **(c) Share helpers via a `scripts/lib/cfg-test-filter.sh` sourced module** — rejected as YAGNI. A third consumer would be the threshold to introduce the shared lib; the *Risks* drift entry tracks the trigger.
- **New top-level workflow file** — spec already excludes this (Out of scope §2). Confirmed: the `docs` job already runs on both `push: master` and `pull_request: master`, and the AGENTS.md "master-only trigger" axiom is satisfied because we are adding a step to a PR-covered job, not a new master-only workflow.
- **`docs.yml` (Pages-deploy workflow)** — spec already excludes this (Out of scope §3) and the AGENTS.md PR-safety-net rule forbids it.
- **Reuse `scripts/check-inline-markers.sh` shape (PostToolUse hook, non-blocking)** — wrong shape. That script is advisory (always exits 0). The new gate must be blocking (exit 1 on retained hit). Use the `check-rustdoc-internal-refs.sh` shape.

### Token-pattern scope (resolves Open Question 2)

Stay with `AC<digits>[lowercase]*` only. The deferred LLM-based guard in the spec tracks the broader `KD<N>` / `Q<N>` alphabet. Expanding now would conflate two leak classes the issue body keeps separate; per AGENTS.md § API Stability "Clean breaks, no compat shims" — but inverted for pre-emptive scope: ship the narrow check now, expand only when a second leak class actually appears.

### Script structure (mirrors precedent)

The new script copies the precedent's structural skeleton 1:1, substitutes the regex, and discards the precedent's two-pattern split (only one pattern needed):

1. `set -euo pipefail`; argument-parse stub (no `--list-skipped` flag at v1 to keep surface small; can be added in a follow-up if developers want a debugging mode).
2. Resolve `REPO_ROOT` via `BASH_SOURCE`; `cd "$REPO_ROOT"`.
3. `command -v rg` check; exit `2` with stderr message on miss.
4. `PATTERN='^\s*(///|//!|#\[doc\s*=).*\bAC[0-9]+[a-z]*\b'` — single regex covering all three doc-comment shapes.
5. Run `rg --type rust -n --no-heading -e "$PATTERN" -g '!**/tests/**' -g '!**/benches/**' -g '!quartzite-test-helpers/src/**' -g '!target/**'` against the workspace; on empty `hits_raw`, exit `0`.
6. Build the `sibling_attached_files` basename set using the precedent's sibling-file regex (verbatim copy). **Round-2 change:** `known_sibling_site` points at `quartzite-style/src/default_style/mod.rs` (which is where the `#[cfg(test)] #[path = "../default_style_tests.rs"] mod tests;` attach now lives, at lines 392-394 — confirmed via Read), **and** the missing-site branch fails loud with `exit 2` and a stderr message instead of silently skipping the smoke check. The silent-skip is what made the precedent's own smoke check no-op after PR #492's `default_style.rs` → `default_style/mod.rs` relocation; the round-2 fix is propagated to BOTH scripts in this PR (subtask 5 below).
7. Run the precedent's `is_cfg_test_enclosed` awk function (verbatim copy) against each hit's line number; classify as `retained` vs `skipped`.
8. On non-empty `retained`, print one line per hit to stderr, append a one-line pointer ("See ai-docs/doc-convention.md / spec for the rule"), exit `1`. On empty `retained`, exit `0`.

The verbatim awk and sibling-regex copies are the deliberate cost of choosing stand-alone over shared-lib (rejected alternative c). The drift risk is captured in *Risks*.

### Round-2 propagation patch to the precedent script (subtask 5)

Per the Propagation Rule, the new script's fail-loud smoke-check behavior MUST also land in the precedent in the same PR. The precedent's current lines 118–126 are:

```
known_sibling_site=quartzite-style/src/default_style.rs
sibling_attached_files=""
if [[ -f $known_sibling_site ]]; then
    if ! rg --type rust --multiline -o "$sibling_re" "$known_sibling_site" >/dev/null 2>&1; then
        echo "error: shape-2 multiline regex no longer matches the known sibling-attached site $known_sibling_site:267-269" >&2
        echo "       update sibling_re in scripts/check-rustdoc-internal-refs.sh" >&2
        exit 2
    fi
fi
```

Three defects in this block, all addressed in subtask 5:

1. **Stale path** — `default_style.rs` was relocated to `default_style/mod.rs` (PR #492). The `if [[ -f ... ]]` guard silently skips the smoke check, defeating its purpose.
2. **Stale line citation** — the error message says `default_style.rs:267-269`, but the current attach lives at `default_style/mod.rs:392-394`.
3. **Silent-skip on missing path** — the `if [[ -f ... ]]; then ... fi` shape means a future relocation also silently disables the check. Fail-loud (`exit 2`) is the correct posture for a script-startup invariant.

The round-2 fix replaces the block with:

```
known_sibling_site=quartzite-style/src/default_style/mod.rs
if [[ ! -f $known_sibling_site ]]; then
    echo "error: known sibling-attached site $known_sibling_site not found" >&2
    echo "       update known_sibling_site in scripts/check-rustdoc-internal-refs.sh" >&2
    exit 2
fi
if ! rg --type rust --multiline -o "$sibling_re" "$known_sibling_site" >/dev/null 2>&1; then
    echo "error: shape-2 multiline regex no longer matches the known sibling-attached site $known_sibling_site:392-394" >&2
    echo "       update sibling_re in scripts/check-rustdoc-internal-refs.sh" >&2
    exit 2
fi
sibling_attached_files=""
```

The new `scripts/check-ac-doc-leaks.sh` adopts the same fixed shape from the start; the two scripts stay structurally identical after the patch.

The precedent's docblock comment lines 22–28 (the worked-example for sibling-file shape) also cite `default_style.rs:267-269`. Subtask 5 updates that comment to cite `default_style/mod.rs:392-394` for consistency.

### Workflow wiring (one new step in `docs` job)

Insert after `Check rustdoc has no repo-internal references` (ci.yml line 446–447) and before `Check RUSTFLAGS uniformity` (line 448–449):

```yaml
      - name: Check rustdoc has no AC# leaks
        run: bash scripts/check-ac-doc-leaks.sh
```

The `docs` job already installs `ripgrep` (ci.yml line 441), already runs on `push: master` + `pull_request: master` (ci.yml line 4–7), and already has `cargo doc` upstream. No further plumbing. The `changes` job's `rust` filter (ci.yml line 28–33) lists `.github/workflows/**` but does NOT list `scripts/**`; per spec line 50 this is acceptable for a regression gate that is already green — a script-only edit without a workflow edit would not trigger the docs job, but the gate has nothing new to catch in that scenario.

### Verification at script-authoring time

After `chmod +x` on the new script, run `bash scripts/check-ac-doc-leaks.sh` from repo root and verify exit code `0` (AC2). This satisfies the spec's "begin life green" requirement and the spec line 12 baseline assertion. Implementer also manually runs the AC3/AC4/AC5/AC6/AC7 injection-and-revert verifications below as part of self-review. After the subtask-5 patch, run `bash scripts/check-rustdoc-internal-refs.sh` from repo root and verify exit code `0` to confirm the new fail-loud smoke check still passes against the updated `known_sibling_site`.

## Decomposition

`M = 5` (round-2 update: subtask 5 added to propagate the smoke-check fix to the precedent script, per the Propagation Rule and the design-review major finding).

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `scripts/check-ac-doc-leaks.sh` with `#!/usr/bin/env bash`, `set -euo pipefail`, REPO_ROOT resolution, `rg` PATH check (exit 2 on miss), and the file's docblock comment naming the rule, the spec/issue, and listing the v1-scope exclusion of `--list-skipped`. Implement the single combined regex pattern `^\s*(///|//!|#\[doc\s*=).*\bAC[0-9]+[a-z]*\b` and the raw `rg` hit collection scoped to `**/src/**` minus `tests/`, `benches/`, `target/`, `quartzite-test-helpers/src/`. On empty raw hits, exit `0`. `chmod +x` the new file. | `scripts/check-ac-doc-leaks.sh` (new) | — |
| 2 | Port the precedent's `#[cfg(test)]`-region filter into the new script: copy the `sibling_re` regex and the `is_cfg_test_enclosed` awk helper VERBATIM from `scripts/check-rustdoc-internal-refs.sh`. **Round-2 change vs round-1:** the `known_sibling_site` MUST be `quartzite-style/src/default_style/mod.rs` (the current location of the sibling-file attach after PR #492's split), AND the missing-site branch MUST `exit 2` with a stderr message instead of silently skipping. This is the same fail-loud shape subtask 5 applies to the precedent — both scripts MUST be structurally identical here after this PR. Build the basename-set. Wire the per-hit classification loop that splits each raw hit into `retained` vs `skipped` arrays. On non-empty `retained`, print one indented line per hit to stderr, append a one-line stderr pointer to the spec / issue #559, exit `1`. On empty `retained`, exit `0`. | `scripts/check-ac-doc-leaks.sh` | 1 |
| 3 | Baseline-verify the script: run `bash scripts/check-ac-doc-leaks.sh` from repo root and confirm exit `0` (AC2). Then, for the implementer's local self-review only, perform five injection-and-revert smoke tests: (a) add a `/// AC1 leak` line to any non-test file in a workspace `src/`, run the script, confirm exit `1` with the file:line in stderr, revert (AC3); (b) same with a `//!` inner doc-comment (AC4); (c) same with a `#[doc = "AC1 leak"]` attribute (AC5); (d) add a `// AC99 plain comment` line, run, confirm exit `0` (AC6); (e) add a `/// AC1 leak` line INSIDE a `#[cfg(test)] mod tests { ... }` block (e.g. at the top of an existing such block in `quartzite-runtime/src/timer_drivers.rs:425+` or `quartzite-renderer/src/render_harness.rs:438+`), run, confirm exit `0` (AC7). Reverts MUST happen before any `git add`. No code/file change committed from this subtask. | (no committed change; verification only) | 2 |
| 4 | Patch `scripts/check-rustdoc-internal-refs.sh` to apply the same fail-loud smoke-check shape that subtask 2 adopts (Propagation Rule, design-review major finding round 1). Specifically: (i) change `known_sibling_site=quartzite-style/src/default_style.rs` → `known_sibling_site=quartzite-style/src/default_style/mod.rs` (line 118); (ii) convert the `if [[ -f $known_sibling_site ]]; then ... fi` wrapper into an upfront `if [[ ! -f $known_sibling_site ]]; then echo ... ; exit 2; fi` guard so a future relocation surfaces at script-startup time instead of silently skipping; (iii) update the smoke-check stderr message line citation from `default_style.rs:267-269` → `default_style/mod.rs:392-394` (line 122); (iv) update the docblock worked-example comment at lines 22–28 to cite `default_style/mod.rs:392-394` consistently. Run `bash scripts/check-rustdoc-internal-refs.sh` from repo root after the edit and confirm exit `0` — proves the smoke check now actually fires (was silently no-op before) and still passes against the relocated site. | `scripts/check-rustdoc-internal-refs.sh` | — |
| 5 | Add the new step to `.github/workflows/ci.yml` `docs` job: insert `- name: Check rustdoc has no AC# leaks\n        run: bash scripts/check-ac-doc-leaks.sh` between the existing `Check rustdoc has no repo-internal references` (line 446–447) and `Check RUSTFLAGS uniformity` (line 448–449) steps. Run `actionlint .github/workflows/ci.yml` and confirm exit `0` (AC9, AGENTS.md AXIOM). | `.github/workflows/ci.yml` | 2 |

## Handoff plan

`M = 5` → two groups (3 + 2). Non-terminal group A is exactly 3 subtasks; terminal group B is 2 subtasks (within the 1..=3 range). Round-2 update: regrouped from round-1's 3 + 1 to 3 + 2 to accommodate the new subtask 4 (precedent-script patch) while keeping both groups within the 3-cap.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task begins Group A with fresh context.
- **Group A:** subtasks 1–3 — initial implementation chunk (3 subtasks; equals the 3-subtask cap for non-terminal groups).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — terminal group (2 subtasks; within the 1..=3 range). Subtask 4 (precedent-script propagation patch) and subtask 5 (workflow wiring) are both small, independent edits and pair naturally in the terminal group.

## Risks

- **Drift between `scripts/check-rustdoc-internal-refs.sh` and `scripts/check-ac-doc-leaks.sh` cfg(test) helpers.** The new script copies the precedent's `sibling_re`, `is_cfg_test_enclosed` awk function, and the (now-fixed) `known_sibling_site` fail-loud smoke check verbatim. If a future contributor fixes a bug in one script's helper, the other script needs the same fix. Mitigation: (i) header comment of the new script names the precedent as the source-of-truth donor file and points at the specific block to keep in sync; (ii) the `known_sibling_site` smoke check fires at script-startup time **and now correctly fails loud when the known site goes missing** (round-2 fix to BOTH scripts), so a regex regression or a future site-relocation surfaces in CI immediately instead of silently masquerading as "no false positives" — the round-1 design's stated mitigation now actually holds in both scripts; (iii) when a third consumer of the helpers appears, extract them to a sourced `scripts/lib/cfg-test-filter.sh` and switch both scripts to source it (deferred YAGNI line — call it out in `ai-docs/learnings.md` if it recurs).
- **False negative — a non-`mod`-block `#[cfg(test)]` shape.** The precedent's filter handles only two shapes (inline `#[cfg(test)] mod NAME { ... }` and sibling-file `#[cfg(test)] #[path = "NAME.rs"] mod IDENT;`). Exotic shapes (`#[cfg(any(test, feature = "bar"))]`, `cfg_if!`-gated test blocks, doc-comments inside `cfg(test)` non-`mod` blocks) would NOT be filtered. Mitigation: this matches the precedent's known limitation; the spec line 38 already accepts the heuristic. If a real false-positive surfaces, fix it in both scripts in the same PR (the propagation rule for the cfg(test) helpers).
- **False negative — multi-line `#[doc = "..."]`.** The regex `^\s*#\[doc\s*=` matches the first line of an attribute. A multi-line attribute (`#[doc = "foo\nbar AC1\n"]`) where the AC token is on a continuation line would NOT match. Mitigation: in-tree there are no such multi-line `#[doc]` attributes; if any appear in the future, widen the regex (or shell out to `syn`-based AST tooling — deferred to the LLM-guard track in the spec's Deferred section).
- **False positive — a non-doc-comment substring `///AC1` inside a string literal or code line that happens to start with whitespace + `///`.** A line like `let x = "/// AC1";` does NOT match (because `^\s*///` requires no non-whitespace before `///`); a line like `    /// AC1 leak` inside a `#[doc = r#"..."#]` raw-string literal that is NOT itself a doc-comment WOULD match. Mitigation: spec line 9 and AC1–AC8 already accept the regex-based heuristic; no in-tree case exists.
- **Workflow-edit-only PR with no script edit — `changes` job filter ignores `scripts/**`.** Acceptable per spec line 50: a script-only edit without a workflow edit will not trigger the docs job. The gate is regression-protective, not net-new-feature; it has nothing to catch when neither the script nor any `.rs` file changes.
- **`actionlint` AGENTS.md axiom skipped.** AGENTS.md flags this as a recurring violation. Mitigation: subtask 5 explicitly requires `actionlint .github/workflows/ci.yml` exit `0` before `git add`, and `self-review.md` is spawned by `/task` Step 10 before the push that opens the PR. The CI `Actionlint` job is the safety net.
- **`bash` invocation vs `./script` invocation.** The new step uses `bash scripts/check-ac-doc-leaks.sh` (matches the two precedent steps on ci.yml lines 447 and 449). This sidesteps any "file is executable but `core.fileMode = false` on Windows checkout" surprise — though the runner is `ubuntu-latest` and `chmod +x` will land in the commit, so the direct invocation `./scripts/check-ac-doc-leaks.sh` would also work. Using `bash ...` keeps the new step consistent with its two siblings.
- **(Round-2) Precedent-script patch (subtask 4) is a Propagation-Rule-driven scope addition.** The round-1 design left the precedent's stale `known_sibling_site` and silent-skip guard in place — design-review correctly flagged this as a major finding because the round-1 § Risks bullet 1.ii claimed startup-time smoke-check coverage that no longer existed in the precedent. Mitigation: subtask 4 is small (4 narrow line edits in one file, plus a docblock comment update), independent of the new-script work (depends-on: nothing), and verified by re-running the precedent script post-edit. The `/task` Step 10 self-review pass over the combined diff will surface any cross-script inconsistency before push.

## Test Design

The script is a CI gate, not Rust code — there is no `#[cfg(test)] mod tests` block. The "test plan" is the implementer's local injection-and-revert smoke matrix (subtask 3 above), plus the CI run itself once the PR is opened. Per AC1–AC10, the smoke matrix exercises every acceptance criterion:

- **AC1 / AC2 — baseline green.** Run `bash scripts/check-ac-doc-leaks.sh` from repo root after `chmod +x`. Expected: exit `0`, no stderr.
- **AC3 — `///` leak detection.** Inject `/// AC1 leak` at the top of `quartzite-style/src/default_style/mod.rs` (a non-test, non-helper file). Expected: exit `1`, stderr contains `quartzite-style/src/default_style/mod.rs:1:/// AC1 leak`. Revert.
- **AC4 — `//!` leak detection.** Inject `//! AC2 leak` at the top of `quartzite-style/src/lib.rs`. Expected: exit `1`, stderr contains the file:line. Revert.
- **AC5 — `#[doc = "..."]` leak detection.** Inject `#[doc = "AC3 leak"]` immediately above the `Palette` struct definition in `quartzite-style-types/src/palette.rs` (or any public struct). Expected: exit `1`, stderr contains the file:line. Revert.
- **AC6 — `//` line comment is NOT a leak.** Inject `// AC99 plain comment` at the top of `quartzite-style/src/default_style/mod.rs`. Expected: exit `0`. Revert. (Implicit confirmation that existing `// ── AC1: …` / `// AC3: …` / `// AC6b: …` markers remain non-failing — they already do, since AC2 baseline passed.)
- **AC7 — `#[cfg(test)] mod tests { ... }`-enclosed leak is filtered.** Inject `/// AC4 leak` immediately after a known inline-shape `#[cfg(test)] mod tests {` line — verified line numbers (re-Read on 2026-05-27): `quartzite-runtime/src/timer_drivers.rs:425-426` (`#[cfg(test)]` / `mod tests {`) and `quartzite-renderer/src/render_harness.rs:438-439` (`#[cfg(test)]` / `mod tests {`). Inject after line 426 or 439 respectively. Expected: exit `0`. Revert. Sibling-file shape (`quartzite-style/src/default_style_tests.rs`, attached via `quartzite-style/src/default_style/mod.rs:392-394`) is exercised by injecting a `/// AC5 leak` line in that file. Expected: exit `0`. Revert.
- **AC8 — workflow step present.** `git diff .github/workflows/ci.yml` shows a new `Check rustdoc has no AC# leaks` step adjacent to the precedent's step (between lines 446–447 and 448–449 of pre-edit ci.yml).
- **AC9 — `actionlint`.** `actionlint .github/workflows/ci.yml` exits `0` after the workflow edit. Required before `git add` per AGENTS.md AXIOM.
- **AC10 — CI triggers cover PR + push.** Verified by reading ci.yml lines 4–7: `on: push: branches: [master]` and `on: pull_request: branches: [master]` are already declared at the workflow level; the `docs` job inherits both. No new top-level workflow needed.

**(Round-2 added) Precedent-script regression verification for subtask 4.** Run `bash scripts/check-rustdoc-internal-refs.sh` from repo root after the subtask-4 edit. Expected: exit `0` (the smoke check now actually fires against the updated `known_sibling_site` and the regex still matches). To prove the fail-loud branch works as intended, do a transient injection: temporarily rename `quartzite-style/src/default_style/mod.rs` aside (`mv ... mod.rs.bak`), re-run the precedent script, confirm exit `2` with the "known sibling-attached site … not found" stderr message, then restore the file. This injection MUST happen before any `git add`.

Fixtures / helpers: none — the smoke matrix uses live source files with manual revert after each injection. Implementer MUST `git status` after the matrix to confirm a clean tree before staging.

## Open questions

None. Both spec-level open questions are resolved in the *Approach* section:

- **OQ1 (stand-alone vs share helpers vs extend precedent):** stand-alone, per the four-leg rationale (self-contained failure message; independent evolution; lower review surface; near-zero shared-infra cost). A future `scripts/lib/cfg-test-filter.sh` extraction is the trigger when a third consumer appears (drift risk recorded above).
- **OQ2 (AC-only vs broader internal-token alphabet):** AC-only. The deferred LLM guard in the spec's Deferred section is the planned home for `KD<N>` / `Q<N>` / broader alphabets.

**Round-2 design-review feedback disposition (audit trail):**

- **Major #1 (stale `known_sibling_site` + silent-skip masking the smoke check):** RESOLVED. Subtask 2 now uses the current path `quartzite-style/src/default_style/mod.rs` and a fail-loud `exit 2` shape. Subtask 5 (new) propagates the same fix to the precedent script in this PR per the Propagation Rule. § Risks bullet 1.ii is now factually accurate in both scripts. The recommended option (a) from the feedback ("fix both, list as an extra subtask") was chosen — option (b) ("document the divergence") would have left the precedent's silent no-op in place, which is worse.
- **Minor #2 (line-number drift in `render_harness.rs`):** RESOLVED. Subtask 3's bullet and the Test Design AC7 entry now cite the verified line numbers `438-439` (re-Read on 2026-05-27). The previous "after line 440" wording is removed.
- **Minor #3 (subtask 3 / Test Design overlap):** ACCEPTED AS-IS per reviewer's suggestion. Subtask 3 stays as a discrete baseline-verify step because the new 3 + 2 group shape (Handoff plan) keeps both groups within the 3-cap cleanly; folding subtask 3 into subtask 2 would yield 4 subtasks (3 + 1), which is also valid but offers no benefit beyond what the round-1 shape already had. Implementer experience is unchanged.
