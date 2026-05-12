# Default Style content for Button / Label / TextEdit / ScrollArea

**Source:** issue #290
**Date:** 2026-05-13
**Tracked in:** #290

> Surfaced from `ai-docs/deferred/widget-backlog.md`. Source spec: [paint-style spec](done/2026-05-09-paint-style.spec.md). With the `Style` trait being generic-only, design a single concrete default `Style` struct shipped in `quartzite-style` whose `draw_widget` covers `Button` / `Label` / `TextEdit` / `ScrollArea`.

## Scope

Ship one concrete `Style` implementation inside `quartzite-style` whose `draw_widget` paints the four widgets the paint-style spec named:

- `quartzite-widgets::Button`
- `quartzite-widgets::Label`
- `quartzite-widgets::TextEdit`
- `quartzite-widgets::ScrollArea`

Concretely:

- New struct `DefaultStyle` (`pub struct DefaultStyle;` — zero-sized) in a new module `quartzite-style/src/default_style.rs`, re-exported from `quartzite-style/src/lib.rs` as `pub use default_style::DefaultStyle;`.
- `impl Style for DefaultStyle` whose `draw_widget` body routes on the runtime type of the `&dyn AsWidget` argument and dispatches to one private inherent method per known widget type. Routing uses the `AsObject::as_any()` upcast already provided by the `Extend` macro (via `widget.object_base()` / `widget` → upcast → `as_any().downcast_ref::<T>()`), since `AsWidget` implementors are `AsObject` implementors.
- Per-widget rendering primitives (all flat, no shadows / gradients / borders beyond a 1 px outline):
  - **Label** — `fill_rect` of `geometry()` with `Palette[ColorRole::Window]` then `draw_text_in(geometry(), &label.text, &font, brush(WindowText), label.alignment)`. The font comes from `widget_base().font` (the existing `Arc<Font>` slot).
  - **Button** — `fill_rect(geometry(), Palette[ColorRole::Button])`, 1 px outline via `draw_rect(geometry(), Pen::new(palette[ButtonText], 1.0), Brush::solid(transparent))`, centred `draw_text_in(geometry(), &button.text, &font, brush(ButtonText), Alignment::Center)`. `checked == true` swaps Button↔Highlight and ButtonText↔HighlightedText. `is_enabled() == false` halves the alpha of the background and text brushes (the standard "disabled" cue).
  - **TextEdit** — `fill_rect(geometry(), Palette[ColorRole::Base])`, 1 px outline using `Palette[ColorRole::Text]`, `draw_text_in(geometry(), &text_edit.plain_text, &font, brush(Text), Alignment::Left)`. `read_only == true` overlays a `Window`-coloured fill at half-alpha so the read-only state is visible.
  - **ScrollArea** — paints the chrome only: `fill_rect(geometry(), Palette[ColorRole::Base])` plus a 1 px outline using `Palette[ColorRole::WindowText]`. The content child (when `content_widget` is `Some`) is **not** recursed into — `Style::draw_widget` receives only `&dyn AsWidget` and has no `WidgetResolver`, so child traversal stays the renderer's job. Scrollbar tracks themselves are *not* drawn in v1; an open question records when they land.
- Common helpers live as private free fns / inherent methods on `DefaultStyle`:
  - `fn brush(palette: &Palette, role: ColorRole) -> Brush` — wraps `Brush::solid(palette.color(role))`.
  - `fn disabled_alpha(c: Color) -> Color` — multiplies alpha by `0.5` (via `Color::with_alpha`).
  - The downcast router — a single function/inherent method whose body is a chain of `if let Some(w) = widget.as_any().downcast_ref::<T>() { ... return; }` arms, plus a fall-through no-op for unknown widget types.
- Unknown-widget fall-through is a deliberate **no-op** (does not touch the painter, does not panic). Documented in code and in the doc comment on `DefaultStyle`.

## Out of scope

- Scrollbar track / thumb rendering on `ScrollArea`. The chrome (background + outline) is all v1 ships. Tracks are deferred (see § Deferred); they need an additional palette role and a thumb-fraction model that this spec does not pin down.
- Recursion into `ScrollArea::content_widget` (or any other child tree) from inside `DefaultStyle`. Style implementors do not own the widget tree; the renderer-side dispatch loop iterates the tree and invokes `Style::draw_widget` per node.
- Per-platform variants (macOS-flavoured, Windows-flavoured) — the source paint-style spec defers these and tracks the work via #284.
- Focus-ring / hover / pressed visual states for `Button`. Only the `checked` and `is_enabled()` cues are wired in v1.
- TextEdit caret blink, selection highlight, scroll offset rendering. Plain-text fill only.
- `Container` and `LineEdit` rendering. Neither is named by the issue body; both fall through the unknown-widget arm and stay no-op until a follow-up spec extends `DefaultStyle`.
- Auto-installing `DefaultStyle` into `StyleRegistry` at process start. The decision recorded under § Key decisions makes registration opt-in; see § Open questions for the deferral note.
- Renderer-side dispatch — *how* `Style::draw_widget` is invoked across the widget tree is `quartzite-renderer`'s problem; see #289.

