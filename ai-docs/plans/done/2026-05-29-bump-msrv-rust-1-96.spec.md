# Bump MSRV to Rust 1.96.0

**Source:** user description — "upver rust to 1.96.0"
**Date:** 2026-05-29
**Tracked in:** #584

## Scope

1. Bump the workspace MSRV declaration from `1.95` to `1.96` at `Cargo.toml:24` (`[workspace.package] rust-version`). All member crates inherit via `rust-version.workspace = true`, so this single line is the only `Cargo.toml` change.
2. Update the human-readable MSRV reference in `README.md:82` ("Rust stable (≥ 1.95)" → "Rust stable (≥ 1.96)").
3. Update the MSRV reference in `design-system/README.md:87` — the numerical-precision style example cites `1.95` "(the MSRV)"; bump to `1.96`.
4. Adopt std's now-stable (1.96) `assert_matches!` in the test suite: convert existing `assert!(matches!(value, Pattern))` idioms in **test code** to `assert_matches!(value, Pattern)`. The macros are stabilized as crate-root re-exports, so the import is `use core::assert_matches;` for `#![no_std]` crates (e.g. quartzite-paint-api) and `use std::assert_matches;` for std crates / integration tests in `tests/` — importing the macro by name, then calling it bare as `assert_matches!(value, Pattern)`. Do NOT add an external `assert_matches` crate. Verified findings below ground this conversion in real call sites.
5. Bump the pinned nightly toolchain in `.github/workflows/miri.yml` from `nightly-2026-05-01` to `nightly-2026-05-29` (the latest nightly that ships the `miri` component). This is the ONLY pinned nightly date in the workflows (`miri.yml:53`); update every in-file literal mention of `nightly-2026-05-01`, including explanatory comments (e.g. the `+nightly` override comment at `miri.yml:62`). Leave `coverage.yml` untouched — it uses the unpinned `nightly` alias (always latest), which needs no change.
6. **Cheap-only `Debug` derives + carve-outs (governing policy, user reversal 2026-05-29).** The prior amendment's full `Debug` cascade onto the `AsWidget` public trait is REVERSED. Governing policy: add `#[derive(Debug)]` ONLY where it is CHEAP — a local `#[derive(Debug)]` on a single (usually private) concrete type with no ripple. Adding a `Debug` SUPERTRAIT to a public trait is EXPENSIVE (it forces `Debug` on every implementor) and is REJECTED for both `AsWidget` (widget surface) and `Object` (runtime). Where converting an `assert!(matches!(...))` site would require an expensive supertrait, the site is CARVED OUT (it stays `assert!(matches!(...))`). Concretely:
   - The macro-generated `AsWidget` trait is UNCHANGED — no `Debug` supertrait. `quartzite-macros` codegen is reverted to its pre-cascade state (`pub trait AsWidget` carries no `core::fmt::Debug` bound). `WidgetView<'a>` does NOT derive `Debug` (its `Other(&'a dyn AsWidget)` variant would demand the expensive supertrait).
   - As a harmless, additive, non-breaking convenience, `#[derive(Debug)]` MAY be added to the individual concrete widget structs (`Button`, `Label`, `TextEdit`, `ScrollArea`, `Container`, `LineEdit`, `WidgetBase`, `ThirdPartyWidget`, `ClippingWidget`). These are user-confirmed cheap local derives. They are NOT required to make any conversion compile — the `WidgetView` sites are carved out regardless of whether these structs derive `Debug`.
   - **Renderer (cheap, in scope):** add `#[derive(Debug)]` to the crate-PRIVATE enum `LocalBrushKind` (`quartzite-renderer/src/vello_painter.rs:156`). It is a private type with no ripple, so the derive is cheap. With it in place, convert the renderer test-code sites (`render_harness.rs`, `event_convert.rs`, `vello_painter.rs`) to `assert_matches!`.
   - This replaces the rejected supertrait approach with local concrete derives. It is additive and non-breaking. Permitted under AGENTS.md § API Stability (pre-publish, no downstream clients), but note nothing breaking is introduced here.
