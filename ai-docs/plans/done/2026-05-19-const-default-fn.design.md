# Design: Const-initialisable `new()` alongside `Default` impls

**Issue:** #484
**Date:** 2026-05-19

## Approach

### Chosen solution

Replace `#[derive(Default)]` (and the runtime-construction hand-written
`Default` impls) on every Group **(A)** type with an explicit
`impl Default for T { #[inline] fn default() -> Self { Self::new() } }`,
backed by a `pub const fn new() -> Self` zero-arg constructor.

The spec § *Scope* item 4 (amended) classifies types that already expose
*any* `pub const fn new(…)` as already conformant — the spec's payoff
(callers can write `const X: T = T::new(…)`) is met.  The 7 geometry
types (Point, PointF, Rect, RectF, Size, SizeF, Margins) fall into this
category and are therefore **out of scope** (see § *Inventory* §
*Out of scope (already conformant)*).

For every Group **(B)** type (atomic-counter ID seeds, `HashMap::new()`
with random hasher, `mpsc::channel()`, `thread::spawn` inside the
constructor, parley `FontContext::new()`, etc.) we leave the type
**entirely untouched** — neither the `#[derive(Default)]` nor the
hand-written `Default` impl is touched.  The opportunistic-lift sweep
identified zero liftable Group **(B)** candidates (see § *Inventory*
below — every const-blocker is either an external dependency we cannot
change here or a load-bearing identity / lifetime invariant the type
cannot drop without semantic breakage).

### Rejected alternatives

- **Add a `Self::ZERO` / `Self::DEFAULT` associated const for every
  geometry type** instead of rewriting `Default`.  Rejected: callers
  already write `Point::new(0, 0)` in const context today; adding both
  `Point::ZERO` and the explicit `Default` rewrite duplicates the const-
  construction path without giving callers anything they cannot already
  spell.  YAGNI.
- **Add a redundant zero-arg `pub const fn new() -> Self` to Point /
  Size / Rect / Margins via rename** (e.g. rename existing `new(x, y)` to
  `new_at(x, y)`).  Rejected: AGENTS.md § *API Stability* permits clean
  renames pre-publish, but the rename produces churn in every call-site
  (Point::new is used pervasively) for zero functional gain — the
  const-construction path already exists.
- **Shape 2 — replace `#[derive(Default)]` on geometry types with explicit
  `impl Default { Self::new(0, 0) }`.** Rejected (design-review round 1,
  spec amendment): callers already get `const P: Point = Point::new(0, 0)`
  today; the spec's payoff is already met; the rewrite is pure churn.
  Spec § *Scope* item 4 (amended) classifies these types as already
  conformant.  They remain in the "Out of scope (already conformant)" table.
- **Lift `Font` into Group (A) by changing `family: String` to
  `Cow<'static, str>` or `&'static str`.**  Rejected: this is a
  workspace-wide API change (every `Font::new("…", …)` call-site shifts
  type), not the "small local refactor" the spec authorises.  Skip per
  Group (B) policy.
- **Lift `ObjectId` / `ConnectionId` to a const `Self::FIRST_ID`.**
  Rejected: the type's whole purpose is process-unique monotonic IDs;
  removing the `AtomicU64::fetch_add` would break every test that
  asserts distinctness / ordering and every consumer that relies on
  uniqueness.
- **Add `const fn new()` returning an "empty" `ObjectTree` /
  `ObjectFactory` using `HashMap::with_hasher(BuildHasherDefault::new())`
  to dodge the random-hasher const-blocker.**  Rejected: switching away
  from the default `RandomState` is a security / DoS-resistance change
  on hashing of attacker-controllable keys (object names, class strings).
  Skip per Group (B) policy.

## Inventory

### Group (A) — rewrite targets (4 types)

| # | Type | Crate | File:Line | New `default()` body | Notes |
|---|------|-------|-----------|----------------------|-------|
| 1 | `Path` | `quartzite-paint-api` | `src/path.rs:69` | `Self::new()` | `Vec::new()` is const since 1.39. Existing `pub const fn new()` covers the const-construction path. Remove `#[derive(Default)]`; add explicit `impl Default`. |
| 2 | `CloseEvent` | `quartzite-events` | `src/window.rs:100` | `Self::new()` | Existing `pub const fn new() -> Self` constructs `Self { accepted: false }` — already const. Remove `#[derive(Default)]`; add explicit `impl Default`. |
| 3 | `Palette` | `quartzite-style-types` | `src/palette.rs:86` | `Self::new()` | Existing hand-written `Default::default()` body: `[Color::WHITE; ROLE_COUNT]` then index assignment for 8 specific roles. Array `[Copy; N]` literal is const; index assignment on a mutable local is const since Rust 1.83 (`const_mut_refs`). **Add `pub const fn new() -> Self` containing the lifted body, mark `#[inline]`, doctest with `const PAL: Palette = Palette::new();`.** |
| 4 | `DefaultStyle` | `quartzite-style` | `src/default_style.rs:58` | `Self::new()` | Unit struct. `pub const fn new() -> Self { Self }`. Marker-only type for `Style` registry; no visual semantics change. |

