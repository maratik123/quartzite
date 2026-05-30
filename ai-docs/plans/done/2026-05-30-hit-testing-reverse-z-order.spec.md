# Hit-testing traversal in reverse z-order

**Source:** issue #395
**Date:** 2026-05-30
**Tracked in:** #395

> Surfaced by `/triage` from [`ai-docs/deferred/widget-backlog.md`](../deferred/widget-backlog.md). Source spec: [`2026-05-13-renderer-style-dispatch.spec.md`](done/2026-05-13-renderer-style-dispatch.spec.md). The paint dispatcher (`dispatch_paint`) walks the widget tree front-to-back (parent-before-child, painter's-algorithm z-order) and paints each visible node. This task adds the inverse traversal: given a point, find the topmost visible widget under it — walking the same tree in **reverse z-order** so the widget painted *last* (visually on top) wins the hit. This is the traversal primitive the input-dispatch pass will build on; it does **not** add an event-dispatch loop.

## Scope

Add a **point → widget** hit-testing free function that mirrors the existing `dispatch_paint` traversal but in reverse z-order, reusing the same tree-walk infrastructure (`WidgetResolver`, `AsWidget::children()`, `WidgetBase::geometry`, `WidgetState::Visible`, `children_clip_rect()`).

- Exposes a single public **free function** that takes a point in the root widget's local coordinate space, a `root: ObjectId`, and a `&dyn WidgetResolver`, and returns the deepest/topmost visible widget whose geometry contains the (transform-adjusted) point, together with the accumulated origin offset from root-local space to that widget's local space.

  ```rust
  pub fn hit_test(
      root: ObjectId,
      point: Point,
      resolver: &dyn WidgetResolver,
  ) -> Option<(ObjectId, Point)>   // (hit id, accumulated origin offset) — see § Key decisions (Q2)
  ```

  The returned `Point` is the sum of the parent-relative origins traversed from the root down to the hit widget — i.e. the offset that maps the hit widget's local origin into root-local space. A caller maps the original root-local `point` into the hit widget's local space by subtracting this offset (`point - offset`), without re-walking the tree.

- Walks the subtree rooted at `root` and returns the **topmost** widget under `point` according to paint z-order. Because paint order is parent-before-child and earlier-sibling-before-later-sibling, the *last-painted* (visually topmost) widget at a point is found by testing children **before** their parent and **later siblings before earlier siblings** (reverse of paint iteration). The first containing widget found in that reverse walk is the hit.

- Mirrors `dispatch_paint`'s coordinate model: child geometry is parent-relative, so the search translates `point` into each child's local space by subtracting the child's `geometry().origin()` before recursing (the inverse of `dispatch_paint`'s `painter.translate(origin)`).

- Mirrors `dispatch_paint`'s visibility filter: a widget with `!WidgetState::Visible` is not a hit candidate and **its entire subtree is skipped** (an invisible parent hides its children from hit-testing, exactly as it hides them from paint).

- Mirrors `dispatch_paint`'s clip handling: when a widget returns `Some(clip)` from `children_clip_rect()`, a point outside that clip rect cannot hit any of that widget's children (the children are visually clipped away). The clipped parent itself may still be a hit if its own geometry contains the point.

- Reuses the existing `WidgetResolver` trait (no new resolver abstraction). Resolver misses on a child id are treated the same way `dispatch_paint` treats them — the missing subtree is skipped (it cannot be a hit), with a `tracing::warn!` event. A `None` root resolves to no hit.

Concretely:

- The hit-test function is **paint-free**: it touches no `Painter`, `Style`, `StyleRegistry`, or `Palette`. It needs only the widget tree (resolver + geometry + visibility + clip rect) and a point.
- Traversal is depth-first; at each node the function first recurses into children in **reverse** child-iteration order (so the topmost sibling is tested first), then — if no child claimed the hit — tests the node's own geometry.
- A point that lands on a parent's chrome but outside all of its children hits the parent. A point inside a child's geometry hits the child (or a descendant), never the parent, matching the visual stacking.

## Out of scope

