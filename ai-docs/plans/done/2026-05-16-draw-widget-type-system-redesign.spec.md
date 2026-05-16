# Draw-widget type-system redesign

**Source:** user description
**Date:** 2026-05-16
**Tracked in:** #373

## Problem

`DefaultStyle::draw_widget` (in `quartzite-style/src/default_style.rs`) currently dispatches on the runtime widget type by chaining six `if let Some(w) = any.downcast_ref::<…>()` arms, one per widget concrete type (`Button` / `Label` / `TextEdit` / `ScrollArea` / `Container` / `LineEdit`), each `return`ing after calling a private per-widget helper. Today's body:

```rust
fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
    let any = widget.as_any();
    if let Some(w) = any.downcast_ref::<Button>()    { return self.draw_button(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<Label>()     { return self.draw_label(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<TextEdit>()  { return self.draw_text_edit(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<ScrollArea>(){ return self.draw_scroll_area(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<Container>() { return self.draw_container(w, painter, palette); }
    if let Some(w) = any.downcast_ref::<LineEdit>()  { self.draw_line_edit(w, painter, palette); }
    // Unknown widget type — deliberate no-op; does not panic.
}
```

The user calls this "ugly" and asks whether the type system can be redesigned to use pattern matching or a visitor pattern (or another Rust-idiomatic mechanism) instead.

The same downcast-chain smell appears in `quartzite-style-dispatch/src/dispatch.rs::children_of` (two arms: `Container::children()`, `ScrollArea::content_widget`); a redesign that surfaces a structural way to enumerate a widget's children would naturally clean both call sites.

## Round-1 answers (locked-in)

- **Reach of the redesign:** _Full trait redesign._ The trait surface (`Style`, `AsWidget`, and adjacent traits) may grow; the redesign is not constrained to keep `Style` as a single-method trait. The Key Decision row that previously read "Trait method count stays at one (`draw_widget`)" is **reversed** by this answer.
- **Widget hierarchy is open-set:** third-party crates can define new widgets that integrate into the tree and are paintable by a `Style`. A closed `enum WidgetKind { Button, Label, … }` is therefore not a viable mechanism (it would force editing this crate to add a widget). Any mechanism the design phase chooses must let an external crate ship a paintable widget without modifying `quartzite-widgets` or `quartzite-style`.

## Round-2 answers (locked-in)

- **Dispatch direction:** _deferred to round 3_ — user requested explicit pros/cons per variant and a re-ask. Captured below in Key Decisions as "_open — see round-3 question 1_".
- **Children-traversal mechanism:** _Into `AsWidget`._ Enumerating a widget's children becomes part of the `AsWidget` trait surface — the exact signature shape (`fn children(&self) -> &[ObjectId]` vs `impl Iterator<Item = &dyn AsWidget>` vs a closure-callback `fn for_each_child(&self, &mut dyn FnMut(&dyn AsWidget))` vs another shape) is for the design phase to pin, but the contract is: every concrete widget exposes its children through `AsWidget`, the `quartzite-style-dispatch::children_of` downcast chain disappears, and the `Extend` proc-macro in `quartzite-macros` either generates a default `no children` impl or requires the widget author to override. Knock-on: `WidgetResolver` (the side that turns `ObjectId` back into `&mut dyn AsWidget`) keeps its current shape — children-as-`ObjectId`s vs children-as-`&dyn AsWidget` is one of the design-time sub-decisions this answer leaves open.

## Round-3 answers (locked-in)

