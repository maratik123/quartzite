# Proposal: cursor-shape mapping for built-in widgets

**Affects:** `quartzite-widgets/src/widgets/{button,label,line_edit,text_edit,scroll_area,container}.rs`
(per-widget cursor defaults), `quartzite-widgets/src/widget_base.rs`
(`WidgetBase::cursor` override semantics + accessor),
`quartzite-application/src/application.rs` (cursor resolution and
the `winit::window::CursorIcon` set call on the window).

**Type:** new visual contract — what `CursorIcon` each built-in
widget reports under each interaction state. Public API gains an
override accessor on `WidgetBase`; no existing signatures move.

**Unblocks:** #404 (cursor-shape change on hover).

## Context

`WidgetBase::cursor` exists in the source today as a field, but
has no mutation path, no per-widget default, and no rule for how
it interacts with widget identity or interaction state. The host
window therefore never calls `winit::window::Window::set_cursor()`
with anything but the default arrow.

The question framing in issue #404 ("cursor-shape change on
hover") is slightly misleading. The cursor in `DefaultStyle` does
**not** change on hover. It changes when the user moves the
pointer **into a widget whose type defaults to a different shape**
— a `LineEdit` reports `Text` whether the user has just entered
the widget or has been sitting in it for ten seconds. Mere hover
on a `Button` does not change the cursor at any point.

This proposal nails down the four pieces: (1) the per-widget
default, (2) the state-driven overrides, (3) the precedence
between override and default, (4) the bounded subset of
`winit::window::CursorIcon` the design system endorses.

## 1. Per-widget default cursor

| Widget | Default shape | `winit` ident |
|---|---|---|
| `Button`     | arrow      | `CursorIcon::Default` |
| `Label`      | arrow      | `CursorIcon::Default` |
| `LineEdit`   | I-beam     | `CursorIcon::Text`    |
| `TextEdit`   | I-beam     | `CursorIcon::Text`    |
| `ScrollArea` | arrow      | `CursorIcon::Default` |
| `Container`  | arrow      | `CursorIcon::Default` |

Notes:

- **`Button`** does not switch to `Pointer` / "hand" on hover.
  That is web convention. Native GUI convention — Qt, GTK, AppKit,
  Win32 — is that a button keeps the arrow. Adopting the hand
  cursor would imply "this is a link", which buttons aren't.
- **`Label`** keeps the arrow even when it contains text. Labels
  are not editable; there is no insertion point to advertise. If
  a downstream app wants selectable label text, the route is to
  swap the `Label` for a `read_only` `LineEdit` (which inherits
  the I-beam correctly via this table).
- **`ScrollArea`** reports the arrow for its full rect including
  the track and thumb at rest. The state-driven override below
  takes over only during a drag.
- **`Container`** is a generic parent — the cursor reported is
  whatever the child under the pointer reports, falling back to
  the container's `Default` only when no child claims the point.

## 2. State-driven overrides

Two — and only two — interaction states override the widget-type
default:

| State | Shape | Applies to |
|---|---|---|
| `enabled == false`        | `CursorIcon::NotAllowed` | any widget |
| `ScrollArea` thumb drag   | `CursorIcon::Grabbing`   | `ScrollArea` only, while `is_pressed` on the thumb |

That is the complete list. Pointer-over-thumb at rest is **not** a
state override — it reports the widget default (arrow). The
`Grabbing` cursor appears only after press-down on the thumb and
holds until the press release.

`NotAllowed` for disabled widgets matches the disabled-widget
visual treatment (×0.5 alpha) — the cursor reinforces the visual
"this widget will not respond" signal. It applies regardless of
the widget's normal default, including disabled `LineEdit` (the
I-beam is replaced by `NotAllowed`, _not_ overlaid on it).

## 3. Hover-no-change rule (explicit)

> **Hover alone does not change cursor shape.** Cursor follows
> widget-type identity and the two state overrides above —
> nothing else.

Implications:

- A `Button` hover changes the **button's** fill (`Button × Hover`)
  but does **not** change the cursor.
- A `ScrollArea` thumb hover changes the **thumb's** fill
  (`ScrollBar × Hover`) but does **not** change the cursor. The
  arrow stays.
- Hover over a disabled widget shows the `NotAllowed` cursor
  because the widget is disabled, _not_ because it is hovered.
  Move the pointer outside the widget and the cursor returns to
  `Default`; move it inside and `NotAllowed` returns immediately.

The rule simplifies cursor resolution to a pure function of
`(widget_type, enabled, is_pressed_on_drag_handle, override)`. No
hover bit is read by the resolver.

## 4. Endorsed `CursorIcon` subset

`winit::window::CursorIcon` enumerates ~36 shapes. The design
system endorses **six**:

| Shape | Use |
|---|---|
| `Default`       | every widget that is not a text editor |
| `Text`          | `LineEdit`, `TextEdit` |
| `NotAllowed`    | any widget when `enabled == false` |
| `Grabbing`      | `ScrollArea` thumb during drag |
| `Wait`          | app-level — long synchronous operation, modal block |
| `Progress`      | app-level — long async operation, UI still interactive |

