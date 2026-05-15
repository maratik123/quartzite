# Design: Default Style content for Container and LineEdit

**Issue:** #318
**Spec:** [`2026-05-15-container-lineedit-rendering.spec.md`](./2026-05-15-container-lineedit-rendering.spec.md)
**Date:** 2026-05-15

## Approach

Extend `DefaultStyle` in `quartzite-style/src/default_style.rs` with two append-only downcast arms (`Container`, then `LineEdit`) routing to two new private inherent methods, `draw_container` and `draw_line_edit`. Both arms reuse the v1 private helpers `brush(palette, role)` and `disabled(color)`; no new helper is required. The arm order is fixed by the spec: `Button → Label → TextEdit → ScrollArea → Container → LineEdit`. The unknown-widget fall-through stays a silent no-op.

### File-split shape — extract the `#[cfg(test)] mod tests` block to a sibling file

`default_style.rs` is currently 970 lines (182 prod / 788 test). The new spec adds:

- ~30 prod lines (two `draw_*` methods + two router arms).
- ~240 test lines (8 new ACs: AC1–AC10 minus the existing-fixture-shared regression checks; the file already carries the recording-painter fixture and four `brush_color_*` helper tests).

Projected total: ~1240 lines — over AGENTS.md's hard 1000-line per-file cap (AC12).

**Chosen split:** extract the `#[cfg(test)] mod tests` block to a sibling file `quartzite-style/src/default_style_tests.rs`, attached via `#[cfg(test)] #[path = "default_style_tests.rs"] mod tests;` at the bottom of `default_style.rs`. After extraction:

- `default_style.rs` → prod-only file, ~212 lines (182 today + ~30 new prod lines). Well inside the 200–400 soft target and far under the 1000 hard cap.
- `default_style_tests.rs` → the entire current test block (lines 183–970) verbatim plus the ~240 new-AC lines, ≈ 1030 lines. AGENTS.md *Code Style* → File size explicitly excludes `#[cfg(test)]` lines from the per-file cap, so this file's size does not trigger the cap. The single counter is gross line count vs. AC12's "no `.rs` file exceeds 1000 lines"; since AC12 refers to AGENTS.md envelope (which excludes test blocks), `default_style_tests.rs` is exempt. Confirm in code with a one-line comment at the top of `default_style_tests.rs`: `//! Sibling test module for default_style.rs; entire body is #[cfg(test)] (test lines are excluded from the per-file size cap).`

### Why `#[path]` and not per-widget submodules

Two options were considered for the prod-side split:

| Shape | Pros | Cons |
|---|---|---|
| `#[path = "default_style_tests.rs"] mod tests;` (chosen) | Single mechanical move; preserves `crate::registry::clear_for_test` reach (sibling file in same crate, same module tree); zero churn on prod code; test module's `super::` references stay valid (the file is still `mod tests` inside `default_style`); reviewable as a "moved 788 lines" diff. | One additional file under `quartzite-style/src/`. |
| Per-widget submodule files (`default_style/button.rs`, `default_style/label.rs`, …) | Cleaner per-widget locality if the crate ever grows toward a dozen+ widgets. | Touches every existing arm (`draw_button`, `draw_label`, `draw_text_edit`, `draw_scroll_area`) for a re-layout the spec does not require; risks accidental rename of the public surface during the move; the v1 design (`2026-05-13-default-style-content.design.md`) explicitly chose "single file" — flipping that decision now widens the PR scope past what spec § Scope demands. |
| Request an explicit file-size-cap exemption on `default_style.rs` | Zero diff overhead. | AC12 names the cap as the gate; an exemption would have to be defended in the PR description and would not match AGENTS.md *Code Style*'s spirit (the cap exists to keep files reviewable). |

Per-widget submodules are rejected for this PR — they re-layout scope already accepted in v1. The `#[path]`-attached test file is the minimum-churn move that satisfies AC12.

### `placeholder_brush` local helper — not extracted

The placeholder path is one selector + one `Brush::solid(...)` call inside a single `if let`. Extracting it would add an inline helper that reads as long as the call site it replaces:

