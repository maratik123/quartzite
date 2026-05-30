# Fix all-features clippy debt across the workspace (quartzite-core + quartzite-runtime)

**Source:** issue #586
**Date:** 2026-05-30
**Tracked in:** #586

## Scope

1. Resolve all 35 clippy errors that fire under `cargo clippy --workspace --all-targets --all-features -- -D warnings` but not under the default-feature CI gate. All are pre-existing debt in serde-gated code; none introduced by a recent PR. The debt spans two crates: **quartzite-core (28 sites)** and **quartzite-runtime (7 sites)**.
2. **quartzite-core (28 sites)** — span three files: `quartzite-core/src/snapshot.rs`, `quartzite-core/src/snapshot/tree.rs`, `quartzite-core/src/value.rs`.
   Lint breakdown (verified on `master`, 2026-05-30):
   | Count | Lint |
   |------:|------|
   | 24 | `unnecessary structure name repetition` (use `Self::` instead of repeating the type name) |
   | 2 | `BTreeMap::default()` is clearer than the current expression |
   | 1 | this could be a `const fn` (`validate_version` in `snapshot/tree.rs:98`) |
   | 1 | single-character string constant used as pattern |
3. **quartzite-runtime (7 sites)** — span two files: `quartzite-runtime/src/snapshot/object.rs`, `quartzite-runtime/src/snapshot/tree.rs`. Verified live on the branch, 2026-05-30:
   | File:Line:Col | Lint | Suggested fix |
   |---|---|---|
   | `snapshot/object.rs:144:22` | `use_self` (structure name repetition) | `Self` |
   | `snapshot/object.rs:314:25` | `default_trait_access` | `BTreeMap::default()` |
   | `snapshot/object.rs:330:25` | `useless_vec` / single-item `into_iter` | `std::iter::once(("count".into(), Value::Bool(true)))` |
   | `snapshot/object.rs:347:22` | `use_self` | `Self` |
   | `snapshot/tree.rs:53:10` | `or_fun_call` (function call inside `ok_or`) | `ok_or_else(\|\| SerializeError::ObjectNotInTree { id: id.raw() })` |
   | `snapshot/tree.rs:244:22` | `use_self` | `Self` |
   | `snapshot/tree.rs:443:33` | `default_trait_access` | `BTreeMap::default()` |
   Two lint classes here are NOT in the quartzite-core set: single-item `into_iter` (`object.rs:330`) and `or_fun_call` / function-call-inside-`ok_or` (`tree.rs:53`). Apply clippy's per-site suggested fix verbatim, same as the quartzite-core sites.
4. Each lint provides the exact mechanical rewrite at its site; apply clippy's suggested fix verbatim unless it conflicts with an AGENTS.md code-style rule.

## Out of scope

- Any behavioural change to serde/snapshot serialization. This is a lint-only cleanup; runtime behaviour and `cargo test --all-features` output must be unchanged.
- Refactoring or restructuring beyond what each lint requires.

## Deferred

- **Macro-doc-deny hard-error class (51 errors) + `--all-targets` extension of the CI clippy step** | Deferred to **#587** (already filed). With all 35 clippy-lint sites clean, `cargo clippy --workspace --all-targets --all-features -- -D warnings` STILL fails — now with 51 PRE-EXISTING `missing '///' documentation` HARD errors emitted by the `#[object_impl]` proc-macro (`compile_error!`, NOT clippy lints) in test/example/bench fixtures across quartzite / quartzite-macros / quartzite-style. `cargo build --workspace --all-targets --all-features` fails identically (confirms not clippy-specific). Root cause: `--all-features` enables BOTH the macro's `undocumented-allow` and `undocumented-deny` features, and `undocumented-deny` wins → undocumented `#[object_impl]` fixtures become hard errors. Resolving this class AND extending the new CI clippy step to `--all-targets` (so the serde-gated `#[cfg(test)]` sites are also linted on the all-features leg) are both deferred to #587. | Separate issue: yes — #587.

## Key decisions

| Question | Decision |
|---|---|
| How to fix each site | Apply clippy's per-site suggested rewrite verbatim: `Self::` for name-repetition, `BTreeMap::default()`, `const fn` for `validate_version`, `char`-pattern for the single-character string-constant pattern, `std::iter::once(...)` for the single-item `into_iter`, `ok_or_else(\|\| ...)` for the function-call-inside-`ok_or`. |
| Why was the quartzite-runtime debt not in the original 28-site count? | `cargo clippy ... -D warnings` aborts compilation at the **first** crate that fails to compile clean. quartzite-core failed first, so clippy never reached quartzite-runtime — its 7 errors were masked in the original `--all-features` reproduction. Cleaning quartzite-core lets the workspace-wide gate proceed and surface them. Both AC1 and the new CI gate (AC6) would otherwise be red on merge, so the fix is in scope. |
| Why does the gate / AC1 omit `--all-targets`? | Cleaning all 35 clippy-lint sites unmasked a SECOND layer: `--all-features` enables both the `#[object_impl]` macro's `undocumented-allow` and `undocumented-deny` features, and the deny wins → 51 undocumented test/example/bench fixtures become `compile_error!` hard errors under `--all-targets`. That deny collision is a distinct, pre-existing class deferred to #587. Both AC1 and the new CI clippy step therefore run WITHOUT `--all-targets` (mirroring the sibling `Build` / `Test` steps, which avoid the collision the same way) so they assert the production serde-gated lib clippy debt is clean and stay green. |
| Is literal `--all-features --all-targets` green in scope? | No. Making `cargo clippy --workspace --all-targets --all-features -- -D warnings` exit 0 requires resolving the 51-error macro-doc-deny class, which is explicitly deferred to #587. This task scopes to the clippy-lint cleanup (35 sites) + a CI gate shaped to stay green. |
| `const fn` conversion of `validate_version` | Apply per clippy suggestion — purely additive (`pub fn` → `pub const fn`); no caller change required. |
| CI gate to prevent regression | Add an `--all-features` clippy step to the existing ubuntu-only `Feature matrix` job (`features:` at `.github/workflows/ci.yml:464`), mirroring the existing `Build` / `Test` steps in that leg. Single-OS coverage, minimal added CI time, reuses the leg already compiling `--all-features`. |
| Clippy on every matrix leg vs `--all-features` only | The new clippy step runs `cargo clippy ${{ matrix.features }} --workspace -- -D warnings` across all four feature legs, mirroring the sibling `Build` / `Test` steps (which also iterate `${{ matrix.features }}`). This guards the serde-gated production lib debt on every leg, including `--all-features`. The step omits `--all-targets` (see the "Why does the gate / AC1 omit `--all-targets`?" row) — the all-features leg would otherwise hit the deferred #587 macro-doc-deny collision and go red. |

