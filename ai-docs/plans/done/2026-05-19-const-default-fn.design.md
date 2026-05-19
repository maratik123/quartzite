# Design: Const-initialisable `new()` alongside `Default` impls

**Issue:** #484
**Date:** 2026-05-19

## Approach

### Chosen solution

Add `pub const fn new() -> Self` to every Group **(A)** type.  For
`Path` and `CloseEvent` (Group A — both keep `#[derive(Default)]`), no
`#[allow(clippy::derivable_impls)]` wrapper needed.  Use an explicit
`impl Default for T { #[inline] fn default() -> Self { Self::new() } }`
only for `Palette`, whose hand-written body
(`[Color::WHITE; ROLE_COUNT]` + 8 index overrides) cannot be expressed
by `#[derive(Default)]`.  `DefaultStyle` is **excluded per the
unit-struct rule** — its struct literal is itself a const expression; `#[derive(Default)]` stays but no `pub const fn new()` is added.

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
- **Replace `#[derive(Default)]` with explicit `impl Default { Self::new() }`
  on Path / CloseEvent / DefaultStyle and suppress `clippy::derivable_impls`
  with `#[allow(..., reason = "…")]`.** Rejected (design-amendment round 1,
  user request): the derive produces a byte-identical result; the `#[allow]`
  is a noise annotation that signals intent but suppresses a lint that is
  correctly firing.  Keeping `#[derive(Default)]` and adding only the
  `pub const fn new()` is cleaner — callers gain the const-construction path
  without any lint suppression.  For `Palette` the derive cannot produce the
  correct body, so the explicit `impl Default` is retained there.
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

### Group (A) — rewrite targets (3 types; `DefaultStyle` excluded per unit-struct rule)

| # | Type | Crate | File:Line | `Default` form | Notes |
|---|------|-------|-----------|----------------|-------|
| 1 | `Path` | `quartzite-paint-api` | `src/path.rs:69` | Keep `#[derive(Default)]` | `Vec::new()` is const since 1.39. `pub const fn new()` already existed; const-binding doctest present. No explicit `impl Default`. |
| 2 | `CloseEvent` | `quartzite-events` | `src/window.rs:100` | Keep `#[derive(Default)]` | Existing `pub const fn new() -> Self` constructs `Self { accepted: false }`. Const-binding doctest present. No explicit `impl Default`. |
| 3 | `Palette` | `quartzite-style-types` | `src/palette.rs:86` | Explicit `impl Default { Self::new() }` | Hand-written body (`[Color::WHITE; ROLE_COUNT]` + 8 index overrides) cannot be expressed by derive. **Already done (initial impl, commit 5b10a286):** `pub const fn new() -> Self` with `#[inline]` and `const PAL: Palette = Palette::new();` doctest. No amendment work. |

### Out of scope (zero-field unit struct — excluded from `pub const fn new()`)

