# Progress: paint-style #47 — ACTIVE
_Updated: 2026-05-09 (7 of 16 subtasks done; entering second checkpoint-handoff)_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-05-09-paint-style
**base_commit:** 837128843ea3e91581aa2f409af3da3c57249b2b
**Last build:** PASS (workspace; clippy --workspace -D warnings clean; tests green; doc gate green)
**Issue:** #47
**Spec:** ai-docs/plans/2026-05-09-paint-style.spec.md
**Design:** ai-docs/plans/2026-05-09-paint-style.design.md (round-2 GO verdict)

## Next action

**Do this immediately:** Implement subtask 8 — replace the `quartzite-paint::Path` stub with re-exports from `quartzite_paint_api` (and from `quartzite_geometry::Alignment`). Delete `quartzite-paint/src/path.rs`; in `quartzite-paint/src/lib.rs` remove `mod path;` / `pub use path::Path;`, then add `pub use quartzite_paint_api::{Font, FontWeight, Image, ImageError, Path, Segment};` and `pub use quartzite_geometry::Alignment;`. Extend `re_exported_color_accessible` (or add a sibling test) to probe one new re-export, e.g. `let _ = Path::new(); let _ = Alignment::default();`.

## Subtasks

- [x] 1. quartzite-geometry: add Alignment enum + macros/core deps (3 new tests)
- [x] 2. quartzite-paint-api: ungate `extern crate alloc;` (no new tests)
- [x] 3. quartzite-paint-api: add `Color::with_alpha` (4 new tests)
- [x] 4. quartzite-paint-api: add Font + FontWeight (5 new tests)
- [x] 5. quartzite-paint-api: add Image + ImageError (6 new tests)
- [x] 6. quartzite-paint-api: add Path + Segment (6 new tests)
- [x] 7. quartzite-paint-api + quartzite-renderer: extend Painter trait, sync VelloPainter (3 painter tests; 11-method coverage)
- [ ] 8. quartzite-paint: replace Path stub with full re-exports of paint-api types  ← CURRENT
- [ ] 9. quartzite-style-types: new leaf crate scaffold (workspace member, Cargo.toml, lib.rs)
- [ ] 10. quartzite-style-types: ColorRole enum + ColorRole::ALL constant
- [ ] 11. quartzite-style-types: Palette (color/with_role, default, ColorRole indexing)
- [ ] 12. quartzite-widgets: remove local Alignment/Font/Palette, re-export from upstream
- [ ] 13. quartzite-style: new downstream crate scaffold
- [ ] 14. quartzite-style: Style trait (Send + Sync, generic-only `draw_widget`)
- [ ] 15. quartzite-style: StyleRegistry (Box::leak, Mutex+OnceLock, poison-recovery test, `clear_for_test`/`poison_for_test` helpers)
- [ ] 16. facade re-exports (`quartzite::paint::*`, `quartzite::style::*`) + `cargo tree -p quartzite-widgets` assertion + workspace doc/clippy gate

## Subtask 8 details (next)

**Goal:** Make `quartzite-paint` a thin re-export shell over `quartzite-paint-api`'s `Path`/`Font`/`Image`/`Segment` plus `quartzite-geometry::Alignment`. Removes the placeholder `Path` stub now that the canonical type lives upstream.

**Files:**
- DELETE `quartzite-paint/src/path.rs` (stub `pub struct Path;`).
- EDIT `quartzite-paint/src/lib.rs`:
  - Remove `mod path;` and `pub use path::Path;`.
  - Extend the existing `pub use quartzite_paint_api::{...};` line to also re-export `Font`, `FontWeight`, `Image`, `ImageError`, `Path`, `Segment`.
  - Add `pub use quartzite_geometry::Alignment;`.
  - Extend the `re_exported_color_accessible` test (or add a sibling `re_exports_full_vocabulary`) calling `let _ = Path::new(); let _ = Alignment::default(); let _ = Font::new("a", 1.0);`.

**Build / test commands at end of subtask 8:**
- `cargo build -p quartzite-paint`
- `cargo test -p quartzite-paint`
- `cargo build` (workspace — ensure widgets and renderer still compile)
- `cargo fmt`
- `cargo clippy --workspace -- -D warnings`

After subtask 8: update this file's "Next action" + "Subtasks" + "Files touched", then continue with subtasks 9–11 if context budget allows (style-types crate scaffold + ColorRole + Palette form a tight cohort), or stop after subtask 8 for a clean handoff.

## Key discoveries (don't re-investigate)

