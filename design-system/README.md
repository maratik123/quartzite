# Quartzite Design System

A design system for **quartzite** — a Rust GUI and object framework
modeled on Qt's signals/slots and property/reflection model, but
implemented in idiomatic Rust with no FFI, no native dependencies,
and no codegen outside proc-macros.

> Quartzite is **design lineage from Qt, not API surface**. The
> visual vocabulary here is what falls out of `DefaultStyle` —
> a flat, no-chrome look meant as a starting point, not a polish
> pass.

## Sources

- **GitHub repo:** <https://github.com/maratik123/quartzite> (master, at commit `9eebb2a1e938`).
  - `quartzite-style-types` — the `ColorRole` enum + `Palette` lookup table that defines the entire color vocabulary.
  - `quartzite-style/src/default_style.rs` — the only concrete `Style` in-tree; every visual rule in this doc comes from it.
  - `quartzite-widgets/src/widgets/*` — the six built-in widgets (`Button`, `Label`, `LineEdit`, `TextEdit`, `ScrollArea`, `Container`).
  - `quartzite-style/tests/snapshots/shared/*.png` — committed golden PNGs of every widget state under `DefaultStyle`. Read these in the upstream repo when verifying paint output against `DefaultStyle`.
  - `quartzite-paint-api/src/{color,font,brush,pen,path,image,painter}.rs` — the painting primitives the style draws against.

The reader is encouraged to explore the repository — especially `quartzite-widgets` and `quartzite-style` — to recreate or extend designs faithfully. Most decisions in this design system are **described** rather than re-derived; the source of truth is the Rust code.

## What's in here

A short manifest of the root:

| Path | What it is |
|---|---|
| `README.md` | This file. Brand context, content fundamentals, visual foundations, iconography. |
| `SKILL.md` | Agent-Skills-compatible entry point so this folder can be loaded as a skill. |
| `fonts/` | (empty — see _Typography_ below.) |
| `assets/` | Logos and snapshot pngs. |
| `assets/quartzite-mark.svg` | 64 px crystalline mark. |
| `assets/quartzite-mark-dark.svg` | Same mark, dark-theme facets, no background. |
| `assets/quartzite-mark-white.svg` | Same mark with a 3 px white halo for colored backgrounds. |
| `assets/quartzite-wordmark.svg` | Mark + `quartzite` lockup. |
| `assets/quartzite-designer-mark.svg` | Sibling-product mark — canonical mark + 2 px `Highlight` ring on the back-right facet. |
| `assets/quartzite-designer-mark-dark.svg` | Designer mark · dark · transparent. |
| `assets/quartzite-designer-mark-white.svg` | Designer mark · 3 px white halo. |
| `assets/quartzite-designer-wordmark.svg` | Designer primary lockup · mark + stacked `quartzite` / `DESIGNER` accent. |
| `assets/quartzite-designer-wordmark-dark.svg` | Designer primary lockup · dark. |
| `Quartzite Designer Logo.html` | Designer logo system showcase — primary lockup, alternate stacked lockup, application icon, construction grid. |
| `Quartzite Designer Assets.html` | Final asset preview — all SVG variants + raster ladder. |
| `preview/` | 24 cards populating the Design System tab (Brand / Colors / Type / Spacing / Components). |
| `preview/card-base.css` | Shared chrome + widget-mock CSS for every preview card. |
| `ui_kits/widgets/` | HTML/JSX recreation of the six built-in widgets + an interactive demo (title-bar toggle flips light ↔ dark). |
| `ui_kits/widgets/index.html` | Working `quartzite-widgets` demo: Counter + new-note form + Signal log. |
| `ui_kits/widgets/{Button,Label,LineEdit,TextEdit,ScrollArea,Container,WindowFrame,App}.jsx` | Per-widget React components mirroring real props + signals. |
| `proposals/text-edit-read-only-overlay.md` | Proposed framework fix for the invisible read-only overlay on `Palette::default`. |
| `proposals/caret-and-selection.md` | Caret + selection visual spec for `LineEdit` and `TextEdit`. Unblocks #405, #317. |
| `proposals/scrollbar.md` | `ColorRole::ScrollBar` + track / thumb geometry for `ScrollArea`. Unblocks #315. |
| `proposals/cursor-shapes.md` | Per-widget cursor defaults + state overrides + endorsed `winit::CursorIcon` subset. Unblocks #404. |
| `proposals/popups-and-tooltips.md` | Popup + tooltip chrome + `ZLayer` paint pass; keeps the no-shadow rule. Unblocks #408. |
| `preview/comp-line-edit-caret.html` | Caret geometry + blink phases. |
| `preview/comp-line-edit-selection.html` | Single-line selection states (focused / unfocused / read-only / disabled). |
| `preview/comp-text-edit-selection.html` | Multi-line wrapped selection geometry. |
| `preview/comp-popup.html` | Popup chrome, row states, `ZLayer` stack. |
| `preview/comp-tooltip.html` | Tooltip chrome, inversion, wrap. |

