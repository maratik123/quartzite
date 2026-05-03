# Design: geometry-events

**Issue:** #45
**Date:** 2026-05-03

## Approach

Create two new crates — `quartzite-geometry` and `quartzite-events` — and register them in the
workspace and the `quartzite` facade.

### `quartzite-geometry`

A `no_std` crate (no `alloc` needed) with zero external dependencies. All types live directly under
the crate root (no sub-modules needed at this scale). The integer types (`Point`, `Size`, `Rect`,
`Margins`) and float types (`PointF`, `SizeF`, `RectF`) are plain structs with
`Copy + Clone + Debug + PartialEq`. Arithmetic ops (`Add`, `Sub`, `Neg`, `Mul`) are implemented via
`core::ops::*`. `From`/`Into` conversions between the integer and float counterparts follow the spec
rule: `PointF → Point` rounds to nearest; `Point → PointF` is exact; same for `Size`/`SizeF`.
`RectF ↔ Rect` delegates to the field conversions.

No feature flags needed for this crate; all types are `Copy` with stack-only fields — no heap
allocation required, so not even `alloc` is needed. `lib.rs` declares `#![no_std]` unconditionally.
This tightens the spec's "`no_std + alloc`" to plain `no_std` — `alloc` is omitted because there is
no heap usage; the spec's phrasing was conservative.

### `quartzite-events`

A `no_std + alloc` crate that depends on `quartzite-geometry` and `quartzite-core`. Uses `bitflags 2`
for `MouseButton` and `KeyModifiers`.

`EventType`, `Event`, and `EventFilter` are generic over a user payload type `T` with a default of
`()` — winit-style: the whole application commits to one user event type at compile time. This gives
zero heap allocation and zero downcast for the `User` variant. `T = ()` works for apps with no custom
events. For a fixed `T`, `dyn Event<T>` is object-safe because the trait has no generic methods and
`EventType<T>` is `Copy` (no heap allocation). The `User(T)` variant carries the payload directly
inside the `Copy` enum, which is valid as long as `T: Copy` or the caller stores the enum by value
(the enum can only be `Copy` when `T: Copy`; non-`Copy` `T` must be retrieved by value before the
enum is dropped, but this is a usage concern, not an API concern).

Concrete event structs own their data by value; no lifetime parameters needed. `KeyEvent::text` is
`alloc::string::String` (the UTF-8 text produced by the key press). `CloseEvent` holds an
`accepted: bool` with a public `accept(&mut self)` method. `EventFilter` is a separate object-safe
trait.

`lib.rs` declares:
```rust
#![no_std]
extern crate alloc;
```

No `std` feature flag is needed because nothing in the crate requires `std` beyond what `alloc`
provides. `quartzite-core` is depended on with `default-features = false` to avoid pulling in its
optional `std` feature.

`quartzite-events::lib.rs` re-exports `quartzite_core::ObjectId` so callers of `EventFilter` do
not need to depend on `quartzite-core` directly.

Source is split into multiple files in `quartzite-events`:
- `src/lib.rs` — crate root, re-exports (including `pub use quartzite_core::ObjectId`),
  `#![deny(missing_docs)]`
- `src/event.rs` — `Event<T>` trait, `EventType<T>`, `KeyEventKind`, `MouseEventKind`,
  `EventFilter<T>`
- `src/mouse.rs` — `MouseButton`, `MouseEvent`
- `src/keyboard.rs` — `Key`, `KeyModifiers`, `KeyEvent`
- `src/window.rs` — `ResizeEvent`, `CloseEvent`
- `src/timer.rs` — `TimerEvent`

### Facade integration

`quartzite/src/lib.rs` gets two new `pub mod geometry` and `pub mod events` blocks, each containing
`pub use quartzite_geometry::*` / `pub use quartzite_events::*`. The `quartzite` root `Cargo.toml`
gains both crates as `default-features = false` path dependencies. `quartzite::events::EventType`
is automatically generic (`quartzite::events::EventType<MyEnum>` works without any extra import
because the re-export carries the generic parameter).

