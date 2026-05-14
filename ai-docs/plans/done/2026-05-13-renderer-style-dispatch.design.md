# Design: Renderer-side dispatch — `Style::draw_widget` across the widget tree

**Issue:** #312
**Spec:** [`2026-05-13-renderer-style-dispatch.spec.md`](./2026-05-13-renderer-style-dispatch.spec.md)
**Date:** 2026-05-14

## Approach

Add a new workspace bridge crate, **`quartzite-style-dispatch`**, that exposes one public free function and one public trait:

```rust
pub trait WidgetResolver {
    fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>;
}

pub fn dispatch_paint(
    root: ObjectId,
    resolver: &dyn WidgetResolver,
    painter: &mut dyn Painter,
    palette: &Palette,
);
```

`dispatch_paint` is the only entry point. It resolves the active `Style` from `quartzite_style::StyleRegistry::try_style()` once at the start of the call (no-op if `None`), then walks the subtree rooted at `root` depth-first using `WidgetResolver::resolve`. For every **visible** widget reached, it invokes `style.draw_widget(widget, painter, palette)` exactly once; before recursing into each **non-root** child it issues `painter.save() → painter.translate(child.geometry().origin()) → … → painter.restore()` so each child's `draw_widget` sees a painter whose origin is the widget's top-left. The root is painted at the painter's incoming origin (no save/translate/restore wrap around the root itself). Per-type child enumeration is a downcast chain that mirrors `DefaultStyle::draw_widget`'s shape: `Container` → `Container::children()`, `ScrollArea` → `ScrollArea::content_widget`. Other widget types are leaves (no children, no recursion). Resolver misses on a non-root id and a hidden subtree share a silent-skip code path, with an extra `tracing::warn!` on the resolver-miss branch.

### Why `quartzite-style-dispatch` (and not `quartzite-renderer-style`)

Two reasons.

1. **The crate has zero dependency on `quartzite-renderer`.** Naming it `quartzite-renderer-style` would imply renderer-coupling that does not exist; § *Technical constraints* of the spec explicitly forbids that dep. `quartzite-style-dispatch` describes what the crate does (dispatches `Style` calls across a widget tree) without referencing the renderer.
2. **Conceptual symmetry with the existing crates.** The workspace already has the noun-pair `quartzite-style-types` (leaf vocabulary) → `quartzite-style` (trait + registry). Adding `quartzite-style-dispatch` (per-tree driver) extends that chain in the obvious direction without inventing a new naming axis. `quartzite-renderer-style` would put the new crate in the "renderer + X" axis (currently empty) and obscure its position in the style stack.

`quartzite-renderer-style` was the other candidate; both compile and satisfy the dep constraint. The rejection above is on naming clarity, not on viability.

### Why a `WidgetResolver` trait (not `&ObjectTree` or `Fn`-only)

The spec § *Key decisions* row Q2 pins this: the bridge crate **does not** depend on `quartzite-runtime`'s `ObjectTree` — it depends on `quartzite-runtime` solely for the `ObjectId` type. The caller owns the tree shape; the bridge crate only needs to map ids to widgets. A trait gives a clean object-safe `&dyn WidgetResolver` parameter without forcing every caller into a closure shape, and lets test fixtures (`HashMap<ObjectId, Box<dyn AsWidget>>`) implement it directly without boxing closures.

A closure-shaped resolver remains ergonomic via a blanket impl (Q3 open question; spec defaults to "yes"):

```rust
impl<F> WidgetResolver for F
where
    F: Fn(ObjectId) -> Option<&dyn AsWidget>,  // see lifetime note below
{
    fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> { self(id) }
}
```