| Type | Crate | File:Line | Reason |
|------|-------|-----------|--------|
| `DefaultStyle` | `quartzite-style` | `src/default_style.rs:58` | Zero-field unit struct — `DefaultStyle` itself is a const expression; `const X: DefaultStyle = DefaultStyle;` compiles without `new()`. Adding `new()` is noise with no payoff (same spirit as pure-unit-variant enum exclusion). Keep `#[derive(Clone, Copy, Debug, Default)]`; no `impl DefaultStyle` block. |

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
| 1 | Fix `Path` + `CloseEvent` — add `Default` back to the existing `#[derive(Clone, Debug, PartialEq)]` on `Path` (`path.rs:69`) and to `#[derive(Clone, Debug, PartialEq, Eq)]` on `CloseEvent` (`window.rs:100`); remove the `#[allow(clippy::derivable_impls, …)]` block at `path.rs:237–240` and the `impl Default for Path` block at `path.rs:241–256`; remove the `#[allow(…)]` block at `window.rs:155–158` and the `impl Default for CloseEvent` block at `window.rs:159–175`. Const-binding doctests on `new()` already in place — no doctest edits needed. Run `cargo test -p quartzite-paint-api` + `cargo test -p quartzite-events`. | `quartzite-paint-api/src/path.rs`, `quartzite-events/src/window.rs` | — |
| 2 | Remove `pub const fn new()` from `DefaultStyle` (unit-struct exclusion, PR #487 round-1 amendment) — remove the entire `impl DefaultStyle { … pub const fn new() … }` block at `default_style.rs:61–77`; revert the two struct-level doctests from `DefaultStyle::new()` back to `DefaultStyle` (at `default_style.rs:48` and `default_style.rs:56`). The `#[derive(Clone, Copy, Debug, Default)]` at line 58 stays. Run `cargo test -p quartzite-style`, then `cargo insta test -p quartzite-style` to confirm no snapshot drift. | `quartzite-style/src/default_style.rs` | — |
| 3 | Workspace-wide validation gates — `cargo fmt -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace --all-features`; `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`; `cargo build -p quartzite --no-default-features --features libm`; confirm snapshots byte-identical. | (no edits — gates) | 1, 2 |

Note on atomicity: subtasks 1–2 are independent at the file level.

## Handoff plan

The decomposition has `M = 3` subtasks → single terminal group of 3 (within
the 1..=3 cap).

- **Entry into Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § *Compaction recovery
  (re-entry)*.  The `/task` parent resumes inside the fresh subagent
  for Group A.
- **Group A:** subtasks 1–3 (terminal, size 3).  Subtask 1 is already done (Path+CloseEvent, commits e33a8ac+8a5ef72).  Subtask 2 removes `pub const fn new()` from DefaultStyle per unit-struct exclusion.  Subtask 3 runs workspace-wide gates.  Subtask 3 verifies AC1 (via grep) and closes AC4 / AC5 / AC7 / AC9.

## Risks

- **Risk (resolved):** `clippy::derivable_impls` previously fired on the
  explicit `impl Default for Path/CloseEvent/DefaultStyle`.  Resolved by
  the design amendment: keep `#[derive(Default)]` for those three types;
  only `Palette` uses an explicit `impl Default` (which is not derivable
  and therefore does not trigger the lint).
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
  not drift.  **Mitigation:** subtask 2 runs `cargo insta test
  -p quartzite-style` explicitly; subtask 3 re-confirms.  If a
  snapshot drifts, the rewrite has hidden a behavioural change that
  needs investigation — do NOT auto-accept the new snapshot.
- **Risk:** `Arc<Palette>` semantics — `WidgetBase` stores
  `Arc<Palette>` and seeds it via `Arc::new(Palette::default())`
  (line 134 of `widget_base.rs`).  After the Palette rewrite, this
  call-site remains `Arc::new(Palette::default())` and still works
  (the `Arc::new` is the non-const piece; Palette construction itself
  is now const-eligible but the surrounding context isn't, which is
  Group-B's correct skip behaviour — no caller update required, AC6).
- **Risk (resolved):** `pub const fn new()` on Palette and `pub const fn new()` on
  DefaultStyle required `///` + `# Examples` each.  Resolved in the initial
  implementation (commit 5b10a286) — doctests already in place; the amendment
  (subtasks 1–2) makes no changes to public API surface.
- **Risk (resolved):** Doc-link breakage from moving Palette's doc prose.
  Resolved in the initial implementation — `Palette::default`'s `///` block
  delegates to `[Self::new]`; intra-doc links resolve correctly.
- **Risk:** `no_std` / `libm` derive-free path — `quartzite-paint-api`
  is `no_std`-friendly via `alloc`.  `Path::new` uses `Vec::new()`
  which IS const in `alloc::vec::Vec`.  Subtask 1's local test plus
  subtask 3's `cargo build -p quartzite --no-default-features
  --features libm` command verify the no_std path stays green.

## Test Design

### Subtask 1 — Path + CloseEvent

- Location: existing `#[cfg(test)] mod tests` in
  `quartzite-paint-api/src/path.rs` and `quartzite-events/src/window.rs`.
- Entry point: existing tests assert `Path::default()` / `Path::new()`
  equivalence; `CloseEvent::default().accepted() == false`.  Both preserved
  by restoring `#[derive(Default)]`.
- Const-binding doctests on `Path::new()` and `CloseEvent::new()` already
  in place from the initial implementation — no doctest edits needed.
- Scenarios: identical to today — no new test required.

### Subtask 2 — DefaultStyle (unit-struct exclusion)

- Location: `quartzite-style/src/default_style.rs`.
- The `impl DefaultStyle { pub const fn new() … }` block at lines 61–77
  is removed entirely, which also deletes the `const STYLE: DefaultStyle =
  DefaultStyle::new()` const-binding doctest it contained.  Two
  struct-level doctests at lines 48 and 56 that reference
  `DefaultStyle::new()` are reverted to `DefaultStyle` (the struct
  literal, which is itself a const expression).
- The `#[derive(Clone, Copy, Debug, Default)]` at line 58 is unchanged.
- No new doctest is added — AC3 does not apply to DefaultStyle (which has
  no `new()` after this subtask).
- Run `cargo test -p quartzite-style` then `cargo insta test -p quartzite-style`
  to confirm no snapshot drift.
- Scenarios: `cargo test -p quartzite-style` green; snapshot pass-through (AC7).
- Fixtures / helpers needed: none.

### Subtask 3 — workspace validation

- Location: workspace root; no source edits.
- Entry point: AGENTS.md § *Build & Test* command list.
- Scenarios:
  - happy path — every gate green (AC4 / AC5).
  - failure path — any gate red surfaces a defect from subtasks 1–2
    that must be fixed before merge.
- Fixtures / helpers needed: none.

## Open questions

(none — round-1 Q1 / Q2 / Q3 in the spec closed every design-affecting
ambiguity, and the inventory + classification pass above closes every
remaining mechanical decision.)
