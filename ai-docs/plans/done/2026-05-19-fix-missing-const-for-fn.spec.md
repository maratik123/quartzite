# Fix `missing_const_for_fn` (promote 59 eligible fns to `const fn`, remove workspace allow)

**Source:** issue #479
**Date:** 2026-05-19
**Tracked in:** #479

## Scope

1. Audit all 59 `clippy::missing_const_for_fn` sites surfaced by `cargo clippy --workspace --all-targets -- -D warnings -A clippy::all -W clippy::missing_const_for_fn`.
2. Promote every flagged function to `const fn` — uniformly across `src/`, `tests/`, and `examples/` (the round-1 answer was "Promote all"; test/example sites get the same treatment as production code). Clippy's nursery lint only fires on eligible fns, so promotion is the expected default. Per-item `#[allow(clippy::missing_const_for_fn, reason = "…")]` is reserved as a fallback for the rare case where promotion turns out to break a downstream call site or interact poorly with a feature-gated module under the doc gate; each such allow carries a written justification.
3. Remove the workspace-level `missing_const_for_fn = "allow"` line from root `Cargo.toml [workspace.lints.clippy]` (line 45–46 today).
4. All gates green on the final commit: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`.
5. Single PR carrying all 59 dispositions, per the issue body's "follow-up to actually promote".

## Out of scope

- Other workspace allows already audited and resolved in PR #478 (`must_use_candidate`, `redundant_pub_crate`, `return_self_not_must_use`, `significant_drop_in_scrutinee`, `type_complexity`). Only `missing_const_for_fn` is in scope.
- New `const fn` promotions for items the lint does **not** flag (i.e. items where the lint would not warn under `warn` posture). Out-of-scope hunts add risk for no signal.
- Reviewing the project-level `nursery = warn` group setting itself.
- The pedantic/nursery group-level posture decision (already settled).

## Deferred

| What | Why | Separate issue needed? |
|---|---|---|
| Promoting non-flagged fns to `const fn` opportunistically (e.g. fns with branches the lint can't yet handle but that *could* be `const fn` with restructuring) | Scope creep; the issue is explicitly about the 59 already-flagged sites | No — falls under future const-fn work if ever desired |

## Key decisions

| Question | Decision |
|---|---|
| Single PR vs split-per-crate? | Single PR — issue body says "59 eligible functions". Mirrors PR #478's audit-in-one-PR model. |
| Default action for each lint hit? | **Promote to `const fn`**. Clippy's `missing_const_for_fn` is opt-in nursery and only fires on fns it has already verified are eligible (no heap alloc, no trait objects, no runtime-only behaviour). |
| Per-item allow vs per-crate allow when promotion is rejected? | **Per-item `#[allow(clippy::missing_const_for_fn, reason = "…")]`**. AGENTS.md *Code Style — Linter posture* requires justification on every `#[allow]`; per-crate without an inline reason hides the per-site judgement. The workspace allow is removed regardless. |
| MSRV concern? | None. Workspace `rust-version = "1.95"`; clippy's lint only fires when promotion is safe on the current MSRV, so all 59 sites are stably-`const`-callable. |
| Treatment of test/example sites (9 in `tests/*.rs` + 2 in `examples/*.rs`) | **Promote all** (round-1 answer). All 11 test/example hits get the same `const fn` promotion as production sites. Uniform diff shape; no test/example-specific per-item allows. Rationale: keeps the disposition rule simple ("one rule across the workspace") and the lint only fires when promotion is mechanically safe regardless of target kind. |
| Trait-impl methods | Clippy already skips `impl Trait for …` methods unless the trait method itself is `const`. Any trait-method hit in the survey is on an inherent `impl Foo`; promote freely. |
| Public API stability | Adding `const fn` is forward-compatible (callers can still call at runtime). No AGENTS.md *API Stability* concern. |

## Technical constraints