### Group (B) — left untouched (18 types)

| Type | Crate | File:Line | Const-blocker | Liftable? | Why not |
|------|-------|-----------|---------------|-----------|---------|
| `Font` | `quartzite-paint-api` | `src/font.rs:216` | `family: String`; default value `"sans-serif"` requires `String::from(&str)` — not const. | No | Lifting requires changing `family` to `&'static str` / `Cow<'static, str>` — workspace-wide API churn (every `Font::new("Arial", …)` shifts type), not "small local". |
| `ObjectFactory` | `quartzite-runtime` | `src/factory.rs:39` | `HashMap::new()` constructs `RandomState::new()` — non-const (reads OS entropy). | No | Switching to a non-randomised hasher is a security regression for attacker-controllable keys. |
| `ObjectTree` | `quartzite-runtime` | `src/object_tree.rs:27` | `HashMap::new()` × 4 + `slotmap::SlotMap::new()` (third-party, non-const). | No | Same hasher-randomness issue; SlotMap upstream not const. |
| `ThreadDriver` | `quartzite-runtime` | `src/timer_drivers.rs:71` | `Arc::new(AtomicBool::new(false))` — `Arc::new` is not const (heap alloc). | No | Removing the `Arc` shared-state pattern would re-architect the cross-thread `stop()` path. |
| `AppDriver` | `quartzite-runtime` | `src/timer_drivers.rs:158` | Same as ThreadDriver. | No | Same. |
| `PoolDriver` | `quartzite-runtime` | `src/timer_drivers.rs:358` | `Arc::new(PoolInner { … })` + `thread::spawn` inside `new()`. | No | The thread spawn is the type's reason to exist; cannot defer. |
| `EventLoop` | `quartzite-runtime` | `src/event_loop.rs:274` | `mpsc::channel()` non-const + `Arc::new(AtomicBool::new(false))`. | No | Channel construction is intrinsic to event-loop semantics. |
| `FontCache` | `quartzite-renderer` | `src/font.rs:46` | `parley::FontContext::new()` scans system fonts via fontconfig / CoreText / DirectWrite. | No | FFI / OS calls — non-const by definition. |
| `ObjectBase` | `quartzite-core` | `src/object_base.rs:285` | `ReceiverGuard::new_pair()` allocates an `Arc`; `ObjectId::new()` is `AtomicU64::fetch_add`; `std::thread::current().id()` reads thread-local. | No | Three concurrent const-blockers, each load-bearing. |
| `ObjectId` | `quartzite-core` | `src/id.rs:58` | `AtomicU64::fetch_add` on a private static counter — process-unique monotonic ID invariant. | No | The atomic IS the type's reason to exist. A `Self::FIRST_ID` would break every distinctness test. |
| `ConnectionId` | `quartzite-core` | `src/id.rs:119` | Same as ObjectId. | No | Same. |
| `Signal<Args>` | `quartzite-core` | `src/signal.rs:317` | `indexmap::IndexMap::new()` (uses default hasher = randomised) × 3. | No | Switching hasher is a behaviour change; IndexMap upstream not const. |
| `WidgetBase` | `quartzite-widgets` | `src/widget_base.rs:143` | `ObjectBase::new()` (transitive — see ObjectBase), `Arc::new(Font::default())` × 2 (Arc heap-alloc + Font is Group B). | No | Multiple layered Group-B chains. |
| `ScrollArea` | `quartzite-widgets` | `src/widgets/scroll_area.rs:79` | `WidgetBase::new()` chain → ObjectBase chain. | No | Same. |
| `TextEdit` | `quartzite-widgets` | `src/widgets/text_edit.rs:59` | `WidgetBase::new()` chain + `Signal::default()`. | No | Same. |
| `LineEdit` | `quartzite-widgets` | `src/widgets/line_edit.rs:67` | Same as TextEdit. | No | Same. |
| `Container` | `quartzite-widgets` | `src/widgets/container.rs:115` | `WidgetBase::new()` chain. | No | Same. |
| `GridLayout` | `quartzite-widgets` | `src/layout/grid_layout.rs:223` | `ObjectBase::new()` chain. | No | Same. |

