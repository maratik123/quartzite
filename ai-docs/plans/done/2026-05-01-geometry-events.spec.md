# Geometry & Events

**Source:** issue #45
**Date:** 2026-05-01
**Tracked in:** #45

## Scope

Two small crates with no external dependencies beyond `quartzite-core`.

### `quartzite-geometry`

- `Point` — integer `i32` (x, y); ops: add, sub, neg
- `Size` — non-negative `i32` (width, height); ops: add, scale
- `Rect` — (origin: Point, size: Size); methods: contains, intersects, united, translated, adjusted
- `Margins` — `i32` (left, top, right, bottom); apply to Rect
- `PointF` — `f32` (x, y); ops: add, sub, neg; conversions to/from `Point`
- `SizeF` — `f32` (width, height); ops: add, scale; conversions to/from `Size`
- `RectF` — (origin: PointF, size: SizeF); same methods as `Rect`; conversions to/from `Rect`

### `quartzite-events`

- `Event<T>` trait — `event_type(&self) -> EventType<T>`; object-safe for fixed `T`; default `T = ()`
- `EventType<T: 'static + Send + Sync = ()>` enum — `Key(KeyEventKind)`, `Mouse(MouseEventKind)`, `Resize`, `Close`, `Timer`, `User(T)`
- `KeyEventKind` enum — `Press`, `Release`
- `MouseEventKind` enum — `Press`, `Release`, `Move`
- `MouseEvent` — position, global_position, button, buttons, modifiers
- `MouseButton` enum — Left, Right, Middle, Back, Forward
- `KeyEvent` — key, text, modifiers, is_repeat; `event_type()` returns `EventType::Key(KeyEventKind::Press/Release)`
- `Key` enum — standard keys (A–Z, 0–9, F1–F12, Return, Escape, …); derives `Hash + Ord`
- `KeyModifiers` — bitflags: Shift, Ctrl, Alt, Meta
- `ResizeEvent` — old_size, new_size
- `CloseEvent` — accepted: bool (call `accept()` to prevent close)
- `EventFilter<T: 'static + Send + Sync = ()>` trait — `event_filter(&mut self, obj: ObjectId, event: &dyn Event<T>) -> bool`
- `TimerEvent` — timer_id: ObjectId

Re-export both as `quartzite::geometry::*` and `quartzite::events::*` in the facade crate.

## Out of scope

- Touch events (deferred)
- Drag & drop events (deferred)
- Wheel events (deferred)
- Platform-specific input (handled by backend, not this crate)

## Deferred

- `TouchEvent` | needs multi-touch design
- `DragDropEvent` | needs clipboard/MIME design
- `WheelEvent` | defer until scroll semantics decided
- `MarginsF` | not needed until paint layer requires it

## Key decisions

| Question | Decision |
|---|---|
| Integer coordinate type | `i32` for `Point`/`Size`/`Rect`/`Margins` (pixel coordinates) |
| Float coordinate type | `f32` for `PointF`/`SizeF`/`RectF` (sub-pixel / GPU coordinates) |
| `PointF` → `Point` conversion | Round to nearest (`f32::round() as i32`); floor is explicit if needed |
| Rect empty check | `size.width == 0 \|\| size.height == 0` |
| EventFilter return | `true` = event consumed (stop propagation); `false` = continue |
| Key enum | Flat enum, not unicode codepoints (platform-mapped); derives `Hash + Ord` |
| `EventType` shape | Nested enums: `Key(KeyEventKind)`, `Mouse(MouseEventKind)` — discriminate without downcasting |
| `User` variant payload | Generic `T: 'static + Send + Sync = ()`; winit-style — app commits to one user event type; zero allocation, zero downcast |

## Technical constraints

- `quartzite-geometry`: `no_std + alloc`; no external deps beyond `core`
- `quartzite-events`: depends on `quartzite-geometry` and `quartzite-core` (for `ObjectId` in `EventFilter`)
- `Event<T>` must be object-safe for fixed `T`: `Box<dyn Event<T>>` and `&dyn Event<T>` must compile
- `MouseButton` and `KeyModifiers` should use the `bitflags` crate
- All public items must have `///` doc comments + `# Examples` blocks (`#![deny(missing_docs)]`)

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Rect::new(Point::new(10,10), Size::new(100,50)).contains(Point::new(50,30))` returns `true` |
| AC2 | `Rect::new(Point::new(10,10), Size::new(100,50)).contains(Point::new(9,10))` returns `false` |
| AC3 | Two non-overlapping `Rect`s: `intersects` returns `false` |
| AC4 | `Rect::united(r1, r2)` returns the smallest rect enclosing both |
| AC5 | `Margins::new(5,5,5,5)` applied to a `Rect` shrinks each edge by 5 |
| AC6 | `RectF::new(PointF::new(0.0,0.0), SizeF::new(1.0,1.0)).contains(PointF::new(0.5,0.5))` returns `true` |
| AC7 | `Point::from(PointF::new(1.7, 2.3))` rounds to nearest: `Point::new(2, 2)` |
| AC8 | `CloseEvent::new()` has `accepted == false`; after `accept()` it is `true` |
| AC9 | `KeyModifiers::CTRL \| KeyModifiers::SHIFT` contains both flags |
| AC10 | `MouseEvent` constructed with `MouseButton::Left` reports `button() == MouseButton::Left` |
| AC11 | `Key` values can be used as keys in a `HashMap` (derives `Hash + Eq`) |
| AC12 | `Key` values can be sorted (derives `Ord`) |
| AC13 | `quartzite-geometry` compiles with `no_default_features` (no_std path) |
| AC14 | `quartzite::geometry::Point` and `quartzite::events::MouseEvent` are accessible via the facade |
| AC15 | `EventType::<()>::User(())` compiles (default `T = ()`) |
| AC16 | `EventType::<MyEnum>::User(MyEnum::Foo)` compiles for a user-defined `enum MyEnum` that is `'static + Send + Sync` |
| AC17 | `Box<dyn Event<()>>` compiles (trait object with default user type) |

## Open questions

_None._
