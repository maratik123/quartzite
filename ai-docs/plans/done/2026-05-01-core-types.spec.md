# Core Types & Traits

**Source:** AI design dialogue (tmp/qt_01..14.log)
**Date:** 2026-05-01

## Scope

Implement `quartzite-core` crate — the foundation every other crate depends on. No UI, no runtime, no proc macros. Must be `no_std`-compatible (with `alloc`).

- `ObjectId` — unique 64-bit handle, never reused
- `ConnectionId` — unique handle per signal connection
- `ReceiverGuard` — zero-sized lifetime token; `Arc<ReceiverGuard>` breaks incoming connections on drop
- `ObjectBase` — shared data for every object: id, name, parent, children, receiver_guard, outgoing_connections, dynamic_properties, thread_id, signals_blocked
- `AsObject` trait — pure, object-safe accessor: `object_base()`, `object_base_mut()`, `as_any()`, `as_any_mut()`
- `Object` trait — meta-system: `meta_object()`, `read_property()`, `write_property()`, `invoke_method()`, `connect_signal()`
- `ObjectExt` trait — blanket convenience methods (id, name, set_name, parent, children, downcast_ref, downcast_mut, is, is_on_current_thread, dynamic_property, set_dynamic_property)
- `Value` enum — dynamic type container: Null, Bool, Int(i64), Float(f64), String, List, Map, Bytes, Object(WeakObjectRef), Custom(Arc<dyn CustomValue>)
- `FromValue` / `IntoValue` traits — typed conversions to/from `Value`
- `TypeError` — conversion error type
- `Signal<Args>` — typed signal with connect/disconnect/emit; SlotEntry with ConnectionType
- `ConnectionType` enum — Direct, Queued, SingleShot
- `MetaObject` — static per-type reflection struct
- `PropertyMeta` — name, type_name, flags (readable, writable, notify, stored, designable, user, constant)
- `SignalMeta` / `MethodMeta` / `ParamMeta` / `EnumMeta` / `EnumEntry`

## Out of scope

- Proc macros (separate crate)
- Object tree / arena / ownership model (quartzite-runtime)
- Widget hierarchy (quartzite-widgets)
- Event system (quartzite-events)
- Python interop (deferred)

## Deferred

- `BlockingQueued` connection type | threading model not yet decided
- Signal-to-signal connections | needs runtime design first
- Computed properties (stored = false + getter closure) | use methods for now
- Property bindings (two-way sync) | BindingEngine is future work
- No-std validation | design for it but don't enforce until runtime exists

## Key decisions

| Question | Decision |
|---|---|
| Object ownership model | Open question — ObjectBase itself does NOT decide arena vs Rc<RefCell<>>; it only stores ObjectId refs |
| `as_any` location | In `AsObject` (not ObjectExt) — required for object-safety of Box<dyn AsObject> |
| `Object` vs `AsObject` | Separate traits: AsObject = pure accessor usable by base types; Object = meta-system, only concrete types |
| Int representation | All integers as `i64` in `Value` |
| Signal arg tuple | `Signal<Args>` where Args is a tuple type (e.g. `Signal<(bool,)>`) |
| WeakObjectRef type | Type alias for weak reference to `dyn Object`; concrete representation deferred to runtime |

## Technical constraints

- `quartzite-core` must compile with `#![no_std]` + `extern crate alloc`
- No dependency on `quartzite-macros` (macros use core, not the reverse)
- `AsObject` must be object-safe (`Box<dyn AsObject>` must work)
- `Object` must be object-safe (`Box<dyn Object>` must work)
- `Signal<Args>` slots stored as `Box<dyn Fn(&Args)>` — not `FnMut` (re-entrancy)
- Atomic counters for ObjectId/ConnectionId: `AtomicU64`, `Ordering::Relaxed`

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ObjectId::new()` returns distinct values across sequential and concurrent calls |
| AC2 | Dropping an `ObjectBase` while another thread holds `Weak<ReceiverGuard>` causes `Weak::upgrade()` to return `None` |
| AC3 | `Value::Int(42).try_into::<i32>()` returns `Ok(42)`; `Value::String("x".into()).try_into::<i32>()` returns `Err(TypeError::TypeMismatch)` |
| AC4 | `Signal<(i32,)>::emit` calls all connected Direct slots with the emitted value |
| AC5 | After `disconnect(id)`, the disconnected slot is not called on subsequent `emit` |
| AC6 | A `SingleShot` connection is called exactly once; subsequent emits do not invoke it |
| AC7 | `ObjectBase::new()` records `thread::current().id()` in `thread_id` |
| AC8 | `ObjectExt::downcast_ref::<T>()` returns `Some(&T)` when the underlying type is `T`, `None` otherwise |
| AC9 | `MetaObject` with zero properties/signals/methods constructs without panic |
| AC10 | `Value::Custom(arc)` round-trips through `clone()` without panic |

## Open questions

- Should `WeakObjectRef` be a type alias or a newtype? (Depends on runtime ownership choice)
- Should `Signal` be `Send + Sync`? (Needed if objects move across threads)
- Should `ObjectBase` derive `Debug`? (Useful for testing; `dynamic_properties` contains `Value`)