### Rejected alternatives

- **Single `quartzite-primitives` crate**: rejected — spec explicitly calls for two crates, and
  keeping geometry `no_std` (no alloc) while events depends on `alloc` (for `String`) and
  `quartzite-core` makes separation the natural boundary.
- **Sub-modules inside `quartzite-core`**: rejected — geometry is reused by future paint/widget
  crates independent of the object model; a separate crate avoids circular dependencies.
- **`enumflags2` for `KeyModifiers`/`MouseButton`**: rejected — the spec specifically names `bitflags`
  crate. Using `bitflags` also avoids importing `enumflags2` (already used for `PropertyFlags`) in a
  no-object-model context.
- **`std` feature for `quartzite-events`**: rejected — all heap types used (`String`) come from
  `alloc`, which is available in `no_std + alloc` environments; adding a `std` feature gate would
  be premature complexity with no current benefit.
- **Flat `EventType` enum (no nesting)**: rejected — flat variants like `KeyPress`/`KeyRelease` and
  `MousePress`/`MouseRelease`/`MouseMove` create ambiguity and duplicate discrimination logic.
  Nesting (`Key(KeyEventKind)`, `Mouse(MouseEventKind)`) lets callers match on the category first,
  then the kind, which is both more ergonomic and avoids variant name collisions.
- **`User(u32)` fixed payload**: rejected — the spec adopted the winit-style generic approach.
  A fixed `u32` would force users to do their own encoding/decoding. `User(T)` is zero-cost and
  type-safe; `T = ()` is the zero-overhead default for apps that need no custom events.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `quartzite-geometry` crate skeleton | `quartzite-geometry/Cargo.toml`, `quartzite-geometry/src/lib.rs` | — |
| 2 | Implement integer geometry types (`Point`, `Size`, `Rect`, `Margins`) | `quartzite-geometry/src/lib.rs` (or split into `src/point.rs`, `src/size.rs`, `src/rect.rs`, `src/margins.rs`) | 1 |
| 3 | Implement float geometry types (`PointF`, `SizeF`, `RectF`) with `From`/`Into` | same files as task 2 | 2 |
| 4 | Register `quartzite-geometry` in workspace `Cargo.toml` and verify `no_std` build | `Cargo.toml` (workspace `members`), CI shell | 1 |
| 5 | Create `quartzite-events` crate skeleton | `quartzite-events/Cargo.toml`, `quartzite-events/src/lib.rs` | 4 |
| 6 | Implement `Event<T>` trait, `EventType<T>`, `KeyEventKind`, `MouseEventKind`, `EventFilter<T>` | `quartzite-events/src/event.rs` | 5 |
| 7 | Implement `MouseButton` (bitflags), `MouseEvent`; `event_type()` returns `EventType::Mouse(MouseEventKind::*)` | `quartzite-events/src/mouse.rs` | 6 |
| 8 | Implement `Key`, `KeyModifiers` (bitflags), `KeyEvent` (`text: alloc::string::String`); `event_type()` returns `EventType::Key(KeyEventKind::*)` | `quartzite-events/src/keyboard.rs` | 6 |
| 9 | Implement `ResizeEvent`, `CloseEvent` | `quartzite-events/src/window.rs` | 6 |
| 10 | Implement `TimerEvent` | `quartzite-events/src/timer.rs` | 6 |
| 11 | Wire facade: add `geometry` and `events` modules to `quartzite/src/lib.rs` and `Cargo.toml` | `Cargo.toml`, `src/lib.rs` | 3, 10 |

11 tasks — within the 7-task soft limit only when grouped by crate boundary. Because the crates are
independent deliverables and each task is already atomic, the decomposition is retained as-is.
Logical grouping: tasks 1–4 (geometry), 5–10 (events), 11 (facade).

## Risks

