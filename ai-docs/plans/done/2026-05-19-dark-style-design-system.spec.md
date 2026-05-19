# Implement dark-style alongside DefaultStyle from design-system

**Source:** issue #488
**Date:** 2026-05-19
**Tracked in:** #488

## Scope

1. Materialise the dark theme that `design-system/README.md` § *Dark theme*
   and `design-system/colors_and_type.css` `[data-theme="dark"]` already
   specify in prose/CSS into runtime Rust as a `Palette`-shaped artefact
   in `quartzite-style-types`. The existing `DefaultStyle` from
   `quartzite-style` is reused unchanged against this dark palette — no
   new `Style` impl, no paint-logic changes.

2. Cover the 11 `ColorRole` slots with the exact RGBA values pinned in
   `design-system/README.md` § *Dark theme* table (also mirrored in
   `design-system/colors_and_type.css` lines 88–117):

   | Role | Dark seed |
   |---|---|
   | `Window`          | `#2B2B2B` |
   | `WindowText`      | `#E8E8E8` |
   | `Button`          | `#3C3C3C` |
   | `ButtonText`      | `#E8E8E8` |
   | `Base`            | `#1E1E1E` |
   | `Text`            | `#E8E8E8` |
   | `Highlight`       | `#1E90FF` (DodgerBlue) |
   | `HighlightedText` | `#FFFFFF` |
   | `Link`            | `#5BB0FF` |
   | `LinkVisited`     | `#C58AFF` |
   | `BrightText`      | `#FF6B6B` |

3. Surface dark-mode goldens for the snapshot suite at
   `quartzite-style/tests/snapshots/shared/` so the dark theme has the
   same visual coverage as `DefaultStyle` driven by `Palette::default`
   (10 light goldens today:
   `button_{idle,hovered,pressed,checked,focused,disabled}.png` +
   `label.png` + `scroll_area_chrome.png` +
   `text_edit_{plain,read_only}.png`). The dark variants use a
   `dark_<existing-name>.png` filename prefix under the same flat
   `shared/` directory.

4. Doc-comments on the new public surface; standard rustdoc/clippy/fmt
   gates plus the doc gate
   (`RUSTDOCFLAGS="-D warnings -D missing-docs"`); no-std / `libm`
   path keeps compiling (`cargo build -p quartzite --no-default-features
   --features libm`).

## Out of scope

- Adding a new `Style` impl. Round 1 nailed the artefact shape as
  *palette only* — `DefaultStyle` is reused against the dark palette
  rather than subclassed/duplicated.
- Light-theme regressions, new `ColorRole` variants, new palette
  invariants, or any change to `DefaultStyle`'s paint logic. The
  framework's *Dark theme* section explicitly states the dark theme
  re-uses every derivation formula unchanged.
- A theme-switching mechanism beyond what `StyleRegistry` /
  `Palette::with_role` already offer. No `on_theme_changed` signal, no
  OS-dark-mode detection, no runtime toggle widget.
- Light/dark snapshot **diffing** infrastructure. Goldens are committed
  per-image as today; cross-theme delta visualisation is not in scope.
- Icon assets, illustrations, fonts, or any visual asset additions
  beyond the colour-role values above. The framework remains
  icon-/image-free; `design-system/README.md` § *Iconography* is
  authoritative.
- `design-system/` content updates. The dark seeds are *already* there;
  this task consumes them, it does not re-derive or re-publish them.
- Dark-theme variants for `quartzite-widgets/tests/snapshots/` (the
  five widget-side goldens: `box_layout`, `button`, `grid_layout`,
  `label`, `line_edit`). Those exercise `WidgetExt::paint`, which is a
  no-op in the current architecture — the style-side suite is the
  representative visual coverage surface.

## Deferred

- OS-dark-mode auto-detection (e.g. via `winit` window theme events) —
  why: separate concern from theme definition itself; needs platform
  matrix. Separate issue needed? yes.
- Per-widget `Style` overrides / multi-style cascade — why: the current
  registry holds a single `&'static dyn Style`; a multi-style model is a
  larger architecture change. Separate issue needed? yes.
- Auto-switching `Palette::default` based on env var or `prefers-color-scheme`
  equivalent — why: would change the meaning of `Palette::default`,
  which the snapshot suite and 1454 existing tests pin to the light
  defaults. Separate issue needed? yes.