`Wait` and `Progress` are scoped to **application-wide** overrides
(set on `Application`, not on any individual widget). They are not
in the per-widget table.

Shapes deliberately not endorsed by `DefaultStyle`:

- **`Pointer`** ("hand") — pseudo-web convention; not native.
- **`Move` / `Grab`** — there is no draggable widget in the
  built-in set apart from the scrollbar thumb, which uses
  `Grabbing` only while pressed (not `Grab` while hovered).
- **`ColResize` / `RowResize` / `EwResize` / `NsResize` / etc.**
  — no resize handle widget ships in the built-ins. Reserved for
  whichever PR adds `Splitter`.
- **`Crosshair` / `Help` / `Cell`** — domain-specific; downstream
  apps can opt in via the override field below.
- **`ZoomIn` / `ZoomOut` / `Alias` / `Copy` / `NoDrop`** — drag-
  and-drop semantics; the framework has no drag-and-drop layer.

Downstream apps are free to set _any_ `CursorIcon` via the
override — endorsement governs what `DefaultStyle` itself emits.

## 5. Precedence

Cursor resolution at any pointer position runs in this order;
first match wins:

```text
1. Application-wide cursor   (Wait / Progress)
2. widget.enabled == false   →  NotAllowed
3. ScrollArea thumb drag     →  Grabbing
4. WidgetBase::cursor        (explicit per-widget override)
5. Per-widget-type default   (table in §1)
```

In Rust:

```rust
fn resolve_cursor(app: &Application, w: &dyn WidgetExt) -> CursorIcon {
    if let Some(app_cursor) = app.cursor_override() {
        return app_cursor;                              // 1
    }
    if !w.is_enabled() {
        return CursorIcon::NotAllowed;                  // 2
    }
    if let Some(drag) = w.as_drag_handle() {
        if drag.is_pressed() {
            return CursorIcon::Grabbing;                // 3
        }
    }
    if let Some(c) = w.widget_base().cursor {
        return c;                                       // 4
    }
    w.default_cursor()                                  // 5
}
```

## API change

Add a single accessor on `WidgetBase`:

```rust
impl WidgetBase {
    pub fn set_cursor(&mut self, c: Option<CursorIcon>) -> &mut Self {
        self.cursor = c;
        self
    }
}
```

And one trait method on `WidgetExt` (returning `CursorIcon::Default`
by default; per-widget impls override for `LineEdit` / `TextEdit`):

```rust
pub trait WidgetExt {
    // ... existing ...
    fn default_cursor(&self) -> CursorIcon { CursorIcon::Default }
}

impl WidgetExt for LineEdit { fn default_cursor(&self) -> CursorIcon { CursorIcon::Text } }
impl WidgetExt for TextEdit { fn default_cursor(&self) -> CursorIcon { CursorIcon::Text } }
```

No other public surface moves.

## Diff sketch

```diff
--- a/quartzite-widgets/src/widget_base.rs
+++ b/quartzite-widgets/src/widget_base.rs
@@ pub struct WidgetBase {
     pub font: Font,
-    pub cursor: Option<CursorIcon>,   // existed, no mutator
+    pub cursor: Option<CursorIcon>,
 }
+
+impl WidgetBase {
+    pub fn set_cursor(&mut self, c: Option<CursorIcon>) -> &mut Self { self.cursor = c; self }
+}

--- a/quartzite-widgets/src/widgets/line_edit.rs
+++ b/quartzite-widgets/src/widgets/line_edit.rs
@@ impl WidgetExt for LineEdit {
     // ...
+    fn default_cursor(&self) -> CursorIcon { CursorIcon::Text }
 }
```

`Application`'s pointer-motion handler grows a call to
`resolve_cursor` and calls `window.set_cursor` only when the
resolved icon differs from the last one set (to avoid spamming the
winit event loop).

## No preview HTML

Cursor shape is not a paint output — it is a host-window call.
The README's *Cursor shapes* section carries the per-widget table
in prose; there is no HTML mock to render. Designers verifying
the behaviour should run `ui_kits/widgets/index.html` under a
production Quartzite build; HTML mocks cannot reproduce `winit`'s
cursor stack.

## Open questions / follow-ups

- **Application-wide `Wait` / `Progress` API.** Out of scope here.
  The resolver in §5 references `app.cursor_override()` as a
  placeholder; the actual setter (`Application::push_cursor` +
  `pop_cursor` scoped guard, vs. a single `Application::cursor_override: Option<CursorIcon>` field) is logged for the #404 PR.
- **Per-region cursor inside a single widget.** A `TextEdit` with
  an embedded image inside the text flow ought to report `Default`
  over the image and `Text` elsewhere. The framework has no inline-
  embed primitive; logged for if/when one ships.
- **Resize-handle widgets** (`Splitter`, resizable `Container`).
  Out of scope; would add `ColResize` / `RowResize` / `NsResize` /
  `EwResize` to the endorsed subset.
- **Drag-and-drop cursors** (`Copy`, `NoDrop`, `Alias`). Out of
  scope; the framework has no DnD layer.