## Deferred

- Scrollbar track + thumb rendering on `ScrollArea` | needs a per-orientation thumb model + an extra `ColorRole::ScrollBar` slot (or equivalent) | new issue when scrollbar interaction lands
- `Button` hover / pressed / focused visual states | needs hover/focus tracking plumbed through `WidgetExt` | new issue when input plumbing lands
- `TextEdit` caret + selection rendering | needs a selection model + caret blink timer | new issue when text editing lands
- `Container` and `LineEdit` default rendering | not in the issue body's covered set | extend `DefaultStyle` in a follow-up

## Key decisions

| Question | Decision |
|---|---|
| Concrete struct name | `DefaultStyle` (zero-sized: `pub struct DefaultStyle;`). Single per-process value; no configuration knobs in v1. |
| Crate / module location | `quartzite-style/src/default_style.rs`, re-exported as `quartzite_style::DefaultStyle`. |
| Routing mechanism inside `draw_widget` | Chain of `widget.as_any().downcast_ref::<T>()` arms, one per supported widget type. `as_any()` is already on every `AsWidget` via the `Extend` macro / `AsObject` supertrait. No new trait, no visitor. |
| Unknown widget fall-through | Silent no-op — does not call any `Painter` method, does not panic. AGENTS.md *API Naming* + *Library safety idioms*: non-panicking by default. Documented on `DefaultStyle` and asserted by a unit test that feeds it a bare `WidgetBase`. |
| `ScrollArea::content_widget` traversal | Out of scope for `DefaultStyle::draw_widget`. Style impls do not own the widget tree; the renderer walks children. |
| Visual idiom | Flat rectangles, 1 px outlines, palette-driven colours. No gradients, shadows, or rounded corners (matches AGENTS.md *Rust idioms*: do not justify visual design with reference to GUI frameworks). |
| Colour-role wiring | `Label` → `Window` / `WindowText`. `Button` (idle) → `Button` / `ButtonText`; `Button` (checked) → `Highlight` / `HighlightedText`. `TextEdit` → `Base` / `Text`. `ScrollArea` → `Base` background + `WindowText` outline. |
| `is_enabled()` rendering | Applies half-alpha to the fill and text brushes (via `Color::with_alpha(c.a() * 0.5)`). Single helper; same treatment for every supported widget. |
| Font source | `widget.widget_base().font` (the existing `Arc<Font>`). `DefaultStyle` does not own a font — it reads the widget's font slot. |
| `Brush` for outlines | `Brush::solid(Color::TRANSPARENT)` paired with a `Pen` whose colour comes from the palette. Painter contract is `draw_rect(rect, &pen, &brush)`. |
| `DefaultStyle` `Default` impl | `impl Default for DefaultStyle { fn default() -> Self { DefaultStyle } }` (consistent with the zero-sized shape). Doc comment cross-links to `StyleRegistry::set_style`. |
| Registration default | Opt-in. `quartzite-style` does **not** auto-install `DefaultStyle` into `StyleRegistry` — neither at process start nor on the first `StyleRegistry::try_style()` miss. Callers (typically `quartzite-renderer` setup, application `main`, or tests) construct `DefaultStyle` and register it explicitly: `StyleRegistry::set_style(Box::new(DefaultStyle));`. (Resolved round 1.) |
| Per-widget dispatch surface | Stays inside `DefaultStyle` as `fn draw_button(&self, ...)`, `fn draw_label(&self, ...)`, etc. They are **inherent private methods on `DefaultStyle`**, not trait methods — the source spec is explicit that per-widget primitives are not on `Style`. |
| Public surface added | `DefaultStyle` struct, `impl Default for DefaultStyle`, `impl Style for DefaultStyle`. Nothing else. Every per-widget helper stays `pub(crate)` or private. |

## Technical constraints

