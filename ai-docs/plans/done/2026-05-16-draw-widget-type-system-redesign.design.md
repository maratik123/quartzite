# Design: Draw-widget type-system redesign — `WidgetView<'a>` borrowed enum

**Issue:** #373
**Spec:** `ai-docs/plans/2026-05-16-draw-widget-type-system-redesign.spec.md`
**Date:** 2026-05-16

## Approach

### Problem framing

Both `DefaultStyle::draw_widget` (in `quartzite-style/src/default_style.rs`)
and `quartzite-style-dispatch::children_of` (in
`quartzite-style-dispatch/src/dispatch.rs`) discriminate on the runtime
widget concrete type via a chain of `any.downcast_ref::<W>()` arms. The
spec locks in:

- **Hybrid `Paint<W>`** dispatch — typed style-side surface paired with a
  widget-side dispatch hook (round-3 answer).
- **Open-set widgets** — third-party crates ship paintable widgets without
  editing `quartzite-widgets` / `quartzite-style`.
- **Children-enumeration on `AsWidget`** — `children_of`'s downcast chain
  disappears (round-2 answer); `Extend` proc-macro emits a sensible
  default.
- **`Style` stays object-safe** (AC11).
- **No `quartzite-widgets → quartzite-style` edge** (AC10).
- **Bit-identical goldens** (AC3) and unchanged dispatch tests (AC5).

The previous design round rejected a per-style `TypeId`-keyed dispatch
table on the grounds that (a) it preserved the runtime-type ladder under
a new name, and (b) it dropped the widget-side dispatch hook AC2
requires. This revision adopts the `WidgetView<'a>` borrowed-enum
mechanism: a tagged-union view over `&dyn AsWidget` that the widget side
owns and the style side pattern-matches.

The mechanism has two halves:

1. **Widget-side dispatch hook (`AsWidget::widget_view`).** Every widget
   exposes `fn widget_view(&self) -> WidgetView<'_>`. Built-in widgets
   return their own variant (`WidgetView::Button(self)` etc.); third-party
   widgets default to `WidgetView::Other(self)`. The `Extend` proc-macro
   generates this method per widget.
2. **Style-side typed dispatch (`Paint<W>` trait + `match`).** Each
   built-in style author writes typed paint code via `impl Paint<W> for
   ConcreteStyle`. The style's `draw_widget` body is a single `match` on
   `widget.widget_view()` that routes each variant to the matching
   `Paint<W>::paint` call. The `WidgetView::Other` arm is a documented
   no-op (style authors override `draw_widget` if they want to handle
   third-party widgets — they can call `widget.widget_view()` themselves
   and downcast the `Other(_)` payload via `AsObject::as_any`).

Crucially, **there is no `TypeId`-keyed runtime table anywhere**. The
match arms are statically typed (`&Button` is a `&Button`, not a downcast
result), and the `Other` escape hatch is the open-set seam. The widget
supplies the type; the style supplies the paint code; pattern matching
welds them at compile time for the built-in variants and at the `Other`
boundary for everything else.

The design phase pins five sub-decisions; each is resolved below before
the decomposition.

### Sub-decision 1 — Per-widget paint trait name and shape: **`Paint<W>` separate trait, NOT inherent methods**

The spec's open question 1 ("Does `Paint<W>` still make sense as a
separate trait, or do the per-widget helpers become inherent methods?")
deserves an explicit answer.

| Candidate | Pros | Cons |
|---|---|---|
| **`Paint<W>` separate trait** (chosen) | Third-party styles can implement `Paint<MyCustomWidget> for MyStyle` against the public trait without touching `DefaultStyle`; satisfies AC2b's "publicly exported … so that a third-party crate can write `impl Paint<MyWidget> for MyStyle`"; carries rustdoc that documents the contract once; symmetric with the `WidgetView::Other(&dyn AsWidget)` escape hatch (a custom `Style` wanting to handle `Other` overrides `draw_widget`, pattern-matches `WidgetView::Other(other)`, downcasts via `other.as_any().downcast_ref::<MyWidget>()`, then calls `self.paint(mw, painter, palette)` where `Self: Paint<MyWidget>`) | Slightly more surface than inherent methods |
| Inherent methods on `DefaultStyle` only | Smallest API surface | Third-party styles cannot share a contract; AC2b explicitly requires the trait be `pub` and re-usable — inherent methods foreclose that. Custom widget authors would have to copy the helper signatures by hand |

**Decision:** `Paint<W>` is a public trait, single method
`fn paint(&self, widget: &W, painter: &mut dyn Painter, palette: &Palette)`,
in a new module `quartzite_style::paint_widget` re-exported at the crate
root as `quartzite_style::Paint`. `DefaultStyle` implements `Paint<W>`
for each of the six built-ins; the typed `draw_button` / `draw_label` /
… helpers become the bodies of those impls (no longer inherent methods).
No blanket impl; no specialization; no supertrait coupling between
`Style` and `Paint<W>`.

This keeps `Style` and `Paint<W>` orthogonal — a `Style` impl that
handles only third-party widgets has zero compile-time obligation to
provide `Paint<Button>`. The `WidgetView::Other` arm in such a style's
`draw_widget` body is the documented fallback path AC2 calls out.

### Sub-decision 2 — Crate home for `WidgetView<'a>` and `Paint<W>`