- **Dispatch direction:** _Hybrid `Paint<W>`._ The redesign combines a typed per-widget paint surface on the style side with a widget-side dispatch entry point on the widget side:
  - **Style side — typed `Paint<W>` surface.** For every paintable widget type `W` (whether shipped by `quartzite-widgets` or by a third-party crate), a `Style` may implement a `Paint<W>` trait whose method signature is `fn paint(&self, widget: &W, painter: &mut dyn Painter, palette: &Palette)`. Concrete widget paint code is therefore typed (`&Button`, `&Label`, …) — no `.downcast_ref::<W>()` chain inside `DefaultStyle`. The exact name of the trait (`Paint<W>` vs `StylePaint<W>` vs another) and whether it lives in `quartzite-style` or `quartzite-style-types` is for the design phase to pin; the contract is that the per-widget paint surface is **typed and generic over `W`**.
  - **Widget side — dispatch entry.** `AsWidget` (or a sibling trait) gains a dispatch hook that the widget itself owns — conceptually `fn paint(&self, style: &dyn Style, painter: &mut dyn Painter, palette: &Palette)` (the precise shape — visitor double-dispatch, an associated `StylePaintToken<W>` value, or another mechanism — is for the design phase to pin). When `Style::draw_widget(&widget)` runs, control flows widget → style: the widget routes back to the matching `Paint<W>` impl on the style. This is what makes the design **open-set without enumeration** — a third-party widget `MyWidget` can supply its own `AsWidget::paint` implementation that calls `(style as &dyn Paint<MyWidget>).paint(self, painter, palette)`, and ships with its own `impl Paint<MyWidget> for SomeStyle` block.
  - **Fallback for built-ins not painted by a custom style.** A third-party `Style` that does not implement `Paint<Button>` (etc.) must compose with `DefaultStyle` (e.g. by delegation) or accept that built-in widgets render through a documented fallback path. The design phase picks the mechanism (blanket impl with default `paint` bodies on `DefaultStyle`, a `PaintFallback` extension trait, or composition through a wrapper); AC2's "compile error or documented fallback path" wording already covers both ends of the design space.
  - **Why hybrid, not pure style-knows-widget or pure widget-knows-itself.** Pure style-side dispatch (one `Paint<W>` per widget) forces the `Style` author to know every widget type in the workspace up front — incompatible with open-set. Pure widget-side dispatch (`AsWidget::paint(&self, &dyn Style)`) forces every widget to know how to drive every `Style` — co-mingling concerns and re-introducing the same dispatch-on-style problem in reverse. The hybrid keeps both directions: widget says "paint me", style supplies the concrete `Paint<W>` impl, and the type system enforces that the two sides agree on `W`.

## Scope

1. Replace the `if let Some(w) = any.downcast_ref::<Widget>()` chain inside `DefaultStyle::draw_widget` with a Rust-idiomatic dispatch mechanism whose shape is chosen at design time, constrained by the round-1 answers above (full trait redesign + open-set).
2. Touch whichever crates the chosen mechanism requires. At minimum `quartzite-style`, `quartzite-widgets`, `quartzite-style-dispatch` (the `children_of` downcast chain must disappear — see scope item 4), and `quartzite-macros` (the `Extend` proc-macro must learn to emit the new children-enumeration method).
3. **Third-party widget integration follows the Hybrid `Paint<W>` contract (round-3 locked answer).** Each paintable widget type `W` is paired with a `Paint<W>` (or design-renamed) trait whose `paint(&self, &W, &mut dyn Painter, &Palette)` method a `Style` may implement. The widget side carries a dispatch hook on `AsWidget` (or a sibling) that routes from `Style::draw_widget(&dyn AsWidget)` to the right `Paint<W>` impl. The design phase pins the exact names, the trait-vs-blanket-impl layout, and the precise widget-side hook signature; the typed-per-widget + widget-side-entry split is fixed. The mechanism is documented in the design and re-stated in `Style`'s rustdoc.
4. **Move children-enumeration into the `AsWidget` trait** (round-2 answer locked-in). The exact signature is for the design phase to pin (slice of `ObjectId` vs iterator of `&dyn AsWidget` vs `for_each_child` closure-callback vs another shape), but the contract is fixed: `quartzite-style-dispatch::children_of`'s downcast chain disappears entirely, replaced by a call through `AsWidget`. The `Extend` proc-macro emits a sensible default (most likely "no children") so that the typical widget with no child slots does not need to override.
5. Preserve **all** currently-observable behaviour:
   - The six per-widget paint outputs (Button hover/pressed/checked/disabled/focused, Label, TextEdit read-only, ScrollArea chrome, Container chrome, LineEdit placeholder/read-only) — every `quartzite-style/tests/snapshots.rs` golden remains bit-identical.
   - The unit tests in `quartzite-style/src/default_style_tests.rs` continue to pass with at most cosmetic edits.
   - The `quartzite-style-dispatch` integration tests in `quartzite-style-dispatch/src/dispatch.rs` continue to pass.
   - Unknown widget types remain a silent no-op (no panic).