## Brand context

The project is a one-author, early-stage Rust GUI framework. The
crates and surface are still small — `Application` + event loop,
six widgets, two layouts (`BoxLayout`, `GridLayout`), a paint API,
and a single `DefaultStyle`. Visual identity flows directly from
the code; there is no separate brand book.

- **Audience:** Rust developers exploring native GUI options outside of FFI bindings (gtk-rs, qt-rs, etc.) and outside of immediate-mode (egui).
- **Voice:** terse, technical, accurate. The README begins with a one-sentence positioning statement and a feature checklist.
- **Visual lineage:** Qt 4/5 platform-flat look. Sharp corners, 1 px strokes, primary action = sky blue (`#0080FF`), text = pure black on pure white.

## CONTENT FUNDAMENTALS

Documentation is **the** content surface for Quartzite. Tone notes
come from reading `README.md`, `ROADMAP.md`, `CONTRIBUTING.md`,
crate-level rustdoc, and inline doc comments.

- **Register:** technical, declarative, no marketing. The lead sentence describes the artifact in 2–3 clauses — what it is, what it draws on, what it is not.
- **Sentence shape:** short. Em-dashes link a name to its definition. Examples below are paraphrased from the repo:
  - "A GUI and object framework for Rust drawing on Qt's signals/slots and property/reflection model."
  - "`Palette` stores one `Color` per role in a fixed-size array indexed by `ColorRole as usize`."
- **Pronoun stance:** no "you", no "we". The reader is implied. Doc comments describe behavior, not advice. Imperatives appear only in build instructions ("Add `quartzite` to your `Cargo.toml`").
- **Casing:** `Title Case` for headings; `Sentence case for sub-points.`; identifiers stay verbatim (`ObjectBase`, `WidgetView::Other`). Acronyms uppercase (`GUI`, `RGBA`, `CSS`).
- **Type names always live inside backticks** when prose mentions them — never bare. Same for module paths (`quartzite-style/src/default_style.rs`).
- **Emoji:** none. Not in README, not in code comments, not in rustdoc. The closest the repo gets is `✅` in a status table — and that is the only emoji-like glyph that appears.
- **Numerical precision:** units are written out. `1 px outline`, `12.0 pt`, `64×64 canvas`, `1.95` (the MSRV). Hex codes follow the `#RRGGBB` convention; alpha is mentioned in prose, not in the literal.
- **Negation:** "Non-goals" are an explicit section. The system says what it _won't do_ as plainly as what it will (e.g. "Not a Qt port or a Qt binding"; "No FFI / native dependencies").
- **Vibe:** rust-stdlib-meets-Qt-API-docs. Dry, exhaustive, every claim verifiable from the source. There is no narrative, no hero copy, no calls to action.

### Specific examples lifted from the repo

> _"A clickable push button. When `checkable` is `false` (the default), each `click()` emits `clicked(false)`. When `checkable` is `true`, each `click()` toggles `checked` and emits `toggled(new_checked)` followed by `clicked(new_checked)`."_ (from `Button`)