7. **Workspace-wide cheap `Debug` sweep (broadening, user 2026-05-29).** Add `#[derive(Debug)]` to every PUBLIC struct/enum in the workspace that can derive it CHEAPLY. CHEAP = a plain `#[derive(Debug)]` compiles with no further change — every field type / variant-payload type already implements `Debug`, AND no public-trait `Debug` supertrait must be added to make it compile. The EMPHASIS is public `struct`/`enum`s; cheaply deriving `Debug` on private / test-only types where it falls out is fine but secondary. The **design phase MUST enumerate the concrete per-crate set** of public types that gain `#[derive(Debug)]` — this enumeration is the real scope of this item and may be SIZABLE. Excluded (NOT cheap, stay out): any type that already derives/impls `Debug` (no-op); any type with a field whose type is not `Debug` and cannot be made `Debug` without an expensive change; anything that would require a `Debug` supertrait on a public trait (`AsWidget`, `Object` stay supertrait-free per the governing policy in Scope item 6). A public type whose only non-`Debug` field could ITSELF be cheaply fixed is a per-type judgment call the design phase makes. The widget-struct derives (item 6 / AC15) and the `LocalBrushKind` derive (item 6 / AC13) are SPECIFIC INSTANCES of this general rule — keep them; they are subsumed by this sweep. This is additive and non-breaking (adding a `Debug` impl never breaks downstream) and permitted under AGENTS.md § API Stability.
8. Run `cargo build` so `Cargo.lock` refreshes, and re-verify all gates pass under 1.96:
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo fmt -- --check`
   - `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`
   - `cargo build -p quartzite --no-default-features --features libm` (no_std / derive-free path)
   - `cargo test` (workspace)

### Verified `assert_matches!` conversion candidates (grepped 2026-05-29)

Std `assert_matches!` is currently **unused** in the tree (zero hits for `assert_matches`). The conversion targets are existing `assert!(matches!(...))` calls in **test code** (`#[cfg(test)]` modules and `tests/`). Confirmed sites (non-exhaustive line refs; design phase owns the exact enumeration):

**Convert (in scope):**
- `quartzite-paint-api/src/path.rs` (lines 261–263, 282, 286)
- `quartzite-style-dispatch/src/dispatch.rs` (lines 514–518, 742)
- `quartzite-paint-api/src/brush.rs` (lines 305, 312)
- `tests/signal_to_signal.rs` (lines 97, 115)
- `quartzite-macros/tests/meta_enum.rs`
- `quartzite-style/src/default_style_tests.rs`
- `quartzite-core/src/value.rs` (lines 718–719, 1084)
- `quartzite-renderer/src/{render_harness,vello_painter,event_convert}.rs` (unblocked by the cheap `#[derive(Debug)]` on the crate-private `LocalBrushKind` — Scope item 6)

**Carve out (remain `assert!(matches!(...))`; known-allowed, excluded from the residual defect-grep):**
- The **7 `WidgetView` sites** in quartzite-widgets — `widget_base.rs` + `widgets/{button,label,text_edit,scroll_area,container,line_edit}.rs` (1 each). Reason: `WidgetView` cannot be `Debug` without the expensive `AsWidget` supertrait (its `Other(&dyn AsWidget)` variant), and that supertrait is rejected per policy.
- The **runtime snapshot sites** in quartzite-runtime matching `Box<dyn Object>` or `ObjectTree` — `snapshot/tree.rs`, `snapshot/object.rs:316`, `snapshot/object.rs:332`, `tests/snapshot.rs`. Reason: these need the expensive `Object` supertrait, also rejected. These sites are `#[cfg(feature = "serde")]`-gated. NOTE: one runtime site (`snapshot/object.rs:421`, scrutinee `ObjectSnapshot`, which IS `Debug`) MAY still be converted — the design phase decides.

There are no `assert!(!matches!(...))` or `debug_assert!(matches!(...))` idioms in the tree, so the conversion is `assert!(matches!(...))` → `assert_matches!(...)` only (modulo carve-outs). The `assert!(matches!(...) if guard)` forms (e.g. `dispatch.rs:516`, `value.rs:1084`) translate to `assert_matches!(value, Pattern if guard)`, which std `assert_matches!` supports. A residual `rg 'assert!\(matches!'` defect-grep at implementation time MUST treat the carve-out sites above as known-allowed (it should return only those sites + any intentional doc-examples).