6. Update doc-comments on `Style::draw_widget` and `DefaultStyle::draw_widget` to describe the new dispatch mechanism; remove the "downcast or visitor pattern, depending on the concrete impl" hand-waving in `Style`'s trait doc.

## Out of scope

- Adding new widget types or extending existing widgets with new state (this is a refactor only).
- Performance optimisations beyond the natural by-product of the dispatch redesign (e.g. removing the linear `if let` chain).
- Changes to the `Painter` trait, `Palette`, or `ColorRole`.
- Auto-installing `DefaultStyle` into the registry (separate decision, already documented in `paint-style` plan).
- Re-evaluating the per-widget hover/pressed/focused visual states.
- Reshuffling `quartzite-style-types` (the existing two-crate split that prevents `quartzite-widgets → quartzite-style`).

## Deferred

- what | why | separate issue needed?
- _none identified at round 2_

## Key decisions

| Question | Decision |
|---|---|
| Pre-publish; no compat shims | AGENTS.md § *API Stability* — free to rename / break `DefaultStyle` and adjacent surfaces without deprecation wrappers. |
| Reach of redesign (round 1 Q1) | **Full trait redesign.** Trait surface may grow; the prior "Style stays single-method" decision from the `paint-style` plan is reversed for this task. |
| Widget hierarchy closed-vs-open (round 1 Q2) | **Open.** Third-party widgets must be supportable without editing `quartzite-widgets`. Closed `enum WidgetKind` is foreclosed. |
| Snapshot goldens are load-bearing | Bit-identical pixels required; any divergence is a regression. |
| Dispatch direction (round 3 Q1) | **Hybrid `Paint<W>`.** Typed per-widget `Paint<W>` trait on the style side + a widget-side dispatch hook on `AsWidget` (or a sibling). Widget→Style→`Paint<W>` flow. Trait/method names and the precise widget-side hook shape pinned by design. See *Round-3 answers* above for the rationale. |
| Children-traversal mechanism (round 2 Q2) | **Into `AsWidget`.** Children enumeration becomes part of the `AsWidget` trait surface; `quartzite-style-dispatch::children_of`'s downcast chain disappears; `Extend` proc-macro emits a default (likely "no children") so most widget authors do not override. Exact signature shape pinned by design. |

## Technical constraints

