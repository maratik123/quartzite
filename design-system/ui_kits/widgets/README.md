# Quartzite Widgets — UI Kit

High-fidelity HTML/JSX recreation of the six built-in widgets in
[`quartzite-widgets`](https://github.com/maratik123/quartzite/tree/master/quartzite-widgets)
painted under [`DefaultStyle`](https://github.com/maratik123/quartzite/blob/master/quartzite-style/src/default_style.rs).

These are **cosmetic recreations**. They follow the real
property/signal/slot shape (`text`, `clicked`, `set_text`, etc.) so
the demo reads like a Quartzite app, but no event loop, no parley
text shaping, and no GPU rendering are involved.

## Components

| File | Mirrors |
|---|---|
| `Button.jsx`     | `quartzite-widgets::Button` + `DefaultStyle::paint::<Button>` |
| `Label.jsx`      | `quartzite-widgets::Label` + `DefaultStyle::paint::<Label>` |
| `LineEdit.jsx`   | `quartzite-widgets::LineEdit` + `DefaultStyle::paint::<LineEdit>` |
| `TextEdit.jsx`   | `quartzite-widgets::TextEdit` + `DefaultStyle::paint::<TextEdit>` |
| `ScrollArea.jsx` | `quartzite-widgets::ScrollArea` + `DefaultStyle::paint::<ScrollArea>` |
| `Container.jsx`  | `quartzite-widgets::Container` + `DefaultStyle::paint::<Container>` |
| `WindowFrame.jsx`| Generic top-level surface chrome (`Window` role) |
| `App.jsx`        | Interactive demo composing the above |

Painting rules mirrored verbatim:

- **1 px outline** on every framed widget. `border: 1px solid #000`.
- **Hover** = role's `Hover` cell. Derived: `c.blend(WindowText.Normal, 0.06)`. For default light palette, `Button × Hover` ≈ `#F0F0F0`.
- **Pressed** = `Highlight × Pressed` fill + `HighlightedText` foreground. Derived: `c.blend(WindowText.Normal, 0.16)`, lands on `#006CD6` for default light.
- **Checked** = `Highlight × Normal` fill + `HighlightedText` foreground.
- **Focus** = additive 2 px `ColorRole::FocusRing` outline, never alpha-halved. Defaults to `Highlight`.
- **Disabled** = α × 0.5 on the resolved color.
- **Read-only** = `Base` fill with a 50 %-α `Window` overlay.
- **Sharp corners.** No radii, no shadows.

## How the demo composes

`App.jsx` renders a `WindowFrame` containing a `Container` with
three groups, all built from the kit's own components:

1. **Counter** — paraphrases `examples/combined.rs`. A `Label`
   shows the current count; three buttons drive `increment`,
   `decrement`, and a checkable `pause` toggle. Each click pushes
   a row into the signal log so you can see the same signals the
   real `Counter` object emits.
2. **Form** — a small "new note" form: two `LineEdit`s (one with
   a placeholder, one toggleable to read-only), one `TextEdit`,
   and a `Submit` button. Submitting pushes the note into the
   list below and clears the form.
3. **Notes list** — a `ScrollArea` wrapping a stack of submitted
   note `Label`s. Empty state is left blank by design — the real
   `ScrollArea` paints chrome only.

A bottom `ScrollArea` panel ("Signal log") shows every emitted
signal in order, mirroring how `signals_slots.rs` listens with
`Signal::connect`.

## What is _not_ here

- Layout primitives. `BoxLayout` and `GridLayout` are exposed via
  CSS `display: flex` and `display: grid` with `gap`. The grid
  spans, stretch factors, and margin handling of the real layouts
  are not re-implemented.
- Cross-thread signal dispatch and the `Application` event loop.
- Snapshot serialization (`serde` feature in `quartzite-core`).

If you are designing _on top of_ Quartzite, lift visual values
from this kit but always cross-check the live framework for
behavior.
