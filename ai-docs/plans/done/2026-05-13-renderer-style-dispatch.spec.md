# Renderer-side dispatch — Style::draw_widget across the widget tree

**Source:** issue #312
**Date:** 2026-05-13
**Tracked in:** #312

> Surfaced by `/triage` from [`ai-docs/deferred/widget-backlog.md`](../deferred/widget-backlog.md). Source spec: [`2026-05-13-default-style-content.spec.md`](done/2026-05-13-default-style-content.spec.md). With `DefaultStyle::draw_widget` already painting a single widget node, this task adds a free-function widget-tree paint dispatcher in a new bridge crate. The dispatcher walks the tree, applies coordinate transforms, and invokes `Style::draw_widget` once per visible widget.

## Scope

Add a per-frame widget-tree traversal helper to a **new bridge crate** (provisional name `quartzite-renderer-style` or `quartzite-style-dispatch`; final name pinned in design — see § *Key decisions*) that:

- Exposes a single public **free function** with the signature:

  ```rust
  pub fn dispatch_paint(
      root: ObjectId,
      resolver: &dyn WidgetResolver,
      painter: &mut dyn Painter,
      palette: &Palette,
  )
  ```

  Callers invoke `dispatch_paint` from inside their own `WidgetRoot::paint` (or equivalent entry point); the bridge crate does **not** define a new `WidgetRoot`-implementing type. The bridge crate does **not** depend on `quartzite-renderer`.

- Walks a widget subtree rooted at the caller-supplied `root: ObjectId` via a **`WidgetResolver` trait** that maps `ObjectId` → `Option<&dyn AsWidget>`. The helper itself does **not** depend on `quartzite-runtime::ObjectTree`; the caller owns the tree shape and implements `WidgetResolver` over whatever backing store it wants.

- For each visible widget (`WidgetExt::is_visible() == true`), invokes `Style::draw_widget(&dyn AsWidget, &mut dyn Painter, &Palette)` exactly once.

- Applies parent-relative-to-child coordinate transforms using the `Painter` `save` / `translate` / `restore` triplet so each `Style::draw_widget` call sees a `Painter` whose origin is at the widget's top-left.

- Resolves the active `Style` from the global `quartzite_style::StyleRegistry` (one `try_style()` lookup per `dispatch_paint` call) and threads through the single caller-supplied `&Palette`. If `try_style()` returns `None`, `dispatch_paint` is a no-op — zero `Painter` calls, zero panics.

- Hides traversal mechanics from the caller — callers wire up a `WidgetResolver`, root `ObjectId`, painter, and palette and let `dispatch_paint` drive `Style::draw_widget` for every visible descendant.

Concretely:

- A new bridge crate is added to the workspace `Cargo.toml` `members` list. Its `[dependencies]` include `quartzite-style`, `quartzite-widgets`, `quartzite-paint-api`, `quartzite-runtime` (for `ObjectId`), and `quartzite-core` (for `AsObject`). It does **not** depend on `quartzite-renderer`.
- One module (provisional path `src/dispatch.rs` re-exported from `src/lib.rs`) holds the `dispatch_paint` free function and the `WidgetResolver` trait.
- The per-frame body: visit-in-tree-order, skip hidden subtrees, translate by each child's `geometry().origin` before recursing into it, restore on the way out.
- Child enumeration is per-type: `Container` exposes `children()` (returning `&[ObjectId]` or equivalent), `ScrollArea` exposes `content_widget` (returning `Option<ObjectId>`). Other widgets (Button / Label / TextEdit / LineEdit) are treated as leaves. The dispatch helper mirrors `DefaultStyle::draw_widget`'s downcast-router shape — one `if let Some(_) = widget.as_any().downcast_ref::<T>()` arm per widget type that *has* children. Leaf widgets fall through and contribute no children.
- An unknown widget type (no matching arm) contributes no children — same silent fall-through as `DefaultStyle`'s unknown-widget arm. The widget itself is still painted via `Style::draw_widget`; only the recursion is skipped.

## Out of scope

