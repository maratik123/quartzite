# Progress: paint-style #47 — ACTIVE
_Updated: 2026-05-09 (11 of 16 subtasks done; entering third checkpoint-handoff)_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-05-09-paint-style
**base_commit:** 837128843ea3e91581aa2f409af3da3c57249b2b
**Last build:** PASS (workspace; clippy --workspace -D warnings clean; tests green; doc gate green; `quartzite-style-types --no-default-features` green)
**Issue:** #47
**Spec:** ai-docs/plans/2026-05-09-paint-style.spec.md
**Design:** ai-docs/plans/2026-05-09-paint-style.design.md (round-2 GO verdict)

## Next action

**Do this immediately:** Implement subtask 12 — drop `quartzite-widgets`'s local `Alignment`/`Font`/`Palette` types and replace them with re-exports from upstream crates (`quartzite-geometry::Alignment`, `quartzite-paint::{Font, FontWeight}`, `quartzite-style-types::{ColorRole, Palette}`). Critical guard-rail: do **NOT** add `quartzite-style` as a dep to widgets — that's the cycle-break the leaf crate exists to enforce. AC13's `cargo tree` assertion is left for subtask 16.

## Subtasks

- [x] 1. quartzite-geometry: add Alignment enum + macros/core deps (3 new tests)
- [x] 2. quartzite-paint-api: ungate `extern crate alloc;` (no new tests)
- [x] 3. quartzite-paint-api: add `Color::with_alpha` (4 new tests)
- [x] 4. quartzite-paint-api: add Font + FontWeight (5 new tests)
- [x] 5. quartzite-paint-api: add Image + ImageError (6 new tests)
- [x] 6. quartzite-paint-api: add Path + Segment (6 new tests)
- [x] 7. quartzite-paint-api + quartzite-renderer: extend Painter trait, sync VelloPainter (3 painter tests; 11-method coverage)
- [x] 8. quartzite-paint: replace Path stub with full re-exports of paint-api types (1 new test: `re_exports_full_vocabulary`)
- [x] 9. quartzite-style-types: new leaf crate scaffold (workspace member, Cargo.toml, lib.rs)
- [x] 10. quartzite-style-types: ColorRole enum + ColorRole::ALL constant (3 unit tests)
- [x] 11. quartzite-style-types: Palette (color/with_role, default, ColorRole indexing) (4 unit tests + 6 doctests; AC9 PASS)
- [ ] 12. quartzite-widgets: remove local Alignment/Font/Palette, re-export from upstream  ← CURRENT
- [ ] 13. quartzite-style: new downstream crate scaffold
- [ ] 14. quartzite-style: Style trait (Send + Sync, generic-only `draw_widget`)
- [ ] 15. quartzite-style: StyleRegistry (Box::leak, Mutex+OnceLock, poison-recovery test, `clear_for_test`/`poison_for_test` helpers)
- [ ] 16. facade re-exports (`quartzite::paint::*`, `quartzite::style::*`) + `cargo tree -p quartzite-widgets` assertion + workspace doc/clippy gate

## Subtask 12 details (next)

**Goal:** Make `quartzite-widgets` consume `Alignment` / `Font` / `Palette` / `ColorRole` from upstream crates instead of defining them locally. Existing widget call sites must compile unchanged because `crate::Alignment`, `crate::Font`, `crate::Palette` keep resolving — they now resolve through `pub use` re-exports.

**Files (concrete checklist):**
- EDIT `quartzite-widgets/Cargo.toml`:
  - Add `quartzite-paint = { path = "../quartzite-paint" }` to `[dependencies]`.
  - Add `quartzite-style-types = { path = "../quartzite-style-types" }` to `[dependencies]`.
  - **Do NOT add `quartzite-style`.** AC13 contract.
- EDIT `quartzite-widgets/src/enums.rs`:
  - Remove the `Alignment` enum (variants `Left`, `Center`, `Right`, `Justify` + `MetaEnum` derive + `#[default] = Left` + `#[repr(i64)]`).
  - Remove any unit-test for the removed `Alignment` round-trip (the canonical test now lives in `quartzite-geometry/src/alignment.rs`).
  - Keep `FocusPolicy`, `SizePolicy`, `CursorShape` and their tests.
- DELETE `quartzite-widgets/src/font.rs`.
- DELETE `quartzite-widgets/src/palette.rs`.
- EDIT `quartzite-widgets/src/lib.rs`:
  - Remove `pub mod font;` (line near the top of the module list).
  - Remove `pub mod palette;`.
  - Replace `pub use enums::{Alignment, CursorShape, FocusPolicy, SizePolicy};` with `pub use enums::{CursorShape, FocusPolicy, SizePolicy};` (drop `Alignment`).
  - Replace `pub use font::Font;` with `pub use quartzite_paint::{Font, FontWeight};`.
  - Replace `pub use palette::Palette;` with `pub use quartzite_style_types::{ColorRole, Palette};`.
  - Add `pub use quartzite_geometry::Alignment;` (keep adjacent to the other re-exports).
