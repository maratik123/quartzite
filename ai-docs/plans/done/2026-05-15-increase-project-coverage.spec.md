# Increase project coverage

**Source:** issue #341
**Date:** 2026-05-15
**Tracked in:** #341

## Scope

1. Identify the top-10 uncovered production-code regions across the workspace from the most recent Codecov report (baseline: PR #339 / commit `bd4c450`, project coverage **90.35%**, 975 missed lines across 86 files). Exclude test helpers and `#[non_exhaustive]` catch-all arms from the ranking.
2. For each identified region, add unit and/or integration tests where the code is deterministically exercisable. Reasonable additions are in scope; achieving 100% line coverage on any individual region is not required.
3. Restructure the two `_ => None` catch-all arms in `quartzite-renderer/src/vello_painter.rs` (`brush_to_peniko` at line 161 and `brush_color` at line 209) so the match is exhaustive over the locally-known `BrushKind` variants (`Solid`, `LinearGradient`, `RadialGradient`, `Custom`) and the `#[non_exhaustive]` upstream-extension case is handled by an explicit typed path — no `#[coverage(off)]` / `#[cfg_attr(coverage_nightly, …)]` attribute introduced. Mechanism details per **Key decisions Q1**.
4. Push the overall project coverage (as reported by `cargo llvm-cov --workspace --doctests` and uploaded to Codecov via `.github/workflows/coverage.yml`) above **93%** as the AC bar for this task. The 95% stretch goal is recorded in `## Open questions`, not in ACs.

## Out of scope

- Tightening the Codecov drop threshold below 1% or flipping `informational: true` to a hard gate — tracked in issue #267 and explicitly deferred there until coverage stabilises.
- Adding per-crate coverage targets to `codecov.yml` or to the workflow — tracked in issue #268.
- Increasing coverage of GPU-only paint paths in `VelloPainter` that require a real Vulkan/Metal/DX12 adapter beyond what the existing snapshot harness already exercises under llvmpipe in the coverage workflow.
- Rewriting `quartzite-runtime`'s event loop or queued-dispatch architecture for testability. Tests added against the existing surface only.
- Changing the coverage tool (`cargo-llvm-cov`), the lcov upload path, or `codecov.yml`'s `target/threshold/informational` knobs.
- Removing the `#[non_exhaustive]` attribute from `quartzite_paint_api::BrushKind`. The restructure handles the upstream-extension case in the renderer; `BrushKind`'s public attribute stays.
- Introducing any new coverage-exclusion attribute (`#[coverage(off)]`, `cfg(coverage_nightly)` plumbing, `// coverage: ignore` comments, `codecov.yml` `ignore:` block) anywhere in the workspace as part of this task.
- Reaching the 95% stretch goal. The task is complete at ≥ 93%; the stretch is a follow-up.

## Deferred

- 95%+ stretch coverage goal | requires closing `quartzite-runtime` concurrency gaps and additional `VelloPainter` paths beyond what is feasible in one task | follow-up issue should be filed by `/task` Step 12 if the AC is met but the stretch is not.
- Per-region tests deemed not feasibly deterministic (e.g. specific event-loop race paths in `quartzite-runtime`) | listed by the design agent in the design doc with one-line "why not in scope" per region | follow-up issue per deferred region as design surfaces them.

## Key decisions

| Question | Decision |
|---|---|
| Q1 — Mechanism for excluding the two `_ => None` catch-all arms (`brush_to_peniko`, `brush_color`) from the coverage metric | **Restructure, no attribute.** Refactor each match to be exhaustive over the four locally-known `BrushKind` variants (`Solid`, `LinearGradient`, `RadialGradient`, `Custom`); funnel the `#[non_exhaustive]` upstream-extension case (any future variant added in `quartzite-paint-api`) through an explicit typed path — e.g. a single private renderer-internal classification (such as a `From<&BrushKind>` for an internal closed enum with an explicit `Unknown` variant, or a dedicated `match` helper colocated with the data) so the exhaustive matches in `brush_to_peniko` / `brush_color` carry no `_` wildcard. The end-state behaviour for an unknown upstream variant is unchanged (both functions return `None`); the difference is that the unreachable-today path is no longer a coverage region in those two functions. The design agent picks the concrete shape (free function vs `impl` method vs `From` impl) and where it lives in `vello_painter.rs`. No `#[coverage(off)]`, no `cfg(coverage_nightly)` plumbing, no `codecov.yml` `ignore:` entry. |
| Q2 — AC bar for "Push overall coverage above 93%" | Codecov-reported project coverage **≥ 93.00%** on the merge commit of the PR closing this issue, measured by the existing `.github/workflows/coverage.yml` (`cargo llvm-cov --workspace --doctests` → lcov → Codecov). Spot-checked locally with the same command before pushing. |
| Q3 — Source of the "top-10 uncovered regions" ranking | Codecov file-level report for `master` at HEAD when the task starts. The design agent records the snapshot (commit SHA + region list) in the design doc so the ranking is reproducible for review. |
| Q4 — Definition of "production-code region" | Code outside `#[cfg(test)]` blocks, `tests/`, `benches/`, `examples/`, `quartzite-widgets/tests/support/`, and `quartzite-style/tests/support/`. Public and private items both count. |
| Q5 — Doctest coverage | Doctests count toward the metric (workflow already passes `--doctests`). Newly added public items must carry the `# Examples` block per AGENTS.md § *Documentation*; those examples contribute to coverage. |

## Technical constraints

- Coverage tool: `cargo-llvm-cov` on nightly with `llvm-tools-preview` (see `.github/workflows/coverage.yml`).
- Workspace members: 13 crates (`quartzite-core`, `quartzite-macros`, `quartzite-runtime`, `quartzite-geometry`, `quartzite-events`, `quartzite-event-types`, `quartzite-paint-api`, `quartzite-paint`, `quartzite-renderer`, `quartzite-style-types`, `quartzite-style`, `quartzite-style-dispatch`, `quartzite-widgets`) plus the root `quartzite` facade.
- All new tests must follow AGENTS.md § *Rust Test Conventions*: unit tests inside `#[cfg(test)] mod tests` in the same file; integration tests under `tests/`; `rstest` for parameterised cases; `pretty_assertions` for diffs; no unjustified `unwrap()`.
- All new public items must carry rustdoc with a `# Examples` block (AGENTS.md § *Documentation*).
- The coverage workflow runs under `xvfb-run` with `WGPU_BACKEND=vulkan` + `llvmpipe` + `LIBGL_ALWAYS_SOFTWARE=1`. New tests must not regress this environment; tests requiring a real (non-llvmpipe) GPU are out of scope.
- `cargo clippy --workspace -- -D warnings`, `cargo fmt -- --check`, and the doc-gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) must pass with the changes.
- The Q1 restructure must compile on stable Rust without any new feature flags or cfgs. `quartzite_paint_api::BrushKind` remains `#[non_exhaustive]`; the renderer continues to import it via the existing `use quartzite_paint_api::{Brush, BrushKind, …};` path.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Codecov-reported project coverage on the merge commit of the PR closing #341 is **≥ 93.00%**, measured by the existing coverage workflow without changes to `codecov.yml`'s `target`/`threshold`/`informational` settings. |
| AC2 | A reproducible snapshot of the top-10 uncovered production-code regions at task start (commit SHA + per-region file/line/missed-count) is captured in the design doc. |
| AC3 | At least 7 of the 10 identified regions receive new tests (unit or integration) that meaningfully exercise previously-uncovered paths; remaining regions either reach AC1 anyway or are listed in the design doc's "deferred regions" section with a one-line rationale per region. |
| AC4 | `quartzite-renderer/src/vello_painter.rs::VelloPainter::brush_to_peniko` and `quartzite-renderer/src/vello_painter.rs::brush_color` no longer contain a `_ => None` catch-all over `BrushKind`. The `#[non_exhaustive]` upstream-extension case is funnelled through a single explicit typed path (per Q1) with a comment naming the upstream source and the rationale. No `#[coverage(off)]` / `cfg(coverage_nightly)` attribute / `codecov.yml` `ignore:` entry is introduced. |
| AC5 | New tests pass under the coverage workflow's environment (`xvfb-run` + llvmpipe), and `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt -- --check`, and the doc gate all pass. |
| AC6 | If the 95% stretch is not reached, a follow-up issue is filed during `/task` Step 12 capturing the remaining gaps (linked to #341 in its body); the deferred row in this spec is closed out by that issue link. |

## Open questions

- **95% stretch goal** — left for a follow-up issue. The known structural blockers (event-loop concurrency in `quartzite-runtime`; real-GPU paint paths in `VelloPainter`) need their own design work and are not addressable in this task without scope creep.
- **Whether `codecov.yml` should grow an `ignore:` block** — explicitly deferred to #267 / #268; not part of this task.
