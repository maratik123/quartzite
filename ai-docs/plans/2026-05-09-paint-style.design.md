# Design: Paint & Style

**Issue:** #47
**Spec:** [`2026-05-09-paint-style.spec.md`](./2026-05-09-paint-style.spec.md)
**Date:** 2026-05-09 (round-2 revision)

## Approach

The plan touches five existing crates and stands up two new ones — `quartzite-style-types`
(leaf) and `quartzite-style` (downstream). The dependency order anchors the entire
decomposition; round 1 collapsed because the original design tried to put `Palette` and the
`Style` trait in the same crate, which created an unbreakable Cargo cycle (`Style::draw_widget`
needs `&dyn AsWidget` from `quartzite-widgets` → `quartzite-widgets` re-exports `Palette` from
`quartzite-style` → cycle). The fix splits `Palette`/`ColorRole` into a leaf crate that widgets
can depend on without pulling in `Style`/`StyleRegistry`. Concretely the order is:

```
quartzite-geometry      (Alignment + macros + core deps; no_std)
        ↓
quartzite-paint-api     (with_alpha, alloc ungate, Font, Image, Path, extended Painter; no_std + alloc)
        ↓
quartzite-paint         (re-exports Path/Font/Image/Alignment + deletes the stub Path)
        ↓
quartzite-style-types   (NEW — Palette, ColorRole; no_std + alloc, leaf)
        ↓
quartzite-widgets       (drop local Alignment/Font/Palette; re-export from upstream)
        ↓
quartzite-style         (NEW — Style trait, StyleRegistry with Box::leak; std-only)
```

`quartzite-renderer` is a sibling consumer of `quartzite-paint-api`: when task 2 extends the
`Painter` trait, the same edit must add no-op bodies for the four new methods to
`VelloPainter` so the workspace compiles. The renderer is **not** an upstream dependency of
the new crates and never imports anything from `quartzite-style`.

Three architectural choices anchor the design.

**1. Foundation types (`Font`, `Image`, `Path`, `Alignment`) live upstream of the trait that
uses them.** The `Painter` trait is in `quartzite-paint-api`; placing `Font`/`Image`/`Path`
in the same crate avoids a circular re-export and keeps the trait object-safe (no generics,
no `Self` returns). `quartzite-paint` re-exports them so callers depending on the
higher-level crate see the full vocabulary in one place. `Alignment` sits even lower, in
`quartzite-geometry`, because (a) it's a `Copy` enum with no allocations and (b) widgets
and paint-api both want to import it without one depending on the other.

**2. `Path` uses an internal `Vec<Segment>` with a `#[non_exhaustive]` Segment enum.** The
builder methods (`move_to`, `line_to`, `cubic_to`, `arc_to`, `close`) return `&mut Self`,
matching the spec. The enum is `#[non_exhaustive]` so future variants (e.g. `QuadTo`) can
land without an SemVer bump. `Path` itself is `Clone + Debug + Default`. The struct field
stays private; the public read accessor is `pub fn segments(&self) -> &[Segment]` — backends
consume the slice. The full implementation lives in `quartzite-paint-api` (next to
`Painter::draw_path`) so the `Painter` trait can name `&Path` directly without a backwards
re-export through `quartzite-paint`. The spec § *Key decisions* table contains the
authorising line: "Where exactly the type lives between paint and paint-api is a
design-phase decision; the Painter-facing alias is the constraint." The design pins it to
`paint-api` and the higher-level `quartzite-paint` re-exports.

**3. `quartzite-style` is a new `std`-only crate. The global registry uses
`OnceLock<Mutex<Option<&'static dyn Style>>>`.** Round 1 stored
`Option<Box<dyn Style>>`, which made `try_style() -> Option<&'static dyn Style>` impossible
to implement (you cannot return a `&'static` reference into a `MutexGuard`). The revised
storage is `OnceLock<Mutex<Option<&'static dyn Style>>>`: `set_style` calls `Box::leak` to
obtain the `'static` reference and stores it; `try_style` clones the `Option<&'static dyn
Style>` (`&'static dyn Style` is `Copy`) and returns it after dropping the guard. Lock
poisoning is recovered via `lock().unwrap_or_else(|e| e.into_inner())` per AGENTS.md
§ *Library safety idioms*. Replacement leaks the previous box; this is acceptable for a
process-lifetime registry (typical apps swap styles zero or one times).

A panicking convenience `style()` is **not** added in this plan — the spec allows it but
defers the decision; YAGNI says omit until a concrete consumer needs it.

### Cycle resolution: leaf crate `quartzite-style-types`

The round-1 blocker was: `quartzite-style` defines `Style::draw_widget(&dyn AsWidget, ...)`
so it must depend on `quartzite-widgets`; `quartzite-widgets` re-exports `Palette` so it
must depend on the crate that defines `Palette`. Both crates cannot be each other's
ancestor. The cycle breaks by splitting:

- `quartzite-style-types` (NEW, leaf, `no_std + alloc`) — defines `Palette`, `ColorRole`,
  `PaletteGroup`. Depends only on `quartzite-paint-api` for `Color`. Knows nothing about
  widgets or styles.