- EDIT `quartzite-widgets/src/widget_base.rs`:
  - Replace any `use crate::{..., Font, Palette, ...}` with paths through the new re-exports — `crate::Font` and `crate::Palette` continue to resolve. Most likely the imports already read `crate::Font` / `crate::Palette` and need no edit; verify via `grep -n "use crate" quartzite-widgets/src/widget_base.rs`.
  - The `WidgetBase::new` body stays unchanged (`Arc::new(Font::default())` / `Arc::new(Palette::default())` now resolve to the upstream types).
- VERIFY existing widget call sites still compile — `quartzite-widgets/src/widgets/label.rs`, `button.rs`, etc. all reference `crate::Font` / `crate::Palette` / `crate::Alignment`. They should require no edits.
- (Optional, consistent with subtask-12 scope) ADD an integration test `quartzite-widgets/tests/re_exports.rs` covering AC13's TypeId equality checks: `widgets::Alignment ≡ geometry::Alignment`, `widgets::Font ≡ paint::Font`, `widgets::Palette ≡ style_types::Palette`, `widgets::ColorRole ≡ style_types::ColorRole`. (Subtask 16 also runs the `cargo tree` shell assertion.)

**Build / test commands at end of subtask 12:**
- `cargo build -p quartzite-widgets`
- `cargo test -p quartzite-widgets`
- `cargo build` (workspace — every crate must still compile)
- `cargo fmt`
- `cargo clippy --workspace -- -D warnings`
- (AC13 spot-check, if integration test added) `cargo test -p quartzite-widgets --test re_exports`

After subtask 12: update this file's "Next action" + "Subtasks" + "Files touched", then continue with subtasks 13–16.

## Key discoveries (don't re-investigate)