- **`no_std` breakage in geometry**: `quartzite-geometry` must not use `std::` types anywhere.
  Mitigation: add `cargo build -p quartzite-geometry --no-default-features` to the CI verify step
  (matches the pattern already used for the quartzite facade).
- **`no_std + alloc` breakage in events**: `quartzite-events` uses `alloc::string::String` and
  depends on `quartzite-core` with `default-features = false`. Mitigation: CI build without default
  features for `quartzite-events` as well.
- **`bitflags` API surface**: `bitflags 2` changed its macro syntax vs v1. Mitigation: use the
  current `bitflags! { pub struct Foo: u8 { ... } }` form and pin to version `2` (no patch).
- **Object-safety of `Event<T>`**: for a fixed `T`, `event_type(&self) -> EventType<T>` is
  object-safe because `EventType<T>` is a concrete type (not a generic return). Adding any
  non-dispatchable method (generic method, by-value `self`, associated const without default) would
  break `Box<dyn Event<T>>`. Mitigation: design doc forbids adding non-object-safe methods to the
  trait; `EventType<T>` is `Copy` only when `T: Copy`, and `Clone` only when `T: Clone` — derive
  generates both conditional impls automatically.
- **`EventType<T>` Copy bound**: `EventType<T>` derives `Copy` only when `T: Copy`. This is
  automatically enforced by the derived impl. For non-`Copy` `T`, callers use `Clone`. Mitigation:
  document in the type that `Copy` requires `T: Copy`; the derive handles this automatically.
- **`Key` `Ord` derived ordering**: derived `Ord` is declaration order — fine for `HashMap` usage
  and sorting, but consumers must not rely on specific numeric ordering. Mitigation: document that
  the ordering is unspecified beyond being stable and total.
- **Circular dependency**: `quartzite-events` depends on `quartzite-core` (for `ObjectId`). If
  `quartzite-geometry` were to depend on `quartzite-core` that would increase the no_std risk.
  Mitigation: `quartzite-geometry` has zero inter-workspace deps.
- **`MouseButton` as bitflags vs enum**: the spec says "enum" for individual buttons but "bitflags"
  for the compound `buttons` field on `MouseEvent`. `MouseButton` is defined as a `bitflags!` struct
  so a single button and a multi-button mask share the same type, matching Qt's `Qt::MouseButton`
  approach. This is noted explicitly in the design.

## Test Design

### Task 2 — integer geometry (`quartzite-geometry/src/`)

Location: `#[cfg(test)] mod tests` in each source file (or a single `tests` module in `lib.rs`).

Entry points and scenarios:

- `Point::new` / field accessors — happy path
- `Point` add/sub/neg arithmetic — happy path + sign edge cases
- `Size::new` — happy path; document non-negative contract (no panic gate in v1)
- `Size` add/scale — happy path
- `Rect::contains` — AC1 (true), AC2 (false), boundary on all four edges
- `Rect::intersects` — AC3 (non-overlapping returns false), adjacent edges, full overlap
- `Rect::united` — AC4 (smallest enclosing rect)
- `Rect::translated` / `Rect::adjusted` — happy path
- `Margins` applied to `Rect` — AC5 (shrink by 5 on each edge); expanding (negative margins)

Fixtures: several named `Rect` constants (e.g. `UNIT_RECT`, `OFFSET_RECT`). Use `rstest` for
parametric boundary tests.

### Task 3 — float geometry (`quartzite-geometry/src/`)

Location: `#[cfg(test)] mod tests` same file.

Scenarios:

- `PointF::new`, `SizeF::new`, `RectF::new` — happy path
- `RectF::contains` — AC6
- `Point::from(PointF)` rounding — AC7 (1.7 → 2, 2.3 → 2)
- `PointF::from(Point)` exact cast
- `SizeF::from(Size)` / `Size::from(SizeF)`
- `RectF::from(Rect)` / `Rect::from(RectF)` — composed field conversions
- Edge cases: `PointF::new(0.5, 0.5)` rounds to `Point::new(1, 1)` (nearest); negative floats