```text
// Inline (chosen):
let (text_arg, brush) = if w.text.is_empty() && !w.placeholder.is_empty() {
    (&w.placeholder, Brush::solid(disabled(palette.color(ColorRole::Text))))
} else {
    (&w.text, brush(palette, ColorRole::Text))
};
painter.draw_text_in(geom, text_arg, &font, &brush, Alignment::Left);

// Extracted variant (rejected):
fn placeholder_brush(palette: &Palette) -> Brush {
    Brush::solid(disabled(palette.color(ColorRole::Text)))
}
// ...with the same if-let at the call site, just one symbol shorter on one branch.
```

YAGNI per AGENTS.md *Code Style* → file-size counter-rule ("counter-rule against over-splitting"). The spec explicitly says "no new helper required" and gives the design discretion; we exercise discretion to keep it inline. Open question to flag for review: if the design-review pass finds the inline arm reads worse, a one-line `fn placeholder_brush(palette: &Palette) -> Brush` would be a trivial follow-up edit.

### Per-widget content (matches spec § Scope verbatim)

| Widget | Operations recorded on `painter`, in order |
|---|---|
| `Container` | `fill_rect(geom, &brush(palette, ColorRole::Window))`; `draw_rect(geom, &Pen::new(palette.color(ColorRole::WindowText), 1.0), &Brush::solid(Color::TRANSPARENT))`. No text. No recursion into `Container::children()`. |
| `LineEdit` (idle, text non-empty) | `fill_rect(geom, &brush(palette, ColorRole::Base))`; `draw_rect(geom, &Pen::new(palette.color(ColorRole::Text), 1.0), &Brush::solid(Color::TRANSPARENT))`; `draw_text_in(geom, &w.text, &font, &brush(palette, ColorRole::Text), Alignment::Left)`. |
| `LineEdit` (text empty, placeholder non-empty) | Same fill + outline; `draw_text_in(geom, &w.placeholder, &font, &Brush::solid(disabled(palette.color(ColorRole::Text))), Alignment::Left)`. **Exactly one `DrawTextIn` event** — placeholder replaces the text in the same call, never appended as a second event. |
| `LineEdit` (`read_only == true`) | Insert `fill_rect(geom, &Brush::solid(disabled(palette.color(ColorRole::Window))))` between the background fill and the outline draw. The read-only overlay is independent of the placeholder path — they compose orthogonally (AC8 exercises both at once). |

Font is read once per call as `let font = w.widget_base().font.clone();` — the v1 `Arc<Font>::clone` pattern used by all four existing arms.

### Router body — new arms appended

```text
fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
    let any = widget.as_any();
    if let Some(w) = any.downcast_ref::<Button>()     { return self.draw_button(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<Label>()      { return self.draw_label(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<TextEdit>()   { return self.draw_text_edit(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<ScrollArea>() { return self.draw_scroll_area(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<Container>()  { return self.draw_container(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<LineEdit>()   { self.draw_line_edit(w, painter, palette); }
    // Unknown widget — deliberate no-op; does not panic. (Unchanged.)
}
```

Order is append-only. The trailing arm (`LineEdit`) drops the explicit `return` because there is no following arm to early-exit before — same shape as today's `ScrollArea` arm. (`clippy::needless_return` would otherwise fire.)

### Module-level imports

`quartzite-style/src/default_style.rs` `use` block grows by two symbols:

```text
use quartzite_widgets::{
    Alignment, AsWidget, Button, Container, Label, LineEdit, ScrollArea, TextEdit, WidgetExt,
};
```

The crate already re-exports `Container` and `LineEdit` from `quartzite_widgets` (confirmed in `quartzite-widgets/src/widgets/mod.rs`); no new dependency edge.

### Rejected alternatives

