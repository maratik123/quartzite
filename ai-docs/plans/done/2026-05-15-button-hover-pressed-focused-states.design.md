# Design: Button hover / pressed / focused visual states

**Issue:** #316
**Spec:** [`2026-05-15-button-hover-pressed-focused-states.spec.md`](./2026-05-15-button-hover-pressed-focused-states.spec.md)
**Date:** 2026-05-15

## Approach

Three threads of work, each isolated to one file:

1. **State storage on `WidgetBase`** — three new `pub bool` fields (`hovered`, `pressed`, `focused`), all defaulting to `false` in both `new()` and the derived `Default`. Matches the existing `enabled` / `visible` pattern.
2. **Ergonomic surface on `WidgetExt`** — six new default-impl methods (`is_hovered`/`set_hovered`, `is_pressed`/`set_pressed`, `is_focused`/`set_focused`), all `#[inline]` one-liners (concrete default-impl bodies with no type params — `#[inline]` only, no `_Simple._` doc tag; mirrors existing `is_enabled`/`set_enabled`/`is_visible`/`set_visible` style). Then four existing event-handler defaults (`on_mouse_press`, `on_mouse_release`, `on_focus_in`, `on_focus_out`) get bodies that call the matching setter — previously no-op, so the behavioural change is observable only on the new tests (verified by spec AC11).
3. **Rendering on `DefaultStyle::draw_button`** — read the three flags via `WidgetExt::is_hovered`/`is_pressed`/`is_focused`, apply the precedence rule (`disabled > pressed > checked > hovered` on the fill/text axis; `focused` an additive outline modifier) to pick the fill/text roles and outline pen, then record the painter calls.

Visual model implemented inside `draw_button`:

| State | Fill role | Text role | Outline pen | Notes |
|---|---|---|---|---|
| idle | `Button` | `ButtonText` | width 1.0, `ButtonText` colour | current behaviour |
| `checked` | `Highlight` | `HighlightedText` | width 1.0, `HighlightedText` colour | current behaviour |
| `pressed` (no `checked`, no `disabled`) | `Highlight` | `HighlightedText` | width 1.0, `HighlightedText` colour | role swap matches `checked` |
| `hovered` only | 25 % blend `0.25 * Highlight + 0.75 * Button` componentwise | `ButtonText` | width 1.0, `ButtonText` colour | blend helper added |
| `disabled` | base role for the active fill state, alpha halved | base role, alpha halved | width 1.0, halved-alpha text colour | wins over every other fill state |
| `focused` (additive) | — | — | width **2.0**, `Highlight` colour | replaces the outline pen regardless of fill state |

Precedence is computed top-down: select a `(fill_role, text_role)` pair from the first matching state in order `pressed` → `checked` → `hovered` → idle, then apply the disabled-alpha halving when `!enabled` (disabled is an alpha modifier, not a role-selector), then pick the outline pen (`width=2, Highlight` — no alpha-halving — if `focused`, else width-1 in the text role colour, again alpha-halved when disabled).

### Where the blend helper lives

The spec's Q1 resolution requires a 25% colour blend that does not exist on `Color` today. Two options:

- **Option A — inherent `Color::blend(self, other, t)` on `quartzite_paint_api::Color`.** Reusable workspace-wide, `const fn`, simple componentwise lerp on `r`/`g`/`b`/`a`. One extra public method.
- **Option B — local `fn blend(a, b, t)` inside `quartzite-style/src/default_style.rs`.** No public-API change; isolated to this issue.

**Chosen: Option A.** Three reasons: (1) the operation is generic enough that future palette work / hover treatments on `Label`/`TextEdit` will want it; (2) `Color` is already the natural owner of channel-wise math (`with_alpha` lives there); (3) the AC3 test independently re-computes the expected blend — having both production and test use the same helper avoids drift. The added method is `const fn` and `#[inline]` — zero binary-size cost.

