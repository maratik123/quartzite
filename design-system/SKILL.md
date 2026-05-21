---
name: quartzite-design
description: Use this skill to generate well-branded interfaces and assets for Quartzite, either for production or throwaway prototypes/mocks/etc. Contains essential design guidelines, colors, type, fonts, assets, and UI kit components for prototyping.
user-invocable: true
---

Read the `README.md` file within this skill, and explore the other available files (`colors_and_type.css`, `preview/`, `assets/`, `ui_kits/widgets/`).

If creating visual artifacts (slides, mocks, throwaway prototypes, etc), copy assets out and create static HTML files for the user to view. If working on production code, you can copy assets and read the rules here to become an expert in designing with this brand.

If the user invokes this skill without any other guidance, ask them what they want to build or design, ask some questions, and act as an expert designer who outputs HTML artifacts _or_ production code, depending on the need.

## Quick reference

- **Palette source of truth:** `quartzite-style-types::Palette::default` — see `quartzite-style-types/src/palette.rs` and `dark_palette.rs` in the upstream repo.
- **Visual rules:** see _VISUAL FOUNDATIONS_ in `README.md`. Flat fills, 1 px outlines, 0 px radii, no shadows, no animations.
- **State derivations:** fill/text precedence `disabled > pressed > checked > hovered > idle`. Hover and Pressed are derived per `c.blend(WindowText.Normal, t)` with `t = 0.06 / 0.16` (`ColorGroup` axis: `Normal` / `Hover` / `Pressed`). Disabled = α × 0.5. Focus = additive 2 px `ColorRole::FocusRing` outline (defaults to `Highlight`).
- **Iconography:** there are no icons in the framework. Use Lucide as a documentation-time substitute and flag it. Never emoji.
- **Type:** `Font::default()` = `sans-serif` 12 pt Normal. Mocks use a DejaVu Sans / Liberation Sans stack.
- **UI kit:** `ui_kits/widgets/` — React/JSX recreations of `Button`, `Label`, `LineEdit`, `TextEdit`, `ScrollArea`, `Container`, plus a `WindowFrame` shell. Import as `<script type="text/babel" src="…">` after React + Babel standalone.
- **Snapshots reference:** `quartzite-style/tests/snapshots/shared/*.png` in the upstream repo — committed golden images for every `DefaultStyle` paint state.
- **Designer app assets:** `assets/quartzite-designer-*.svg` — sibling-product mark + wordmark, building on the canonical Quartzite mark with a 2 px `Highlight` ring on the back-right facet. Showcases at `Quartzite Designer Logo.html` and `Quartzite Designer Assets.html`.

## Source repository

<https://github.com/maratik123/quartzite> (master). When in doubt about a paint rule, read `quartzite-style/src/default_style.rs`. When in doubt about a widget's props/signals, read `quartzite-widgets/src/widgets/*.rs`.
