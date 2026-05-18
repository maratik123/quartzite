# Proposal: make the `read_only` overlay actually visible on `Palette::default`

**Affects:** `quartzite-style/src/default_style.rs`, `quartzite-style/tests/snapshots/shared/text_edit_read_only.png`, `quartzite-style/tests/snapshots/shared/line_edit_read_only.png` (if present).

**Type:** behavioural fix to `DefaultStyle`. Public API unchanged; no signatures move.

## Problem

`DefaultStyle::paint<TextEdit>` and `Paint<LineEdit>` currently overlay
the read-only state with:

```rust
let overlay = disabled(palette.color(ColorRole::Window));
painter.fill_rect(geom, &Brush::solid(overlay));
```

`disabled(c)` halves the alpha of `c`. On `Palette::default`, both
`ColorRole::Window` and `ColorRole::Base` resolve to `Color::WHITE`,
so the overlay is `rgba(255,255,255,0.5)` over `Color::WHITE` —
which composites to `Color::WHITE`. The result is **a read-only
field visually identical to a writable one** on the seeded palette.

The snapshot test `text_edit_read_only` passes today only because
the golden PNG _also_ records that identical output. The state is
correct in the type system but invisible to the user.

A theme can rescue this by overriding `Window` or `Base` (the
dark palette proposed in `colors_and_type.css` does so — see
`preview/dark-comp-form.html`), but the framework's own _default_
should ship a distinguishable read-only state.

## Proposal

Tint with the **foreground role** instead of the background role,
and dim the text. Both values are derived through `ColorRole` so
the change re-derives correctly for any palette.

- Background overlay: `WindowText.with_alpha(0.10)`
  - Light: `rgba(0, 0, 0, 0.10)` over `#FFFFFF` → `#E6E6E6`
  - Dark (proposed): `rgba(232, 232, 232, 0.10)` over `#1E1E1E` → `≈#323232`
- Text color when `read_only`: `Text.with_alpha(0.65)`
  - Light: black at 65 % α — clearly dimmed against the gray field
  - Dark: light gray at 65 % α — same effect on dark substrate

Both `LineEdit` and `TextEdit` share the same overlay path; both
are patched in lockstep.

## Diff

```diff
--- a/quartzite-style/src/default_style.rs
+++ b/quartzite-style/src/default_style.rs
@@
 impl Paint<TextEdit> for DefaultStyle {
     fn paint(&self, w: &TextEdit, painter: &mut dyn Painter, palette: &Palette) {
         let geom = w.geometry();
         let font = w.widget_base().font.clone();

         painter.fill_rect(geom, &brush(palette, ColorRole::Base));
         if w.read_only {
-            let overlay = disabled(palette.color(ColorRole::Window));
-            painter.fill_rect(geom, &Brush::solid(overlay));
+            painter.fill_rect(geom, &Brush::solid(read_only_overlay(palette)));
         }
         painter.draw_rect(
             geom,
             &Pen::new(palette.color(ColorRole::Text), 1.0),
             &Brush::solid(Color::TRANSPARENT),
         );
+        let text_color = if w.read_only {
+            palette.color(ColorRole::Text).with_alpha(0.65)
+        } else {
+            palette.color(ColorRole::Text)
+        };
         painter.draw_text_in(
             geom,
             &w.plain_text,
             &font,
-            &brush(palette, ColorRole::Text),
+            &Brush::solid(text_color),
             Alignment::Left,
         );
     }
 }
@@
 impl Paint<LineEdit> for DefaultStyle {
     fn paint(&self, w: &LineEdit, painter: &mut dyn Painter, palette: &Palette) {
         let geom = w.geometry();
         let font = w.widget_base().font.clone();

         painter.fill_rect(geom, &brush(palette, ColorRole::Base));
         if w.read_only {
-            painter.fill_rect(
-                geom,
-                &Brush::solid(disabled(palette.color(ColorRole::Window))),
-            );
+            painter.fill_rect(geom, &Brush::solid(read_only_overlay(palette)));
         }
         painter.draw_rect(
             geom,
             &Pen::new(palette.color(ColorRole::Text), 1.0),
             &Brush::solid(Color::TRANSPARENT),
         );
-        let (text_arg, text_brush) = if w.text.is_empty() && !w.placeholder.is_empty() {
-            (
-                w.placeholder.as_str(),
-                Brush::solid(disabled(palette.color(ColorRole::Text))),
-            )
-        } else {
-            (w.text.as_str(), brush(palette, ColorRole::Text))
-        };
+        let text_role_color = palette.color(ColorRole::Text);
+        let (text_arg, text_brush) = if w.text.is_empty() && !w.placeholder.is_empty() {
+            (
+                w.placeholder.as_str(),
+                Brush::solid(disabled(text_role_color)),
+            )
+        } else if w.read_only {
+            (w.text.as_str(), Brush::solid(text_role_color.with_alpha(0.65)))
+        } else {
+            (w.text.as_str(), Brush::solid(text_role_color))
+        };
         painter.draw_text_in(geom, text_arg, &font, &text_brush, Alignment::Left);
     }
 }
@@
 /// Halves the alpha of `color` to signal the "disabled" visual state.
 ///
 /// With the default palette (all roles fully opaque), maps `1.0 → 0.5`.
 #[inline]
 fn disabled(color: Color) -> Color {
     color.with_alpha(color.a() * 0.5)
 }
+
+/// Returns the read-only overlay brush colour for the current palette.
+///
+/// Tints the editable surface with the foreground role
+/// (`ColorRole::WindowText`) at a low alpha. This guarantees a visible
+/// effect on every palette — even when `Window` and `Base` share a
+/// colour (as on `Palette::default`, where both seed to
+/// [`Color::WHITE`]) — because `WindowText` always carries strong
+/// contrast against `Window` and `Base` by definition.
+///
+/// Compared to the previous overlay (`Window.with_alpha(0.5)`), this
+/// derives from the slot most likely to differ from `Base`, not the
+/// slot most likely to match it.
+#[inline]
+fn read_only_overlay(palette: &Palette) -> Color {
+    palette.color(ColorRole::WindowText).with_alpha(0.10)
+}
```

## Snapshot tests

The committed golden PNGs at
`quartzite-style/tests/snapshots/shared/text_edit_read_only.png`
(and `line_edit_read_only.png` if/when added) will need regenerating
with the new render. Suggested commit message for that:

```
quartzite-style: refresh text_edit_read_only golden after AC overlay change
```

`text_edit_plain.png` is unaffected — the writable branch is unchanged.

## Backward compatibility

- **API:** none of `DefaultStyle`, `Paint`, `Style`, `Palette`, or
  `ColorRole` change.
- **Themes:** any downstream `Palette` override automatically gains
  a visible read-only state. Themes that deliberately want
  read-only to look identical to writable are free to implement
  their own `Paint<TextEdit>` impl and skip the overlay.
- **Disabled / read-only semantics:** unchanged. Disabled still
  α-halves the resolved color; read-only still overlays + dims.

## Reference card

The visual target lives in `preview/comp-text-edit.html` of the
Quartzite design system project. Light and dark pairs render the
target behavior side-by-side.