If design-review pushes back on Option A as YAGNI, falling back to Option B is a one-file local rewrite. The decomposition isolates the blend helper into its own task so the choice can flip cheaply.

### Why state mutation goes on `WidgetExt` defaults, not on `Button`

The spec is explicit: `pressed` / `focused` mutation lives in `WidgetExt` default-impl bodies so every widget gets the state-machine for free once the input-plumbing pass routes events. Putting it on `Button` alone would force every future widget that wants press/focus visuals to duplicate the four-line state machine. The `WidgetExt` defaults are inherited via blanket `impl<T: AsWidget> WidgetExt for T {}`, so the behaviour propagates automatically.

`hovered` mutation is intentionally **not** wired to a default — `MouseEventKind` has no `Enter`/`Leave` variants, so there is no obvious hook to mutate from in v1. The input-plumbing pass will synthesise enter/leave from `Move` + hit-testing and call `set_hovered` directly. Spec confirms this in § Out of scope.

### Why the four hooks' default-body change is safe

Spec AC11 guarantees the four hooks are not invoked anywhere in the test suite today — verified by grep across `quartzite-widgets/src/`, `quartzite-style/src/`, `quartzite-style/tests/`. The only call sites of `on_mouse_press`/`on_mouse_release` in the workspace are on the **`WindowRoot`** trait in `quartzite-renderer` (which is a separate trait with the same method names — not `WidgetExt`). `on_focus_in`/`on_focus_out` have no call sites at all. Therefore the behavioural change from "no-op" to "mutate the matching flag" is observable only via the new AC7 tests that call the defaults directly.

### Rejected alternatives