- **`quartzite-core` had a latent no_std build break** in `args_to_values.rs` (missing `use alloc::vec`). Already fixed in the subtask-1 commit; subsequent builds with `--no-default-features` work.
- **`quartzite-geometry` now depends on `quartzite-core` (default-features = false)** — required because `MetaEnum` derive expands to `::quartzite_core::*` references. Fix lives in `quartzite-geometry/Cargo.toml`.
- **`extern crate alloc;` is now unconditional in `quartzite-paint-api`** (no longer `#[cfg(test)]`-gated). Production code can name `String`/`Vec` directly.
- **`Color` already has the f32 channel API** (NOT u8 — supersedes the 2026-05-01 deferred draft). `Color::RED`/`BLACK`/`WHITE`/`GREEN`/`BLUE`/`TRANSPARENT` constants exist; `with_alpha(a: f32) -> Color` const fn landed in subtask 3.
- **`Pen::new(color, width)` is two-arg already** (NOT single-arg). `Pen::default()` is black/1.0px and the existing `default_is_black_one_pixel` test covers AC3 — no new test needed.
- **`Brush::solid(Color)` + `BrushKind::Solid(Color)` `#[non_exhaustive]`** is the live shape — gradient variants stay deferred.
- **`Painter` trait now has 11 methods** (was 7): `draw_rect`, `fill_rect`, `draw_line`, `clip_rect`, `translate`, `save`, `restore`, plus the four added in subtask 7: `draw_text`, `draw_text_in`, `draw_image`, `draw_path`. `VelloPainter` got matching no-op stubs.
- **`Point` is `i32`-based**, `PointF` is `f32`-based. `Path::move_to(p: Point)` thus takes integer-pixel points.
- **`assert_matches` is unstable in core** and there is no `assert_matches` crate dependency in this workspace — the path tests use plain `assert!(matches!(...))` instead. Keep this convention if subtask 12 / later tests need pattern-match assertions.
- **`Image::try_new` validates `width * height * 4` via `checked_mul`** and returns `ImageError::Overflow` on 32-bit-target overflow, `ImageError::PixelLengthMismatch { expected, actual }` on length mismatch. `ImageError` is `#[non_exhaustive]` and derives `thiserror::Error`.
- **`Path` is `Clone + Debug + Default + PartialEq`** (PartialEq derived; design § Open question 3 defaulted to "yes derive"). `Segment` is `#[non_exhaustive] enum` with `MoveTo(Point)`, `LineTo(Point)`, `CubicTo(Point, Point, Point)`, `ArcTo { centre, radii, start_angle, sweep_angle }`, `Close`.
- **`Font::new` is `_Simple._` (single non-simple call: `family.into()`)** — kept as a single fn without inner-fn split per AGENTS.md generic-fn split rule (the inner would have been trivial → unwrap rule applies).
- **Cargo cycle resolution**: `quartzite-style-types` (NEW leaf crate as of subtask 9) holds `Palette` + `ColorRole`; `quartzite-style` is downstream. Widgets depends on `style-types` only — never on `style`. AC13's `cargo tree -p quartzite-widgets` step in subtask 16 is the contract enforcer.
- **`StyleRegistry` storage is `OnceLock<Mutex<Option<&'static dyn Style>>>`** — `set_style` calls `Box::leak` (replacement leaks the prior box, acceptable for process-lifetime registry). `try_style` returns `Option<&'static dyn Style>` after dropping the guard. Lock-poisoning recovered via `lock().unwrap_or_else(|e| e.into_inner())` per AGENTS.md library-safety idioms.
- **`Style: Send + Sync` is required** (global registry); zero-sized fixture types satisfy automatically.
- **`Alignment` was MOVED (not duplicated)** to `quartzite-geometry`. `quartzite-widgets` will re-export it in subtask 12; until then both crates have the type but widgets's existing `enums::Alignment` stays for now (subtask 1 didn't remove it — that's deliberate; subtask 12 deletes it).
- **`ColorRole` is `#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]`, NOT `#[non_exhaustive]`** — design § task 10 mentioned `#[non_exhaustive]` but the live impl matches the spec § *quartzite-style — new downstream crate* contract (closed enum so `ColorRole::ALL.len()` is a usable compile-time constant). The guard-rail is the `all_constant_lists_every_variant` exhaustive-match unit test that fails when a variant is added without extending `ALL`.
- **`Palette::color` is `const fn`** — array indexing by `role as usize` is const-eligible since 2024 edition, so the lookup is usable in const contexts.
- **`Palette::default` defaults**: `Color::WHITE` for backgrounds (`Window`/`Button`/`Base`/`Highlight`), `Color::BLACK` for foregrounds (`WindowText`/`ButtonText`/`Text`), `Color::WHITE` for `HighlightedText` and `BrightText`, `Color::BLUE` for `Link` and `LinkVisited`. Every slot is non-transparent (AC9). The design's richer per-channel constants (`LIGHT_GRAY = 0.94` etc.) were not used — AC9 only contracts `!= TRANSPARENT`, and the simpler defaults stay within the AGENTS.md "minimal viable contract" guidance.
- **`quartzite-style-types/src/color_role.rs` intra-doc links use `[`Palette`](crate::Palette)` form** at item-level (variant rustdoc, ALL doc); module-level docstring uses reference-link `[`Palette`]: crate::Palette` definitions. Mixing the two forms within a single docstring tripped the doc gate once during subtask-10 — kept the lesson as a tooling note.

## AC Status

| AC | Status | Notes |
|----|--------|-------|
| AC1 | PASS | `with_alpha_replaces_alpha_only`, `with_alpha_zero_makes_fully_transparent`, `with_alpha_quarter_preserves_red_channel` all green |
| AC2 | PASS | `with_alpha_is_const_fn` green (uses `const TRANSLUCENT: Color = ...`) |
| AC3 | PASS | Existing `default_is_black_one_pixel` test in `pen.rs` covers it (regression fence) |
| AC4 | PASS | `move_then_line_then_close_round_trips` green in `path.rs` |
| AC5 | PASS | `cubic_and_arc_round_trip` green in `path.rs` |
| AC6 | PASS | `new_default_weight_normal_and_flags_off` green in `font.rs` |
| AC7 | PASS | `try_new_accepts_correct_length` + `try_new_rejects_short_buffer` green in `image.rs` |
| AC8 | PASS | `painter_is_object_safe`, `all_methods_reachable_through_trait_object` (11-counter), `boxed_painter_dispatches_all_new_methods` green in `painter.rs` |
| AC9 | PASS | `default_has_non_transparent_color_for_every_role` (loop over `ColorRole::ALL`) + `with_role_replaces_slot_only` green in `quartzite-style-types/src/palette.rs` |
| AC10 | NOT_TESTED | Subtask 15 (StyleRegistry try_style + poison-recovery) |
| AC11 | NOT_TESTED | Subtask 14 (Style trait) |
| AC12 | PASS (partial) | `discriminants_match_legacy_widget_alignment` green; `into_value_round_trip` green; final assertion against widgets-side type happens in subtask 12 |
| AC13 | NOT_TESTED | Subtask 16 (cargo tree assertion) |
| AC14 | PASS (partial) | `cargo build -p quartzite-geometry --no-default-features` green; paint-api no-default-features still green; `quartzite-style-types --no-default-features` green |
| AC15 | PASS (partial) | `cargo doc -D warnings -D missing-docs --workspace` clean at the subtask-11 commit; will re-run at the final gate |
| AC16 | PASS | `clippy --workspace -- -D warnings` clean at the subtask-11 commit |