| Type | Crate | Reason |
|---|---|---|
| **`WidgetView<'a>`** | `quartzite-widgets` | The enum variants name concrete widget types from `quartzite-widgets` (`&'a Button`, `&'a Label`, …). The natural home is alongside `AsWidget` itself, in `quartzite-widgets::widget_base`. AC10 holds trivially — the enum does not need `quartzite-style` for anything. Third-party crates that ship widgets carrying their own `WidgetView::Other(self)` body simply re-use the enum (already public). |
| **`Paint<W>`** | `quartzite-style` | `Paint<W>` is a peer of `Style` — same crate. Third-party crates already depend on `quartzite-style` to write a custom `Style`; the trait lives in the same dependency. AC10 holds trivially — nothing in `Paint<W>` requires re-exporting from widgets. The leaf crate `quartzite-style-types` is foreclosed because `Paint<W>`'s `paint` method takes `&W` where `W: AsWidget`, and `AsWidget` lives in `quartzite-widgets` — putting `Paint<W>` in `quartzite-style-types` would force the leaf to depend on widgets, closing the cycle that the leaf exists to break. |

The spec's hint ("`WidgetView` … must be in a crate that
`quartzite-style` can import … but that `quartzite-widgets` can export
without depending on `quartzite-style`") rules out putting `WidgetView`
in `quartzite-style` (widgets would have to depend on style to use the
enum, closing the cycle). `quartzite-widgets` is the natural home.

### Sub-decision 3 — Widget-side dispatch hook on `AsWidget`: `fn widget_view(&self) -> WidgetView<'_>` with proc-macro default

The hook signature:

```rust
// quartzite-widgets::widget_base
pub trait AsWidget: AsObject {
    fn widget_base(&self) -> &WidgetBase;
    fn widget_base_mut(&mut self) -> &mut WidgetBase;

    /// Widget-side dispatch hook. Returns a typed view of `self` so style
    /// implementors can pattern-match without downcasting.
    ///
    /// Built-in widgets in `quartzite-widgets` return their own variant
    /// (`WidgetView::Button(self)` etc.). Widgets defined outside this
    /// crate return `WidgetView::Other(self)` by default — style
    /// implementors that want to handle them override `Style::draw_widget`
    /// and pattern-match on `Other`'s `&dyn AsWidget` payload (typically
    /// via `AsObject::as_any().downcast_ref::<MyWidget>()`).
    fn widget_view(&self) -> WidgetView<'_>;

    /// Children enumeration (sub-decision 5).
    fn children(&self) -> WidgetChildren<'_> { WidgetChildren::Empty }
}
```

**Why this is not the previous-round impasse.** The previous design
chased a `&dyn Style → &dyn Paint<W>` conversion that stable Rust cannot
express. This design sidesteps the conversion entirely: the widget hands
the *style* a typed reference (`&'a Button` inside `WidgetView::Button`),
and the style implementor — at the point of `impl Style for
ConcreteStyle` — knows its own concrete type, so the `match` arm
`WidgetView::Button(w) => self.paint(w, painter, palette)` resolves the
typed `paint` call statically. The trait-object boundary `&dyn Style` is
crossed *before* the `match`, not *during* it.

**Proc-macro role.** The `Extend` proc-macro's existing trait emission
already builds the `AsWidget` trait declaration when a struct is marked
`#[root]` (only `WidgetBase` carries this today). For non-root widgets,
the proc-macro emits an `impl AsWidget for ConcreteWidget` body via
delegation. The new contract:

1. **`AsWidget` trait declaration** (emitted by `#[derive(Extend)]` on
   `WidgetBase`): `widget_view` becomes a required method with NO
   default body — every concrete widget must supply one. Reason: the
   default would have to be `WidgetView::Other(self)`, but `self` here is
   `&WidgetBase` (the root type), and we want concrete widgets to return
   their own typed variant, not `Other(&WidgetBase)`. A `self`-pointing
   default would discard the type information the enum exists to
   preserve.
2. **`impl AsWidget for ConcreteWidget`** (emitted by
   `#[derive(Extend)]` on each concrete widget): the proc-macro emits
   the `widget_view` body. For built-ins shipping inside
   `quartzite-widgets`, the macro emits the matching variant
   automatically — keyed by the widget's struct identifier. For
   third-party widgets that derive `Extend` from `quartzite-macros`, the
   macro emits `WidgetView::Other(self)` by default — the third-party
   author opts out of `Other` only by hand-overriding `widget_view`
   inside their own `impl AsWidget` block (rare).

   The discrimination "built-in → typed variant; everything else → Other"
   is decided **inside `quartzite-widgets`'s own derive-call sites**,
   *not* by the proc-macro inspecting outside-crate context (the
   proc-macro can't see the calling crate's identity). The mechanism:
   `quartzite-macros::Extend` exposes a `#[widget_view(variant = "Button")]`
   helper attribute that built-in widgets use to opt into a typed
   variant; widgets without the attribute default to
   `WidgetView::Other(self)`.

The proc-macro change is therefore the smallest possible: parse a new
optional `#[widget_view(...)]` attribute, emit either
`WidgetView::Button(self)` (or whatever variant name the attribute names)
or `WidgetView::Other(self)`.

### Sub-decision 4 — Fallback for built-in widgets under a custom `Style`: documented `WidgetView::Other` no-op + spec AC2 covers compile error vs documented fallback

The spec's AC2 wording: "Omitting support for a built-in widget produces
a compile error or a documented fallback path." The design picks
**documented fallback** because a compile error would close the open-set
contract — a custom `Style` that intentionally only paints
`MyCustomWidget` would otherwise fail to compile until it stubbed all
six built-ins.

