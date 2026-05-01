# Design: Core Types & Traits (`quartzite-core`)

**Issue:** —
**Date:** 2026-05-01

## Approach

`quartzite-core` is the foundation layer: pure data types, traits, and the signal/slot mechanism.
No UI, no runtime, no proc macros, no async. The crate must compile under `#![no_std]` + `extern
crate alloc`.

The current workspace is a single-crate placeholder (`quartzite` with a stub `src/lib.rs`). The
first structural step is converting the workspace to a proper multi-crate setup and creating
`quartzite-core` as a member crate.

### Chosen approach

**Composition + trait layering, no inheritance.**

- `ObjectBase` is a plain data struct held by composition inside every object type.
- `AsObject` is the object-safe accessor trait; `Object` adds the meta-system; `ObjectExt` is a
  non-object-safe blanket impl for ergonomics.
- `Signal<Args>` stores slots as `Box<dyn Fn(&Args)>` (not `FnMut`) to allow re-entrant emission
  without borrow issues.
- `Value` is a closed enum with a `Custom` escape hatch via
  `Arc<dyn CustomValue>`.
- IDs (`ObjectId`, `ConnectionId`) are newtype wrappers around `u64`, incremented via
  `AtomicU64`.
- `ReceiverGuard` is a zero-sized `Arc` token; incoming connections hold `Weak<ReceiverGuard>`.
  Dropping `ObjectBase` drops the `Arc`, breaking all weak refs.

### Rejected alternatives

| Alternative | Reason rejected |
|---|---|
| Single monolithic `ObjectTrait` | Breaks object-safety when ergonomic methods are included |
| `FnMut` slots | Prevents re-entrant `emit` without `RefCell`; excluded by spec |
| `u32` for IDs | 64-bit is required for concurrent uniqueness under `AtomicU64` |
| `std::any::TypeId` in `Value` | Requires `std`; `dyn CustomValue` with `as_any` is `no_std`-safe |

### `no_std` strategy

- Use `alloc::{string::String, vec::Vec, collections::BTreeMap, sync::Arc, boxed::Box}`.
- `thread::current().id()` is a `std` API — `thread_id` field is stored as `std::thread::ThreadId`
  but gated behind `#[cfg(feature = "std")]`. The `std` feature is enabled by default; `no_std`
  builds skip thread-affinity recording (AC7 is a `std`-only acceptance criterion).
- `AtomicU64` is available in `core::sync::atomic`.

### `WeakObjectRef` representation

Defined as `type WeakObjectRef = alloc::sync::Weak<dyn Object>`. This is a type alias for now;
promoted to a newtype when `quartzite-runtime` picks its ownership model (arena vs.
`Rc<RefCell<>>`). The alias lives in `quartzite-core` so downstream crates can name it.

### `Signal` thread-safety

`Signal<Args>` is **not** `Send + Sync` in this iteration. The runtime will decide threading
semantics. Slots are `Box<dyn Fn(&Args)>` (no `Send` bound). A `Send + Sync` variant can be added
later without breaking the API.

---

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Convert workspace to multi-crate; create `quartzite-core` skeleton | `Cargo.toml` (root), `quartzite-core/Cargo.toml`, `quartzite-core/src/lib.rs` | — |
| 2 | `ObjectId` and `ConnectionId` newtypes with atomic generation | `quartzite-core/src/id.rs` | 1 |
| 3 | `ReceiverGuard` zero-sized struct + `Arc`/`Weak` usage | `quartzite-core/src/receiver_guard.rs` | 1 |
| 4 | `ObjectBase` struct (id, name, parent, children, receiver_guard, outgoing_connections, dynamic_properties, thread_id, signals_blocked) | `quartzite-core/src/object_base.rs` | 2, 3 |
| 5 | `AsObject` + `Object` traits; `ObjectExt` blanket impl | `quartzite-core/src/traits.rs` | 4 |
| 6 | `Value` enum, `CustomValue` trait, `TypeError`, `FromValue` / `IntoValue` | `quartzite-core/src/value.rs` | 1 |
| 7 | `Signal<Args>`, `ConnectionType`, `SlotEntry`, connect/disconnect/emit | `quartzite-core/src/signal.rs` | 2, 3 |
| 8 | Meta-types: `MetaObject`, `PropertyMeta`, `SignalMeta`, `MethodMeta`, `ParamMeta`, `EnumMeta`, `EnumEntry` | `quartzite-core/src/meta.rs` | 6 |
| 9 | `lib.rs` public API surface — re-exports, feature flags, `#![no_std]` gate | `quartzite-core/src/lib.rs` | 1–8 |

Total: 9 tasks. Within the 7-task guideline only when treating tasks 1 and 9 as infrastructure
bookends. All tasks are narrow and logically cohesive; no further split is warranted.

---

## Risks

- **`no_std` breakage at `thread_id`:** `std::thread::ThreadId` is not available in `no_std`.
  Mitigation: gate the field and `is_on_current_thread` behind `#[cfg(feature = "std")]`; the
  feature is on by default so AC7 passes in normal builds.
- **`WeakObjectRef = Weak<dyn Object>` object-safety:** `dyn Object` requires `Object` to be
  object-safe. The spec mandates this; any method added to `Object` later that breaks object-safety
  will require a companion non-object-safe extension trait. Mitigation: enforce object-safety via a
  compile-time `fn _assert_object_safe() { let _: Box<dyn Object>; }` in a test.
- **`Signal` re-entrancy:** Slots stored as `Box<dyn Fn(&Args)>` allows calling `emit` inside a
  slot (re-entrant emission). The slot list must be snapshotted before iteration (clone the `Vec`
  of entries) to avoid borrow issues if a slot disconnects itself. Mitigation: `emit` clones the
  slot index before iterating.