- Workspace `[workspace.lints.clippy] missing_const_for_fn = "allow"` lives at root `Cargo.toml` line 45–46 today. Removal is a single-line delete (plus the preceding comment).
- Each crate already declares `[lints] workspace = true`; per-crate `[lints.clippy]` blocks are incompatible with `[lints] workspace = true` (PR #478 design phase Subtask 4 already documented this — Cargo rejects the combination). All non-promotion dispositions therefore land as per-item `#[allow]` attributes.
- 59 sites across 26 files (survey grouped below). 49 in `src/`, 9 in `tests/`, 2 in `examples/`.
- `cargo clippy --workspace --all-targets -- -D warnings` is the canonical gate (AGENTS.md *Build & Test*); `--all-targets` covers tests + examples + benches.
- Doc gate runs with `--all-features`; any `const fn` promotion on items in feature-gated modules (`serde`, `style`, `widgets`) must be re-checked under the doc gate.
- `cargo build -p quartzite --no-default-features --features libm` must still compile — relevant for any promotion on items in `quartzite-paint-api` (on the `libm` derive-free path).
- Per AGENTS.md *Corrections Log* — Boundary rule 2 exception (in-flow learning capture during `/task` Steps 8–12): if survey reveals a recurring `const fn` promotion pattern worth recording (e.g. a category of fns where promotion changes API ergonomics in a surprising way), capture it as a learning entry in the same turn as the fix commit.

### Round-1 survey data (full `--all-targets`)

Hit counts after flipping `missing_const_for_fn = "allow"` → `"warn"` and running `cargo clippy --workspace --all-targets 2>&1`:

| Crate | File | Hits | Notes |
|---|---|---|---|
| quartzite-core | src/meta.rs | 6 | hand-written `EnumMeta` static helpers (`noop_lookup_entry_by_name`, …) |
| quartzite-macros | tests/object_impl.rs | 5 | `#[slot]` / `#[invokable]` methods on test-fixture widgets |
| quartzite-events | src/keyboard.rs | 5 | inherent `impl` getters |
| quartzite-core | src/object_base.rs | 5 | inherent `impl` getters |
| quartzite-renderer | src/vello_painter.rs | 4 | inherent `impl` accessors |
| quartzite-events | src/mouse.rs | 4 | inherent `impl` getters |
| quartzite-renderer | src/application_builder.rs | 3 | builder methods (`new`, opt-out flags) |
| quartzite-paint-api | src/font.rs | 3 | font-property getters |
| quartzite-widgets | src/layout/grid_layout.rs | 2 | layout accessors |
| quartzite-style | src/default_style.rs | 2 | style helpers |
| quartzite-runtime | src/object_ref.rs | 2 | inherent `impl` getters |
| quartzite-paint-api | src/brush.rs | 2 | brush helpers |
| quartzite-core | tests/object_safety.rs | 2 | `_assert_object_safe_*` compile-time fixtures |
| quartzite-core | src/id.rs | 2 | `ObjectId` getters |
| quartzite-widgets | src/layout/box_layout.rs | 1 | layout helper |
| quartzite-style-types | src/palette.rs | 1 | palette accessor |
| quartzite-style | tests/third_party_paint.rs | 1 | third-party-paint fixture |
| quartzite-style | tests/snapshots.rs | 1 | snapshot-test helper |
| quartzite-renderer | tests/support/mod.rs | 1 | test-support helper |
| quartzite-renderer | src/application.rs | 1 | builder helper |
| quartzite-geometry | src/rect.rs | 1 | inherent helper |
| quartzite-geometry | src/lib.rs | 1 | free fn |
| quartzite-events | src/window.rs | 1 | inherent getter |
| quartzite-core | src/value.rs | 1 | inherent helper |
| examples | hello_object.rs | 1 | `#[slot]` on demo `Counter` |
| examples | combined.rs | 1 | `#[slot]` on demo widget |

**Total: 59 hits across 26 files (49 in `src/`, 9 in `tests/`, 2 in `examples/`).** Survey reproduced via the recipe in the issue body, ack against the workspace `Cargo.toml [workspace.lints.clippy]` allow flipped to `warn`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | All 59 `missing_const_for_fn` hits resolved — each is either promoted to `const fn` or carries a per-item `#[allow(clippy::missing_const_for_fn, reason = "…")]` with a written justification. |
| AC2 | Workspace-level `missing_const_for_fn = "allow"` line and its preceding comment are removed from root `Cargo.toml [workspace.lints.clippy]`. |
| AC3 | `cargo clippy --workspace --all-targets -- -D warnings` exits 0 on the final commit. |
| AC4 | `cargo test --workspace` exits 0 on the final commit. |
| AC5 | `cargo fmt -- --check` exits 0 on the final commit. |
| AC6 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` exits 0 on the final commit. |
| AC7 | `cargo build -p quartzite --no-default-features --features libm` exits 0 on the final commit. |
| AC8 | PR body contains a per-disposition summary (promotion count + per-item-allow count + per-file table) and an AC checklist with all seven items checked. |

## Open questions

(None — all round-1 ambiguities resolved.)
