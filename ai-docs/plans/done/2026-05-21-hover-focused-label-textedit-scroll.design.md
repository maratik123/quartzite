# Design: Hover / pressed / focused rendering on Label / TextEdit / ScrollArea

**Issue:** #403
**Date:** 2026-05-21

## Approach

The spec is unusually pinned: visual mappings for all three widgets are spelled out cell-by-cell in § Key decisions, the two-axis palette API (`palette.color(role, group)` + `ColorRole::FocusRing` + the post-#402 derivation) already exists on `master`, and the post-#402 `impl Paint<Button> for DefaultStyle` (`quartzite-style/src/default_style.rs:79-131`) is an in-tree pattern reference for the exact code shape. The design's job is therefore to translate the spec mappings into three minimal paint-impl rewrites + one test-organisation plan + handoff-grouping, not to invent architecture.

**Chosen approach — direct in-place rewrites of the three `Paint<W>` impls.** Each of `impl Paint<Label>`, `impl Paint<TextEdit>`, `impl Paint<ScrollArea>` for `DefaultStyle` gets the same skeleton the Button impl already uses (`enabled`/`hovered`/`pressed`/`focused` reads → `ColorGroup` selector → per-widget role mapping → `maybe_disabled` wrap → focus-outline branch → painter calls). No new public API, no `Style`/`Paint` trait changes, no new helpers in v1.

**State-group selector — inlined per impl, not extracted (yet).** The Button impl uses an inline `if pressed { Pressed } else if hovered { Hover } else { Normal }`. The spec § Deferred row 1 explicitly leaves a `fn state_group(w: &impl WidgetExt) -> ColorGroup` extraction to the design phase as optional. After this work the file will contain four call sites of the same 6-line selector (Button + 3 new), which crosses the typical extract-on-3rd-duplicate threshold and earns its keep as a tiny `#[inline] const fn state_group(pressed: bool, hovered: bool) -> ColorGroup` private helper inside `default_style.rs`. The helper takes booleans (not `&impl WidgetExt`) to (a) sidestep a generic-monomorphisation pass for what is plainly a 3-arm branch, (b) keep the call site readable (`state_group(pressed, hovered)`), (c) carry the `_Simple._` shape: no branches/loops beyond the inline match, no calls, ≤ 1 non-simple call → tag as `#[inline]` concrete fn per `ai-docs/code-style.md` Simple-doc-tag rule. **Decision:** extract. Touches all four state-aware impls in one commit; cost is one tiny helper, benefit is no copy/paste drift between Button and the three new impls when a future spec changes precedence (e.g. introducing `ColorGroup::Disabled`).

**Visual mappings — verbatim from spec § Key decisions.** The three widget rows in that table are pinned cell-by-cell; the design must not reinterpret them. Recap of the chosen role/group pairs (for each, every resolved colour passes through `maybe_disabled(_, enabled)` and focused widens the outline to 2 px with `FocusRing × Normal`):

- **Label:** fill = `(Window | Highlight, group)` (role-swap on `pressed`); text = `(WindowText | HighlightedText, group)` (same role-swap); no idle outline; focused adds 2 px `FocusRing × Normal` outline.
- **TextEdit:** fill = `(Base | Highlight, group)`; text = `(Text | HighlightedText, group)`; outline = idle 1 px `(Text | HighlightedText, group)` (matches text colour under pressed for legibility), widened to 2 px `FocusRing × Normal` when focused; read-only overlay paints unchanged on top of the state fill (orthogonal to state visuals).
- **ScrollArea:** fill = `(Base | Highlight, group)`; outline = idle 1 px `(WindowText | HighlightedText, group)`, widened to 2 px `FocusRing × Normal` when focused; no text rendering at all (unchanged from the current chrome-only impl).

The `pressed`-or-`checked` role-swap from Button collapses to plain `pressed` for these three widgets — none of them carries a `checked` flag per spec § Key decisions row "State precedence". This is encoded as a direct `if pressed { … } else { … }` per impl; no shared helper warranted.

**Outline-colour rule under pressed.** Spec § Key decisions row "TextEdit visual mapping" specifies the pressed outline reads `HighlightedText × Pressed` (matches text for legibility under inverted fill); row "ScrollArea visual mapping" the same. Idle/hover keeps the role consistent with their idle baselines (`Text` for TextEdit, `WindowText` for ScrollArea) so the outline tracks the text colour where text exists, and the idle stroke colour where it doesn't. The Label impl never has an idle outline, so this rule does not apply.

**Read-only overlay (TextEdit) — paint-order preserved.** The current impl paints: `fill_rect(Base)` → optional `fill_rect(overlay)` → `draw_rect(outline)` → `draw_text_in(text)`. The new impl keeps the same 4-step order with state lookups substituted in: `fill_rect(state_fill)` → optional `fill_rect(overlay)` → `draw_rect(state_outline, state_outline_width)` → `draw_text_in(state_text)`. AC2 calls out the `(read_only, hovered)` combination test: both the hover-state base fill AND the read-only overlay fill must be captured. Existing `text_edit_read_only_inserts_overlay_fill` / `text_edit_read_only_dims_text` tests rely on `events[1]` being the overlay — that ordering is preserved.

**Read-only text colour interaction.** The current impl's text-colour rule is `if read_only { Text × Normal with READ_ONLY_TEXT_ALPHA } else { Text × Normal }`. The new impl generalises to: `if read_only { state_text_colour.with_alpha(READ_ONLY_TEXT_ALPHA) } else { state_text_colour }`. That is — the read-only dim composes over whatever text colour the state branch resolves. Under `pressed + read_only` the result is `HighlightedText × Pressed` with 0.65 alpha. Under `hovered + read_only` it's `Text × Hover` with 0.65 alpha. This is the natural extension of the existing rule; the spec § Scope bullet "the read-only overlay still applied on top" and § Key decisions row "Read-only overlay applies on top of every state's fill (orthogonal)" both license it.

**Snapshot test plan — 12 new goldens.** AC6 enumerates them exactly: `label_{hovered,pressed,focused}.png`, `text_edit_{hovered,pressed,focused}.png`, `scroll_area_{hovered,pressed,focused}.png` under `shared/` (light); plus `dark_label_focused.png`, `dark_text_edit_focused.png`, `dark_scroll_area_focused.png` under the dark-theme `render_dark` flow. No naming deviation from the AC6 list.

**Rejected alternatives:**

- **Trait-level `Paint::state_group` default method.** Adds public API (the spec § Out of scope explicitly forbids trait redesign without a Design Amendment). The 6-line selector does not warrant a default method on `Paint<W>` — `Paint<W>` is parameterised by widget type, not by state, so the helper has no natural home on the trait. The private `state_group` free fn in `default_style.rs` carries zero public-API cost and stays out of the trait surface.
- **Extract `compute_state_colors` per widget.** Each impl would gain a 10-line helper named `label_colors(w, palette) -> (Brush, Brush, Option<(Color, f32)>)` etc. Net file growth, no readability win (the impls are short and linear), and it spreads the visual mapping over two functions per widget where it currently fits in one. Skip.
- **Generic `paint_framed_state_aware(geom, fill_role, text_role, outline_role, …)` shared across TextEdit / ScrollArea / Container / LineEdit.** Tempting because the chrome shapes overlap, but the per-widget visual decisions diverge (Label has no idle outline; ScrollArea has no text; TextEdit carries read-only overlay; Container is out of scope; LineEdit is out of scope). Generalising now would over-fit on the three in-scope widgets and force a refactor when LineEdit / Container join in a future spec. Spec § Scope final bullet ("If the design phase discovers a concrete trait-level need …") covers this — no such need surfaced; defer.
- **Roll the read-only-text dim into a separate `state_text_dim_for_read_only(color)` helper.** Single call site, no win. Inline as `if w.read_only { c.with_alpha(READ_ONLY_TEXT_ALPHA) } else { c }`.
- **Skip the `state_group` helper extraction and leave four copies of the selector in place.** Considered. The Button impl alone holds the selector today; adding three identical copies makes the four-way drift risk tangible. The helper is 4 lines + `#[inline] const`; extraction is cheaper than the next copy-paste audit.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Extract `#[inline] const fn state_group(pressed: bool, hovered: bool) -> ColorGroup` into `default_style.rs` (private), then refactor the existing Button impl to call it. No behaviour change. The helper is `const`-able and pure (3-arm match, no calls) → classifies as `_Simple._` shape → carries `#[inline]` concrete-fn marker per `ai-docs/code-style.md` Simple-doc-tag rule. Doc comment: `/// _Simple._` on a private fn is not needed (only public items carry doc requirements); the `#[inline]` marker alone is sufficient. | `quartzite-style/src/default_style.rs` | — |
| 2 | Rewrite `impl Paint<Label> for DefaultStyle` to read `enabled` / `hovered` / `pressed` / `focused`, compute `(fill_role, text_role) = if pressed { (Highlight, HighlightedText) } else { (Window, WindowText) }`, resolve `(fill_color, text_color)` via `maybe_disabled(palette.color(role, group), enabled)`, paint `fill_rect(geom, &Brush::solid(fill_color))`, then if `focused` paint `draw_rect(geom, &Pen::new(palette.color(FocusRing, Normal), FOCUS_RING_WIDTH), &Brush::solid(Color::TRANSPARENT))` — **FocusRing pen colour is NOT wrapped through `maybe_disabled`** (full alpha always, per design-system additive-overlay rule + Button impl), finally paint `draw_text_in(geom, &w.text, &font, &Brush::solid(text_color), w.alignment)`. Idle baseline = identical to today (Window/Normal fill + WindowText/Normal text with `w.alignment`). | `quartzite-style/src/default_style.rs` | 1 |
| 3 | Rewrite `impl Paint<TextEdit> for DefaultStyle` to read the same four flags, compute `(fill_role, text_role) = if pressed { (Highlight, HighlightedText) } else { (Base, Text) }`, compute `outline_role_idle = if pressed { HighlightedText } else { Text }`, resolve `(fill_color, text_color, outline_color)` via `maybe_disabled(palette.color(role, group), enabled)`, paint `fill_rect(state_fill)` → if `read_only` paint `fill_rect(read_only_overlay)` → paint `draw_rect` with `(outline_color, 1.0)` if not focused, else `(palette.color(FocusRing, Normal), FOCUS_RING_WIDTH)` — **FocusRing pen NOT through `maybe_disabled`** → paint `draw_text_in(geom, &w.plain_text, &font, &Brush::solid(if read_only { text_color.with_alpha(READ_ONLY_TEXT_ALPHA) } else { text_color }), Alignment::Left)`. Event order preserved (fill / overlay / outline / text). | `quartzite-style/src/default_style.rs` | 1 |
| 4 | Rewrite `impl Paint<ScrollArea> for DefaultStyle` to read the same four flags, compute `fill_role = if pressed { Highlight } else { Base }`, `outline_role_idle = if pressed { HighlightedText } else { WindowText }`, resolve `(fill_color, outline_color)` via `maybe_disabled(palette.color(role, group), enabled)`, paint `fill_rect(state_fill)` → paint `draw_rect` with `(outline_color, 1.0)` if not focused, else `(palette.color(FocusRing, Normal), FOCUS_RING_WIDTH)` — **FocusRing pen NOT through `maybe_disabled`**. No text rendering. Event order preserved (fill / outline). | `quartzite-style/src/default_style.rs` | 1 |
| 5 | Add recording-painter unit tests for Label / TextEdit / ScrollArea state branches (AC1, AC2, AC3, AC4). For each widget: `hovered_*` (fill + text + outline colours where applicable), `pressed_*` (fill + text + outline colours), `focused_*` (outline width 2.0 + colour FocusRing × Normal), `disabled_and_focused_*` (outline at 2 px under disabled), `precedence_pressed_hovered_*` (pressed wins on fill axis). For TextEdit specifically also `text_edit_read_only_hovered_*` (overlay AND hover base fill both captured per AC2). All new tests use a `pinned_palette()`-style fixture (reuse the existing one or add a sibling that pins `Window` / `Base` / `WindowText` / `Text` / `Highlight` / `HighlightedText` / `FocusRing` if needed). Re-confirm existing idle tests (`label_records_fill_and_text_with_label_alignment`, `text_edit_records_fill_outline_and_text`, `text_edit_read_only_inserts_overlay_fill`, `text_edit_read_only_dims_text`, `text_edit_writable_keeps_full_alpha_text`, `read_only_overlay_derives_from_custom_window_text`, `scroll_area_records_fill_and_outline_only`) still pass byte-for-byte (AC5). | `quartzite-style/src/default_style_tests.rs` | 2, 3, 4 |
| 6 | Add 12 snapshot tests + 12 golden PNGs (AC6). Light: `label_hovered`, `label_pressed`, `label_focused`, `text_edit_hovered`, `text_edit_pressed`, `text_edit_focused`, `scroll_area_hovered`, `scroll_area_pressed`, `scroll_area_focused`. Dark: `dark_label_focused`, `dark_text_edit_focused`, `dark_scroll_area_focused`. Each test follows the existing `button_hovered_renders` / `dark_button_focused_renders` shape: build the widget, set the single flag via `set_hovered` / `set_pressed` / `set_focused`, set canvas geometry via `canvas_rect()`, render with `Palette::default()` (light) or `DARK_PALETTE` via `render_dark`, assert via `snapshot_assert(name, &image)`. PNGs are produced by running the test once and committing the generated file (the existing snapshot harness writes the golden when none exists). **Acceptance check after first run:** visually inspect the three `*_pressed.png` goldens and confirm each differs visibly from its `*_focused.png` sibling (pressed → Highlight fill; focused → idle fill + 2 px ring outline). | `quartzite-style/tests/snapshots.rs`, `quartzite-style/tests/snapshots/shared/{label,text_edit,scroll_area,dark_label,dark_text_edit,dark_scroll_area}_*.png` | 2, 3, 4 |
| 7 | Final gate sweep — run `cargo fmt --check` (no diff), `cargo clippy --workspace --all-targets -- -D warnings` clean (AC7), `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean (AC8), `cargo test -p quartzite-style` green incl. all new + existing tests, `cargo build -p quartzite --no-default-features --features libm` green. Confirm `default_style.rs` line count stays under the 500-line soft target (spec § Technical constraints budget: 280 → expected ~330–360 after this work). | `quartzite-style/src/default_style.rs`, `quartzite-style/src/default_style_tests.rs`, `quartzite-style/tests/snapshots.rs` | 5, 6 |

## Handoff plan

`M = 7` (three groups, 3 + 3 + 1):

- **Group A:** subtasks 1–3 — helper extraction + Label rewrite + TextEdit rewrite (size 3, the non-terminal cap). Entry into Group A is itself a `/context-reset` re-entry per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — ScrollArea rewrite + recording-painter tests + snapshot tests + goldens (size 3, the non-terminal cap).
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group C with fresh context.
- **Group C:** subtask 7 — terminal group (1 subtask; within the 1..=3 range). Runs in its own `/context-reset` subagent and closes Step 8 of `/task`.

## Risks

- **Snapshot test goldens differ in CI vs local font rendering.** The existing `button_hovered_renders` / `dark_button_focused_renders` snapshots already render with the same `parley/skrifa` stack, and the existing `label.png` / `text_edit_plain.png` goldens are committed and stable. Risk is the same as for any new snapshot in this suite; mitigation = follow the existing pattern verbatim (same canvas size, same widget construction shape, same harness call), and trust the GPU-tests CI job's golden-image diff to flag drift.
- **`*_pressed.png` ↔ `*_focused.png` visual similarity.** After Subtask 6's first-run visual confirmation, if any pressed/focused golden pair is observed to look near-identical (e.g. under the dark palette where `Highlight × Pressed` may be close to the idle fill for some roles), record the observation here so the historical reasoning is preserved for future reviewers.
- **`text_edit_pressed` snapshot under default palette may look near-identical to `text_edit_focused` because of the design-system derivation.** Default `Highlight × Pressed` is `#006CD6`, but the read-only overlay rule says it's overlaid with `WindowText.with_alpha(0.10)`. The non-read-only pressed case will simply show the Highlight-pressed fill behind black text; the focused case shows the same idle fill behind a 2 px FocusRing outline. These are visually distinct (fill colour vs outline-width-and-colour); the PNGs should differ pixel-wise and the golden-diff test will fire if they don't. Mitigation = inspect the rendered PNGs once after commit to confirm visible difference.
- **Hover + pressed flags both true on Label/TextEdit/ScrollArea.** AC4 prescribes precedence: `pressed` wins. The selector `state_group(pressed, hovered)` enforces this — `pressed` checked first. Same precedence as Button, which has a passing test (`pressed_button_uses_highlight_pressed` + the precedence tests). No new risk.
- **`maybe_disabled` already accepts a `Color` and composes over any `palette.color(role, group)` result.** Confirmed by reading `default_style.rs:274` + the Button impl. No new failure mode.
- **File-size budget pressure.** Spec § Technical constraints expects `default_style.rs` to stay under 500 after this work. Current 280 → estimated +60-80 lines (three impl rewrites cost ~15-20 each, helper costs ~4, no new tests in this file). Headroom remains. If a future spec adds Container/LineEdit state-awareness and the file crosses 500, splitting into per-widget paint-impl files becomes the natural move; that is a follow-up not in this spec.
- **No `ColorRole::FocusRing` in old themes.** Spec § Out of scope point 3 says no new roles; `FocusRing` shipped via #402 and `DARK_PALETTE` already seeds it. `Palette::default` and `DARK_PALETTE` both have it. Risk = none.
- **API stability — pre-publish: clean breaks.** This spec touches no public API (per § Key decisions row "No new public API"). Even if it did, AGENTS.md § API Stability allows clean breaks. Risk = none for this spec.
- **Propagation Rule — does this design edit trigger sister-file updates?** No. The edits touch `quartzite-style/src/default_style.rs`, `quartzite-style/src/default_style_tests.rs`, `quartzite-style/tests/snapshots.rs`, and new `.png` files. None of these is in a sync group per AGENTS.md § Propagation Rule. The `quartzite-widgets/tests/support/mod.rs` ↔ `quartzite-style/tests/support/mod.rs` Snapshot-helper sync group is not touched — `support/mod.rs` is unchanged. No propagation fires.

## Test Design

### Recording-painter tests — `quartzite-style/src/default_style_tests.rs`

Location: `#[cfg(test)] mod tests` (path attribute `default_style_tests.rs`) under `quartzite-style/src/default_style.rs`.

Entry points: `DefaultStyle::draw_widget` → routes to the relevant `impl Paint<W>` body. Tests instantiate the widget, toggle state flags via `WidgetExt::set_hovered` / `set_pressed` / `set_focused` / `set_enabled`, call `draw_widget` with a `RecordingPainter`, and inspect the captured `PaintEvent` sequence.

#### Per-widget hover / pressed / focused tests (mirror Button shape)

Each widget (Label / TextEdit / ScrollArea) gets:

- `hovered_<widget>_uses_derived_hover_fill` — set `hovered=true`, assert: `FillRect` brush colour equals `palette.color(<fill_role>, ColorGroup::Hover)` AND differs from `palette.color(<fill_role>, ColorGroup::Normal)`. For Label and TextEdit also assert `DrawTextIn` brush equals `palette.color(<text_role>, ColorGroup::Hover)`. For TextEdit also assert outline pen colour equals `palette.color(Text, ColorGroup::Hover)`. For ScrollArea also assert outline pen colour equals `palette.color(WindowText, ColorGroup::Hover)`.

- `pressed_<widget>_uses_highlight_pressed` — set `pressed=true`, assert: `FillRect` brush colour equals `palette.color(Highlight, ColorGroup::Pressed)`. For Label and TextEdit also `DrawTextIn` brush equals `palette.color(HighlightedText, ColorGroup::Pressed)`. For TextEdit also outline pen colour equals `palette.color(HighlightedText, ColorGroup::Pressed)`. For ScrollArea also outline pen colour equals `palette.color(HighlightedText, ColorGroup::Pressed)`.

- `focused_<widget>_uses_2px_focus_ring_outline` — set `focused=true`, assert: a `DrawRect` event exists with `pen.width() == 2.0` and `pen.color() == palette.color(FocusRing, ColorGroup::Normal)`. For Label this is the only `DrawRect` event (no idle outline). For TextEdit / ScrollArea this `DrawRect` replaces the idle 1 px outline (verify pen width differs from a control idle render at width 1.0). Apply `#[allow(clippy::float_cmp, reason = ...)]` same as Button.

- `disabled_and_focused_<widget>_paints_outline_under_disabled` — set `enabled=false` + `focused=true`, assert: focused-outline `DrawRect` pen width still `2.0`, AND its colour equals `palette.color(FocusRing, ColorGroup::Normal)` at **full alpha** (NOT half-alpha). The FocusRing pen colour is exempt from `maybe_disabled` — it is an additive overlay that "never alpha-halves" per `design-system/README.md` § *Animation, hover, press* and the shipped Button impl (`default_style.rs:108-115`) + its passing test `precedence_disabled_pressed_focused`. Open Question 1 resolved: follow design-system rule + Button behaviour (full alpha).

- `precedence_pressed_hovered_<widget>_picks_pressed_fill` — set both `pressed=true` + `hovered=true`, assert: `FillRect` brush colour equals `palette.color(Highlight, ColorGroup::Pressed)`, NOT `palette.color(<fill_role>, ColorGroup::Hover)`. Confirms the state-group selector picks `Pressed` over `Hover`.

#### TextEdit-only — `(read_only, hovered)` overlay-plus-state test

- `text_edit_read_only_hovered_overlay_plus_hover_base_fill` (AC2 specific). Set `read_only=true` + `hovered=true`, assert: 4 events captured in order = `FillRect(state_fill)` / `FillRect(overlay)` / `DrawRect(outline)` / `DrawTextIn(text)`. First `FillRect` brush colour equals `palette.color(Base, ColorGroup::Hover)`. Second `FillRect` brush colour equals `palette.color(WindowText, ColorGroup::Normal).with_alpha(READ_ONLY_OVERLAY_ALPHA)`. Outline pen colour equals `palette.color(Text, ColorGroup::Hover)`. Text brush colour equals `palette.color(Text, ColorGroup::Hover).with_alpha(READ_ONLY_TEXT_ALPHA)`.

#### Existing tests — pinned baseline

All eight pre-existing tests on the three widgets must pass unchanged (AC5). Enumerate explicitly:

- `label_records_fill_and_text_with_label_alignment`
- `text_edit_records_fill_outline_and_text`
- `text_edit_read_only_inserts_overlay_fill` (relies on `events[1]` being the overlay — confirm event order preserved)
- `text_edit_read_only_dims_text` (relies on `events[3]` being the text — confirm 4-event order preserved when `read_only=true`)
- `text_edit_writable_keeps_full_alpha_text`
- `read_only_overlay_derives_from_custom_window_text` (relies on `events[1]` being the overlay)
- `scroll_area_records_fill_and_outline_only`
- `unknown_widget_type_produces_no_events`

These tests don't toggle any state flag, so the new code paths default to `Normal` group + idle role mapping. The idle behaviour is identical to today; the existing assertions remain true byte-for-byte.

#### Fixtures

- Reuse `RecordingPainter::default()`, `PaintEvent`, `first_fill`, `first_draw_text_in`, `first_draw_rect`, `brush_color` helpers as-is.
- Use `Palette::default()` as the primary fixture for the new state tests — the default derivation produces numerically distinct Normal / Hover / Pressed colour values for `Window`, `Base`, `Highlight`, `WindowText`, `Text`, `HighlightedText`, and `FocusRing`; recording-painter tests compare colours numerically, so the 6% / 16% blended derivations suffice. Reserve `pinned_palette()` only when an assertion must visually disambiguate one role from another in the same image (the Button pattern: `hovered_button_uses_derived_hover_fill` uses `pinned_palette()` because Window-hover at 6% blend from `Palette::default()` is not visually distinguishable in a golden — but for the recording-painter unit tests numeric equality is the gate, not visual distinction).

### Snapshot tests — `quartzite-style/tests/snapshots.rs`

Location: top-level `#[test]` functions in `quartzite-style/tests/snapshots.rs`.

Entry point: `harness_or_skip("<name>") → render_widget(|painter| DefaultStyle.draw_widget(...)) → snapshot_assert("<name>", &image)`. Dark variants route through `render_dark("dark_<name>", |painter| ...)`.

Scenarios — 12 tests, one per golden listed in AC6:

| Test fn | Golden filename | Palette | Flag |
|---|---|---|---|
| `label_hovered_renders` | `label_hovered.png` | `Palette::default()` | `set_hovered(true)` |
| `label_pressed_renders` | `label_pressed.png` | `Palette::default()` | `set_pressed(true)` |
| `label_focused_renders` | `label_focused.png` | `Palette::default()` | `set_focused(true)` |
| `text_edit_hovered_renders` | `text_edit_hovered.png` | `Palette::default()` | `set_hovered(true)` |
| `text_edit_pressed_renders` | `text_edit_pressed.png` | `Palette::default()` | `set_pressed(true)` |
| `text_edit_focused_renders` | `text_edit_focused.png` | `Palette::default()` | `set_focused(true)` |
| `scroll_area_hovered_renders` | `scroll_area_hovered.png` | `Palette::default()` | `set_hovered(true)` |
| `scroll_area_pressed_renders` | `scroll_area_pressed.png` | `Palette::default()` | `set_pressed(true)` |
| `scroll_area_focused_renders` | `scroll_area_focused.png` | `Palette::default()` | `set_focused(true)` |
| `dark_label_focused_renders` | `dark_label_focused.png` | `DARK_PALETTE` | `set_focused(true)` |
| `dark_text_edit_focused_renders` | `dark_text_edit_focused.png` | `DARK_PALETTE` | `set_focused(true)` |
| `dark_scroll_area_focused_renders` | `dark_scroll_area_focused.png` | `DARK_PALETTE` | `set_focused(true)` |

Build pattern: identical to `button_hovered_renders` / `dark_button_focused_renders`. Widgets use `Label::new("hi".into())` / `TextEdit::new()` + `plain_text = "abc".into()` / `ScrollArea::new()`; geometry = `canvas_rect()`.

### `quartzite-style/tests/support/mod.rs`

Not touched. The Snapshot-helper sync group (`quartzite-widgets/tests/support/mod.rs` ↔ `quartzite-style/tests/support/mod.rs`) does not need a propagation pass for this spec.

## Open questions

_None remain._

- **Open Question 1 (resolved):** FocusRing pen colour under disabled → **full alpha**, matching the design-system additive-overlay rule ("never alpha-halved") and the shipped Button impl. Spec amended accordingly.
- **Open Question 2 (resolved):** Existing `text_edit_read_only_inserts_overlay_fill` test left unchanged (AC5); the new `text_edit_read_only_hovered_overlay_plus_hover_base_fill` test covers the combination.