- `quartzite-style` already depends on `quartzite-paint-api`, `quartzite-paint`, `quartzite-widgets`, `quartzite-style-types`. `DefaultStyle` needs nothing further — every type used (`Color`, `Pen`, `Brush`, `Painter`, `Font`, `AsWidget`, concrete widget types, `Palette`, `ColorRole`, `Alignment`, `Rect`) is already in the graph.
- `DefaultStyle: Send + Sync` — required by the `Style: Send + Sync` bound. Zero-sized + no interior state makes this automatic.
- `quartzite-style` is **not** `no_std` (the source spec pins this — global registry uses `std`); `DefaultStyle` is free to use `std` if needed, though the implementation as scoped above does not require it beyond what the crate already pulls.
- Routing uses `widget.as_any().downcast_ref::<T>()` — `as_any` lives on `AsObject` and `AsWidget` upcasts there via the `Extend` macro, so no extra trait surface is added.
- Doc gate: every new public item carries `///` first-line docs, `# Examples` for items with single-line bodies / where useful, and `# Parameters` when applicable per workspace doc convention.
- Lint gate: `cargo clippy --workspace -- -D warnings` clean.
- `widget_base().enabled` is read via `WidgetExt::is_enabled()`; widget-specific fields (`Button::text`, `Button::checked`, `Label::text`, `Label::alignment`, `TextEdit::plain_text`, `TextEdit::read_only`) are public on the concrete types (verified by reading the current sources of each widget).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite_style::DefaultStyle` is a zero-sized, `Default`-implementing struct that satisfies `Style + Send + Sync`. A unit test boxes it as `Box<dyn Style>` and asserts `Send + Sync` on the trait object. |
| AC2 | `DefaultStyle::default().draw_widget(&Button::new("OK".into()), &mut recording_painter, &Palette::default())` records at least one `fill_rect` call covering the widget's `geometry()` and one `draw_text_in` call whose `text` argument equals `"OK"` and whose `alignment` is `Alignment::Center`. A recording painter fixture (lives in the test module) captures the method calls in order. |
| AC3 | `DefaultStyle::default().draw_widget(&Label::new("hi".into()), …)` records a `fill_rect` over `geometry()` and a `draw_text_in` whose `alignment` equals the label's `alignment` (defaulted `Alignment::Left`). |
| AC4 | `DefaultStyle::default().draw_widget(&TextEdit::new(), …)` with `plain_text == "abc"` records a `fill_rect` over `geometry()` using the `Base` palette colour and a `draw_text_in` whose `text` equals `"abc"`. |
| AC5 | `DefaultStyle::default().draw_widget(&ScrollArea::new(), …)` records the chrome (a `fill_rect` plus a `draw_rect` outline) and **does not** record any `draw_text*` call. The content child is not traversed (no recursive invocation visible to the painter). |
| AC6 | Routing to an unknown widget type (a bare `WidgetBase`) does **not** record any painter call and does **not** panic — the recording painter's captured-call list is empty after the call. |
| AC7 | A `Button` with `checked == true` and a `Button` with `checked == false` produce different `fill_rect` colours when drawn with the same `Palette::default()` (checked uses `Highlight`, idle uses `Button`). The assertion compares the captured `Brush` colours, not the rendered pixels. |
| AC8 | A `Button` with `is_enabled() == false` produces a `fill_rect` whose brush alpha equals half the alpha that the same button drawn with `is_enabled() == true` produces. Same comparison applies to the text brush. |
| AC9 | `DefaultStyle` and its routing arms compile cleanly under `cargo clippy --workspace -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. |
| AC10 | `StyleRegistry::set_style(Box::new(DefaultStyle))` followed by `StyleRegistry::try_style()` returns `Some(_)` whose `draw_widget` produces the same recorded calls as AC2 for the same `Button`. (Demonstrates `DefaultStyle` is usable through the registry; does not change registry behaviour.) |

## Open questions

- Scrollbar track / thumb rendering on `ScrollArea` is intentionally deferred; design will revisit once scrollbar interaction semantics are pinned (likely after #230 — `Slider`).
- Hover / pressed / focused visual states on `Button` await an input-plumbing pass (no `WidgetBase::hovered` or `pressed` flags exist today).
- The `Container` and `LineEdit` arms — both fall through the unknown-widget no-op until a follow-up plan extends `DefaultStyle`; not a blocker for this issue.

## Resolution log

- **Round 1, Q1 — Registration default.** Resolved: **opt-in only**. `quartzite-style` does not auto-install `DefaultStyle` on first `StyleRegistry::try_style()` miss or on crate load. Callers register explicitly via `StyleRegistry::set_style(Box::new(DefaultStyle));`. Recorded in § Key decisions and § Out of scope; AC10 exercises the explicit-registration path.
