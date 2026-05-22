# Audit per-job RUSTFLAGS strictness uniformly across the workspace

**Source:** issue #385
**Date:** 2026-05-23
**Tracked in:** #385

## Problem statement

Issue #385 was surfaced by `/triage` from the `ci-docs-workflow.md` deferred-items queue. It was added as a post-merge follow-up to the `2026-05-14-macos-cargo-path-pin` PR (#341), whose Scope item 5 noted:

> today's behaviour was non-uniform — only the `clippy` lint flag enforced `-D warnings`; the swap unifies it

The follow-up trigger was: *"post-merge follow-up if any latent-warning fallout from this PR's swap is non-trivial"*.

**Observed state on master at audit time (2026-05-23):**

- All 7 workflow files (`base_benchmarks.yml`, `ci.yml`, `coverage.yml`, `docs.yml`, `fork_pr_benchmarks_run.yml`, `miri.yml`; plus `fork_pr_benchmarks_track.yml` which installs no toolchain) install Rust via `actions-rust-lang/setup-rust-toolchain@v1`.
- No invocation overrides the action's default `rustflags: -D warnings` (verified: `grep -n "rustflags" .github/workflows/*.yml` returns empty).
- No workflow sets an explicit `RUSTFLAGS` env-var that would override the action default.
- The `clippy` job's `cargo clippy --workspace --all-targets -- -D warnings` carries the `-- -D warnings` lint flag in addition to the env-var; this is belt-and-braces, not divergence.
- `RUSTDOCFLAGS=-D warnings -D missing-docs` is set in `ci.yml` (docs job) and `docs.yml` (publish job) — but **not** in `coverage.yml`'s `cargo llvm-cov --workspace --doctests` invocation, meaning rustdoc-level warnings during doctest compilation in coverage do not deny-warn.
- Master CI has been green continuously since #341 merged (verified: `gh run list --workflow=ci.yml --branch=master --limit=5` — all `success`). The trigger condition *"latent-warning fallout … non-trivial"* **did not fire** in the ~9 days between merge and audit.

The audit therefore exists in a state where the headline outcome is already favourable: RUSTFLAGS strictness is uniform. The work in this spec is (a) extending the audit to RUSTDOCFLAGS + per-job lint-flag dimensions per Round 1 Q1, and (b) producing a mechanical guard so future drift is caught at PR time per Round 1 Q2.

## Audit table

Live state on master, 2026-05-23 (AC1, AC2):

| Workflow | Job | `setup-rust-toolchain@v1` | RUSTFLAGS | RUSTDOCFLAGS | Per-job `-- -D warnings` |
|---|---|---|---|---|---|
| `ci.yml` | `format` | yes | `-D warnings` (default) | — (no rustdoc) | — (no clippy) |
| `ci.yml` | `build` | yes | `-D warnings` (default) | — (no rustdoc) | — |
| `ci.yml` | `test` | yes | `-D warnings` (default) | — (no rustdoc) | — |
| `ci.yml` | `clippy` | yes | `-D warnings` (default) | — (no rustdoc) | **yes** (`cargo clippy ... -- -D warnings`) |
| `ci.yml` | `gpu-tests` | yes | `-D warnings` (default) | — (no rustdoc) | — |
| `ci.yml` | `docs` | yes | `-D warnings` (default) | `-D warnings -D missing-docs` (step env) | — |
| `ci.yml` | `features` | yes | `-D warnings` (default) | — (no rustdoc) | — |
| `coverage.yml` | `coverage` | yes (nightly) | `-D warnings` (default) | `-D warnings -D missing-docs` (step env — post-AC3 fix) | — |
| `docs.yml` | `build` | yes | `-D warnings` (default) | `-D warnings -D missing-docs` (step env) | — |
| `miri.yml` | `miri` | yes (nightly-2026-05-01) | `-D warnings` (default) | — (no rustdoc) | — |
| `base_benchmarks.yml` | `benchmark_base_branch` | yes | `-D warnings` (default) | — (no rustdoc; `cargo bench` only) | — |
| `fork_pr_benchmarks_run.yml` | `benchmark_fork_pr_branch` | yes | `-D warnings` (default) | — (no rustdoc; `cargo bench` only) | — |
| `fork_pr_benchmarks_track.yml` | `track_fork_pr_branch` | **no toolchain** | n/a | n/a | n/a |

**Conclusion:** 12 `setup-rust-toolchain@v1` invocations across 6 workflow files, all inheriting the action default `rustflags: -D warnings`. RUSTFLAGS uniformity confirmed (AC2). Post-AC3 fix, RUSTDOCFLAGS is uniformly `-D warnings -D missing-docs` at every cargo invocation that exercises rustdoc. Per-job `-- -D warnings` lint flag dimension carries the flag only on clippy (belt-and-braces, documented but not mechanically enforced — see Key decisions).