- Workspace already builds clean on `cargo build && cargo clippy --workspace -- -D warnings && cargo test && RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. The redesign must keep all four gates green.
- `Style: Send + Sync` is non-negotiable (`StyleRegistry` hands out `&'static dyn Style`).
- `AsWidget::widget_base()` and `as_any()` (via `AsObject`) are wired through the `Extend` proc-macro on every concrete widget. Any new `AsWidget` requirement must either be implemented by hand on every widget *or* added to the `Extend` codegen.
- `quartzite-widgets` must NOT gain a `quartzite-style` dep — the existing two-crate split via `quartzite-style-types` exists precisely to keep that direction broken (`quartzite-widgets/tests/no_style_dep.rs` enforces this). If the chosen mechanism wants per-widget paint code to live alongside the widget definition, the paint code must depend only on `quartzite-style-types` / `quartzite-paint-api`, not on `quartzite-style`.
- The `Style` trait must remain object-safe — `Box<dyn Style>` and `&'static dyn Style` are the only ways the registry hands it out.
- File-size budget per AGENTS.md: target 200–400 lines, soft 500/800; today `default_style.rs` is ~232 lines with tests in a sibling file.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `DefaultStyle::draw_widget` (or its renamed successor) contains zero occurrences of `.downcast_ref::<` inside the function body. The per-widget paint helpers (`draw_button`, `draw_label`, …) take their widget arguments as concrete typed references (`&Button`, `&Label`, …) under a `Paint<W>` (or design-renamed) trait, not as `&dyn Any` / `&dyn AsWidget` with an internal downcast. |
| AC2 | The new dispatch is the Hybrid `Paint<W>` mechanism (round-3 locked-in answer): a typed per-widget `Paint<W>` trait on the style side, paired with a widget-side dispatch hook on `AsWidget` (or a sibling), routing widget → style → `Paint<W>`. Omitting support for a built-in widget produces a compile error or a documented fallback path. The design rationale is captured in `ai-docs/key-decisions.md` (or its successor location). |
| AC2b | The `Paint<W>` (or design-renamed) trait is publicly exported from `quartzite-style` (or `quartzite-style-types` — design's call) so that a third-party crate can write `impl Paint<MyWidget> for MyStyle { … }` without depending on internal modules. The trait's rustdoc documents the contract and links to the widget-side dispatch hook. |
| AC3 | All seven existing `quartzite-style/tests/snapshots.rs` goldens pass byte-identically on every supported `WGPU_BACKEND`. |
| AC4 | All `quartzite-style/src/default_style_tests.rs` tests pass without semantic edits (cosmetic edits — e.g. import paths, method renames — are fine). |
| AC5 | All `quartzite-style-dispatch/src/dispatch.rs` tests pass; `children_of`'s body contains zero occurrences of `.downcast_ref::<` and is rewritten to call through the new `AsWidget` children-enumeration surface (round-2 locked answer). |
| AC5b | The `Extend` proc-macro in `quartzite-macros` emits the new `AsWidget` children-enumeration method automatically for every concrete widget; a widget that derives `#[derive(Extend)]` without supplying any explicit override compiles, runs, and reports "no children" through the new contract. A `quartzite-macros` codegen test covers this. |
| AC6 | Unknown widget types remain a silent no-op (no panic, no warning). A regression test guarantees this — at minimum a `WidgetBase`-only fixture exercising the new dispatch path. |
| AC7 | `Style::draw_widget`'s rustdoc no longer talks about "downcast or a visitor pattern, depending on the concrete impl" hand-waving — it names the Hybrid `Paint<W>` mechanism, links to the `Paint<W>` (or design-renamed) trait, and describes the open-set extension contract for third-party widgets. |
| AC8 | A third-party-widget integration test (in `quartzite-style/tests/` or `quartzite-style-dispatch/tests/`) defines a new widget *outside* the `quartzite-widgets` source tree, defines an `impl Paint<ThatWidget> for ThatStyle { … }` (or design-renamed trait) outside `quartzite-style`, and demonstrates that `Style::draw_widget(&that_widget)` dispatches into that impl. The test also covers the documented fallback path (third-party widget under `DefaultStyle`), proving the open-set contract holds in both directions. |
| AC9 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo clippy --workspace -- -D warnings`, and `cargo build -p quartzite --no-default-features --features libm` all stay green. |
| AC10 | The `quartzite-widgets` crate continues to have no dependency on `quartzite-style` (`quartzite-widgets/tests/no_style_dep.rs` passes unchanged). |
| AC11 | `Box<dyn Style>` and `&'static dyn Style` continue to compile — `Style` remains object-safe. |

## Open questions

Items left for the design phase to pin (none block design from starting; the Hybrid `Paint<W>` shape is locked at the contract level):

- Exact name of the per-widget paint trait — `Paint<W>` vs `StylePaint<W>` vs another. Spec uses `Paint<W>` as a working name.
- Crate home for the new trait(s) — `quartzite-style` vs `quartzite-style-types` vs a new `quartzite-paint-trait` crate. Constraint: must not pull a `quartzite-widgets → quartzite-style` edge (AC10).
- Exact widget-side dispatch hook on `AsWidget` (or a sibling) — visitor double-dispatch with `fn paint(&self, &dyn Style, &mut dyn Painter, &Palette)`, an associated `StylePaintToken<W>` value, an enum-of-`TypeId`-keyed dispatch table, or another mechanism that preserves object safety on `Style`.
- Fallback strategy for built-in widgets under a custom `Style` that does not implement `Paint<W>` for all built-ins — blanket impl with default `paint` bodies on `DefaultStyle`, a `PaintFallback` extension trait, composition through a wrapper style, or another design-time choice.
- Exact children-enumeration signature on `AsWidget` (round-2 locked answer leaves shape open) — `fn children(&self) -> &[ObjectId]` vs `impl Iterator<Item = &dyn AsWidget>` vs `for_each_child` closure-callback vs another shape. Constraint: `Extend` proc-macro must emit a sensible default (likely "no children").