- An event-dispatch loop / routing of `MouseEvent` to `WidgetExt::on_mouse_press` etc. This spec covers the **hit-test traversal primitive only**; wiring it into `WidgetRoot::on_mouse_press` / `quartzite-renderer`'s window dispatch is a separate follow-up (the "event-dispatch design beyond what `WidgetRoot::on_mouse_press` does today" the source spec § Deferred names). New issue when the dispatch loop lands.
- Mouse-capture / grab semantics (a widget capturing all subsequent events until release). Future input-plumbing concern.
- Hover / enter / leave tracking and cursor-shape changes. Tracked separately (#404 cursor-shape, hover-state items in the backlog).
- Z-order overrides / `raise()` / `lower()` / explicit stacking beyond child-iteration order. v1 reverse-z-order is purely the inverse of paint iteration order; no per-widget z-index field exists.
- Overlapping-sibling tie-breaking beyond "later-painted sibling wins." Containers lay children out without overlap today; the reverse-iteration rule is the complete tie-break.
- Per-widget custom hit shapes (non-rectangular hit regions, e.g. circular buttons). v1 hit-tests against the rectangular `geometry()`; a `WidgetExt::contains_point`-style override is a future concern.
- Transforming the result into screen/global coordinates. The function works in root-local space; callers convert if they need global coordinates.
- Multi-window hit-testing. Each call hit-tests one tree, exactly like `dispatch_paint` walks one tree.

## Deferred

- Event-dispatch loop routing `MouseEvent` to the hit widget's `on_mouse_press` / `on_mouse_release` / focus transitions | needs an event-routing design + mutable tree access (`&mut dyn AsWidget`) | new issue when the input-dispatch pass lands.
- Per-widget custom (non-rectangular) hit shapes via a `WidgetExt::contains_point` override | needs a hit-shape abstraction + audit of every widget type | new issue if a non-rectangular widget surfaces.
- Mouse-capture / pointer-grab so a pressed widget keeps receiving moves until release | needs the event-dispatch loop first | new issue with the dispatch loop.

## Key decisions

| Question | Decision |
|---|---|
| Mirror of `dispatch_paint` | The traversal is the structural inverse of `dispatch_paint` (`quartzite-style-dispatch/src/dispatch.rs`): same depth-first walk over `WidgetResolver` + `AsWidget::children()`, same visibility-skips-subtree rule, same `children_clip_rect()` handling, same resolver-miss `warn!`-and-skip — but child iteration is **reversed** and the test runs child-before-parent so the visually-topmost widget wins. |
| Coordinate model | `point` is in the root widget's local space (`(0,0)` = root's top-left), matching the space `dispatch_paint` paints the root in. Recursion subtracts each child's `geometry().origin()` from the point — the exact inverse of `dispatch_paint`'s `painter.translate(child.geometry().origin())`. |
| Visibility filter | `!WidgetState::Visible` ⇒ the widget is not a hit candidate and its whole subtree is skipped (mirrors `dispatch_paint`). A hidden root ⇒ `None`. |
| Clip handling | A widget whose `children_clip_rect()` is `Some(clip)` only exposes its children to hits when the point lies inside `clip`; a point outside `clip` skips that widget's children but the widget itself can still be the hit. Mirrors the paint-side clip. |
| Resolver-miss policy | Resolver returns `None` for a referenced child ⇒ that subtree contributes no hit and a `tracing::warn!` fires (same message shape as `dispatch_paint`). A `None` root ⇒ `None` hit (+ `warn!`). |
| Hit shape | Rectangular `geometry()` only in v1. `Rect::contains(point)` is the membership test (`quartzite-geometry/src/rect.rs`). Non-rectangular shapes are § Deferred. |
| Return shape (Q2, round 1) | **`Option<(ObjectId, Point)>`** — the hit widget's id plus the accumulated origin offset (sum of parent-relative origins from root to hit), so a caller maps the root-local `point` into the hit widget's local space via `point - offset` without re-walking. `None` = no hit (point outside the visible tree, hidden root, or `None` root). A full ancestor *path* was considered and rejected: the single accumulated offset is what callers need for local-space mapping, and id + offset is the minimal sufficient result; a path is recoverable later via the resolver if ever needed. |
| Crate location (Q1, round 1) | **New paint-free crate** (provisional name `quartzite-hit-test`; `quartzite-widget-tree` is an alternative; final name pinned in design — single workspace `members` slot at workspace root). It depends on **no** paint/style types (`Painter`/`Style`/`StyleRegistry`/`Palette`) — only `quartzite-widgets` + `quartzite-geometry` + `quartzite-core` (`ObjectId`) + `tracing`. The read-only `WidgetResolver` trait **moves** from `quartzite-style-dispatch` into this crate; `quartzite-style-dispatch` adds a dep on the new crate and **re-exports** `WidgetResolver` so `dispatch_paint`'s signature is unchanged. This gives the cleanest dep cone: hit-test sits below the paint bridge, sharing one resolver type. (Whether the resolver lands in this hit-test crate or in a still-lower shared widget-tree crate is a design-phase placement detail; either satisfies the paint-free + single-resolver constraints.) |

## Technical constraints