- `quartzite-style` (NEW, downstream, `std`) — defines the `Style` trait and
  `StyleRegistry`. Depends on `quartzite-style-types` (for `Palette` re-export and use in
  `draw_widget`'s signature), `quartzite-paint`, `quartzite-widgets` (for `&dyn AsWidget`).
  Re-exports `Palette` and `ColorRole` so downstream callers depending on
  `quartzite-style` see one vocabulary.

`quartzite-widgets` then depends on `quartzite-style-types` (for `Palette` re-export)
**only**, and never on `quartzite-style`. AC13 is the contract: `cargo tree -p
quartzite-widgets` lists `quartzite-style-types` but **not** `quartzite-style`.

### Rejected alternatives

- **`Path` as a builder type that finalises into a `PathData`** — rejected. Adds a second
  type for no win; the spec wants chained mutation on `&mut Self` and read access via
  `segments()`.
- **`Image` with a `Cow<'_, [u8]>` instead of `Vec<u8>`** — rejected. The spec fixes the v1
  shape to `Vec<u8>` and pre-publish API stability lets us evolve later. `Cow` adds a
  lifetime parameter to a leaf type for no real consumer.
- **`StyleRegistry` returning `Arc<dyn Style>` instead of `&'static dyn Style`** — rejected.
  Adds an Arc bump on every style access (hot path during paint) for the marginal benefit
  of letting old styles drop on replacement. The leak on replace is bounded by how many
  times a program swaps style (typically zero or one).
- **`StyleRegistry` storing `Option<Box<dyn Style>>` and returning `&'static dyn Style` via
  Box::leak inside `try_style`** — rejected (and was the round-1 blocker). Leaking on every
  read makes successive reads return *different* `'static` references and double-leaks the
  same box on hot paths. Leaking happens **once at install time** in `set_style`; reads are
  pure.
- **`Alignment` as a `bitflags!` value** — rejected. The existing `quartzite-widgets::Alignment`
  is a `MetaEnum` with `i64` discriminants and a property-system round-trip test
  (`Alignment::Center.into_value() == Value::Int(1)`). AC12 explicitly requires that
  round-trip to keep working, so the move must be verbatim. Bitflag-based "horizontal |
  vertical" alignment is a future spec.
- **`Style` trait carrying `draw_button` / `draw_label` / etc. as required methods** —
  rejected by the spec (round-1 interview decision). Concrete styles dispatch internally
  via downcast or a visitor; the trait stays generic-only.
- **`parking_lot::Mutex` instead of `std::sync::Mutex` for the registry** — rejected. The
  spec is explicit: AGENTS.md § *Library safety idioms* mentions
  `lock().unwrap_or_else(|e| e.into_inner())`, which is the std-Mutex idiom. AC10 calls
  out poison recovery as a test requirement; `parking_lot::Mutex` has no `PoisonError` to
  recover from.
- **Adding only `quartzite-macros` to `quartzite-geometry` (per spec wording)** — partly
  required, partly extended. The `MetaEnum` derive expansion references `quartzite-core`
  types (`EnumEntry`, `EnumMeta`, `IntoValue`, `FromValue`). The macro's `crate_root()`
  helper (`quartzite-macros/src/util.rs:76`) resolves to `::quartzite_core` when neither
  the facade nor `quartzite-core` is in the dependency graph — meaning the generated code
  refers to a crate that isn't there. The spec lists only `quartzite-macros`, but the
  derive *cannot compile* in a crate that has no path to the core trait crate. The design
  therefore adds `quartzite-core = { path = "../quartzite-core", default-features = false }`
  to `quartzite-geometry`. The *generated* `no_std`-friendliness of `quartzite-core` is
  established (its existing `--no-default-features` build is part of the workspace CI).
  This deviation from the spec wording is captured in § Open questions.

## Decomposition

Tasks are ordered strictly by dependency. The crate ordering is the new dependency graph
(`geometry → paint-api → paint → style-types → widgets → style`). Within a crate, additive
work (types) precedes mutating work (Painter trait + VelloPainter sync) so the workspace
compiles after every task in isolation where possible.

| #  | Task | Files | Depends on |
|----|------|-------|------------|
| 1  | **`quartzite-geometry`: add `Alignment` + macros/core deps.** Add `quartzite-macros = { path = "../quartzite-macros" }` and `quartzite-core = { path = "../quartzite-core", default-features = false }` to `quartzite-geometry/Cargo.toml`. Create `quartzite-geometry/src/alignment.rs` containing the `Alignment` enum copied verbatim from `quartzite-widgets/src/enums.rs` (variants + `MetaEnum` derive + `#[default] = Left` + `#[repr(i64)]`). Add `pub mod alignment;` and `pub use alignment::Alignment;` to `quartzite-geometry/src/lib.rs`. Verify `cargo build -p quartzite-geometry --no-default-features` still passes (AC14). | `quartzite-geometry/Cargo.toml`, `quartzite-geometry/src/alignment.rs`, `quartzite-geometry/src/lib.rs` | — |
| 2  | **`quartzite-paint-api`: ungate `extern crate alloc;`.** In `quartzite-paint-api/src/lib.rs`, move `extern crate alloc;` out of the `#[cfg(test)]` gate so production code can name `alloc::string::String` and `alloc::vec::Vec`. The crate stays `#![no_std]`. AC14 (`--no-default-features`) still holds. | `quartzite-paint-api/src/lib.rs` | — |
| 3  | **`quartzite-paint-api`: `Color::with_alpha`.** Add `pub const fn with_alpha(self, a: f32) -> Color` to `Color` (file `quartzite-paint-api/src/color.rs`). Returns `Self { a, ..self }`. `#[inline]`, `# Examples`, `# Parameters`. AC1 + AC2. | `quartzite-paint-api/src/color.rs` | — |
| 4  | **`quartzite-paint-api`: `Font` + `FontWeight`.** New file `quartzite-paint-api/src/font.rs`. `pub struct Font { family: String, size_pt: f32, weight: FontWeight, italic: bool, underline: bool, strikethrough: bool }`. `pub enum FontWeight { Thin=100, ExtraLight=200, Light=300, Normal=400, Medium=500, SemiBold=600, Bold=700, ExtraBold=800, Black=900 }` with `#[repr(u16)]`, `#[default = Normal]`, `Copy + Clone + Debug + PartialEq + Eq + Hash`. Constructor `Font::new(family: impl Into<String>, size_pt: f32) -> Self` (per AGENTS.md § *Generic-fn split*: body is small enough — three lines — that the conversion-style generic does not need an inner-fn split). Accessors: `family() -> &str`, `size_pt() -> f32`, `weight() -> FontWeight`, `italic() -> bool`, `underline() -> bool`, `strikethrough() -> bool`. `Default::default()` returns `Font { family: "sans-serif".into(), size_pt: 12.0, weight: FontWeight::Normal, italic: false, underline: false, strikethrough: false }`. Add `mod font;` + `pub use font::{Font, FontWeight};` to `lib.rs`. Depends on task 2 (alloc must be importable in production). | `quartzite-paint-api/src/font.rs`, `quartzite-paint-api/src/lib.rs` | 2 |
| 5  | **`quartzite-paint-api`: `Image` + `ImageError`.** New file `quartzite-paint-api/src/image.rs`. `pub struct Image { width: u32, height: u32, pixels: Vec<u8> }`. Constructor `pub fn try_new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, ImageError>` validates `pixels.len() == (width as usize).checked_mul(height as usize).and_then(\|n\| n.checked_mul(4)).ok_or(ImageError::Overflow)?`. Accessors `width()`, `height()`, `pixels() -> &[u8]`. `pub enum ImageError` (thiserror derive) with variants `PixelLengthMismatch { expected: usize, actual: usize }` and `Overflow` (width × height × 4 overflow `usize`). Add `mod image;` + `pub use image::{Image, ImageError};` to `lib.rs`. Depends on task 2. | `quartzite-paint-api/src/image.rs`, `quartzite-paint-api/src/lib.rs` | 2 |
| 6  | **`quartzite-paint-api`: `Segment` + `Path` (full).** New file `quartzite-paint-api/src/path.rs`. `#[non_exhaustive] pub enum Segment { MoveTo(Point), LineTo(Point), CubicTo(Point, Point, Point), ArcTo { centre: Point, radii: Size, start_angle: f32, sweep_angle: f32 }, Close }` deriving `Clone + Debug + PartialEq`. `pub struct Path { segments: Vec<Segment> }` deriving `Clone + Debug + Default`. Builder methods `new() -> Self`, `move_to(&mut self, p: Point) -> &mut Self`, `line_to`, `cubic_to(c1, c2, p)`, `arc_to(centre, radii, start_angle, sweep_angle)`, `close()`, plus reader `segments(&self) -> &[Segment]`. Add `mod path;` + `pub use path::{Path, Segment};` to `lib.rs`. `arc_to` semantics fixed at **centre-and-radii, angles in radians, positive sweep CCW** (matches vello/peniko). Depends on task 2. | `quartzite-paint-api/src/path.rs`, `quartzite-paint-api/src/lib.rs` | 2 |
| 7  | **`quartzite-paint-api`: extend `Painter` + sync `VelloPainter`.** Add four required methods to `Painter` in `quartzite-paint-api/src/painter.rs`: `draw_text(&mut self, pos: Point, text: &str, font: &Font, brush: &Brush)`, `draw_text_in(&mut self, rect: Rect, text: &str, font: &Font, brush: &Brush, alignment: Alignment)`, `draw_image(&mut self, rect: Rect, image: &Image)`, `draw_path(&mut self, path: &Path, pen: &Pen, brush: &Brush)`. Update the trait-level rustdoc example to call one of the new methods. Update `RecordingPainter` in `#[cfg(test)] mod tests` to track 11 calls (from 7) and update `painter_is_object_safe` / `all_methods_reachable_through_trait_object` to call all 11. **Same task** updates `quartzite-renderer/src/vello_painter.rs` to add empty `#[inline]` impls for the four new methods (no-op stubs; actual rendering deferred to a follow-up plan per spec § *Out of scope*). The renderer must compile in this same atomic edit because `quartzite-renderer` is in the workspace and is a sibling consumer of the trait. | `quartzite-paint-api/src/painter.rs`, `quartzite-renderer/src/vello_painter.rs` | 1, 4, 5, 6 |
| 8  | **`quartzite-paint`: re-exports + delete stub `Path`.** Delete `quartzite-paint/src/path.rs`. In `quartzite-paint/src/lib.rs`, remove `mod path;` / `pub use path::Path;`, then add `pub use quartzite_paint_api::{Font, FontWeight, Image, ImageError, Path, Segment};` and `pub use quartzite_geometry::Alignment;` to the existing re-export line. Extend the existing `re_exported_color_accessible` test to also probe a new re-export (e.g. `let _ = Path::new();`, `let _ = Alignment::default();`). | `quartzite-paint/src/lib.rs`, `quartzite-paint/src/path.rs` (deleted) | 6, 7 |
| 9  | **New crate scaffold — `quartzite-style-types`.** Add `quartzite-style-types` to workspace members in the root `Cargo.toml`. Create `quartzite-style-types/Cargo.toml` (`#![no_std]` + `extern crate alloc;` allowed via dep on `quartzite-paint-api` only — no `quartzite-paint` dep, no widget dep). Dependencies: `quartzite-paint-api = { path = "../quartzite-paint-api", default-features = false }`. Dev-dependencies: `rstest = "0.26"`. Create `quartzite-style-types/src/lib.rs` with the standard lint header (`#![no_std]`, `#![deny(missing_docs)]`, `#![warn(clippy::undocumented_unsafe_blocks)]`, `extern crate alloc;`) and module declarations. | root `Cargo.toml`, `quartzite-style-types/Cargo.toml`, `quartzite-style-types/src/lib.rs` | 4 (depends on `quartzite-paint-api::Color` only — though task 2 is a transitive dep, the explicit edge here is "the leaf crate exists and compiles") |
| 10 | **`quartzite-style-types::ColorRole` enum.** New file `quartzite-style-types/src/color_role.rs`. Variants: `Window`, `WindowText`, `Button`, `ButtonText`, `Base`, `Text`, `Highlight`, `HighlightedText`, `Link`, `LinkVisited`, `BrightText`. Derives: `Copy + Clone + Debug + PartialEq + Eq + Hash`. `#[non_exhaustive]` so future variants are additive. No `MetaEnum` derive (the role is internal to style and not currently a property type — revisit if widgets gain a `palette_role: ColorRole` property). A `pub const ALL: &[ColorRole] = &[ColorRole::Window, ..., ColorRole::BrightText];` constant (used by AC9's loop assertion and by `Palette::default()`). | `quartzite-style-types/src/color_role.rs`, `quartzite-style-types/src/lib.rs` | 9 |
| 11 | **`quartzite-style-types::Palette` type.** New file `quartzite-style-types/src/palette.rs`. `pub struct Palette { values: [Color; N] }` where `N = ColorRole::ALL.len()` (compile-time constant). Lookup `pub fn color(&self, role: ColorRole) -> Color` indexes by `role as usize`. Builder `pub fn with_role(self, role: ColorRole, color: Color) -> Palette` (returns owned, not `&mut self`, per spec). `Default::default()` installs Quartzite's default light-themed palette using the same constants from the deleted `quartzite-widgets/src/palette.rs` (`LIGHT_GRAY = 0.94`, `BUTTON_GRAY = 0.88`, `SELECTION_BLUE = (0.0, 0.47, 0.83, 1.0)`); every `ColorRole` variant gets a non-`TRANSPARENT` colour. The Link / LinkVisited / BrightText defaults pick visible blues and a pure white: `LINK_BLUE = Color::new(0.0, 0.42, 0.85, 1.0)`, `LINK_VISITED_PURPLE = Color::new(0.50, 0.20, 0.78, 1.0)`, `BrightText = Color::WHITE`. | `quartzite-style-types/src/palette.rs`, `quartzite-style-types/src/lib.rs` | 10 |
| 12 | **`quartzite-widgets`: drop local `Alignment`/`Font`/`Palette`; add re-exports.** Remove `Alignment` from `quartzite-widgets/src/enums.rs` (keep `FocusPolicy`, `SizePolicy`, `CursorShape`); remove the `Alignment` `MetaEnum` round-trip test from the same file (preserved in `quartzite-geometry/src/alignment.rs`). Delete `quartzite-widgets/src/font.rs` and `quartzite-widgets/src/palette.rs`. In `quartzite-widgets/Cargo.toml` add `quartzite-paint = { path = "../quartzite-paint" }` and `quartzite-style-types = { path = "../quartzite-style-types" }`. In `quartzite-widgets/src/lib.rs`: remove `pub mod font;`, `pub mod palette;`, the `pub use font::Font;`, the `pub use palette::Palette;`, and `Alignment` from the `pub use enums::{...}` line. Add `pub use quartzite_geometry::Alignment;`, `pub use quartzite_paint::Font;` (with `FontWeight`), and `pub use quartzite_style_types::{ColorRole, Palette};`. Update `quartzite-widgets/src/widget_base.rs` imports — replace `use crate::{..., Font, Palette, ...}` with paths through the re-exports (functionally unchanged because `crate::Font` / `crate::Palette` keep resolving). The `WidgetBase::new` body is unchanged: `Arc::new(Font::default())` and `Arc::new(Palette::default())` now resolve to the upstream types. **Critical: do NOT add `quartzite-style` to `quartzite-widgets`'s `Cargo.toml` — this is the cycle-break the leaf crate exists to enforce.** | `quartzite-widgets/Cargo.toml`, `quartzite-widgets/src/lib.rs`, `quartzite-widgets/src/enums.rs`, `quartzite-widgets/src/font.rs` (deleted), `quartzite-widgets/src/palette.rs` (deleted), `quartzite-widgets/src/widget_base.rs` (import-only changes; kept tests pass) | 8, 11 |
| 13 | **New crate scaffold — `quartzite-style`.** Add `quartzite-style` to workspace members in the root `Cargo.toml`. Create `quartzite-style/Cargo.toml` (no `#![no_std]` — the registry uses `std::sync::{Mutex, OnceLock}`). Dependencies: `quartzite-paint = { path = "../quartzite-paint" }`, `quartzite-paint-api = { path = "../quartzite-paint-api" }`, `quartzite-widgets = { path = "../quartzite-widgets" }` (for `&dyn AsWidget`), `quartzite-style-types = { path = "../quartzite-style-types" }`, `quartzite-geometry = { path = "../quartzite-geometry" }`. Dev-dependencies: `rstest = "0.26"`, `serial_test = "3"` (the registry tests share global state). Create `quartzite-style/src/lib.rs` with the standard lint header (no `#![no_std]`) and module declarations. Re-export `pub use quartzite_style_types::{ColorRole, Palette};` so callers see the full vocabulary in one place. | root `Cargo.toml`, `quartzite-style/Cargo.toml`, `quartzite-style/src/lib.rs` | 11, 12 |
| 14 | **`quartzite-style::Style` trait.** New file `quartzite-style/src/style.rs`. `pub trait Style: Send + Sync { fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette); }`. Object-safe (verified by an inline doctest constructing `Box<dyn Style>` from a unit-struct fixture with a single-method impl). The bound `Send + Sync` is required because the registry stores a `&'static dyn Style` reachable from any thread (`std::sync::Mutex` enforces nothing here — the bound on the trait does). Imports: `use quartzite_paint_api::Painter; use quartzite_widgets::AsWidget; use quartzite_style_types::Palette;`. | `quartzite-style/src/style.rs`, `quartzite-style/src/lib.rs` | 13 |
| 15 | **`quartzite-style::StyleRegistry`** (with `Box::leak`). New file `quartzite-style/src/registry.rs`. `pub struct StyleRegistry;` namespace. Internal storage: `static REGISTRY: OnceLock<Mutex<Option<&'static dyn Style>>> = OnceLock::new();`. Internal helper `fn registry() -> &'static Mutex<Option<&'static dyn Style>> { REGISTRY.get_or_init(\|\| Mutex::new(None)) }`. `pub fn set_style(style: Box<dyn Style>)` — calls `let leaked: &'static dyn Style = Box::leak(style);`, then locks via `lock().unwrap_or_else(\|e\| e.into_inner())` and replaces the `Option`. `pub fn try_style() -> Option<&'static dyn Style>` — locks via `unwrap_or_else(\|e\| e.into_inner())`, copies the `Option<&'static dyn Style>`, drops the guard, returns. The `Box::leak` happens **once per `set_style` call**; replacement leaks the previous box (acceptable for a process-lifetime registry). The `*Box<dyn Style>` is `Send + Sync` because the trait requires `Send + Sync`, so the leaked `&'static dyn Style` is also `Send + Sync` and the `Mutex` is well-formed. | `quartzite-style/src/registry.rs`, `quartzite-style/src/lib.rs` | 14 |
| 16 | **Doc + lint sweep, build-gate verification, AC13 cargo-tree assertion.** Every new public item across tasks 1–15 carries: one-line summary, `# Examples`, `# Parameters` (when ≥ 1 non-receiver arg). Recursive-simple fns marked `#[inline]` (concrete fn / inherent method) or `_Simple._` (generic / trait method declaration). Run the full gate: `cargo build`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`, `cargo build -p quartzite-paint-api --no-default-features`, `cargo build -p quartzite-geometry --no-default-features`, `cargo build -p quartzite-style-types --no-default-features`. **AC13 special check**: run `cargo tree -p quartzite-widgets --prefix none --no-dedupe` and assert `quartzite-style` does NOT appear in the output (only `quartzite-style-types` should). This is the cycle-break contract — both an automated CI step and a manual reviewer check. | (CI gate; all crates) | 1–15 |

Total: 16 tasks. Tasks 7 and 12 are the largest (multi-file edits across crate boundaries);
the rest are localised. The dependency graph is a DAG with no cross-edges except the
documented `style → widgets → style-types` chain.

> **Note on the cycle-break contract.** Round 1's design carried this same caveat
> incorrectly: it claimed widgets could re-export `Palette` from `quartzite-style` because
> "re-export-only deps are fine." That is wrong — Cargo resolves the dependency graph
> regardless of *how* a crate uses its dependencies, and a re-export is a real edge.
> Round 2 fixes this by making `Palette` live in a leaf crate (`quartzite-style-types`) so
> widgets re-exports from the leaf, not from `quartzite-style`. AC13 now mechanically
> asserts the cycle is broken.

## Risks

- **`quartzite-geometry` no_std preservation.** Adding `quartzite-macros` (proc-macro,
  host-only) and `quartzite-core` (with `default-features = false`) must not break
  `cargo build -p quartzite-geometry --no-default-features` (AC14). *Mitigation:*
  `quartzite-core` already has a `--no-default-features` build target in workspace CI;
  using it without `std` is a supported configuration. Task 16's gate runs the
  `--no-default-features` build for both `geometry` and the new `style-types` leaf.

- **`StyleRegistry::set_style` leaks the previous box.** Returning `&'static dyn Style`
  from `try_style` requires the box to live forever; replacing it via `set_style`
  necessarily leaks the old one. *Mitigation:* documented in the `set_style` rustdoc
  (`# Memory note` section per AGENTS.md library-safety idioms — explicitly call out the
  leak); bounded by user behaviour (typical apps swap styles zero or one times). A future
  plan can switch to `Arc<dyn Style>` with a `Box::leak`-free impl if real workloads
  emerge. Out of scope for v1.

- **Shared global state across tests.** `StyleRegistry` is a process-wide singleton; tests
  that mutate it must run serially. *Mitigation:* every test that touches `set_style`
  carries `#[serial]` from the `serial_test` crate (already in the workspace dev-deps and
  added explicitly to `quartzite-style/Cargo.toml`). Tests that only call `try_style()`
  cannot assume a prior state — they must run as part of the serial group too.

- **Mutex-poison recovery test reliability.** AC10 demands a poisoned-mutex scenario. The
  standard pattern (panic inside a `lock()` guard from a spawned thread, then `.join()`
  and observe `PoisonError` on next lock) is timing-insensitive but interacts with
  `OnceLock` initialisation. *Mitigation:* the test acquires the lock once via
  `try_style()` to force `OnceLock` initialisation, then uses a `#[cfg(test)] pub(crate)
  fn poison_for_test()` helper that spawns a thread which locks and panics. The
  recovery branch (`unwrap_or_else(|e| e.into_inner())`) is exercised on the next
  `try_style()` call.

- **`Painter` trait churn breaks `VelloPainter` and any external impls.** Adding four new
  required methods is a breaking change (no `default` impls). *Pre-publish — AGENTS.md
  § API Stability authorises clean breaks.* `VelloPainter` is updated atomically in task 7.
  External impls don't yet exist.

- **Removing `quartzite-widgets::Font`/`Palette` types breaks any downstream.** Same
  answer: pre-publish, AGENTS.md authorises. The replacements are re-exports from upstream
  crates; existing widget call sites (`label.rs`, `widget_base.rs`) compile unchanged
  because `crate::Font` and `crate::Palette` continue to resolve through the new
  `pub use` re-exports.

- **`Alignment` discriminant drift.** The move must preserve `Left=0`, `Center=1`,
  `Right=2`, `Justify=3` exactly (AC12). *Mitigation:* AC12 is the test, and the move is
  verbatim — copy the file content rather than retyping. Task 1 includes the round-trip
  test in `quartzite-geometry/src/alignment.rs`.

- **Doc-test coverage on new public items.** AC15 (`cargo doc -D missing-docs` clean) is
  workspace-wide. *Mitigation:* task 16 is a dedicated sweep; the doc-convention is
  well-established (every `#[derive]` enum and every public fn with ≥ 1 non-receiver arg
  gets `# Examples` + `# Parameters`).

- **`Path::arc_to` semantics ambiguous.** The spec lists `arc_to(centre, radii,
  start_angle, sweep_angle)`. *Mitigation:* design fixes the semantics to centre-and-radii
  (centre at `centre: Point`, semi-axes `radii: Size`, angles in radians, positive
  `sweep_angle` is CCW). This matches vello/peniko's arc model and is documented in the
  `Segment::ArcTo` rustdoc. The spec's AC5 requires only that arguments round-trip without
  loss, not that any backend rasterises them.

- **`quartzite-widgets` adds two upstream deps in one PR.** Task 12 adds both
  `quartzite-paint` (for `Font`) and `quartzite-style-types` (for `Palette`/`ColorRole`).
  *Mitigation:* both are upstream of widgets in the dependency graph; no new cycle is
  introduced. The atomic Cargo.toml edit is small (two added lines) and task 16's
  `cargo tree` assertion confirms no cycle.

- **Cross-file edit atomicity in task 7.** Updating `Painter` (in `paint-api`) and
  `VelloPainter` (in `renderer`) must happen in one commit, or `cargo build --workspace`
  fails between commits. *Mitigation:* documented in the task description; reviewer
  catches a split via the cargo build failure on the intermediate commit.

- **`Send + Sync` bound on `Style`.** Required by the registry (`&'static dyn Style`
  shared across threads). Adding `Send + Sync` to a public trait is a hard requirement on
  every implementor. *Mitigation:* concrete `Style` implementations are workspace-internal
  in v1; pre-publish API stability authorises the bound. Documented in `Style`'s rustdoc.

## Test Design

For each AC the corresponding test lives in the file marked **(AC#)** below. The test name
is given as a `snake_case` Rust ident.

### AC1 — `Color::with_alpha` zeroes alpha, preserves channels

- Location: `quartzite-paint-api/src/color.rs` `#[cfg(test)] mod tests`.
- Test: `with_alpha_zero_makes_transparent` — `assert_eq!(Color::RED.with_alpha(0.0).a(), 0.0); assert_eq!(Color::RED.with_alpha(0.0).r(), 1.0);`.
- Test: `with_alpha_preserves_other_channels` — `Color::new(0.1, 0.2, 0.3, 1.0).with_alpha(0.5)` equals `Color::new(0.1, 0.2, 0.3, 0.5)`.
- Test: `with_alpha_quarter_keeps_red` — AC1's exact wording: `Color::RED.with_alpha(0.25).r() == 1.0`.

### AC2 — `Color::with_alpha` is `const fn` returning `Color`

- Location: `quartzite-paint-api/src/color.rs` `#[cfg(test)] mod tests`.
- Test: `with_alpha_is_const` — assigned to `const C: Color = Color::RED.with_alpha(0.25);` to force const evaluation. The test asserts `assert_eq!(C.a(), 0.25);`.
- Doctest in the rustdoc `# Examples` block confirms callable-from-const.

### AC3 — `Pen::default()` invariants preserved

- Location: `quartzite-paint-api/src/pen.rs` `#[cfg(test)] mod tests` (existing
  `default_is_black_one_pixel` test already covers `width == 1.0` and `color == BLACK`).
- *No new test required* — the existing test already passes; AC3 is a regression fence.
  Task 16's `cargo test --workspace` re-runs it. The design treats AC3 as a *non-regression*
  contract: any task touching `Pen` (none in this plan) must keep it green.
- *Cross-task safety:* tasks 1–15 do not edit `pen.rs`. The existing test guards against
  drift.

### AC4 — `Path` `move_to → line_to → close` round-trip

- Location: `quartzite-paint-api/src/path.rs` `#[cfg(test)] mod tests`.
- Test: `move_then_line_then_close_round_trips` — Build
  `Path::new().move_to(p0).line_to(p1).close()` and assert the slice has 3 elements with
  discriminants `MoveTo`, `LineTo`, `Close` in order via `assert_matches!`.

### AC5 — `Path` `cubic_to + arc_to` round-trip

- Location: `quartzite-paint-api/src/path.rs` `#[cfg(test)] mod tests`.
- Test: `cubic_and_arc_round_trip` — `Path::new().cubic_to(c1, c2, p).arc_to(centre,
  radii, 0.0, core::f32::consts::PI)` and assert each segment matches the input via
  `assert_matches!` on `Segment::CubicTo` and `Segment::ArcTo { centre, radii,
  start_angle, sweep_angle }` exactly.
- Supplementary tests: `empty_path_returns_empty_slice`, `builder_returns_mut_self`
  (compile-only — `let mut p = Path::new(); p.move_to(...).line_to(...);`),
  `path_default_is_empty`.

### AC6 — `Font::new("Arial", 12.0)` defaults

- Location: `quartzite-paint-api/src/font.rs` `#[cfg(test)] mod tests`.
- Test: `new_default_weight_normal_and_flags_off` — `Font::new("Arial", 12.0)` returns
  `family() == "Arial"`, `size_pt() == 12.0`, `weight() == FontWeight::Normal`, `italic()
  == false`, `underline() == false`, `strikethrough() == false`.
- Supplementary tests: `default_is_sans_serif_12pt` (`Font::default().family() ==
  "sans-serif" && size_pt() == 12.0`), `font_weight_default_is_normal` (`FontWeight::default()
  == FontWeight::Normal`), `font_weight_numeric_value` (e.g. `FontWeight::Bold as u16 ==
  700`), `font_clone_round_trip`.

### AC7 — `Image::try_new` validation

- Location: `quartzite-paint-api/src/image.rs` `#[cfg(test)] mod tests`.
- Test: `try_new_accepts_correct_length` — `Image::try_new(2, 2, vec![0u8; 16])` returns
  `Ok` with `width()==2`, `height()==2`, `pixels().len()==16`.
- Test: `try_new_rejects_short_buffer` — `Image::try_new(2, 2, vec![0u8; 15])` returns
  `Err(ImageError::PixelLengthMismatch { expected: 16, actual: 15 })` (matched via
  `assert_matches!`).
- Supplementary tests: `try_new_rejects_long_buffer`, `try_new_zero_zero_empty`
  (`Image::try_new(0, 0, vec![])` returns `Ok`), `image_error_display` (Display text
  exists; `ImageError` derives `thiserror::Error`).

### AC8 — `Painter` trait remains object-safe through new methods

- Location: `quartzite-paint-api/src/painter.rs` `#[cfg(test)] mod tests`.
- Test: `painter_is_object_safe` (existing test, **expanded**) — `Box<dyn Painter>`
  constructible from `RecordingPainter`; all 11 methods callable through both `&mut dyn
  Painter` and `Box<dyn Painter>`.
- Test: `all_methods_reachable_through_trait_object` (existing test, **expanded**) —
  `RecordingPainter::calls` is now `[u8; 11]`; assert each method records exactly one call:
  `[1; 11]`.
- Test: `mock_painter_satisfies_trait` — minimal `struct Mock;` impl with empty bodies for
  every method compiles, verifying the trait remains object-safe even with the four new
  methods.

### AC9 — Default `Palette` is non-transparent for every `ColorRole`

- Location: `quartzite-style-types/src/palette.rs` `#[cfg(test)] mod tests`.
- Test: `default_palette_every_role_non_transparent` — `for role in ColorRole::ALL { assert_ne!(Palette::default().color(*role), Color::TRANSPARENT, "role {:?} is transparent", role); }`. Loop-driven per AC9's explicit wording ("loop-driven assertion, not per-role enumeration").
- Supplementary tests: `default_palette_window_is_set` (`Palette::default().color(ColorRole::Window) != Color::TRANSPARENT`), `with_role_replaces_only_target` (`Palette::default().with_role(ColorRole::Window, Color::RED).color(ColorRole::Window) == Color::RED`; other roles unchanged via spot-check).

### AC10 — `StyleRegistry` initial-`None`, set-then-`Some`, poison recovery

- Location: `quartzite-style/src/registry.rs` `#[cfg(test)] mod tests`. All tests carry
  `#[serial]` from `serial_test`.
- Test: `try_style_none_or_some_initially_then_set_returns_some` — Because the registry is
  process-global, "initially `None`" is asserted only by the *first* serial test in the
  module: a single test that (a) acquires the lock and clears it via a
  `#[cfg(test)] pub(crate) fn clear_for_test()` helper, (b) asserts `try_style().is_none()`,
  (c) `set_style(Box::new(NullStyle))`, (d) asserts `try_style().is_some()`. Combining
  initial-None and set-then-Some into one serial test avoids ordering fragility.
- Test: `set_replaces_previous` — `clear_for_test(); set_style(Box::new(StyleA)); set_style(Box::new(StyleB));` then assert the second `try_style()` returns the latter (compared by a marker method or `Any::type_id` after downcast).
- Test: `poison_recovery` — Acquire `try_style()` once to force `OnceLock` init.
  `clear_for_test();` then spawn a thread that calls a `pub(crate) fn poison_for_test()`
  helper which locks and panics. `.join()` returns `Err`. Subsequent `try_style()` returns
  `None` (or `Some(_)` if pre-set) without panicking — the `unwrap_or_else(|e|
  e.into_inner())` recovery is exercised. Asserts no panic and no deadlock.
- Fixture: `struct NullStyle;` with `impl Style for NullStyle { fn draw_widget(&self, _w: &dyn AsWidget, _p: &mut dyn Painter, _pal: &Palette) {} }`. Two marker variants `StyleA`, `StyleB` for the replacement test.

### AC11 — `Style` trait satisfied by `draw_widget` only

- Location: `quartzite-style/src/style.rs` `#[cfg(test)] mod tests`.
- Test: `style_trait_object_safe_with_only_draw_widget` — Define `struct OnlyDraw;` with `impl Style for OnlyDraw { fn draw_widget(&self, _: &dyn AsWidget, _: &mut dyn Painter, _: &Palette) {} }`; construct `Box<dyn Style>` from it. Compiling the impl proves no other required methods exist.
- Doctest in `pub trait Style` rustdoc demonstrating the same single-method pattern.

### AC12 — `quartzite-geometry::Alignment` discriminants + property round-trip

- Location: `quartzite-geometry/src/alignment.rs` `#[cfg(test)] mod tests`.
- Test: `alignment_default_is_left` — `Alignment::default() == Alignment::Left`.
- Test: `alignment_into_value_round_trip` — `Alignment::Center.into_value() == Value::Int(1)` and `Alignment::from_value(Value::Int(1)) == Ok(Alignment::Center)`. Uses `quartzite_core::{IntoValue, FromValue, Value}` (the `quartzite-core` dep added in task 1 makes these importable).
- Test: `discriminant_values_match_spec` — `Alignment::Left as i64 == 0`, `Alignment::Center as i64 == 1`, `Alignment::Right as i64 == 2`, `Alignment::Justify as i64 == 3`.

### AC13 — Widgets re-exports the upstream types; `quartzite-style` not a dep

- Location: integration test `quartzite-widgets/tests/re_exports.rs` (new file).
- Test: `widgets_alignment_is_geometry_alignment` — `assert_eq!(TypeId::of::<quartzite_widgets::Alignment>(), TypeId::of::<quartzite_geometry::Alignment>());`.
- Test: `widgets_font_is_paint_font` — `assert_eq!(TypeId::of::<quartzite_widgets::Font>(), TypeId::of::<quartzite_paint::Font>());`.
- Test: `widgets_palette_is_style_types_palette` — `assert_eq!(TypeId::of::<quartzite_widgets::Palette>(), TypeId::of::<quartzite_style_types::Palette>());`. **Critically uses `quartzite-style-types`, not `quartzite-style`** — proves widgets sees the leaf type, not the downstream re-export.
- Test: `widgets_color_role_is_style_types` — same TypeId check for `ColorRole`.
- **CI step (AC13's mechanical assertion):** `cargo tree -p quartzite-widgets --prefix none --no-dedupe | grep -v '^quartzite-style\b' && ! cargo tree -p quartzite-widgets --prefix none --no-dedupe | grep -q '^quartzite-style v'`. (The first half is presence of any other crate; the second half asserts `quartzite-style` (not the leaf `style-types`) is absent. The exact shell incantation lives in task 16's CI gate.)
- All existing `widget_base::tests` continue to pass without changes.

### AC14 — `--no-default-features` builds for `paint-api` and `geometry`

- Location: CI gate (`.github/workflows/*.yml`) and task 16's manual gate.
- Commands: `cargo build -p quartzite-paint-api --no-default-features`, `cargo build -p quartzite-geometry --no-default-features`. Plus the new `cargo build -p quartzite-style-types --no-default-features` (the leaf is `no_std + alloc`).
- Failure mode caught: any production-code use of `std`-only items in the three `no_std`
  crates fails the build. Tasks 2 (alloc ungate), 4 (Font with String), 5 (Image with Vec),
  6 (Path with Vec) all use `alloc::*` paths which are valid in `no_std + alloc`.

### AC15 — `cargo doc -D warnings -D missing-docs --workspace` clean

- Location: CI gate.
- Command: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`.
- Failure mode caught: any new public item without `///` docs, any broken intra-doc link
  (the `#![deny(rustdoc::broken_intra_doc_links)]` lint is already on every crate header).

### AC16 — `cargo clippy --workspace -- -D warnings` clean

- Location: CI gate.
- Command: `cargo clippy --workspace -- -D warnings`.
- Failure mode caught: any new clippy-flagged lint (e.g. missing `#[inline]` on a small
  method, `or_fun_call`, `redundant_closure`).

### Cross-task fixtures

- `RecordingPainter` in `quartzite-paint-api/src/painter.rs` — extended to 11 counters.
- `NullStyle`, `StyleA`, `StyleB` in `quartzite-style/src/registry.rs` `#[cfg(test)]`
  module — minimal `Style` impls for registry tests.
- `clear_for_test()` and `poison_for_test()` in `quartzite-style/src/registry.rs` —
  `#[cfg(test)] pub(crate) fn` helpers used only by the registry tests.
- `serial_test::serial` attribute on every `StyleRegistry` test (process-wide singleton).
- `assert_matches!` from `core::assert_matches` (or the `assert_matches` crate if not
  yet stabilised in the project's MSRV) for `Segment` round-trip checks.
- `core::any::TypeId` for AC13's TypeId equality checks.

## Open questions

- **Should `quartzite-geometry` add `quartzite-core` as a hard dep?** The spec lists only
  `quartzite-macros` for the `MetaEnum` derive. The macro's `crate_root()` helper
  (`quartzite-macros/src/util.rs:76`) resolves to `::quartzite_core` when neither the
  `quartzite` facade nor `quartzite-core` is found in the dependency graph — meaning the
  generated code refers to a crate that isn't there and the build fails. This design
  treats it as a mechanical consequence (yes, both deps required, with `quartzite-core`
  using `default-features = false` to preserve `no_std`). Asking for confirmation that the
  spec didn't intend either (a) a different macro family for geometry's `Alignment` or (b)
  inlining the `EnumMeta`/`IntoValue`/`FromValue` traits into geometry. Default is "add
  both deps" per the analysis above.

- **Should `style()` (panicking accessor) ship in v1?** The spec says "may exist alongside;
  design phase decides." The design rejects it for YAGNI: `try_style().expect("style not
  installed")` at call sites is one extra line and keeps the panic surface explicit.
  Re-asking the product owner before close-out.

- **Should `Path`/`Image`/`Font` impl `PartialEq`?** Spec is silent. `Path` and `Image`
  carry `Vec`s — derive is straightforward. `Font` derive is also fine (`String` + `f32`
  + bools). Default to `derive(PartialEq)` on all three for `assert_eq!` ergonomics; ask
  if there's a reason not to.

- **Image error handling — overflow vs. length-mismatch.** The spec says
  `pixels.len() == (width * height * 4) as usize` is the validation. On 32-bit platforms,
  `width as usize * height as usize * 4` can overflow. The design adds an `Overflow`
  variant to `ImageError` to handle this case explicitly via `checked_mul`. Asking for
  confirmation; an alternative is to silently accept that very large dimensions aren't
  supported (panic-on-overflow in debug, wrap in release). Default is the explicit
  `Overflow` variant.

- **Send + Sync bound on `Style`.** Required by the registry (a `&'static dyn Style` is
  shared across threads). The bound goes on every implementor. Confirming this is
  acceptable as an API constraint — current `quartzite-runtime` queues callbacks via
  `Send + 'static` so the precedent exists; default is "yes, add the bound."

- **Light/dark `PaletteGroup`.** The spec § *quartzite-style-types* mentions
  "Light/dark variants are exposed via `PaletteGroup` rather than enum doubling." This
  design implements only `Palette` + `Default::default()` (light theme). `PaletteGroup`
  is an open question — its shape isn't pinned by any AC. Defer to a follow-up plan? The
  design treats `PaletteGroup` as out-of-scope for v1 and notes it explicitly so review
  doesn't surprise anyone.