The fallback path is: a custom `Style::draw_widget` body that doesn't
handle a built-in variant simply doesn't paint it. The match's
catch-all (or any unmatched arm) is documented as a no-op. `DefaultStyle`
itself handles all six built-ins so the round-trip `Box<dyn Style>`
through `StyleRegistry` continues to paint everything.

A composition wrapper (the previous design's `WrapWithDefault<S>`) is
**dropped** from this revision — the spec does not require it, the
`WidgetView` match is explicit enough that style authors can spell out
their own fallback, and YAGNI per AGENTS.md.

**The `WidgetView::Other` arm carries explicit rustdoc on the enum
variant** documenting why a silent no-op is intentional (AC6: "unknown
widget types remain a silent no-op (no panic, no warning)"). The
boundary is load-bearing: a `tracing::warn!` on every unknown widget
would be wrong because *most* third-party widgets are intentional
extension points — emitting a warning per paint frame would spam logs.
The variant doc spells this out so a future reviewer doesn't add a warn
"helpfully".

`DefaultStyle`'s `draw_widget` body for the `Other` arm is `{}` — no
painter calls, no panic, no log. This matches today's "unknown widget
type — deliberate no-op" comment in `default_style.rs` line 71.

### Sub-decision 5 — Children-enumeration signature on `AsWidget`: `fn children(&self) -> WidgetChildren<'_>` borrowed enum (carries forward from round 2)

This decision is unchanged from the previous design — the round-2 lock
stands. Signature:

```rust
// quartzite-widgets::widget_base
pub enum WidgetChildren<'a> {
    /// Slice of child ids (the common case — `Container::children()`).
    Slice(&'a [ObjectId]),
    /// At most one child (the `ScrollArea::content_widget` case).
    Optional(Option<ObjectId>),
    /// No children — the default for leaf widgets.
    Empty,
}

impl<'a> IntoIterator for WidgetChildren<'a> {
    type Item = ObjectId;
    type IntoIter = WidgetChildrenIter<'a>;
    fn into_iter(self) -> Self::IntoIter { /* ... */ }
}
```

The trait carries a default `fn children(&self) -> WidgetChildren<'_> {
WidgetChildren::Empty }`. Concrete widgets that have children
(`Container`, `ScrollArea`) override it by hand in their source files.
The `Extend` proc-macro emits nothing extra for `children` — the trait
default suffices for leaf widgets.

The enum lives in `quartzite-widgets::widget_base`, public, re-exported
at the crate root. `quartzite-style-dispatch::children_of` (and its
local `ChildIds`/`ChildIdsIter`) deletes entirely; the dispatcher's `for
child_id in children_of(widget)` becomes `for child_id in
widget.children()`.

### `Style` trait shape

`Style` stays object-safe with the same single method:

```rust
// quartzite-style::style
pub trait Style: Send + Sync {
    /// Paints `widget` using `painter`. Implementors typically `match`
    /// on `widget.widget_view()` and route each variant to the matching
    /// `Paint<W>` impl. The `WidgetView::Other` arm is a documented
    /// fallback path — by default a silent no-op.
    fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette);
}
```

`DefaultStyle`'s body becomes:

```rust
impl Style for DefaultStyle {
    fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette) {
        match widget.widget_view() {
            WidgetView::Button(w)     => self.paint(w, painter, palette),
            WidgetView::Label(w)      => self.paint(w, painter, palette),
            WidgetView::TextEdit(w)   => self.paint(w, painter, palette),
            WidgetView::ScrollArea(w) => self.paint(w, painter, palette),
            WidgetView::Container(w)  => self.paint(w, painter, palette),
            WidgetView::LineEdit(w)   => self.paint(w, painter, palette),
            WidgetView::Other(_)      => {} // documented no-op per AC6
        }
    }
}
```

The `self.paint(...)` calls resolve via `Paint<W>` impls on
`DefaultStyle`. Each `self.paint(w, ...)` call is unambiguous because
`w`'s type is known statically (`&Button`, `&Label`, …) — no turbofish
required, no `TypeId` lookup, no downcast.

**Zero `.downcast_ref::<` in the body.** AC1 satisfied literally.

### Summary of adopted contract

- New enum: `WidgetView<'a>` in `quartzite-widgets::widget_base`, public,
  with one variant per built-in widget plus an `Other(&'a dyn AsWidget)`
  escape hatch.
- New required method on `AsWidget`: `fn widget_view(&self) ->
  WidgetView<'_>`. Emitted by the `Extend` proc-macro via a
  `#[widget_view(variant = "Button")]` helper attribute for built-ins;
  defaults to `WidgetView::Other(self)` for widgets without the
  attribute (third-party widgets).
- New default method on `AsWidget`: `fn children(&self) ->
  WidgetChildren<'_> { WidgetChildren::Empty }` (round-2 lock,
  unchanged).
- New trait: `Paint<W: AsWidget + ?Sized>` in `quartzite-style`, single
  typed `paint(&self, &W, &mut dyn Painter, &Palette)` method. No
  supertrait coupling to `Style`.
- `Style` trait: unchanged signature; `DefaultStyle::draw_widget` body
  becomes a single `match` over `widget.widget_view()`. The six
  inherent `draw_button`/etc. helpers move into `impl Paint<W> for
  DefaultStyle` blocks (one per built-in).
- `quartzite-style-dispatch::children_of` + `ChildIds`/`ChildIdsIter`
  deleted; the dispatcher calls `widget.children()` directly.
