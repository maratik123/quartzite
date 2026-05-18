# ci: extend clippy check to cover all targets

**Source:** issue #465
**Date:** 2026-05-18
**Tracked in:** #465

## Scope

1. Update the `Clippy` step in `.github/workflows/ci.yml` (`clippy` job) so its `cargo clippy` invocation lints **all compilable targets** (lib + bins + benches + tests + examples), not just the default lib+bins set, so workspace-level lints (`missing_docs = "deny"`, `clippy::*`, etc.) are enforced on bench / integration-test / example code at PR time rather than post-merge.
2. Fix the existing `clippy::let_underscore_lock` violation that the widened scope surfaces in `quartzite-test-helpers/src/lib.rs` (in the `#[cfg(test)] mod tests` block — `let _: MutexGuard<'static, ()> = test_lock();` at line 75). This is in-scope because the widened CI gate cannot be merged green without it; the design agent picks the precise fix idiom (named binding `let _g: …`, explicit `drop(test_lock())`, or moving the compile-time return-type check out of the runtime test).
3. Run `actionlint .github/workflows/ci.yml` before commit (AGENTS.md AXIOM).
4. Verify locally that the updated invocation passes against the current tree after the lint fix in step 2.

## Out of scope

- Touching any other workflow file (`base_benchmarks.yml`, `docs.yml`, `coverage.yml`, `miri.yml`, etc.). The `Docs` job already covers all targets via `--all-features`; the benchmark workflow already builds bench targets.
- Adding new workspace lints or changing `[workspace.lints.*]`.
- Adding `--all-features` to clippy (orthogonal — feature-gated code paths are not currently failing, and the linked issue #465 does not request it).
- Adding new bench / test / example files or refactoring existing ones beyond the one-line `let_underscore_lock` fix called out above (the three bench-file doc-comment fixes already landed in commit `1e2e823`).
- Other clippy-failing patterns surfaced after the `let_underscore_lock` fix unrelated to the widened-target gate. If the widened invocation surfaces additional pre-existing violations after the design agent's first local rerun, they are pulled into scope as a single batch (same rationale: gate can't merge green otherwise); the design agent enumerates them in its design doc.

## Deferred

(none)

## Key decisions

| Question | Decision |
|---|---|
| Workflow file to edit | `.github/workflows/ci.yml`, `clippy` job, `Clippy` step (currently line 187). |
| Target-selection flag | `--all-targets` (user answer, Q1 round 1). Covers benches + integration tests + examples, not just `--benches` (issue body's literal proposal). Catches the same regression class (workspace-lint enforcement) across all compilable target kinds in one flag, with a marginal extra compile cost amortised by sccache. |
| New clippy invocation | `cargo clippy --workspace --all-targets -- -D warnings`. |
| Apply on all OS matrix legs | Yes — keep the existing `ubuntu-latest` / `macos-latest` / `windows-latest` matrix unchanged. Targets compile identically across legs, and any platform-specific lint regressions deserve PR-level signal. |
| Pre-existing `let_underscore_lock` failure surfaced by `--all-targets` | Fixed in the same PR (Scope item 2). Without it, AC3 / AC6 fail. The precise fix idiom is a design-phase choice. |
| Verification before commit | `actionlint .github/workflows/ci.yml` + locally re-run `cargo clippy --workspace --all-targets -- -D warnings` against the post-fix tree (must exit 0). |

## Technical constraints

- Workspace lints in root `Cargo.toml` (`[workspace.lints.rust] missing_docs = "deny"` and the `[workspace.lints.clippy]` set) propagate to **all** target kinds via each crate's `[lints] workspace = true`. Bench / integration-test / example targets therefore need to satisfy the same lints as library code.
- `cargo clippy` without target-selection flags lints only the default target set (lib + bins). It excludes benches, integration tests, and examples. `--all-targets` is equivalent to `--lib --bins --tests --benches --examples` and is the canonical way to make clippy match the broadest compilable surface. The same `Docs` CI job uses `cargo doc --no-deps --workspace --all-features`, which compiles all targets transitively, but rustdoc and clippy enforce different lint sets — neither subsumes the other.
- `actionlint` is the required pre-commit gate for any modified workflow file (AGENTS.md AXIOM).
- The existing clippy step runs on the `clippy` matrix (3 OS legs) and uses sccache; widening to `--all-targets` adds the compile cost of integration tests + examples + benches on cold caches but is amortised on warm caches. Acceptable per user's Q1 answer.
- Current tree (post `1e2e823`) has one pre-existing `clippy::let_underscore_lock` violation in `quartzite-test-helpers/src/lib.rs:75` that surfaces only under `--all-targets` / `--benches` / `--tests`; no other violations were observed in the spec-writer's local rerun. The design agent re-verifies in its first local clippy invocation.
- File-count surface area widened by this change (informational; aids the design agent's risk analysis): 6 example files under `examples/`, ~30 integration-test files under `tests/` and `*/tests/`, 3 bench files under `benches/` and `*/benches/`.

## Acceptance Criteria

| #   | Criterion |
|-----|-----------|
| AC1 | `.github/workflows/ci.yml` `Clippy` step invocation is exactly `cargo clippy --workspace --all-targets -- -D warnings` on every matrix leg. |
| AC2 | `actionlint .github/workflows/ci.yml` exits 0. |
| AC3 | `cargo clippy --workspace --all-targets -- -D warnings` exits 0 locally against the post-fix tree (including the `let_underscore_lock` fix from Scope item 2). |
| AC4 | The PR introducing this change shows the clippy job exercising bench + integration-test + example targets — verified by inspecting the green `Clippy` job log for compilation of at least `benches/macro_object.rs`, `quartzite-core/benches/signal_property.rs`, `quartzite-runtime/benches/object_tree.rs`, one `*/tests/*.rs` integration test per workspace member that has one, and at least one file under `examples/`. |
| AC5 | A hypothetical regression — re-removing the crate-level `//!` doc from any of the three bench files, or removing a `//!` from any integration-test / example file — would fail the `Clippy` job on a PR (verified by manual reasoning from the chosen invocation, not by actually breaking master). |
| AC6 | Master CI remains green after merge (post-merge `Clippy` job exits 0 on the master push trigger). |

## Open questions

(none)
