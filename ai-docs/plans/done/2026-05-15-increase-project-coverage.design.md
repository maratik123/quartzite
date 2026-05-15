# Design: Increase project coverage

**Issue:** #341
**Date:** 2026-05-15
**Spec:** [`ai-docs/plans/2026-05-15-increase-project-coverage.spec.md`](./2026-05-15-increase-project-coverage.spec.md)

## Approach

The spec splits into two largely-independent work streams:

1. **Coverage uplift** — pick the largest production-code uncovered regions, add deterministic unit / integration tests that meaningfully exercise them. The current workspace coverage at HEAD `5eca289` is **90.42 % lines / 92.36 % regions** (see [§ Top-10 snapshot](#top-10-uncovered-production-code-regions-snapshot)); we need ≥ **93.00 %** project coverage on the merge commit (AC1). The 90 → 93 jump is ~ 270 lines, which is well inside reach if we cover the 5–6 largest gaps (e.g. `event_convert.rs` alone is 126 missed lines of pure-function code).
2. **`vello_painter.rs` catch-all restructure** (AC4) — replace the two `_ => None` arms in `brush_to_peniko` and `brush_color` with an exhaustive match plus a single explicit typed funnel for the `#[non_exhaustive]` upstream-extension case.

### Q1 catch-all restructure — chosen shape

**Decision:** introduce a private, renderer-internal closed enum `LocalBrushKind<'a>` colocated with `vello_painter.rs`, together with a single classifier `LocalBrushKind::from_brush_kind(&BrushKind) -> Self` whose body is the only place a `_` wildcard over `BrushKind` lives. Both `brush_to_peniko` and `brush_color` then match exhaustively on `LocalBrushKind`.

```rust
// Private renderer-internal classification of BrushKind into the four variants the
// renderer knows how to handle today, plus an explicit Unknown bucket that funnels
// every future upstream variant. Mirrors quartzite_paint_api::BrushKind 1:1 by
// reference; this isolates the single `_ => Unknown` wildcard to one location so
// neither `brush_to_peniko` nor `brush_color` needs a catch-all arm.
enum LocalBrushKind<'a> {
    Solid(&'a Color),
    LinearGradient { start: &'a Point, end: &'a Point, start_color: &'a Color, end_color: &'a Color },
    RadialGradient { centre: &'a Point, radius: f32, start_color: &'a Color, end_color: &'a Color },
    Custom(&'a peniko::Gradient),
    /// Forward-compat sink for any future BrushKind variant added in
    /// `quartzite-paint-api` (the upstream type is `#[non_exhaustive]`).
    /// The renderer falls back to "no brush" semantics, matching today's
    /// `_ => None` behaviour. See `quartzite_paint_api::BrushKind`.
    Unknown,
}

impl<'a> LocalBrushKind<'a> {
    fn from_brush_kind(k: &'a BrushKind) -> Self {
        match k {
            BrushKind::Solid(c) => Self::Solid(c),
            BrushKind::LinearGradient { start, end, start_color, end_color } =>
                Self::LinearGradient { start, end, start_color, end_color },
            BrushKind::RadialGradient { centre, radius, start_color, end_color } =>
                Self::RadialGradient { centre, radius: *radius, start_color, end_color },
            BrushKind::Custom(g) => Self::Custom(g),
            // Upstream `BrushKind` is `#[non_exhaustive]` — keep the sink here so the
            // exhaustive matches in `brush_to_peniko` / `brush_color` never need `_`.
            _ => Self::Unknown,
        }
    }
}
```

Then `brush_to_peniko` / `brush_color` match exhaustively over `LocalBrushKind` (no `_`):

```rust
fn brush_to_peniko(&self, brush: &Brush) -> Option<peniko::Brush> {
    match LocalBrushKind::from_brush_kind(brush.kind()) {
        LocalBrushKind::Solid(c) => Some(peniko::Brush::Solid(Self::color_to_peniko(*c))),
        LocalBrushKind::LinearGradient { start, end, start_color, end_color } => { ... }
        LocalBrushKind::RadialGradient { centre, radius, start_color, end_color } => { ... }
        LocalBrushKind::Custom(g) => Some(peniko::Brush::Gradient(g.clone())),
        LocalBrushKind::Unknown => None,
    }
}

fn brush_color(brush: &Brush) -> Option<peniko::Color> {
    match LocalBrushKind::from_brush_kind(brush.kind()) {
        LocalBrushKind::Solid(c) => Some(Self::color_to_peniko(*c)),
        LocalBrushKind::LinearGradient { .. }
        | LocalBrushKind::RadialGradient { .. }
        | LocalBrushKind::Custom(_)
        | LocalBrushKind::Unknown => None,
    }
}
```

**Why this shape (alternatives rejected):**

| Alternative | Why rejected |
|---|---|
| `From<&BrushKind> for LocalBrushKind<'_>` impl | Equivalent behaviour but a free associated fn `from_brush_kind` is more discoverable in a single-call-site adapter — `From` adds a trait jump for grep/rustdoc without buying ergonomics. The spec leaves the choice to the designer (Q1: "free function vs `impl` method vs `From` impl"). |
| Repeated exhaustive `match` over `BrushKind` directly in both call sites | Both call sites would still need a `_ => …` arm (BrushKind is `#[non_exhaustive]`) — fails AC4. |
| Helper fn `is_unknown_brush(&BrushKind) -> bool` + early-return | Forces a duplicated 4-arm match per call site after the early return. More code, same number of wildcard sites. |
| `unreachable!()` in the `_` arm | Would `panic!` if the upstream crate adds a variant in a downstream rebuild — violates library safety idioms (the renderer must degrade gracefully, not crash). |
| Strip `#[non_exhaustive]` from `BrushKind` | Out of scope per spec (`## Out of scope` line 21). |

The classifier lives as a free `impl LocalBrushKind` block at module scope between `color_to_dynamic` and the new `brush_to_peniko`. The `Unknown` arm has a `/// Forward-compat sink…` rustdoc comment naming the upstream source (`quartzite_paint_api::BrushKind`) and the rationale, per AC4.

### Coverage uplift — chosen targets

Five targets are easy wins (pure functions / simple branches / Debug impls): `event_convert.rs`, `value.rs`, `meta.rs`, `rect.rs::RectF::*`, `render_harness.rs` accessor accessors + Debug. These alone close ~ 230 missed lines. Three more (`object/parse.rs` macro error paths, `wrapped_handler.rs` mouse/key dispatch, `vello_painter.rs` italic / underline / strikethrough — gated on the small Font-builder addition described in [§ R8 prerequisite: Font builder setters](#r8-prerequisite-font-builder-setters)) are moderately tractable. The remaining two (`connect.rs` queued/auto arms, `timer.rs` connect_tick_queued/auto) are partially testable via Direct-only counterparts; their concurrency races are listed in [§ Deferred regions](#deferred-regions).

### R8 prerequisite: Font builder setters

`quartzite-paint-api/src/font.rs` exposes only `Font::new(family, size_pt)` and read-only accessors (`italic()`, `underline()`, `strikethrough()`, `weight()`); the three style flags and `weight` are private with no public setters. The R8 coverage branches at `vello_painter.rs` L260 (italic), L308–314 (underline), L316–322 (strikethrough) cannot be reached from a test without a way to flip those flags.

**Decision:** add three small `pub fn with_italic(self, v: bool) -> Self`, `pub fn with_underline(self, v: bool) -> Self`, `pub fn with_strikethrough(self, v: bool) -> Self` consuming-builder setters to `Font` in `quartzite-paint-api/src/font.rs`. Each:

- takes `self` by value and returns `Self` (consistent with idiomatic Rust consuming-builder pattern; `Font` is already `Clone`),
- carries a one-line `///` doc plus a `# Examples` block per AGENTS.md § *Documentation*,
- is `#[inline]` (concrete method on a concrete struct, one assignment, no branches — `Simple`-shaped),
- defaults the flag value through the caller (no implicit "set to true" — explicit `bool` keeps the API symmetric for `with_x(false)` resets).

This is a public-API addition, not a break (no existing surface changes; new fns are purely additive). The clean-breaks rule in AGENTS.md § *API Stability* allows free additions pre-publish.

**Rejected alternatives:**

| Alternative | Why rejected |
|---|---|
| `Font { italic: true, ..Font::new(…) }` literal in test code | `italic` / `underline` / `strikethrough` are private to `quartzite-paint-api` — not accessible from `quartzite-renderer`'s test mod. |
| `#[cfg(test)] pub(crate) fn with_italic_for_test(…)` | Lives in the wrong crate (`quartzite-paint-api`) and would not be visible across the crate boundary to `quartzite-renderer`'s `#[cfg(test)]` module. `pub` is required. |
| Drop R8 from the covered set entirely (option (b) in design-review) | Loses ~ 14 covered lines and forces the coverage budget to redistribute onto smaller / less deterministic regions. The Font setter additions are ~ 30 lines of trivial public-API code that themselves contribute new doctest coverage. Strict economy win for option (a). |
| Add only `with_italic` and defer underline / strikethrough | Asymmetric (`italic` / `underline` / `strikethrough` are sibling flags on the same struct with identical shape); deferring two of three would force partial R8 with no payoff. |
| Mutator setters (`set_italic(&mut self, v: bool)`) instead of consuming builders | Existing `Font::new(…)` returns `Self`; chained construction (`Font::new("Arial", 12.0).with_italic(true).with_underline(true)`) reads naturally and matches idiomatic Rust builder style. The `Font::default()` test in the same file already uses chained-style construction implicitly. |

## Top-10 uncovered production-code regions (snapshot)

**Workspace HEAD when snapshot taken:** `5eca28920e7ce674565b51d76da63595fc86aa1c` (master, 2026-05-15).
**Total at snapshot:** 17 059 regions / 10 242 lines — coverage **92.36 % regions** / **90.42 % lines**.
**Generated by:** `xvfb-run -a cargo +nightly llvm-cov --workspace --doctests --lcov --output-path /tmp/cov.lcov` followed by per-file post-processing that strips lines at or after the first `#[cfg(test)]` boundary (per Q4: "Code outside `#[cfg(test)]` blocks"). Non-exhaustive `_ => …` arms (Q1 scope item 1) are flagged in the table footnotes; they are excluded from per-region "feasible target lines" counts.

The 90.42 → 93.00 % project target is **+264 lines** (rounded). Closing R1–R5 alone (~ 230 lines) plus partial R6–R10 gets there with margin.

| # | File | Missed lines (prod) | Uncovered ranges (line:count) | Function(s) / nature | Deterministic? |
|---|------|---------------------|-------------------------------|----------------------|----------------|
| R1 | `quartzite-renderer/src/event_convert.rs` | 126 | 81–120(40), 122–150(29), 157–173(17), 178–190(13), 55–63(9), 65–73(9), 35–37(3), 43–51(9) | `size_from_physical`, `mouse_button_from_winit`, `mouse_event_from_winit`, `key_from_winit` (huge `match s.as_str()` + `match named`), `modifiers_from_winit`, `key_event_from_winit` | Yes — pure functions over public winit types |
| R2 | `quartzite-core/src/connect.rs` | 51 | 336–346(11), 355–363(9), 169–176(8), 310–314(5), 296–299(4), 348–351(4), 165–178(14 in `_signal_to_signal` Queued arm), 180–198(in Auto arm) | `connect_signal_to_signal` Queued/Auto closure paths (lines 165–198); `connect_signals` arity/type mismatch error arms (293–315); `connect_signals` Queued (335–353) and Auto (354–368) arms | Partial — Direct + error arms fully testable; Queued/Auto need queued-dispatcher fixture (deferred) |
| R3 | `quartzite-core/src/meta.rs` | 45 | 283–293(11), 617–625(9), 629–635(7), 400–405(6), 196–198(3), 238–240(3), and smaller spots | `MethodMeta::new`, `MetaObject::fmt` (`Debug`), `MetaObject::eq` (`PartialEq`), `EnumMeta::fmt`, `EnumMeta::eq`, `noop_lookup_*` placeholders, accessors | Yes — trivial constructor + Debug/PartialEq tests |
| R4 | `quartzite-renderer/src/wrapped_handler.rs` | 35 | 169–177(9), 179–184(6), 205–208(4), 186–188(3), 197–199(3), 210–212(3), 215(1) and assorted | `WrappedHandler::dispatch_window_event_inner` arms for `MouseInput { Pressed / Released }`, `CursorMoved`, `ModifiersChanged`, `KeyboardInput { Pressed / Released }` | Yes — the existing `dispatch_window_event_inner` test entry point (line 130) already lets tests inject `WindowEvent`s with a `fake_id` |
| R5 | `quartzite-core/src/value.rs` | 32 | 398–404(7), 216–219(4), 383–386(4), 437–440(4), 112–114(3), 184–186(3), 211–222(11 in `Value::type_name`) | `FromValue::from_value` error arms for `f64`, `f32`, `bool`, `String`, etc. (passing the wrong `Value` variant); `Clone for Box<dyn CustomValue>`; `Value::type_name` arms not hit by existing tests | Yes — pure functions, type-mismatch construction is trivial |
| R6 | `quartzite-macros/src/object/parse.rs` | 23 | 43–46(4), 50–53(4), 58–61(4), 161–164(4), 142–144(3), 146(1), 49–55(7) | `parse()` error paths: non-named-field struct, non-struct (enum/union), generic struct, `#[prop]` name-value rejection, unknown prop option | Yes — the existing `parse_err` test helper accepts a `quote!` `TokenStream` |
| R7 | `quartzite-geometry/src/rect.rs` | 17 | 465–472(8), 137–139(3), 491–493(3), 443–444(2), 474(1) | `RectF::united`, `RectF::translated`, `RectF::intersects` (false branches), `Rect::is_empty` zero-size paths | Yes — pure const fns |
| R8 | `quartzite-renderer/src/vello_painter.rs` | 17[^1] | 309–314(6), 317–322(6), 205(1), 212(1), 247(1), 261(1) | `_ => None` catch-all arms at L205 / L212 (eliminated by AC4 restructure); `emit_layout_glyphs` underline branch (309–314) and strikethrough branch (317–322); italic-font branch in `push_font_style` (261); `Segment::_ => {}` non-exhaustive wildcard at 247 (excluded from ranking) | Yes — call `VelloPainter::draw_text` with `Font` constructed via `.with_underline(true)` / `.with_strikethrough(true)` / `.with_italic(true)` and a non-empty `FontCache` |
| R9 | `quartzite-runtime/src/timer.rs` | 17 | 312–319(8), 321–324(4), 471–473(3), 500(1), 517(1), 753–754(2) | `Timer::connect_tick_queued` (signature + body 312–324); `Timer::signals_blocked` accessor (471–473); `start()` early-return when already running (500); callback `running=false` race (517) | Partial — `signals_blocked` and `connect_tick_queued` smoke (Direct delivery via a queued slot) are testable; multi-thread races deferred |
| R10 | `quartzite-renderer/src/render_harness.rs` | 16 | 236–242(7 in `Debug::fmt`), 258–260(3 in `width`), 274–276(3 in `height`), 293–295(3 in `scale_factor`) | `RenderHarness::fmt` (`Debug`), `RenderHarness::width/height/scale_factor` (`const fn` not hit at runtime; only by `no_run` doctests) | Yes — trivial unit tests; doctests are `no_run` so accessor bodies need direct tests |

**Missed-line totals across R1–R10:** 126 + 51 + 45 + 35 + 32 + 23 + 17 + 17 + 17 + 16 = **379 lines** in production-only ranking. The 93 % AC needs **≥ 264** newly-covered lines (264 / 10 242 ≈ 2.58 pp uplift).

**Planned coverage budget** (per-region newly-covered lines, after option (a) — Font setters land so R8's underline + strikethrough + italic branches are reachable from tests):

| Region | Total missed (snapshot) | Plan covers (estimated) | Remaining missed | Rationale for the estimate |
|---|---|---|---|---|
| R1 — `event_convert.rs` | 126 | ~ 120 | ~ 6 | `rstest` table covers every mapped branch; the small remainder is the `_ => None` fall-through arm at L152 (`#[non_exhaustive]`-shaped, excluded from ranking — not counted in the "cover" column) plus 1–2 unmapped NamedKey arms not reachable through public winit constructors. |
| R3 — `meta.rs` | 45 | ~ 40 | ~ 5 | Debug/PartialEq/`MethodMeta::new`/`noop_lookup_*` are all trivially testable. The ~ 5 line remainder is in EnumMeta field accessors only hit by larger consumers. |
| R5 — `value.rs` | 32 | ~ 28 | ~ 4 | One error-arm test per `FromValue` impl + `Clone for Box<dyn CustomValue>` + `Value::type_name` arms covers the bulk. ~ 4 lines remain in conversion edge paths (e.g. `i64 → u32` overflow checked at runtime by `try_into` whose specific Err formatting isn't asserted). |
| R7 — `rect.rs` | 17 | 17 | 0 | All four entry points are pure `const fn`s; tests cover every branch end-to-end. |
| R8 — `vello_painter.rs` | 17 | ~ 14 | ~ 3 | AC4 refactor eliminates the two `_ => None` arms (L205, L212 — 2 lines). italic (L260) + underline (L308–314 = 6 lines) + strikethrough (L316–322 = 6 lines) covered via task 7's Font setters. The remaining ~ 3 lines are inside the new `LocalBrushKind::Unknown` arm + `_ => Self::Unknown` (deferred per § *Deferred regions*) and the `Segment::_ => {}` wildcard at L247 (also deferred). |
| R10 — `render_harness.rs` | 16 | ~ 13 | ~ 3 | Width/height/scale_factor/Debug accessor tests cover the four entry points. The ~ 3 line remainder is in failure paths inside `RenderHarnessBuilder::build()` that require GPU absence to exercise. |
| R6 — `object/parse.rs` | 23 | ~ 23 | 0 | `parse_err` is fully driven by `quote!` token streams; every error path is reachable. |
| R4 — `wrapped_handler.rs` | 35 | ~ 25 | ~ 10 | MouseInput / CursorMoved / ModifiersChanged / KeyboardInput happy paths covered via `dispatch_window_event_inner`. ~ 10 line remainder is in less-trivial state transitions (focus tracking, modifier bitmask serialisation) that need richer fixtures. |
| R2 — `connect.rs` (partial) | 51 | ~ 12 | ~ 39 | Direct-path error arms only. Queued/Auto closure bodies + cross-thread Auto deferred — see § *Deferred regions* (~ 30 lines of the 39 sit there). |
| R9 — `timer.rs` (partial) | 17 | ~ 5 | ~ 12 | `signals_blocked` + `connect_tick_queued` smoke + `start()` idempotency cover ~ 5 lines. Multi-thread callback races deferred. |
| Task 7 doctest gain (Font setters) | — (new code) | + 3 | — | Each new `with_*` builder ships with a `# Examples` doctest (per AGENTS.md § *Documentation*); doctests count under Q5. The three doctests themselves add ~ 3 covered lines on top of the table above. |
| **Subtotal** | **379** | **~ 300** | **~ 82** | |

**Headroom against the 93 % AC bar:** plan delivers ~ 300 newly-covered lines vs the required ~ 264 → **margin ≈ 36 lines ≈ 0.35 pp** above 93.00 %. The headroom is honest but tight; task 13's verification step exists specifically to confirm the run lands above 93.00 % and to backstop with cheap accessor tests in `keyboard.rs` / `mouse.rs` / `signal.rs` / `event_loop.rs` / `box_layout.rs` if the actual measurement falls short.

[^1]: After the AC4 restructure, line 205 (and 212) cease to be coverage regions because the `_` wildcard arms are removed in favour of an exhaustive match on `LocalBrushKind`. The single remaining `_ => Unknown` arm lives inside `LocalBrushKind::from_brush_kind` and is itself excluded from the ranking per spec scope item 1 (it's the `#[non_exhaustive]` upstream-extension catch-all).

## Decomposition

Subtask granularity: each row = one focused PR commit. Ordered so tests land before production refactor when both touch the same file (TDD per AGENTS.md § *Workflow*). All thirteen subtasks live in the same PR (single issue #341); the upper bound of 7 subtasks per the design agent rules is exceeded here because the work is fundamentally a batched test-addition pass — splitting into multiple issues would just multiply CI cost without changing the actual diff. See [§ Open questions](#open-questions) for the rationale.

**Task → region map** (`task N covers region Rk`, ordered by task #):

| Task | Region | Task | Region |
|---|---|---|---|
| 1 | R1 (`event_convert.rs`) | 8 | R8 production refactor (`vello_painter.rs`) |
| 2 | R3 (`meta.rs`) | 9 | R8 tests (`vello_painter.rs`) |
| 3 | R5 (`value.rs`) | 10 | R4 (`wrapped_handler.rs`) |
| 4 | R7 (`rect.rs`) | 11 | R2 partial (`connect.rs`) |
| 5 | R10 (`render_harness.rs`) | 12 | R9 partial (`timer.rs`) |
| 6 | R6 (`object/parse.rs`) | 13 | (verification / non-coding — no region) |
| 7 | R8 prerequisite — Font setters (`font.rs`) | | |

Task 7 (Font builder setters) is a public-API enabler for R8 (tasks 8 + 9). Task 13 (verification + optional AC6 follow-up) is a non-coding step kept in the table for ordering but excluded from the 13-coding-task count where relevant.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **R1 tests** — add a parameterised `rstest` for `key_from_winit` covering Character A-Z / 0-9 / unmapped, Named keys Enter/Escape/Backspace/Tab/Space/Delete/Insert/Home/End/PageUp/PageDown/Arrow{Left,Right,Up,Down}/F1–F12, and an unmapped NamedKey + non-Character / non-Named branch returning `None`. Add tests for `size_from_physical` (normal, `u32::MAX` saturation, zero), `mouse_button_from_winit` (5 buttons + `Other(_)`), `mouse_event_from_winit` (Pressed/Released), `modifiers_from_winit` (each modifier alone + combined), `key_event_from_winit` (mapped + unmapped keys). | `quartzite-renderer/src/event_convert.rs` `#[cfg(test)]` module (already exists at line 195) | — |
| 2 | **R3 tests** — `MetaObject` / `EnumMeta` `Debug` format, `PartialEq` reflexive/symmetric/inequality, `MethodMeta::new` accessor coverage, `noop_lookup_entry_by_name` / `_by_value` smoke. | `quartzite-core/src/meta.rs` new `#[cfg(test)] mod tests` block (or extend if one exists) | — |
| 3 | **R5 tests** — `FromValue::from_value` error arm for every implementor (`f64`, `f32`, `bool`, `String`, `i32`, `i64`, `u32`, `usize`, …) via `Value::Bool(false)` mismatch; `Clone for Box<dyn CustomValue>` via a minimal `CustomValue` impl; `Value::type_name` for every variant including `Custom`, `Object`, `Duration`. | `quartzite-core/src/value.rs` `#[cfg(test)]` module | — |
| 4 | **R7 tests** — `RectF::united` (overlapping, disjoint, identical), `RectF::translated` (zero / negative / positive offsets), `RectF::intersects` (false branches: disjoint right/bottom), `Rect::is_empty` (zero-size). | `quartzite-geometry/src/rect.rs` `#[cfg(test)]` module | — |
| 5 | **R10 tests** — `RenderHarness::width/height/scale_factor` direct accessor assertions (avoid relying solely on `no_run` doctests), `RenderHarness::fmt` `Debug` output snapshot containing each of `width=` `height=` `scale_factor=`. | `quartzite-renderer/src/render_harness.rs` `#[cfg(test)]` module (extend the existing block) | — |
| 6 | **R6 tests** — `parse_err` cases: tuple-struct (`Fields::Unnamed`), unit struct, enum, union, generic struct with type/lifetime/const params, `#[prop = …]` name-value form, unknown `#[prop(foo)]` option. | `quartzite-macros/src/object/parse.rs` `#[cfg(test)]` module (extend) | — |
| 7 | **R8 prerequisite — Font builder setters** — add `pub fn with_italic(self, v: bool) -> Self`, `pub fn with_underline(self, v: bool) -> Self`, `pub fn with_strikethrough(self, v: bool) -> Self` to `impl Font`. Each: one-line summary doc + `# Examples` doctest block (asserting `.italic()` / `.underline()` / `.strikethrough()` flips), `#[inline]`. Public-API addition only — no existing surface changes. See [§ R8 prerequisite: Font builder setters](#r8-prerequisite-font-builder-setters). | `quartzite-paint-api/src/font.rs` | — |
| 8 | **R8 production-code restructure (AC4)** — introduce `LocalBrushKind<'a>` and `LocalBrushKind::from_brush_kind`; replace the two `_ => None` arms in `brush_to_peniko` and `brush_color` with exhaustive matches over `LocalBrushKind`. **Production behaviour unchanged.** | `quartzite-renderer/src/vello_painter.rs` | — |
| 9 | **R8 tests** — extend the existing `all_painter_methods_are_invocable` block: add `draw_text` calls with `Font::new("Arial", 12.0).with_italic(true)` (covers line 260), `.with_underline(true)` (covers 308–314), `.with_strikethrough(true)` (covers 316–322). All must run without panic under the existing `FontCache::new()` setup. Also add a unit test for `LocalBrushKind::from_brush_kind` that classifies each known variant and verifies fall-through to `Unknown` is unreachable today (no public way to construct a hypothetical future BrushKind — the test asserts every known variant maps to its named arm via `matches!`, covering the four known arms exhaustively). | `quartzite-renderer/src/vello_painter.rs` `#[cfg(test)]` module | 7, 8 |
| 10 | **R4 tests** — extend the `dispatch_window_event_inner` test block: synthetic `WindowEvent::MouseInput` (Pressed then Released, Left button), assertion that `CountingRoot::press_calls` / `release_calls` increment and `pressed_buttons` mask updates; `WindowEvent::CursorMoved` updates `cursor_position`; `WindowEvent::ModifiersChanged` flips through Shift/Ctrl/Alt/Meta; `WindowEvent::KeyboardInput` with a winit `KeyEvent` whose `logical_key = Character("a")` increments `on_key_press` / `on_key_release`. Use the existing `fake_id` helper. | `quartzite-renderer/src/wrapped_handler.rs` `#[cfg(test)]` module (extend) | — |
| 11 | **R2 partial tests** — already-covered Direct path stays untouched. Add a unit test for the `arity_mismatch` and `type_mismatch` error arms via `connect_signals::<_, _, (i32, i32)>` against a 1-arg receiver and a 1-arg sender with mismatched `type_name`. Use the existing in-file test helpers. Queued/Auto arms are deferred — see [§ Deferred regions](#deferred-regions). | `quartzite-core/src/connect.rs` `#[cfg(test)]` module (extend) | — |
| 12 | **R9 partial tests** — `Timer::signals_blocked` accessor smoke; `connect_tick_queued` smoke that does not actually deliver (verify the returned `ConnectionId` is non-zero / can be disconnected); `start()` while already running is a no-op (idempotency check). Multi-thread races deferred. | `quartzite-runtime/src/timer.rs` `#[cfg(test)]` module (extend) | — |
| 13 | **Verify ≥ 93 % locally + AC6 follow-up** — run `xvfb-run -a cargo +nightly llvm-cov --workspace --doctests --summary-only`, paste the TOTAL line into the PR body. If still below 93 %, pick whichever of `keyboard.rs` / `mouse.rs` / `signal.rs` / `event_loop.rs` / `box_layout.rs` accessors is cheapest to close (each is < 12 lines of accessor / variant tests). If the run lands ≥ 93 % but < 95 %, file the stretch-goal follow-up issue (linked to #341) before `/task` Step 12; otherwise the deferred row in the spec is auto-closed. | (zero or one of) `quartzite-events/src/{keyboard,mouse}.rs`, `quartzite-core/src/signal.rs`, `quartzite-runtime/src/event_loop.rs`, `quartzite-widgets/src/layout/box_layout.rs` | 1–12 |

> The 13-task count exceeds the 7-task soft cap from the design rules. Splitting into multiple issues was considered and rejected: every subtask is a small, additive test or a one-localised refactor (tasks 7 + 8), and the coverage gate is workspace-wide so all tests need to merge together to move the Codecov metric. The atomic nature of the diff (`cargo test --workspace` runs once, `cargo llvm-cov` runs once) makes a single PR strictly cheaper. The cap is a heuristic against scope creep; here the spec was explicitly designed as a batched pass.

## Risks

- **AC4 restructure changes `vello_painter.rs` line numbers** referenced elsewhere — search for any `vello_painter.rs:\d+` references in `CHANGELOG.md`, doc comments, or other plans before commit. Mitigation: `rg 'vello_painter\.rs:\d+'` after the edit; update any stale line citations.
- **`LocalBrushKind` adds an extra layer between `BrushKind` and the renderer's two callers.** Performance impact is zero (the function is `#[inline]`-eligible and the borrow-only enum is stack-allocated; `cargo build --release` should inline it away). **Decision (finalised):** `LocalBrushKind::from_brush_kind` is a concrete method on a concrete type and its body is a 5-arm `match` — match arms count as branching under AGENTS.md § *`#[inline]` and the `_Simple._` doc tag* (the `_Simple._` shape requires "no branches/loops"). Tag with `#[inline]`, **do not** add a `_Simple._` marker.
- **Doctest gate (`# Examples` block required for new public items)** — tasks 1–6 / 9–12 are test-only and introduce no new public items, so no new doctests are required from them. Task 7 (Font builder setters) **does** add three new public items (`with_italic`, `with_underline`, `with_strikethrough`) — each MUST ship with a one-line summary `///` doc plus a `# Examples` block per AGENTS.md § *Documentation* (the doctest also counts toward coverage per Q5). Task 8 (AC4 restructure) keeps the existing `brush_to_peniko` / `brush_color` signatures (still private/`impl`-method scope) and adds `LocalBrushKind` as a **private** module-internal type, so no doc-gate impact.
- **`xvfb-run + llvmpipe` environment fragility** — task 5 (`render_harness` Debug test) does not touch the GPU, but R10's deeper accessor tests must not accidentally invoke `RenderHarnessBuilder::new(…).build()` (which needs a GPU adapter and is `#[test]`-gated on `SKIP_RENDER_SNAPSHOT`). Mitigation: construct a `RenderHarness` test fixture directly via the existing pub(crate) constructor only inside a `#[test]` that already skips on no-GPU, OR write the accessor tests purely against fields visible from a `cfg(test)` constructor. If neither is feasible, accessor coverage drops to "doctest only" — still acceptable since doctests count per Q5.
- **Q1 fragility against future BrushKind variant** — when `quartzite_paint_api::BrushKind` grows a new variant, the renderer falls through to `LocalBrushKind::Unknown` and returns `None`. Same end-state as today's `_ => None`. The test for `LocalBrushKind::from_brush_kind` exhaustively maps every existing variant; a new variant would slip through silently. Mitigation: a `// FIXME(after BrushKind extension): map the new variant in LocalBrushKind::from_brush_kind` reminder at the `_ => Self::Unknown` line. (No machine gate — that would re-introduce the `#[non_exhaustive]` problem.)
- **R2 / R9 queued-delivery test fixtures** — setting up a real queued dispatcher in a unit test requires a running `qrt::Application` event loop, which the spec puts out of scope. The Direct path tests cover the framework wiring; we accept residual ~ 30 missed lines in those two files. Acceptable because R1 + R3 + R5 + R7 + R8 + R10 already over-shoot the 93 % bar (see § Top-10 numbers).
- **AC1 measurement variance** — Codecov merges the doctest profdata into the main lcov; the workflow's lcov differs slightly from local because the workflow uses `WGPU_BACKEND=vulkan` + `WGPU_ADAPTER_NAME=llvmpipe` + `LIBGL_ALWAYS_SOFTWARE=1`. Spot-check locally with the same env vars before pushing. Coverage workflow env is identical to local `xvfb-run -a cargo +nightly llvm-cov` once the env block is exported.

## Test Design

### R1 — `event_convert.rs`

- **Location:** `quartzite-renderer/src/event_convert.rs` `#[cfg(test)] mod tests` (already exists at line 195)
- **Entry points:** `size_from_physical`, `mouse_button_from_winit`, `mouse_event_from_winit`, `key_from_winit`, `modifiers_from_winit`, `key_event_from_winit`
- **Scenarios:**
  - `size_from_physical`: `(800, 600)` → `Size::new(800, 600)`; `(u32::MAX, 0)` → `Size::new(i32::MAX, 0)`; `(0, u32::MAX)` saturates the height
  - `mouse_button_from_winit`: every variant of winit `MouseButton` including `Other(7)` → `MouseButtons::empty()`
  - `mouse_event_from_winit`: build with `Pressed { button=Left, pos=(10, 20) }` and assert resulting `MouseEvent` kind / button / pos; mirror for `Released`
  - `key_from_winit`: parametric `rstest` table over (winit Key → expected `Option<Key>`); cover every A-Z / 0-9 character (case-insensitive), every NamedKey mapped, one unmapped NamedKey (`NamedKey::Hyper` or similar) → `None`, and the `WinitKey::Unidentified` / `WinitKey::Dead` branch → `None`
  - `modifiers_from_winit`: feed `winit::event::Modifiers` with each of Shift/Ctrl/Alt/Super alone; combined Shift+Ctrl
  - `key_event_from_winit`: mapped `Character("a")` returns `Some(KeyEvent { key: Key::A, … })`; unmapped key returns `None`; `repeat=true` propagates; `text` field round-trips
- **Fixtures:** none; winit types are constructed from `pub fn`s or struct literals where fields are `pub`.
- **Note:** `WinitKey::Unidentified` and `WinitKey::Dead` are wildcard arms via `_ => None` (line 152) — they are non-exhaustive catch-alls and excluded from the ranking, but adding a test that hits them is free if winit's API allows constructing them; otherwise skip.

### R2 partial — `connect.rs`

- **Location:** `quartzite-core/src/connect.rs` `#[cfg(test)] mod tests` (extend existing block at line 374)
- **Entry points:** `connect_signals` (the typed variant)
- **Scenarios:**
  - Arity mismatch: define a 2-arg `Sender` signal and a 1-arg `Receiver` signal, attempt connect, assert `Err(SignalConnectionError::ArityMismatch { from: 2, to: 1 })`
  - Type mismatch: define `Sender` with `Signal<(i32,)>` and `Receiver` with `Signal<(f64,)>`, assert `Err(SignalConnectionError::TypeMismatch { … })`
  - Direct delivery happy path (extend an existing test if there isn't one) — assert the slot fires
- **Fixtures:** the in-file `Sender` / `Receiver` types already exist; add a `Sender2` / `Receiver2` if needed for the arity scenario.
- **Deferred:** `Queued` / `Auto` arms in `connect_signal_to_signal` (lines 165–198) and `connect_signals` (335–368) — see § Deferred regions.

### R3 — `meta.rs`

- **Location:** `quartzite-core/src/meta.rs` new `#[cfg(test)] mod tests` block at end of file
- **Entry points:** `MetaObject::fmt` (Debug), `MetaObject::eq`, `EnumMeta::fmt`, `EnumMeta::eq`, `MethodMeta::new`, `noop_lookup_entry_by_name`, `noop_lookup_entry_by_value`
- **Scenarios:**
  - Build two `MetaObject` statics with identical fields → `PartialEq::eq` returns true; tweak `class_name` → returns false; mirror for `EnumMeta`
  - `format!("{:?}", meta)` contains `"MetaObject"`, `"class_name"`, `"properties"`, `"signals"`, `"methods"`, `"enums"`; mirror for EnumMeta containing `"EnumMeta"`, `"name"`, `"entries"`
  - `MethodMeta::new("foo", &[], "()")` round-trips fields
  - `noop_lookup_entry_by_name("anything")` and `noop_lookup_entry_by_value(42)` return `None`
- **Fixtures:** small `static` `EnumEntry` / `ParamMeta` / `PropertyMeta` slices — already-existing constructors handle this.

### R4 — `wrapped_handler.rs`

- **Location:** `quartzite-renderer/src/wrapped_handler.rs` `#[cfg(test)] mod tests` (extend; existing helpers `fake_id`, `CountingRoot`, `make_handler` are already in place at lines 240–320)
- **Entry points:** `WrappedHandler::dispatch_window_event_inner`
- **Scenarios:**
  - `WindowEvent::MouseInput { state: Pressed, button: Left, … }` followed by `Released` — assert `press_calls == 1`, `release_calls == 1`, and that `pressed_buttons` is empty after the round-trip
  - `WindowEvent::CursorMoved { position: PhysicalPosition::new(50.0, 60.0) }` — verify `handler.cursor_position` updates (use `#[cfg(test)] pub(crate) fn cursor_position_for_test()` accessor if not already exposed)
  - `WindowEvent::ModifiersChanged(mods)` — feed a `winit::event::Modifiers` with Shift set; verify subsequent `KeyboardInput` propagates `KeyModifier::Shift`
  - `WindowEvent::KeyboardInput { event, … }` with `event.logical_key = WinitKey::Character("a".into())`, state Pressed — assert `key_press_calls == 1`
- **Fixtures:** existing helpers + a minimal `winit::event::KeyEvent` builder. winit's `KeyEvent` has `pub` fields, so it's directly constructible.
- **Risk:** if `WrappedHandler::cursor_position` / `modifiers` are private and have no test accessor, add `#[cfg(test)] pub(crate) fn cursor_position(&self) -> winit::dpi::PhysicalPosition<f64> { self.cursor_position }`. Mirror for `modifiers`.

### R5 — `value.rs`

- **Location:** `quartzite-core/src/value.rs` `#[cfg(test)] mod tests`
- **Entry points:** `FromValue::from_value` for each implementor (`i32`, `i64`, `u32`, `usize`, `f32`, `f64`, `bool`, `String`); `Box<dyn CustomValue>::clone`; `Value::type_name`
- **Scenarios:**
  - For each `FromValue` impl, pass `Value::Bool(true)` (or another wrong variant) — assert `Err(TypeError { expected, got: "Bool" })`
  - For `u32::from_value(Value::Int(-1))` — assert `Err(TypeError)` (the `i64 → u32` checked conversion path)
  - Implement a `MyVal(i64)` test type that impls `CustomValue`, wrap in `Box<dyn CustomValue>`, clone, assert `as_any().downcast_ref::<MyVal>()` works on the clone
  - `Value::type_name` for `Custom(_)`, `Object(_)`, `Duration(_)` — currently hit only by doctests; runtime test ensures the arm doesn't regress
- **Fixtures:** small `TestCustom(i64)` struct in the test mod.

### R6 — `object/parse.rs`

- **Location:** `quartzite-macros/src/object/parse.rs` `#[cfg(test)] mod tests` (extend; `parse_err` helper at line 223)
- **Entry points:** `parse(TokenStream)`
- **Scenarios:**
  - Tuple struct: `quote! { struct Foo(i32); }` → error message contains `"only supports named-field structs"`
  - Unit struct: `quote! { struct Foo; }` → same error
  - Enum: `quote! { enum Foo { A, B } }` → error contains `"only supports structs"`
  - Union: `quote! { union Foo { x: i32 } }` → same error
  - Generic struct (type param): `quote! { struct Foo<T> { #[prop] x: T } }` → error contains `"generic structs not yet supported"`
  - Generic struct (lifetime): `quote! { struct Foo<'a> { #[prop] x: &'a str } }` → same error
  - Generic struct (const): `quote! { struct Foo<const N: usize> { #[prop] x: [i32; N] } }` → same error
  - Name-value `#[prop]`: `quote! { struct Foo { #[prop = "x"] x: i32 } }` → error contains `"does not support name-value"`
  - Unknown option: `quote! { struct Foo { #[prop(weird_option)] x: i32 } }` → error contains `"unknown #[prop] option"`
- **Fixtures:** existing `parse_err` + `quote!`.

### R7 — `rect.rs`

- **Location:** `quartzite-geometry/src/rect.rs` `#[cfg(test)] mod tests`
- **Entry points:** `RectF::united`, `RectF::translated`, `RectF::intersects`, `Rect::is_empty`
- **Scenarios:**
  - `united`: two overlapping rects → bounding union; two disjoint rects → bounding union; identical rects → input unchanged
  - `translated`: zero offset (identity); positive offset; negative offset
  - `intersects`: false on disjoint-right, false on disjoint-bottom, false on touching-edges (exclusive bottom-right semantics)
  - `is_empty`: `Size::new(0, 10)` and `Size::new(10, 0)` both empty; `Size::new(1, 1)` not empty
- **Fixtures:** none.

### R8 — `vello_painter.rs`

- **Location:** `quartzite-renderer/src/vello_painter.rs` `#[cfg(test)] mod tests` (extend the existing block; helpers `make_scene_and_cache` already present)
- **Entry points:** `VelloPainter::draw_text`, `LocalBrushKind::from_brush_kind`
- **Scenarios:**
  - **Underline:** `Font::new("Arial", 12.0).with_underline(true)` (or whichever public setter exists; if not, extend the existing `all_painter_methods_are_invocable` test to add such a Font) + `draw_text(Point::new(0,0), "abc", &font, &solid_brush)` — assert no panic, and that `scene.encode()` (if accessible) reflects the extra `stroke` call
  - **Strikethrough:** symmetric to underline
  - **Italic:** `Font::new("Arial", 12.0).with_italic(true)` + `draw_text` — assert no panic
  - **LocalBrushKind classification (round-trip):** for each `BrushKind::{Solid, LinearGradient, RadialGradient, Custom}`, build a `Brush`, call `LocalBrushKind::from_brush_kind(brush.kind())`, assert via `matches!` it maps to the corresponding `LocalBrushKind` arm
- **Fixtures:** existing `make_scene_and_cache()`. The italic / underline / strikethrough setters land in task 7 (Font builder setters) as a prerequisite — see [§ R8 prerequisite: Font builder setters](#r8-prerequisite-font-builder-setters). Task 9 depends on task 7 in the decomposition table.

### R9 partial — `timer.rs`

- **Location:** `quartzite-runtime/src/timer.rs` `#[cfg(test)] mod tests` (extend; cfg-test block starts at line 592)
- **Entry points:** `Timer::signals_blocked`, `Timer::connect_tick_queued`, `Timer::start` (idempotency)
- **Scenarios:**
  - `signals_blocked`: brand new timer → `false`; after `block_signals(true)` → `true`
  - `connect_tick_queued`: a `Timer::new(…)`, build a dummy `ReceiverGuard`, call `connect_tick_queued(thread_id, |_args| {}, guard_weak)`, assert returned `ConnectionId` is non-zero; immediately `disconnect_tick(id)` and assert it returns `Ok(())` / `true`
  - `start` idempotency: `timer.start(driver.clone())` twice in succession; assert `is_running()` and `fire_count` after the second call equal what it was after the first (no double-arm)
- **Fixtures:** existing test driver (look for `ManualDriver` or similar in the file's existing test mod) and `Arc::new(ReceiverGuard::new(…))`.

### R10 — `render_harness.rs`

- **Location:** `quartzite-renderer/src/render_harness.rs` `#[cfg(test)] mod tests` (extend; cfg-test block starts at line 442)
- **Entry points:** `RenderHarness::fmt` (Debug), `width`, `height`, `scale_factor`
- **Scenarios:**
  - Builder-built harness when a GPU adapter is available (gated like existing tests on `SKIP_RENDER_SNAPSHOT` / `build().is_ok()`): `assert_eq!(h.width(), 64)`, `assert_eq!(h.height(), 32)`, `assert_eq!(h.scale_factor(), 1.5)`
  - `format!("{:?}", h)` contains `"RenderHarness"` and each of `"width: 64"`, `"height: 32"`, `"scale_factor: 1.5"`
- **Fixtures:** same gating pattern as `render_widget_no_op_produces_clear_color_image` (line 504) — skip on no-GPU. Under llvmpipe in CI the test runs and the accessor regions are counted.
- **Fallback:** if no-GPU early-returns prevent the accessor bodies from running anywhere except via the existing `no_run` doctests, the const accessors will still be visible to llvm-cov as zero-region functions and won't block AC1 (other targets supply the missing coverage). Accept this and move on.

## Deferred regions

Production-code uncovered regions that the design surfaces as **not feasibly deterministic in this task**. Each line is a candidate follow-up issue (per AC6 / spec § *Deferred*).

| Region | Lines | Why deferred |
|---|---|---|
| `quartzite-core/src/connect.rs::connect_signal_to_signal` Queued / Auto closure paths | 165–198 (~ 25 lines) | Closure bodies run only when a queued dispatcher (`quartzite-runtime`'s `Application` event loop) is registered via `set_queued_dispatcher`. Unit tests in `quartzite-core` cannot bring that loop up without circular dependency (`quartzite-core` is upstream of `quartzite-runtime`). Need an integration test in `tests/` of a leaf crate or in `quartzite-runtime` that exercises Queued delivery end-to-end. |
| `quartzite-core/src/connect.rs::connect_signals` Queued / Auto arms | 335–368 (~ 30 lines) | Same root cause as above — need a live queued dispatcher. |
| `quartzite-core/src/connect.rs::connect_signal_to_signal` `ConnectionType::Auto` cross-thread branch (lines 187–197) | ~ 10 lines | The "different thread" branch requires the receiver `Mutex` to be held on a different thread at emit time. Needs a multi-thread integration test inside `quartzite-runtime`. |
| `quartzite-runtime/src/timer.rs::Timer::connect_tick_auto` cross-thread branch | partial (lines around `connect_tick_auto` + driver-fire callback) | The cross-thread Auto path requires firing the timer on a thread other than the receiver's owning thread — possible but flaky in CI under llvmpipe. Out of scope for this task. |
| `quartzite-runtime/src/timer.rs::Timer::start` callback inner state.running race (line 517) | 1 line | The `if !state.running.load(…) { return; }` arm fires only when the driver invokes the callback after `stop()` flips the atomic — a rare race that's hard to deterministically simulate without injecting a fake driver. Acceptable miss. |
| `quartzite-renderer/src/vello_painter.rs::Segment::_ => {}` wildcard | line 247 | `#[non_exhaustive]` upstream-extension catch-all (`quartzite_paint_api::Segment`). Excluded from ranking per spec scope item 1. Same shape as the AC4 BrushKind problem but **not** in AC4's scope — separate follow-up. |
| `quartzite-renderer/src/vello_painter.rs::LocalBrushKind::Unknown` variant + `from_brush_kind` `_ => Self::Unknown` arm | 1 line (variant decl) + 1 line (match arm) — both inside the new `LocalBrushKind` machinery introduced by task 8 | `#[non_exhaustive]` upstream-extension catch-all created by the AC4 restructure. The `Unknown` variant is reachable only when `quartzite_paint_api::BrushKind` grows a future variant; today there is no public way to construct one. **Explicitly excluded from the ranking per spec scope item 1** ("Exclude test helpers and `#[non_exhaustive]` catch-all arms from the ranking"). No `#[coverage(off)]` attribute or `codecov.yml` `ignore:` entry is introduced (per AC4); the regions are reported by llvm-cov and accepted as uncovered. |
| `quartzite-runtime/src/factory.rs::Object` trait impl for the in-test `TestObj` type | all uncovered lines | Entire 35-line block is inside `#[cfg(test)]` — already excluded from the ranking per Q4; mentioned here only to confirm the analyst's filter matches Q4. |
| `quartzite-core/src/connect.rs` test-helper trait-impl arms | all uncovered lines (542–562, 663–682, 740–759 etc.) | All inside `#[cfg(test)]` — excluded per Q4. |

Total deferred-but-meaningful production lines: ~ 70. Closing them would require integration tests inside `quartzite-runtime` (Queued / Auto cross-thread fixtures) and is the bulk of the 95 % stretch goal per the spec.

## Open questions

- **Subtask count exceeds the 7-task design-rule soft cap.** Splitting into multiple issues was considered and rejected because (a) all tasks share a single AC (the workspace coverage gate), (b) the diff is dominantly additive tests + one localised refactor, (c) CI cost is strictly lower as one PR. Confirm with the reviewer or escalate to product owner if the cap should be enforced strictly.
- **95 % stretch goal** — explicitly deferred per spec § *Open questions*. AC6 fires the follow-up issue if 93 % ≤ coverage < 95 %.
- **Whether the `LocalBrushKind` shape should live in a new file** (e.g. `quartzite-renderer/src/local_brush_kind.rs`) instead of inline in `vello_painter.rs`. Inline is preferred under the file-size hard cap (vello_painter is currently ~ 635 lines; the addition is < 50 lines and stays well under 1000). If the reviewer prefers a separate file, the refactor is trivial.

