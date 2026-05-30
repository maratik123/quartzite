# Design: Hit-testing traversal in reverse z-order

**Issue:** #395
**Date:** 2026-05-30

## Approach

Add a paint-free `hit_test` free function that is the structural inverse of
`dispatch_paint` (`quartzite-style-dispatch/src/dispatch.rs`). It walks the same
`WidgetResolver` + `AsWidget::children()` tree, applies the same
visibility-skips-subtree and `children_clip_rect()` rules, and emits the same
resolver-miss `warn!` — but iterates children in **reverse** and tests
**child-before-parent**, so the visually-topmost (last-painted) widget wins the
hit. It returns `Option<(ObjectId, Point)>`: the hit id plus the accumulated
origin offset from root-local space down to the hit widget (pinned § Key
decisions Q2).

### Crate placement (resolves Q1 + the "final crate name" open question)

The spec pins a **new paint-free crate** and asks the design to (a) pick the
final name and (b) decide where the shared `WidgetResolver` trait lands.

- **Name:** `quartzite-hit-test`. Chosen over `quartzite-widget-tree` because the
  crate's sole public surface in v1 is the hit-test function; the resolver is a
  shared dependency, not the crate's headline purpose. If a broader read-only
  tree-walk crate is later wanted, the resolver can move down again (pre-publish,
  free to restructure per `AGENTS.md` § *API Stability*).
- **Resolver home:** the `WidgetResolver` trait + its blanket
  `impl<F: Fn(...) -> Option<&'static dyn AsWidget>>` **move into
  `quartzite-hit-test`**, not into a still-lower crate. Rationale: a separate
  lower crate would be a one-trait crate (YAGNI — counter to the over-splitting
  rule in `AGENTS.md` § *Code Style → File size*). `quartzite-style-dispatch`
  gains a dependency on `quartzite-hit-test` and **re-exports** `WidgetResolver`
  (`pub use quartzite_hit_test::WidgetResolver;`) so `dispatch_paint`'s public
  signature and every existing import path (`quartzite_style_dispatch::WidgetResolver`,
  `quartzite::style_dispatch::WidgetResolver`) stay valid.

This yields the dep cone: `quartzite-hit-test` → `quartzite-widgets`,
`quartzite-geometry`, `quartzite-core`, `tracing`; and
`quartzite-style-dispatch` → `quartzite-hit-test` (+ its existing paint deps).
The hit-test crate touches **no** `Painter`/`Style`/`StyleRegistry`/`Palette`.

### Traversal shape (the inverse of `visit`)

A recursive helper `find_hit(id, point, resolver) -> Option<(ObjectId, Point)>`:

1. `resolver.resolve(id)` → `None`: `warn!` (same message shape) and return `None`.
2. `!Visible`: return `None` (subtree skipped — invisible hides children too).
3. Compute `clip = widget.children_clip_rect()`. Children are only candidates
   when `clip.is_none() || clip.unwrap().contains(point)` (the clip rect is in
   the clipping node's own local space — the space `point` is in when the node's
   children are gated; e.g. a `ScrollArea` clip = `(0,0,200,150)`).
4. If children are exposed, iterate `widget.children()` in **reverse** order. For
   each child: `child_point = point - child.geometry().origin()`; recurse. The
   first child (in reverse) that returns `Some((hit, offset))` wins — return
   `Some((hit, child_origin + offset))` (accumulate the offset upward).
5. No child claimed the hit: if `widget.geometry()` (this node's geometry is
   relative to *its* parent, so at this recursion level the node occupies
   `Rect::new(Point::ZERO=origin-subtracted-already... )`) — **see the coordinate
   note below** — contains `point`, return `Some((id, Point::default()))`; else
   `None`.

