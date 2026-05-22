# Design: LineEdit hover / pressed / focused visual states (folds in #407 disabled-axis parity)

**Issue:** #406 (folds in #407)
**Date:** 2026-05-22

## Approach

This is a single-widget state-axis follow-up to the just-merged #403 (PR #524). The spec is pinned cell-by-cell in § Key decisions, every helper is already in place on `master`, and `impl Paint<TextEdit> for DefaultStyle` (`quartzite-style/src/default_style.rs:161-219`) is the in-tree pattern reference for the exact code shape this LineEdit impl needs. The design's job is therefore to translate the spec mappings into one in-place rewrite of `impl Paint<LineEdit> for DefaultStyle` + a test-organisation plan + handoff-grouping. No new architecture.

**Chosen approach — direct in-place rewrite of `impl Paint<LineEdit> for DefaultStyle` (`quartzite-style/src/default_style.rs:273-303`).** The new body adopts the post-#403 `impl Paint<TextEdit>` skeleton verbatim (`enabled`/`hovered`/`pressed`/`focused` reads → `state_group` selector → per-widget role mapping → `maybe_disabled` wrap → focus-outline branch → painter calls), then layers LineEdit's placeholder-when-empty branch over the resolved text colour. No new public API, no `Style`/`Paint` trait changes, no new helpers — `state_group` (line 310), `maybe_disabled` (line 352), `read_only_overlay` (line 341), `disabled` (line 330), and the `FOCUS_RING_WIDTH` const (line 23) are all reused as-is.

**Visual mappings — verbatim from spec § Key decisions.** Recap of the chosen role/group pairs (for each non-focus-ring colour, the resolved value passes through `maybe_disabled(_, enabled)`; the focus-ring pen is exempt per the design-system additive-overlay rule — `design-system/README.md` § *Animation, hover, press*):