### Task 6 — `EventType<T>` and sub-kinds (`quartzite-events/src/event.rs`)

Scenarios:

- `EventType::<()>::Key(KeyEventKind::Press)` and `EventType::<()>::Key(KeyEventKind::Release)`
  pattern-match correctly via nested match
- `EventType::<()>::Mouse(MouseEventKind::Press)`, `MouseEventKind::Release`,
  `MouseEventKind::Move` — same
- `EventType::<()>::User(())` compiles and stores the unit payload — AC15
- `EventType::<MyEnum>::User(MyEnum::Foo)` compiles for a user-defined `enum MyEnum: 'static +
  Send + Sync` — AC16
- `Box<dyn Event<()>>` compiles (object-safe for fixed `T = ()`) — AC17
- `EventType<()>` is `Copy` (since `()` is `Copy`); verify via `let _ = et; let _ = et;`

### Task 7 — mouse events (`quartzite-events/src/mouse.rs`)

`MouseEvent` field types:
- `position: Point` (cursor position in widget-local coordinates)
- `global_position: Point` (cursor position in screen coordinates)
- `button: MouseButton` (the button that triggered this event)
- `buttons: MouseButton` (all currently pressed buttons as a bitmask)
- `modifiers: KeyModifiers` (active keyboard modifiers at event time)

Scenarios:

- `MouseEvent` construction — AC10 (`button() == MouseButton::LEFT`)
- `MouseButton` bitflags combination (`LEFT | RIGHT` contains both)
- `MouseEvent::buttons()` reflects multi-button mask
- `MouseEvent::event_type()` returns `EventType::<()>::Mouse(MouseEventKind::Press)` for a press
  event and `EventType::<()>::Mouse(MouseEventKind::Move)` for a move event

### Task 8 — keyboard events (`quartzite-events/src/keyboard.rs`)

Scenarios:

- `KeyModifiers::CTRL | KeyModifiers::SHIFT` — AC9 (contains both)
- `Key` in `HashMap` — AC11
- `Key` sorting (`BTreeSet` or `Vec::sort`) — AC12
- `KeyEvent` is_repeat flag toggling
- `KeyEvent::event_type()` returns `EventType::<()>::Key(KeyEventKind::Press)` when not a release,
  and `EventType::<()>::Key(KeyEventKind::Release)` for a release event

### Task 9 — window events (`quartzite-events/src/window.rs`)

`ResizeEvent` field types:
- `old_size: Size`
- `new_size: Size`

Scenarios:

- `CloseEvent::new()` accepted is false — AC8
- `CloseEvent::accept()` sets accepted to true — AC8
- `ResizeEvent::new(old, new)` stores and returns both sizes unchanged

### Task 10 — timer event (`quartzite-events/src/timer.rs`)

Scenarios:

- `TimerEvent::new(id)` stores and returns `timer_id()` unchanged

### Task 11 — facade (`quartzite/src/lib.rs`)

Scenarios:

- `quartzite::geometry::Point` accessible — AC14
- `quartzite::events::MouseEvent` accessible — AC14
- `quartzite::events::EventType::<()>` accessible and usable as generic — AC15
- (Compile tests, no runtime assertion needed)

## File map (all new files)

```
quartzite-geometry/
  Cargo.toml
  src/
    lib.rs        (crate root; pub mod declarations; re-exports; #![no_std] #![deny(missing_docs)])
    point.rs      (Point, PointF)
    size.rs       (Size, SizeF)
    rect.rs       (Rect, RectF)
    margins.rs    (Margins)

quartzite-events/
  Cargo.toml
  src/
    lib.rs        (crate root; pub mod declarations; re-exports incl. ObjectId; #![no_std] extern crate alloc; #![deny(missing_docs)])
    event.rs      (Event<T> trait, EventType<T>, KeyEventKind, MouseEventKind, EventFilter<T>)
    mouse.rs      (MouseButton, MouseEvent)
    keyboard.rs   (Key, KeyModifiers, KeyEvent)
    window.rs     (ResizeEvent, CloseEvent)
    timer.rs      (TimerEvent)
```