1. **New `ColorRole` variants (`Hover`, `Pressed`, `FocusRing`)** — Spec § Out of scope rules this out for v1. Would force every theme + `Palette::default` to seed three new slots; orthogonal to the visual-state work. Deferred.
2. **State storage on `Button` instead of `WidgetBase`** — Forces every future widget that wants hover/pressed/focused visuals to duplicate the fields. The issue body + source spec both pin storage on `WidgetBase`. Rejected.
3. **`is_pressed` etc. on `WidgetBase` (inherent methods) plus shim through `WidgetExt`** — Adds two parallel surfaces. `WidgetExt` already owns ergonomics (`is_enabled`/`set_enabled`); add the new methods there alone and let direct field access cover the low-level path. Rejected.
4. **Computing `(fill_role, text_role)` via a free helper rather than inline `let` chain in `draw_button`** — Adds a new private fn whose only caller is `draw_button`. The body is six lines of `if`/`else` priority — clear enough inline. Rejected.
5. **One unified `WidgetState` enum (`Idle | Hovered | Pressed | Checked | Disabled | Focused`)** — Loses the orthogonality of `disabled` and `focused` (both compose with every other state). The bool-bag is closer to how every other GUI toolkit models this and keeps each axis assignable independently. Rejected.
6. **Snapshot tests for every state *combination* (e.g. `button_pressed_focused.png`, `button_checked_disabled_hovered.png`)** — Combinatorial explosion. Spec § Key decisions explicitly limits snapshot coverage to one golden per single-state visible variant; recording-painter tests cover the combination axes. Rejected.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `Color::blend(self, other: Color, t: f32) -> Color` — `const fn` componentwise lerp `(1.0 - t) * self + t * other` on `r`/`g`/`b`/`a`, `#[inline]` (concrete inherent method — no type params, so `#[inline]` only, no `_Simple._` doc tag; mirrors `Color::with_alpha`). Includes `///` + `# Parameters` + `# Examples` per `ai-docs/doc-convention.md`. Add 3–4 unit tests (`t=0` → self, `t=1` → other, `t=0.25` → expected blend, alpha is blended too). | `quartzite-paint-api/src/color.rs` | — |
| 2 | Add three `pub bool` fields `hovered`, `pressed`, `focused` to `WidgetBase`. Initialise all three to `false` in `WidgetBase::new()`. (`Default for WidgetBase` already calls `new()`, so the derived defaults flow through.) Update the `new_widget_base_defaults` test in the existing `#[cfg(test)] mod tests` to also assert the three new fields are `false`. Update the struct docstring's `# Examples` block if needed (none of the existing examples depend on the new fields, so likely unchanged). | `quartzite-widgets/src/widget_base.rs` | — |
| 3 | Add six new `#[inline]` default methods to `WidgetExt` — `is_hovered`/`set_hovered`, `is_pressed`/`set_pressed`, `is_focused`/`set_focused` — each one-liner over `self.widget_base().<field>` / `self.widget_base_mut().<field> = value`. Each carries `///` first-line doc + `# Examples` per doc-convention. Group them under a new `// ── hovered/pressed/focused ──` section in the trait body, after the existing `// ── enabled ──` block. | `quartzite-widgets/src/widget_ext.rs` | 2 |
| 4 | Update the four event-handler default-impl bodies in `WidgetExt`: `on_mouse_press` → `self.set_pressed(true)`, `on_mouse_release` → `self.set_pressed(false)`, `on_focus_in` → `self.set_focused(true)`, `on_focus_out` → `self.set_focused(false)`. Update each docstring to describe the new default behaviour ("Default impl sets `WidgetBase::pressed = true`. Override to add widget-specific reaction."). | `quartzite-widgets/src/widget_ext.rs` | 3 |
| 5 | Add unit tests inside `widget_ext.rs`'s existing `#[cfg(test)] mod tests`: (a) AC2 — six tests `is_hovered_default_false` / `set_hovered_flips` (and the two siblings) that flip each new flag on a `WidgetBase` instance and assert `is_*` reflects the change; (b) AC7 — four tests `on_mouse_press_default_sets_pressed`, `on_mouse_release_default_clears_pressed`, `on_focus_in_default_sets_focused`, `on_focus_out_default_clears_focused`. Each instantiates a `WidgetBase`, calls the default, asserts `widget_base.pressed` / `widget_base.focused` flipped. Uses a no-op-`MouseEvent` fixture (any constructor; the body ignores the parameter). | `quartzite-widgets/src/widget_ext.rs` | 4 |
| 6 | Refactor `DefaultStyle::draw_button` to honour the precedence rule. The body computes (in order): (i) `let (fill_role, text_role) = …` based on `pressed > checked > hovered > idle` (disabled is an alpha modifier applied in step iii, not a role-selector); (ii) `let fill_color = …` — either `palette.color(fill_role)`, or the 25 % blend `palette.color(Button).blend(palette.color(Highlight), 0.25)` for the pure-hover branch; (iii) `let fill_color = maybe_disabled(fill_color, enabled);` (iv) `let text_color = maybe_disabled(palette.color(text_role), enabled);` (v) `let (outline_pen_color, outline_pen_width) = if focused { (palette.color(Highlight), 2.0) } else { (text_color, 1.0) };` (vi) record `fill_rect` → `draw_rect` → `draw_text_in` as today. Note: the `focused` outline pen colour is NOT alpha-halved when disabled — spec says the focus outline is the **additive** 2 px `Highlight` regardless of fill state; cf. AC6(a) which keeps the 2 px `Highlight` outline on a `disabled+focused+pressed` button. | `quartzite-style/src/default_style.rs` | 1, 3 |
| 7 | Add recording-painter unit tests to `default_style.rs`'s existing `#[cfg(test)] mod tests` block — one per AC: AC3 (hovered → 25 % blend fill, text unchanged, idle outline), AC4 (pressed → `Highlight` fill + `HighlightedText` text), AC5 (focused → 2 px `Highlight` outline, fill unchanged), AC6 four sub-cases (precedence). Each new test toggles the relevant flag on a `Button` via `WidgetExt::set_hovered`/`set_pressed`/`set_focused` (or direct `widget_base_mut().<field> = true` — both are valid; prefer the setter to also exercise AC2 indirectly) and asserts `brush_color(brush) == expected` plus a `!=` vs the idle baseline where relevant. Re-uses the existing `RecordingPainter`, `brush_color`, `first_fill`, `first_draw_text_in` helpers. | `quartzite-style/src/default_style.rs` | 6 |
| 8 | Add three new snapshot tests + commit goldens — `button_hovered_renders` / `button_pressed_renders` / `button_focused_renders` — mirroring the existing `button_idle_renders` pattern. Each constructs a `Button`, sets the appropriate flag via the `WidgetExt` setter, renders via the harness, and calls `snapshot_assert("button_<state>", &image)`. Goldens land in `quartzite-style/tests/snapshots/shared/` (per spec — single shared golden, no per-backend override needed at first). Generation procedure: run `QUARTZITE_REGENERATE_SNAPSHOTS=1 cargo test -p quartzite-style --test snapshots button_hovered_renders` on the developer's local backend, move the resulting `tests/snapshots/<backend>/button_hovered.png` into `tests/snapshots/shared/`, repeat for `_pressed` and `_focused`. | `quartzite-style/tests/snapshots.rs`, `quartzite-style/tests/snapshots/shared/button_hovered.png`, `quartzite-style/tests/snapshots/shared/button_pressed.png`, `quartzite-style/tests/snapshots/shared/button_focused.png` | 6 |