### Out of scope (already conformant)

Spec § *Scope* item 4 (amended) — types that already expose a
`pub const fn new(…)` (zero-arg or multi-arg) so callers can already use
them in `const`/`static` contexts.  The spec's payoff is already met;
rewriting their `Default` impl would be pure churn.  Listed for
auditability; no work required.

| Type | Crate | File:Line | Const-construction path |
|------|-------|-----------|-------------------------|
| `Color` | `quartzite-paint-api` | `src/color.rs:175` | `Color::BLACK` (associated const). Spec calls this out explicitly. |
| `Pen` | `quartzite-paint-api` | `src/pen.rs:68` | `Pen::new(Color::BLACK, 1.0)` (existing const fn). Spec calls this out. |
| `Brush` | `quartzite-paint-api` | `src/brush.rs:236` | `Brush::solid(Color::WHITE)` (existing const fn). Same shape as Color / Pen. |
| `Point` | `quartzite-geometry` | `src/point.rs:58` | `Point::new(x, y)` (multi-arg const fn). Callers write `const P: Point = Point::new(0, 0)` today. |
| `PointF` | `quartzite-geometry` | `src/point.rs:129` | `PointF::new(x, y)` (multi-arg const fn). Same shape as Point. |
| `Rect` | `quartzite-geometry` | `src/rect.rs:16` | `Rect::new(origin, size)` (multi-arg const fn). |
| `RectF` | `quartzite-geometry` | `src/rect.rs:275` | `RectF::new(origin, size)` (multi-arg const fn). |
| `Size` | `quartzite-geometry` | `src/size.rs:50` | `Size::new(w, h)` (multi-arg const fn). |
| `SizeF` | `quartzite-geometry` | `src/size.rs:136` | `SizeF::new(w, h)` (multi-arg const fn). |
| `Margins` | `quartzite-geometry` | `src/margins.rs:17` | `Margins::new(left, top, right, bottom)` (multi-arg const fn). |

### Out of scope (pure unit-variant enums with `#[default]`)

Spec § *Key decisions* row on enums — pure unit-variant enums are out
of scope because `MyEnum::Variant` is already a const expression callers
can spell directly.

| Type | Crate | File:Line | Default variant |
|------|-------|-----------|-----------------|
| `FontWeight` | `quartzite-paint-api` | `src/font.rs:248` | `Normal` |
| `FocusPolicy` | `quartzite-widgets` | `src/enums.rs:14` | `NoFocus` |
| `SizePolicy` | `quartzite-widgets` | `src/enums.rs:37` | `Fixed` |
| `CursorShape` | `quartzite-widgets` | `src/enums.rs:62` | `Arrow` |
| `Alignment` | `quartzite-geometry` | `src/alignment.rs:19` | `Left` |
| `ScrollPolicy` | `quartzite-widgets` | `src/widgets/scroll_area.rs:17` | `AsNeeded` |
| `Direction` | `quartzite-widgets` | `src/layout/box_layout.rs:19` | `Horizontal` |
| `Value` | `quartzite-core` | `src/value.rs:147` | `Null` — unit variant.  Per spec spirit (default variant is a const expression, callers can write `Value::Null` directly), out of scope.  The non-default variants carry data (`Int(i64)`, `String(String)`, `Map(BTreeMap<…>)` etc.) but the `#[default]` discriminator is a bare unit, so the const-eligibility of the *default* is already there. |

### Out of scope (private / test-only)