- Per-widget repaint / damage tracking. The v1 traversal walks the whole tree every frame.
- Hit-testing / event dispatch. This spec covers paint only. Reverse-order traversal for hit-testing is a separate follow-up.
- Per-window palettes / dark-light theme switching. The dispatch helper accepts a `&Palette` per call; palette ownership / storage is decided by a future plan.
- Auto-installing `DefaultStyle` into the registry on first use. The decision in `2026-05-13-default-style-content.spec.md` stays: registration is opt-in; callers call `StyleRegistry::set_style(Box::new(DefaultStyle))` themselves.
- Modifying `Painter` to carry tree / style context. The dispatch helper drives `save` / `translate` / `restore` against the existing `Painter` surface; no new `Painter` methods.
- A generic `AsWidget::children()` method on the widget trait. v1 hard-codes per-type child enumeration the same way `DefaultStyle::draw_widget` hard-codes per-type painting; a future plan may abstract this out.
- Multi-window dispatch. Each call to `dispatch_paint` walks one tree; the dispatch helper has no multi-window awareness.
- `Container`'s layout pass. Layouts are run by `quartzite-widgets`' layout code (or the caller) **before** paint; the dispatch helper reads `geometry()` and trusts it is set.
- `ScrollArea` content-clipping / scroll-offset handling beyond a single `translate`. Clipping the content rect, scrolling, and dispatching the chrome separately from the content are out of v1.
- A new `WidgetRoot`-implementing type. The free-fn surface lets callers retain their own `WidgetRoot::paint` body and call `dispatch_paint` from inside it.

## Deferred

- A generic `AsWidget::children() -> &[ObjectId]` (or similar) trait method that would let the dispatch loop walk children without per-type knowledge | needs a coherent default + audit of every widget type | new issue if the per-type chain grows beyond ~5 arms.
- Hit-testing traversal that mirrors the paint-traversal in reverse z-order | needs an event-dispatch design beyond what `WidgetRoot::on_mouse_press` does today | new issue when input plumbing lands.
- Damage / dirty-rect tracking so only changed subtrees are repainted | needs a `WidgetExt::invalidate()` + scheduler integration | new issue once perf data justifies it.
- Per-widget clip-rect (e.g. `ScrollArea` clipping its content) | needs `Painter::clip_rect` wiring + nesting semantics | new issue when scroll content rendering lands.

## Key decisions

