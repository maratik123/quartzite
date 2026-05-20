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
  - `quartzite-style/tests/snapshots/shared/*.png` — committed golden PNGs of every widget state under `DefaultStyle`. Truth source for the screenshots in this system.
  - `quartzite-paint-api/src/{color,font,brush,pen,path,image,painter}.rs` — the painting primitives the style draws against.

The reader is encouraged to explore the repository — especially `quartzite-widgets` and `quartzite-style` — to recreate or extend designs faithfully. Most decisions in this design system are **described** rather than re-derived; the source of truth is the Rust code.

## What's in here

A short manifest of the root:

| Path | What it is |
|---|---|
| `README.md` | This file. Brand context, content fundamentals, visual foundations, iconography. |
| `SKILL.md` | Agent-Skills-compatible entry point so this folder can be loaded as a skill. |
| `colors_and_type.css` | CSS custom properties + element styles mirroring `Palette::default` and `Font::default`. |
| `fonts/` | (empty — see _Typography_ below.) |
| `assets/` | Logos and snapshot pngs. |
| `assets/quartzite-mark.svg` | 64 px crystalline mark. |
| `assets/quartzite-wordmark.svg` | Mark + `quartzite` lockup. |
| `assets/snapshots/` | The 10 committed golden PNGs from `quartzite-style`. |
| `preview/` | 24 cards populating the Design System tab (Brand / Colors / Type / Spacing / Components). |
| `preview/card-base.css` | Shared chrome + widget-mock CSS for every preview card. |
| `ui_kits/widgets/` | HTML/JSX recreation of the six built-in widgets + an interactive demo (title-bar toggle flips light ↔ dark). |
| `ui_kits/widgets/index.html` | Working `quartzite-widgets` demo: Counter + new-note form + Signal log. |
| `ui_kits/widgets/{Button,Label,LineEdit,TextEdit,ScrollArea,Container,WindowFrame,App}.jsx` | Per-widget React components mirroring real props + signals. |
| `proposals/text-edit-read-only-overlay.md` | Proposed framework fix for the invisible read-only overlay on `Palette::default`. |

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

> _"Precedence for fill/text axis: pressed > checked > hovered > idle. `disabled` is an alpha modifier applied after role selection (not a role-selector)."_ (from `DefaultStyle::paint::<Button>`)

> _"`focused` is an additive outline modifier — always 2 px Highlight, never alpha-halved."_ (from `DefaultStyle::paint::<Button>`)

When writing in the Quartzite voice: pick the precise verb,
quote the field name, and finish the sentence. Do not garnish.

## VISUAL FOUNDATIONS

Almost every visual decision is encoded inside
`quartzite-style/src/default_style.rs`. The rules below are
restatements of that file.

### Color

- **11 semantic roles**, declared in `ColorRole`: `Window`, `WindowText`, `Button`, `ButtonText`, `Base`, `Text`, `Highlight`, `HighlightedText`, `Link`, `LinkVisited`, `BrightText`. These are slots, not concrete colors — a dark theme and a light theme populate the same slots with different RGBA.
- **`Palette::default` seeds every slot to a non-transparent value.** Backgrounds white, foregrounds black, `Highlight` sky-blue (`#0080FF`), `Link` blue (`#0000FF`), `HighlightedText` and `BrightText` white. The defaults are explicitly described in source as "intentionally minimal — the goal is to satisfy the `default != Color::TRANSPARENT` invariant for every role rather than to produce a polished theme."
- **Hover state** is `Button.blend(Highlight, 0.25)` — for the default palette that lands on `#BFDFFF`. The blend is computed at draw time, not stored.
- **Disabled state** halves the alpha of whatever role-color was selected (`color.with_alpha(color.a() * 0.5)`). It is an alpha modifier, not a role swap.
- **Pressed and checked** swap to `Highlight` fill + `HighlightedText` foreground. Pressed wins over checked.
- **Focus** is additive: a 2 px `Highlight` outline overlaid on top of the idle/hover/pressed look. Never alpha-halved.
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
- **Hover** = blend toward `Highlight` 25 %.
- **Press / checked** = solid `Highlight` fill, `HighlightedText` foreground.
- **Disabled** = ×0.5 alpha on whatever the resolved color was.
- **Focus** = 2 px `Highlight` outline overlay.