- **`SingleShot` race under concurrent emit:** `Signal` is not `Send`; concurrent emission is not
  supported in this iteration. Mitigation: documented limitation; no extra locking needed.
- **`Value::List` / `Value::Map` recursive ownership:** `List` contains `Vec<Value>`, `Map`
  contains `BTreeMap<String, Value>`. Deep cloning is O(n). Mitigation: no lazy cloning in v1;
  note in docs.

---

## Test Design

Tests live in the same file (`#[cfg(test)]` module) unless otherwise noted.

### Task 2 — `ObjectId` / `ConnectionId`

- Location: `quartzite-core/src/id.rs` `#[cfg(test)]`
- Scenarios:
  - `new_returns_distinct_sequential` — two sequential `ObjectId::new()` calls produce different values (AC1 sequential)
  - `new_returns_distinct_concurrent` — spawn N threads each calling `ObjectId::new()` once; collect into a `HashSet`, assert len == N (AC1 concurrent)
  - Same pair for `ConnectionId`

### Task 3 — `ReceiverGuard`

- Location: `quartzite-core/src/receiver_guard.rs` `#[cfg(test)]`
- Scenarios:
  - `weak_upgrades_while_arc_alive` — `Weak::upgrade()` returns `Some` before drop
  - `weak_returns_none_after_arc_dropped` — drop `Arc<ReceiverGuard>`, then `Weak::upgrade()` returns `None` (AC2)
  - `concurrent_drop_and_upgrade` — drop from one thread while another calls `upgrade()` in a loop; must not panic

### Task 4 — `ObjectBase`

- Location: `quartzite-core/src/object_base.rs` `#[cfg(test)]`
- Scenarios:
  - `new_records_thread_id` — `ObjectBase::new()` and `is_on_current_thread()` return `true` (AC7, `std` feature)
  - `signals_blocked_default_false` — fresh base has `signals_blocked == false`
  - `dynamic_properties_empty_on_new` — no properties initially

### Task 5 — Traits

- Location: `quartzite-core/src/traits.rs` `#[cfg(test)]` + `tests/object_safety.rs`
- Scenarios:
  - `object_safety_as_object` — compile-time test `let _: Box<dyn AsObject>;`
  - `object_safety_object` — compile-time test `let _: Box<dyn Object>;`
  - `downcast_ref_correct_type` — `ObjectExt::downcast_ref::<T>()` returns `Some` (AC8)
  - `downcast_ref_wrong_type` — returns `None` (AC8)

### Task 6 — `Value`

- Location: `quartzite-core/src/value.rs` `#[cfg(test)]`
- Fixtures: helper `fn int_val() -> Value { Value::Int(42) }` etc.
- Scenarios:
  - `int_try_into_i32_ok` — `Value::Int(42)` → `Ok(42i32)` (AC3)
  - `string_try_into_i32_err` — `Value::String("x".into())` → `Err(TypeError::TypeMismatch)` (AC3)
  - `custom_clone_no_panic` — `Value::Custom(arc).clone()` does not panic (AC10)
  - `null_is_default` — `Value::default() == Value::Null`
  - `list_round_trip` — build `Value::List`, retrieve elements by index
  - `map_round_trip` — build `Value::Map`, retrieve by key
  - Use `rstest` for parameterized `FromValue` / `IntoValue` round-trips

### Task 7 — `Signal`

- Location: `quartzite-core/src/signal.rs` `#[cfg(test)]`
- Scenarios:
  - `emit_calls_connected_direct_slots` — connect two slots; emit; both called (AC4)
  - `disconnect_removes_slot` — connect, disconnect, emit; disconnected slot not called (AC5)
  - `single_shot_called_once` — `SingleShot` slot called on first emit; not called on second (AC6)
  - `emit_with_no_slots_does_not_panic` — emit on empty signal
  - `reentrant_emit_does_not_deadlock` — slot calls `emit` again on same signal; completes without deadlock
  - `receiver_guard_breaks_connection` — connect with `Weak<ReceiverGuard>`; drop the `Arc`; emit; slot not called

### Task 8 — `MetaObject`

- Location: `quartzite-core/src/meta.rs` `#[cfg(test)]`
- Scenarios:
  - `empty_meta_object_constructs` — zero properties, signals, methods — no panic (AC9)
  - `property_meta_flags_readable_writable` — construct `PropertyMeta` with flags, read them back
  - `enum_meta_entry_lookup` — `EnumMeta` with two entries; lookup by name

---

## Open Questions

- **`WeakObjectRef` as alias vs. newtype:** Type alias is simpler now but may need to become a
  newtype when `quartzite-runtime` picks arena vs. `Rc<RefCell<>>`. Recommend deciding before
  `quartzite-runtime` design starts.
- **`Signal: Send + Sync`:** Not enforced here. If objects must cross thread boundaries, all slots
  would need `+ Send`. Recommend deferring until the threading model is settled in
  `quartzite-runtime`.
- **`ObjectBase: Debug`:** Useful for tests (`dynamic_properties` contains `Value`, which can
  contain `dyn CustomValue`). Requires `CustomValue: Debug`. Recommend adding `Debug` behind a
  `derive` with a manual fallback for `dyn CustomValue`, gated on `feature = "debug"` or derived
  unconditionally with a `fmt::Debug` supertrait on `CustomValue`.
- **Workspace structure:** The current `Cargo.toml` is a single-package manifest. Task 1 converts
  it to a workspace; confirm whether the top-level `quartzite` facade crate should be created now
  or deferred until all member crates exist.
