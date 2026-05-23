# Proposal: scrollbar chrome — `ColorRole::ScrollBar`, track + thumb geometry

**Affects:** `quartzite-style-types/src/color_role.rs` (new `ScrollBar`
variant + light/dark seed entries in `Palette::default`),
`quartzite-style/src/default_style.rs` (paint impl for `ScrollArea`),
`quartzite-widgets/src/widgets/scroll_area.rs` (track + thumb fields,
hit-test plumbing), `quartzite-style/tests/snapshots/shared/*` (new
golden PNGs).

**Type:** new `ColorRole` slot + new visual contract on
`DefaultStyle::paint<ScrollArea>`. The `ColorRole` enum gains one
variant; `Palette` gains one row. The Rust-side `ColorRole` change
is **out of scope here** — flagged below as the trigger for the
implementation PR.

**Unblocks:** #315 (scrollbar track + thumb rendering on ScrollArea).

## Context

`DefaultStyle::paint<ScrollArea>` today paints `Base` + a 1 px
`Text` outline and stops. `ScrollArea` carries `horizontal_policy`,
`vertical_policy`, and `content_widget`; there is no thumb, no
track, no hit testing. The existing mock at
`preview/comp-scroll-area.html` paints a placeholder thumb in pure
black at α 0.85 — its own comment flags this as "a documentation
hint — the framework does not yet render a bar primitive". That
gap is what this block closes.

