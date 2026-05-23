# Proposal: popups, tooltips, and elevation

**Affects:** `quartzite-widgets/src/widgets/{popup,tooltip}.rs`
(new widgets — neither exists today), `quartzite-widgets/src/widget_base.rs`
(`z_order: ZLayer` field), `quartzite-style/src/default_style.rs`
(paint impls for the new widgets), `quartzite-application/src/application.rs`
(layered paint pass).

**Type:** new visual contract on `DefaultStyle` for two new
widget types; new `ZLayer` enum on `WidgetBase`. Public API gains
two widgets and one enum; existing widget signatures unchanged.

**Unblocks:** #408 (multi-pass rendering for popups / tooltips).

## Context

`README.md` § *Borders & strokes* states:

> No drop shadows. No inner shadows. The paint API has no shadow
> primitive.

This rule exists because every framed widget in `DefaultStyle`
sits on the same z-plane. The moment popups and tooltips enter the
picture, the assumption breaks — a `Popup` containing a list of
menu items needs to read as **above** the trigger widget, and a
`Tooltip` needs to read as **above** everything else. Standard GUI
toolkits solve this with shadows. This proposal keeps the rule and
solves the substrate problem two other ways.

## 1. Decision: keep the no-shadow rule

**Keep.** Reasoning:

- The flat aesthetic is the framework's identity. Adding a shadow
  primitive to support two widget types would change the look of
  every theme — themes that opt into shadows on popups would
  inevitably opt into shadows elsewhere.
- The paint API has no shadow primitive. Adding `draw_shadow` is
  a 3- to 4-week piece of work that touches `vello`'s pipeline.
  Substrate separation can be solved with primitives already in
  the API.
- Both alternatives below produce a stacking signal as strong as
  a 4 px soft shadow without inventing one.

The rule is restated verbatim in the updated *Borders & strokes*
section and now linked from the new *Elevation and overlays*
section, which carves out the popup / tooltip exception by
substituting other primitives rather than relaxing the ban.

## 2. Substrate separation without shadows

Two mechanisms, one per widget:

| Widget | Mechanism | Why |
|---|---|---|
| `Popup`   | **2 px `WindowText` border** (heavier than the 1 px widget border) + reserved inset from trigger | Border-weight contrast is the only flat primitive available that reads as "this is a different surface" |
| `Tooltip` | **Inverted fill** — `WindowText` background, `Window` foreground, no border | Role swap produces full-luminance contrast against any palette substrate. Border becomes redundant — the colour boundary is the border. |

Both mechanisms re-use existing `ColorRole` slots. No new role is
needed and no new primitive enters the paint API.

A side effect of the heavier border on `Popup`: stacked popups
(nested submenus) get **3 px**, then **4 px**, then capped — the
border-weight reads as a literal depth measure. This is described
in the open-questions section as a deferred extension; the v1
spec is one popup at a time.

## 3. `Popup` chrome

Used for: combobox dropdowns, context menus, autocomplete lists.

```text
+==========================+   <-- 2 px WindowText
|| Item 1                 ||
|| Item 2 (hover)         ||   <-- inner row painted Highlight × Normal
|| ─────────────────────  ||   <-- 1 px WindowText separator (optional)
|| Item 3                 ||
+==========================+
```

| Token | Value |
|---|---|
| Fill                | `ColorRole::Window` |
| Border              | **2 px `WindowText`** (twice the widget-frame weight) |
| Border radius       | 0 px |
| Outer padding       | 0 px (rows carry their own) |
| Row hover fill      | `Highlight × Normal` |
| Row hover text      | `HighlightedText` |
| Row pressed fill    | `Highlight × Pressed` |
| Row disabled        | row text × α 0.5 (no fill change) |
| Separator           | 1 px `WindowText` horizontal line, full width |
| Inset from trigger  | **4 px** below the trigger's bottom edge (or above, if no room below) |
| Min width           | `max(trigger.width(), text_max_width)` |
| Max width / height  | unconstrained; clipped to window bounds |

Inset from trigger matters: a popup that touches the trigger
visually merges with it. The 4 px gap reads as "separate surface
floating on top". Below-vs-above is a placement decision driven by
available window space, not a visual axis.

## 4. `Tooltip` chrome

Used for: short hover-after-delay strings (≥ 500 ms hover; no
click).

```text
+--------------------------+
|  Tooltip text here       |   <-- white-on-black on light theme
+--------------------------+        dark-on-light on dark theme
```

| Token | Value |
|---|---|
| Fill                | `ColorRole::WindowText` (inverted) |
| Text                | `ColorRole::Window` (inverted) |
| Border              | **none** — the inverted fill is the border |
| Border radius       | 0 px |
| Padding             | 4 px vertical, 8 px horizontal |
| Font                | `Font::default()` — same family/size as the framework |
| Max width           | 280 px before wrap (heuristic; theme-overridable via `Tooltip::max_width`) |
| Inset from anchor   | 4 px from the anchor widget's bottom edge |

The role swap is the only mechanism — no border, no second-pass
overlay. On the default light palette the tooltip is black with
white text; on the proposed dark palette it is `#E8E8E8` Mercury
with `#2B2B2B` Mine Shaft text. Both reads as strongly elevated
against their respective substrates.

Disabled tooltips don't exist — tooltips don't have enabled state
because they don't accept events. Tooltips never gain focus, so
the focus-ring overlay never applies.

