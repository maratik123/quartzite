# Default Style content for Container and LineEdit

**Source:** issue #318
**Date:** 2026-05-15
**Tracked in:** #318

> Surfaced by `/triage` from `ai-docs/deferred/widget-backlog.md`. Source spec: [`2026-05-13-default-style-content.spec.md`](done/2026-05-13-default-style-content.spec.md). v1 `DefaultStyle` covers `Label`, `Button`, `TextEdit`, and `ScrollArea`; `Container` and `LineEdit` currently fall through the unknown-widget arm and stay no-op. This follow-up extends `DefaultStyle` with one arm per widget.

## Scope

Extend `quartzite_style::DefaultStyle` with two additional downcast arms in the existing `draw_widget` router, plus the corresponding private inherent methods, so that:

- `Container` paints chrome — **fill + outline**: `Window` background then a 1 px `WindowText` outline.
- `LineEdit` paints the single-line-text equivalent of the existing `TextEdit` arm: `Base` background, 1 px `Text` outline, single-line `draw_text_in` of `line_edit.text`, with the same `read_only` half-alpha `Window` overlay as `TextEdit`. When `line_edit.text.is_empty() && !line_edit.placeholder.is_empty()`, draw the placeholder text instead of the empty `text`, using a half-alpha `Text` brush (`disabled(palette.color(Text))`), `Alignment::Left`.

Concretely:

- Two new downcast arms added to the existing `if let Some(_) = any.downcast_ref::<T>()` chain inside `impl Style for DefaultStyle::draw_widget` in `quartzite-style/src/default_style.rs`, in the order `Button → Label → TextEdit → ScrollArea → Container → LineEdit`. Order is documented in code; append-only.
- Two new private inherent methods on `DefaultStyle`:
  - `fn draw_container(&self, w: &Container, painter: &mut dyn Painter, palette: &Palette)`
  - `fn draw_line_edit(&self, w: &LineEdit, painter: &mut dyn Painter, palette: &Palette)`
- Per-widget operations:
  - **Container** — `fill_rect(geometry(), brush(Window))` then `draw_rect(geometry(), Pen::new(palette.color(WindowText), 1.0), Brush::solid(Color::TRANSPARENT))`. No text. No recursion into `Container::children` (the renderer walks children — same contract as `ScrollArea`).
  - **LineEdit** — `fill_rect(geometry(), brush(Base))`, the same optional read-only half-alpha `Window` overlay as `TextEdit` (when `line_edit.read_only`), 1 px outline via `draw_rect(geometry(), Pen::new(palette.color(Text), 1.0), Brush::solid(Color::TRANSPARENT))`, then **one** `draw_text_in(geometry(), …, &font, …, Alignment::Left)`:
    - When `line_edit.text.is_empty() && !line_edit.placeholder.is_empty()` → text arg = `&line_edit.placeholder`, brush = `Brush::solid(disabled(palette.color(Text)))`.
    - Otherwise → text arg = `&line_edit.text`, brush = `brush(Text)`. This covers both the non-empty-text path and the both-empty path (the latter records a `DrawTextIn` with `text == ""`, matching v1 `TextEdit` parity).
- Reuse the existing private free helpers `brush(palette, role)` and `disabled(color)` in the same module. The placeholder brush is built inline (`Brush::solid(disabled(palette.color(ColorRole::Text)))`); no new helper required, but design may extract a `placeholder_brush` local fn if it reads cleaner.
- `Container` and `LineEdit` already implement `AsObject` via the `Extend` macro (`object_base()` / `as_any()` are present); no widget-side change.

## Out of scope