## Scope

1. Enumerate every `actions-rust-lang/setup-rust-toolchain@v1` invocation and record observed effective `RUSTFLAGS`, `RUSTDOCFLAGS`, and per-cargo-invocation `-- -D warnings` lint flag at each call site, producing a one-table summary as part of the PR description / spec body.
2. Confirm uniformity (or document each drift point) across all three dimensions:
   - **`RUSTFLAGS`** — every `setup-rust-toolchain@v1` invocation inherits the action default `-D warnings` (no `rustflags:` override and no env-var override).
   - **`RUSTDOCFLAGS`** — every cargo invocation that exercises rustdoc (`cargo doc`, `cargo llvm-cov --doctests`, anything that compiles doctests) runs with `RUSTDOCFLAGS=-D warnings -D missing-docs`.
   - **Per-job lint flag (`-- -D warnings`)** — relationship to env-var: env-var parity is the contract; per-cargo-invocation `-- -D warnings` is belt-and-braces (allowed where present, not required where absent).
3. Close the one observed `RUSTDOCFLAGS` drift point: `coverage.yml`'s `cargo llvm-cov --workspace --doctests` step must either gain `env: RUSTDOCFLAGS: "-D warnings -D missing-docs"`, or carry a one-line `# RUSTDOCFLAGS-exempt: <reason>` comment documenting intentional divergence. Design picks one.
4. Add a new `scripts/check-rustflags-uniformity.sh` (mirroring the `scripts/check-rustdoc-internal-refs.sh` precedent) that mechanically asserts:
   - No `.github/workflows/*.yml` invocation of `actions-rust-lang/setup-rust-toolchain@v1` carries a `rustflags:` input unless that workflow file × job pair appears in the script's allow-list.
   - No `.github/workflows/*.yml` block sets a `RUSTFLAGS` env-var that would override the action default unless the workflow file × job pair appears in the allow-list.
   - Every cargo invocation that exercises rustdoc (`cargo doc`, `cargo llvm-cov ... --doctests`) is preceded by an `env: RUSTDOCFLAGS: "-D warnings -D missing-docs"` block, OR the workflow file × job pair appears in the script's `RUSTDOCFLAGS-exempt` allow-list with a documented reason.
   - At authoring time the allow-list is empty (today's state). The script must fail closed if it finds an unannounced override.
5. Wire the guard into PR CI by adding a `Check RUSTFLAGS uniformity` step in `ci.yml`'s `docs` job (same job that hosts `Check rustdoc has no repo-internal references` — keeps docs-adjacent meta-checks together; runs on every PR via the existing `pull_request:` trigger).
6. Verify each modified workflow with `actionlint <file>` before commit (AGENTS.md AXIOM gate).

## Out of scope

- Switching toolchain action / cache action — orthogonal; the swap to `setup-rust-toolchain@v1` is the upstream cause of today's uniformity.
- Adding `-D <new-lint>` flags beyond the `-D warnings` baseline — that's a separate code-quality discussion, not a strictness-uniformity audit.
- Changing branch-protection / required-status-checks.
- Substantive code changes to fix latent warnings — none surfaced; nothing to fix in production code.
- Removing the `clippy` job's explicit `-- -D warnings` lint flag — it remains belt-and-braces; the guard does not require either parity direction for the lint flag.
- Migrating the guard to a Rust binary, pre-commit hook, or `.claude/rules/` rule file — bash script with PR-CI wiring matches the precedent (`check-rustdoc-internal-refs.sh`) and is sufficient.

## Deferred

- Promoting the guard to a workspace-wide instruction-file size-check sibling (e.g. running it alongside `scripts/check-instruction-file-sizes.sh` once that lands per #383) | parallel CI-gate tooling, separate concern | track if/when both gates exist | separate issue not needed at this time.

## Key decisions

| Question | Decision |
|---|---|
| Trigger-condition assessment | The deferred row's trigger ("if latent-warning fallout from this PR's swap is non-trivial") **did not fire**: master has been green since #341 merged; no `RUSTFLAGS`-related fix commits land in `git log --since="2026-05-14"`. The audit therefore confirms a favourable status quo AND installs a forward-looking guard rather than driving a remediation. |
| Live state verified | `grep -n "rustflags" .github/workflows/*.yml` → empty (no overrides). `grep -n "RUSTFLAGS\|RUSTDOCFLAGS" .github/workflows/*.yml` → only the two `RUSTDOCFLAGS` sites in `ci.yml` (docs job) and `docs.yml`. All `setup-rust-toolchain@v1` invocations inherit `rustflags: -D warnings` action default. |
| Audit scope dimension(s) (Round 1 Q1) | **All three: RUSTFLAGS + RUSTDOCFLAGS + per-job `-- -D warnings` lint flag.** Broadest scope chosen so the audit covers every place a future workflow edit could silently weaken strictness. |
| Output artefact (Round 1 Q2) | **Mechanical guard: new `scripts/check-rustflags-uniformity.sh` + PR-CI step that fails on unannounced overrides.** Documentation-only snapshot rejected (would go stale on next workflow edit). The audit table (Scope #1) still gets produced — as the *output* the guard validates today — and lives in the PR description / spec body. |
| Lint-flag dimension contract | Env-var (`RUSTFLAGS`) parity is the asserted contract. Per-cargo `-- -D warnings` lint flag is permitted as belt-and-braces but not required for parity (clippy job retains it). The guard checks env-var/setup-toolchain-input parity only; lint-flag dimension is documented in the audit table without being mechanically enforced. |
| Guard host CI job | `ci.yml` `docs` job, next to the existing `Check rustdoc has no repo-internal references` step — same precedent author, same PR-CI exposure, no new workflow file needed. |
| Drift-point resolution (`coverage.yml`) | Design picks one of (a) add `env: RUSTDOCFLAGS: "-D warnings -D missing-docs"` to the `cargo llvm-cov --doctests` step, or (b) annotate with `# RUSTDOCFLAGS-exempt: <reason>` and add the workflow×job pair to the script's allow-list. Default leaning: (a) — closes the only observed drift cheaply. |
| Allow-list shape | Header comment in the script naming approved overrides (workflow file × job, with one-line reason). At authoring time the allow-list is empty. Same shape as the in-script "Known false-positive sites" header in `check-rustdoc-internal-refs.sh`. |

## Technical constraints

- `actionlint` MUST pass on every modified workflow file (AGENTS.md AXIOM).
- The audit MUST not regress passing jobs on any OS.
- The action's `rustflags:` input default is `-D warnings` (verified 2026-05-14 in `2026-05-14-macos-cargo-path-pin.spec.md` Technical constraints; still current).
- No `setup-rust-toolchain@v1` invocation in any of the 7 workflow files overrides `rustflags:` today (verified 2026-05-23 via `grep -n "rustflags" .github/workflows/*.yml`).
- The guard script must be POSIX-compatible bash and runnable as `bash scripts/check-rustflags-uniformity.sh` (mirrors `bash scripts/check-rustdoc-internal-refs.sh` invocation in `ci.yml`).
- The guard script MUST fail closed (non-zero exit on any unannounced override). At authoring time it MUST exit 0 against the current workspace.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | The PR / spec body contains a one-table audit summary listing every workflow file × job × observed effective RUSTFLAGS × RUSTDOCFLAGS × per-job `-- -D warnings` lint flag. |
| AC2 | The table confirms RUSTFLAGS uniformity at `-D warnings` across all `setup-rust-toolchain@v1` invocations (no overrides; action default inherited). |
| AC3 | The `coverage.yml` drift point (missing `RUSTDOCFLAGS=-D warnings -D missing-docs` on the `cargo llvm-cov --doctests` step) is closed: either (a) by adding the env, or (b) by adding a one-line `# RUSTDOCFLAGS-exempt: <reason>` comment AND adding the workflow×job pair to the guard script's allow-list. |
| AC4 | `scripts/check-rustflags-uniformity.sh` exists, is executable as `bash scripts/check-rustflags-uniformity.sh` from the repo root, and exits 0 against the current workspace (post-AC3 fix). |
| AC5 | The guard script fails non-zero against at least one synthetic counter-example exercised in the design phase (e.g. a temporary `rustflags:` override added to a workflow file → script exits non-zero; revert → script exits 0). The counter-example need not be committed; it is design-phase validation evidence. |
| AC6 | `ci.yml`'s `docs` job carries a new `Check RUSTFLAGS uniformity` step that runs `bash scripts/check-rustflags-uniformity.sh` after the existing `Check rustdoc has no repo-internal references` step. |
| AC7 | `actionlint` exits 0 against every modified workflow file. |
| AC8 | Master CI on the merge commit remains green (`Format`, `Build`, `Test`, `Clippy`, `GPU tests`, `Docs`, `Feature matrix` — plus the appropriate `*-pass` aggregators). |

## Open questions

- Drift-point resolution shape (close vs. annotate) for `coverage.yml`'s `cargo llvm-cov --doctests`. → Design picks; default leaning is to close (add the env block).
- Whether the guard script should also assert that the toolchain action pin (`@v1` today) is uniform across files — adjacent concern but not part of the strictness-uniformity contract; design may include if cheap, otherwise leave for #384 (which already tracks pinning policy).