- Dark-mode goldens for `quartzite-widgets/tests/snapshots/` once
  `WidgetExt::paint` actually paints — why: today those tests render
  empty chrome because the widget-side paint is a no-op; revisiting
  belongs with whichever task lights up widget-side painting. Separate
  issue needed? probably (file when widget paint exists).

## Key decisions

| Question | Decision |
|---|---|
| API shape of the new dark surface | **Palette only.** Add a dark `Palette` artefact in `quartzite-style-types`; reuse `DefaultStyle` against it. No new `Style` impl, no new crate, no changes to `quartzite-style`. (Round 1 Q1.) |
| Concrete API form: `pub const DARK_PALETTE` vs `Palette::dark() -> Self` `const fn` | Default to `pub const DARK_PALETTE: Palette` in `quartzite-style-types`, mirroring the `Color::WHITE` / `Color::BLACK` named-seed precedent and keeping the call site to a single identifier reference. Both forms are equally `const`-evaluable; design agent may justify the `Palette::dark()` form instead if it yields better cross-crate ergonomics, but the spec defaults to the constant. |
| Source of truth for the 11 dark-slot RGBA values | `design-system/README.md` § *Dark theme* table + `design-system/colors_and_type.css` `[data-theme="dark"]` block. Both already agree; this task pulls the numbers into Rust. |
| Palette invariants the dark seeds must satisfy | (1) every role non-transparent; (2) `Highlight` ≠ `HighlightedText` — same as `Palette::default`. New unit tests mirror `quartzite-style-types::palette::tests` for the dark constructor. |
| Derived state formulae (hover blend / disabled α / focus outline / read-only overlay) | Unchanged. `DefaultStyle::paint` already computes them at paint time; the dark seeds re-use the same formulae per `design-system/README.md` § *Derived state values follow the framework's same formulas*. |
| Crate placement | `quartzite-style-types` — the dark palette is a pure `Palette` value, and `quartzite-style-types` is the leaf crate where `Palette` lives. `quartzite-style` requires zero changes. |
| Snapshot file naming convention | `dark_<existing-name>.png` (e.g. `dark_button_idle.png`) under `quartzite-style/tests/snapshots/shared/`, mirroring the existing flat layout. Avoids a subdirectory split that would require `support/mod.rs` changes. |
| Snapshot test count | One per existing `DefaultStyle` golden (10 dark variants) so dark coverage matches light coverage exactly. |
| `StyleRegistry` / `Palette::default` semantics | Untouched. The dark theme is opt-in via the new dark palette; `Palette::default` remains the light seed, and `StyleRegistry` remains an explicit `set_style` call. |

## Technical constraints

- `Palette` is `const`-constructible (`Palette::new()` is `const fn`,
  `with_role` is `const fn`) — the new dark constant must therefore be
  expressible as a `Palette::new().with_role(...).with_role(...)`
  chain evaluated at compile time, per the `const-default-fn` task
  convention.
- `Color::new(r, g, b, a)` is `const fn`; hex-to-linear conversion must
  be done by hand (e.g. `0x2B / 255.0 ≈ 0.169`) as no `Color::from_hex`
  exists. Match the 3-decimal-place precision used in the
  `design-system/README.md` § *Dark theme* worked Rust example
  (`Color::new(0.169, 0.169, 0.169, 1.0)` for `#2B2B2B`).
- `Color::WHITE` (`#FFFFFF`) is already a `pub const`; the dark seed
  for `HighlightedText` reuses it verbatim.
- Snapshot pipeline already supports per-PNG goldens under
  `quartzite-style/tests/snapshots/shared/` and per-backend overrides
  under `<backend>/`; new dark goldens use the same harness
  (`harness_or_skip` + `snapshot_assert`) and the same
  `scripts/update-snapshots.sh --crate style` regeneration path. No
  helper changes anticipated.
- The dark goldens are driven by `DefaultStyle` paired with the new
  dark palette. The design agent decides the exact wiring (e.g.
  parameterising the existing snapshot fixtures with a `Palette` arg vs
  a thin per-test helper that swaps the palette before paint) — both
  paths are open as long as `support/mod.rs` stays untouched.