- Reuses the existing read-only paint-time `WidgetResolver` trait (`fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>`), currently in `quartzite-style-dispatch/src/dispatch.rs`. Per the Q1 decision the trait **moves** into the new paint-free crate; `quartzite-style-dispatch` re-exports it so `dispatch_paint`'s public signature is unchanged. This is the immutable `&dyn AsWidget` resolver — **not** the separate mutable layout-time `WidgetResolver` (`fn resolve_widget_mut(&mut self, id) -> Option<&mut WidgetBase>`) in `quartzite-widgets/src/layout/mod.rs`, which stays put. Hit-test does **not** define a second resolver abstraction. Pre-publish, free to restructure per `AGENTS.md` § *API Stability*.
- The blanket `impl<F: Fn(ObjectId) -> Option<&'static dyn AsWidget>> WidgetResolver for F` (and its `&'static` lifetime caveat) moves with the trait into the new crate.
- `AsWidget`, `WidgetChildren`, `WidgetState`, and the `children()` / `children_clip_rect()` / `widget_base()` accessors are generated by `quartzite-macros`' `#[extend]` codegen and surfaced via `quartzite-widgets`; the new crate consumes them through `quartzite-widgets` exactly as `quartzite-style-dispatch` does today (no new macro work).
- `AsWidget::children()` returns `WidgetChildren<'_>` (`quartzite-widgets/src/widget_base.rs`), which is `IntoIterator<Item = ObjectId>`. Reverse iteration over it: collect to a small stack buffer or iterate the underlying slice in reverse (the `Slice` and `Optional` variants are the only non-empty shapes; `Optional` has at most one element, so only `Slice` needs reversing).
- `WidgetBase::geometry` is parent-relative; `Rect::origin()` / `Rect::contains(Point)` exist (`quartzite-geometry/src/rect.rs:157`). No new geometry API is required.
- `children_clip_rect()` is on `AsWidget` (`quartzite-widgets`); the clip rect is in the parent's local space (same space the point is in at that recursion level).
- The function takes everything by reference / value (`ObjectId`, `Point`, `&dyn WidgetResolver`) and returns an owned result — no internal state outlives the call. No `unsafe`, no `unwrap()`/`expect()` outside test fixtures; resolver misses are `warn!` events, not panics (`AGENTS.md` § *API Naming* / *Library safety idioms*).
- A `debug_span!` guard wraps the body per `AGENTS.md` § *Code Style → Tracing* (lifecycle-level; one span per hit-test call is not hot).
- Doc gate: every new public item (the free fn, any new public return type) carries `///` first-line docs + a `# Examples` block + `# Parameters` where applicable. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `#[cfg(test)] mod tests` block reusing the `StubResolver` (HashMap-backed) fixture shape from `quartzite-style-dispatch/src/dispatch.rs` — no `RecordingPainter`/`MarkStyle` needed (hit-test is paint-free).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | A public free function `hit_test(root: ObjectId, point: Point, resolver: &dyn WidgetResolver) -> Option<(ObjectId, Point)>` exists. Given a single visible root whose geometry contains `point`, it returns `Some((root, Point::ZERO))` (root-local offset is zero); given a point outside the root's geometry, it returns `None`. |
| AC1b | The returned offset for a nested hit equals the sum of parent-relative origins from root to the hit widget. For `Container(at 0,0) { Inner(at 10,20) { Leaf(at 5,5) } }`, a point inside `Leaf` returns `(Leaf, Point::new(15,25))`; subtracting the offset from the original root-local point yields the point in `Leaf`'s local space. |
| AC2 | Reverse z-order: for a tree with two overlapping/stacked siblings where the second-iterated sibling is painted on top, a point inside both siblings' geometry hits the **second** sibling (the visually-topmost one), not the first. A test with two siblings at the same origin asserts the later-iterated sibling wins. |
| AC3 | Child-before-parent: a point inside a child's geometry hits the **child** (or a deeper descendant), never the parent, even though the parent also contains the point. A point on the parent's chrome but outside every child hits the **parent**. |
| AC4 | Coordinate transform: for `Container(at 0,0) { Label(at 10,20, size 50×20) }`, a point at `(15,25)` (root-local) hits the `Label`; a point at `(5,5)` hits the `Container`. The recursion correctly subtracts child origins. |
| AC5 | Visibility: a hidden root ⇒ `None`. A hidden non-root child (and its whole subtree) is never returned even when the point lies inside its geometry — the parent is the hit instead (or `None` if the point is outside the parent too). |
| AC6 | Clip: for a `ScrollArea` (or any widget) with `children_clip_rect() == Some(clip)`, a point outside `clip` but inside a child's geometry does **not** hit the child (it hits the clipping widget or `None`); a point inside both `clip` and the child hits the child. |
| AC7 | Resolver-miss: when `resolver.resolve` returns `None` for a child `ObjectId`, that subtree yields no hit and a `tracing::warn!` event fires; the parent and sibling subtrees still hit-test normally. A `None` root returns `None` and emits a `warn!`. |
| AC8 | A miss with no containing widget anywhere in the (visible) tree returns `None` with no panic. |
| AC9 | A doc-test or runnable `# Examples` block demonstrates end-to-end usage: a caller builds a fixture tree, calls `hit_test` with a point, and asserts the returned `(ObjectId, Point)`. Compiles under the doc gate, runs under `cargo test`. |
| AC10 | `cargo build`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` are clean. If Q1 introduces or moves a crate, it participates in the `--workspace` gates and is listed in workspace `members`. |

## Open questions

- **Overlapping-sibling semantics in practice** — today's `Container` layouts do not overlap siblings, so AC2's "later sibling wins" rule is exercised only by a synthetic test. The rule is the correct inverse of paint order regardless; revisit if an explicit z-index / `raise()` API lands (§ Out of scope).
- **Final crate name + exact resolver home** — `quartzite-hit-test` vs `quartzite-widget-tree`, and whether the shared `WidgetResolver` lands in the hit-test crate or a still-lower widget-tree crate. Design phase picks; any choice satisfying the paint-free + single-shared-resolver constraints is acceptable (`AGENTS.md` § *API Stability* — pre-publish, free to restructure).