**Coordinate note (the one subtlety vs. `dispatch_paint`):** `dispatch_paint`
translates the painter by `child.geometry().origin()` *before* recursing, so each
node paints with `(0,0)` at its own top-left and uses `widget.geometry()`
directly only for the child-translate step. For hit-testing, `point` enters
`find_hit` already in the **current node's local space** (root call: point is in
root-local space, AC1 `Point::ZERO` offset). Therefore the node's own membership
test is `Rect::new(Point::default(), widget.geometry().size()).contains(point)`
(local-space rect: origin at zero, the node's size) — **not**
`widget.geometry().contains(point)`, because `geometry()` is parent-relative.
Equivalently and more simply: the membership test at the node is
`point.x() >= 0 && point.y() >= 0 && point inside size`. The child recursion
subtracts `child.geometry().origin()` to move `point` into the child's local
space, mirroring the paint-side translate. This local-space model is what makes
the accumulated-offset return (Q2) fall out naturally: each level adds back the
`child.geometry().origin()` it subtracted.

> Implementation note for the impl subagent: the cleanest formulation keeps a
> single helper that receives `point` already in the node's local space and
> returns the offset *relative to that node*; the root caller adds nothing
> (offset starts at `Point::default()`), satisfying AC1's `Point::ZERO`. AC1b's
> `(Leaf, Point::new(15,25))` = `Inner.origin (10,20) + Leaf.origin (5,5)` is the
> sum the upward accumulation produces.

### Reverse iteration over `WidgetChildren`

`AsWidget::children()` returns `WidgetChildren<'_>` (`Slice` / `Optional` /
`Empty`). `Optional` has ≤1 element and `Empty` has none, so only `Slice` needs
reversing. Reverse-iterate by collecting the `into_iter()` of `WidgetChildren`
into a `SmallVec`/`Vec`/array-on-stack and walking it `.rev()`, OR match the enum
and `.iter().rev()` the underlying slice. Prefer matching to avoid allocation on
the common `Slice` path; `Optional`/`Empty` reverse trivially. (No public API
addition to `WidgetChildren` is required — `IntoIterator` + the existing enum
suffice.)

### Rejected alternatives

- **Resolver in a new lower `quartzite-widget-tree` crate** — rejected: one-trait
  crate, YAGNI; the move can happen later if a real tree-walk crate emerges.
- **Returning an ancestor `Vec<ObjectId>` path** — rejected by Q2; id + single
  accumulated offset is the minimal sufficient result, path recoverable via the
  resolver later.
- **Adding `hit_test` to `quartzite-style-dispatch` itself** — rejected by Q1:
  would keep hit-test coupled to the paint/style dep cone, defeating the
  paint-free goal.
- **Keeping `WidgetResolver` in `quartzite-style-dispatch` and depending
  upward** — rejected: would make `quartzite-hit-test` depend on the paint
  bridge (circular intent), inverting the desired dep direction.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `quartzite-hit-test` crate skeleton: `Cargo.toml` (deps `quartzite-core`, `quartzite-geometry` `default-features = false`, `quartzite-widgets`, `tracing`; dev-deps `quartzite-test-helpers`, `tracing-test`; `[lints] workspace = true`; docs.rs metadata) + empty `src/lib.rs` (`//!` crate doc); add `"quartzite-hit-test"` to workspace `members`. | `quartzite-hit-test/Cargo.toml` (new), `quartzite-hit-test/src/lib.rs` (new), `Cargo.toml` (workspace) | — |
| 2 | **Move** `WidgetResolver` trait + blanket `impl<F>` + their docs from `quartzite-style-dispatch/src/dispatch.rs` into `quartzite-hit-test/src/resolver.rs`; `pub use` from `quartzite-hit-test/src/lib.rs`. **When moving the trait's doc example, rewrite its import** from `use quartzite_style_dispatch::WidgetResolver;` (dispatch.rs:28) to `use quartzite_hit_test::WidgetResolver;` — moved verbatim it would import the resolver from the crate that depends on hit-test, creating a dev-dependency back-edge. Keep the intra-doc link `[quartzite_widgets::layout::WidgetResolver]` unchanged (still resolves since `quartzite-widgets` is a dep). In `quartzite-style-dispatch`: add `quartzite-hit-test` dep, delete the local trait, add `pub use quartzite_hit_test::WidgetResolver;` in `dispatch.rs`/`lib.rs` so the existing `pub use dispatch::{WidgetResolver, dispatch_paint};` and all downstream import paths stay valid. | `quartzite-hit-test/src/resolver.rs` (new), `quartzite-hit-test/src/lib.rs`, `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-style-dispatch/src/lib.rs`, `quartzite-style-dispatch/Cargo.toml` | 1 |
| 3 | Implement `hit_test` free fn + private `find_hit` recursive helper in `quartzite-hit-test/src/hit_test.rs` (local-space coordinate model, reverse child iteration, child-before-parent test, visibility-skips-subtree, clip gate, resolver-miss `warn!`, accumulated-offset return, `debug_span!` guard, `/// # Examples` doc-test). `pub use` from `lib.rs`. | `quartzite-hit-test/src/hit_test.rs` (new), `quartzite-hit-test/src/lib.rs` | 2 |
| 4 | `#[cfg(test)] mod tests` in `hit_test.rs` covering AC1–AC8 with a `StubResolver` (HashMap-backed, ported paint-free from `dispatch.rs`); `tracing_test::traced_test` for AC7. | `quartzite-hit-test/src/hit_test.rs` | 3 |
| 5 | Workspace-gate verification + optional umbrella wiring: confirm `cargo build`/`test`/`clippy --workspace --all-targets`/doc gate clean across the moved trait + new crate; decide + (if chosen) add umbrella re-export. See § Risks. | `src/lib.rs` (umbrella, optional), `Cargo.toml` (umbrella, optional) | 4 |

## Handoff plan

`M = 5` → two groups (3 + 2). Grouping is mandatory for every `M ≥ 1`;
non-terminal groups MUST be exactly 3 consecutive subtasks; the terminal group
may hold 1..=3.

- **Entry into Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) before
  starting subtask 1.
