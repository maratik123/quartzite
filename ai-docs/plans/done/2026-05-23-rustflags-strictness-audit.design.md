# Design: Audit per-job RUSTFLAGS strictness uniformly across the workspace

**Issue:** #385
**Spec:** `ai-docs/plans/2026-05-23-rustflags-strictness-audit.spec.md`
**Date:** 2026-05-23

## Approach

The spec already verified the headline outcome is favourable: `RUSTFLAGS` is uniform at `-D warnings` across all 12 `setup-rust-toolchain@v1` invocations (action default inherited; no overrides anywhere). The deliverable is therefore (a) one drift-point fix in `coverage.yml`, (b) a mechanical guard script that mirrors `scripts/check-rustdoc-internal-refs.sh`, and (c) wiring the guard into PR-CI in `ci.yml`'s `docs` job.

### Audit table (AC1, AC2)

Live state on master, 2026-05-23:

| Workflow | Job | `setup-rust-toolchain@v1` | RUSTFLAGS | RUSTDOCFLAGS | Per-job `-- -D warnings` |
|---|---|---|---|---|---|
| `ci.yml` | `format` | yes | `-D warnings` (default) | — (no rustdoc) | — (no clippy) |
| `ci.yml` | `build` | yes | `-D warnings` (default) | — (no rustdoc) | — |
| `ci.yml` | `test` | yes | `-D warnings` (default) | — (no rustdoc) | — |
| `ci.yml` | `clippy` | yes | `-D warnings` (default) | — (no rustdoc) | **yes** (`cargo clippy ... -- -D warnings`) |
| `ci.yml` | `gpu-tests` | yes | `-D warnings` (default) | — (no rustdoc) | — |
| `ci.yml` | `docs` | yes | `-D warnings` (default) | `-D warnings -D missing-docs` (step env) | — |
| `ci.yml` | `features` | yes | `-D warnings` (default) | — (no rustdoc) | — |
| `coverage.yml` | `coverage` | yes (nightly) | `-D warnings` (default) | **MISSING** (`cargo llvm-cov --doctests` exercises rustdoc) | — |
| `docs.yml` | `build` | yes | `-D warnings` (default) | `-D warnings -D missing-docs` (step env) | — |
| `miri.yml` | `miri` | yes (nightly-2026-05-01) | `-D warnings` (default) | — (no rustdoc) | — |
| `base_benchmarks.yml` | `benchmark_base_branch` | yes | `-D warnings` (default) | — (no rustdoc; `cargo bench` only) | — |
| `fork_pr_benchmarks_run.yml` | `benchmark_fork_pr_branch` | yes | `-D warnings` (default) | — (no rustdoc; `cargo bench` only) | — |
| `fork_pr_benchmarks_track.yml` | `track_fork_pr_branch` | **no toolchain** | n/a | n/a | n/a |

Confirmation: 12 invocations, all inherit `rustflags: -D warnings`. The single drift point is `coverage.yml`'s `cargo llvm-cov --doctests` step missing `RUSTDOCFLAGS`. Per-job lint flag dimension carries `-- -D warnings` only on clippy (belt-and-braces, by-design — not enforced).

**Artefact placement (AC1).** This table is the AC1 deliverable. Implementer copies the table verbatim into the PR description at PR-open time (`gh pr create --body` payload) and into the spec body before the spec moves to `done/` at Step 12, so the audit survives `done/` archival.

### Decision on AC3 (drift-point resolution)

**Pick (a) — add `env: RUSTDOCFLAGS: "-D warnings -D missing-docs"` to `coverage.yml`'s `cargo llvm-cov --doctests` step.**