1. **Per-widget submodule split (`default_style/{button,label,text_edit,scroll_area,container,line_edit}.rs`).** Rejected above: widens scope past spec § Scope and re-litigates the v1 single-file decision.
2. **Extracting a `placeholder_brush` free fn.** Rejected: inline path is one expression shorter at the call site once you account for the `if-let` selector; YAGNI per *Code Style* file-size counter-rule.
3. **Two `DrawTextIn` events on LineEdit (`text` + `placeholder` separately).** Rejected by spec § Key decisions ("Always exactly one `DrawTextIn` event per `draw_widget` call"). AC6 explicitly enforces this: "No second `DrawTextIn` for the empty `text`."
4. **Painting `placeholder` whenever it is non-empty (regardless of `text`).** Rejected by spec § Scope: placeholder is shown only when `text.is_empty() && !placeholder.is_empty()`. AC7 enforces the opposite branch (non-empty text wins over placeholder).
5. **Adding a `ColorRole::Frame` slot for Container's outline.** Rejected by spec § Out of scope. Container reuses `Window` (bg) + `WindowText` (outline).
6. **Recording-painter fixture in `tests/support/` for cross-crate reuse.** Rejected for v1 by the parent design and re-rejected here: the existing in-module fixture covers every assertion both specs need, and the spec does not ask for cross-crate sharing.
7. **Disabled-alpha treatment of `LineEdit` content.** Rejected by spec § Out of scope (parity with v1 `TextEdit`, which did not ship disabled-alpha).
8. **GPU snapshot tests in `tests/snapshots.rs` for Container/LineEdit.** Deferred — flagged in § Open questions. The spec's ACs are event-recording assertions; snapshot coverage is a separate workstream.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Extract the existing `#[cfg(test)] mod tests` block (lines 183–970) verbatim into a new sibling file `default_style_tests.rs`; replace it in `default_style.rs` with `#[cfg(test)] #[path = "default_style_tests.rs"] mod tests;`. Add a one-line top-of-file comment in `default_style_tests.rs` explaining the `#[path]` arrangement. Verify `cargo test -p quartzite-style` still passes (all existing tests run unchanged). | `quartzite-style/src/default_style.rs`, `quartzite-style/src/default_style_tests.rs` (new) | — |
| 2 | Grow the `use` block in `default_style.rs` to include `Container`. Add the `Container` arm to `draw_widget` (after `ScrollArea`) and implement `draw_container` body fully: `fill_rect(geom, &brush(palette, ColorRole::Window))` then `draw_rect(geom, &Pen::new(palette.color(ColorRole::WindowText), 1.0), &Brush::solid(Color::TRANSPARENT))`. Add `Container` + `ObjectId` to the `use` block in `default_style_tests.rs`. Add AC1 + AC2 tests. Commit only after clippy passes (no empty body). | `quartzite-style/src/default_style.rs`, `quartzite-style/src/default_style_tests.rs` | 1 |
| 3 | Grow the `use` block in `default_style.rs` to include `LineEdit`. Add the `LineEdit` arm to `draw_widget` (after `Container`) and implement `draw_line_edit` body fully: `fill_rect` (Base) → optional `fill_rect` (read-only overlay) → `draw_rect` (outline) → single `draw_text_in` branched by `text.is_empty() && !placeholder.is_empty()`. Add `LineEdit` to the `use` block in `default_style_tests.rs`. Add AC3, AC4, AC6, AC7 tests. Commit only after clippy passes (no empty body). | `quartzite-style/src/default_style.rs`, `quartzite-style/src/default_style_tests.rs` | 1 |
| 4 | Add AC5 + AC8 tests for `LineEdit` read-only: empty-everything-with-read-only (4 events, overlay between bg and outline), and read-only-plus-placeholder (still 4 events, overlay + placeholder `DrawTextIn`). | `quartzite-style/src/default_style_tests.rs` | 3 |
| 5 | Verify AC9 regression (`unknown_widget_type_produces_no_events` still passes) and AC10 router-stability (all v1 tests unchanged). No new test file content. | (no edit — verification only) | 4 |
| 6 | Update the module-level doc comment on `DefaultStyle` (in `default_style.rs`) to enumerate the two new supported widgets (`Container` and `LineEdit`). Re-run `cargo clippy --workspace -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` to satisfy AC11. Verify `wc -l default_style.rs` < 250 and `wc -l default_style_tests.rs` records the total for the PR description. | `quartzite-style/src/default_style.rs` | 2, 3, 4 |

6 tasks. Tasks 2 and 3 are independent of each other once 1 lands — both add a fully-bodied `draw_*` method (no empty-body intermediate commit, which avoids a `clippy::unused_variables` failure at any commit boundary). Task 4 layers on 3 because it tests `LineEdit` read-only paths whose `draw_line_edit` body must already be in place. Task 5 is verification-only; task 6 is the cleanup pass.

## Risks