| Question | Decision |
|---|---|
| Crate location (Q1, round 1) | **New bridge crate.** Provisional name `quartzite-renderer-style` or `quartzite-style-dispatch`; final name pinned in design (single workspace `members` slot; lives at workspace root alongside other `quartzite-*` crates). Neither `quartzite-renderer` nor `quartzite-style` gains a dep on the other; the bridge crate depends on `quartzite-style` + `quartzite-widgets` + `quartzite-paint-api` + `quartzite-runtime` + `quartzite-core` and **not** on `quartzite-renderer`. |
| Tree-resolution API (Q2, round 1) | **`WidgetResolver` trait** that the caller implements: `fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>`. The dispatch helper takes `&dyn WidgetResolver`. The caller maps `ObjectId` → `&dyn AsWidget` however it wants (directly off a `quartzite-runtime::ObjectTree`, a custom test fixture, or any other backing store). The bridge crate does **not** know about `ObjectTree`. A blanket impl for `Fn(ObjectId) -> Option<&dyn AsWidget>` is a design-level detail (not specified here). |
| Public surface shape (Q3, round 3) | **Free function** — `pub fn dispatch_paint(root, resolver, painter, palette)`. Callers invoke it from inside their own `WidgetRoot::paint`. The bridge crate exposes the function plus the `WidgetResolver` trait; no new concrete `WidgetRoot`-implementing type, no default-impl method on the existing `WidgetRoot` trait. Rejected alternatives: (a) default-impl method on `WidgetRoot` (would force `quartzite-renderer` to depend on `quartzite-style` + `quartzite-widgets`, undoing the bridge separation); (b) new concrete `WidgetRoot`-implementing type (would force the bridge crate to depend on `quartzite-renderer` for the `WidgetRoot` trait; the free-fn keeps the bridge dependency-light and lets callers keep ownership of their own `WidgetRoot::paint` body). |
| What does the helper traverse? | A widget subtree starting at a caller-supplied `root: ObjectId`, walked by repeatedly invoking `WidgetResolver::resolve` to fetch each visited node. |
| Visibility filter | Skip the whole subtree when `!is_visible()` — invisible parents short-circuit the recursion. The widget node is **not** drawn and its children are **not** visited. The root being hidden produces zero paints. |
| Coordinate transform | `painter.save()` → `painter.translate(child.geometry().origin)` → recurse into child → `painter.restore()`. The root is drawn at the painter's current origin (no save/translate/restore around the root itself); each child is drawn with its parent-relative origin applied. Each `Style::draw_widget` sees a `Painter` whose `(0,0)` is the widget's top-left; the widget's own `geometry()` is in widget-local coordinates. |
| Traversal order | Depth-first, parent-before-child (painter's-algorithm z-order), in the order returned by `Container::children()` / `ScrollArea::content_widget`. The parent's `Style::draw_widget` runs **before** any of its children's. |
| Palette source | Single `&Palette` passed in per `dispatch_paint` call; the helper threads it unchanged into every `Style::draw_widget` invocation. v1 callers pass `&Palette::default()`. Per-window palette storage is a future plan. |
| Style source | `StyleRegistry::try_style()` once per `dispatch_paint` call; if `None`, the call is a no-op (no paint, no panic). Documented and asserted by a unit test. |
| Per-widget child enumeration | Per-type downcast chain (matches `DefaultStyle`'s shape). v1 arms: `Container` → `Container::children()`, `ScrollArea` → `ScrollArea::content_widget`. Unknown widget types contribute no children — silent fall-through, no panic. Adding a new container-shaped widget = adding one arm in the dispatch crate (mirrors `DefaultStyle`). |
| Resolver-miss policy | If the resolver returns `None` for an `ObjectId` referenced as a child, the dispatch helper logs at `warn` level (via `tracing`) and skips that subtree — same silent-skip shape as `!is_visible()`. The root `ObjectId` returning `None` is also a no-op (no paint, no panic) — same warn-and-skip semantics. |

## Technical constraints

- The new bridge crate is added to the workspace `Cargo.toml` `members` list. Its `[dependencies]` include `quartzite-style`, `quartzite-widgets`, `quartzite-paint-api`, `quartzite-runtime` (for `ObjectId`), and `quartzite-core` (for `AsObject`). It does **not** depend on `quartzite-renderer` — the free-fn surface keeps the bridge crate above renderer and style without coupling them.
- `quartzite-renderer` currently does **not** depend on `quartzite-widgets` or `quartzite-style`. `quartzite-style` has `quartzite-renderer` as a **dev**-dependency only. The new bridge crate sits **above** both renderer and style — no existing cycle, and the bridge crate does not introduce one.
- `WidgetExt::paint(&self, &mut dyn Painter)` and `WidgetRoot::paint(&self, &mut dyn Painter)` are stable surfaces; this spec does **not** rename, rebound, or extend them. The free-fn surface is called from inside the caller's existing `WidgetRoot::paint` body.
- The `WidgetResolver` trait is the only abstraction over tree storage; the bridge crate does **not** import `ObjectTree`. Real callers wire `impl WidgetResolver for &ObjectTree { fn resolve(&self, id) -> Option<&dyn AsWidget> { ... } }` (or similar), tests wire fixture lookups.
- `Painter::save` / `translate` / `restore` exist today; the dispatch loop relies on save/restore being correctly paired (verified by a unit test on a recording painter).
- The `dispatch_paint` free function takes everything it needs by reference (`&dyn WidgetResolver`, `&mut dyn Painter`, `&Palette`) and returns `()` — no internal state outliving the call.
- Doc gate: every new public item (the free fn, the `WidgetResolver` trait, the crate's `lib.rs` top-level docs) carries `///` first-line docs, a `# Examples` block where idiomatic, and `# Parameters` when applicable. Doc-test or example covers the end-to-end wiring path.
- Lint gate: `cargo clippy --workspace -- -D warnings` clean. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | A free function `pub fn dispatch_paint(root: ObjectId, resolver: &dyn WidgetResolver, painter: &mut dyn Painter, palette: &Palette)` exists in the new bridge crate's public API. The `WidgetResolver` trait (with method `fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>`) is also public. Given a widget tree and a root `ObjectId`, `dispatch_paint` invokes `Style::draw_widget` exactly once per visible widget in the subtree. |
| AC2 | When the root is hidden (`!is_visible()`), `dispatch_paint` records zero `Style::draw_widget` calls. A recording painter / mock style fixture verifies this. |
| AC3 | When a non-root `Container` is hidden, `dispatch_paint` records zero paints for that subtree (the hidden container itself **and** its visible descendants are skipped). |
| AC4 | `dispatch_paint` records paints in **depth-first, parent-before-child** order for a tree `Container { Label, Container { Button } }`. The recorded sequence is `[Container, Label, Container, Button]`. |
| AC5 | Before recursing into each non-root child, `dispatch_paint` issues `painter.save()` then `painter.translate(child.geometry().origin)`, and issues `painter.restore()` after that child's subtree completes. A recording painter verifies the save/translate/restore call sequence is well-paired (every `save` has a matching `restore`). The root is painted at the painter's incoming origin (no save/translate/restore around the root). |
| AC6 | `ScrollArea::content_widget == None` produces exactly one paint (the `ScrollArea` itself, no children). `ScrollArea` with `content_widget == Some(id)` produces two paints (`ScrollArea` then the content widget) when both are visible. |
| AC7 | An unknown widget type at the root (e.g. a bare `WidgetBase` not matched by the per-type downcast chain) is painted via `Style::draw_widget` once and contributes zero children — the recording sees a single paint and no recursion. |
| AC8 | When `StyleRegistry::try_style()` returns `None`, `dispatch_paint` is a no-op: zero `Painter` calls, zero panics. A unit test asserts the recording painter's captured-call list is empty. |
| AC9 | A doc-test or runnable example in the bridge crate demonstrates end-to-end wiring: caller implements `WidgetResolver` over a fixture tree (or `ObjectTree`), calls `dispatch_paint` from inside a `WidgetRoot::paint` body, and the recording painter captures the expected paint sequence. The example compiles under the doc gate and runs under `cargo test`. |
| AC10 | The resolver-miss path: when `WidgetResolver::resolve` returns `None` for a child `ObjectId`, `dispatch_paint` skips that subtree and emits a `tracing::warn!` event. A unit test wires a resolver that returns `None` for one mid-tree id and asserts the recording painter sees zero paints for that subtree, with the parent and sibling subtrees painted normally. A second unit test asserts that a `None` root also produces zero paints and a `warn` event. |
| AC11 | `cargo build`, `cargo test`, `cargo clippy --workspace -- -D warnings`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` are all clean on the changed crates. The new bridge crate is included in the workspace `members` and participates in `--workspace` lint / doc gates. |

## Open questions

- Final crate name (`quartzite-renderer-style` vs `quartzite-style-dispatch` vs other) — design phase picks one; both candidates compile and satisfy the dep constraint.
- Whether `WidgetResolver` should have a blanket impl for `Fn(ObjectId) -> Option<&dyn AsWidget>` so closure-shaped resolvers are usable without writing an explicit `impl` block. Sensible default = yes (and add it); design phase confirms.
- Per-widget clip-rect for `ScrollArea`-style content clipping: when scroll-area content rendering lands, the dispatch helper will need `painter.clip_rect(content_rect)` before recursing into `content_widget`. Out of v1 scope; tracked under § Deferred.
- Multi-pass rendering (e.g. paint pass 1 = chrome, pass 2 = overlay) for popups / tooltips: not modelled in v1; the dispatch helper does a single in-tree-order pass. Revisit when popups land.
- Whether the global `Palette` should be palette-per-window (each `WidgetRoot` carries its own) or one-per-process. v1 takes a `&Palette` per `dispatch_paint` call; the storage decision is deferred.
- Whether `WidgetExt::paint` is still useful once `Style::draw_widget` covers every widget the dispatch helper visits. A future plan may collapse the two; for v1 they coexist (the dispatch helper drives `Style`, not `WidgetExt::paint`).