- **Group A:** subtasks 1–3 — crate skeleton, resolver move + re-export,
  `hit_test`/`find_hit` implementation (3 subtasks; non-terminal cap met).
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — test module (AC1–AC8) and the workspace-gate +
  optional umbrella-wiring close-out — terminal group (2 subtasks; within the
  1..=3 range).

## Risks

- **The resolver MOVE breaks downstream imports if the re-export is missed** —
  `quartzite_style_dispatch::WidgetResolver` is consumed by
  `quartzite-style/tests/snapshots.rs:28`, the umbrella `src/lib.rs:413`
  (`crate::style_dispatch::WidgetResolver`), and the umbrella doc-comment at
  `src/lib.rs:340`. Mitigation: subtask 2 adds `pub use
  quartzite_hit_test::WidgetResolver;` so the existing
  `pub use dispatch::{WidgetResolver, dispatch_paint};` continues to resolve;
  the umbrella `pub use quartzite_style_dispatch::*;` then keeps re-exporting it
  unchanged. AC10 `--workspace` gates catch any missed path. **Note:** the
  `layout::WidgetResolver` in `quartzite-widgets/src/layout/mod.rs` is a
  *different* trait (`resolve_widget_mut`) — it must NOT be touched; verify no
  cross-wiring during the move.
- **`-D warnings` doc gate aborts on first failure, masking later ones** — the
  moved trait docs + new crate docs + new fn `# Examples` are all new
  `missing-docs`/intra-doc-link surfaces. After the enumerated items pass, re-run
  `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace
  --all-features` to surface any same-class doc error the gate hid behind the
  first. Surface any newly-revealed out-of-contract item to the orchestrator.
- **Coordinate-model inversion is the bug-prone spot** — using
  `widget.geometry().contains(point)` (parent-relative) at the node membership
  test instead of the local-space `size`-rect test silently passes AC1 (root at
  origin 0,0) but fails AC1b/AC4 (nested origins). Mitigation: AC1b + AC4 are
  explicit regression tests with non-zero origins; the § Approach coordinate note
  pins the correct formulation.
- **Reverse iteration allocation** — collecting children to reverse them on a hot
  path would be wasteful, but hit-test is a per-input-event call (not per-frame),
  and `debug_span!`-lifecycle-level per the spec; matching the `Slice` variant and
  `.iter().rev()` avoids allocation entirely. No `large_stack_arrays` concern
  (no fixed array).
- **Umbrella re-export scope creep** — exposing `hit_test` through the umbrella
  `quartzite` crate needs a feature flag decision. Mitigation: keep subtask 5's
  umbrella wiring OPTIONAL. The spec's AC10 only requires the new crate
  participate in `--workspace` gates and appear in `members` (both done by
  subtask 1); a umbrella `quartzite::hit_test` re-export is a nicety, not an AC.
  If added, gate it behind the existing `widgets` feature (hit-test needs only
  widgets+geometry+core, no style) — but flag to the orchestrator that this
  introduces a new public surface on the facade and may warrant its own feature;
  default to NOT wiring the umbrella unless the orchestrator confirms.