> _"Precedence for fill/text axis: disabled > pressed > checked > hovered > idle. `disabled` is an alpha modifier applied after role selection; `pressed` reads `Highlight × Pressed`, `checked` reads `Highlight × Normal`."_ (from `DefaultStyle::paint::<Button>`, post #402)

> _"`focused` is an additive outline modifier — always 2 px `ColorRole::FocusRing`, never alpha-halved."_ (from `DefaultStyle::paint::<Button>`, post #402)

When writing in the Quartzite voice: pick the precise verb,
quote the field name, and finish the sentence. Do not garnish.

## VISUAL FOUNDATIONS

Almost every visual decision is encoded inside
`quartzite-style/src/default_style.rs`. The rules below are
restatements of that file.

### Color

- **13 semantic roles**, declared in `ColorRole`: `Window`, `WindowText`, `Button`, `ButtonText`, `Base`, `Text`, `Highlight`, `HighlightedText`, `Link`, `LinkVisited`, `BrightText`, `FocusRing`, `ScrollBar`. These are slots, not concrete colors — a dark theme and a light theme populate the same slots with different RGBA. (`FocusRing` is added by issue #402; its default value equals `Highlight × Normal`. `ScrollBar` is added by `proposals/scrollbar.md`; only the thumb reads from it — the track is `Base`.)
- **`Palette::default` seeds every slot to a non-transparent value.** Backgrounds white, foregrounds black, `Highlight` sky-blue (`#0080FF`), `Link` blue (`#0000FF`), `HighlightedText` and `BrightText` white, `FocusRing` sky-blue (matching `Highlight`). The defaults are explicitly described in source as "intentionally minimal — the goal is to satisfy the `default != Color::TRANSPARENT` invariant for every role rather than to produce a polished theme."
- **State groups.** Issue #402 adds a `ColorGroup` axis (`Normal` / `Hover` / `Pressed`) orthogonal to `ColorRole`. Lookups become `palette.color(role, group)`. `Hover` and `Pressed` cells are **derived** from the role's `Normal` value at construction time via `Hover(c) = c.blend(WindowText.Normal, 0.06)` and `Pressed(c) = c.blend(WindowText.Normal, 0.16)`. For the default light palette this lands on `#F0F0F0` / `#D6D6D6` (Button) and `#0078F0` / `#006CD6` (Highlight). Themes opt out per cell.
- **Disabled state** halves the alpha of whatever role-color was selected (`color.with_alpha(color.a() * 0.5)`). It is an alpha modifier, not a role swap. Not a `ColorGroup` variant — stays as mathematical post-processing.
- **Pressed and checked** both swap to the `Highlight` role for fill + `HighlightedText` for text. Pressed reads `Highlight × Pressed`; checked reads `Highlight × Normal`. Pressed wins over checked on the group axis.
- **Focus** is additive: a 2 px outline reading `ColorRole::FocusRing × Normal` overlaid on top of the idle/hover/pressed look. `FocusRing` defaults to `Highlight`'s value; themes that need a divergent focus ring (high-contrast amber, etc.) override the slot directly.
- **Links** are blue (`#0000FF`); visited and unvisited are seeded to the same value. Theme implementors are expected to differentiate them.

### Type

- **Default font: `Font::default()` → family `"sans-serif"`, size `12.0 pt`, weight `400`, no italic/underline/strikethrough.** The backend renderer (`VelloPainter` via parley/skrifa) is expected to resolve the family.
- **`FontWeight` is a CSS-numeric enum** (`Thin 100 → Black 900`, `Normal = 400`, `Bold = 700`).
- **There is no type scale defined in the framework.** Widgets carry a `Font` on their `WidgetBase` and the style draws with whatever is set. This design system adds a minimal scale (`24/18/14/12/11/10` pt) for documentation use only — it is not part of the framework's contract.
- **Alignment** is a 9-cell grid via `quartzite-geometry::Alignment` (`Left`, `Right`, `Top`, `Bottom`, `Center`, plus combinations). Buttons center; Labels and inputs left-align.

### Backgrounds

- **No images.** No patterns. No gradients in the default palette. `Brush::solid(color)` is what `DefaultStyle` calls at every paint site.
- **`Brush` does support gradients** — `BrushKind::LinearGradient { ... }`, `RadialGradient { ... }`, and `Custom(peniko::Gradient)` as an escape hatch — but `DefaultStyle` never reaches for them.
- **Surfaces are flat fills:** `Window` behind containers, `Base` behind inputs, `Button` behind buttons.

### Borders & strokes

- **1 px black outline (Pen width 1.0)** on every framed widget at rest — `Button`, `LineEdit`, `TextEdit`, `ScrollArea`, `Container`.
- **2 px Highlight outline** on focus. The outline is additive — it sits on top of the 1 px stroke during paint order, but visually replaces it.
- **No rounded corners.** `painter.draw_rect` takes a `Rect` with no radius. The look is uncompromisingly geometric.
- **No drop shadows. No inner shadows.** The paint API has no shadow primitive.

### Layout & spacing

- **Geometry is integer pixels.** `Rect::new(Point::new(0, 0), Size::new(64, 64))` is the canvas the snapshot tests render into. There is no spacing token system.
- **Two layouts ship in `quartzite-widgets`:** `BoxLayout` and `GridLayout`. Both honor child margins, hints, and stretch factors. The framework does **not** prescribe a base unit (no 4/8 grid baked in).
- **Padding inside a button is whatever the renderer's text-in-rect resolves to** — `painter.draw_text_in(geom, ..., Alignment::Center)` lays the text inside the full rect.

### Animation, hover, press

- **No animations.** `DefaultStyle` paints from instantaneous boolean state (`is_hovered`, `is_pressed`, `is_focused`, `is_enabled`, `checked`). Transitions are not modeled in the paint API.
- **Hover** = role's `Hover` cell. Default derivation `c.blend(WindowText.Normal, 0.06)` — light theme darkens, dark theme lightens.
- **Press** = `Highlight × Pressed` fill, `HighlightedText` foreground. Default derivation `c.blend(WindowText.Normal, 0.16)`.
- **Checked** = `Highlight × Normal` fill, `HighlightedText` foreground. Same role swap; group stays `Normal`.
- **Disabled** = ×0.5 alpha on whatever the resolved color was. Not a `ColorGroup` variant; mathematical post-processing.
- **Focus** = 2 px `ColorRole::FocusRing` outline overlay. Defaults to `Highlight`; themes can diverge.

### Transparency & blur

- **Only one use of transparency in `DefaultStyle`:** the read-only overlay on `LineEdit` and `TextEdit`. The painter fills `Base`, then fills again with `Window.with_alpha(0.5)` to dim the field. There is no blur. There is no backdrop effect.
- **`Color::TRANSPARENT`** is used as a "no fill" sentinel passed to `painter.draw_rect`'s `brush` argument when the call site only wants a stroke.

### Imagery vibe

- **No imagery in the framework.** No stock photos. No illustrations. No icon font.
- **`Image` is a (width, height, RGBA) blob** the painter can `draw_image(rect, image)`. Applications supply their own images.
- **If imagery is added by a downstream app**, the framework's lineage suggests: utilitarian, sharp, no filters. Treat it like a Qt screenshot, not a marketing hero.

### Layout chrome

- **No fixed nav bars, no sticky headers, no chrome.** `Container` is a generic widget that holds child ids; the only adornment is a 1 px `WindowText` outline. `ScrollArea` paints `Base` + 1 px outline and delegates content to children.

## WIDGET SPECS

The rules below are per-widget contracts on top of the
system-wide *Visual Foundations*. Each subsection corresponds to
a proposal under `proposals/` that the upstream repo treats as
the binding spec.

### Text input

Caret and selection rules for `LineEdit` and `TextEdit`. Full
proposal: `proposals/caret-and-selection.md`. Reference cards:
`preview/comp-line-edit-caret.html`,
`preview/comp-line-edit-selection.html`,
`preview/comp-text-edit-selection.html`.

- **Caret width: `1 px`.** Matches the widget-frame stroke
  convention. Two-pixel carets compete visually with the 2 px
  `FocusRing` overlay.
- **Caret height: full line-box** for the current `Font` (`ascent + descent + line_gap`).
  Same height the selection rectangle uses, so the two
  affordances are dimensionally consistent.
- **Caret color: `ColorRole::Text`** \u2014 not `WindowText`. The
  caret is glyph-adjacent; inputs paint glyphs from `Text` over
  `Base`.
- **Caret position is pixel-snapped** to an integer column. No
  AA. Matches the integer-pixel geometry rule.
- **Caret position when empty: left-aligned at the padding inset.**
  Vertically centered for `LineEdit`, top-left of the content
  rect for `TextEdit`. Matches the `Alignment::Left` the text
  would render at if non-empty \u2014 the caret does not jump on
  first keystroke.
- **Caret blink: `530 ms` on / `530 ms` off**, `1.06 s` square-
  wave cycle. Mirrors the X11/GNOME default
  `QApplication::cursorFlashTime()` on a stock install. Phase is
  global on `Application` so multiple visible carets blink in
  lockstep.
- **Reduced-motion fallback: steady-on.** When the host reports
  `prefers_reduced_motion`, the paint pass treats
  `caret_visible_now` as `true` unconditionally.
- **Selection rectangle hugs the line-box**, not glyph metrics.
  Pixel-snapped, no AA.
- **Selection fill: `ColorRole::Highlight \u00d7 Normal`.** Selected
  glyphs are overdrawn in `ColorRole::HighlightedText` clipped
  to the selection range.
- **Multi-line wrap (`TextEdit`):** per-visual-line rectangles
  tiled vertically with **no inter-line gap**. For each visual
  line the rect is `[line_start_x, line_end_x)`, where
  `line_start_x` = `sel_start_x` only if the selection starts
  on this line (else `content_left`) and `line_end_x` =
  `sel_end_x` only if the selection ends on this line (else
  `content_right`). The practical consequence: a multi-line
  wrapped selection has a tidy right edge — L1 extends to
  `content_right` because the selection wraps past it, L2 fills
  the line, L3 hugs `sel_end_x` on the right. Single-line
  (non-wrapping) selections hug both ends.
- **Read-only: caret hidden, selection allowed.** Read implies
  copy; copy requires selection. Removing selection from
  read-only would contradict the field name.
- **Disabled: no caret, no selection rendered.** The selection
  range in `selection_anchor` is **preserved**, not cleared;
  re-enabling the widget restores its visible selection. The
  disabled overlay (\u00d7 0.5 alpha) is applied after composition.
- **Unfocused with selection: macOS-style greyed.** Selection
  fill is `disabled(Highlight)` (alpha-halved `Highlight`); the
  glyphs revert to `ColorRole::Text` rather than
  `HighlightedText`. The selection is preserved \u2014 hiding it on
  blur would lose user state; saturating it would falsely
  suggest the widget still holds focus.

### ScrollArea

Track + thumb geometry. Full proposal: `proposals/scrollbar.md`.
Reference card: `preview/comp-scroll-area.html`.

- **Track width: `12 px`.** Same for both axes.
- **Thumb min-length: `24 px`.** Clamp when `content / viewport`
  is large.
- **Thumb radius: `0 px`.** Per the no-rounded-corners rule.
- **Thumb inset: `0 px`** on both axes. Thumb fills the full
  12 px lane.
- **Track fill: `ColorRole::Base`.** The track is not a new
  surface; it is the same `Base` the surrounding widget paints,
  separated from the content rect by a 1 px `WindowText` stroke
  on the inner edge.
- **Thumb fill state machine** uses the `ColorGroup` axis from
  #402:
  - idle      = `ScrollBar \u00d7 Normal`
  - hover     = `ScrollBar \u00d7 Hover`   (hover on the **thumb only** \u2014 not the empty track region)
  - pressed   = `ScrollBar \u00d7 Pressed` (holds for the entire drag, including when the mouse leaves the thumb mid-drag)
- **Disabled: whole widget \u00d7 0.5 alpha.** Not a role swap.
- **Two-axis case:** when both lanes are present, the
  bottom-right `12 \u00d7 12` corner is a `Base` fill with the
  surrounding 1 px stroke continuation. The corner is not
  interactive.
- **Visibility = reserved space, not overlay.** The
  `ScrollArea` content rect is computed **excluding** the
  scrollbar lanes. Overlay scrollbars (auto-hiding bars
  floating over content) are rejected \u2014 they require a fade
  animation, and the framework has no animation primitive.
- **`ScrollPolicy`** governs lane presence:
  - `AsNeeded` (default) \u2014 lane appears when content overflows on that axis.
  - `AlwaysOn` \u2014 lane always reserved, even with no overflow.
  - `AlwaysOff` \u2014 lane never reserved; mouse-wheel still scrolls.

### Cursor shapes

Per-widget defaults and the bounded `winit::window::CursorIcon`
subset endorsed by `DefaultStyle`. Full proposal:
`proposals/cursor-shapes.md`. No HTML reference card \u2014 cursor
shape is a host-window call, not a paint output.

- **Hover alone does not change cursor shape.** Cursor follows
  widget-type identity + state overrides, nothing else. Hover
  over a `Button` changes the **button's** fill but not the
  cursor.
- **Per-widget defaults:**

  | Widget | Default shape | `winit` ident |
  |---|---|---|
  | `Button`     | arrow  | `CursorIcon::Default` |
  | `Label`      | arrow  | `CursorIcon::Default` |
  | `LineEdit`   | I-beam | `CursorIcon::Text`    |
  | `TextEdit`   | I-beam | `CursorIcon::Text`    |
  | `ScrollArea` | arrow  | `CursorIcon::Default` |
  | `Container`  | arrow  | `CursorIcon::Default` |

- **State-driven overrides** \u2014 the only two:
  - `enabled == false` on any widget          \u2192 `CursorIcon::NotAllowed`
  - `ScrollArea` thumb while `is_pressed`      \u2192 `CursorIcon::Grabbing`
- **Endorsed `CursorIcon` subset:** `Default`, `Text`,
  `NotAllowed`, `Grabbing`, `Wait`, `Progress`. `Wait` and
  `Progress` are app-level overrides on `Application`, not
  per-widget. `Pointer` ("hand") is **not** endorsed \u2014 buttons
  keep the arrow, matching native convention.
- **Resolution precedence** (first match wins):
  1. Application-wide cursor (`Wait` / `Progress`)
  2. `widget.enabled == false` \u2192 `NotAllowed`
  3. `ScrollArea` thumb drag \u2192 `Grabbing`
  4. `WidgetBase::cursor` explicit override
  5. Per-widget-type default (table above)

### Elevation and overlays

Popups and tooltips, and the carve-out from the no-shadow rule.
Full proposal: `proposals/popups-and-tooltips.md`. Reference
cards: `preview/comp-popup.html`, `preview/comp-tooltip.html`.

- **No drop shadows. No inner shadows.** Restated verbatim from
  *Visual Foundations \u00b7 Borders & strokes*. The paint API has no
  shadow primitive, and `DefaultStyle` neither adds one nor
  works around its absence for elevation. Both `Popup` and
  `Tooltip` solve substrate separation with primitives already
  in the API.
- **`Popup` separates from substrate with `2 px WindowText`
  border** \u2014 twice the widget-frame weight \u2014 plus a `4 px`
  inset gap from the trigger widget. Border-weight contrast is
  the only flat primitive that reads as "different surface"
  without inventing a shadow.
- **`Popup` chrome:**
  - fill        = `ColorRole::Window`
  - border      = 2 px `WindowText`
  - radius      = 0 px
  - row hover   = `Highlight \u00d7 Normal` fill + `HighlightedText`
  - row pressed = `Highlight \u00d7 Pressed` fill + `HighlightedText`
  - separator   = 1 px `WindowText` full-width line
  - min width   = `max(trigger.width(), text_max_width)`
  - max size    = unconstrained; clipped to window bounds
- **`Tooltip` separates from substrate by inversion** \u2014 fill =
  `ColorRole::WindowText`, text = `ColorRole::Window`, **no
  border**. The colour boundary against the substrate is itself
  the separation signal; a 1 px stroke would compete with the
  glyphs and the inversion is already full-luminance contrast.
- **`Tooltip` chrome:**
  - fill        = `ColorRole::WindowText`
  - text        = `ColorRole::Window`
  - border      = none
  - padding     = 4 px vertical, 8 px horizontal
  - font        = `Font::default()`
  - max width   = `280 px` before wrap (theme-overridable)
  - anchor inset = 4 px from the anchor's bottom edge
  - delay       = 500 ms hover
  - tooltips never accept focus and never paint a focus ring.
- **Z-order is an explicit `ZLayer` field on `WidgetBase`**, not
  an implicit overlay pass:

  ```rust
  pub enum ZLayer { Content = 0, Overlay = 1, Tooltip = 2 }
  ```

  The paint pass walks the tree once per layer in ascending
  order. `Popup` defaults to `Overlay`; `Tooltip` defaults to
  `Tooltip`; everything else defaults to `Content`. Making the
  field public lets downstream apps lift their own widgets
  above `Content` without subclassing `Popup`.
- **Stacked popups (nested submenus)** are out of scope for v1.
  When they ship, the border weight grows by 1 px per level,
  capped at 3 px. The cap matches the limit of "border weight as
  depth signal" before the popup itself starts to look heavy.

## ICONOGRAPHY

**Quartzite ships no icons.** There is no icon font, no SVG sprite,
no built-in glyph system in `quartzite-paint-api` beyond
`Painter::draw_image` (raw RGBA blobs) and `Painter::draw_path`
(`Path` made of move-to / line-to / quad / cubic / close primitives).

The `quartzite-style` snapshot suite contains a handful of PNGs —
but those are golden-image **test artifacts**, not icons. They
live in `quartzite-style/tests/snapshots/shared/` in the upstream
repo.

When working with Quartzite, follow these rules:

1. **No emoji.** The source has none and the painter has no color-emoji rasterizer. If a UI needs symbols, draw them.
2. **No unicode-as-iconography.** Don't reach for `▶ ◀ ✓ ★`. The default font (`sans-serif`) is whatever the renderer resolves, and glyph coverage is not guaranteed. Specifically, the `text_edit_plain` and `text_edit_read_only` snapshots use the string `"abc"` — three lowercase letters — because that's what is guaranteed to render.
3. **For HTML mocks in this design system, the substitute icon set is [Lucide](https://lucide.dev/).** Stroke-based, 1 px (actually 2 px @ 24 px), monochrome, no fill. This matches Quartzite's flat-stroke aesthetic better than Heroicons (which has a heavier filled variant) or Material Icons (which is multi-weight and feels Google-branded). **This is a documentation-time substitution. Production Quartzite UIs are expected to either ship their own raster icons via `Painter::draw_image` or vector glyphs via `Painter::draw_path`.**
4. **Stroke style for any path-drawn icon:** `Pen::new(palette.color(ColorRole::WindowText), 1.0)` — matches the framing stroke on every widget. No fill (`Brush::solid(Color::TRANSPARENT)`).

### Substitution flag

> ⚠️ **No icons exist in `maratik123/quartzite`.** Lucide is used in
> mocks and the UI kit as a placeholder. If the project ships its
> own icon assets later, replace `cdn.jsdelivr.net/.../lucide`
> references everywhere they appear.

## Typography substitution

The framework asks for `"sans-serif"` and lets the renderer
resolve it. The committed golden PNGs were rendered with
parley/skrifa against whatever the harness host resolved. For
HTML mocks here we use a stack rooted on **DejaVu Sans** /
**Liberation Sans** — the families a stock Linux text shaper
typically lands on, and the closest visual match to the
snapshot PNGs.

> ⚠️ **Font substitution flag.** No `.ttf` / `.otf` files were
> distributed with the source repo. `fonts/` is intentionally
> empty. If you want pixel-identical mockups, drop the actual
> render target font into `fonts/` and the CSS stack will pick
> it up automatically (it's listed first in `--qz-font-family`).

## Designer mark

**Quartzite Designer** is a sibling application that ships with
the framework. Its mark is the canonical Quartzite mark with one
addition: a **2 px `Highlight` outline along the back-right facet
boundary** — `(32,32) → (54,20) → (54,44) → (32,58) → close` —
drawn with `stroke-linejoin="bevel"` so the acute corners at
(54,20) and (54,44) terminate cleanly on the hex silhouette.
The outline reads as the framework's focused-widget visual idiom
applied to a single facet — "Quartzite, with a thing selected".

The Designer wordmark uses a stacked primary lockup: `quartzite`
in DejaVu Sans Regular 28 pt with `DESIGNER` set 12 pt / weight
500 / letter-spacing 2.4 px directly beneath, painted in
`ColorRole::Highlight` (`#0080FF` light, `#1E90FF` dark). The
showcase files at `Quartzite Designer Logo.html` and
`Quartzite Designer Assets.html` enumerate the variants — light,
dark, halo, and the construction grid (vertices identical to the
base mark; selection ring overlaid).

## Dark theme

The framework ships a single `Palette::default` (light) plus a
single `DefaultStyle`. **A dark theme is a Palette override**, not
a new `Style` — same 13 `ColorRole` slots, different RGBA in each
slot. Both palettes must satisfy the same two invariants:

1. Every role is non-transparent.
2. `Highlight` is visually distinct from `HighlightedText`.

The dark seeds proposed in `colors_and_type.css`
(`[data-theme="dark"]`) and demonstrated in
`preview/dark-*.html`:

| Role | Light seed | **Dark seed** | Common name (dark) |
|---|---|---|---|
| `Window`          | `#FFFFFF` | **`#2B2B2B`** | Mine Shaft |
| `WindowText`      | `#000000` | **`#E8E8E8`** | Mercury |
| `Button`          | `#FFFFFF` | **`#3C3C3C`** | Eclipse |
| `ButtonText`      | `#000000` | **`#E8E8E8`** | Mercury |
| `Base`            | `#FFFFFF` | **`#1E1E1E`** | Nero |
| `Text`            | `#000000` | **`#E8E8E8`** | Mercury |
| `Highlight`       | `#0080FF` | **`#1E90FF`** | DodgerBlue (X11 — close cousin of `SKY_BLUE`, brighter for dark substrate) |
| `HighlightedText` | `#FFFFFF` | `#FFFFFF`     | White (unchanged) |
| `Link`            | `#0000FF` | **`#5BB0FF`** | Light Dodger Blue (coined — no catalogued match; `#0000FF` is illegible against `#2B2B2B`) |
| `LinkVisited`     | `#0000FF` | **`#C58AFF`** | Charoite (coined — purple silicate mineral; sister to `Quartzite`. No catalogued match.) |
| `BrightText`      | `#FFFFFF` | **`#FF6B6B`** | Pastel Red (Qt convention: red signals attention against a coloured banner) |
| `FocusRing` *(new, #402)* | `#0080FF` | **`#1E90FF`** | DodgerBlue — mirrors `Highlight` by default; theme-overridable |
| `ScrollBar` *(new, scrollbar.md)* | **`#C8C8C8`** | **`#5A5A5A`** | Silver Sand / Mortar — thumb only; track is `Base` |

> **Naming source.** "Common name" labels are documentation-only —
> the framework does not use them. They come from curated
> aggregators (HtmlCssColor, color-name.com, SchemeColor, ArtyClick)
> picking the entry with the closest ΔE to each seed. Two seeds
> have no catalogued match: `#5BB0FF` (we coin **Light Dodger Blue**
> rather than approximate it to Maya Blue `#73C2FB` or French Sky
> Blue `#77B5FE` — neither matches) and `#C58AFF` (we coin
> **Charoite** after the purple silicate mineral, keeping the
> `-ite` mineral-suffix theme `Quartzite` already commits to;
> documented "Lilac" hexes range from `#C8A2C8` to `#DCD0FF` and
> none coincides with our seed — see
> `preview/dark-link-visited-lilac-compare.html`).

**Derived state values follow the framework's same formulas**, so
they re-compute correctly against the dark slots:

- **Hover** = `c.blend(WindowText.Normal, 0.06)` — on dark that
  lights toward Mercury (`#E8E8E8`). For `Button`, lands on
  `#464646` (HOVER_ECLIPSE). For `Highlight`, lands on `#2A95FE`
  (HOVER_DODGER_BLUE).
- **Pressed** = `c.blend(WindowText.Normal, 0.16)` — same direction,
  stronger nudge. `Button × Pressed` = `#585858` (PRESSED_ECLIPSE);
  `Highlight × Pressed` = `#3E9EFB` (PRESSED_DODGER_BLUE).
- **Disabled** = `color.with_alpha(color.a() * 0.5)` — α-halving
  fades dark colors against the dark Window in the same way it
  fades light colors against the light Window.
- **Focus** = additive 2 px `FocusRing` outline overlay — same rule;
  defaults to `Highlight`'s Normal value.
- **Read-only** = Base fill with 50 % Window overlay — same rule;
  the overlay's source colour is whatever Window resolves to.

**In Rust**, a dark theme is implemented either by overriding
`Palette::default`'s slots with `.with_role(...)` at app start, or
by registering a new `Style` impl in `StyleRegistry`:

```rust
let dark = Palette::default()
    .with_role(ColorRole::Window,          Color::new(0.169, 0.169, 0.169, 1.0)) // #2B2B2B  Mine Shaft
    .with_role(ColorRole::WindowText,      Color::new(0.910, 0.910, 0.910, 1.0)) // #E8E8E8  Mercury
    .with_role(ColorRole::Button,          Color::new(0.235, 0.235, 0.235, 1.0)) // #3C3C3C  Eclipse
    .with_role(ColorRole::ButtonText,      Color::new(0.910, 0.910, 0.910, 1.0)) // #E8E8E8  Mercury
    .with_role(ColorRole::Base,            Color::new(0.118, 0.118, 0.118, 1.0)) // #1E1E1E  Nero
    .with_role(ColorRole::Text,            Color::new(0.910, 0.910, 0.910, 1.0)) // #E8E8E8  Mercury
    .with_role(ColorRole::Highlight,       Color::new(0.118, 0.564, 1.000, 1.0)) // #1E90FF  DodgerBlue (X11)
    .with_role(ColorRole::HighlightedText, Color::WHITE)                          // #FFFFFF
    .with_role(ColorRole::Link,            Color::new(0.357, 0.690, 1.000, 1.0)) // #5BB0FF  Light Dodger Blue
    .with_role(ColorRole::LinkVisited,     Color::new(0.773, 0.541, 1.000, 1.0)) // #C58AFF  Charoite
    .with_role(ColorRole::BrightText,      Color::new(1.000, 0.420, 0.420, 1.0)) // #FF6B6B Pastel Red
    .with_role(ColorRole::FocusRing,       Color::new(0.118, 0.564, 1.000, 1.0)) // #1E90FF DodgerBlue (mirrors Highlight)
    .with_role(ColorRole::ScrollBar,       Color::new(0.353, 0.353, 0.353, 1.0)); // #5A5A5A Mortar
```

The UI-kit demo at `ui_kits/widgets/index.html` carries a
`☾ dark` / `☀ light` toggle in its title bar — flipping it swaps
the `data-theme` attribute on `<body>`, which retargets every
`var(--qz-*)` through the CSS override block. This is a mocking
shortcut; the production path is the `Palette` swap above.