- No new dependencies. `quartzite-style-types` and `quartzite-style`
  already pull everything required (`quartzite-paint-api::Color`,
  `ColorRole`, `Palette`).
- AGENTS.md *Design system* trigger conditions 1, 3, and 4 all fire
  for this task (any `Style` impl incl. `DefaultStyle`; `Palette` /
  `ColorRole` seeds; snapshot tests under
  `quartzite-style/tests/snapshots/`) — design agent must Read
  `design-system/README.md` § *VISUAL FOUNDATIONS* and § *Dark theme*
  + `design-system/colors_and_type.css` before designing.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-style-types` exposes a `pub const DARK_PALETTE: Palette` (or, if the design agent justifies it, a `pub const fn Palette::dark() -> Self`) whose 11 `ColorRole` slots match the RGBA values pinned in `design-system/README.md` § *Dark theme*. Either form is callable from application code in ≤ 2 lines. |
| AC2 | The new dark palette satisfies both `Palette` invariants: every role is non-transparent, and `Highlight ≠ HighlightedText`. Two unit tests in `quartzite-style-types::palette::tests` mirror the existing `default_has_non_transparent_color_for_every_role` and `default_highlight_differs_from_highlighted_text` against the dark palette. |
| AC3 | The new dark palette is usable from a `const` context — a `const _: Palette = DARK_PALETTE;` (or `Palette::dark()` if that form is chosen) compiles, exercising the `const fn` chain at compile time with zero runtime overhead. |
| AC4 | `Palette::default()` returns the unchanged **light** seed values; no existing test is altered to accommodate the dark theme; the `default_*` tests in `quartzite-style-types::palette::tests` pass without modification. |
| AC5 | 10 dark-variant golden PNGs land under `quartzite-style/tests/snapshots/shared/dark_*.png`, one per existing `DefaultStyle` golden (`dark_button_{idle,hovered,pressed,checked,focused,disabled}.png`, `dark_label.png`, `dark_scroll_area_chrome.png`, `dark_text_edit_{plain,read_only}.png`). |
| AC6 | 10 new `#[test]` fns in `quartzite-style/tests/snapshots.rs` (or a sibling file) drive `DefaultStyle` against the new dark palette and assert against the AC5 goldens via the existing `snapshot_assert` helper. No changes to `quartzite-style/tests/support/mod.rs` (the snapshot-helper sync group stays untouched). |
| AC7 | New public items carry rustdoc per `ai-docs/doc-convention.md`: one-line summary, `# Examples` block on single-line-doc items, intra-doc-links to `Palette` / `ColorRole` / `DefaultStyle`. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` exits 0. |
| AC8 | `cargo test --workspace` exits 0; `cargo clippy --workspace --all-targets -- -D warnings` exits 0; `cargo fmt -- --check` exits 0; `cargo build -p quartzite --no-default-features --features libm` exits 0. |
| AC9 | `design-system/colors_and_type.css` `[data-theme="dark"]` block and `design-system/README.md` § *Dark theme* table are unchanged by this PR (they are the spec, not the deliverable). `quartzite-style/src/` is unchanged except for any `pub use` re-export of the new dark palette name if the design agent picks that route; `DefaultStyle` and its paint logic are untouched. |

## Open questions

- `pub const DARK_PALETTE: Palette` vs `pub const fn Palette::dark() -> Self`.
  Spec defaults to the constant (mirrors `Color::WHITE`/`Color::BLACK`,
  one-identifier reference at call site); design agent may pick the
  `const fn` form if cross-crate ergonomics or future palette-family
  parity argues otherwise. Either way AC1 / AC3 are satisfied.
- Will an app ever want *both* the light and dark palettes registered
  simultaneously (e.g. per-window theme), or is single-active sufficient?
  Sensible default: single-active (registry holds one style; the chosen
  palette is selected at startup). Revisit if multi-window-per-theme
  becomes a requirement.
- Should the dark goldens go under a `dark/` subdir
  (`shared/dark/dark_button_idle.png`) instead of a flat `dark_` prefix?
  Defaulted to flat-prefix in *Key decisions* to avoid touching
  `support/mod.rs`; revisit if the per-theme count grows.