Eight tasks, one per logical step. Tasks 2, 3, 4, 5 chain sequentially inside `quartzite-widgets`; tasks 6, 7 chain inside `quartzite-style` and depend on both 1 (blend helper) and 3 (new accessors). Task 8 (snapshot goldens) depends on 6 and is independent of tasks 5 / 7. Tasks 1 and 2 are independent of each other and can run in parallel.

## Risks

- **`Color::blend` is new public API.** Cost is one extra public method on `quartzite_paint_api::Color`. *Mitigation:* `const fn` + `#[inline]` (concrete inherent method — no `_Simple._` doc tag per AGENTS.md AXIOM; mirrors `Color::with_alpha`) — zero binary-size impact; the API is the natural sibling of `with_alpha` and trivially documentable. AGENTS.md § *API Stability* explicitly endorses clean public-API additions pre-publish.
- **Behavioural change of the four `WidgetExt` hooks (no-op → flag-mutate) could surprise downstream widgets that override one of them.** Today no widget in the workspace overrides any of the four (verified by `grep -rn "fn on_mouse_press\|fn on_focus_in" quartzite-widgets/src/quartzite-style/`). When a future widget overrides one and **does not** call the default via `Self::set_pressed(self, …)`, the state flag will stop tracking — that's the documented Rust override semantics and is the desired behaviour (the widget can opt out by overriding). *Mitigation:* document in the four hooks' rustdoc that the default mutates `WidgetBase::<field>` and that overrides "should call `self.set_pressed(true)` / etc. if they want the default flag-mutation to still happen".
- **Idle outline colour today is `text_color` (`ButtonText`), not `Mid`/`Dark` as the spec describes.** Spec § Key decisions text says "idle outline is 1 px in `Mid`/`Dark` per the default-style-content spec" — but the actual idle outline in `default_style.rs:81-85` uses `text_color`, and `ColorRole::Mid`/`Dark` do **not** exist in `quartzite-style-types::ColorRole`. This is a documentation drift in the spec, not a code defect; the AC5 test compares the focused outline pen to `Highlight` at width 2.0, and the idle-baseline outline is whatever `draw_button` currently writes (`text_color` at width 1.0). The design preserves current idle behaviour. *Mitigation:* AC5 test wording compares pen attributes against the *idle baseline* (`!=`), not against a literal `Mid`/`Dark` colour — so the test holds. Flagged as an open question for the design-review pass — if the user wants the idle outline changed to a separate role, that's a follow-up beyond this issue.
- **Snapshot tests are GPU-bound and skip when no adapter is available.** Existing `harness_or_skip` already covers this — no new test infrastructure needed. CI runs the snapshot tests on a `vulkan` runner; the `shared/` goldens are produced by the developer once and committed.
- **Snapshot goldens may need per-backend overrides if the renderer drifts.** Spec says single shared golden per state. *Mitigation:* the `support/mod.rs` lookup is "backend override → shared → fail" — if a future backend diverges, the override can be added without rewriting the test.
- **`# Examples` doc gate for the six new accessors.** Each must include a compiling doctest per `ai-docs/doc-convention.md`. *Mitigation:* mirror the existing `is_enabled` / `set_enabled` doctest shape — three-line examples that construct a `WidgetBase`, call the setter, and assert the getter.
- **Re-using the existing `disabled()` / `maybe_disabled()` helpers in `draw_button`.** The disabled-alpha halving applies to the fill role chosen by the precedence rule. The chosen-fill flow goes: pick role → resolve to `Color` → optionally blend (hover branch) → `maybe_disabled` → record. For the disabled-pressed combination, the `pressed` fill role (`Highlight`) is resolved, then halved — consistent with AC6(a). *Mitigation:* the existing `maybe_disabled` helper is reused as-is.
- **No panic / unsafe surface.** Every new code path is straight-line `bool`/`Color` math returning `()`. No `unwrap()`, no `unsafe` block, no `unreachable!`.
- **No `Send + Sync` impact.** All three new fields are `bool` — `WidgetBase`'s auto-derived `Send + Sync` is unchanged. The existing `default_style_is_send_sync` test continues to cover this.