| Type | Crate | File:Line | Reason |
|------|-------|-----------|--------|
| `RecordingPainter` | `quartzite-style` | `src/default_style_tests.rs:57` | Private test helper (`struct` not `pub`). No downstream callers; spec payoff (callers gaining a `const fn new()`) does not apply. |

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rewrite `Path` in `quartzite-paint-api` — remove `#[derive(Default)]`; add explicit `#[inline] impl Default for Path { fn default() -> Self { Self::new() } }` with doc + `# Examples`. Run `cargo test -p quartzite-paint-api`. | `quartzite-paint-api/src/path.rs` | — |
| 2 | Rewrite `CloseEvent` in `quartzite-events` — remove `#[derive(Default)]`; add explicit `#[inline] impl Default for CloseEvent { fn default() -> Self { Self::new() } }` with doc + `# Examples`. Run `cargo test -p quartzite-events`. | `quartzite-events/src/window.rs` | — |
| 3 | Rewrite `Palette` in `quartzite-style-types` — add `pub const fn new() -> Self` lifting the existing `[Color::WHITE; ROLE_COUNT]` + 8 role overrides into the const body (`#[inline]`, doc + `# Examples` block including `const PAL: Palette = Palette::new();`); rewrite `impl Default for Palette { #[inline] fn default() -> Self { Self::new() } }`. Move existing detailed doc-prose about the colour seeds from `Default::default` into the new `new()` doc; keep `Default::default` doc terse ("returns `Self::new()`"). Run `cargo test -p quartzite-style-types`. | `quartzite-style-types/src/palette.rs` | — |
| 4 | Rewrite `DefaultStyle` in `quartzite-style` — remove `#[derive(Default)]` from the `pub struct DefaultStyle;` unit struct; add `impl DefaultStyle { /// … # Examples ... #[inline] pub const fn new() -> Self { Self } }` with const-binding doctest; add explicit `#[inline] impl Default for DefaultStyle { fn default() -> Self { Self::new() } }`. Run `cargo test -p quartzite-style`, then `cargo insta test -p quartzite-style` to confirm no snapshot drift. | `quartzite-style/src/default_style.rs` | — |
| 5 | Workspace-wide validation gates — `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-features`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`, and confirm `quartzite-style/tests/snapshots/` is byte-identical. Fix any lint surfaced by `clippy::needless_const_for_fn`, `clippy::derivable_impls`, or related. | (no edits — gates) | 1, 2, 3, 4 |

Note on atomicity: subtasks 1–4 are independent at the file level (no
import cross-edits, no shared traits being modified, no cross-crate
signature changes).  Their `Depends on` column is `—` because each
subtask compiles standalone — the workspace-wide gate (subtask 5)
batches the final validation per AGENTS.md § *Build & Test*.

## Handoff plan

The decomposition has `M = 5` subtasks → two groups, sized 3 + 2 (both
within the 1..=3 cap; the second is terminal at exactly 2).

- **Entry into Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § *Compaction recovery
  (re-entry)*.  The `/task` parent resumes inside the fresh subagent
  for Group A.
- **Group A:** subtasks 1–3 — three independent crate-local rewrites
  (`quartzite-paint-api`, `quartzite-events`, `quartzite-style-types`).
  Each crate's local test command is run inside the subtask.  At end of
  Group A, the paint-api / events / style-types crates compile and test
  green in isolation.
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § *Compaction recovery
  (re-entry)*.  The `/task` parent resumes in Group B with fresh
  context.
- **Group B:** subtasks 4–5 — DefaultStyle rewrite (touches the only
  `Style` impl, requires snapshot test pass-through) and the
  workspace-wide validation gate.  Terminal group, size 2 (within the
  1..=3 range).  Subtask 5 closes AC4 / AC5 / AC7 / AC9 in one pass.

## Risks

- **Risk:** `clippy::derivable_impls` lint may fire on the explicit
  `impl Default for T { fn default() -> Self { Self::new() } }` after
  the rewrite — clippy can synthesise the body and complain that
  `#[derive(Default)]` would have done the same.  **Mitigation:** for
  the 4 Group (A) types, `Self::new()` carries a const-eligibility
  intent the derive cannot express (`#[derive(Default)]` produces a
  non-const `default()`).  If clippy fires, add
  `#[allow(clippy::derivable_impls, reason = "explicit impl preserves
  const-construction semantics; derive defeats AC1 const-eligibility
  goal")]` at the impl site.
- **Risk:** `clippy::needless_const_for_fn` may fire on Palette's new
  `const fn new()` if clippy decides the body is trivially const-able
  via derive.  **Mitigation:** unlikely — Palette's body contains
  index-assignment, which the derive does not produce.  If it fires,
  re-evaluate; likely no allow needed.
- **Risk:** AC7 (snapshot-tests byte-identical) — `DefaultStyle` is
  swept by the design-system trigger conditions (AGENTS.md § *Design
  system* row 1: "any `Style` impl, including `DefaultStyle`").  The
  rewrite here is purely structural (unit struct, no behaviour change
  in `Style::draw_widget` or any `Paint<W>` impl), so snapshots should
  not drift.  **Mitigation:** subtask 5 runs `cargo insta test
  -p quartzite-style` explicitly; subtask 5 re-confirms.  If a
  snapshot drifts, the rewrite has hidden a behavioural change that
  needs investigation — do NOT auto-accept the new snapshot.