- Caret blink, selection highlight, multi-line wrap, or scroll offset rendering for `LineEdit`. Plain-text fill only, single line.
- Hover / pressed / focused visual states on `LineEdit` (same input-plumbing prerequisite as the deferred `Button` follow-up; tracked separately).
- Disabled-alpha treatment of `LineEdit` (the `TextEdit` arm shipped without it; this spec keeps parity. `LineEdit` disabled-cue lands when `TextEdit` gets one, in a separate spec).
- Recursion into `Container::children` from inside `DefaultStyle`. Style implementors do not own the widget tree; renderer-side dispatch (#312) iterates children.
- A new `ColorRole` for `Container` chrome (e.g. `ColorRole::Frame`). Container reuses existing `Window` / `WindowText` slots.
- Auto-installing `DefaultStyle` into `StyleRegistry`. Registration stays opt-in; unchanged from the parent spec.
- Renderer-side dispatch changes — `Style::draw_widget` invocation is `quartzite-renderer`'s problem (#312).
- Renaming or restructuring the existing four arms (`Button` / `Label` / `TextEdit` / `ScrollArea`). This spec is append-only at the router level.
- Placeholder rendering for `TextEdit` — out of scope here; `TextEdit` did not ship with placeholder support in v1 and adding it is a separate concern (different widget, different arm).

## Deferred

- LineEdit caret + selection rendering | needs a selection model + caret blink timer; same prerequisite as `TextEdit` caret work | new issue when text editing lands
- LineEdit hover / pressed / focused visual states | needs hover/focus tracking plumbed through `WidgetBase` | covered when the Button state plumbing lands
- LineEdit disabled-alpha treatment | parity with TextEdit; both land together | new issue when TextEdit gets disabled-alpha
- Container content clipping (clip-rect to `geometry()` so children that overflow are cut) | requires renderer-side dispatch decisions outside `Style::draw_widget`'s contract | tracked under #312

## Key decisions

| Question | Decision |
|---|---|
| Where the new arms live | Inside the existing `impl Style for DefaultStyle::draw_widget` chain in `quartzite-style/src/default_style.rs`. Append two arms after `ScrollArea`; append two new private `draw_container` / `draw_line_edit` methods to the existing `impl DefaultStyle` block. No new module. |
| Arm order | `Button → Label → TextEdit → ScrollArea → Container → LineEdit`. Append-only; documented inline. |
| Routing mechanism | Same `widget.as_any().downcast_ref::<T>()` chain as v1. No new trait surface. |
| Unknown widget fall-through | Unchanged (silent no-op). |
| **Container visual treatment** | **Fill + outline.** `fill_rect(geometry(), brush(Window))` followed by `draw_rect(geometry(), Pen::new(palette.color(WindowText), 1.0), Brush::solid(Color::TRANSPARENT))`. No text. _(Round-1 Q1.)_ |
| Container colour-role wiring | `Window` (background) + `WindowText` (1 px outline pen colour). Container is a generic-widget grouping surface, not a text-input data surface; `Base` is reserved for the latter (`TextEdit`, `LineEdit`, `ScrollArea`). |
| Container child traversal | Out of scope for `DefaultStyle::draw_widget`. Renderer walks children. (Same contract as the v1 `ScrollArea` arm.) |
| LineEdit colour-role wiring | `Base` background + `Text` 1 px outline + `Text` text brush (full-alpha for `text`, half-alpha via `disabled()` for `placeholder`). Matches the `TextEdit` arm exactly; preserves "text-input surface = `Base`" convention. |
| LineEdit `read_only` rendering | Same overlay treatment as `TextEdit`: insert `fill_rect(geometry(), Brush::solid(disabled(palette.color(Window))))` between the background fill and the outline draw when `line_edit.read_only == true`. |
| **LineEdit `placeholder` rendering** | **Half-alpha when text is empty.** Branch on `line_edit.text.is_empty() && !line_edit.placeholder.is_empty()`: if so, the single `DrawTextIn` event uses `&line_edit.placeholder` as the text and `Brush::solid(disabled(palette.color(ColorRole::Text)))` as the brush. Otherwise, the `DrawTextIn` event uses `&line_edit.text` and `brush(Text)`. Always exactly one `DrawTextIn` event per `draw_widget` call (no separate text+placeholder pair). _(Round-1 Q2.)_ |
| LineEdit placeholder + `read_only` interaction | The two states are independent. `read_only` controls the overlay; placeholder controls the text path. A read-only LineEdit with empty `text` and non-empty `placeholder` records both the overlay `FillRect` and the placeholder `DrawTextIn`. |
| LineEdit `is_enabled()` rendering | No disabled-alpha treatment in v1 — matches `TextEdit` parity. The placeholder's `disabled()` use is unrelated to `is_enabled()`; it's specifically the "empty-text affordance" cue. (Deferred row above.) |
| Font source | `widget.widget_base().font.clone()` — same as the four v1 arms. |
| Public surface added | None. Both new methods are private inherent methods; no new `pub use`, no new struct. |
| Helpers added | Reuse `brush` / `disabled`. Design may add a tiny local `placeholder_brush` free fn (`Brush::solid(disabled(palette.color(ColorRole::Text)))`) if it improves readability; spec does not mandate either shape. |
| `Container` v1 has only `WindowText` for the outline, no border-radius / shadow / gradient | AGENTS.md *Rust idioms*: no GUI-framework justification; stay flat. |

## Technical constraints

- `quartzite-style` already depends on `quartzite-widgets` (which exports `Container` and `LineEdit`), `quartzite-paint-api`, `quartzite-paint`, and `quartzite-style-types`. No new dependency edges needed; every type used (`Color`, `Pen`, `Brush`, `Painter`, `Font`, `AsWidget`, `Container`, `LineEdit`, `Palette`, `ColorRole`, `Alignment`, `Rect`) is already in the graph.
- `DefaultStyle: Send + Sync` is preserved (still zero-sized; no new state).
- Routing uses `widget.as_any().downcast_ref::<T>()`; `Container` and `LineEdit` both `impl AsObject` via the `Extend` macro, so `as_any()` is already present (confirmed by `cargo expand -p quartzite-widgets --lib widgets::container` / `widgets::line_edit`).
- Public-item doc gate: every new public item carries `///` first-line docs. *This spec adds no public items* — the new methods are private. The doc comment on `DefaultStyle` itself is updated to enumerate the new supported widgets.
- File size: `default_style.rs` is currently 970 lines (above the 800 soft cap, below the 1000 hard cap). Adding two arms + tests will likely push it past 1000. Design must either (a) extract the `#[cfg(test)]` block to a sibling `default_style/tests.rs` module file via `#[path]` / `mod tests;` — *not* a separate integration test crate, since `crate::registry::clear_for_test` is `pub(crate)`; or (b) split each `draw_*` method into a sibling private module file (`default_style/button.rs`, `default_style/label.rs`, …). Design picks the shape; both options preserve the `pub(crate)` reach into `registry::clear_for_test` and neither changes the public surface.
- Lint gate: `cargo clippy --workspace -- -D warnings` clean.
- Doc gate: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean.
- `Container::children()` is `&[ObjectId]` — `DefaultStyle` does not read it (no recursion).
- `LineEdit` public fields used: `text: String`, `read_only: bool`, `placeholder: String`. All confirmed `pub` on the current `quartzite-widgets::LineEdit` source.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `DefaultStyle::default().draw_widget(&Container::new(), &mut recording_painter, &Palette::default())` records exactly, in order: one `FillRect` over `geometry()` with `Brush::solid(palette.color(ColorRole::Window))`, then one `DrawRect` over `geometry()` with `Pen::new(palette.color(ColorRole::WindowText), 1.0)` and a transparent fill brush. No `DrawText` / `DrawTextIn` events. Recording painter fixture is the existing one from v1. |
| AC2 | `Container` routing does not traverse children: `Container::add_child(some_id)` followed by `DefaultStyle::default().draw_widget(&container, …)` records the same events as AC1 (no extra events). |
| AC3 | `DefaultStyle::default().draw_widget(&LineEdit::new(), &mut recording_painter, &palette)` where `palette` is `Palette::default().with_role(ColorRole::Base, …).with_role(ColorRole::Text, …)` (explicit roles to avoid the `Palette::default()` `WHITE`-collapse trap from the v1 spec's AC7 risk) records, in order: `FillRect` (background, `Base`) → `DrawRect` (outline, `Text` pen) → `DrawTextIn` (text). The `DrawTextIn` event has `text == ""`, `alignment == Alignment::Left`, brush = `Brush::solid(palette.color(ColorRole::Text))`. |
| AC4 | `LineEdit` with `let mut e = LineEdit::new(); e.text = "abc".into();` records a `DrawTextIn` whose `text == "abc"`, `alignment == Alignment::Left`, and brush = `Brush::solid(palette.color(ColorRole::Text))`. |
| AC5 | `LineEdit` with `read_only == true` (and empty text + empty placeholder) records the read-only overlay: ordered events are `FillRect` (background, `Base`) → `FillRect` (overlay, `disabled(palette.color(Window))`) → `DrawRect` (outline, `Text` pen) → `DrawTextIn` (`text == ""`, `Text` brush). The overlay `FillRect`'s brush colour equals `disabled(palette.color(ColorRole::Window))`. |
| AC6 | `LineEdit` with `text.is_empty() && placeholder == "hint"` records exactly one `DrawTextIn` event whose `text == "hint"`, `alignment == Alignment::Left`, and brush = `Brush::solid(disabled(palette.color(ColorRole::Text)))`. The full event sequence is `FillRect` (background, `Base`) → `DrawRect` (outline) → `DrawTextIn` (`"hint"`, half-alpha `Text`). No second `DrawTextIn` for the empty `text`. |
| AC7 | `LineEdit` with non-empty `text` ignores `placeholder`: `let mut e = LineEdit::new(); e.text = "abc".into(); e.placeholder = "hint".into();` records a single `DrawTextIn` with `text == "abc"`, full-alpha `Text` brush (placeholder branch not taken). |
| AC8 | `LineEdit` with `read_only == true && text.is_empty() && placeholder == "hint"` records both the overlay and the placeholder, in order: `FillRect` (background, `Base`) → `FillRect` (overlay, `disabled(Window)`) → `DrawRect` (outline) → `DrawTextIn` (`"hint"`, half-alpha `Text`). |
| AC9 | The unknown-widget arm is unaffected: `DefaultStyle::default().draw_widget(&WidgetBase::new(), …)` still records no events (regression check on the v1 AC6 behaviour). |
| AC10 | Routing order is stable — adding the `Container` and `LineEdit` arms does not change the events recorded for `Button`, `Label`, `TextEdit`, or `ScrollArea` (every v1 AC test in `default_style.rs` continues to pass unchanged). Enforced by leaving v1 tests untouched and re-running them. |
| AC11 | `cargo clippy --workspace -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` both pass on the changed crate. |
| AC12 | `default_style.rs` (or its split-out submodule, if design chooses to extract) compiles cleanly and stays within the AGENTS.md file-size envelope — specifically, no `.rs` file in `quartzite-style/src/` exceeds 1000 lines after this change, or design records an explicit exemption with rationale. |

## Open questions

- **Splitting `default_style.rs`** — the file is already 970 lines pre-edit; design picks between extracting the `#[cfg(test)]` block, splitting per-widget methods into submodules, or requesting an explicit file-size-cap exemption. Not blocking the spec.
- **`placeholder_brush` local helper vs inline `Brush::solid(disabled(palette.color(Text)))`** — readability call for the design pass; both produce identical events.

## Resolution log

- **Round 1 Q1 (Container visual treatment):** *Fill + outline.* `fill_rect(geometry, brush(Window))` then `draw_rect` with a `WindowText` 1 px pen. Captured in Scope, Key decisions, AC1, AC2.
- **Round 1 Q2 (LineEdit placeholder when text empty):** *Half-alpha.* Draw `line_edit.placeholder` using `Brush::solid(disabled(palette.color(Text)))` at `Alignment::Left` when `text.is_empty() && !placeholder.is_empty()`; otherwise draw `line_edit.text` with the full-alpha `Text` brush. Exactly one `DrawTextIn` per call. Captured in Scope, Key decisions, AC3, AC4, AC6, AC7, AC8.