Modified files:

```
Cargo.toml              (workspace members + quartzite facade deps)
quartzite/src/lib.rs    (add pub mod geometry, pub mod events)
```

## Cargo.toml sketches

### `quartzite-geometry/Cargo.toml`

```toml
[package]
name = "quartzite-geometry"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Geometry primitives (Point, Size, Rect, Margins) for quartzite"

# No [features] — always no_std (no alloc needed; all types are Copy + stack-only)

[dev-dependencies]
rstest = "0.26"
pretty_assertions = "1"

[package.metadata.docs.rs]
rustdoc-args = ["--cfg", "docsrs"]
rustc-args = ["--cfg", "docsrs"]
```

### `quartzite-events/Cargo.toml`

```toml
[package]
name = "quartzite-events"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Event model (MouseEvent, KeyEvent, EventFilter, …) for quartzite"

# No [features] — always no_std + alloc (String for KeyEvent::text; no std required)

[dependencies]
quartzite-core     = { path = "../quartzite-core", default-features = false }
quartzite-geometry = { path = "../quartzite-geometry" }
bitflags           = "2"

[dev-dependencies]
rstest = "0.26"
pretty_assertions = "1"

[package.metadata.docs.rs]
rustdoc-args = ["--cfg", "docsrs"]
rustc-args = ["--cfg", "docsrs"]
```

### `quartzite/Cargo.toml` patch (add to `[dependencies]`)

```toml
quartzite-geometry = { path = "quartzite-geometry", default-features = false }
quartzite-events   = { path = "quartzite-events",   default-features = false }
```

Both use `default-features = false` to avoid enabling any optional `std` feature that either crate
might gain in the future, consistent with how `quartzite-core` is already wired.

## Key implementation notes

### `EventType<T>` generic enum

```rust
/// Discriminant for every event kind.
///
/// `T` is the application-level user event payload type. Use the default `T = ()` for
/// applications with no custom events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventType<T: 'static + Send + Sync = ()> {
    /// A keyboard event (press or release).
    Key(KeyEventKind),
    /// A mouse event (press, release, or move).
    Mouse(MouseEventKind),
    /// The widget/window was resized.
    Resize,
    /// A close request was received.
    Close,
    /// A timer fired.
    Timer,
    /// A user-defined event with application-specific payload.
    User(T),
}

// Copy is only available when T: Copy
impl<T: 'static + Send + Sync + Copy> Copy for EventType<T> {}
```

`KeyEventKind` and `MouseEventKind` remain non-generic, `Copy`, `Clone`, `Debug`, `PartialEq`, `Eq`.

### `Event<T>` trait

```rust
/// Object-safe trait implemented by all event types.
///
/// `T` must match the application's chosen user event type; use `T = ()` for the common case.
pub trait Event<T: 'static + Send + Sync = ()> {
    /// Returns the discriminant describing which kind of event this is.
    fn event_type(&self) -> EventType<T>;
}
```

Object-safety analysis: `event_type` takes `&self` (not `self`) and returns `EventType<T>` — a
concrete type for a fixed `T`. No generic type parameters on the method itself; no associated types;
no `where Self: Sized` clauses needed. `Box<dyn Event<T>>` and `&dyn Event<T>` are both valid for
any fixed `T: 'static + Send + Sync`.

Concrete structs implement `Event<T>` for *all* `T: 'static + Send + Sync`:

```rust
impl<T: 'static + Send + Sync> Event<T> for MouseEvent {
    fn event_type(&self) -> EventType<T> {
        EventType::Mouse(self.kind)
    }
}
// Similarly for KeyEvent, ResizeEvent, CloseEvent, TimerEvent
```

This blanket-over-`T` approach lets callers use any of the concrete structs with any application
event type, without the caller needing to specify `T` at construction time.

### `EventFilter<T>` trait

