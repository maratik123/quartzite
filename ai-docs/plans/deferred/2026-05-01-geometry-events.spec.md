# Geometry & Events

**Source:** AI design dialogue (tmp/qt_01..14.log)
**Date:** 2026-05-01

## Scope

Two small crates with no external dependencies beyond `quartzite-core`.

### `quartzite-geometry`

- `Point` — integer (x, y); ops: add, sub, neg
- `Size` — non-negative (width, height); ops: add, scale
- `Rect` — (origin: Point, size: Size); methods: contains, intersects, united, translated, adjusted
- `Margins` — (left, top, right, bottom); apply to Rect

### `quartzite-events`

- `Event` trait — `event_type(&self) -> EventType`; object-safe
- `EventType` enum — Mouse, Keyboard, Resize, Close, Timer, User(u32)
- `MouseEvent` — position, global_position, button, buttons, modifiers
- `MouseButton` enum — Left, Right, Middle, Back, Forward
- `KeyEvent` — key, text, modifiers, is_repeat; `EventType` = KeyPress or KeyRelease
- `Key` enum — standard keys (A–Z, 0–9, F1–F12, Return, Escape, …)
- `KeyModifiers` — bitflags: Shift, Ctrl, Alt, Meta
- `ResizeEvent` — old_size, new_size
- `CloseEvent` — accepted: bool (call `accept()` to prevent close)
- `EventFilter` trait — `event_filter(&mut self, obj: ObjectId, event: &dyn Event) -> bool`
- `TimerEvent` — timer_id: ObjectId

## Out of scope

- Touch events (deferred)
- Drag & drop events (deferred)
- Platform-specific input (handled by backend, not this crate)

## Deferred

- `TouchEvent` | needs multi-touch design
- `DragDropEvent` | needs clipboard/MIME design
- `WheelEvent` | defer until scroll semantics decided

## Key decisions

| Question | Decision |
|---|---|
| Coordinate type | `i32` for Point/Size/Rect (pixel coordinates) |
| Rect empty check | `size.width == 0 \|\| size.height == 0` |
| EventFilter return | `true` = event consumed (stop propagation); `false` = continue |
| Key enum | Flat enum, not unicode codepoints (platform-mapped) |

## Technical constraints

- `quartzite-geometry`: no_std + alloc; no external deps beyond `core`
- `quartzite-events`: depends on `quartzite-geometry` and `quartzite-core` (for ObjectId in EventFilter)
- `Event` must be object-safe (`Box<dyn Event>`)
- `MouseButton` and `KeyModifiers` should use `bitflags` crate

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Rect::new(Point::new(10,10), Size::new(100,50)).contains(Point::new(50,30))` returns `true` |
| AC2 | `Rect::new(Point::new(10,10), Size::new(100,50)).contains(Point::new(9,10))` returns `false` |
| AC3 | Two non-overlapping rects: `intersects` returns `false` |
| AC4 | `Rect::united(r1, r2)` returns the smallest rect enclosing both |
| AC5 | `Margins::new(5,5,5,5)` applied to a `Rect` shrinks each edge by 5 |
| AC6 | `CloseEvent::new()` has `accepted == false`; after `accept()` it is `true` |
| AC7 | `KeyModifiers::CTRL | KeyModifiers::SHIFT` contains both flags |
| AC8 | `MouseEvent` constructed with `MouseButton::Left` reports `button() == MouseButton::Left` |

## Open questions

- Should `Point` and `Size` have floating-point (`PointF`, `SizeF`) variants for sub-pixel rendering?
- Should `Key` enum derive `Hash` + `Ord` for use in key-binding maps?