Rationale:
- The default leaning in the spec's Key-decisions table is (a) and there is no semantic reason to exempt coverage from rustdoc strictness — doctest compilation behaves identically under llvm-cov as under plain `cargo test --doc`.
- The alternative (b) introduces an exemption entry on day one for no operational benefit, which dilutes the guard script's "fail closed against any divergence" contract.
- (a) keeps the allow-list empty at authoring time. An empty allow-list is the strongest signal that *every* observed override on master is intentional.
- Cost is one line of YAML; no behaviour change is expected because no workspace doctest emits warnings today (`docs.yml`'s identical RUSTDOCFLAGS gate has been green continuously).

### Guard script architecture — `scripts/check-rustflags-uniformity.sh`

Mirrors `scripts/check-rustdoc-internal-refs.sh` precedent: POSIX-compatible bash, repo-root-resolution boilerplate (`cd -- "$REPO_ROOT"`), `set -euo pipefail`, `rg` as the primary search tool with fallback diagnostics, fail-closed semantics on unannounced findings, exit codes `0` (clean) / `1` (gate fails) / `2` (usage / tooling error).

The script asserts three orthogonal invariants. Each check has its own allow-list header so divergent exemption rationales stay separated:

**Invariant A — no `rustflags:` input override on `setup-rust-toolchain@v1`.**
- Pattern: scan `.github/workflows/*.yml` for `rustflags:` lines appearing within a `with:` block of an `actions-rust-lang/setup-rust-toolchain@vN` step.
- Implementation: line-number capture via `rg -n` over all workflow files; per match, walk up ~20 lines to find the enclosing `uses: actions-rust-lang/setup-rust-toolchain@`. If found AND the workflow×job pair is not in `ALLOWLIST_RUSTFLAGS_INPUT`, retain as a finding.
- Authoring-time allow-list: **empty.**

**Invariant B — no `RUSTFLAGS:` env-var override anywhere.**
- Pattern: scan workflow files for any line matching `^\s*RUSTFLAGS:` (job-level `env:`, step-level `env:`, workflow-level `env:`). Setting the env at any level would override the action default.
- Implementation: `rg -n '^\s*RUSTFLAGS:\s'` over `.github/workflows/*.yml`. Per match, derive the enclosing workflow×job pair from line context (job header = the most recent `^\s{2}[a-z][a-z0-9_-]*:$` followed by `name:` / `runs-on:` within the file) and consult `ALLOWLIST_RUSTFLAGS_ENV`.
- Authoring-time allow-list: **empty.**

**Invariant C — every cargo invocation exercising rustdoc carries `RUSTDOCFLAGS=-D warnings -D missing-docs`.**
- Pattern: scan workflow `run:` lines for any of `cargo doc(\s|$)`, `cargo llvm-cov.*--doctests`, `cargo rustdoc(\s|$)`. For each hit, check that the same step carries an `env:` block (i.e. an `env:` key indented one level deeper than the step's `- name:` and containing `RUSTDOCFLAGS:`) whose value matches `"-D warnings -D missing-docs"`.
- Implementation: drive via `yq` if available; if not, use a multi-line `rg --multiline` pattern that finds `run: .*<rustdoc-token>.*` and its sibling `env:` block within the step's indentation level. Workflow file structure is well-formed YAML so the indentation heuristic is reliable. To avoid implementing a YAML parser in bash, the script's chosen primary path:
  1. Use `rg -n --pcre2 -U 'run:[^\n]*(?:cargo doc(\s|$)|cargo llvm-cov[^\n]*--doctests|cargo rustdoc(\s|$))[\s\S]{0,400}?(?=^\s{0,8}- |\Z)' .github/workflows/` (multiline, with greedy bound) to capture each rustdoc-cargo step's full text block.
  2. For each captured block, assert it contains `RUSTDOCFLAGS: "-D warnings -D missing-docs"` (literal substring with the canonical value).
  3. If the block is missing the env, derive workflow×job from line context and consult `ALLOWLIST_RUSTDOCFLAGS_EXEMPT`. Authoring-time allow-list: **empty** (post-AC3 fix).
- The 400-char lookahead bound prevents catastrophic backtracking and is well over the longest existing step body (`docs.yml` build step body is ~120 chars).
- **Step-shape assumption (acknowledged in script header).** The Invariant C pattern assumes single-line `run:` with the step's `env:` block appearing *after* the `run:` key within the step indent — true for every current rustdoc invocation. Future steps using `run: |`-multiline shapes or `env:` placed before `run:` would not be detected by this pattern alone. The script header carries a one-line note documenting the shape constraint so a future contributor knows to widen the pattern (or add an allow-list entry) before introducing a divergent step shape. The fail-closed `ALLOWLIST_RUSTDOCFLAGS_EXEMPT` is the documented escape hatch in the meantime.

**Output / failure reporting.** Each invariant accumulates findings into its own array. After all three checks, the script prints all findings in a unified format `<workflow-file>:<line>: <invariant-name>: <message>` and exits `1` if any finding is retained, else `0`. Mirrors the `printf '  %s\n' "${retained[@]}" >&2` style in `check-rustdoc-internal-refs.sh`.

**Allow-list shape** — header comment block analogous to `check-rustdoc-internal-refs.sh`'s "Known #[cfg(test)]-enclosed false-positive sites" preamble. Three separate header sections (one per invariant), each currently containing the line "(none at authoring time)". Adding an entry requires (a) the workflow file × job pair on one line, (b) a one-line reason explaining why the override is intentional, (c) updating the corresponding bash array (`ALLOWLIST_RUSTFLAGS_INPUT=()`, `ALLOWLIST_RUSTFLAGS_ENV=()`, `ALLOWLIST_RUSTDOCFLAGS_EXEMPT=()`) with the same `<file>:<job>` token.

**Why ripgrep, not yq.** `check-rustdoc-internal-refs.sh` already requires `rg` and `ci.yml`'s `docs` job already installs `ripgrep` via `apt-get install -y libfontconfig1-dev ripgrep` (line 441). Reusing `rg` keeps the new guard's tooling profile identical to the precedent — no new apt package, no environment churn. The script's `rg`-availability check fails with exit code `2` and a clear message, matching the precedent.

### CI wiring (AC6)

Add a `Check RUSTFLAGS uniformity` step to `ci.yml`'s `docs` job *after* the existing `Check rustdoc has no repo-internal references` step (line 446-447). Single line `run: bash scripts/check-rustflags-uniformity.sh`. No env, no checkout — the existing checkout at line 418 provides the script. The `ripgrep` dependency is already installed at line 441 alongside `libfontconfig1-dev` for the sibling check.

### Counter-example validation (AC5, design-phase only)

Three counter-examples, each transient and reverted (not committed):

1. **Invariant A** — temporarily add `rustflags: "-D warnings -W unused"` under `coverage.yml`'s `setup-rust-toolchain@v1` `with:` block. Run `bash scripts/check-rustflags-uniformity.sh`. Expect exit code `1` with finding `coverage.yml: rustflags-input override on setup-rust-toolchain@v1 (job: coverage)`. Revert. Re-run. Expect exit code `0`.
2. **Invariant B** — temporarily add `RUSTFLAGS: "-D warnings"` to `ci.yml`'s top-level `env:` block (line 9-10). Run script. Expect exit code `1` with finding `ci.yml: RUSTFLAGS env-var override`. Revert. Re-run. Expect exit code `0`.
3. **Invariant C** — temporarily remove the `env: RUSTDOCFLAGS:` block from `docs.yml`'s `Build docs` step (line 51-52). Run script. Expect exit code `1` with finding `docs.yml: cargo doc invocation missing RUSTDOCFLAGS (job: build)`. Revert. Re-run. Expect exit code `0`.

The counter-example diffs are never `git add`-ed; the validation is purely observational and recorded in the implementation Step 10 self-review output.

### Open question — toolchain-pin uniformity

The spec's Open question asks whether the guard should also assert pin uniformity (`@v1` everywhere). Decision: **out of scope of this guard**, leave to #384. Evidence the inclusion would be cheap (`grep -n 'setup-rust-toolchain@' .github/workflows/*.yml` already shows all 12 invocations use `@v1`). Reasons to defer:

- Pin uniformity is conceptually orthogonal to strictness uniformity (a `@v2` migration would not weaken `-D warnings` unless the upstream default changes — that's a separate audit).
- #384 already owns "pinning policy" and is the right tracker for any pin-related guard.
- Adding a fourth invariant on day one expands the script's scope without operational benefit — every workflow file currently uses `@v1` (verified live this turn).
- Keeping the guard tightly scoped to the spec's three named dimensions matches the spec's `## Out of scope` line "Migrating the guard to a Rust binary, pre-commit hook, or `.claude/rules/` rule file — bash script with PR-CI wiring … is sufficient": minimum-viable scope.

### Rejected alternatives

- **Rust binary instead of bash.** Rejected per spec `## Out of scope` line. The precedent script is bash; consistency wins over type-safety for a ~150-line YAML scanner.
- **Pre-commit hook integration.** Rejected per spec `## Out of scope` line. PR-CI is the contractually required gate; pre-commit is opt-in and inconsistent.
- **Validate the per-job `-- -D warnings` lint flag mechanically.** Rejected per spec Key-decisions row "Lint-flag dimension contract": env-var parity is the asserted contract, lint flag is documented in the audit table only.
- **Documentation-only audit snapshot** (no guard script). Rejected per spec Key-decisions row "Output artefact": would go stale on next workflow edit.
- **Annotate `coverage.yml` with `# RUSTDOCFLAGS-exempt:` instead of fixing.** Rejected — see "Decision on AC3" above. Leaving the allow-list empty is the stronger invariant signal.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Close AC3 drift: add `env: RUSTDOCFLAGS: "-D warnings -D missing-docs"` to `coverage.yml`'s `Generate coverage` step (single key added under the existing `run:` line; mirrors `ci.yml` docs job pattern). | `.github/workflows/coverage.yml` | — |
| 2 | Author `scripts/check-rustflags-uniformity.sh` with the three-invariant architecture, three separate allow-list arrays (all empty), bash repo-root resolution, `set -euo pipefail`, `rg` dependency check, unified output format, and exit codes `0`/`1`/`2`. Mark executable (`chmod +x`). | `scripts/check-rustflags-uniformity.sh` | 1 |
| 3 | Run AC5 counter-example validation (design-phase, no commit): three transient edits per § *Counter-example validation*, each followed by a script run that observes the expected exit code `1` and the revert that observes exit code `0`. Record the three pass/fail observations in the implementation Step 10 self-review output. **No `git add` / `git commit` for this subtask — observations only; the next commit comes from subtask 4.** | (transient — no committed change) | 2 |
| 4 | Wire the guard into PR-CI: add a `Check RUSTFLAGS uniformity` step in `ci.yml`'s `docs` job immediately after the `Check rustdoc has no repo-internal references` step. Single `run: bash scripts/check-rustflags-uniformity.sh`. | `.github/workflows/ci.yml` | 2 |
| 5 | Verify with `actionlint .github/workflows/coverage.yml .github/workflows/ci.yml` (AC7) and `cargo build` to refresh `Cargo.lock` (no Cargo change expected; this is a workflow-hygiene step before commit). Confirm `bash scripts/check-rustflags-uniformity.sh` exits 0 against the post-AC3 workspace (AC4). | (validation; no edit) | 1, 2, 4 |

## Handoff plan

`M = 5` subtasks. Two groups (3 + 2):

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Required by `.claude/agents/design.md` § Rules → handoff-grouping (c): every group fans out under `/context-reset`, including the first.
- **Group A:** subtasks 1–3 — close AC3 drift, author guard script, design-phase counter-example validation. This group is self-contained: the script lands alongside the drift fix it validates, and the counter-examples prove the script catches all three invariants before any CI wiring depends on it.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — wire CI step into `ci.yml`, then `actionlint` + `cargo build` final verification. Terminal group (2 subtasks; within the 1..=3 range).

## Risks

- **`rg --multiline --pcre2` portability.** The Invariant C scanner uses PCRE2 multiline. Mitigation: GitHub-hosted Ubuntu runners ship a recent `ripgrep` with PCRE2 enabled; `ci.yml`'s `docs` job already calls `apt-get install -y ripgrep`. The script's `command -v rg` check is followed by a PCRE2 capability check (`rg --pcre2 -e 'x' /dev/null >/dev/null 2>&1`); on failure, exit code `2` with a clear message.
- **False-positive on Invariant C due to indentation heuristic.** A `cargo doc` step that legitimately omits `RUSTDOCFLAGS` (none exists today) would be flagged. Mitigation: the `ALLOWLIST_RUSTDOCFLAGS_EXEMPT` header in the script is the documented escape hatch. Empty at authoring time is the desired state.
- **False-negative if a future workflow uses `cargo +nightly doc` or `cargo doc-*` aliases.** Mitigation: the `rg` pattern includes the `cargo doc(\s|$)` word-boundary plus `cargo rustdoc(\s|$)` plus the doctests bound. Aliases / other forms surface in PR review; the precedent script (`check-rustdoc-internal-refs.sh`) has the same heuristic boundary and the project has not regressed.
- **`coverage.yml`'s nightly toolchain warns where stable doesn't.** Adding `RUSTDOCFLAGS=-D warnings -D missing-docs` to the coverage step might surface a nightly-only doctest warning that has been silently passing. Mitigation: **before pushing subtask 1's commit**, run `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo llvm-cov --workspace --lcov --output-path lcov.info --doctests` locally (using the nightly toolchain `coverage.yml` pins); if a warning surfaces, treat it as a `/bugfix` detour from `/task` Step 10 self-review (not a design defect). Likelihood is low — `docs.yml` already runs the same `RUSTDOCFLAGS` gate on `cargo doc --no-deps --workspace --all-features` against stable and is green.
- **Drift introduced post-merge (forgotten allow-list update).** Mitigation: the PR-CI gate (Step 4) catches any new override at PR-time. Master CI on `push:` will also exercise the gate.
- **`actionlint` rejection of the new step shape.** Mitigation: subtask 5 explicitly runs `actionlint` per AGENTS.md AXIOM before commit. The step shape is identical to the existing `Check rustdoc has no repo-internal references` step (`run: bash scripts/...`), so failure is unlikely.

## Test Design

- **Location:** `scripts/check-rustflags-uniformity.sh` is a shell script; testing is per-invariant counter-example execution (no `#[cfg(test)]` mod). Mirrors `check-rustdoc-internal-refs.sh`'s test posture (no Rust-side tests; design-phase validation only).
- **Entry point:** `bash scripts/check-rustflags-uniformity.sh` invoked from repo root.
- **Scenarios:**
  - Happy path: post-AC3 workspace → exit 0.
  - Invariant A fail: transient `rustflags:` override on any `setup-rust-toolchain@v1` step → exit 1, finding emitted.
  - Invariant B fail: transient `RUSTFLAGS:` env at workflow/job/step level → exit 1, finding emitted.
  - Invariant C fail: transient deletion of `env: RUSTDOCFLAGS:` block on any rustdoc cargo step → exit 1, finding emitted.
  - Usage error: invocation with an unknown flag → exit 2, usage message to stderr.
  - Missing `rg` on PATH (synthetic): the `command -v rg` check fails → exit 2, clear error.
- **Fixtures / helpers:** none — counter-examples are direct edits to live workflow files, reverted before commit. No fixture directory needed.
- **PR-CI side test:** the new `Check RUSTFLAGS uniformity` step runs on every PR to master via the existing `pull_request:` trigger in `ci.yml`. The first PR opened after merge exercises the gate against the merged tree; expected to pass.

## Open questions

- None blocking. The single Open question in the spec (toolchain-pin uniformity) is resolved above: **out of scope of this guard, leave to #384.**