```rust
/// Filter installed on an object to intercept events before they reach their target.
///
/// Return `true` to consume the event (stop propagation); `false` to continue.
pub trait EventFilter<T: 'static + Send + Sync = ()> {
    /// Called for each event dispatched to `obj`.
    fn event_filter(&mut self, obj: ObjectId, event: &dyn Event<T>) -> bool;
}
```

Object-safe for fixed `T`: `event_filter` takes `&mut self`, `ObjectId` (Copy), and
`&dyn Event<T>` (fat pointer, no generics on the method).

### `MouseEvent` fields

```rust
pub struct MouseEvent {
    position:        Point,
    global_position: Point,
    button:          MouseButton,
    buttons:         MouseButton,
    modifiers:       KeyModifiers,
    kind:            MouseEventKind,   // internal — determines event_type()
}
```

`position` and `global_position` use `quartzite_geometry::Point` (integer pixel coordinates).
`button` is the button that triggered this specific event; `buttons` is a bitmask of all buttons
currently held down. `modifiers` captures the keyboard modifier state at event time.

### `ResizeEvent` fields

```rust
pub struct ResizeEvent {
    old_size: Size,
    new_size: Size,
}
```

Both fields use `quartzite_geometry::Size`.

### `ObjectId` re-export in `quartzite-events`

`quartzite-events/src/lib.rs` includes:

```rust
pub use quartzite_core::ObjectId;
```

This allows `EventFilter` users to write `use quartzite_events::ObjectId` rather than having to
add `quartzite-core` as a direct dependency.

### `KeyEvent::text` field type

`text: alloc::string::String` — the UTF-8 text produced by the key press (empty string for
non-printable keys). This is the sole reason `quartzite-events` requires `alloc`; all other fields
are `Copy`.

### `MouseButton` as bitflags

`MouseButton` is modeled as a `bitflags!` struct (u8 or u16) so that both a single-button value and
a multi-button mask (`buttons` field on `MouseEvent`) share the same type. Individual constants
(`MouseButton::LEFT`, `MouseButton::RIGHT`, etc.) are the single-bit values. This is consistent with
Qt's approach and avoids an `enum` + separate `MouseButtons` bitset type.

### `Key` ordering

`#[derive(Ord, PartialOrd)]` gives declaration-order total ordering. Document that the ordering is
stable within a binary but not semantically meaningful (not Unicode codepoints, not alphabetic).

### `Event<T>` object safety (summary)

For a fixed `T: 'static + Send + Sync`:
- `event_type(&self) -> EventType<T>` — dispatchable (concrete return type, `&self` receiver)
- No generic methods, no associated types, no `where Self: Sized`
- Therefore `dyn Event<T>` is a valid trait object; `Box<dyn Event<T>>` and `&dyn Event<T>` compile

### `CloseEvent::accepted`

Private field, exposed read-only via `pub fn accepted(&self) -> bool` and mutated only via
`pub fn accept(&mut self)`. No `reject()` — default is rejected (`false`), `accept()` is one-way
for this design (matches spec).

### `no_std` in `quartzite-geometry`

`lib.rs` will have:
```rust
#![no_std]
#![deny(missing_docs)]
```
No `extern crate alloc` needed: all types are `Copy` with stack-only fields; no heap allocation
required.

### `no_std + alloc` in `quartzite-events`

`lib.rs` will have:
```rust
#![no_std]
#![deny(missing_docs)]
extern crate alloc;
```
`alloc` is used solely for `alloc::string::String` in `KeyEvent::text`. No `std` feature flag is
introduced.

### `#[inline]` rule

All field getters, `new()` constructors, `Default::default()` delegates, and `From` impls qualify
for `#[inline]` per AGENTS.md. Arithmetic operator impls (one operation, no branch) also qualify.
Generic functions (including the `Event<T>` blanket impls) are excluded — the compiler already has
their bodies via monomorphization.

## Open questions

- None. The spec marks "Open questions: None."