- `quartzite-widgets → quartzite-style` dependency NOT introduced (AC10
  holds): `WidgetView` lives in `quartzite-widgets` and has zero
  references to `quartzite-style` types.
- `Style` object-safety preserved (AC11): no generic method, no `Self:
  Sized` bound; `Paint<W>` is a separate trait and stays off `Style`'s
  shape.
- `WidgetView::Other` is the open-set escape hatch (AC2 + AC8): a
  third-party widget surfaces as `Other(&dyn AsWidget)`; a custom
  `Style` that wants to handle it overrides `draw_widget` and
  pattern-matches `Other`'s payload through `AsObject::as_any()`.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Introduce `WidgetView<'a>` (marked `#[non_exhaustive]`) and `WidgetChildren<'a>` / `WidgetChildrenIter<'a>` types in `quartzite-widgets::widget_base`; add `fn widget_view(&self) -> WidgetView<'_>` as a **required** method on `AsWidget` and `fn children(&self) -> WidgetChildren<'_> { WidgetChildren::Empty }` as a **default** method. Update the `WidgetBase` `#[derive(Extend)]` site so the `WidgetBase`'s own `widget_view` returns `WidgetView::Other(self)` (it is not a paintable widget in its own right). Re-export `WidgetView` and `WidgetChildren` from the crate root. `WidgetView` is marked `#[non_exhaustive]` so future built-in variants can be added without breaking custom `Style` implementations that match exhaustively — they will be forced to include a catch-all arm. Unit tests in `widget_base.rs` `#[cfg(test)]`: the three `WidgetChildren` variants iterate correctly; `WidgetBase::widget_view()` returns `Other`. | `quartzite-widgets/src/widget_base.rs`, `quartzite-widgets/src/lib.rs` | — |