## Test Design

All tests live in `quartzite-hit-test/src/hit_test.rs` `#[cfg(test)] mod tests`
(plus the doc-test for AC9). Fixture: `StubResolver(HashMap<ObjectId, Box<dyn
AsWidget>>)` ported paint-free from `dispatch.rs` (no `RecordingPainter` /
`MarkStyle` / `StyleRegistry` — hit-test is paint-free, so no `test_lock()`
style-registry serialization is needed either, unlike the dispatch tests).

- **AC1 — single root** (`hit_test`): visible root with geometry containing
  point → `Some((root, Point::default()))`; point outside root → `None`. Fixture:
  one `Container`/`WidgetBase` with set geometry.
- **AC1b — accumulated offset** (`find_hit` via `hit_test`):
  `Container(0,0){ Inner(10,20){ Leaf(5,5) } }`, point inside `Leaf` →
  `(Leaf, Point::new(15,25))`; assert `point - offset` lands in Leaf-local space.
- **AC2 — reverse z-order**: two siblings at the **same** origin/geometry; point
  inside both → the **second-iterated** sibling wins. Edge: equal geometry is the
  tie-break exercise.
- **AC3 — child-before-parent**: point inside a child's geometry → child (or
  descendant), never parent; point on parent chrome outside every child →
  parent. Edge: point exactly on a child/parent boundary (membership is
  inclusive-left/top, exclusive-right/bottom per `Rect::contains`).
- **AC4 — coordinate transform**: `Container(0,0){ Label(10,20, 50×20) }`; point
  `(15,25)` → `Label`; point `(5,5)` → `Container`. Guards the local-space vs
  parent-relative bug.
- **AC5 — visibility**: hidden root → `None`; hidden non-root child + subtree
  never returned even with point inside its geometry → parent is the hit (or
  `None`). Edge: hidden parent with visible child — whole subtree skipped.
- **AC6 — clip**: `ScrollArea` (or a hand-written `ClippingWidget` à la
  `dispatch.rs:921` returning `Some(clip)` from `children_clip_rect()`); point
  outside `clip` but inside a child's geometry → does NOT hit child (hits
  clipping widget or `None`); point inside both → hits child.
- **AC7 — resolver-miss** (`tracing_test::traced_test`): child id resolves to
  `None` → that subtree yields no hit + `warn!` fires (assert
  `logs_contain(...)`); parent + sibling subtrees still hit-test. `None` root →
  `None` + `warn!`.
- **AC8 — empty miss**: point with no containing widget anywhere in a visible
  tree → `None`, no panic.
- **AC9 — doc-test**: runnable `# Examples` on `hit_test`: build a fixture tree,
  call `hit_test`, assert `(ObjectId, Point)`. Must compile under the doc gate
  and run under `cargo test`. Mirror the `MapResolver` doc-example shape from
  `quartzite-style-dispatch/src/lib.rs`.

Helpers needed: a small `fn leaf(geom: Rect) -> Label` (or `WidgetBase`) +
`fn container(children, geom) -> Container` builder pattern matching the
`dispatch.rs` test style; `StubResolver::insert`.

## Open questions

- **`Point::ZERO` does not exist in `quartzite-geometry`.** AC1 / the spec write
  `Some((root, Point::ZERO))`, but `Point` has no `ZERO` associated const — it
  derives `Default` (`(0,0)`) and has `Point::new`. The impl will use
  `Point::default()` (or `Point::new(0,0)`). Flagging because the AC text reads as
  if `Point::ZERO` exists; behaviour is identical, only the literal differs. (No
  blocker — could optionally add a `pub const ZERO` to `Point` as a tidy-up, but
  that is out of this spec's scope; recommend `Point::default()`.)
- **Umbrella facade exposure of `hit_test`** — should `quartzite` re-export a
  `hit_test` module (and under which feature)? Not required by any AC; subtask 5
  treats it as optional pending orchestrator confirmation (see § Risks).
