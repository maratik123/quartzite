# Resolve `#[object_impl]` doc-deny debt under `--all-features --all-targets`

**Source:** issue #587
**Date:** 2026-05-30
**Tracked in:** #587

## Scope

1. Resolve the 51 `error: Annotated item ... is missing '///' documentation` hard errors that `quartzite-macros`'s `#[object_impl]` / `#[object]` / `#[extend]` doc-enforcement emits under `cargo clippy --workspace --all-targets --all-features -- -D warnings` (equivalently `cargo build --workspace --all-targets --all-features`). The errors are `compile_error!`s from the macro, not clippy lints. They appear only when `--all-features` enables both `quartzite-macros/undocumented-allow` and `quartzite-macros/undocumented-deny` (deny wins) AND `--all-targets` compiles the test/example/bench fixture targets.
2. Apply the resolution strategy chosen in Key decisions across all 51 sites, spanning these targets (counts verified live on master 2026-05-30):

   | Count | Target |
   |---|---|
   | 23 | `quartzite-macros` tests (`object_impl`, `via_facade`, `object`, `extend`) |
   | 15 | `examples/` (`hello_object`, `combined`, plus `object_tree`, `signals_slots`) |
   | 9 | `tests/signal_to_signal.rs` |
   | 3 | `benches/macro_object.rs` |
   | 1 | `quartzite-style` test (`third_party_paint`) |

   (Counts per-file may shift slightly under design; the authoritative target is "zero remaining doc-deny errors under the full gate.")
3. Extend the Feature matrix Clippy step (`.github/workflows/ci.yml`, job `features`, currently `cargo clippy ${{ matrix.features }} --workspace -- -D warnings`) to add `--all-targets`, so the serde-gated test-module paths #586's gate does not currently lint under `--all-features` become covered.

## Out of scope

- The 35-site `--all-features` clippy-lint cleanup already completed in #586.
- Changing the `undocumented-allow` / `undocumented-deny` feature semantics or the "deny wins when both active" precedence in `quartzite-macros` (the macro's doc-policy mechanism stays as-is).
- Adding `--all-targets` to the non-Feature-matrix Build/Test steps (the top-level `clippy` job already uses `--all-targets` without `--all-features`; this issue only changes the Feature matrix Clippy step).

## Deferred

- (none identified)

## Key decisions

| Question | Decision |
|---|---|
| Repro / exact macro message | `error: Annotated item \`<Type>::<member>\` is missing \`///\` documentation. Opt out via \`#[undocumented(allow)]\` or set the lint level to warn/allow via \`#[object_impl(undocumented = "warn")]\` or feature \`undocumented-allow\`.` |
| Available escape hatches (per the macro message) | (a) `#[undocumented(allow)]` per-item; (b) `#[object_impl(undocumented = "warn")]` / `#[object(undocumented = "...")]` per-annotated-block; (c) feature `undocumented-allow` global — but `--all-features` also turns on `undocumented-deny`, which wins, so (c) alone cannot suppress under the target gate. |
| Resolution strategy across the 51 sites | **Split by target kind.** `examples/` (15 sites): add genuine `///` documentation — examples are user-facing reference material, so they model the documented-by-default norm. Macro-internal fixtures — `quartzite-macros` tests (23), `tests/signal_to_signal.rs` (9), `benches/macro_object.rs` (3), `quartzite-style` test (1) = 36 sites — use opt-out attributes (`#[undocumented(allow)]` per-item or `#[object_impl(undocumented = "warn")]` per-block, design picks per-target), because doc prose on internal fixtures is noise. |
| Examples-vs-fixtures principle | Examples are reference material users read → genuine docs. Tests/benches are internal fixtures where doc prose is noise → opt-out attributes. |
| Same-PR gate extension | Yes — extend the Feature matrix Clippy step to `--all-targets` in the same PR that clears the 51 errors, gating the fix against regression. |
| Pre-publish API stability | Clean breaks allowed; no shims (AGENTS.md § API Stability) — not load-bearing here. |

## Technical constraints

- `actionlint .github/workflows/ci.yml` MUST pass before `git add` of the workflow change (AGENTS.md AXIOM).
- The Feature matrix `features` job installs `libfontconfig1-dev` (PR #588) so `--workspace` clippy can compile the renderer; adding `--all-targets` keeps that dependency necessary.
- Full gate must end green: `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0.
- Existing strict gates remain green: `cargo clippy --workspace --all-targets -- -D warnings` and the doc gate `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`.
- Fixtures live in test / example / bench targets; whatever strategy is chosen must not regress those targets under the default (non-`--all-features`) feature set, where `undocumented-deny` is NOT active.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0 (zero `missing '///' documentation` macro errors). |
| AC2 | `cargo build --workspace --all-targets --all-features` exits 0. |
| AC3 | The Feature matrix Clippy step in `.github/workflows/ci.yml` (job `features`) runs `cargo clippy ${{ matrix.features }} --workspace --all-targets -- -D warnings`. |
| AC4 | `actionlint .github/workflows/ci.yml` passes. |
| AC5 | The pre-existing gates stay green: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the `--all-features` doc gate. |
| AC6 | The 15 `examples/` sites carry genuine `///` documentation (not opt-out attributes); the 36 macro-internal fixture sites (`quartzite-macros` tests, `tests/signal_to_signal.rs`, `benches/macro_object.rs`, `quartzite-style` test) are suppressed via the macro's opt-out attributes. |

## Open questions

- (none) — resolution strategy resolved (round 1 answer): genuine docs on `examples/`, opt-out attributes on macro-internal fixtures. The design phase chooses the per-fixture mechanics: which exact opt-out attribute placement (per-item `#[undocumented(allow)]` vs per-block `#[object_impl(undocumented = "warn")]`, whichever is fewer edits per target) and the docstring wording for each example site.
