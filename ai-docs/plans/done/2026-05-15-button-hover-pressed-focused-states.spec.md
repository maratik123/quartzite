# Button hover / pressed / focused visual states

**Source:** issue #316
**Date:** 2026-05-15
**Tracked in:** #316

> Surfaced by `/triage` from [`ai-docs/deferred/widget-backlog.md`](../deferred/widget-backlog.md). v1 `DefaultStyle::draw_button` paints only the `checked` and `is_enabled()` cues. Hover / pressed / focused visual states require new state fields on `WidgetBase` plus a rendering path in `DefaultStyle` that consumes them. The source [default-style-content spec](done/2026-05-13-default-style-content.spec.md) explicitly defers this to "when input plumbing lands."

## Scope

Add `hovered`, `pressed`, and `focused` state to widgets and render them on `Button`.

Concretely:

- New `pub` fields on `quartzite_widgets::WidgetBase`:
  - `pub hovered: bool` — `true` while the mouse cursor is over the widget's `geometry()`.
  - `pub pressed: bool` — `true` while a mouse button is held with press-initiated state on this widget.
  - `pub focused: bool` — `true` while the widget owns keyboard focus.
  All three default to `false` in `WidgetBase::new()` / `Default`.
- New ergonomic accessors / setters on `quartzite_widgets::WidgetExt` (blanket-impl on every `AsWidget`):
  - `fn is_hovered(&self) -> bool`, `fn set_hovered(&mut self, value: bool)`.
  - `fn is_pressed(&self) -> bool`, `fn set_pressed(&mut self, value: bool)`.
  - `fn is_focused(&self) -> bool`, `fn set_focused(&mut self, value: bool)`.
  All `#[inline]` `_Simple._`-shape readers/writers backed by the `WidgetBase` fields.
- Updated `WidgetExt` event-handler defaults so the deferred input-plumbing pass only routes events; flag mutation is automatic:
  - `on_mouse_press` default → `self.set_pressed(true)` (was no-op).
  - `on_mouse_release` default → `self.set_pressed(false)` (was no-op).
  - `on_focus_in` default → `self.set_focused(true)` (was no-op).
  - `on_focus_out` default → `self.set_focused(false)` (was no-op).
  Hover flag mutation is **not** added to a default hook in v1 — `MouseEventKind` has no `Enter` / `Leave` variants yet (see § Out of scope); the input-plumbing pass will synthesise enter/leave and call `set_hovered` directly.
- `DefaultStyle::draw_button` consumes the three flags. Visual treatment derived from existing `ColorRole`s (no new palette roles in v1 — see § Key decisions); precedence specified under § Key decisions.
- A `RecordingPainter`-style unit test per state (hover / pressed / focused / hover+focused) in `quartzite-style/src/default_style.rs` mirrors the AC2–AC8 style: tests toggle the `WidgetBase` flags directly and assert the captured `Brush` / `Pen` colours and event count differ from the idle baseline.
- A unit test in `widget_ext.rs` confirms each updated event-handler default mutates the matching flag (e.g. `on_mouse_press(&MouseEvent { … })` sets `pressed == true`; `on_focus_in()` sets `focused == true`; etc.).
- Snapshot tests are extended in `quartzite-style/tests/snapshots.rs` with one golden per visible state (hover, pressed, focused — disabled is already covered) using the existing `shared/` golden flow.

## Out of scope

