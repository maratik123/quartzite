---
name: quartzite-design
description: Use this skill to generate well-branded interfaces and assets for Quartzite, either for production or throwaway prototypes/mocks/etc. Contains essential design guidelines, colors, type, fonts, assets, and UI kit components for prototyping.
user-invocable: true
---

Read the `README.md` file within this skill, and explore the other available files (`colors_and_type.css`, `preview/`, `assets/`, `ui_kits/widgets/`).

If creating visual artifacts (slides, mocks, throwaway prototypes, etc), copy assets out and create static HTML files for the user to view. If working on production code, you can copy assets and read the rules here to become an expert in designing with this brand.

If the user invokes this skill without any other guidance, ask them what they want to build or design, ask some questions, and act as an expert designer who outputs HTML artifacts _or_ production code, depending on the need.

## Quick reference

- **Palette source of truth:** `colors_and_type.css` (mirrors `quartzite-style-types::Palette::default`).
- **Visual rules:** see _VISUAL FOUNDATIONS_ in `README.md`. Flat fills, 1 px outlines, 0 px radii, no shadows, no animations.
- **State derivations:** `pressed > checked > hovered > idle`; disabled = α × 0.5; focus = additive 2 px Highlight outline.
- **Iconography:** there are no icons in the framework. Use Lucide as a documentation-time substitute and flag it. Never emoji.
- **Type:** `Font::default()` = `sans-serif` 12 pt Normal. Mocks use a DejaVu Sans / Liberation Sans stack.
- **UI kit:** `ui_kits/widgets/` — React/JSX recreations of `Button`, `Label`, `LineEdit`, `TextEdit`, `ScrollArea`, `Container`, plus a `WindowFrame` shell. Import as `<script type="text/babel" src="…">` after React + Babel standalone.
- **Snapshots reference:** `assets/snapshots/*.png` — committed golden images from the framework's `quartzite-style/tests/snapshots/shared/`.

## Source repository

<https://github.com/maratik123/quartzite> (master). When in doubt about a paint rule, read `quartzite-style/src/default_style.rs`. When in doubt about a widget's props/signals, read `quartzite-widgets/src/widgets/*.rs`.