- **`quartzite-core` had a latent no_std build break** in `args_to_values.rs` (missing `use alloc::vec`). Already fixed in the subtask-1 commit; subsequent builds with `--no-default-features` work.
- **`quartzite-geometry` now depends on `quartzite-core` (default-features = false)** — required because `MetaEnum` derive expands to `::quartzite_core::*` references. Fix lives in `quartzite-geometry/Cargo.toml`.
- **`extern crate alloc;` is now unconditional in `quartzite-paint-api`** (no longer `#[cfg(test)]`-gated). Production code can name `String`/`Vec` directly.
- **`Color` already has the f32 channel API** (NOT u8 — supersedes the 2026-05-01 deferred draft). `Color::RED`/`BLACK`/`WHITE`/`GREEN`/`BLUE`/`TRANSPARENT` constants exist; `with_alpha(a: f32) -> Color` const fn landed in subtask 3.
- **`Pen::new(color, width)` is two-arg already** (NOT single-arg). `Pen::default()` is black/1.0px and the existing `default_is_black_one_pixel` test covers AC3 — no new test needed.
- **`Brush::solid(Color)` + `BrushKind::Solid(Color)` `#[non_exhaustive]`** is the live shape — gradient variants stay deferred.
- **`Painter` trait now has 11 methods** (was 7): `draw_rect`, `fill_rect`, `draw_line`, `clip_rect`, `translate`, `save`, `restore`, plus the four added in subtask 7: `draw_text`, `draw_text_in`, `draw_image`, `draw_path`. `VelloPainter` got matching no-op stubs.
- **`Point` is `i32`-based**, `PointF` is `f32`-based. `Path::move_to(p: Point)` thus takes integer-pixel points.
- **`assert_matches` is unstable in core** and there is no `assert_matches` crate dependency in this workspace — the path tests use plain `assert!(matches!(...))` instead. Keep this convention if subtask 8 / later tests need pattern-match assertions.
- **`Image::try_new` validates `width * height * 4` via `checked_mul`** and returns `ImageError::Overflow` on 32-bit-target overflow, `ImageError::PixelLengthMismatch { expected, actual }` on length mismatch. `ImageError` is `#[non_exhaustive]` and derives `thiserror::Error`.
- **`Path` is `Clone + Debug + Default + PartialEq`** (PartialEq derived; design § Open question 3 defaulted to "yes derive"). `Segment` is `#[non_exhaustive] enum` with `MoveTo(Point)`, `LineTo(Point)`, `CubicTo(Point, Point, Point)`, `ArcTo { centre, radii, start_angle, sweep_angle }`, `Close`.
- **`Font::new` is `_Simple._` (single non-simple call: `family.into()`)** — kept as a single fn without inner-fn split per AGENTS.md generic-fn split rule (the inner would have been trivial → unwrap rule applies).
- **Cargo cycle resolution**: `quartzite-style-types` (new leaf crate) holds `Palette` + `ColorRole`; `quartzite-style` is downstream. Widgets depends on `style-types` only — never on `style`. AC13's `cargo tree -p quartzite-widgets` step in subtask 16 is the contract enforcer.
- **`StyleRegistry` storage is `OnceLock<Mutex<Option<&'static dyn Style>>>`** — `set_style` calls `Box::leak` (replacement leaks the prior box, acceptable for process-lifetime registry). `try_style` returns `Option<&'static dyn Style>` after dropping the guard. Lock-poisoning recovered via `lock().unwrap_or_else(|e| e.into_inner())` per AGENTS.md library-safety idioms.
- **`Style: Send + Sync` is required** (global registry); zero-sized fixture types satisfy automatically.
- **`Alignment` was MOVED (not duplicated)** to `quartzite-geometry`. `quartzite-widgets` will re-export it in subtask 12; until then both crates have the type but widgets's existing `enums::Alignment` stays for now (subtask 1 didn't remove it — that's deliberate; subtask 12 deletes it).

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
| AC9 | NOT_TESTED | Subtask 11 (Palette default) |
| AC10 | NOT_TESTED | Subtask 15 (StyleRegistry try_style + poison-recovery) |
| AC11 | NOT_TESTED | Subtask 14 (Style trait) |
| AC12 | PASS (partial) | `discriminants_match_legacy_widget_alignment` green; `into_value_round_trip` green; final assertion against widgets-side type happens in subtask 12 |
| AC13 | NOT_TESTED | Subtask 16 (cargo tree assertion) |
| AC14 | PASS (partial) | `cargo build -p quartzite-geometry --no-default-features` green; paint-api no-default-features still green |
| AC15 | PASS (partial) | `cargo doc -D warnings -D missing-docs --workspace` clean at the subtask-7 commit; will re-run at the final gate |
| AC16 | PASS | `clippy --workspace -- -D warnings` clean at the subtask-7 commit |

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

## Commit log on this branch

- e03c82c: `feat(paint-style): geometry+paint-api foundation (subtasks 1–3 of 16)`
- (next): `feat(paint-style): paint-api Font/Image/Path + extended Painter (subtasks 4–7 of 16)`

## Handoff guardrails

- **Do NOT edit on master.** Run `git branch --show-current` before any commit; must read `feat/2026-05-09-paint-style`. Recovery procedure in AGENTS.md if it ever shows `master`.
- **Stage files explicitly by name.** Never `git add -A` / `git add .` (AGENTS.md).
- **Never `--no-verify`.** Never `git reset --hard`. Never `--amend` an existing commit (always new commit on hook failure).
- **Per subtask:** `cargo build` + targeted `cargo test <name>` + update this file. Workspace-wide `cargo test` + `cargo clippy --workspace -- -D warnings` only at end of a logical group / handoff.
- **End each Agent dispatch with a commit** so subsequent handoffs have clean baselines.
- **Maximum 3 handoffs total.** Subtasks 1–3 → 4–7 → 8–16. Now entering the third dispatch (subtasks 8–16): paint re-exports, then style-types, widgets refactor, style crate, facade.
- **Resolution log in spec.** The Cargo-cycle resolution and the Box::leak storage decision are pre-baked into the spec — do NOT re-litigate.