`ColorGroup` (#402) provides the `Normal` / `Hover` / `Pressed`
axis already. The scrollbar thumb is a textbook user of all three.

## 1. New `ColorRole::ScrollBar`

Add one slot. Track is **not** a new role — the track lane is
painted with `ColorRole::Base` (the same fill the input widgets
use), separated from content by a 1 px `WindowText` stroke. Only
the thumb needs its own colour.

```rust
#[repr(u8)]
pub enum ColorRole {
    Window = 0,
    WindowText,
    Button,
    ButtonText,
    Base,
    Text,
    Highlight,
    HighlightedText,
    Link,
    LinkVisited,
    BrightText,
    FocusRing,      // #402
    ScrollBar,      // <— new
}
```

`ScrollBar` is appended to the end so existing
`ColorRole as usize` indexing is stable. `Palette`'s backing array
grows by one slot.

### Seeds

| Role | Light seed | Dark seed | Common name |
|---|---|---|---|
| `ScrollBar` | **`#C8C8C8`** | **`#5A5A5A`** | Silver Sand / Mortar |

Rationale:

- **Light `#C8C8C8`**: a mid-gray that reads as "interactive
  affordance" against `#FFFFFF` `Base`, without competing with the
  black 1 px outline or the sky-blue `Highlight`. ΔL* ≈ 18 from
  pure white — visible but quiet.
- **Dark `#5A5A5A`**: the mirror move on a dark substrate. ΔL* ≈
  18 from `#1E1E1E` `Base`, so the thumb pops on dark by the same
  perceptual margin it does on light.
- Neither seed reuses an existing palette colour. Reusing
  `WindowText × disabled` would collapse the disabled-text colour
  and the idle-scrollbar colour into the same value, which makes
  a disabled scrollbar with disabled text indistinguishable.

### Derived state

`ScrollBar × Hover` and `ScrollBar × Pressed` follow the standard
`ColorGroup` derivation from #402:

| Cell | Light | Dark |
|---|---|---|
| `ScrollBar × Normal`  | `#C8C8C8` | `#5A5A5A` |
| `ScrollBar × Hover`   | `#BCBCBC` | `#636363` |
| `ScrollBar × Pressed` | `#A8A8A8` | `#717171` |

(`Hover = c.blend(WindowText.Normal, 0.06)`; `Pressed = c.blend(WindowText.Normal, 0.16)` — same formula every other role uses.)

## 2. Track + thumb geometry

All values are integer pixels — no fractional widths.

| Token | Value | Notes |
|---|---|---|
| Track width                 | **12 px** | applies to both axes |
| Thumb min-length            | **24 px** | clamps thumb size when content_size / viewport ≫ 1 |
| Thumb radius                | **0 px**  | per the "no rounded corners" rule |
| Thumb inset (perpendicular) | **0 px**  | thumb fills the full 12 px track width |
| Thumb inset (along axis)    | **0 px**  | thumb top/bottom are flush at extremes |
| Track-to-content stroke     | **1 px `WindowText`** | divides content rect from track lane |
| Outer frame                 | **1 px `WindowText`** | unchanged from current `ScrollArea` chrome |
| Corner block (both axes)    | **12 × 12 `Base` fill** | bottom-right corner when both bars present; non-interactive |

The 12 px track width is the same width the existing
`comp-scroll-area.html` placeholder uses, so any downstream code
that hard-codes 12 keeps working.

### Reserved space

The `ScrollArea` content rect is computed **excluding** the
scrollbar lanes:

```rust
let v_lane = if needs_vertical   { 12 } else { 0 };
let h_lane = if needs_horizontal { 12 } else { 0 };
let content_rect = Rect::new(
    Point::new(geom.x() + 1, geom.y() + 1),
    Size::new(geom.w() - 2 - v_lane, geom.h() - 2 - h_lane),
);
```

Overlay scrollbars (Mac-style auto-hiding bars floating over
content) are explicitly rejected — they require a fade animation
to feel correct, and the framework has no animation primitive.

## 3. State variants

```text
                    track          thumb
idle              Base           ScrollBar × Normal
hover (over bar)  Base           ScrollBar × Hover
pressed (drag)    Base           ScrollBar × Pressed
disabled          Base × α0.5    ScrollBar × Normal × α0.5
```

Hover precedence: hover over the **thumb** triggers the hover
state. Hover over the track-only area (above/below the thumb) does
**not** — track is a click target for paging, not a hover target.
This matches Qt and avoids a flickery hover that follows the mouse
across the empty track.

Pressed state holds for the full drag, regardless of whether the
mouse is still over the thumb mid-drag. Same precedence rule as
`Button`: pressed > checked > hovered > idle.

## 4. Orientation

Vertical and horizontal scrollbars are geometrically symmetric. A
horizontal bar is a vertical bar rotated 90°, same widths and
insets. Both can coexist; when both are present the bottom-right
12 × 12 corner is filled with `Base` (no thumb, no interaction).

```text
+-----------------+--+
|                 |##|   v-track on the right
|   content       |  |
|                 |##|   ## = thumb
|                 |  |
+-----------------+--+
|######           |  |   h-track on the bottom
+-----------------+--+
                    ^^   12×12 Base corner block
```

The 1 px `WindowText` track-to-content stroke runs along the inner
edge of both lanes. The outer 1 px frame is unchanged.

## 5. Visibility rule

`ScrollPolicy` already has three variants in
`quartzite-widgets/src/widgets/scroll_area.rs`:

| Policy | Lane visible | Behaviour |
|---|---|---|
| `AsNeeded` (default) | when `content_size.axis > viewport.axis` | reserved when shown, freed when hidden |
| `AlwaysOn`           | always | lane always reserved, even with no overflow |
| `AlwaysOff`          | never | mouse-wheel still scrolls; lane never reserved |

`AsNeeded` is the default and matches the existing field. The
content rect re-computes when overflow toggles, which can cause a
one-frame layout shift when content grows past the threshold —
this is acceptable and matches Qt. Apps that want zero-shift
behaviour should pin both policies to `AlwaysOn`.

## Diff sketch

```diff
--- a/quartzite-style-types/src/color_role.rs
+++ b/quartzite-style-types/src/color_role.rs
@@ pub enum ColorRole {
     BrightText,
     FocusRing,
+    /// Thumb of `ScrollArea`'s scrollbar.
+    ///
+    /// Track is painted with `ColorRole::Base`; only the thumb
+    /// reads from this slot.
+    ScrollBar,
 }

--- a/quartzite-style-types/src/palette.rs
+++ b/quartzite-style-types/src/palette.rs
@@ impl Default for Palette {
     fn default() -> Self {
         Self::new()
             // ... existing rows ...
+            .with_role(ColorRole::ScrollBar,
+                       Color::new(0.784, 0.784, 0.784, 1.0)) // #C8C8C8
     }
 }

--- a/quartzite-style/src/default_style.rs
+++ b/quartzite-style/src/default_style.rs
@@ impl Paint<ScrollArea> for DefaultStyle {
     fn paint(&self, w: &ScrollArea, painter: &mut dyn Painter, palette: &Palette) {
         let geom = w.geometry();
         painter.fill_rect(geom, &brush(palette, ColorRole::Base));

+        let (v, h) = w.lane_visibility();
+        if v {
+            let track = w.v_track_rect();
+            let thumb = w.v_thumb_rect();
+            painter.fill_rect(track, &brush(palette, ColorRole::Base));
+            // 1 px divider on the inner edge of the lane.
+            painter.draw_line(track.top_left(), track.bottom_left(),
+                              &Pen::new(palette.color(ColorRole::WindowText), 1.0));
+            let group = thumb_group(w);  // Normal / Hover / Pressed
+            painter.fill_rect(thumb, &Brush::solid(palette.color(ColorRole::ScrollBar, group)));
+        }
+        if h { /* symmetric — bottom track + thumb */ }
+
         painter.draw_rect(geom,
             &Pen::new(palette.color(ColorRole::WindowText), 1.0),
             &Brush::solid(Color::TRANSPARENT));
     }
 }
```

## Snapshot tests

New golden PNGs to commit:

- `scroll_area_v_idle.png` — vertical bar, idle.
- `scroll_area_v_hover.png` — vertical bar, hover on thumb.
- `scroll_area_v_pressed.png` — vertical bar, drag.
- `scroll_area_both.png` — vertical + horizontal, 12×12 corner block.
- `scroll_area_no_overflow.png` — `AsNeeded` policy with content
  fitting the viewport; no lanes painted.
- `scroll_area_disabled.png` — whole widget × 0.5 alpha.

## Trigger PR

`ColorRole::ScrollBar` is a public-API change to
`quartzite-style-types`. It cannot land alongside this proposal
(documentation-only); it ships as its own PR with:

- the enum variant addition
- the `Palette::default` row (light)
- the dark-theme override row (in this design system,
  `colors_and_type.css` and the README dark-seed table)
- regeneration of the snapshot suite

This proposal is the trigger for that PR; it should be linked from
the implementation PR's description.

## Reference card

- `preview/comp-scroll-area.html` — full rewrite. Light + dark
  side-by-side. Idle / hover / pressed / disabled / both-axes /
  no-overflow / always-on permutations.

## Open questions / follow-ups

- **Click-on-track paging.** Whether a click in the empty track
  region scrolls one page or jumps the thumb to that position is
  an input-handling decision, not a visual one. Recommended
  default: page-scroll (Qt convention). Logged for the #315 PR.
- **Scroll-wheel acceleration.** Visual is unaffected; logged for
  whichever PR adds wheel handling.
- **Mini-scrollbars** (e.g. an 8 px thumb-only variant for
  embedded contexts). Out of scope; if added later, ride on a
  `ScrollArea::compact: bool` field and reduce all of (track
  width, thumb min, corner block) to 8 px.
- **Track ticks / row markers** (e.g. a Find-in-page yellow tick
  band). Out of scope; would need a separate paint API extension.