> **Lifetime caveat.** A `Fn(ObjectId) -> Option<&dyn AsWidget>` closure cannot easily return a borrow whose lifetime is tied to `self` (the higher-ranked-trait-bound form `for<'a> Fn(ObjectId) -> Option<&'a dyn AsWidget>` requires the returned ref to outlive every `'a`, which doesn't fit a backing-store lookup). In practice, callers either implement `WidgetResolver` directly on their tree wrapper (the recommended shape; lifetimes are straightforward) or use a closure that captures a `&'static`-equivalent reference. **Decision:** ship the blanket impl with the simple `Fn(ObjectId) -> Option<&dyn AsWidget>` bound and document that callers wanting a borrow-from-self lookup should implement the trait directly. If the blanket impl turns out unusable in practice, it can be removed without a public-API break (additive removal of a blanket impl is a breaking change in general, but this crate is pre-publish per `AGENTS.md` § *API Stability*, so clean removal is allowed). A test in the bridge crate exercises a closure-shaped resolver against a fixture tree to prove the blanket impl is at least usable for the simple case.

### Name collision with `quartzite_widgets::WidgetResolver`

`quartzite-widgets::layout` already exports a `WidgetResolver` trait, but with a **different** shape:

```rust
// quartzite-widgets::layout
pub trait WidgetResolver {
    fn resolve_widget_mut(&mut self, id: ObjectId) -> Option<&mut WidgetBase>;
}
```

vs the new bridge-crate trait:

```rust
// quartzite-style-dispatch
pub trait WidgetResolver {
    fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>;
}
```

The two coexist in distinct crates — name collision only manifests for a caller that imports both. The spec uses `WidgetResolver` as the bridge-crate trait name explicitly (§ *Scope*, AC1); a rename would require a spec change. Callers that import both crates' traits will path-qualify, identically to how `quartzite_core::ObjectId` and `quartzite_runtime::ObjectTree` already coexist in user code. The new trait's documentation cross-links to `quartzite_widgets::WidgetResolver` and explains the difference (mutable layout-time vs. immutable paint-time).

### Traversal algorithm

`dispatch_paint` shape:

```text
fn dispatch_paint(root, resolver, painter, palette):
    let style: &'static dyn Style = StyleRegistry::try_style()?;  // None -> early return (no-op)
    visit(root, resolver, painter, palette, style, /*is_root=*/ true)

fn visit(id, resolver, painter, palette, style: &'static dyn Style, is_root):
    let widget = match resolver.resolve(id) {
        Some(w) => w,
        None    => { tracing::warn!(?id, "dispatch_paint: resolver miss"); return; }
    };
    if !widget.is_visible() { return; }                 // skip entire subtree

    style.draw_widget(widget, painter, palette);        // parent-before-child

    for &child_id in children_of(widget) {              // per-type enumeration (below)
        // Look up the child to read its origin BEFORE the save/translate pair.
        // The child's geometry is in its parent's coordinate space, which is
        // exactly what the painter's current transform expects.
        let Some(child) = resolver.resolve(child_id) else {
            tracing::warn!(?child_id, "dispatch_paint: resolver miss");
            continue;
        };
        if !child.is_visible() { continue; }

        let origin = child.geometry().origin();
        painter.save();
        painter.translate(origin);
        // Recurse with is_root = false. The child has already been resolved +
        // visibility-checked here, so the recursive call's resolve/visibility
        // will succeed by construction — but the call site is the same code
        // path for uniformity and so the resolver-miss / hidden-subtree
        // logging stays at one place.
        visit(child_id, resolver, painter, palette, style, /*is_root=*/ false);
        painter.restore();
    }
```

A subtle point: the child must be **resolved and visibility-checked once** to know its `geometry().origin()` before issuing the save/translate pair, because if the child is invisible we want to skip the save/translate entirely (zero painter calls for an invisible subtree per AC3). The recursive call into `visit` will resolve the same child a second time — that is acceptable: `WidgetResolver::resolve` is by contract a cheap lookup (e.g. `HashMap` get), and avoiding the double-lookup would require restructuring the recursion to pass `&dyn AsWidget` instead of `ObjectId` (and thereby leak per-call lookup state into the recursion shape, which is uglier than one extra hash hit per child). Tests assert AC2/AC3 (zero paints for hidden subtrees) against this shape directly.

Alternative form considered: hoist the resolve+visibility check out of `visit` so the recursive call receives `(&dyn AsWidget, ObjectId)` directly. Rejected because the resolver-miss path on the **root** needs the same shape as the resolver-miss path on a child (per AC10's second test: `None` root → zero paints + `warn`). Keeping the resolve at the top of `visit` makes the root and non-root paths share one branch.

### Per-type child enumeration

A downcast chain inside a private helper `fn children_of(widget: &dyn AsWidget) -> &[ObjectId]` (or an iterator-shaped equivalent; see below) mirrors `DefaultStyle::draw_widget`'s router:

```text
fn children_of(widget: &dyn AsWidget) -> ChildIds<'_> {
    let any = widget.as_any();
    if let Some(c) = any.downcast_ref::<Container>()  { return ChildIds::Slice(c.children()); }
    if let Some(s) = any.downcast_ref::<ScrollArea>() {
        return ChildIds::Optional(s.content_widget);  // Option<ObjectId>
    }
    ChildIds::Empty
}
```

Two return shapes (`&[ObjectId]` for `Container`, `Option<ObjectId>` for `ScrollArea`) collapse to one enum so the caller iterates uniformly:

```rust
enum ChildIds<'a> {
    Slice(&'a [ObjectId]),
    Optional(Option<ObjectId>),
    Empty,
}
impl<'a> IntoIterator for ChildIds<'a> { /* yields ObjectId */ }
```

Concrete iterator shape can be a hand-rolled `enum`-backed `Iterator` or a `Box<dyn Iterator>`. The hand-rolled `enum` is cheap and avoids the allocation, so that's the planned shape. Unknown widget types fall through to `ChildIds::Empty`, matching the spec's "unknown widget → contributes no children" silent fall-through (AC7).

Rejected alternative: add a generic `AsWidget::children()` trait method. Out of v1 scope per spec § *Out of scope*; the downcast-chain mirrors `DefaultStyle` and keeps the bridge crate self-contained.

### Crate layout

`quartzite-style-dispatch/`:

```text
Cargo.toml
src/
  lib.rs        (crate-level docs + re-exports)
  dispatch.rs   (WidgetResolver trait + dispatch_paint free fn + ChildIds helper)
tests/                       (optional — only if an integration-shaped test is needed)
```

`src/lib.rs` mirrors the existing `quartzite-style/src/lib.rs` shape: crate doc comment with a runnable `# Examples` block satisfying AC9's doc-test gate, `#![deny(missing_docs)]` + the rest of the `clippy::*` and `rustdoc::*` lints used by sibling crates, then `mod dispatch;` + `pub use dispatch::{WidgetResolver, dispatch_paint};`.

`src/dispatch.rs` (target size ~250–350 lines including `#[cfg(test)]` block; well under the 400-line soft cap from `AGENTS.md` § *Code Style → File size*):

- `pub trait WidgetResolver { fn resolve(…) -> Option<&dyn AsWidget>; }`
- Blanket `impl<F> WidgetResolver for F where F: Fn(ObjectId) -> Option<&dyn AsWidget>` (per Q3 open question — design's recommendation: ship it with the lifetime caveat documented).
- `pub fn dispatch_paint(root, resolver, painter, palette)` — body delegates to the private `visit` recursion.
- Private `fn visit(id, resolver, painter, palette, style, is_root)`.
- Private `fn children_of(widget) -> ChildIds<'_>` + the `ChildIds` enum + its `IntoIterator` impl.
- `#[cfg(test)]` module with a `RecordingPainter` + a `StubResolver` fixture (HashMap-backed) + ~10 unit tests covering every AC.

### Tracing instrumentation

Per `AGENTS.md` § *Code Style → Tracing*, `dispatch_paint` "meaningfully mutates application state" only via the painter; the function is a pure dispatcher. A `debug_span!("style_dispatch::dispatch_paint", root = ?root)` guard wraps the body. The resolver-miss `warn!` events fire inside the recursion. No high-frequency gating needed (one span per frame is not hot; the per-widget paint events are already inside the painter's own span if it has one). `tracing` is a transitive dep via `quartzite-runtime` — explicit `tracing` entry in the bridge crate's `Cargo.toml` is required (don't rely on transitive deps for direct use).

### Cargo.toml (full)

```toml
[package]
name = "quartzite-style-dispatch"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Widget-tree paint dispatch bridging quartzite-widgets and quartzite-style"

[dependencies]
quartzite-core       = { path = "../quartzite-core" }
quartzite-runtime    = { path = "../quartzite-runtime" }
quartzite-paint-api  = { path = "../quartzite-paint-api" }
quartzite-style      = { path = "../quartzite-style" }
quartzite-widgets    = { path = "../quartzite-widgets" }
tracing              = "0.1"

[dev-dependencies]
quartzite-geometry   = { path = "../quartzite-geometry" }

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
rustc-args = ["--cfg", "docsrs"]
```

`quartzite-runtime` is included for `ObjectId` (re-exported from `quartzite-core`, but the spec § *Technical constraints* names `quartzite-runtime` as the canonical home — the workspace conventionally takes `ObjectId` from `quartzite-runtime` per `quartzite-widgets/Cargo.toml`'s comments; here both paths exist and either dep would satisfy the use, so we follow the spec wording). Actually, double-check: `quartzite-core/src/lib.rs:68` re-exports `ObjectId` from `id::ObjectId`, and `quartzite-runtime/src/lib.rs` itself does **not** re-export `ObjectId`; the runtime depends on core. To keep the dependency cone minimal, **the design uses `quartzite_core::ObjectId` (path via `quartzite-core`, not `quartzite-runtime`)**. The spec's "quartzite-runtime (for ObjectId)" wording is a slight imprecision worth flagging in the design (see § *Open questions*); the bridge crate functionally needs only `quartzite-core` for the id type, not `quartzite-runtime`. We drop `quartzite-runtime` from `[dependencies]` unless something else surfaces. **Revised `[dependencies]`:**

```toml
[dependencies]
quartzite-core       = { path = "../quartzite-core" }
quartzite-paint-api  = { path = "../quartzite-paint-api" }
quartzite-style      = { path = "../quartzite-style" }
quartzite-widgets    = { path = "../quartzite-widgets" }
tracing              = "0.1"
```

Note: the spec also mentions `quartzite-core (for AsObject)`. `AsObject` is re-exported by `quartzite-core` and consumed transitively by `AsWidget` (the supertrait chain). The bridge crate touches `AsObject` only through the `as_any()` method on `&dyn AsWidget` — which is available because `AsWidget: AsObject`. No explicit `use AsObject` is needed in the bridge crate; the `quartzite-core` dep stays in for `ObjectId` regardless.

`tracing` version: query `crates.io` per `AGENTS.md` § *Dependency Versions* AXIOM before committing. The placeholder `"0.1"` matches the major already pinned across the workspace (`quartzite-runtime`, `quartzite-core` use `tracing = "0.1"`); implementation phase will re-verify the live `max_stable_version` and write the observed major into the `Cargo.toml`.

### Workspace registration

`Cargo.toml` (workspace root):

```toml
[workspace]
members = [
    "quartzite-core",
    "quartzite-macros",
    "quartzite-runtime",
    "quartzite-geometry",
    "quartzite-events",
    "quartzite-event-types",
    "quartzite-paint-api",
    "quartzite-paint",
    "quartzite-renderer",
    "quartzite-style-types",
    "quartzite-style",
    "quartzite-style-dispatch",       # new
    "quartzite-widgets",
]
```

The `quartzite-style-dispatch` slot is placed after `quartzite-style` to keep the style-stack contiguous; the existing list is roughly dependency-order so the new entry slots in naturally above `quartzite-widgets` (the bridge crate depends on widgets; both kinds of ordering — alphabetical and dep-order — work; the workspace already isn't strictly alphabetical, so the dep-order placement matches the existing convention).

The facade crate (`quartzite/Cargo.toml`) is **not** modified by this task. The bridge crate is a separate crate consumed by application binaries that want renderer-side dispatch; whether and when the facade re-exports it is a follow-up (the facade's `style` feature today gates `quartzite-style`; a future `style-dispatch` feature could gate the new crate). Tracked in § *Open questions*.

### Rejected alternatives

1. **Default-impl method on `WidgetRoot` (`fn paint_tree(&self, …) { … }`)** — would force `quartzite-renderer` (the only crate currently defining `WidgetRoot`) to depend on `quartzite-style` + `quartzite-widgets`, exactly the cycle the bridge crate exists to avoid. Rejected by spec § *Key decisions* Q3, restated here.
2. **New concrete `WidgetRoot`-implementing type in the bridge crate** — would force the bridge crate to depend on `quartzite-renderer` for the `WidgetRoot` trait. Rejected by spec § *Key decisions* Q3.
3. **Take `&ObjectTree` directly** — couples the bridge crate to `quartzite-runtime::ObjectTree`, making it untestable without booting a full runtime. The `WidgetResolver` trait keeps the bridge crate testable in isolation against a `HashMap`-backed stub. Rejected by spec § *Key decisions* Q2.
4. **Closure-only API (`F: Fn(ObjectId) -> Option<&dyn AsWidget>`)** — closures can't easily return `&self`-bound borrows (HRTB friction documented above). A trait with a blanket impl gives both shapes. Rejected as the only public form.
5. **A generic `AsWidget::children() -> &[ObjectId]` method** — out of v1 scope per spec § *Out of scope* and § *Deferred*. The per-type downcast chain mirrors `DefaultStyle` and is the explicit shape the spec asks for.
6. **Resolve children eagerly into a `Vec<(&dyn AsWidget, Point)>` before recursing** — would mean every dispatch_paint call materialises an intermediate vector at every parent. The recursion shape above visits children lazily, allocating nothing per parent. Rejected on YAGNI / performance grounds.
7. **Reverse-order traversal so children paint after siblings of a higher z-order** — out of v1 scope; the spec pins parent-before-child painter's-algorithm order (AC4). Rejected.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create new bridge crate skeleton: `quartzite-style-dispatch/Cargo.toml` (deps per § *Cargo.toml*; live-query `tracing` version per AGENTS.md AXIOM) + empty `src/lib.rs` with crate-level doc comment, `#![deny(rustdoc::broken_intra_doc_links)]` + sibling lint preamble, `#![deny(missing_docs)]`, `mod dispatch;` placeholder, no `pub use` yet. Add `"quartzite-style-dispatch"` to workspace `members` (Cargo.toml). Verify `cargo build -p quartzite-style-dispatch` succeeds. | `Cargo.toml`, `quartzite-style-dispatch/Cargo.toml`, `quartzite-style-dispatch/src/lib.rs`, `quartzite-style-dispatch/src/dispatch.rs` | — |
| 2 | Define `pub trait WidgetResolver { fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>; }` in `dispatch.rs` with full `///` doc + `# Examples` block (no-op fixture impl). Re-export from `lib.rs` (`pub use dispatch::WidgetResolver;`). | `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-style-dispatch/src/lib.rs` | 1 |
| 3 | Define the private `enum ChildIds<'a>` + its `IntoIterator` impl (`type Item = ObjectId`). Add `fn children_of(widget: &dyn AsWidget) -> ChildIds<'_>` with the downcast chain (Container → Slice, ScrollArea → Optional, fallthrough → Empty). | `quartzite-style-dispatch/src/dispatch.rs` | 2 |
| 4 | TDD test scaffolding: in `#[cfg(test)] mod tests`, define `RecordingPainter` (vec of `PaintEvent`s mirroring `quartzite-style`'s default_style.rs fixture but **including** `Save`/`Restore`/`Translate`) and `StubResolver(HashMap<ObjectId, Box<dyn AsWidget>>)`. Write one failing test that constructs a single `WidgetBase` root and asserts `dispatch_paint` calls `Style::draw_widget` exactly once. (Implementation of `dispatch_paint` not yet present — test fails at link.) | `quartzite-style-dispatch/src/dispatch.rs` | 3 |
| 5 | Add a `test-support` feature to `quartzite-style` that publicly re-exports `StyleRegistry::clear_for_test` under `#[cfg(feature = "test-support")]`. Add `quartzite-style = { path = "../quartzite-style", features = ["test-support"] }` to the bridge crate's `[dev-dependencies]`. This is required for AC8 — `StyleRegistry::try_style()` returns `None` only before any `set_style` call, and `clear_for_test` is `pub(crate)` in `quartzite-style`, so it is unreachable from the bridge crate without this feature gate. | `quartzite-style/src/registry.rs`, `quartzite-style/Cargo.toml`, `quartzite-style-dispatch/Cargo.toml` | 4 |
| 6 | Implement `pub fn dispatch_paint(root, resolver, painter, palette)` with `debug_span!` guard and the early-return-on-`try_style()`-None shape. Add private `fn visit(...)` per the algorithm above. AC1 test from step 4 now passes; add AC8 test (no `Style` set → `StyleRegistry::clear_for_test()` first → zero painter calls, `#[serial]`-gated). | `quartzite-style-dispatch/src/dispatch.rs` | 5 |
| 7 | Implement save/translate/restore around each child (AC5). Add AC5 test (Container with two visible children at distinct origins; assert recorded events are `[Draw(parent), Save, Translate(c1.origin), Draw(c1), Restore, Save, Translate(c2.origin), Draw(c2), Restore]`). | `quartzite-style-dispatch/src/dispatch.rs` | 6 |
| 8 | Implement visibility filter (`!is_visible()` → skip subtree, no paint, no save/translate). Add AC2 test (root hidden → zero events) and AC3 test (non-root Container hidden → parent paints, children skipped, no save/translate around the hidden subtree). | `quartzite-style-dispatch/src/dispatch.rs` | 7 |
| 9 | Wire `Container::children()` enumeration in `children_of`. Add AC4 test (depth-first parent-before-child order on `Container { Label, Container { Button } }`; assert the recorded `Draw` events are `[Container, Label, Container, Button]` interleaved with the correct save/restore pairs). | `quartzite-style-dispatch/src/dispatch.rs` | 8 |
| 10 | Wire `ScrollArea::content_widget` enumeration. Add AC6 test (ScrollArea with `content_widget = None` → 1 paint; with `Some(id)` and visible content → 2 paints). | `quartzite-style-dispatch/src/dispatch.rs` | 9 |
| 11 | Add AC7 test (unknown widget type at the root — a bare `WidgetBase` — yields exactly 1 paint and no recursion). The `children_of` fallthrough already handles this; the test asserts the wired shape. | `quartzite-style-dispatch/src/dispatch.rs` | 10 |
| 12 | Resolver-miss path: emit `tracing::warn!(?id, "dispatch_paint: resolver miss")` on both the root-miss branch and the child-miss branch. Add AC10 tests: (a) mid-tree id missing → subtree skipped, parent + sibling subtree painted; (b) root id missing → zero paints. Use `tracing_test::traced_test` (or a manual `tracing` subscriber installation) to assert the warn event fires. | `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-style-dispatch/Cargo.toml` (add `tracing-test` as dev-dep) | 11 |
| 13 | Add the closure-shaped blanket `impl<F> WidgetResolver for F where F: Fn(ObjectId) -> Option<&dyn AsWidget>`. Add a test (`closure_resolver_compiles_and_works`) that uses a closure resolver against a **`&'static`-backed fixture** (e.g. a leaked `HashMap` via `Box::leak`) — the blanket impl parses as `for<'a> Fn(ObjectId) -> Option<&'a dyn AsWidget>`, so the returned reference must be `'static`. Document this lifetime restriction in the blanket impl's doc comment; recommend implementing `WidgetResolver` directly on any tree wrapper that borrows non-statically. Also add a one-line cross-reference note to `quartzite-widgets/src/layout/mod.rs`'s existing `WidgetResolver` doc comment pointing to the bridge-crate trait and explaining the different shape (mutable layout-time vs. immutable paint-time). | `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-widgets/src/layout/mod.rs` | 12 |
| 14 | Crate-level rustdoc + AC9 doc-test: extend `lib.rs` `# Examples` block with end-to-end wiring (caller defines a `StubResolver` over a fixture tree, calls `dispatch_paint`, captures paint events through a recording painter). Verify the doc-test compiles and runs under `cargo test -p quartzite-style-dispatch --doc`. | `quartzite-style-dispatch/src/lib.rs` | 13 |
| 15 | Run the full doc/lint gate locally: `cargo build`, `cargo test`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. Fix any new findings. (AC11.) | (all changed files) | 14 |

15 tasks total (1 task added: `test-support` feature on `quartzite-style` for AC8 registry-reset access). The spec calls for one crate, one entry point, one trait, and ~10 unit tests; the decomposition is intentionally fine-grained so each step has a single failing test driving the next implementation slice (TDD per `AGENTS.md` § *Workflow*).

> **Scope-size note.** 15 tasks exceeds the `> 7 tasks → propose splitting` threshold in `.claude/agents/design.md` § *Rules*. The split is structural rather than logical: tasks 6–13 are one-AC-one-task slices of the same `dispatch.rs` body, each adding ~30 lines + 1 test. Logically this is one cohesive change (the bridge crate as a single deliverable); splitting it across multiple issues would force inter-issue dependencies on a yet-unmerged crate. Recommendation: keep as one issue, treat tasks 6–13 as TDD increments inside a single PR. If reviewer feedback prefers a split, tasks 1–5 (skeleton + trait + helper + test-support feature) could land as PR-A and tasks 6–15 (implementation + tests + docs) as PR-B.

## Risks

- **`WidgetResolver` name collision with `quartzite_widgets::layout::WidgetResolver`** — *mitigation:* document the difference in the bridge-crate trait's doc comment, cross-link to the widgets one. Callers importing both will path-qualify; this is identical to the existing `quartzite_core::ObjectId` + `quartzite_runtime` situation in the workspace. **No code change in `quartzite-widgets` is needed** (the layout trait is unchanged; only the bridge crate's trait is new).
- **Blanket `impl<F> WidgetResolver for F: Fn(...)` lifetime friction** — closures returning `&self`-bound borrows hit the HRTB form. *Mitigation:* ship the simple blanket impl, document the limitation, and recommend implementing `WidgetResolver` directly on the caller's tree wrapper. If users hit the friction in practice, remove the blanket impl (pre-publish, no compat obligation per AGENTS.md § *API Stability*).
- **`tracing::warn!` on every resolver miss** — could be noisy in a malformed tree. *Mitigation:* warn-and-continue is the spec-mandated shape (§ *Key decisions* row "Resolver-miss policy" + AC10); the alternative (panic / silent drop) is worse. Spans are at `debug_span!`/`warn!` level so they're filterable by users who don't want them.
- **Double-resolve of each child (once to read origin + visibility, once inside the recursion's `visit`)** — *mitigation:* `WidgetResolver::resolve` is documented as a cheap lookup; the double hit is acceptable. If profiling later shows this matters, the recursion can be restructured to pass `&dyn AsWidget` plus the geometry origin into the next call without touching the public API.
- **No `Painter::clip_rect` for `ScrollArea`** — content can paint outside the scroll area's bounds. *Mitigation:* explicitly out of v1 scope per spec § *Out of scope*; § *Deferred* tracks the follow-up.
- **`#[cfg(test)] tracing-test` dev-dep is new to the workspace** — *mitigation:* if it isn't already used elsewhere, query crates.io for the live `max_stable_version` per AGENTS.md AXIOM; alternative is a manual `tracing::subscriber` installation in the test. Either is fine; the test must assert the warn event fires, not just that the function returns.
- **The bridge crate is added to `cargo doc --workspace --all-features`** — it has no features today, so `--all-features` is a no-op for it. Doc gate must still pass; the doc-test inside the crate-level `# Examples` block is the AC9 enforcement. *Mitigation:* AC11 calls out the full gate; task 14 runs it locally before commit.
- **The `quartzite-runtime` dep in the spec was imprecise (`ObjectId` is from `quartzite-core`, not `quartzite-runtime`)** — *mitigation:* design § *Cargo.toml* drops `quartzite-runtime` from `[dependencies]`; `quartzite-core` is the canonical home of `ObjectId`. Surfaced in § *Open questions* in case the spec author intended the runtime dep for a reason not captured.
- **Panic / unsafe surface** — none. The bridge crate adds no `unsafe` blocks, no `unwrap()` outside test fixtures, no `expect(...)`. Resolver misses are `tracing::warn!` events; hidden subtrees and missing styles are silent early returns. Per AGENTS.md § *API Naming*, all new public surface is safe and panic-free.

## Test Design

All tests live in `quartzite-style-dispatch/src/dispatch.rs` under `#[cfg(test)] mod tests`. No integration tests in `tests/` directory — the unit-test shape is sufficient and matches the sibling-crate pattern (`quartzite-style`'s `default_style.rs` uses the same shape).

### Shared fixtures

- **`RecordingPainter`** — `Vec<PaintEvent>` capturing every painter call. Mirrors `quartzite-style/src/default_style.rs`'s `RecordingPainter` but adds `Save`, `Restore`, `Translate(Point)` variants (those exist in the default_style fixture too; copy the shape verbatim). One `PaintEvent::DrawWidgetMarker(ObjectId)` variant is *not* added because the recording happens at the painter level; `Style::draw_widget` is observed via the `Mark`-prefixed fill/draw events that `DefaultStyle` would emit. For tests that want to assert "draw_widget was called for widget X", a custom `MarkStyle` fixture replaces `DefaultStyle` and records the widget's `ObjectId` directly (see below).
- **`MarkStyle`** — a test-only `Style` impl whose `draw_widget` body pushes a `PaintEvent::DrawMark(ObjectId)` into the `RecordingPainter`. Lets tests assert paint **order** independent of any concrete `DefaultStyle` content. Installed via `StyleRegistry::set_style(Box::new(MarkStyle::new()))` inside `#[serial]`-gated tests.
- **`StubResolver(HashMap<ObjectId, Box<dyn AsWidget>>)`** — `impl WidgetResolver` returns `self.0.get(&id).map(|b| &**b as &dyn AsWidget)`. The tests build a small tree (e.g. `Container { Label, Container { Button } }`) by inserting widgets one by one and wiring `Container::add_child` / `ScrollArea::content_widget` manually.

### Scenarios (one test per AC)

| AC | Test name | Tree | Asserts |
|---|---|---|---|
| AC1 | `dispatch_paint_invokes_draw_widget_once_per_visible_widget` | Single visible `WidgetBase` root | exactly 1 `DrawMark(root_id)` event |
| AC2 | `hidden_root_produces_zero_paints` | Root hidden | zero events |
| AC3 | `hidden_subtree_skipped_with_no_save_or_translate` | `Container(visible) { Container(hidden) { Label(visible) } }` | events = `[DrawMark(outer)]`; no `Save`/`Translate`/`Restore`/inner-Container-`DrawMark` |
| AC4 | `depth_first_parent_before_child_order` | `Container { Label, Container { Button } }` | `DrawMark` order = `[Container, Label, Container, Button]` (with appropriate `Save`/`Translate`/`Restore` interleaved) |
| AC5 | `save_translate_restore_wraps_each_non_root_child` | `Container(at 0,0) { Label(at 10,20) }` | events = `[DrawMark(outer), Save, Translate(10,20), DrawMark(label), Restore]`; root has no surrounding save/translate |
| AC6a | `scroll_area_without_content_paints_only_chrome` | `ScrollArea { content_widget = None }` | exactly 1 `DrawMark(scroll_area_id)` |
| AC6b | `scroll_area_with_content_paints_chrome_and_content` | `ScrollArea { content_widget = Some(label_id) }` | exactly 2 `DrawMark` events: `[scroll_area_id, label_id]` (with surrounding save/translate around the label) |
| AC7 | `unknown_widget_type_paints_once_no_recursion` | Bare `WidgetBase` root (no Container / ScrollArea downcast) | exactly 1 `DrawMark`, no save/translate |
| AC8 | `no_style_installed_is_noop` | Single visible root; call `StyleRegistry::clear_for_test()` (re-exported via `test-support` feature on `quartzite-style`) before the call | zero events; `#[serial]` to gate the registry; `clear_for_test` is accessible via the `test-support` dev-dep added in task 5 |
| AC10a | `resolver_miss_mid_tree_skips_subtree_and_warns` | `Container { Label(present), Label(missing-id), Label(present)}` | events = `[outer, first, third]`; `tracing` capture shows one `warn` with `id = missing-id` |
| AC10b | `resolver_miss_on_root_produces_zero_paints_and_warns` | `StubResolver` empty; root id not present | zero events; one `warn` |
| (closure blanket) | `closure_resolver_compiles_and_works` | Same as AC1 but resolver is `|id| ...` closure | exactly 1 `DrawMark` |
| AC9 (doc-test) | crate-level `# Examples` block in `lib.rs` | mirror of AC4 setup | doc-test compiles + runs under `cargo test --doc` |

`StyleRegistry::clear_for_test` is `pub(crate)` inside `quartzite-style` — accessing it from the bridge crate would require either making it `pub` (API surface expansion) or living with the test sharing process state. **Decision:** call `StyleRegistry::set_style(Box::new(MarkStyle::new()))` at the top of each `#[serial]` test that needs a known style installed; for AC8 (`try_style()` returns `None`), install a `MarkStyle`, run the test, then install a `NoopStyle` (sentinel) — but a cleaner shape is to **install `MarkStyle` only inside the tests that need it** and let AC8 run **first** in a fresh process or `#[serial]` with explicit ordering. The cleanest path: AC8 doesn't actually need `StyleRegistry::try_style()` to be `None` from a clean state — instead, it can use a fresh in-process state by gating on `#[serial]` and ensuring the test runs before any `set_style` call (the registry starts at `None`). Tests are `#[serial]`-gated regardless; if test order matters, use a single composite `#[test]` covering both pre-set and post-set behaviour.

Actually, re-reading: `clear_for_test` could be re-exported under a `#[cfg(test)]` path or under a `test-support` feature. **Recommendation:** add a `pub fn clear_for_test()` to `quartzite-style::StyleRegistry` gated behind a `test-support` Cargo feature on `quartzite-style`. The bridge crate's `[dev-dependencies]` enables that feature. This keeps the production API of `quartzite-style` unchanged while letting bridge-crate tests get a fresh registry. **This is a small adjacent change to `quartzite-style`** and worth flagging as part of the design — added as task 5b below if reviewer wants it; default plan keeps `clear_for_test` `pub(crate)` and works around it via `MarkStyle` installation per test (cheapest path).

### Fixtures / helpers

- `fn build_tree() -> (StubResolver, ObjectId)` — single helper that constructs the canonical AC4 tree (`Container { Label, Container { Button } }`) and returns the resolver + the root id. Reused by AC3, AC4, AC5, AC10 tests with minor mutations (hide a node, drop a node from the map).
- `fn install_mark_style() -> ()` — installs `MarkStyle` into `StyleRegistry`. Idempotent on subsequent calls.
- `fn drain_events(painter: &mut RecordingPainter) -> Vec<PaintEvent>` — convenience for `std::mem::take(&mut painter.events)`.
- For the closure-resolver test: a `HashMap` captured in a closure makes the lifetime trivial (the closure borrows the map).

### Tracing capture

The cleanest shape is `tracing_test::traced_test` (crates.io: live-verify the major during implementation). It attaches a subscriber for the test's duration and exposes a `logs_contain(&str)` helper. Alternative: implement a tiny `tracing::Subscriber` inside the test module that appends events to a `Mutex<Vec<...>>` and assert against it manually. **Recommendation:** use `tracing-test` if its live major is compatible with `tracing = 0.1.*`; otherwise hand-roll. Either way the `warn` events should be observable.

## Open questions

- **Final crate name** — design picks `quartzite-style-dispatch` per § *Approach*. Spec § *Open questions* row 1 lists both candidates; the reasoning above (no renderer dep, conceptual symmetry) selects the style-dispatch variant. Confirm with reviewer before locking in.
- **Closure blanket impl** — design ships it with the documented HRTB-lifetime caveat. Spec § *Open questions* row 2 marks the sensible default as "yes". Confirm.
- **`quartzite-runtime` dep listed in spec § *Technical constraints***  — the actual need is for `ObjectId`, which lives in `quartzite-core`. Design drops `quartzite-runtime` from the bridge crate's `[dependencies]`. Spec author should confirm the wording wasn't tracking a separate dependency need. (If runtime is intentionally pinned, e.g., to keep id semantics consistent, add it back; otherwise the smaller dep cone is preferable.)
- **`StyleRegistry::clear_for_test` access from outside `quartzite-style`** — design works around it via per-test `MarkStyle` installation. If reviewer prefers the cleaner shape, add a `test-support` feature to `quartzite-style` that re-exports `clear_for_test` publicly under `#[cfg(feature = "test-support")]`. Not in the primary decomposition; surfaced here.
- **`tracing-test` as a dev-dep** — query crates.io for the live `max_stable_version` during task 11. If the live major is incompatible with `tracing = 0.1.*`, hand-roll the subscriber. Either path satisfies AC10's assertion requirement.
- **Facade `style-dispatch` feature in `quartzite/Cargo.toml`** — out of scope for this task; tracked as a future plan. The bridge crate is consumable directly via `[dependencies]` `quartzite-style-dispatch = { path = ... }` without facade re-exports.
- **Whether the `Painter` `save`/`translate`/`restore` triplet should be hoisted into a `transformed_subpaint(...)` helper closure parameter** — design uses inline `save`/`translate`/`restore` calls; the alternative ("RAII guard" struct holding `&mut dyn Painter` and `Drop`-impl'ing `restore`) is rejected on YAGNI but viable later if more bridge crates need the same shape.
