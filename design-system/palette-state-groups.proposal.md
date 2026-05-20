# Proposal: Palette state groups — `Hover`, `Pressed`, and a `FocusRing` role

**Affects:** `quartzite-style-types/src/{color_role.rs,palette.rs,dark_palette.rs,lib.rs}`, `quartzite-style/src/default_style.rs`, `design-system/colors_and_type.css`, `design-system/README.md` § *Dark theme* and § *Color*.

**Type:** new public API on `quartzite-style-types`. Breaking change to `Palette::color` and `Palette::with_role` signatures. Pre-publish — no external users.

**Issue:** #402

**Companion:** `palette-state-groups-swatches.html` — side-by-side swatches of every cell in both themes plus a live `Button` preview in each state.

## Problem

State-dependent button colours in `DefaultStyle::draw_button` are computed from existing roles via blending and role-swapping (issue #316). Three limitations follow:

1. **No per-theme override.** A theme that wants its hover colour to differ from the `Button → Highlight` blend has to ship its own `Style` impl. The `Palette` cannot express it.
2. **`FocusRing` colour is hardcoded to `Highlight`.** A theme whose focus ring should differ from its selection colour (orange ring against a blue selection, e.g.) cannot diverge without forking `DefaultStyle`.
3. **Future widgets duplicate the heuristic.** When `LineEdit` (#406), `TextEdit`, and `ScrollArea` (#403) need hover / pressed visuals, each painter will copy the same blend-and-swap logic from `draw_button`. There is no shared vocabulary.

The deferred row for #402 in `widget-backlog.md` calls out the underlying constraint: state colours belong in the `Palette`, not in the style.

## Proposal

Two changes, both in `quartzite-style-types`.

### 1. New `ColorGroup` axis on `Palette`

A second enum, **orthogonal** to `ColorRole`. Palette lookups become `palette.color(role, group)`.

```rust
/// Interaction state that selects a colour variant within a [`ColorRole`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorGroup {
    /// Resting state — the role's primary colour.
    #[default]
    Normal,
    /// Pointer is over the widget but no button is pressed.
    Hover,
    /// Widget is being actuated (mouse-down, key-down).
    Pressed,
}

impl ColorGroup {
    pub const ALL: &'static [Self] = &[Self::Normal, Self::Hover, Self::Pressed];
}
```

**`Disabled` is intentionally not a group.** It stays a mathematical alpha-modifier (`color.with_alpha(0.5)`) applied after role resolution, matching today's behaviour. Promoting it would force every theme to seed 11 redundant `Disabled` cells whose values are mechanically derivable from the `Normal` cells.

**`Focused` is intentionally not a group.** Focus is a stroke modifier, not a fill modifier — it does not change the role colours, it draws an additional outline. See change 2.

### 2. New `ColorRole::FocusRing`

A dedicated slot for the focus-ring stroke colour. Single value per theme, no meaningful `Hover` / `Pressed` variants (the cells exist in the matrix and mirror `Normal`).

```rust
pub enum ColorRole {
    // ... existing 11 ...
    /// Stroke colour of the focus outline drawn by `DefaultStyle` when
    /// `WidgetExt::is_focused()` is true.
    FocusRing,
}
```

Seeded to `Highlight`'s value in both shipped themes — themes that want a divergent focus ring (a contrast-test accessibility variant, an orange ring against a blue selection) override the slot directly.

### Invariants

Both shipped palettes (`Palette::default` and `DARK_PALETTE`) satisfy:

1. Every cell is non-transparent.
2. `Highlight` × `Normal` is visually distinct from `HighlightedText` × `Normal`. (Existing invariant, unchanged.)
3. For every role whose `Hover` / `Pressed` cells are not explicitly overridden, the cell is **derived** from the role's `Normal` value via the rule in § *Default derivation*.

## Default derivation

v1 ships **zero new non-derived colours**. Every meaningful state cell in both shipped palettes is computed from the role's `Normal` value at construction time via:

```text
Hover(c)   = c.blend(palette.color(WindowText, Normal), 0.06)
Pressed(c) = c.blend(palette.color(WindowText, Normal), 0.16)
```

The blend target is the palette's own `WindowText × Normal` slot — always the maximum-contrast foreground for that theme. The semantics are consistent across themes:

- **Light theme**: blends toward `#000000`. Hover and pressed both darken — 6 % and 16 % respectively. Matches Qt Fusion, Windows Common Controls, macOS Aqua.
- **Dark theme**: blends toward `#E8E8E8` (Mercury). Hover and pressed both lighten by the same amounts. The RGB direction inverts; the *meaning* ("more interaction = more contrast against the rest of the UI") is preserved.

This trades Qt-traditional "darker-on-press in dark theme" for a single-formula derivation that themes can rely on without seeding cells. Themes that want the Qt convention override the affected cells explicitly — see § *Overrides*.

## Concrete values

The tables below show every cell of both shipped palettes. Bold cells are the meaningful state values (computed via derivation; the listed RGBA is the result). `=` means the cell mirrors `Normal` byte-for-byte (the role has no meaningful state difference in v1; derivation produces the same value because `Normal == WindowText` already, or because the role is itself a foreground role).

### Light theme — `Palette::default`

| Role | `Normal` | `Hover` | `Pressed` |
|---|---|---|---|
| `Window` | `#FFFFFF` | `=` | `=` |
| `WindowText` | `#000000` | `=` | `=` |
| `Button` | `#FFFFFF` (`WHITE`) | **`#F0F0F0`** (`HOVER_WHITE`) | **`#D6D6D6`** (`PRESSED_WHITE`) |
| `ButtonText` | `#000000` | `=` | `=` |
| `Base` | `#FFFFFF` | `=` | `=` |
| `Text` | `#000000` | `=` | `=` |
| `Highlight` | `#0080FF` (`SKY_BLUE`) | **`#0078F0`** (`HOVER_SKY_BLUE`) | **`#006CD6`** (`PRESSED_SKY_BLUE`) |
| `HighlightedText` | `#FFFFFF` | `=` | `=` |
| `Link` | `#0000FF` | `=` | `=` |
| `LinkVisited` | `#0000FF` | `=` | `=` |
| `BrightText` | `#FFFFFF` | `=` | `=` |
| `FocusRing` *(new role)* | `#0080FF` | `=` | `=` |

### Dark theme — `DARK_PALETTE`

| Role | `Normal` | `Hover` | `Pressed` |
|---|---|---|---|
| `Window` | `#2B2B2B` (`MINE_SHAFT`) | `=` | `=` |
| `WindowText` | `#E8E8E8` (`MERCURY`) | `=` | `=` |
| `Button` | `#3C3C3C` (`ECLIPSE`) | **`#464646`** (`HOVER_ECLIPSE`) | **`#585858`** (`PRESSED_ECLIPSE`) |
| `ButtonText` | `#E8E8E8` | `=` | `=` |
| `Base` | `#1E1E1E` (`NERO`) | `=` | `=` |
| `Text` | `#E8E8E8` | `=` | `=` |
| `Highlight` | `#1E90FF` (`DODGER_BLUE`) | **`#2A95FE`** (`HOVER_DODGER_BLUE`) | **`#3E9EFB`** (`PRESSED_DODGER_BLUE`) |
| `HighlightedText` | `#FFFFFF` | `=` | `=` |
| `Link` | `#5BB0FF` | `=` | `=` |
| `LinkVisited` | `#C58AFF` | `=` | `=` |
| `BrightText` | `#FF6B6B` | `=` | `=` |
| `FocusRing` *(new role)* | `#1E90FF` | `=` | `=` |

## Naming convention

The project's existing catalog (`Color::WHITE`, `Color::SKY_BLUE`, `Color::MINE_SHAFT`, `Color::ECLIPSE`, `Color::CHAROITE`, etc.) names every concrete colour. The state-cells follow that pattern in two flavours:

- **Derived cells** — named `<GROUP>_<SOURCE>` where `<SOURCE>` is the name of the `Normal` colour they derive from. v1 introduces six documentary names:
  - `HOVER_WHITE` / `PRESSED_WHITE` — derived from `Color::WHITE` (Button in light theme).
  - `HOVER_SKY_BLUE` / `PRESSED_SKY_BLUE` — derived from `Color::SKY_BLUE` (Highlight in light).
  - `HOVER_ECLIPSE` / `PRESSED_ECLIPSE` — derived from `Color::ECLIPSE` (Button in dark).
  - `HOVER_DODGER_BLUE` / `PRESSED_DODGER_BLUE` — derived from `Color::DODGER_BLUE` (Highlight in dark).

  These names are **documentary**. The framework does not need to materialise them as separate `Color::HOVER_WHITE` constants — `Palette` computes the values via the derivation rule. The names give documentation, tests, and any downstream `match` arms a stable identifier to refer to.

- **Non-derived cells** — a fresh colour introduced by an override (see § *Overrides*). These follow the existing convention: web-search a catalog name (HtmlCssColor / color-name.com / SchemeColor / ArtyClick) with the closest ΔE match, or improvise a name that fits the mineral-suffix theme the project already commits to (`Quartzite`, `Charoite`). v1 introduces none.

## Overrides

Themes that disagree with derived values opt out per cell:

```rust
let dark_qt_style = DARK_PALETTE
    .with_role(ColorRole::Button,    ColorGroup::Pressed, Color::OBSIDIAN)        // hypothetical darker variant
    .with_role(ColorRole::Highlight, ColorGroup::Pressed, Color::AZURITE_DEEP);   // hypothetical darker blue
```

Each override introduces a non-derived colour that follows the naming convention above — `OBSIDIAN`, `AZURITE_DEEP` are illustrative; a real Qt-style dark theme would web-search for a name with the closest ΔE match to its target hex. v1 ships no such overrides; the derivation is the only behaviour the framework commits to.

The visual swatch sheet at `palette-state-groups-swatches.html` shows every derived cell at 64×64 px against its theme's `Window`, plus live `Button` mocks and an A/B comparison strip overlaying #316's existing 25 %-blend hover next to the new derived `Button.Hover` cell.

## `DefaultStyle::draw_button` refactor

State resolution collapses to a single lookup. The 25 % `Color::blend` from #316 disappears from `draw_button`; `Color::blend` itself stays as a general primitive.

```rust
fn draw_button(&self, w: &Button, painter: &mut dyn Painter, palette: &Palette) {
    let geom = w.geometry();

    let group = if w.is_pressed() { ColorGroup::Pressed }
                else if w.is_hovered() { ColorGroup::Hover }
                else { ColorGroup::Normal };

    let (fill_role, text_role) = if w.checked || w.is_pressed() {
        (ColorRole::Highlight, ColorRole::HighlightedText)
    } else {
        (ColorRole::Button, ColorRole::ButtonText)
    };

    let fill = maybe_disabled(palette.color(fill_role, group), w.is_enabled());
    let text = maybe_disabled(palette.color(text_role, group), w.is_enabled());

    painter.fill_rect(geom, &Brush::solid(fill));
    painter.draw_rect(geom, &Pen::new(text, 1.0), &Brush::solid(Color::TRANSPARENT));
    painter.draw_text_in(geom, &w.text, &font, &Brush::solid(text), Alignment::Center);

    if w.is_focused() {
        let ring = palette.color(ColorRole::FocusRing, ColorGroup::Normal);
        painter.draw_rect(geom, &Pen::new(ring, 2.0), &Brush::solid(Color::TRANSPARENT));
    }
}
```

State precedence (`disabled > pressed > checked > hovered` for the fill/text axis; `focused` additive) is preserved — `pressed` and `checked` both select the `Highlight` role, `pressed` wins the group axis so a `pressed && checked` button still picks `Highlight × Pressed` rather than `Highlight × Normal`. `disabled` continues as alpha-half post-resolution.

## Migration

| Change | Affected sites in-tree |
|---|---|
| `Palette::color(role) -> Color` becomes `Palette::color(role, group) -> Color` | `default_style.rs` (≈ 8 call sites), `palette.rs` doctests, `dark_palette.rs` doctests, every `colors_and_type.css` cross-reference |
| `Palette::with_role(role, color)` becomes `Palette::with_role(role, group, color)` | `dark_palette.rs` (12 chained calls), `palette.rs` doctests |
| Convenience: `Palette::role(role)` → `palette.color(role, ColorGroup::Normal)` | added |
| Convenience: `Palette::with_role_all_groups(role, color)` → seeds all three group cells | added (lets the existing `DARK_PALETTE` chain stay terse: the non-stateful roles use this) |
| `ColorRole::ALL` grows from 11 to 12 (adds `FocusRing`) | `color_role.rs::all_constant_lists_every_variant` test gains an arm |
| New `ColorGroup::ALL` constant + matching exhaustive-match test in `color_role.rs` (or in a new `color_group.rs`) | added |

A `Palette::new()` (the `const fn` constructor) seeds every cell to `Color::WHITE` except the foregrounds and accents called out in the existing seed list, and every non-`Normal` cell of every role to its `Normal` value. The existing invariant `default != Color::TRANSPARENT` for every role becomes "for every `(role, group)` pair".

## Snapshot tests

| Golden | Status | Why |
|---|---|---|
| `button_idle.png` | unchanged | Idle reads `Button × Normal` = the existing `Button` slot. |
| `button_hovered.png` | **regenerate** | Hover now reads `Button × Hover` (derived `HOVER_WHITE` = `#F0F0F0`) instead of the 25 % blend (`#BFDFFF`). Visible difference. |
| `button_pressed.png` | **regenerate** | Pressed now reads `Highlight × Pressed` (derived `PRESSED_SKY_BLUE` = `#006CD6`) instead of `Highlight × Normal` (`#0080FF`). Subtle but real difference. |
| `button_checked.png` | unchanged | Checked reads `Highlight × Normal` — unchanged from today. |
| `button_focused.png` | unchanged (byte-identical) | Focus ring reads `FocusRing × Normal`, seeded to `Highlight`'s `Normal` value — same RGBA as before. |
| `button_disabled.png` | unchanged | Disabled still alpha-halves the resolved colour. |
| `text_edit_*`, `scroll_area_chrome`, `label`, `text_edit_read_only` | unchanged | None reach for state groups in v1. |

Two new goldens are *not* added — the existing `button_hovered` / `button_pressed` slots regenerate in place.

## Backward compatibility

- **Public API:** breaking. `Palette::color` and `Palette::with_role` signatures change. Project is pre-publish — the standing direction is breaking changes are free.
- **Themes:** any `Palette` built via `with_role(role, color)` chains needs updating. The new `with_role_all_groups(role, color)` shorthand keeps non-stateful slot lines unchanged in spirit; only `Button`, `Base`, and `Highlight` rows grow from 1 to 3 chained calls (or 1 + the per-group setters).
- **`DARK_PALETTE`:** updated in lockstep — Button / Base / Highlight gain explicit `Hover` and `Pressed` cells; the new `FocusRing` slot is seeded to `DodgerBlue` (`#1E90FF`) to match `Highlight × Normal`.
- **`colors_and_type.css`:** new CSS custom properties land alongside the existing eleven: `--qz-button-hover`, `--qz-button-pressed`, `--qz-base-hover`, `--qz-base-pressed`, `--qz-highlight-hover`, `--qz-highlight-pressed`, `--qz-focus-ring`. The existing `--qz-button-hover` (which holds the `#316` 25 %-blend value `#BFDFFF`) updates to `#F0F0F0`.

## Open questions

- **Should `FocusRing` get per-group cells?** I.e. a different ring colour when the widget is also hovered. Default: no — the focus ring is a binary cue, the inner fill already moves under hover. If we ever want it, the `ColorGroup` axis is already there.
- **Should `Base.Hover` / `Base.Pressed` be seeded with meaningful values in v1?** Default: no — editable surfaces (`LineEdit`, `TextEdit`) don't render hover/pressed visuals yet; the cells mirror `Normal` until #406 lands. The matrix infrastructure is ready when that issue picks up.
- **Are the derivation factors (6 % hover, 16 % pressed) the right defaults?** Default: yes — they match Qt Fusion's idle-vs-hover separation in light theme and produce visibly distinct hover/pressed in dark theme. Tunable in implementation; if a future round of design-review wants 8 %/18 % or 5 %/14 %, the change is one line in `Palette::color`. Themes that need different curves shadow the cells via `with_role`.
- **Should the dark theme keep the Qt-traditional "darker on press" convention via overrides?** Default: no — v1 ships pure derivation in both themes. The trade-off is documented in § *Default derivation*. If user feedback says dark pressed reads wrong, the fix is two `with_role` lines in `DARK_PALETTE` and two new named colours (web-searched per § *Naming convention*) — not a formula change.
- **Should `FocusRing` live in the existing `color_role.rs` file or a new module?** Default: in `color_role.rs` next to the other roles. Theme-author mental model is "one enum, one file".

## Reference

Visual: `palette-state-groups-swatches.html` (this folder) — light / dark side-by-side, every cell at 64×64, plus button mocks in `Normal` / `Hover` / `Pressed` and a `Highlight × Pressed` vs. `Highlight × Normal` direct comparison.