## Technical constraints

- After the fix, `cargo clippy --all-features --workspace -- -D warnings` (no `--all-targets`) must pass clean (0 errors). Note: `cargo clippy --workspace --all-targets --all-features -- -D warnings` (WITH `--all-targets`) will STILL fail with 51 macro-doc-deny hard errors — that class is deferred to #587 and is OUT of scope here.
- The default-feature gate `cargo clippy --workspace --all-targets -- -D warnings` must remain clean.
- `cargo test --all-features` and `cargo fmt -- --check` must remain clean.
- No blanket `#[allow(clippy::...)]` to silence a lint — fix the site (AGENTS.md § Code Style: no blanket allow without justification).
- **`BTreeMap::default()` import finding (verified live, 2026-05-30):** the two quartzite-runtime `default_trait_access` sites (`object.rs:314`, `tree.rs:443`) both sit inside `#[cfg(test)] mod tests` blocks where `BTreeMap` is NOT in scope — neither file has a top-level `use std::collections::BTreeMap` (the production `BTreeMap` usages are fully-qualified `std::collections::BTreeMap::new()`, and `tree.rs` imports `HashMap` but not `BTreeMap`). Applying clippy's `BTreeMap::default()` rewrite therefore requires the design subagent to add `use std::collections::BTreeMap;` to each test module (same pattern as the quartzite-core test-module case), OR write the fully-qualified `std::collections::BTreeMap::default()` form. The `std::iter::once` rewrite (`object.rs:330`) needs no import (fully pathed).
- The CI gate lands in the existing `Feature matrix` job (`features:` at `.github/workflows/ci.yml:464`), an ubuntu-only matrix over feature sets (`--no-default-features --features libm` / `std` / `derive`, plus `--all-features`) with `Build` and `Test` steps. A new `Clippy` step (`cargo clippy ${{ matrix.features }} --workspace -- -D warnings` — WITHOUT `--all-targets`, mirroring the sibling `Build` / `Test` steps so the all-features leg avoids the deferred #587 macro-doc-deny collision) is added alongside, mirroring those steps. The existing default-features `clippy` job (line ~147, 3-OS matrix) is left unchanged.
- The `features` job already sets `RUSTC_WRAPPER: sccache`; the added clippy step inherits it. No new toolchain component is needed (the default-features `clippy` job installs `components: clippy`; the `features` job does not — the design subagent must add `components: clippy` to the `features` job's `Install Rust toolchain` step so the new clippy step has the component available).
- `actionlint .github/workflows/ci.yml` MUST pass before staging the modified workflow (AGENTS.md actionlint AXIOM).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | The production serde-gated `--all-features` clippy-LINT debt is clean: `cargo clippy --all-features --workspace -- -D warnings` exits 0 with no errors. (NOTE: deliberately NO `--all-targets` — that combination trips the deferred #587 macro-doc-deny class, which is out of scope here; see Deferred + Key decisions.) |
| AC2 | `cargo clippy --workspace --all-targets -- -D warnings` (default features) still exits 0. |
| AC3 | `cargo test --all-features` passes with unchanged behaviour. |
| AC4 | `cargo fmt -- --check` passes. |
| AC5 | No new `#[allow(clippy::...)]` attributes were added to silence any of the 35 sites (28 in quartzite-core + 7 in quartzite-runtime). |
| AC6 | The `Feature matrix` job (`features:` in `.github/workflows/ci.yml`) gains a clippy step that mirrors the sibling `Build` / `Test` steps: `cargo clippy ${{ matrix.features }} --workspace -- -D warnings` — WITHOUT `--all-targets`. This guards the serde-gated PRODUCTION lib clippy debt across all 4 feature legs and stays green. (Coverage gap: serde-gated `#[cfg(test)]` test-module sites are NOT linted on the all-features leg under this shape — they need `--all-targets`, blocked by the #587 deny collision — tracked in #587.) |
| AC7 | `actionlint .github/workflows/ci.yml` passes after the workflow edit. |

## Open questions

- (none)