- **`#[path]` test-module move silently drops a test.** The extraction is a verbatim cut-and-paste of lines 183–970 plus a one-line `#[path]` attribute. The risk is a stray edit during the move (line break, indentation, import re-shuffle) that compiles but silently drops a test name. *Mitigation:* before the move, capture `cargo test -p quartzite-style --no-run --message-format=json | jq -r '.executable // empty'` produces the test-binary path, then `cargo test -p quartzite-style -- --list 2>&1 | sort > /tmp/tests_before`; after the move, repeat into `/tmp/tests_after` and `diff` the two files. Lists must be identical.
- **`super::disabled` from the test module after the move.** The existing test at line 481 calls `super::disabled(...)`. Inside a `#[path]`-attached module the file is still `mod tests` *inside* the parent module (`default_style`), so `super::` still resolves to `crate::default_style` and `super::disabled` keeps compiling. *Mitigation:* no rename; verify with `cargo test -p quartzite-style text_edit_read_only_inserts_overlay_fill` after the move.
- **`crate::registry::clear_for_test` reach from the sibling test file.** `clear_for_test` is `pub` (not `pub(crate)`) gated on `cfg(any(test, feature = "test-support"))`. The sibling test file is still inside the `quartzite-style` crate, so any visibility path works. *Mitigation:* none required; AC10 (`registry_round_trip_dispatches_default_style`) is the canary — if the extraction breaks the call, that test fails in task 1.
- **Routing chain order change.** The spec mandates `Button → Label → TextEdit → ScrollArea → Container → LineEdit`. Each `downcast_ref::<T>()` short-circuits on first match and the concrete types are disjoint, so order is observable only by reviewers, not by behaviour. *Mitigation:* document the order in a single one-line comment above the chain (already done in v1; extend the comment to mention the two new arms).
- **`Palette::default()` `WHITE`-collapse trap for AC3.** Spec AC3 explicitly constructs a palette with `Palette::default().with_role(ColorRole::Base, …).with_role(ColorRole::Text, …)` to avoid the trap. The recording-painter assertions for `LineEdit` background must compare `brush_color(...)` against `palette.color(ColorRole::Base)` from that same explicit palette, not a literal `Color::WHITE`. *Mitigation:* test design (below) follows the v1 AC7 pattern: build the explicit palette as a local in each test, never assume `Palette::default()` separates `Base` from `Window` from `Highlight`.
- **`text_changed` signal emission during construction.** `LineEdit::new()` initialises `text: String::new()` and never fires `text_changed`. Tests that mutate `e.text = "abc".into()` directly (vs. `e.set_text("abc".into())`) bypass the slot, but `DefaultStyle::draw_widget` reads `&w.text` directly — the signal path is irrelevant. *Mitigation:* none; documented here for reviewers.
- **`Container::children()` slice access in `draw_container`.** `draw_container` does **not** read `w.children()`. The spec is explicit (AC2: "Container routing does not traverse children"). *Mitigation:* AC2 is the regression check; `draw_container` body never touches `children()`.
- **Doc gate (`RUSTDOCFLAGS=-D missing-docs`).** No new public items — both `draw_container` and `draw_line_edit` are private inherent methods. The `DefaultStyle` doc comment updates its enumerated-widgets list to include `Container` and `LineEdit`; that update is part of task 7. *Mitigation:* CI gate (AC11) catches a missing update.
- **No API backward-compat concern.** Crate has not been published; the two added arms are additive at the routing layer and add no public surface.
- **No panic / unsafe surface added.** Every new code path returns `()`; no `unwrap()`, no `unsafe`. Downcast misses fall through cleanly per v1's design.
- **Clippy: `needless_return` on the trailing arm.** The trailing `LineEdit` arm drops the explicit `return` (same as the current trailing `ScrollArea` arm does today). If a future widget arm is appended after `LineEdit`, the new arm must add `return` and the `LineEdit` arm gains an explicit `return self.draw_line_edit(...)`. *Mitigation:* documented inline by the existing comment.
- **`Brush` `Eq` semantics for `read_only` overlay comparison.** AC5 / AC8 compare the overlay's brush colour against `disabled(palette.color(ColorRole::Window))`. The test does this via `brush_color(...)` (already in the fixture); no `Brush::eq` is invoked. *Mitigation:* fixture stays unchanged.
- **File-size cap interpretation on `default_style_tests.rs`.** AGENTS.md *Code Style* → File size says "target 200–400 lines per `.rs` file **excluding `#[cfg(test)]`**". The new tests file's entire body is `#[cfg(test)]`-scoped (gated by the parent `default_style.rs` attribute). Net excluded-from-cap lines: zero — the file does not count against any cap. *Mitigation:* call this out in the top-of-file comment so a future contributor doesn't misread the size.