### Transparency & blur

- **Only one use of transparency in `DefaultStyle`:** the read-only overlay on `LineEdit` and `TextEdit`. The painter fills `Base`, then fills again with `Window.with_alpha(0.5)` to dim the field. There is no blur. There is no backdrop effect.
- **`Color::TRANSPARENT`** is used as a "no fill" sentinel passed to `painter.draw_rect`'s `brush` argument when the call site only wants a stroke.

### Imagery vibe

- **No imagery in the framework.** No stock photos. No illustrations. No icon font.
- **`Image` is a (width, height, RGBA) blob** the painter can `draw_image(rect, image)`. Applications supply their own images.
- **If imagery is added by a downstream app**, the framework's lineage suggests: utilitarian, sharp, no filters. Treat it like a Qt screenshot, not a marketing hero.

### Layout chrome

- **No fixed nav bars, no sticky headers, no chrome.** `Container` is a generic widget that holds child ids; the only adornment is a 1 px `WindowText` outline. `ScrollArea` paints `Base` + 1 px outline and delegates content to children.

## ICONOGRAPHY

**Quartzite ships no icons.** There is no icon font, no SVG sprite,
no built-in glyph system in `quartzite-paint-api` beyond
`Painter::draw_image` (raw RGBA blobs) and `Painter::draw_path`
(`Path` made of move-to / line-to / quad / cubic / close primitives).

The `quartzite-style` snapshot suite contains a handful of PNGs —
but those are golden-image **test artifacts**, not icons. They live
in `assets/snapshots/` here for reference.

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

## Visual snapshots reference

The eight `DefaultStyle` snapshots from
`quartzite-style/tests/snapshots/shared/` are mirrored into
`assets/snapshots/` for reference inside this design system.
They are 64×64 PNGs.

| Snapshot | What it shows |
|---|---|
| `button_idle.png`        | Button at rest. White fill, 1 px black outline, "OK" centered. |
| `button_hovered.png`     | Same with fill blended 25 % toward sky blue. |
| `button_pressed.png`     | Full sky-blue fill, white text. |
| `button_checked.png`     | Identical to pressed — `checked` falls under the same arm. |
| `button_focused.png`     | Idle look + 2 px sky-blue outline overlay. |
| `button_disabled.png`    | Whole composition ×0.5 alpha. |
| `label.png`              | White surface, left-aligned "hi". |
| `scroll_area_chrome.png` | Empty white field, 1 px outline. |
| `text_edit_plain.png`    | White field, "abc" top-left, 1 px outline. |
| `text_edit_read_only.png`| Same with a 50 % white overlay across the whole rect. |

## Dark theme

The framework ships a single `Palette::default` (light) plus a
single `DefaultStyle`. **A dark theme is a Palette override**, not
a new `Style` — same 11 `ColorRole` slots, different RGBA in each
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

- **Hover** = `Button.blend(Highlight, 0.25)` → on dark that
  lands on `#345067` (saturated steel-blue), not light blue.
- **Disabled** = `color.with_alpha(color.a() * 0.5)` — α-halving
  fades dark colors against the dark Window in the same way it
  fades light colors against the light Window.
- **Focus** = additive 2 px Highlight outline overlay — same rule.
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
    .with_role(ColorRole::BrightText,      Color::new(1.000, 0.420, 0.420, 1.0)); // #FF6B6B Pastel Red
```

The UI-kit demo at `ui_kits/widgets/index.html` carries a
`☾ dark` / `☀ light` toggle in its title bar — flipping it swaps
the `data-theme` attribute on `<body>`, which retargets every
`var(--qz-*)` through the CSS override block. This is a mocking
shortcut; the production path is the `Palette` swap above.