## Test Design

### Task 1 — `Color::blend`

- **Location:** `quartzite-paint-api/src/color.rs` — existing `#[cfg(test)] mod tests`.
- **Entry point:** `Color::blend(self, other, t)`.
- **Scenarios:**
  - `blend_at_zero_returns_self`: `Color::RED.blend(Color::BLUE, 0.0) == Color::RED`.
  - `blend_at_one_returns_other`: `Color::RED.blend(Color::BLUE, 1.0) == Color::BLUE`.
  - `blend_at_quarter_lerps_componentwise`: `Color::new(1.0, 0.0, 0.0, 1.0).blend(Color::new(0.0, 0.0, 1.0, 1.0), 0.25)` returns `Color::new(0.75, 0.0, 0.25, 1.0)`.
  - `blend_lerps_alpha_too`: source `a=1.0`, target `a=0.0`, `t=0.5` → `a == 0.5`.
  - `blend_is_const_fn`: `const _: Color = Color::RED.blend(Color::BLUE, 0.5);` proves `const`-eval.
- **Fixtures / helpers:** none — pure value math.

### Task 2 — `WidgetBase` field defaults

- **Location:** `quartzite-widgets/src/widget_base.rs` — existing `#[cfg(test)] mod tests`.
- **Entry point:** `WidgetBase::new()`, `WidgetBase::default()`.
- **Scenarios:**
  - `new_widget_base_defaults` (extend existing test): also assert `!w.hovered && !w.pressed && !w.focused`.
  - `default_widget_base_matches_new`: optional belt-and-braces — `let w = WidgetBase::default(); assert!(!w.hovered && !w.pressed && !w.focused);`.
- **Fixtures / helpers:** none.

### Task 3 + 5 — `WidgetExt` accessors and event-handler defaults (AC2, AC7)

- **Location:** `quartzite-widgets/src/widget_ext.rs` — existing `#[cfg(test)] mod tests`.
- **Entry points:** `WidgetExt::is_hovered` / `set_hovered`, `is_pressed` / `set_pressed`, `is_focused` / `set_focused`, `on_mouse_press`, `on_mouse_release`, `on_focus_in`, `on_focus_out`.
- **Scenarios (AC2 — six tests, one per setter/getter pair):**
  - `is_hovered_default_false`, `set_hovered_flips`: `let mut w = WidgetBase::new(); assert!(!w.is_hovered()); w.set_hovered(true); assert!(w.is_hovered());` and the same shape for `pressed` / `focused`.