- **Idle / Hover / Pressed:** `(fill_role, text_role) = if pressed { (Highlight, HighlightedText) } else { (Base, Text) }`. Outline tracks the text colour when text is present, with a press-time swap to `HighlightedText` for legibility under the inverted Highlight fill: `outline_role_idle = if pressed { HighlightedText } else { Text }`. This is identical to the post-#403 `impl Paint<TextEdit>` (lines 171-175 + 178-182), substituting LineEdit's `Text`-role outline for TextEdit's `Text`-role outline (they happen to coincide — LineEdit pre-#406 already uses `Text` for its 1 px outline at line 284).
- **Focused:** widens the outline to `FOCUS_RING_WIDTH = 2.0` using `palette.color(FocusRing, Normal)`, full alpha. Otherwise outline width is `1.0` and colour is `outline_color_idle` (the `maybe_disabled`-wrapped state lookup).
- **Disabled (#407 fold-in):** every fill / text / non-focus-ring pen colour passes through `maybe_disabled(_, enabled)` before painting. This is the visible #407 change — the pre-spec LineEdit impl wraps zero colours in `maybe_disabled`, so disabled-LineEdit on `master` today renders identically to enabled-LineEdit. Post-spec, disabled-LineEdit matches the post-#403 `Paint<TextEdit>` disabled treatment (halved alpha on Base fill + Text outline + Text glyph brush).
- **Read-only overlay:** preserves the existing 4-step paint order `fill_rect(Base) → optional fill_rect(overlay) → draw_rect(outline) → draw_text_in(text)` with state lookups substituted in. Existing read-only-text dim composes over the state-resolved text colour: `text_color.with_alpha(READ_ONLY_TEXT_ALPHA)` (mirrors `impl Paint<TextEdit>` lines 205-210).
- **Placeholder branch:** preserves the existing `if/else if/else` shape — placeholder wins over read-only-text dim when `text.is_empty() && !placeholder.is_empty()`. The new wrinkle is that the placeholder is drawn at `disabled(text_color)` where `text_color` is already `maybe_disabled(palette.color(text_role, group), enabled)`. Composition: enabled-placeholder = `× 0.5` (existing fixed dim); disabled-placeholder = `× 0.5 × 0.5 = × 0.25` alpha. This is the spec's intentional composition (§ Key decisions row "Disabled-state interaction") — the placeholder dim is applied to whatever colour the state branch resolves, exactly as TextEdit's read-only dim is applied to whatever colour the state branch resolves.

**Paint order — preserved across all branches.**

```
fill_rect(state_fill)
  optional: fill_rect(read_only_overlay)
draw_rect(outline_color, outline_width)
draw_text_in(text_or_placeholder, text_brush)
```

This matches the current LineEdit impl's event order (lines 278-301): bg → optional overlay → outline → text. The existing `line_edit_read_only_inserts_overlay` test (lines 1118-1150) asserts `events[1]` is the overlay; that ordering is preserved, so the existing 5 `line_edit_*` tests pass byte-for-byte under idle-enabled conditions (AC6).

**Text-brush selection ladder — preserves the existing 3-arm `if/else if/else`.** The pre-spec body uses (line 287-300):

```rust
let text_role_color = palette.color(ColorRole::Text, ColorGroup::Normal);
let (text_arg, text_brush) = if w.text.is_empty() && !w.placeholder.is_empty() {
    (w.placeholder.as_str(), Brush::solid(disabled(text_role_color)))
} else if w.read_only {
    (w.text.as_str(), Brush::solid(text_role_color.with_alpha(READ_ONLY_TEXT_ALPHA)))
} else {
    (w.text.as_str(), Brush::solid(text_role_color))
};
```

The new body substitutes the state-resolved + disabled-wrapped `text_color` for `text_role_color`:

```rust
let text_color = maybe_disabled(palette.color(text_role, group), enabled);
let (text_arg, text_brush) = if w.text.is_empty() && !w.placeholder.is_empty() {
    (w.placeholder.as_str(), Brush::solid(disabled(text_color)))
} else if w.read_only {
    (w.text.as_str(), Brush::solid(text_color.with_alpha(READ_ONLY_TEXT_ALPHA)))
} else {
    (w.text.as_str(), Brush::solid(text_color))
};
```

Both modifiers (`disabled` for placeholder, `with_alpha(READ_ONLY_TEXT_ALPHA)` for read-only) now compose orthogonally on top of the state-resolved text colour. The placeholder branch wins over read-only when both apply (existing precedence preserved — `line_edit_read_only_with_placeholder_overlays_and_renders_placeholder` at line 1152 already pins that behaviour).

**Rejected alternatives:**

- **Add a `Paint::state_aware_paint(...)` default method on the `Paint` trait.** Spec § Out of scope rules out trait redesign without a Design Amendment. The #403 design already considered this and rejected it; the same logic applies here.
- **Extract a sibling `line_edit.rs` paint-impl module.** Spec § Technical constraints budget shows `default_style.rs` at 358 lines, comfortably below the 500-line soft target. The LineEdit impl will grow modestly (maybe +20-30 lines) and stay under 400. No file-split needed; the per-widget impls staying co-located keeps the four state-aware impls (Label / TextEdit / ScrollArea / LineEdit) visually adjacent for cross-widget consistency audits.
- **Generalise a `paint_framed_state_aware(...)` helper across TextEdit / LineEdit / ScrollArea.** The #403 design already rejected this — per-widget visual decisions diverge (read-only overlay only on TextEdit and LineEdit; text rendering varies in alignment + content selection; placeholder is LineEdit-only). Generalising now would over-fit on the existing four widgets and complicate the next addition.
- **Apply `maybe_disabled` only when `!enabled`, gated by `if !enabled { ... }`.** `maybe_disabled` is already a no-op when `enabled` (line 353: `if enabled { color } else { disabled(color) }`), so the unconditional wrap is free and reads cleanly. Mirrors the shipped Button / Label / TextEdit / ScrollArea impls.
- **Re-extract a `line_edit_text_brush(text_color, enabled, read_only, placeholder_empty, text_empty)` helper.** Single call site in `default_style.rs`; no win. The 3-arm `if/else if/else` is local, linear, and self-documenting.
- **Add `dark_line_edit_hovered.png` / `dark_line_edit_pressed.png` goldens.** Spec § Deferred row 1 explicitly defers these to a follow-up — `DARK_PALETTE` derives Hover/Pressed via a small luminance shift on Base/Highlight that produces near-identical PNGs to idle/focused under the dark theme. The #403 spec capped the same way; this spec follows that precedent.
- **Snapshot-test combined-state cells (`line_edit_focused_read_only.png`, `line_edit_disabled_focused.png`, `line_edit_pressed_placeholder.png`).** Spec § Deferred row 2 caps at single-flag goldens, mirroring the Button / #403 precedent. Combinations are recording-painter-tested.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rewrite `impl Paint<LineEdit> for DefaultStyle` (currently `default_style.rs:273-303`) to read `enabled` / `hovered` / `pressed` / `focused` via `WidgetExt`, compute `group = state_group(pressed, hovered)`, compute `(fill_role, text_role) = if pressed { (Highlight, HighlightedText) } else { (Base, Text) }`, compute `outline_role_idle = if pressed { HighlightedText } else { Text }`, resolve `(fill_color, text_color, outline_color_idle)` each via `maybe_disabled(palette.color(role, group), enabled)`, paint `fill_rect(state_fill)` → if `w.read_only` paint `fill_rect(read_only_overlay(palette))` → paint `draw_rect` with `(outline_color_idle, 1.0)` if not focused, else `(palette.color(FocusRing, Normal), FOCUS_RING_WIDTH)` — **FocusRing pen colour NOT wrapped in `maybe_disabled`** (full alpha always, per design-system additive-overlay rule + shipped Button / Label / TextEdit / ScrollArea behaviour) → resolve `text_brush` via the preserved 3-arm ladder (`placeholder ? disabled(text_color) : read_only ? text_color.with_alpha(READ_ONLY_TEXT_ALPHA) : text_color`) → paint `draw_text_in(geom, text_arg, &font, &Brush::solid(text_brush), Alignment::Left)`. Event order preserved (fill / optional-overlay / outline / text). Idle-enabled baseline = identical to today (Base/Normal fill + Text/Normal outline + Text/Normal text). | `quartzite-style/src/default_style.rs` | — |
| 2 | Add LineEdit state-axis recording-painter unit tests in `default_style_tests.rs` alongside the existing five `line_edit_*` tests (around lines 1023-1244): `hovered_line_edit_uses_derived_hover_fill` (AC1 — fill = `Base × Hover`, outline pen = `Text × Hover`, text brush = `Text × Hover`); `pressed_line_edit_uses_highlight_pressed` (AC1 — fill = `Highlight × Pressed`, outline pen = `HighlightedText × Pressed`, text brush = `HighlightedText × Pressed`); `focused_line_edit_uses_2px_focus_ring_outline` (AC1 — outline pen width = `2.0`, colour = `FocusRing × Normal`, with `#[allow(clippy::float_cmp, reason = ...)]` matching the TextEdit precedent); `disabled_and_focused_line_edit_paints_outline_under_disabled` (AC3 — FocusRing pen at full alpha + 2 px under `set_enabled(false) + set_focused(true)`); `precedence_pressed_hovered_line_edit_picks_pressed_fill` (AC3 — pressed wins on fill axis); `line_edit_disabled_idle_dims_base_text_outline` (AC2 / #407 fold-in anchor — `set_enabled(false)`, no other flags; assert fill = `Brush::solid(maybe_disabled(palette.color(Base, Normal), false))`, outline pen colour = `maybe_disabled(palette.color(Text, Normal), false)` at width `1.0`, text glyph brush = `Brush::solid(maybe_disabled(palette.color(Text, Normal), false))`); `line_edit_read_only_hovered_overlay_plus_hover_base_fill` (AC5 — `read_only=true` + `hovered=true`: 4 events captured in order = `FillRect(Base × Hover)` / `FillRect(WindowText × Normal @ READ_ONLY_OVERLAY_ALPHA)` / `DrawRect(Text × Hover @ 1.0)` / `DrawTextIn(text, Brush::solid(palette.color(Text, Hover).with_alpha(READ_ONLY_TEXT_ALPHA)))`); `line_edit_hovered_placeholder_tracks_hover_text` (AC4 — `text=""`, `placeholder="hint"`, `set_hovered(true)`: placeholder DrawTextIn brush = `Brush::solid(disabled(palette.color(Text, Hover)))`); `line_edit_pressed_placeholder_tracks_pressed_text` (AC4 — `text=""`, `placeholder="hint"`, `set_pressed(true)`: placeholder DrawTextIn brush = `Brush::solid(disabled(palette.color(HighlightedText, Pressed)))`); `line_edit_disabled_placeholder_composes_double_dim` (AC4 / #407 fold-in flowing through placeholder — `text=""`, `placeholder="hint"`, `set_enabled(false)`: placeholder DrawTextIn brush = `Brush::solid(disabled(maybe_disabled(palette.color(Text, Normal), false)))` which is `disabled(disabled(palette.color(Text, Normal)))` ≈ `× 0.25` alpha). Reuse `RecordingPainter`, `first_fill`, `first_draw_rect`, `first_draw_text_in`, `brush_color`, `line_edit_palette()`, `line_edit_read_only_palette()` helpers; use `Palette::default()` where the numeric derivation suffices (matching the post-#403 TextEdit-state tests' choice for the same reason — recording-painter tests gate on numeric equality, not visual distinction). The existing five `line_edit_*` tests (`line_edit_records_fill_outline_and_empty_text`, `line_edit_records_text_when_non_empty`, `line_edit_placeholder_drawn_when_text_empty`, `line_edit_non_empty_text_ignores_placeholder`, plus the four read-only/writable variants) must remain unchanged and continue to pass (AC6). | `quartzite-style/src/default_style_tests.rs` | 1 |
| 3 | Add 9 snapshot tests + 9 golden PNGs (AC7). Light variants (under `quartzite-style/tests/snapshots/shared/`): `line_edit_idle.png`, `line_edit_hovered.png`, `line_edit_pressed.png`, `line_edit_focused.png`, `line_edit_disabled.png` (the visible #407 fold-in anchor), `line_edit_read_only.png`, `line_edit_placeholder.png`. Dark variants (via the existing `render_dark` flow): `dark_line_edit_idle.png`, `dark_line_edit_focused.png`. Each test follows the existing `text_edit_*_renders` / `dark_text_edit_*_renders` shape: build a `LineEdit::new()`, set the single flag via `set_hovered` / `set_pressed` / `set_focused` / `set_enabled(false)` (or `read_only = true`, or `placeholder = "hint".into()`), set canvas geometry via `canvas_rect()`, render with `Palette::default()` (light) or `DARK_PALETTE` via `render_dark`, assert via `snapshot_assert(name, &image)`. **Import update:** add `LineEdit` to the `use quartzite_widgets::{...}` line at the top of `snapshots.rs` (currently `Button, Label, ScrollArea, TextEdit, WidgetExt` at line 27 — extend to include `LineEdit`). Each `LineEdit::new()` call site that needs visible content sets `w.text = "abc".into()` or `w.placeholder = "hint".into()` per the AC7 variant. PNGs are produced by running the test once and committing the generated file (the existing snapshot harness writes the golden when none exists). **Acceptance check after first run:** visually inspect `line_edit_disabled.png` and confirm it visibly differs from `line_edit_idle.png` (half-alpha Base fill + half-alpha Text outline + half-alpha Text glyphs) — this is the #407 fold-in's visible anchor. Also inspect `line_edit_pressed.png` and confirm it visibly differs from `line_edit_focused.png` (pressed → Highlight fill + HighlightedText glyphs; focused → idle Base fill + 2 px FocusRing outline). | `quartzite-style/tests/snapshots.rs`, `quartzite-style/tests/snapshots/shared/line_edit_*.png` (7 PNGs), `quartzite-style/tests/snapshots/shared/dark_line_edit_*.png` (2 PNGs) | 1 |
| 4 | Final gate sweep — run `cargo fmt --check` (no diff), `cargo clippy --workspace --all-targets -- -D warnings` clean (AC8), `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean (AC9), `cargo test -p quartzite-style` green (existing + new tests), `cargo build -p quartzite --no-default-features --features libm` green. Confirm `default_style.rs` line count stays under the 500-line soft target (currently 358 → expected ~380-400 after this work, well within headroom). Confirm no `actionlint` runs needed (no workflow files modified). Confirm no Propagation-Rule sync-group fires (only `default_style.rs` / `default_style_tests.rs` / `snapshots.rs` + new PNGs touched — none of these is in a sync group; `tests/support/mod.rs` is NOT touched, so the Snapshot-helper sync group with `quartzite-widgets/tests/support/mod.rs` stays quiet). | `quartzite-style/src/default_style.rs`, `quartzite-style/src/default_style_tests.rs`, `quartzite-style/tests/snapshots.rs` | 2, 3 |

> **Note on Subtask 1 row density (design-review Round-1 Recommendation 1).** Subtask 1's prose is dense (~30 lines folded into a single table cell). This is intentional and accepted as-is: the row is the entire `impl Paint<LineEdit>` rewrite contract — every role/group/order/conditional is cell-level pinned so the implementing subagent in Group A has zero re-derivation cost. Splitting into "what" + "how" appendix was considered and rejected — the implementer would have to cross-reference two locations, increasing the chance of drift between the contract and the appendix during a context-reset re-entry. The same density appears in the post-#403 design's `impl Paint<TextEdit>` rewrite row, which shipped without issue.

## Handoff plan

`M = 4` (two groups, 3 + 1):

- **Group A:** subtasks 1–3 — paint impl rewrite + recording-painter tests + snapshot tests + goldens (size 3, the non-terminal cap). Entry into Group A is itself a `/context-reset` re-entry per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtask 4 — terminal group (1 subtask; within the 1..=3 range). Runs in its own `/context-reset` subagent and closes Step 8 of `/task`.

## Risks

- **#407 fold-in is a visible behavioural change.** Pre-spec disabled-LineEdit renders identically to enabled-LineEdit (no `maybe_disabled` calls in the current impl); post-spec it renders at half alpha on Base / Text outline / Text glyphs. This is the explicit goal of the fold-in per spec § Key decisions row "Disabled-state interaction" and AC2. Mitigation = the `line_edit_disabled_idle_dims_base_text_outline` recording-painter test (Subtask 2) and the `line_edit_disabled.png` snapshot golden (Subtask 3) both anchor the new behaviour explicitly, so any accidental regression is caught at test-write time.
- **Placeholder double-dim under disabled may surprise.** The composition `disabled(maybe_disabled(palette.color(Text, Normal), false))` produces `≈ × 0.25` alpha. Spec § Key decisions row "Disabled-state interaction" calls this out as intentional. Mitigation = the `line_edit_disabled_placeholder_composes_double_dim` recording-painter test pins the exact arithmetic and documents the composition in the test name. If a future spec wants single-dim placeholder under disabled, this test will be the canonical revision target.
- **`Palette::default` Hover/Pressed derivation may produce near-identical PNGs for `line_edit_hovered.png` vs `line_edit_idle.png`.** The default-light `Base × Hover` is derived as `Base × Normal . blend(WindowText × Normal, 0.06)`, which is a 6% shift from white. This is the same risk pattern called out in the #403 design risks for `text_edit_hovered.png`; the existing `text_edit_hovered.png` golden is visibly distinct enough to commit, and `line_edit_hovered.png` will use the same derivation on the same palette, so the same outcome is expected. Mitigation = inspect the rendered PNG after first run; if not visibly distinct, document the observation in the PR body and either accept the subtle delta (committing the PNG as-is) or swap to a `pinned_palette()`-style fixture for that single snapshot. The #403 spec accepted subtle deltas; this spec follows that precedent.
- **`line_edit_pressed.png` vs `line_edit_focused.png` visual similarity.** Same pattern as the #403 design's risk for `text_edit_pressed.png` vs `text_edit_focused.png`. Pressed → Highlight fill + HighlightedText glyphs (visually different from idle); focused → idle Base fill + 2 px FocusRing outline. The two should differ pixel-wise (fill colour vs outline-width-and-colour). Mitigation = visual inspection after first run, same as #403.
- **`line_edit_placeholder.png` vs `line_edit_idle.png` similarity.** With an empty text and `placeholder = "hint"`, the placeholder is drawn at `disabled(palette.color(Text, Normal))` ≈ half-alpha black on the Base background. The idle golden has empty text (no glyphs rendered at all). The two PNGs will differ in the text region; the golden-image diff test will catch this. Mitigation = visual inspection after first run.
- **File-size budget — `default_style.rs` currently 358 lines.** Spec § Technical constraints budget caps soft target at 500. Expected delta from the rewrite: +20 to +40 lines (state-axis branches, no new helpers). Final ~380-400 lines. Headroom remains. No file-split needed in this spec; if a future spec adds Container state-awareness and the file crosses 500, per-widget paint-impl files become the natural move.
- **Existing five `line_edit_*` tests rely on `Normal`-group lookups and event order.** The new impl resolves to `Normal` group when all state flags default to `false` and `enabled = true` (the default `WidgetBase` initialisation). `maybe_disabled` is a no-op when `enabled`. Event order (fill / optional-overlay / outline / text) is unchanged. Each of the five existing tests passes byte-for-byte — AC6 explicitly requires this and Subtask 2 re-validates it without modifying the tests.
- **`line_edit_read_only_inserts_overlay` test (line 1118) asserts `events[1]` is the overlay.** The new impl preserves the `fill → optional-overlay → outline → text` order. Confirmed by reading both the current LineEdit impl (lines 278-281) and the post-#403 TextEdit impl (lines 187-190). Risk = none.
- **No `actionlint` gate fires.** No workflow files modified. Confirmed.
- **No Propagation-Rule sync-group fires.** The edits touch `default_style.rs`, `default_style_tests.rs`, `snapshots.rs`, and new `.png` files. None of these is in a sync group per AGENTS.md § Propagation Rule. `quartzite-style/tests/support/mod.rs` is NOT touched, so the Snapshot-helper sync group with `quartzite-widgets/tests/support/mod.rs` stays quiet. Confirmed by reading the rule table.
- **API stability — pre-publish: clean breaks.** This spec touches no public API (per § Key decisions row "No new public API"). Confirmed. Risk = none.
- **Doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs"`).** No new public items added. The existing `impl Paint<LineEdit>` carries no doc comment (impls of pub trait for concrete types don't require `missing_docs` per the workspace lint config). Risk = none; Subtask 4's `cargo doc` invocation confirms.

## Test Design

### Recording-painter tests — `quartzite-style/src/default_style_tests.rs`

Location: `#[cfg(test)] mod tests` (path attribute `default_style_tests.rs`) under `quartzite-style/src/default_style.rs`.

Entry points: `DefaultStyle::draw_widget` → routes to `impl Paint<LineEdit>`. Tests instantiate a `LineEdit::new()`, toggle state flags via `WidgetExt::set_hovered` / `set_pressed` / `set_focused` / `set_enabled`, set `text` / `placeholder` / `read_only` fields directly, call `draw_widget` with a `RecordingPainter`, and inspect the captured `PaintEvent` sequence.

#### New tests (10 total — AC1, AC2, AC3, AC4, AC5)

State-axis tests (mirror post-#403 TextEdit shape):

- `hovered_line_edit_uses_derived_hover_fill` (AC1): set `set_hovered(true)`, assert: `FillRect` brush = `palette.color(Base, Hover)`, `DrawRect` pen = `palette.color(Text, Hover)` at width `1.0`, `DrawTextIn` brush = `palette.color(Text, Hover)`. Use `Palette::default()`.
- `pressed_line_edit_uses_highlight_pressed` (AC1): set `set_pressed(true)`, assert: `FillRect` brush = `palette.color(Highlight, Pressed)`, `DrawRect` pen = `palette.color(HighlightedText, Pressed)` at width `1.0`, `DrawTextIn` brush = `palette.color(HighlightedText, Pressed)`. Use `Palette::default()`.
- `focused_line_edit_uses_2px_focus_ring_outline` (AC1): set `set_focused(true)`, assert: `DrawRect` pen colour = `palette.color(FocusRing, Normal)`, pen width = `2.0`. Apply `#[allow(clippy::float_cmp, reason = "exact representable f32/f64 literal comparison in test — value is a power-of-two or integer-encoded fraction")]` per the post-#403 TextEdit precedent.
- `disabled_and_focused_line_edit_paints_outline_under_disabled` (AC3): set `set_enabled(false) + set_focused(true)`, assert: `DrawRect` pen colour = `palette.color(FocusRing, Normal)` at full alpha (NOT halved), pen width = `2.0`. Use the same `#[allow(clippy::float_cmp, ...)]` attribute. Mirrors `disabled_and_focused_text_edit_paints_outline_under_disabled` verbatim (line-range citation dropped — test name is unambiguous and line numbers drift; design-review Round-1 Note 1).
- `precedence_pressed_hovered_line_edit_picks_pressed_fill` (AC3): set both `set_pressed(true) + set_hovered(true)`, assert: `FillRect` brush = `palette.color(Highlight, Pressed)`, NOT `palette.color(Base, Hover)`.

Disabled-axis test (AC2 / #407 fold-in anchor):

- `line_edit_disabled_idle_dims_base_text_outline` (AC2): set `set_enabled(false)` and nothing else; assert all three of: `FillRect` brush = `Brush::solid(maybe_disabled(palette.color(Base, Normal), false))`; `DrawRect` pen colour = `maybe_disabled(palette.color(Text, Normal), false)` at width `1.0`; `DrawTextIn` glyph brush = `Brush::solid(maybe_disabled(palette.color(Text, Normal), false))`. **This test is the explicit anchor for the #407 fold-in** — it would fail against the pre-spec impl (which does not call `maybe_disabled` at all), so it documents the new behaviour. Use `Palette::default()`. Reference the `super::maybe_disabled` private helper via `super::` path, matching how existing tests reference `super::disabled` / `super::READ_ONLY_OVERLAY_ALPHA` (line 1095, 1138).

Read-only + state interaction test (AC5):

- `line_edit_read_only_hovered_overlay_plus_hover_base_fill` (AC5): set `read_only = true` + `set_hovered(true)`, assert: 4 events captured in order = `FillRect(palette.color(Base, Hover))` / `FillRect(palette.color(WindowText, Normal).with_alpha(super::READ_ONLY_OVERLAY_ALPHA))` / `DrawRect(palette.color(Text, Hover) @ 1.0)` / `DrawTextIn("", palette.color(Text, Hover).with_alpha(super::READ_ONLY_TEXT_ALPHA))`. Mirrors `text_edit_read_only_hovered_overlay_plus_hover_base_fill` verbatim (line-range citation dropped — test name is unambiguous and line numbers drift; design-review Round-1 Note 1). Use `Palette::default()`.

Placeholder + state interaction tests (AC4):

- `line_edit_hovered_placeholder_tracks_hover_text` (AC4): set `text = ""`, `placeholder = "hint".into()`, `set_hovered(true)`; assert `DrawTextIn` brush = `Brush::solid(super::disabled(palette.color(Text, Hover)))` and text arg = `"hint"`. Use `Palette::default()`.
- `line_edit_pressed_placeholder_tracks_pressed_text` (AC4): set `text = ""`, `placeholder = "hint".into()`, `set_pressed(true)`; assert `DrawTextIn` brush = `Brush::solid(super::disabled(palette.color(HighlightedText, Pressed)))` and text arg = `"hint"`. Use `Palette::default()`. **Carry a one-line inline `//` comment in the test body** noting that `HighlightedText` is intentional (role-swap on press, not a copy-paste typo of `Text`) — cite spec § Key decisions row "Outline role mapping" so the swap is self-documenting on review (design-review Round-1 Note 2).
- `line_edit_disabled_placeholder_composes_double_dim` (AC4 — #407 fold-in flows through placeholder): set `text = ""`, `placeholder = "hint".into()`, `set_enabled(false)`; assert `DrawTextIn` brush = `Brush::solid(super::disabled(super::maybe_disabled(palette.color(Text, Normal), false)))` (which numerically is `disabled(disabled(palette.color(Text, Normal)))` ≈ `× 0.25` alpha on the default palette) and text arg = `"hint"`. Use `Palette::default()`.

#### Existing tests — pinned baseline (AC6)

All five pre-existing `line_edit_*` tests + their read-only variants must pass unchanged. Enumerate explicitly (lines 1023-1244 of `default_style_tests.rs`):

- `line_edit_records_fill_outline_and_empty_text` (line 1023)
- `line_edit_records_text_when_non_empty` (line 1056)
- `line_edit_placeholder_drawn_when_text_empty` (line 1074)
- `line_edit_non_empty_text_ignores_placeholder` (line 1101)
- `line_edit_read_only_inserts_overlay` (line 1119)
- `line_edit_read_only_with_placeholder_overlays_and_renders_placeholder` (line 1153)
- `line_edit_read_only_dims_text` (line 1181)
- `line_edit_read_only_empty_text_dims_text` (line 1202)
- `line_edit_writable_keeps_full_alpha_text` (line 1230)

These tests construct a `LineEdit` without toggling any state flag and with `is_enabled() == true` by default. The new impl resolves to `state_group(false, false) = ColorGroup::Normal` and `maybe_disabled(_, true) = identity`, so every assertion (`palette.color(Base, Normal)`, `palette.color(Text, Normal)`, `super::disabled(palette.color(Text, Normal))`, etc.) holds byte-for-byte. Event count and ordering (3 events for empty + writable, 4 for empty + read-only, 4 for read-only + placeholder, etc.) are preserved.

#### Fixtures

- Reuse `RecordingPainter::default()`, `PaintEvent`, `first_fill`, `first_draw_text_in`, `first_draw_rect`, `brush_color` (lines 22-172).
- Reuse the existing `line_edit_palette()` (line 1000) and `line_edit_read_only_palette()` (line 1010) helpers where they pin Base/Text values for the existing tests; use `Palette::default()` for the new state-axis tests (the 6% / 16% blended derivations produce numerically distinct values per `ColorGroup`, sufficient for numeric assertions). This mirrors the choice made for the post-#403 Label / TextEdit / ScrollArea state-axis tests, which use `Palette::default()` for the same reason.

### Snapshot tests — `quartzite-style/tests/snapshots.rs`

Location: top-level `#[test]` functions in `quartzite-style/tests/snapshots.rs`.

Entry point: `harness_or_skip("<name>") → render_widget(|painter| DefaultStyle.draw_widget(...)) → snapshot_assert("<name>", &image)`. Dark variants route through `render_dark("dark_<name>", |painter| ...)`.

Scenarios — 9 tests, one per golden listed in AC7:

| Test fn | Golden filename | Palette | Setup |
|---|---|---|---|
| `line_edit_idle_renders` | `line_edit_idle.png` | `Palette::default()` | `LineEdit::new(); w.text = "abc".into();` |
| `line_edit_hovered_renders` | `line_edit_hovered.png` | `Palette::default()` | `LineEdit::new(); w.text = "abc".into(); w.set_hovered(true);` |
| `line_edit_pressed_renders` | `line_edit_pressed.png` | `Palette::default()` | `LineEdit::new(); w.text = "abc".into(); w.set_pressed(true);` |
| `line_edit_focused_renders` | `line_edit_focused.png` | `Palette::default()` | `LineEdit::new(); w.text = "abc".into(); w.set_focused(true);` |
| `line_edit_disabled_renders` | `line_edit_disabled.png` | `Palette::default()` | `LineEdit::new(); w.text = "abc".into(); w.set_enabled(false);` |
| `line_edit_read_only_renders` | `line_edit_read_only.png` | `Palette::default()` | `LineEdit::new(); w.text = "abc".into(); w.read_only = true;` |
| `line_edit_placeholder_renders` | `line_edit_placeholder.png` | `Palette::default()` | `LineEdit::new(); w.placeholder = "hint".into();` (text stays empty) |
| `dark_line_edit_idle_renders` | `dark_line_edit_idle.png` | `DARK_PALETTE` | `LineEdit::new(); w.text = "abc".into();` |
| `dark_line_edit_focused_renders` | `dark_line_edit_focused.png` | `DARK_PALETTE` | `LineEdit::new(); w.text = "abc".into(); w.set_focused(true);` |

Build pattern: identical to `text_edit_plain_renders` / `dark_text_edit_focused_renders`. Each test:

```rust
let Some(mut harness) = harness_or_skip("<name>_renders") else { return; };
let mut w = LineEdit::new();
w.set_geometry(canvas_rect());
w.text = "abc".into();  // (or .placeholder = "hint".into() for placeholder)
w.<flag setter>;          // (omitted for idle)
let image = harness.render_widget(|painter| {
    DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
});
snapshot_assert("<name>", &image);
```

For the `line_edit_disabled_renders` snapshot (the #407 fold-in anchor): visual inspection after first run must confirm the rendered image visibly differs from `line_edit_idle.png` — Base fill at half alpha (≈ 50% light grey on white background, depending on `Palette::default()` Base value), Text outline at half alpha, Text glyphs at half alpha. The visible difference is the artefact that closes #407.

**PR-body callout (design-review Round-1 Recommendation 2):** the Step 12 PR body must surface the visible #407 fold-in explicitly. Include a "Visible behaviour change" bullet under **Summary** noting `line_edit_disabled.png` vs `line_edit_idle.png` as the auditable before/after pair (the PR diff already contains both PNGs — the bullet just calls out which two to compare so reviewers see the half-alpha treatment without trawling the snapshot directory).

Dark variants use the existing `render_dark` helper (lines 191-197 of `snapshots.rs`).

**Required `use` update at the top of `snapshots.rs`:** the current import line 27 reads `use quartzite_widgets::{AsWidget, Button, Label, ScrollArea, TextEdit, WidgetExt};`. Add `LineEdit` to the list (alphabetically: `Button, Label, LineEdit, ScrollArea, TextEdit`).

### `quartzite-style/tests/support/mod.rs`

Not touched. The Snapshot-helper sync group (`quartzite-widgets/tests/support/mod.rs` ↔ `quartzite-style/tests/support/mod.rs`) does not need a propagation pass for this spec.

## Open questions

_None remain._ The spec § Open questions section is already empty ("Round-1 resolved the only design-affecting ambiguity"), and every visual / test-shape / fold-in decision is pinned cell-by-cell in § Key decisions and § Acceptance Criteria. The design above translates those decisions into atomic implementation tasks without introducing any new ambiguity.