## Test Design

All new tests live in `quartzite-style/src/default_style_tests.rs` (the sibling file created in task 1). They reuse the existing `RecordingPainter`, `PaintEvent`, and `brush_color` fixture verbatim. New tests are appended to the bottom of the existing test module in a section banner:

```text
// ── Container + LineEdit (spec 2026-05-15) ──────────────────────────────
```

### Test fixtures and helpers — reused verbatim

- `RecordingPainter` — `Painter` impl with `Vec<PaintEvent>` capture, identical to v1.
- `first_fill`, `first_draw_text_in`, `first_draw_rect` — already cover every assertion the spec needs; no new helper.
- `brush_color(b: &Brush) -> Color` — already covers `Brush::Solid` (the only `Brush` kind `DefaultStyle` ever constructs); no extension required.

No new fixture or helper is added in this PR.

### Per-AC test layout

For each test below:
- **Location:** `quartzite-style/src/default_style_tests.rs` (sibling `mod tests` of `default_style.rs`).
- **Entry point:** `DefaultStyle::default().draw_widget(&widget, &mut painter, &palette)`.
- **Recording painter:** `RecordingPainter::default()`.

#### AC1 — `container_records_fill_and_outline`

- **Setup:** `let c = Container::new();` and an explicit palette to avoid the `Palette::default()` `WHITE`-collapse trap (both `Window` and `Base` default to `WHITE`, so a mis-wiring that uses `Base` instead of `Window` would pass if we don't discriminate):
  ```text
  let palette = Palette::default()
      .with_role(ColorRole::Window, Color::new(0.9, 0.9, 0.9, 1.0))
      .with_role(ColorRole::WindowText, Color::new(0.1, 0.1, 0.1, 1.0));
  ```
- **Assertions:** exactly 2 events. `events[0]` is `PaintEvent::FillRect { rect, brush }` with `rect == c.widget_base().geometry` and `brush_color(brush) == palette.color(ColorRole::Window)` (≠ `palette.color(ColorRole::Base)` — discriminated). `events[1]` is `PaintEvent::DrawRect { rect, pen, brush }` with `rect == c.widget_base().geometry`, `pen.color() == palette.color(ColorRole::WindowText)` (≠ `palette.color(ColorRole::Text)` — discriminated), `pen.width() == 1.0`, and `brush_color(brush) == Color::TRANSPARENT`. No `DrawText` / `DrawTextIn` events present.
- **Why:** locks in spec AC1 (fill+outline + colour-role wiring + no text). The explicit palette ensures a mis-wiring that used `Base` for the background or `Text` for the outline colour would fail the assertion.

#### AC2 — `container_routing_ignores_children`

- **Setup:** `let mut c = Container::new(); c.add_child(ObjectId::new());` (and a second `add_child` to be defensive about loop edge cases). Use the same explicit palette from AC1.
- **Assertions:** identical to AC1 — exactly 2 events; `add_child` does not change the recorded sequence.
- **Why:** locks in spec AC2 (renderer owns child traversal, not `Style::draw_widget`).
- **Note:** brings `quartzite_core::ObjectId` into the test module's `use` block (already present elsewhere; if not, add it).

#### AC3 — `line_edit_records_fill_outline_and_empty_text`

- **Setup:** `let e = LineEdit::new();` and an explicit palette pinning `Base` and `Text` to non-white values to avoid the `Palette::default()` `WHITE`-collapse trap:
  ```text
  let palette = Palette::default()
      .with_role(ColorRole::Base, Color::new(0.95, 0.95, 0.95, 1.0))
      .with_role(ColorRole::Text, Color::BLACK);
  ```
- **Assertions:** exactly 3 events. `events[0]` is `FillRect` with `brush_color == palette.color(Base)`. `events[1]` is `DrawRect` with `pen.color() == palette.color(Text)` and `pen.width() == 1.0`. `events[2]` is `DrawTextIn` with `text == ""`, `alignment == Alignment::Left`, `brush_color(&brush) == palette.color(Text)` (full-alpha, not the placeholder half-alpha — confirms the both-empty path takes the non-placeholder branch).
- **Why:** locks in spec AC3 (event count + ordering + brush wiring + the empty-text fallthrough to the `&w.text` branch).

#### AC4 — `line_edit_records_text_when_non_empty`

- **Setup:** `let mut e = LineEdit::new(); e.text = "abc".into();` with the same explicit palette as AC3.
- **Assertions:** `first_draw_text_in(&events)` matches `DrawTextIn { text, alignment, brush, .. }` with `text == "abc"`, `alignment == Alignment::Left`, `brush_color(&brush) == palette.color(Text)`.
- **Why:** locks in spec AC4 (non-empty text path; full-alpha text brush).

#### AC5 — `line_edit_read_only_inserts_overlay`

- **Setup:** `let mut e = LineEdit::new(); e.read_only = true;` with an explicit palette pinning `Base`, `Text`, and `Window` to distinguishable values:
  ```text
  let palette = Palette::default()
      .with_role(ColorRole::Base, Color::new(0.95, 0.95, 0.95, 1.0))
      .with_role(ColorRole::Text, Color::BLACK)
      .with_role(ColorRole::Window, Color::new(0.9, 0.9, 0.9, 1.0));
  ```
- **Assertions:** exactly 4 events. `events[0]` is `FillRect` (Base background). `events[1]` is `FillRect` with `brush_color == super::disabled(palette.color(ColorRole::Window))` — the read-only overlay. `events[2]` is `DrawRect` (outline). `events[3]` is `DrawTextIn { text: "", ..}`. The empty-text + empty-placeholder path still draws the empty `&w.text` (not the placeholder).
- **Why:** locks in spec AC5 (overlay between bg and outline; correct overlay colour).

#### AC6 — `line_edit_placeholder_drawn_when_text_empty`

- **Setup:** `let mut e = LineEdit::new(); e.placeholder = "hint".into();` with the AC3 explicit palette.
- **Assertions:** exactly 3 events. `events[2]` (the single `DrawTextIn`) matches `text == "hint"`, `alignment == Alignment::Left`, `brush_color(&brush) == super::disabled(palette.color(ColorRole::Text))` (half-alpha). Critically, count `events.iter().filter(|e| matches!(e, PaintEvent::DrawTextIn { .. })).count() == 1` — guards against the rejected "two `DrawTextIn` events" alternative.
- **Why:** locks in spec AC6 (placeholder path + exactly-one-`DrawTextIn` invariant + half-alpha brush).

#### AC7 — `line_edit_non_empty_text_ignores_placeholder`

- **Setup:** `let mut e = LineEdit::new(); e.text = "abc".into(); e.placeholder = "hint".into();` with the AC3 explicit palette.
- **Assertions:** single `DrawTextIn` event with `text == "abc"` (not `"hint"`), `brush_color(&brush) == palette.color(ColorRole::Text)` (full-alpha, not half-alpha). Confirms the branch selector `text.is_empty() && !placeholder.is_empty()` evaluates `false` when `text` is non-empty.
- **Why:** locks in spec AC7 (placeholder branch not taken when text present).

#### AC8 — `line_edit_read_only_with_placeholder_overlays_and_renders_placeholder`

- **Setup:** `let mut e = LineEdit::new(); e.read_only = true; e.placeholder = "hint".into();` with the AC5 explicit palette.
- **Assertions:** exactly 4 events. `events[0]` `FillRect` (Base). `events[1]` `FillRect` (overlay, `disabled(Window)`). `events[2]` `DrawRect` (outline). `events[3]` `DrawTextIn { text: "hint", brush_color = disabled(Text), alignment = Left }`. Composes the two orthogonal axes (read-only overlay + placeholder text path) in one observation.
- **Why:** locks in spec AC8 (orthogonality of `read_only` and placeholder).

#### AC9 — existing `unknown_widget_type_produces_no_events` regression

- **Action:** no new test. The existing test at v1 file line 521 continues to run; verify it passes after task 6.
- **Why:** locks in spec AC9 (unknown-widget no-op unchanged by the two new arms).

#### AC10 — router-stability regression

- **Action:** no new test. The existing tests `button_records_fill_outline_and_centred_text` (AC2 v1), `label_records_fill_and_text_with_label_alignment` (AC3 v1), `text_edit_records_fill_outline_and_text` (AC4 v1), `scroll_area_records_fill_and_outline_only` (AC5 v1), plus the hover/pressed/focused suite, all continue to run unchanged. The mechanical guarantee is the append-only arm order — every v1 widget reaches its arm before the two new ones.
- **Why:** locks in spec AC10 (route stability).

#### AC11 — lint / doc gates

- **Action:** task 7 runs `cargo clippy --workspace -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` locally. CI re-runs both. No test artefact.

#### AC12 — file-size envelope

- **Action:** task 1 (test-block extraction) drops `default_style.rs` from 970 lines to ~182 lines; tasks 3 + 4 + 7 push it back to ~212 lines. `default_style_tests.rs` lands at ~1030 lines but its entire body is `#[cfg(test)]`-gated, so the AGENTS.md *Code Style* per-file cap (which excludes `#[cfg(test)]`) does not apply. AC12's intent — keep reviewable per-file size under 1000 — is satisfied for `default_style.rs` (prod file) and explicitly carve-out-satisfied for `default_style_tests.rs` (test-only file) via the top-of-file comment.

### Fixtures and helpers — summary

- `RecordingPainter`, `PaintEvent`, `first_fill`, `first_draw_text_in`, `first_draw_rect`, `brush_color` — all reused verbatim from the existing test module.
- New imports added to the `use` block in `default_style_tests.rs`: `Container`, `LineEdit` (from `quartzite_widgets`), and `ObjectId` (from `quartzite_core`, for AC2's `add_child` call).

### Sample sizes & numbers

| File | Lines (prod) | Lines (test) | Lines (total) |
|---|---|---|---|
| `quartzite-style/src/default_style.rs` (after PR) | ~212 | 1 (the `#[path]` attribute line) | ~213 |
| `quartzite-style/src/default_style_tests.rs` (new file) | 0 | ~1030 | ~1030 |
| `quartzite-style/src/lib.rs` | unchanged | unchanged | unchanged |

The prod-only `default_style.rs` lands solidly inside the 200–400 soft target.

## Open questions

None of these block implementation; they are flagged for visibility:

- **GPU snapshot tests for Container / LineEdit in `quartzite-style/tests/snapshots.rs`.** The v1 spec shipped 10 snapshot tests for `Button` / `Label` / `TextEdit` / `ScrollArea` (`button_idle_renders`, `label_renders`, `text_edit_plain_renders`, `text_edit_read_only_renders`, etc.). This spec's ACs are event-recording only; whether to add `container_renders`, `line_edit_idle_renders`, `line_edit_read_only_renders`, and `line_edit_placeholder_renders` snapshots is **not mandated by spec § Acceptance Criteria**. Recommendation: open a follow-up issue once this PR lands — same model used for the v1 follow-ups that landed snapshot tests for hover / pressed / focused.
- **`placeholder_brush` extraction.** Inline `Brush::solid(disabled(palette.color(ColorRole::Text)))` is one expression. If reviewers find the call-site reads better with a one-line helper, the cost of extraction is trivial and the spec leaves the choice to design. Current design says inline; design-review may flip this.
- **`Container::children()` clip-rect.** Spec § Out of scope defers clip-rect-to-`geometry()` for overflow-cutting. Container draws fill+outline only in v1. The deferred row in the spec routes this work through #312 (renderer-side dispatch).
- **`LineEdit` caret blink / selection rendering.** Spec § Deferred routes this through the same selection-model + caret-timer prerequisite that gates `TextEdit` caret work. Out of scope.
- **`LineEdit` hover / pressed / focused states.** Deferred — same input-plumbing prerequisite as the now-shipped `Button` hover/pressed/focused work, but for `LineEdit` the spec defers until that plumbing is generalised across widgets.
- **`LineEdit` disabled-alpha treatment.** Spec § Deferred says it lands together with `TextEdit` disabled-alpha in a future spec. Parity argument; not in scope here.