- **Scenarios (AC7 — four tests):**
  - `on_mouse_press_default_sets_pressed`: build a `MouseEvent` via `MouseEvent::new(Point::default(), Point::default(), MouseButtons::empty(), MouseButtons::empty(), Default::default(), MouseEventKind::Press)`, call `w.on_mouse_press(&e)`, assert `w.widget_base().pressed`.
  - `on_mouse_release_default_clears_pressed`: pre-set `w.set_pressed(true)`, call `w.on_mouse_release(&e)`, assert `!w.widget_base().pressed`.
  - `on_focus_in_default_sets_focused`: `w.on_focus_in()`, assert `w.widget_base().focused`.
  - `on_focus_out_default_clears_focused`: pre-set `w.set_focused(true)`, call `w.on_focus_out()`, assert `!w.widget_base().focused`.
- **Fixtures / helpers:** a one-line `fn fake_mouse_event(kind: MouseEventKind) -> MouseEvent` helper local to the test module (or inline construction at each call site — preference is up to the implementer, both are clean).

### Task 6 + 7 — `DefaultStyle::draw_button` visual-state coverage (AC3, AC4, AC5, AC6)

All tests live in `quartzite-style/src/default_style.rs`'s existing `#[cfg(test)] mod tests`, reusing the in-module `RecordingPainter`, `brush_color`, `first_fill`, `first_draw_text_in` helpers. Standard fixture pattern:

```text
let palette = Palette::default()
    .with_role(ColorRole::Highlight, Color::SKY_BLUE)
    .with_role(ColorRole::Button, Color::WHITE);   // for AC3 — pin Button so the blend has two distinct endpoints
let mut btn = Button::new("x".into());
btn.set_<state>(true);
let mut painter = RecordingPainter::default();
DefaultStyle.draw_widget(&btn, &mut painter, &palette);
```

- **AC3 — `hovered_button_uses_blended_fill`.** Expected blend = `Color::WHITE.blend(Color::SKY_BLUE, 0.25)` = `Color::new(0.75, 0.875, 1.0, 1.0)` (formula: `r = 0.75*1 + 0.25*0 = 0.75`, `g = 0.75*1 + 0.25*0.5 = 0.875`, `b = 0.75*1 + 0.25*1 = 1.0`, `a = 1.0`). Assert `brush_color(first_fill(&painter.events)) == expected_blend` and `!= palette.color(ColorRole::Button)` (idle baseline) and `text_color == ColorRole::ButtonText`. The test computes `expected_blend` via the same `Color::blend` the production code uses — this is the canonical guard against drift between production and test.
- **AC4 — `pressed_button_uses_highlight_roles`.** Pressed-only button (`!checked`, `!hovered`, `!focused`, enabled). Assert `brush_color(first_fill(…)) == palette.color(ColorRole::Highlight)` AND `brush_color(first_draw_text_in(…)).text_color == palette.color(ColorRole::HighlightedText)`. Also assert `!=` vs the idle-baseline fill (`palette.color(ColorRole::Button)`).
- **AC5 — `focused_button_uses_2px_highlight_outline`.** Focused-only button, otherwise idle. Inspect the `DrawRect` event: assert `pen.width() == 2.0` AND `pen.color() == palette.color(ColorRole::Highlight)`. Assert `!=` idle-baseline `Pen` (width 1.0, `ButtonText` colour).
- **AC6 — four sub-cases, each its own test:**
  - `precedence_disabled_pressed_focused`: set `disabled`, `pressed`, `focused`. Assert the fill colour's `r/g/b` equals `palette.color(ColorRole::Highlight)`'s `r/g/b` (since pressed selects `Highlight`) and `a == palette.color(ColorRole::Highlight).a() * 0.5` (disabled halves alpha). Also assert outline pen is width 2.0 in `palette.color(ColorRole::Highlight)` colour — additive focus modifier survives disabled.
  - `precedence_checked_hovered_keeps_checked_fill`: set `checked`, `hovered`. Assert fill == `palette.color(ColorRole::Highlight)` (checked wins over hover for the fill axis).
  - `precedence_pressed_checked_both_map_to_highlight`: set `pressed`, `checked`. Assert fill == `palette.color(ColorRole::Highlight)` (both happen to map to the same role; equality is the assertion).
  - `precedence_focused_hovered_blend_plus_outline`: set `focused`, `hovered`. Assert fill == `Color::WHITE.blend(Color::SKY_BLUE, 0.25)` AND outline pen is width 2.0 in `Highlight`.