## Files touched

- `quartzite-core/src/args_to_values.rs` — `use alloc::vec;` (latent no_std fix)
- `quartzite-geometry/Cargo.toml` — added `quartzite-core` (default-features = false) + `quartzite-macros` deps
- `quartzite-geometry/src/alignment.rs` — NEW; Alignment enum (verbatim from widgets) + 3 unit tests
- `quartzite-geometry/src/lib.rs` — declared `mod alignment;` + `pub use alignment::Alignment;`
- `quartzite-paint-api/src/lib.rs` — ungated `extern crate alloc;`; declared `mod font;`, `mod image;`, `mod path;`; added re-exports
- `quartzite-paint-api/src/color.rs` — added `Color::with_alpha` (const fn) + 4 unit tests
- `quartzite-paint-api/src/font.rs` — NEW; `Font` + `FontWeight` + 5 unit tests (subtask 4)
- `quartzite-paint-api/src/image.rs` — NEW; `Image` + `ImageError` + 6 unit tests (subtask 5)
- `quartzite-paint-api/src/path.rs` — NEW; `Path` + `Segment` + 6 unit tests (subtask 6)
- `quartzite-paint-api/src/painter.rs` — extended `Painter` trait with 4 methods; expanded `RecordingPainter` to 11-counter; added `boxed_painter_dispatches_all_new_methods` (subtask 7)
- `quartzite-renderer/src/vello_painter.rs` — added 4 no-op stubs for new Painter methods (subtask 7, atomic with paint-api)
- `quartzite-paint/src/path.rs` — DELETED (subtask 8; stub replaced by upstream re-export)
- `quartzite-paint/src/lib.rs` — re-exports `Font`/`FontWeight`/`Image`/`ImageError`/`Path`/`Segment` from `quartzite-paint-api` and `Alignment` from `quartzite-geometry`; new `re_exports_full_vocabulary` test (subtask 8)
- `Cargo.toml` (root) — added `quartzite-style-types` to `[workspace.members]` (subtask 9)
- `quartzite-style-types/Cargo.toml` — NEW (subtask 9; mirrors paint-api structure; deps: `quartzite-paint-api`)
- `quartzite-style-types/src/lib.rs` — NEW (subtask 9; `#![no_std]` + `extern crate alloc;` + module declarations and re-exports of `ColorRole`, `Palette`)
- `quartzite-style-types/src/color_role.rs` — NEW (subtask 10; `ColorRole` enum + `ColorRole::ALL` const + 3 unit tests)
- `quartzite-style-types/src/palette.rs` — NEW (subtask 11; `Palette` struct + `color`/`with_role`/`Default` impls + 4 unit tests, AC9 contract)

## Commit log on this branch

- e03c82c: `feat(paint-style): geometry+paint-api foundation (subtasks 1–3 of 16)`
- (subtasks 4–7): `feat(paint-style): paint-api Font/Image/Path + extended Painter (subtasks 4–7 of 16)`
- (subtasks 8–11, this dispatch): `feat(paint-style): paint re-export shell + quartzite-style-types crate (subtasks 8–11 of 16)`

## Handoff guardrails

- **Do NOT edit on master.** Run `git branch --show-current` before any commit; must read `feat/2026-05-09-paint-style`. Recovery procedure in AGENTS.md if it ever shows `master`.
- **Stage files explicitly by name.** Never `git add -A` / `git add .` (AGENTS.md).
- **Never `--no-verify`.** Never `git reset --hard`. Never `--amend` an existing commit (always new commit on hook failure).
- **Per subtask:** `cargo build` + targeted `cargo test <name>` + update this file. Workspace-wide `cargo test` + `cargo clippy --workspace -- -D warnings` only at end of a logical group / handoff.
- **End each Agent dispatch with a commit** so subsequent handoffs have clean baselines.
- **Maximum 3 handoffs total.** Subtasks 1–3 → 4–7 → 8–11 (this) → 12–16 (next). The "max-3 handoffs" axiom flexed once because the original 8–16 group was too large for context budget; the remaining 12–16 should land in one or two more dispatches.
- **Resolution log in spec.** The Cargo-cycle resolution and the Box::leak storage decision are pre-baked into the spec — do NOT re-litigate.