| 2 | Update the `Extend` proc-macro to parse a new `#[widget_view(variant = "<Name>")]` helper attribute on `#[derive(Extend)]` structs and emit the matching `widget_view` body. When the attribute is absent (third-party widgets and any built-in lacking the marker), emit `WidgetView::Other(self)`. Amend `emit_root_trait_and_impl` so `AsWidget`'s trait declaration includes `widget_view` and `children` methods with the correct doc-comments — **gated on the root trait being `AsWidget` only** (layout types such as `BoxLayout`/`GridLayout` derive `Extend` as `#[root]` emitting `AsBoxLayout`/`AsGridLayout`; those traits must NOT carry `widget_view`/`children`). For impl-body emission, `widget_view` is emitted only inside `impl AsWidget for ConcreteWidget` blocks (i.e., when the parent-chain's resolved root trait is `AsWidget`). Add a `widgets_root()` path-resolution helper (mirroring the existing `crate_root()` for `quartzite-core`) so emitted `WidgetView` and `WidgetChildren` references in proc-macro output resolve correctly when `#[derive(Extend)]` is invoked from a third-party crate (Task 9 / AC8 depends on this). Add codegen unit tests: (a) attribute present → emits `WidgetView::<Variant>(self)`; (b) attribute absent → emits `WidgetView::Other(self)`; (c) `emit_root_trait_and_impl` emits both `widget_view` and `children` in the `AsWidget` declaration with doc-attributes (extend `root_trait_methods_carry_docs`); (d) non-`AsWidget` root trait does NOT emit these methods. | `quartzite-macros/src/extend/parse.rs`, `quartzite-macros/src/extend/codegen.rs`, `quartzite-macros/src/extend/mod.rs`, `quartzite-macros/src/util.rs` | 1 |
| 3 | Annotate every built-in widget with `#[widget_view(variant = "<Name>")]`: `Button` → `Button`, `Label` → `Label`, `TextEdit` → `TextEdit`, `ScrollArea` → `ScrollArea`, `Container` → `Container`, `LineEdit` → `LineEdit`. Compile-check (each widget's existing unit tests pass, plus a new test per widget asserting `w.widget_view()` returns the matching variant carrying `&self`). | `quartzite-widgets/src/widgets/button.rs`, `…/label.rs`, `…/text_edit.rs`, `…/scroll_area.rs`, `…/container.rs`, `…/line_edit.rs` | 2 |
| 4 | Override `AsWidget::children` for `Container` (returns `WidgetChildren::Slice(&self.children)`) and `ScrollArea` (returns `WidgetChildren::Optional(self.content_widget)`). The override is a hand-written `impl AsWidget for X { fn children(&self) -> WidgetChildren<'_> { … } }` block in each widget's source file (it shadows the default; `widget_view` from the macro continues to live in the macro-emitted impl). Unit tests for each override: empty `Container` → `Empty`; non-empty → `Slice` with same `ObjectId`s; `ScrollArea` with `None`/`Some(id)` → `Optional(None)`/`Optional(Some(id))`. | `quartzite-widgets/src/widgets/container.rs`, `quartzite-widgets/src/widgets/scroll_area.rs` | 3 |
| 5 | Add the `Paint<W: AsWidget + ?Sized>` trait to `quartzite-style` in a new `paint_widget` module. Single method `fn paint(&self, &W, &mut dyn Painter, &Palette)`. Re-export at the crate root as `quartzite_style::Paint`. Trait rustdoc names the `WidgetView` match pattern, links to `Style::draw_widget`, and includes the canonical `# Examples` block (an inline `impl Paint<Button> for FakeStyle` snippet that satisfies the `# Examples` requirement per AGENTS.md). Unit tests: `&dyn Paint<Button>` constructs, asserting object-safety of the parameterised trait object; `Box<dyn Paint<Button>>` is `Send + Sync` when the impl carries those auto-traits (no extra supertrait bound on `Paint<W>`). | `quartzite-style/src/paint_widget.rs`, `quartzite-style/src/lib.rs` | — |
| 6 | Rewrite `DefaultStyle`: move each of the six existing inherent helpers (`draw_button`, `draw_label`, `draw_text_edit`, `draw_scroll_area`, `draw_container`, `draw_line_edit`) into `impl Paint<W> for DefaultStyle` blocks (one per built-in widget); the bodies are byte-identical to today's helpers — only the surrounding impl block changes. `DefaultStyle::draw_widget` becomes a single `match widget.widget_view()` with six typed arms calling `self.paint(w, …)` and an `Other(_)` no-op arm. Zero `.downcast_ref::<` occurrences in the new body. The six existing `default_style_tests.rs` test groups continue to pass (AC4) — cosmetic edits expected (e.g. `DefaultStyle::draw_button(&w, …)` becomes `<DefaultStyle as Paint<Button>>::paint(&style, &w, …)` or `style.paint(&w, …)` after a `use quartzite_style::Paint;` import). | `quartzite-style/src/default_style.rs`, `quartzite-style/src/default_style_tests.rs` (cosmetic edits only) | 1, 3, 5 |
| 7 | Update `Style`'s rustdoc (`quartzite-style/src/style.rs`) and `DefaultStyle`'s rustdoc (`quartzite-style/src/default_style.rs`) to describe the new `WidgetView` + `Paint<W>` mechanism and link to `WidgetView` / `Paint`. Remove the "downcast or visitor pattern, depending on the concrete impl" hand-waving from `Style::draw_widget`'s doc. Add a `## Implementing Paint<W> for a third-party widget` worked-example block in `Style`'s module-level rustdoc; the example matches the third-party integration test (task #9) verbatim so the test acts as a doc-test guarantee. Add explicit rustdoc on `WidgetView::Other` documenting the intentional silent no-op semantics (per spec direction issue 5). | `quartzite-style/src/style.rs`, `quartzite-style/src/default_style.rs`, `quartzite-widgets/src/widget_base.rs` (variant rustdoc on `Other`) | 1, 5, 6 |
| 8 | Delete `quartzite-style-dispatch::children_of` and the local `ChildIds` / `ChildIdsIter` enums; rewrite the `visit` body to call `widget.children()` directly. Update the file's imports (drop `Container`, `ScrollArea` from the use list). The eleven `dispatch.rs` test cases continue to pass unchanged (AC5). | `quartzite-style-dispatch/src/dispatch.rs` | 4 |
| 9 | Add the third-party integration test from spec AC8 in `quartzite-style/tests/third_party_paint.rs`: define a `ThirdPartyWidget` outside `quartzite-widgets` (via `#[derive(Extend)]` inside the test file, no `#[widget_view]` attribute → defaults to `Other`), define `impl Paint<ThirdPartyWidget> for ThirdPartyStyle` outside `quartzite-style`, and a custom `Style` impl that pattern-matches `WidgetView::Other(other)` then downcasts via `other.as_any().downcast_ref::<ThirdPartyWidget>()`. Prove `Style::draw_widget(&that_widget, …)` dispatches into the typed `Paint<ThirdPartyWidget>::paint` body. Also test the documented fallback (third-party widget under `DefaultStyle` → no-op) and the symmetric path (built-in `Button` under `ThirdPartyStyle` that doesn't handle Button-variant → no-op). | `quartzite-style/tests/third_party_paint.rs` (new) | 3, 5, 6 |
| 10 | Cross-gate verification: run `cargo build`, `cargo test`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`, and the seven snapshot tests in `quartzite-style/tests/snapshots.rs`. Verify byte-identical goldens on every available `WGPU_BACKEND` (AC3). Verify `quartzite-widgets/tests/no_style_dep.rs` still passes (AC10). Verify `Box<dyn Style>` and `&'static dyn Style` still compile via the existing assertions in `quartzite-style/src/style.rs` (AC11). | (no file edits; verification only) | 1–9 |

**M = 10.**

## Handoff plan

- **Group A — Widget-side surface (`WidgetView` + macro + per-widget
  variants):** subtasks 1–3. After subtask 3, spawn `/context-reset`.
- **Group B — Children override + `Paint<W>` + `DefaultStyle` rewrite:**
  subtasks 4–6. After subtask 6, spawn `/context-reset`.
- **Group C — Docs, dispatcher cleanup, third-party integration test +
  final verification:** subtasks 7–10. (Terminal group, sized 1..=3 …
  but this group has 4 subtasks, so split as below.)
- **Group C₁ — Docs + dispatcher cleanup:** subtasks 7–8. After subtask
  8, spawn `/context-reset`.
- **Group C₂ — Integration test + cross-gate verification:** subtasks
  9–10. (Terminal group, size 2.)

Rationale: Group A is the self-contained widget-side surface change
(enum + macro + per-widget annotations); cleanly separable from the
style-side work and reaches a stable green build at its boundary (every
widget compiles with the new `widget_view` method, the workspace still
builds because `Style::draw_widget` hasn't changed yet — the new
`WidgetView` is an unused-import-style "dead" surface until Group B
wires it in). Group B is the load-bearing style-side work — `Paint<W>`,
the `DefaultStyle` migration, and the `Container`/`ScrollArea` children
overrides — and reaches a stable green test state at its boundary
(`default_style_tests.rs` + snapshots both pass). Group C₁ cleans up
the dispatcher and rustdoc; the deletion of `children_of` requires
Group A's shipped `AsWidget::children`. Group C₂ ships the spec's AC8
third-party test and gates the full cross-tool verification. Each
non-terminal group sits at ≤ 3 subtasks; the terminal group has 2.

## Risks

| # | Risk | Mitigation |
|---|---|---|
| R1 | **`widget_view` is a required method on `AsWidget` (no default), so adding it is a hard breaking change for every concrete widget.** Pre-publish per AGENTS.md § *API Stability* — clean breaks are allowed, no compat shims. | Task #2's proc-macro change emits `widget_view` automatically for every `#[derive(Extend)]` widget. The only widgets that need hand-edits are the six built-ins (Task #3 adds the `#[widget_view(...)]` attribute). Existing third-party widgets in `examples/` (none currently) would compile with `WidgetView::Other(self)` automatically. The breaking change is mechanical and proc-macro-driven. |
| R2 | **The `Extend` proc-macro must learn a new helper attribute (`#[widget_view]`); proc-macro changes are subtle.** | Task #2 includes codegen unit tests covering (a) attribute present, (b) attribute absent, (c) malformed attribute (compile error). The existing codegen test infrastructure in `quartzite-macros/src/extend/codegen.rs::tests` is the template. |
| R3 | **Snapshot drift.** Re-routing the per-widget paint bodies through `Paint<W>` impls instead of inherent methods could in principle re-order generated code if rustc optimisations differ. | Task #6 keeps the six `draw_*` helper bodies byte-identical (only the surrounding `impl` block changes — from `impl DefaultStyle { fn draw_button(...) {...} }` to `impl Paint<Button> for DefaultStyle { fn paint(...) {...} }`). Task #10 runs all seven snapshot tests on every available backend and confirms byte-identical pixels (AC3). |
| R4 | **`AsWidget::widget_view` returning `Other(self)` for `WidgetBase` itself is a discoverability footgun.** A user could pass a bare `WidgetBase` to `Style::draw_widget` and silently get no painting. | This matches today's behaviour: the existing downcast chain in `DefaultStyle::draw_widget` does not handle `WidgetBase` (it has no `if let Some(w) = any.downcast_ref::<WidgetBase>()` arm) and falls through silently. AC6 explicitly requires "Unknown widget types remain a silent no-op (no panic, no warning)". The `WidgetView::Other` arm preserves the contract. Variant rustdoc (task #7) calls this out so future readers don't mistake the no-op for a bug. |
| R5 | **`WidgetView::Other(&'a dyn AsWidget)` carries a `dyn` trait object — third-party styles wanting to handle a specific custom widget through `Other` must downcast via `AsObject::as_any()`.** This re-introduces a downcast at the *third-party* style's call site. | Acceptable trade-off: AC1 only forbids `.downcast_ref::<` inside `DefaultStyle::draw_widget`. Third-party styles that handle their own widgets typically have one or two concrete types — a single `downcast_ref::<MyWidget>()` at their call site is the open-set seam, not a runtime ladder. The variant rustdoc (task #7) documents this idiom and the third-party integration test (task #9) demonstrates it. |
| R6 | **`#[widget_view(variant = "X")]` introduces a name-string that must match a real enum variant in `WidgetView` — a typo in the attribute would compile inside the proc-macro but fail at the usage site (the emitted `WidgetView::Xyz(self)` would not resolve).** | This IS the right failure mode — the typo manifests as a clear compile error pointing at the variant name. Task #2's codegen tests include a positive case (correct variant compiles) and an integration check in task #3 (every built-in's annotation resolves). The proc-macro deliberately does not try to validate the variant name against the enum (it cannot see the enum's definition from a different crate); this is consistent with how other helper-attribute-driven macros in the workspace behave. |
| R7 | **The proc-macro change in Task #2 affects every `#[derive(Extend)]` use site in the workspace** — not just paintable widgets. Layout types (`BoxLayout`, `GridLayout`) also derive `Extend`. | The new `widget_view` method is on `AsWidget`, which is *only* emitted when `#[root]` is present (today, only `WidgetBase` carries `#[root]`). Non-`#[root]` `#[derive(Extend)]` users get parent-chain delegation, which now delegates `widget_view` through the chain to whichever concrete widget supplies the override. Layout types don't derive `AsWidget` — they derive their own `AsLayoutBase` mixin trait. The proc-macro change is scoped: parse the helper attribute on every `#[derive(Extend)]` (cheap), emit `widget_view` body only inside `impl AsWidget for ConcreteWidget` blocks (which only fire for descendants of `WidgetBase`). |
| R8 | **`AsWidget` is a generated trait** (proc-macro emits its declaration when `WidgetBase` is `#[root]`). Adding `widget_view` and `children` to it requires the proc-macro's `emit_root_trait_and_impl` to include the new methods. | Task #2 amends `emit_root_trait_and_impl` in `quartzite-macros/src/extend/codegen.rs` to include both `widget_view` (required, with `# Examples` doc) and `children` (default body, with `/// _Simple._` marker per AGENTS.md for simple default-method bodies returning `WidgetChildren::Empty`). The existing test at `root_trait_methods_carry_docs` (line 640 of `codegen.rs`) is amended to also assert `widget_view` carries a doc attribute. |
| R9 | **`children_of`'s deletion in `quartzite-style-dispatch` must preserve the existing 11 test cases** (eight ACs + three covering ScrollArea / unknown widget / closure resolver). The `Container::children() → &[ObjectId]` and `ScrollArea::content_widget → Option<ObjectId>` shapes are load-bearing for several tests. | Task #4 overrides `AsWidget::children` so the produced iterator yields the same `ObjectId`s in the same order as today's `children_of`. Task #8 confirms by running the eleven dispatch tests unchanged. |
| R10 | **File-size budget.** `default_style.rs` is 232 lines today; moving each helper into a separate `impl Paint<W> for DefaultStyle` block adds one `impl` header (~3 lines) per built-in × 6 = +18 lines. Each helper body is unchanged. New `match` body in `draw_widget` is ~10 lines (replaces a 21-line downcast chain). Net: ~232 - 21 + 18 + 10 = ~239 lines. Safely under the soft 500. | No mitigation needed; tracked. |
| R11 | **`Style: Send + Sync` is non-negotiable** (`StyleRegistry` hands out `&'static dyn Style`). Adding `Paint<W>` does NOT change `Style`'s shape (Sub-decision 1 keeps them orthogonal — no supertrait coupling). | Task #5's unit test asserts `Box<dyn Style>` still satisfies `Send + Sync`; AC11 trivially holds because `Style`'s trait signature is unchanged. The existing `style_trait_object_is_send_sync` test in `quartzite-style/src/style.rs` covers this. |
| R12 | **The third-party integration test (Task #9) requires `AsObject::as_any()` to be accessible for downcasting `WidgetView::Other`'s payload.** `AsObject::as_any()` is already part of the macro-emitted `AsObject` impl (`quartzite-macros/src/extend/codegen.rs::emit_as_object_impl` lines 198–235); the test relies on existing behaviour. | No new mitigation needed; the test exercises the existing surface. |
| R13 | **Round-trip via `StyleRegistry::set_style(Box::new(DefaultStyle))` must still work.** | Task #6's tests include the `registry_round_trip_dispatches_default_style` test from `default_style_tests.rs` unchanged (per AC4). `DefaultStyle` remains a zero-sized struct. |
| R14 | **`Container::children()` name collision.** `Container` already has an inherent `pub fn children(&self) -> &[ObjectId]`; the new `AsWidget::children(&self) -> WidgetChildren<'_>` trait method co-exists with the same name but different return type. | This is intentional and safe: Rust's inherent-method-wins-on-concrete-receiver rule means `container.children()` on `&Container` still calls the inherent method (returning `&[ObjectId]`); the trait method is only reachable through `&dyn AsWidget`. `children_of`'s rewrite (Task #8) calls `widget.children()` on `&dyn AsWidget`, so it uses the trait method. Both shapes are load-bearing and must coexist. The inherent method is retained unchanged; the trait override (Task #4) adds the separately-named `impl AsWidget for Container { fn children(&self) -> WidgetChildren<'_> { … } }` block. |

## Test Design

| Area | Location | Entry point | Scenarios | Fixtures |
|---|---|---|---|---|
| `WidgetView` enum + `WidgetChildren` enum + `IntoIterator` chain (task 1) | `quartzite-widgets/src/widget_base.rs` `#[cfg(test)]` | `WidgetView::Other(&base)`; `WidgetChildren::*.into_iter()` | `WidgetBase::widget_view()` returns `Other(&self)`; `WidgetChildren::Slice(N ids).into_iter()` yields N ids in order; `Optional(Some)` yields one; `Optional(None)` and `Empty` yield zero | Inline `ObjectId::new()` ids |
| `Extend` macro codegen for `widget_view` (task 2) | `quartzite-macros/src/extend/codegen.rs` `#[cfg(test)]` | `super::codegen(parse(quote! { … }))` | (a) Struct with `#[widget_view(variant = "Button")]` → emitted body contains `WidgetView :: Button (self)`. (b) Struct without the attribute → emitted body contains `WidgetView :: Other (self)`. (c) `emit_root_trait_and_impl` emits `fn widget_view(&self) -> WidgetView<'_>` and `fn children(&self) -> WidgetChildren<'_>` in the trait declaration, both carrying doc-attributes (extend `root_trait_methods_carry_docs`). | Inline `TokenStream` fixtures (existing pattern in the file) |
| `Container::children()` / `ScrollArea::children()` overrides (task 4) | `quartzite-widgets/src/widgets/{container,scroll_area}.rs` `#[cfg(test)]` | `Container::children()` / `ScrollArea::children()` (the new `AsWidget` trait method, distinct from the inherent `Container::children() -> &[ObjectId]` that already exists) | Empty container → `Empty`; non-empty → `Slice` with same `ObjectId`s as the inherent `Container::children()`; `ScrollArea` with `None` → `Optional(None)`; with `Some(id)` → `Optional(Some(id))` | Inline construction |
| Per-widget `widget_view()` (task 3) | Each of `quartzite-widgets/src/widgets/{button,label,text_edit,scroll_area,container,line_edit}.rs` `#[cfg(test)]` | `w.widget_view()` | For each built-in: match the returned `WidgetView` variant; assert the payload reference points to the same widget (compare by raw pointer / object id). | Existing widget constructors |
| `Paint<W>` object-safety + send/sync (task 5) | `quartzite-style/src/paint_widget.rs` `#[cfg(test)]` | `Box<dyn Paint<Button>>`, `assert_send_sync` over an `impl Paint<Button>` zero-sized type | Construct `Box<dyn Paint<Button>>` via a fixture `Send + Sync` zero-sized style; assert the box is `Send + Sync`; assert `&dyn Paint<Button>` compiles (object-safe). | Inline zero-sized fixture style implementing `Paint<Button>` |
| `DefaultStyle` per-widget paint (task 6) | `quartzite-style/src/default_style_tests.rs` (existing 30+ tests) | `DefaultStyle.draw_widget(&w, ...)`; `<DefaultStyle as Paint<W>>::paint(&style, &w, …)` for direct typed access | All existing tests pass with at most cosmetic edits (import-path renames; calling the helpers through the `Paint` trait). AC4 explicitly requires this. New test: assert `WidgetView::Other(&base)` arm is reached for `WidgetBase` and produces zero painter calls (AC6 regression). | Existing `RecordingPainter` fixture |
| `Style` rustdoc + `WidgetView::Other` rustdoc gate (task 7) | `cargo doc` | n/a | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` succeeds. Intra-doc links from `Style` and `DefaultStyle` to `Paint<W>` and `WidgetView` resolve. | Workspace |
| `dispatch_paint` regression (task 8) | `quartzite-style-dispatch/src/dispatch.rs` `#[cfg(test)]` (existing 11 tests) | `dispatch_paint(root, resolver, painter, palette)` | All 11 existing tests pass unchanged (AC5). | Existing `StubResolver`, `RecordingPainter`, `MarkStyle` fixtures |
| Third-party integration test (task 9) | `quartzite-style/tests/third_party_paint.rs` (new) | `Style::draw_widget(&that_widget, ...)` | (a) `ThirdPartyWidget` defined via `#[derive(Extend)]` inside the test file (no `#[widget_view]` attribute → defaults to `Other`); `impl Paint<ThirdPartyWidget> for ThirdPartyStyle` defined; custom `Style` impl matches `WidgetView::Other(other)` then `if let Some(w) = other.as_any().downcast_ref::<ThirdPartyWidget>() { self.paint(w, ...) }`; assert paint body runs. (b) `Button` under a custom `Style` that doesn't handle `WidgetView::Button` → no painter calls (AC2 documented fallback). (c) `ThirdPartyWidget` under `DefaultStyle` → no painter calls (AC6 silent no-op). | `ThirdPartyWidget` defined via `#[derive(Extend)]` + `#[base] widget_base: WidgetBase` inside the test file |
| Snapshot bit-identity (task 10) | `quartzite-style/tests/snapshots.rs` (existing 7 tests) | `DefaultStyle::draw_widget` via `RenderHarness` | All 7 goldens pass byte-identically on every available `WGPU_BACKEND`. AC3. | Existing harness + golden PNGs |
| Object-safety + send/sync (task 10) | `quartzite-style/src/style.rs` `#[cfg(test)]` (existing `style_trait_object_is_send_sync`) | `assert_send_sync::<Box<dyn Style>>()`, `assert_send_sync::<&'static dyn Style>()` | Existing assertions still compile. AC11. | Existing fixtures |
| Cycle-break (task 10) | `quartzite-widgets/tests/no_style_dep.rs` (existing) | `cargo tree -p quartzite-widgets` | Existing test still passes — no line starts with `quartzite-style ` (trailing space). AC10. | Existing fixture |
| No-default-features build (task 10) | `cargo build -p quartzite --no-default-features --features libm` | n/a | Builds clean. AC9. | Workspace |

## Open questions

- **O1 — `#[widget_view(variant = "X")]` helper-attribute syntax.** The
  design uses `variant = "<Name>"` with a quoted string for parser
  simplicity. Alternative: a bare identifier (`#[widget_view(Button)]`).
  Both work; the quoted-string form is closer to how `serde`'s
  `#[serde(rename = "X")]` reads. Decision is cosmetic, pick during
  implementation. **Recommended:** quoted-string form for consistency
  with other `quartzite-macros` attribute parsing patterns.

- **O2 — `WidgetView` non-exhaustiveness marker.** **Resolved in Task 1:** `WidgetView` is marked `#[non_exhaustive]`. The `Other` arm is the open-set escape hatch; `#[non_exhaustive]` is the type-system-level counterpart that forces custom `Style` match arms to include a catch-all, matching `std` conventions (`std::io::ErrorKind`, etc.).

- **O3 — `WidgetView::Other(&'a dyn AsWidget)` payload type.** The
  `dyn AsWidget` payload lets a third-party `Style` downcast via
  `AsObject::as_any()`. Alternative: `Other(&'a dyn AsObject)` would be
  narrower (just enough for downcast) and decouple `WidgetView` from
  `AsWidget`'s wider surface. But every `AsWidget` is also `AsObject`
  (chain), and styles that handle `Other` typically also need the
  widget's geometry/visibility (which lives on `AsWidget::widget_base`).
  Keeping `&dyn AsWidget` is the path of least friction.

  **Recommended:** keep `Other(&'a dyn AsWidget)`.

- **O4 — Compile-error vs silent-no-op for unhandled built-in variants
  in a custom `Style`.** Spec AC2 explicitly allows "compile error OR
  documented fallback path". The design picks documented fallback. A
  future improvement could provide a `#[derive(DispatchAllBuiltins)]`
  macro that emits a `match` covering every built-in variant with
  per-variant `Paint<W>` calls, generating a compile error when any
  built-in's `Paint<W>` impl is missing on `Self`. Out of scope for this
  spec.

  **Recommended:** documented fallback is the contract for now; the
  derive macro is a YAGNI follow-up.