- **AC11 — `idle_button_unchanged_after_new_flags`.** Either a fresh regression test, or rely on the existing `button_records_fill_outline_and_centred_text` / `checked_button_uses_highlight_colour` / `disabled_button_halves_fill_and_text_alpha` passing as-is after the refactor. Recommended: keep all three existing tests intact and add one explicit `idle_button_three_events_unchanged` test that constructs `Button::new("OK".into())` and asserts the recorded event sequence is exactly the three events the current spec records, with the same brush/pen colours.

### Task 8 — snapshot goldens (AC8)

- **Location:** `quartzite-style/tests/snapshots.rs`.
- **Entry point:** `harness.render_widget(|painter| DefaultStyle::default().draw_widget(&w, painter, &Palette::default()))`.
- **Scenarios:** three new `#[test]` fns mirroring `button_idle_renders`:
  - `button_hovered_renders`: `w.set_hovered(true)`, golden name `button_hovered`.
  - `button_pressed_renders`: `w.set_pressed(true)`, golden name `button_pressed`.
  - `button_focused_renders`: `w.set_focused(true)`, golden name `button_focused`.
- **Fixtures / helpers:** existing `support::harness_or_skip` and `support::snapshot_assert`. No new helpers.
- **Golden generation:** documented in the decomposition row for task 8. Reviewer should verify the three new PNGs land in `tests/snapshots/shared/` (not in `tests/snapshots/<backend>/`).

### Gate coverage

- **AC9 (`cargo clippy --workspace -- -D warnings`).** No new `#[allow(...)]` should be introduced. The `_Simple._` doc tag and `#[inline]` markers on the six accessors keep clippy quiet. Verified locally before commit per AGENTS.md.
- **AC10 (rustdoc gate).** All new public surface — `Color::blend`, the three new `WidgetBase` fields, the six new `WidgetExt` methods, and the four updated `WidgetExt` hook docstrings — carry `///` first-line docs and `# Examples` per `ai-docs/doc-convention.md`. The hook docstrings update prose only; their existing `# Parameters` blocks stay intact.

## Open questions

_None remain after design-review round 1._

**Resolved decisions (previously open questions):**

- **Idle-outline role (spec drift).** Spec § Key decisions describes the idle outline as "1 px in `Mid`/`Dark`", but `ColorRole::Mid`/`Dark` do not exist and the current code uses `text_color` (`ColorRole::ButtonText`). **Decision: preserve current behaviour** — idle outline = `text_color` at width 1.0. Switching to a different colour role is a follow-up beyond this issue. AC5 / AC6 tests assert against this baseline (`!=` idle) without referencing the non-existent roles.
- **Override-contract note in the four `WidgetExt` hook docstrings.** **Decision: include the note.** Each hook's rustdoc will contain one line noting that default impl mutates `WidgetBase::<field>` and that overrides "should call `self.set_<flag>(true/false)` from the override body if they want the default flag-mutation to still happen." Silent default-impl mutation is a subtle contract worth documenting.