## 5. Z-order

**Explicit field**, not implicit overlay pass. Add a new enum on
`WidgetBase`:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ZLayer {
    Content = 0,    // default — every existing widget
    Overlay = 1,    // popups
    Tooltip = 2,    // tooltips
}

pub struct WidgetBase {
    // ... existing ...
    pub z_order: ZLayer,
}
```

Three layers — ascending, content at the bottom. The paint pass in
`Application` walks the widget tree once per layer, in
ascending order. Within a layer the existing depth-first traversal
order is preserved. `Popup` defaults to `Overlay`; `Tooltip`
defaults to `Tooltip`; everything else defaults to `Content`.

### Why explicit, not implicit

An implicit "popup-detect" overlay pass would have to walk the
tree looking for popup widgets and re-paint them last. That is
the same algorithm as a layered pass, with the addition of a type
test. Making the field explicit:

- Allows downstream apps to put _their own_ widgets above
  `Content` without making them `Popup` subclasses.
- Makes the paint order deterministic and inspectable — `z_order`
  is a `pub` field, the layer order is fixed at three.
- Decouples the visual layer from the widget hierarchy. A
  `Popup` can be a child of any `Container` in the tree and still
  paint on top.

### Three layers, not more

`Content` / `Overlay` / `Tooltip` exhausts the cases the
built-ins need:

- Modal dialogs share `Overlay` with popups — they trigger an
  app-level cursor (`Wait`) and a backdrop, both handled
  separately from paint order.
- "Above the tooltip" has no use case in the built-in set. If
  one emerges, the enum grows; the public API will absorb
  another `ZLayer` variant.

## Diff sketch

```diff
--- a/quartzite-widgets/src/widget_base.rs
+++ b/quartzite-widgets/src/widget_base.rs
@@ pub struct WidgetBase {
     pub font: Font,
     pub cursor: Option<CursorIcon>,
+    pub z_order: ZLayer,
 }
+
+#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
+pub enum ZLayer { Content = 0, Overlay = 1, Tooltip = 2 }
+
+impl Default for ZLayer { fn default() -> Self { ZLayer::Content } }

--- a/quartzite-application/src/application.rs
+++ b/quartzite-application/src/application.rs
@@ fn paint_window(...) {
-    paint_tree(root, painter, palette);
+    for layer in [ZLayer::Content, ZLayer::Overlay, ZLayer::Tooltip] {
+        paint_tree_layer(root, painter, palette, layer);
+    }
 }

--- a/quartzite-style/src/default_style.rs
+++ b/quartzite-style/src/default_style.rs
@@
+impl Paint<Popup> for DefaultStyle {
+    fn paint(&self, w: &Popup, painter: &mut dyn Painter, palette: &Palette) {
+        let geom = w.geometry();
+        painter.fill_rect(geom, &brush(palette, ColorRole::Window));
+        painter.draw_rect(geom,
+            &Pen::new(palette.color(ColorRole::WindowText), 2.0),
+            &Brush::solid(Color::TRANSPARENT));
+        // rows are painted by Popup's children (typically Buttons or Labels)
+    }
+}
+
+impl Paint<Tooltip> for DefaultStyle {
+    fn paint(&self, w: &Tooltip, painter: &mut dyn Painter, palette: &Palette) {
+        let geom = w.geometry();
+        painter.fill_rect(geom, &brush(palette, ColorRole::WindowText));
+        painter.draw_text_in(geom, &w.text, &w.widget_base().font,
+            &brush(palette, ColorRole::Window), Alignment::LeftTop);
+    }
+}
```

## Snapshot tests

New golden PNGs to commit:

- `popup_idle.png` — 4-row popup, no hover.
- `popup_row_hover.png` — middle row hovered.
- `popup_with_separator.png` — three rows with a separator between rows 2 and 3.
- `tooltip_short.png` — single-line tooltip.
- `tooltip_wrap.png` — two-line tooltip clamped at 280 px max-width.
- `popup_dark.png` / `tooltip_dark.png` — dark-theme variants.

## Reference cards

- `preview/comp-popup.html` — popup chrome, row states, separator, light + dark.
- `preview/comp-tooltip.html` — tooltip chrome, wrap, light + dark.

## Open questions / follow-ups

- **Nested popups (submenu trees).** v1 spec is one popup at a
  time. Nested popups would inherit `ZLayer::Overlay`; visual
  separation between popup levels is the open question. Suggested:
  bump the border to 3 px on level-2, cap at 3 px (further levels
  share the weight). Logged for the #408 PR's stretch goal.
- **Popup dismiss behaviour.** Click-outside vs. press-and-drag-
  select. Pure input handling; not visual. Logged separately.
- **Tooltip arrows / pointer triangles.** Some toolkits draw a
  small triangle pointing at the anchor. Rejected here: the flat
  no-shadow language reads as "card", and cards in flat design
  don't point. Tooltips appear at a fixed inset; the inset says
  which widget they belong to.
- **Tooltip delay.** 500 ms default; configurable per-`Application`.
  Not a visual question; logged for the #408 PR.
- **Modal dialog backdrop.** Out of scope. If `Dialog` ships, the
  backdrop tinted overlay is its own design question — likely a
  `Window × α 0.5` fill on a fourth `ZLayer::Backdrop` between
  `Content` and `Overlay`.
- **Tooltips that contain rich text.** Out of scope. v1 tooltips
  are plain strings only.