## Out of scope

- **Doc-example (`///` / `//!`) `assert!(matches!(...))` occurrences** in `quartzite-paint-api/src/{image,path,brush}.rs` and `quartzite-core/src/snapshot/tree.rs`. These are public rustdoc examples, not test code; converting them would require an `# use std::assert_matches::assert_matches;` preamble in each example and changes user-facing docs. Left as-is; the design phase MAY convert them if it judges the consistency worthwhile, but they are not required by the ACs.
- CI **stable**-channel workflow changes. `.github/workflows/ci.yml`, `docs.yml`, etc. use `actions-rust-lang/setup-rust-toolchain@v1` with the default `stable` channel (no explicit version pin), so CI already tracks current stable and needs no edit. `coverage.yml` uses the unpinned `nightly` alias (always latest) — also unaffected by an MSRV bump, no edit. (NOTE: `miri.yml`'s pinned-nightly bump is NOW an explicit in-scope item per user decision — see Scope item 5 — so it is no longer out of scope.)
- Adding a `rust-toolchain.toml` / `rust-toolchain` file. None exists today; introducing one is a separate decision.
- Refactoring any code to exploit other 1.96-new language/library features beyond the `assert_matches!` adoption above.

## Deferred

- Converting doc-example `assert!(matches!(...))` to `assert_matches!` | user-facing-doc churn requiring per-example `use` preamble; low value relative to risk | no separate issue needed (revisit only if a doc-style pass touches those files).

## Key decisions

| Question | Decision |
|---|---|
| Where is MSRV declared? | Single source of truth at `Cargo.toml:24` `[workspace.package]`; all member crates inherit via `rust-version.workspace = true`. Bump only the one line. |
| CI toolchain pin? | No change — CI uses default `stable` channel, no version pin. |
| `design-system/README.md:87` MSRV ref? | In scope — bump `1.95` → `1.96` (user-confirmed round 1). |
| Adopt std `assert_matches!`? | Yes (user-confirmed round 1) — convert `assert!(matches!(...))` in **test code** only; doc-examples out of scope; no external crate. |
| `assert_matches!` import path? | Crate-root macro re-export: `use core::assert_matches;` (no_std crates) / `use std::assert_matches;` (std crates, `tests/`). WHY: the nightly **module** path `std::assert_matches::assert_matches` is unstable/absent on stable and fails E0432; only the crate-root `#[stable(since = "1.96.0")]` macro re-export resolves. Empirically verified on `rustc 1.96.0`. |
| `signal.rs` "clippy 1.95" comments (lines 1024/1074/1130/1180) | Code comments describing a specific lint's behavior, NOT MSRV declarations. Leave as-is unless the lint behavior actually changed under 1.96 (verify during gate run; if an `#[allow]` is now unnecessary under 1.96, removing it is in scope as clean-up). |
| Which nightly date to pin in `miri.yml`? | `nightly-2026-05-29` — the latest nightly that ships the `miri` component per rustup component history (latest miri date = 2026-05-29; `rust-src` ships on every nightly). The project pins a specific date precisely to guarantee miri availability, so the chosen date MUST be one with miri. Bump `nightly-2026-05-01` → `nightly-2026-05-29`. |
| Governing `Debug`-derive policy? | **`Debug` derives only where CHEAP.** A local `#[derive(Debug)]` on a single (usually private) concrete type with no ripple is CHEAP — KEEP/add these (e.g. `LocalBrushKind`, the concrete widget structs). Adding a `Debug` SUPERTRAIT to a public trait is EXPENSIVE (forces `Debug` on every implementor) and is REJECTED — specifically for `AsWidget` (widget cascade) and `Object` (runtime). Where a conversion site would require an expensive supertrait, CARVE IT OUT of AC4 (it stays `assert!(matches!(...))`). **This supersedes the prior "full `Debug` cascade across the widget API" decision (2026-05-29), which is now REVERSED — no `AsWidget`/`Object` supertrait is added.** |
| How are the 7 `WidgetView` sites handled? | CARVED OUT — they stay `assert!(matches!(WidgetView::…))`. `WidgetView` cannot be `Debug` without the rejected expensive `AsWidget` supertrait (its `Other(&dyn AsWidget)` variant). (Reversal of the prior round's cascade-to-convert decision.) |
| Do the concrete widget structs derive `Debug`? | Yes — additive, harmless, non-breaking cheap local derives on `Button`, `Label`, `TextEdit`, `ScrollArea`, `Container`, `LineEdit`, `WidgetBase`, `ThirdPartyWidget`, `ClippingWidget`. NOT required to make any conversion compile (the `WidgetView` sites are carved out regardless). `WidgetView` itself does NOT derive `Debug`. |
| Renderer sites? | Convert (`render_harness.rs`, `event_convert.rs`, `vello_painter.rs`) after adding a cheap `#[derive(Debug)]` to the crate-private enum `LocalBrushKind` (`vello_painter.rs:156`). Private type → cheap → in scope. |
| Workspace-wide cheap `Debug` sweep? | **Yes (user broadening, 2026-05-29).** Every PUBLIC struct/enum that can CHEAPLY derive `Debug` (all field/payload types already `Debug`; no public-trait supertrait needed) SHOULD get `#[derive(Debug)]`. This GENERALIZES the earlier per-type cheap derives — the concrete widget structs (AC15) and `LocalBrushKind` (AC13) are now specific instances of this single rule, subsumed by the sweep. Additive, non-breaking, permitted under AGENTS.md § API Stability. The **design phase owns the concrete per-crate enumeration** of public types gaining `Debug` (potentially sizable); the cheap-only rule and the supertrait exclusion (`AsWidget`/`Object` stay supertrait-free) bound it. |

## Technical constraints

- The workspace already compiles, lints, docs, and tests clean under the locally-installed `rustc 1.96.0 (ac68faa20 2026-05-25)` per the orchestrator's pre-checks. This is primarily a declaration + prose bump plus a mechanical test-macro migration, not a feature-code migration.
- Std's `assert_matches!` / `debug_assert_matches!` are `#[stable(feature = "assert_matches", since = "1.96.0")]` **crate-root re-exports** (libcore `lib.rs` re-exports them from `crate::macros`). The import is therefore `use core::assert_matches;` (for `#![no_std]` crates such as quartzite-paint-api) or `use std::assert_matches;` (for std crates / `tests/` integration tests), importing the macro by name and calling it bare. Each test module performing conversions needs the import. No `#![feature(...)]` attribute is required on 1.96. NOTE: the obsolete nightly **module** path `use std::assert_matches::assert_matches;` does NOT resolve on stable — it fails with E0432 ("could not find `assert_matches` in `std`/`core`"); only the crate-root macro re-export is stable. Verified compiling on `rustc 1.96.0 (ac68faa20 2026-05-25)` for plain, guard (`Pattern if guard`), and message (`Pattern, "msg {…}"`) forms, including under `#![no_std]`.
- `cargo build` MUST run before commit so `Cargo.lock` refreshes (AGENTS.md § Workflow).
- Stage files explicitly; never `git add -A`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Cargo.toml:24` reads `rust-version = "1.96"`; no other `Cargo.toml` carries a divergent literal MSRV (all members inherit). |
| AC2 | `README.md:82` reads "Rust stable (≥ 1.96)". |
| AC3 | `design-system/README.md:87` cites `1.96` (not `1.95`) as the MSRV in the numerical-precision example. |
| AC4 | Every `assert!(matches!(...))` in **test code** (`#[cfg(test)]` modules + `tests/`) is converted to `assert_matches!(...)` EXCEPT the carve-out sites below, each affected module carrying the crate-root import — `use core::assert_matches;` in `#![no_std]` crates / `use std::assert_matches;` in std crates and `tests/`. The renderer sites (`quartzite-renderer/src/{render_harness,vello_painter,event_convert}.rs`) ARE included (unblocked by the cheap `LocalBrushKind` derive — AC13). No external `assert_matches` crate is added (verify `Cargo.toml` files unchanged in dependency lists). Doc-example occurrences are intentionally excluded (see Out of scope). **Carve-outs that remain `assert!(matches!(...))` (known-allowed, excluded from the residual defect-grep):** (a) the 7 `quartzite-widgets` `WidgetView` sites — `widget_base.rs` + `widgets/{button,label,text_edit,scroll_area,container,line_edit}.rs` (1 each); (b) the runtime snapshot sites in quartzite-runtime matching `Box<dyn Object>` or `ObjectTree` — `snapshot/tree.rs`, `snapshot/object.rs:316`, `snapshot/object.rs:332`, `tests/snapshot.rs` (these are `#[cfg(feature = "serde")]`-gated). Both groups would require the rejected expensive supertrait (`AsWidget` / `Object`) to make their scrutinee `Debug`. The single runtime site `snapshot/object.rs:421` (scrutinee `ObjectSnapshot`, which IS `Debug`) MAY be converted — the design phase decides. |
| AC5 | `cargo build` succeeds and `Cargo.lock` is refreshed/staged. |
| AC6 | `cargo clippy --workspace --all-targets -- -D warnings` passes. |
| AC7 | `cargo fmt -- --check` passes. |
| AC8 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes. |
| AC9 | `cargo build -p quartzite --no-default-features --features libm` passes (no_std path). |
| AC10 | `cargo test` passes across the workspace. |
| AC11 | No remaining literal `1.95` MSRV reference in `Cargo.toml` files, `README.md`, or `design-system/README.md` (verify via grep). |
| AC12 | `.github/workflows/miri.yml` pins `nightly-2026-05-29`, with no remaining `nightly-2026-05-01` literal anywhere in the file (including comments — verify via grep), AND `actionlint .github/workflows/miri.yml` passes (AGENTS.md AXIOM: actionlint MUST pass before `git add` on any modified workflow file). `coverage.yml` is unchanged. |
| AC13 | The crate-private enum `LocalBrushKind` (`quartzite-renderer/src/vello_painter.rs:156`) carries a cheap `#[derive(Debug)]` (private type, no ripple), enabling the renderer-site conversions in AC4. |
| AC14 | The macro-generated `AsWidget` trait is UNCHANGED — it declares NO `Debug` supertrait. `quartzite-macros` codegen is reverted to its pre-cascade state (`pub trait AsWidget` carries no `core::fmt::Debug` bound), and the `quartzite-macros` codegen test(s) assert the trait has no `Debug` supertrait (or simply do not require one). `WidgetView<'_>` does NOT derive `Debug`. |
| AC15 | The concrete widget structs `Button`, `Label`, `TextEdit`, `ScrollArea`, `Container`, `LineEdit`, `WidgetBase`, `ThirdPartyWidget`, and `ClippingWidget` each derive `Debug` — additive, harmless, non-breaking cheap local derives (user-confirmed). This is NOT required to compile any conversion (the `WidgetView` sites are carved out per AC4 regardless). This is a specific instance of the workspace-wide cheap-`Debug` sweep (AC16). |
| AC16 | Every PUBLIC struct/enum in the design phase's enumerated cheap-`Debug` set carries `#[derive(Debug)]`, and the workspace builds clean (AC5/AC6/AC8/AC10). "Cheap" = all field/variant-payload types already `Debug` and no public-trait `Debug` supertrait is added (`AsWidget` / `Object` stay supertrait-free per AC14 and the Scope-item-6 policy). The AC is satisfied when the design's enumerated list is fully applied and the workspace compiles, docs, lints, and tests clean — it is checked against that enumerated list (the design owns the concrete set), NOT an unbounded "literally every type". The AC13 (`LocalBrushKind`) and AC15 (widget structs) derives are members of this set. Types already deriving/impl'ing `Debug` (no-op) and any type whose plain derive would fail to compile (non-`Debug` field, or supertrait required) are correctly absent from the set. |

## Open questions

- (none)