- **Event plumbing** (renderer → widget tree). Hit-testing, `MouseEnter` / `MouseLeave` synthesis, focus traversal (Tab / Shift+Tab), per-widget routing for press/release through the widget hierarchy, and the renderer-side glue that actually invokes the `WidgetExt` event-handler hooks are deferred to a separate "input-plumbing pass" issue. Tests toggle the flags directly via `WidgetExt::set_hovered` / `set_pressed` / `set_focused` and also exercise the updated event-handler defaults by calling them directly. The deferred row recorded in [`widget-backlog.md` lines 246 / 249](../deferred/widget-backlog.md) describes that follow-up — this spec **unblocks the rendering side and pre-wires the `WidgetExt`-side flag-mutation defaults**. Note: the `on_mouse_press` / `on_mouse_release` / `on_focus_in` / `on_focus_out` default-impl bodies become the canonical state-machine for `pressed` / `focused` here; the input-plumbing pass only routes events to the right widget.
- New `ColorRole` variants. v1 derives hover / pressed / focused visuals from existing roles (`Button`, `ButtonText`, `Highlight`, `HighlightedText`) plus alpha blending and pen-width adjustments. A future palette extension MAY add `Hover` / `Pressed` / `FocusRing` roles; that decision belongs in its own spec.
- Hover / pressed / focused rendering on `Label` / `TextEdit` / `ScrollArea` / other widgets. Only `Button` rendering changes; `DefaultStyle::draw_label`, `draw_text_edit`, `draw_scroll_area` are untouched. (Other widgets' rendering can opt in via a follow-up once the input plumbing pass lands.)
- Changes to `MouseEventKind` (currently `Press`, `Release`, `Move` — no `Enter` / `Leave`). Synthesis of enter/leave events from `Move` + hit-testing is part of the deferred input-plumbing pass.
- Mutating `WidgetBase::focus_policy` semantics. `focused` and `focus_policy` are orthogonal — the renderer is responsible for honouring `focus_policy` when deciding *whether* to grant focus; this spec only adds the *state* flag.
- Cursor-shape changes on hover (e.g. `CursorShape::Hand` when hovering a button). Out of scope; the existing `WidgetBase::cursor` field stays unchanged.

## Deferred

- Event-driven flag updates (renderer-side hit-testing + focus traversal + Enter/Leave synthesis) | needs a coordinated renderer / event-types / widget-base API design pass | the existing "input plumbing pass" deferred row in `widget-backlog.md` lines 246 / 249 — this issue surfaces that row's complement
- New palette roles for hover / pressed / focus ring | requires a `ColorRole` extension that touches every theme + `Palette::default` seeding | follow-up spec when a designer-driven theming overhaul lands
- Hover / pressed / focused rendering on `Label` / `TextEdit` / `ScrollArea` | requires per-widget visual idioms + likely a wider `Style` redesign | follow-up after v1 lands and event plumbing is in place
- Cursor-shape change on hover | `WidgetBase::cursor` already exists but no path mutates it from hover state | follow-up with the input-plumbing pass

## Key decisions

| Question | Decision |
|---|---|
| Where do the state flags live | On `WidgetBase` (universal). Issue body and source spec are explicit: "new fields on `WidgetBase` (`hovered`, `pressed`, `focused`)". Per-widget storage would force every widget to duplicate the state. |
| Public visibility of the new fields | `pub`, matching every other `WidgetBase` field. `WidgetExt` accessors are the ergonomic surface; direct field access stays available for low-level callers (event plumbing pass, tests). |
| `WidgetExt` accessor naming | `is_hovered` / `set_hovered`, `is_pressed` / `set_pressed`, `is_focused` / `set_focused` — matches the existing `is_enabled` / `set_enabled` and `is_visible` / `set_visible` shape. |
| Event-driven mutation of the flags (`pressed`, `focused`) | **In scope, via `WidgetExt` defaults.** Round-1 Q3 resolution: `on_mouse_press` → `set_pressed(true)`, `on_mouse_release` → `set_pressed(false)`, `on_focus_in` → `set_focused(true)`, `on_focus_out` → `set_focused(false)`. The deferred input-plumbing pass only routes events; flag mutation is automatic. Tests exercise both the direct `set_*` setters and the four updated defaults. |
| Event-driven mutation of the flag (`hovered`) | **Deferred to the input-plumbing pass.** `MouseEventKind` lacks `Enter` / `Leave` variants in v1, so there is no obvious hook to update from. The input-plumbing pass will synthesise enter/leave from `Move` + hit-testing and call `set_hovered` directly. |
| Where the new render path lives | Inside the existing `DefaultStyle::draw_button` in `quartzite-style/src/default_style.rs`. No new module, no new public API on `Style` or `DefaultStyle`. |
| Visual treatment — `hovered` | Round-1 Q1 resolution: fill alpha-blended **25% toward `Highlight`** over the base `Button` role fill (`mix = 0.25 * highlight + 0.75 * button` componentwise on `r`/`g`/`b`/`a`). The blend helper does NOT exist on `Color` today — the design phase will choose between adding a `Color::blend(other, t)` (or `lerp`) inherent method on `quartzite_paint_api::Color` versus a local `fn` inside `default_style.rs`; either is fine. Text role unchanged (`ButtonText`). |
| Visual treatment — `pressed` | Round-1 Q1 resolution: **role swap to `Highlight` / `HighlightedText`** — same fill + text mapping as `checked`. Distinguishable from `checked` only when the precedence rule applies (see below); identical visual otherwise. |
| Visual treatment — `focused` | Round-1 Q1 resolution: **outline pen width = 2 px in `Highlight` colour** (idle outline is 1 px in `Mid`/`Dark` per the default-style-content spec). Additive — applies regardless of the fill state. |
| State precedence | Round-1 Q2 resolution: `disabled` > `pressed` > `checked` > `hovered` for the **fill / text colour axis**. `disabled` paints at half-alpha and overrides every other state. `focused` is an **additive outline modifier** orthogonal to the fill state — i.e. a focused-and-disabled button still gets the 2 px `Highlight` outline (though the fill is half-alpha) and a focused-and-pressed button keeps the `Highlight` fill plus the 2 px outline. |
| Snapshot coverage | One additional golden per visible state (hover, pressed, focused) under the existing `shared/` flow. Disabled is already covered (AC11 of the snapshot-tests spec); `checked+disabled+hovered` combinations are NOT enumerated — only the single-state goldens. |
| Recording-painter tests for new states | Added alongside AC2 (button-fill) / AC7 (checked) / AC8 (disabled) in `quartzite-style/src/default_style.rs` — each new test toggles one flag and asserts the captured `Brush` colour or `Pen` width differs from the idle baseline. |
| No new public API beyond accessors | `DefaultStyle`'s public surface is unchanged. `Style` trait is unchanged. Only `WidgetBase` gains three `pub` fields and `WidgetExt` gains six default-impl methods. |
| `Send + Sync` impact | None. All three new fields are `bool`; `WidgetBase`'s `Send + Sync` story is unchanged. |

## Technical constraints

- `quartzite-widgets` already exports `WidgetBase` and `WidgetExt`; no new dependency edges.
- `quartzite-style` already depends on `quartzite-widgets` and uses `WidgetExt::is_enabled()` inside `draw_button` (see `default_style.rs:69`); reading the three new flags follows the same pattern via `WidgetExt::is_hovered` / `is_pressed` / `is_focused`.
- The existing `RecordingPainter` fixture (`default_style.rs` lines 174–279) is reusable as-is for the new unit tests.
- The existing snapshot harness (`quartzite-style/tests/snapshots.rs` + `tests/support/mod.rs`) is reusable — new tests follow the AC2 / AC11 pattern.
- Doc gate: every new public item carries `///` first-line docs + `# Examples` where applicable per `ai-docs/doc-convention.md`.
- Lint gate: `cargo clippy --workspace -- -D warnings` clean.
- Doc gate: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean.
- `WidgetExt` is a blanket-impl trait (`impl<T: AsWidget> WidgetExt for T {}`). New methods on the trait are inherited by every widget for free.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite_widgets::WidgetBase` carries three new `pub bool` fields named `hovered`, `pressed`, `focused`, each defaulting to `false` in `WidgetBase::new()` and `WidgetBase::default()`. A unit test in `quartzite-widgets/src/widget_base.rs` asserts the defaults. |
| AC2 | `quartzite_widgets::WidgetExt` provides default methods `is_hovered` / `set_hovered` / `is_pressed` / `set_pressed` / `is_focused` / `set_focused` mirroring the existing `is_enabled` / `set_enabled` shape. A unit test in `widget_ext.rs` flips each flag on a `WidgetBase` instance and asserts the getter reflects the change. |
| AC3 | `DefaultStyle::draw_widget(&btn_with_hovered, …)` records a `FillRect` whose `Brush` colour equals the 25 % blend `0.25 * Highlight + 0.75 * Button` (componentwise on `r`/`g`/`b`/`a`) under `Palette::default()`. The recording-painter test computes the expected blend with the same blend helper the production code uses (added by this task — see § Key decisions) and asserts equality plus a difference vs the idle baseline. |
| AC4 | `DefaultStyle::draw_widget(&btn_with_pressed, …)` records a `FillRect` whose `Brush` colour equals `palette.color(ColorRole::Highlight)` (same role swap as `checked`) and a text draw whose `Pen` colour equals `palette.color(ColorRole::HighlightedText)` under `Palette::default()`. The recording-painter test asserts both bindings and the idle-baseline difference. |
| AC5 | `DefaultStyle::draw_widget(&btn_with_focused, …)` records a `DrawRect` whose `Pen` width equals `2.0` and whose `Pen` colour equals `palette.color(ColorRole::Highlight)` under `Palette::default()` (idle outline is width `1.0` per the default-style-content spec). The recording-painter test asserts both Pen attributes change vs the idle baseline. |
| AC6 | State-precedence for the fill / text axis is `disabled` > `pressed` > `checked` > `hovered`, and `focused` is an additive outline modifier. Recording-painter tests exercise: (a) `(disabled, pressed)` → fill matches the disabled-button colour (half-alpha) and the `Pen` outline still goes 2 px `Highlight` if `focused` is also set; (b) `(checked, hovered)` → fill matches `Highlight` (checked wins over hover); (c) `(pressed, checked)` → fill matches `Highlight` (both happen to map to `Highlight`; assertion is on equality); (d) `(focused, hovered)` → fill matches the hovered blend (25% toward Highlight) AND outline is 2 px `Highlight`. |
| AC7 | `WidgetExt::on_mouse_press` default impl sets `WidgetBase::pressed = true`; `on_mouse_release` default impl sets `WidgetBase::pressed = false`; `on_focus_in` default impl sets `WidgetBase::focused = true`; `on_focus_out` default impl sets `WidgetBase::focused = false`. A unit test in `widget_ext.rs` instantiates a `WidgetBase`, calls each default in turn, and asserts the flag transitions. |
| AC8 | One snapshot test per visible state (`button_hovered.png`, `button_pressed.png`, `button_focused.png`) lives in `quartzite-style/tests/snapshots.rs` using the existing `shared/` flow, with goldens committed alongside the existing button golden. |
| AC9 | `cargo clippy --workspace -- -D warnings` clean. |
| AC10 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean. |
| AC11 | All existing `default_style.rs` tests (AC2–AC10 of the default-style-content spec) still pass — the new state flags default to `false`, so idle-button rendering produces the same event sequence as before this spec. The previously-no-op `on_mouse_press` / `on_mouse_release` / `on_focus_in` / `on_focus_out` defaults are not invoked in any existing test (verified by grep), so their behavioural change is observable only in the new AC7 tests. |

## Open questions

_None remain after round 1; all three design-affecting questions (visual mapping, state precedence, event-handler defaults) were resolved by the user — see § Key decisions._