- **Risk:** `Arc<Palette>` semantics — `WidgetBase` stores
  `Arc<Palette>` and seeds it via `Arc::new(Palette::default())`
  (line 134 of `widget_base.rs`).  After the Palette rewrite, this
  call-site remains `Arc::new(Palette::default())` and still works
  (the `Arc::new` is the non-const piece; Palette construction itself
  is now const-eligible but the surrounding context isn't, which is
  Group-B's correct skip behaviour — no caller update required, AC6).
- **Risk:** `pub const fn new()` on Palette and `pub const fn new()` on
  DefaultStyle add new public items — `missing_docs = "deny"` requires
  `///` + `# Examples` on each.  **Mitigation:** subtasks 4 and 5
  explicitly call out the doctest with `const X: T = T::new();` so the
  doc gate and AC3 close together.
- **Risk:** Doc-link breakage — moving Palette's existing prose about
  colour seeds from `Default::default`'s `///` block onto the new
  `new()`'s `///` block may break intra-doc links if any external
  reference points at `Palette::default`.  **Mitigation:** keep a
  terse `Default::default` `///` block that says "returns
  `[Self::new]`" with the bracketed link, so `Palette::default`
  remains documented and any intra-doc link resolves.
- **Risk:** `no_std` / `libm` derive-free path — `quartzite-paint-api`
  is `no_std`-friendly via `alloc`.  `Path::new` uses `Vec::new()`
  which IS const in `alloc::vec::Vec`.  Subtask 2's local test plus
  subtask 5's `cargo build -p quartzite --no-default-features
  --features libm` command verify the no_std path stays green.

## Test Design

### Subtask 1 — Path

- Location: existing `#[cfg(test)] mod tests` in
  `quartzite-paint-api/src/path.rs`.
- Entry point: existing tests assert `Path::default()` and
  `Path::new()` produce equivalent paths.  Preserved by the rewrite.
- Scenarios: identical to today — no new test required.

### Subtask 2 — CloseEvent

- Location: existing `#[cfg(test)] mod tests` in
  `quartzite-events/src/window.rs` if present; otherwise the existing
  doctest on `CloseEvent::new` covers the construction.
- Entry point: `CloseEvent::default().accepted() == false`; preserved.
- Scenarios: identical to today.

### Subtask 3 — Palette

- Location: existing `#[cfg(test)] mod tests` in
  `quartzite-style-types/src/palette.rs`.
- Entry point:
  - existing `default_has_non_transparent_color_for_every_role` —
    must still pass against the rewritten `Palette::default()` (which
    now delegates to `Palette::new()`).
  - existing `default_highlight_differs_from_highlighted_text` —
    must still pass.
  - **new doctest** on `pub const fn new()`:
    ```
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Color;
    /// use quartzite_style_types::{ColorRole, Palette};
    ///
    /// const PAL: Palette = Palette::new();
    /// assert_eq!(PAL.color(ColorRole::Window), Color::WHITE);
    /// ```
    ```
    The `const PAL` binding is what proves const-eligibility per AC3.
- Scenarios: happy path covered by existing tests; AC3 doctest closes
  the const-binding scenario.
- Fixtures / helpers needed: none.

### Subtask 4 — DefaultStyle

- Location: `quartzite-style/src/default_style.rs` — add a small
  `#[cfg(test)] mod tests` block (the file currently has no inline
  tests for `DefaultStyle` itself; existing visual tests live in
  `default_style_tests.rs` as integration-style files).
- Entry point:
  - **new test** `default_style_new_constructs` — `let _: DefaultStyle
    = DefaultStyle::new();` — trivial, but the file gains its first
    `#[cfg(test)]` block.  Note that AGENTS.md § *Workflow* allows
    skipping the block when the file's logic is <50 lines or trivial;
    DefaultStyle file IS > 50 lines but most of it is `Paint<W>` impls
    (covered by `default_style_tests.rs`).  Adding the block is
    optional; the **new doctest** on `pub const fn new() -> Self`
    suffices to close AC3.
  - **new doctest** on `pub const fn new()`:
    ```
    /// # Examples
    ///
    /// ```
    /// use quartzite_style::DefaultStyle;
    ///
    /// const STYLE: DefaultStyle = DefaultStyle::new();
    /// let _ = STYLE; // const-binding compiles
    /// ```
    ```
- Snapshot tests under `quartzite-style/tests/snapshots/` must remain
  byte-identical (AC7).
- Scenarios: const-binding compiles (AC3); snapshot pass-through (AC7).
- Fixtures / helpers needed: none.

### Subtask 5 — workspace validation

- Location: workspace root; no source edits.
- Entry point: AGENTS.md § *Build & Test* command list.
- Scenarios:
  - happy path — every gate green (AC4 / AC5).
  - failure path — any gate red surfaces a defect from subtasks 1–5
    that must be fixed before merge.
- Fixtures / helpers needed: none.

## Open questions

(none — round-1 Q1 / Q2 / Q3 in the spec closed every design-affecting
ambiguity, and the inventory + classification pass above closes every
remaining mechanical decision.)
