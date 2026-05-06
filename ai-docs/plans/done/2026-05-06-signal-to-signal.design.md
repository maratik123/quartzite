# Design: Signal-to-signal connections

**Issue:** #49
**Date:** 2026-05-06

## Approach

### Overview

The feature adds two orthogonal capabilities:

1. **`Object::emit_signal`** — a new method on the `Object` trait that dispatches a signal by name
   using a `&[Value]` argument slice.  Codegen in `quartzite-macros` generates the implementation
   for every `#[derive(Object)]` type.

2. **`connect_signal_to_signal`** — a free function (dynamic path) and a generic typed free function
   (typed path) that installs a forwarding callback on the source signal.  Both paths are `std`-only.

### `Object::emit_signal` method

Added to the `Object` trait in `quartzite-core/src/traits.rs`:

```text
fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()>;
```

Returns `Some(())` when the signal name is known and arity matches; `None` otherwise.  The
implementation is purely a dynamic dispatch bridge — it does NOT re-check `signals_blocked`
(the forwarding callback that calls it already handles blocking on the source object side via
the generated `emit_<signal>` wrapper, which goes through the `emit!` macro).

Codegen in `emit_object_impl` (in `quartzite-macros/src/object_impl/codegen.rs`) adds a
sixth delegation shim, and a new helper function `emit_emit_signal` is added to
`quartzite-macros/src/object/codegen.rs` that generates the match body for each `#[derive(Object)]`
type.  The generated hidden-mod function is named `__emit_signal_<TypeName>`.

Generated shape for a type with signals `clicked: Signal<()>` and `moved: Signal<(i32, i32)>`:

```rust
// inside __quartzite_TypeName hidden mod
fn __emit_signal_TypeName(
    this: &mut TypeName,
    name: &str,
    args: &[quartzite_core::Value],
) -> Option<()> {
    match name {
        "clicked" => {
            if args.len() != 0 { return None; }
            quartzite_core::emit!(this.clicked, &());
            Some(())
        }
        "moved" => {
            if args.len() != 2 { return None; }
            let arg0 = quartzite_core::FromValue::from_value(args[0].clone()).ok()?;
            let arg1 = quartzite_core::FromValue::from_value(args[1].clone()).ok()?;
            quartzite_core::emit!(this.moved, &(arg0, arg1));
            Some(())
        }
        _ => None,
    }
}
```

The `emit!` macro already guards `signals_blocked`, so `emit_signal` correctly suppresses
emission when signals are blocked on the target object.

### `SignalConnectionError` error type

New `std`-gated enum in `quartzite-core/src/signal.rs` (or a new `quartzite-core/src/connect.rs`
module — the latter is preferred to keep `signal.rs` under the 500-line soft limit):

```rust
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SignalConnectionError {
    #[error("unknown signal `{0}` on source object")]
    UnknownFromSignal(String),
    #[error("unknown signal `{0}` on target object")]
    UnknownToSignal(String),
    #[error("arity mismatch: source signal has {from} parameters, target has {to}")]
    ArityMismatch { from: usize, to: usize },
    #[error("type mismatch at parameter {index}: source `{from}`, target `{to}`")]
    TypeMismatch { index: usize, from: String, to: String },
}
```

### `connect_signal_to_signal` (dynamic path)

Free function in `quartzite-core/src/connect.rs` (new file, `std`-only module):

```rust
#[cfg(feature = "std")]
pub fn connect_signal_to_signal(
    from: &mut dyn Object,
    from_signal: &str,
    to: Arc<Mutex<dyn Object + Send>>,
    to_signal: &str,
    conn_type: ConnectionType,
) -> Result<ConnectionId, SignalConnectionError>
```

**Algorithm:**

1. Look up `from_signal` via `from.meta_object().signal(from_signal)` → `Err(UnknownFromSignal)` if `None`.
2. Lock `to`, look up `to_signal` via `to_meta.signal(to_signal)` → `Err(UnknownToSignal)` if `None`.
3. Validate arity: `from_meta.params.len() == to_meta.params.len()` → `Err(ArityMismatch)` if not.
4. Validate each `type_name` pair → `Err(TypeMismatch { index, from, to })` for the first mismatch.
5. Downgrade `to` to `Weak<Mutex<dyn Object + Send>>`.
6. Build forwarding callback `move |args: &[Value]| { ... }` that:
   - Upgrades the `Weak`; silently returns if `None`.
   - Locks the mutex (`.lock().ok()?`).
   - Calls `to_obj.emit_signal(to_signal_name, args)` — drops the result.
7. Connect the callback to `from_signal` via `from.connect_signal(from_signal, Box::new(cb))`.

**ConnectionType routing within the callback:**

The `conn_type` governs how the forwarding itself is posted:

- `Direct`: callback is installed as a plain `connect_signal` (which uses `Signal::connect` → Direct slot).
- `Queued` / `SingleShot`: forwarding closure posts to the dispatcher or fires once — handled by wrapping accordingly.
- `Auto`: at callback-fire time, compare `std::thread::current().id()` against `to`'s owner thread id
  (retrieved from `to_obj.object_base().thread_id` while holding the lock); post to
  `queued_dispatcher()` when they differ.

Because `connect_signal` only installs `Direct` slots, the `conn_type` dispatch is handled
inside the closure body, not by calling `connect_auto`/`connect_queued` directly.  This is
necessary because `dyn Object` does not expose typed `Signal<Args>` fields — only the dynamic
`connect_signal` interface is available.

The forwarding callback signature (`Fn(&[Value]) + Send + Sync`) already satisfies `SignalCallback`.

**Liveness:** `Weak<Mutex<dyn Object + Send>>` — the connection silently skips when all strong
`Arc` holders are released.  The `to` object's `ReceiverGuard` is *not* used here because
the callback does not go through `connect_auto`/`connect_queued`; instead the `Weak` upgrade
provides the equivalent liveness guarantee.

### Typed API (`connect_signals`)

A generic free function in `quartzite-core/src/connect.rs`:

```rust
#[cfg(feature = "std")]
pub fn connect_signals<From, To, Args>(
    from_obj: &mut From,
    from_signal: fn(&mut From) -> &mut Signal<Args>,
    to: Arc<Mutex<To>>,
    to_signal_name: &str,
    conn_type: ConnectionType,
) -> Result<ConnectionId, SignalConnectionError>
where
    From: Object,
    To: Object + Send + 'static,
    Args: Clone + Send + 'static,
```

This function:

1. Gets `from_signal_field = from_signal(from_obj)` to extract the `Signal<Args>` field.
2. Looks up `to_signal_name` on `to` (after locking) for validation.
3. Validates arity (number of `Args` tuple elements vs `to_meta.params.len()`).  The arity of
   `Args` is determined at compile time — `Args: SignalArgs` (see below) or simply via the
   `SignalMeta` of the source signal obtained by a separate lookup on `from_obj.meta_object()`.

   Actually, to avoid a separate `SignalArgs` trait, the typed API accepts a `from_signal_name:
   &str` parameter for validation, rather than computing arity from the Rust type.  This is simpler
   and consistent with the dynamic path:

```rust
#[cfg(feature = "std")]
pub fn connect_signals<From, To, Args>(
    from_obj: &mut From,
    from_signal_name: &str,
    from_signal_field: fn(&mut From) -> &mut Signal<Args>,
    to: Arc<Mutex<To>>,
    to_signal_name: &str,
    conn_type: ConnectionType,
) -> Result<ConnectionId, SignalConnectionError>
where
    From: Object,
    To: Object + Send + 'static,
    Args: ArgsToValues + Clone + Send + 'static,
```

4. Validates arity and type names using both `SignalMeta`s (same logic as the dynamic path).
5. Downgrades `to` to `Weak<Mutex<To>>`.
6. Builds the forwarding closure.  Unlike the dynamic path the closure converts `&Args` to
   `&[Value]` using `IntoValue` (the same conversion done by the generated `connect_signal`
   wrappers), then calls `to_obj.emit_signal(to_signal_name, &values)`.

   For `ConnectionType::Auto` / `Queued` the same dispatch-in-closure approach is used.

7. Installs the closure via `from_signal_field(from_obj).connect(cb)` (Direct) or the auto/queued
   variants depending on `conn_type`:
   - `Direct` / `SingleShot`: use `Signal::connect_typed`.
   - `Queued`: use `Signal::connect_queued` — needs `to_obj.object_base().receiver_guard()`.
   - `Auto`: use `Signal::connect_auto` — needs `to_obj.object_base().thread_id` and
     `receiver_guard()`.

   For Queued/Auto: lock `to` once at connect time to capture `thread_id` and
   `Arc::downgrade(receiver_guard())`, then release the lock.

**Note on `Args` → `&[Value]` conversion inside the closure:**
`Args` is the tuple type of the source signal.  The closure signature differs by
`ConnectionType`:

- `Direct` / `SingleShot`: `Signal::connect_typed` passes **`&Args`** to the closure (`Fn(&Args)`).
  Call `args.to_values()` directly (already a `&Args` borrow).
- `Queued` / `Auto`: `Signal::connect_queued` / `connect_auto` pass **owned `Args`** to the
  closure (`Fn(Args)`).  To call `to_values(&self)`, borrow the owned value first:
  ```rust
  let values = (&args).to_values();
  ```
  or equivalently `args_ref.to_values()` where `let args_ref = &args;`.

In all cases the result is `Vec<Value>` which is passed as `&values` to `emit_signal`.
The conversion is done by `ArgsToValues::to_values`, which calls
`IntoValue::into_value(element.clone())` on each tuple element — same pattern as the generated
`__connect_signal_dynamic_*` function in `object/codegen.rs`.

For the typed API we need `Args: Clone + Send + 'static` to use `connect_auto` / `connect_queued`.
For Direct-only we need only `Args: 'static`.  Since `connect_signals` must support all
`ConnectionType` variants, `Args: Clone + Send + 'static` is required uniformly.

Additionally, because `emit_signal` requires `&[Value]`, the `Args` type must be decomposable
into values.  We introduce a helper trait `ArgsToValues` implemented via blanket impls for tuples
up to a reasonable arity (or via a macro).  However, a simpler approach — avoiding new public
traits — is to build the `&[Value]` conversion inside the closure by calling into the generated
`SignalCallback` path.  The cleanest approach given the current codegen:

The closure captures `to_signal_name` (a `String`), `Weak<Mutex<To>>`, and converts each
tuple element individually.  The typed API can use `Vec<Value>` as a temporary for the
`&[Value]` argument:

```rust
// inside the forwarding closure
let values: Vec<Value> = vec![
    IntoValue::into_value(args.0.clone()),
    // ... one per element
];
to_locked.emit_signal(&to_signal_name, &values)
```

For this to work generically, we need tuple decomposition.  The cleanest path without adding a
new public trait is to expose a `pub trait ArgsToValues: 'static` in `quartzite-core` with
blanket impls for `()`, `(A,)`, `(A, B)`, … up to the maximum supported arity (currently the
codebase uses up to 2 in tests).  We add impls up to arity 8 (generous but not unreasonable;
similar to `std::cmp` blanket arities).

This `ArgsToValues` trait is `pub` but `#[doc(hidden)]` — it is a codegen helper, not user-
facing API.

The trait declaration should carry a doc comment:

```rust
/// Internal helper: converts a signal argument tuple to a `Vec<Value>`.
#[doc(hidden)]
pub trait ArgsToValues: 'static {
    fn to_values(&self) -> Vec<Value>;
}
```

**Zero-arg / unit tuple:** The `()` impl must produce an empty `Vec<Value>` with **no**
`IntoValue` call:

```rust
impl ArgsToValues for () {
    #[inline]
    fn to_values(&self) -> Vec<Value> { vec![] }
}
```

No `IntoValue for ()` impl is needed or added.  `Signal<()>` works because this impl produces
an empty slice — `emit_signal("clicked", &[])` succeeds when arity is 0.

**`to_values` takes `&self`:** every impl borrows the tuple (does not move it) and produces a
`Vec<Value>` by cloning each element via `IntoValue::into_value(element.clone())`.

Alternative: skip `ArgsToValues` entirely for the typed path and instead reconstruct the
`&[Value]` differently — call the **source signal's** generated `SignalCallback` stub.  But
that would require registering a nested callback, which is circular.

Chosen approach: `ArgsToValues` sealed-ish trait, impls for arities 0–8, placed in a
`quartzite-core/src/args_to_values.rs` sub-module (or directly in `connect.rs` if line count
allows).

### `std`-feature gating

Both `emit_signal` on `Object` and `connect_signal_to_signal` / `connect_signals` are
`#[cfg(feature = "std")]`-gated.  The `emit_signal` codegen helper and the `Object` trait
method declaration are similarly gated.

The `Object` trait's `emit_signal` method is declared with a default implementation returning
`None` for `no_std` builds, which allows the `no_std` path to compile without any change to
existing `impl Object for T` blocks.  Wait — since `Object` requires `#[object_impl]` to
generate the impl, we cannot add a method to the trait without updating codegen.  The safest
approach: add `emit_signal` as a new required method in the `Object` trait and update
`emit_object_impl` to always emit it, including for `no_std`.  For `no_std` builds, the
generated body is always `None` (no match, just `None`) because the dynamic emit path is `std`-
only.  The `emit_emit_signal` helper generates a different body under `cfg(feature = "std")`.

Actually the cleanest division: **always declare** `emit_signal` on the `Object` trait (it is a
required method, not `cfg`-gated), but the **implementation body** emitted by codegen returns `None`
unconditionally on `no_std`.  Under `std` it emits the full match.  This keeps the trait object-
safe in both modes and requires no `cfg` on the trait method signature itself.

### File layout summary

| File (new or changed) | Change |
|---|---|
| `quartzite-core/src/traits.rs` | Add `fn emit_signal(&mut self, signal: &str, args: &[Value]) -> Option<()>` to `Object` trait |
| `quartzite-core/src/connect.rs` | **New file**: `SignalConnectionError`, `connect_signal_to_signal`, `connect_signals`, `ArgsToValues` |
| `quartzite-core/src/lib.rs` | Declare `pub mod connect`; re-export `SignalConnectionError`, `connect_signal_to_signal`, `connect_signals` (all `std`-gated) |
| `quartzite-macros/src/object/codegen.rs` | Add `emit_emit_signal` helper; call it from `codegen()` |
| `quartzite-macros/src/object_impl/codegen.rs` | Add `emit_signal` delegation shim to `emit_object_impl` |
| `src/lib.rs` (facade) | Add `connect_signal_to_signal`, `connect_signals`, `SignalConnectionError` to `prelude` (`std`-gated) |

### Rejected alternatives

- **Macro for typed API (`connect_signals!(from.sig => to.sig, ...)`)**:  A `macro_rules!`
  approach avoids the `ArgsToValues` trait entirely — the macro can expand directly to typed code.
  Rejected because macros are harder to discover via IDE, doc, and type-checking.  A generic
  function with explicit signal-name strings is more idiomatic and easier to test.

- **`BlockingQueued` support**: Out of scope per spec (blocked on #48).

- **Storing the `ConnectionType` in the `SlotEntry` and routing inside `Signal`**: Signal already
  routes based on which slot map the entry is in (slots, queued_slots, auto_slots).  The dynamic
  path cannot use those maps directly because it does not have a typed `Signal<Args>` reference.
  The dispatch-in-closure approach is the only option for the dynamic path.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `emit_signal` to `Object` trait (declaration + codegen for `emit_signal` shim and `__emit_signal_*` helper). **Note:** the existing test `object_impl_emits_all_five_delegations` in `quartzite-macros/src/object_impl/codegen.rs` (line ~493) checks exactly 5 delegation shims — it **must be updated to 6** when the `emit_signal` shim is added. | `quartzite-core/src/traits.rs`, `quartzite-macros/src/object/codegen.rs`, `quartzite-macros/src/object_impl/codegen.rs` | — |
| 2 | Add `ArgsToValues` trait and tuple impls (arities 0–8) | `quartzite-core/src/connect.rs` (new) | — |
| 3 | Add `SignalConnectionError` enum | `quartzite-core/src/connect.rs` | — |
| 4 | Implement `connect_signal_to_signal` (dynamic path) | `quartzite-core/src/connect.rs` | 1, 3 |
| 5 | Implement `connect_signals` (typed path) | `quartzite-core/src/connect.rs` | 1, 2, 3 |
| 6 | Wire up `quartzite-core/src/lib.rs` exports and `src/lib.rs` prelude additions | `quartzite-core/src/lib.rs`, `src/lib.rs` | 3, 4, 5 |
| 7 | Integration tests (same-thread Direct, cross-thread Auto A→B→C chain, disconnect, liveness) | `tests/signal_to_signal.rs` (new) | 1–6 |

Seven tasks, within the limit.

## Risks

- **`no_std` path compilation**: Adding `emit_signal` as a required `Object` method means every
  existing `impl Object for T` block generated by `#[object_impl]` must emit it.  The codegen
  already controls all such impls, so the change is mechanical.  Risk: low if codegen is updated
  atomically with the trait change.  Mitigation: add a `cargo build -p quartzite --no-default-features`
  step to the task checklist.

- **`Args` arity mismatch at `emit_signal` call site**: The forwarding closure passes `&[Value]`
  built from the source signal's args.  If the source and target signals have equal arity (validated
  at connect time) but different `type_name` strings (e.g. `i32` vs `i64`), the `FromValue`
  conversion inside `emit_signal` will return `None` (arity match but type mismatch at emit time).
  This is acceptable behavior — `emit_signal` returns `None` and the forwarding silently fails.
  The connect-time `type_name` validation makes this scenario rare in practice.

- **Mutex deadlock on re-entrant chains**: A→B→C is safe because each forwarding callback locks
  a different `Mutex<Object>`.  A→A (self-loop) would deadlock.  No mitigation needed per spec
  (self-loops are not a supported use case and AC9 only covers distinct objects).

- **`Auto` dispatch inside closure**: The closure must capture `to_obj.object_base().thread_id`
  at connect time (while holding the lock).  If the target object's thread identity changes after
  connection, the dispatch will be wrong — but this is documented behavior (same limitation as
  `connect_auto` which also captures thread id at connect time).

- **`connect_signal` only installs `Direct` slots**: The dynamic `connect_signal_to_signal` uses
  `from.connect_signal(...)` which calls the generated `__connect_signal_dynamic_*` function, which
  always uses `Signal::connect` (Direct).  To support `Queued` and `Auto`, the forwarding closure
  must implement the dispatch internally.  This means `SingleShot` semantics are also only possible
  by implementing them inside the closure.  The `conn_type` parameter is fully respected but
  entirely within the closure body.  **Risk**: the `SingleShot` variant is handled inside the
  closure by a shared `AtomicBool` flag; after the first fire the closure becomes a no-op.  This
  is a correctness risk.  Mitigation: the dynamic path returns a `ConnectionId` that the caller
  can use to disconnect; `SingleShot` is therefore not strictly needed for the dynamic path.
  Since the spec only requires `Direct` and `Auto` for the integration tests (AC5–AC6), and
  `SingleShot` is an edge case, the dynamic path can support all `ConnectionType` variants or
  restrict to `Direct` / `Queued` / `Auto`.  The spec does not list `SingleShot` as a named AC.
  For correctness, implement `SingleShot` in the closure.

- **`connect_signal` callback is `Fn` not `FnOnce`**: `Signal::connect` requires `Fn(&Args) + Send
  + 'static`.  A `SingleShot`-in-closure approach using `AtomicBool` is compatible with `Fn`.

- **Line count of `connect.rs`**: `ArgsToValues` impls for 8 arities + two public functions +
  error type may approach 200 lines.  Well within limits.

- **`thiserror` for `SignalConnectionError`**: `thiserror` is already a dependency of
  `quartzite-core`.  No new dependency needed.

## Test Design

### Task 1 — `Object::emit_signal` codegen

**Location:** `quartzite-macros/src/object/codegen.rs` `#[cfg(test)]` and
`quartzite-macros/src/object_impl/codegen.rs` `#[cfg(test)]`

**Entry point:** `emit_emit_signal` (codegen function), `emit_object_impl` (delegation)

**Scenarios:**
- Generated `__emit_signal_Foo` for a struct with two signals: output contains both signal name
  strings, arity guard, `FromValue` conversions, `emit!` macro calls.
- Zero-arg signal (`Signal<()>`): no `FromValue` conversions, arity guard checks `0`.
- Unknown signal name → `None` arm present.
- `emit_signal` shim appears in `impl quartzite::core::Object for Foo` block.
- Unit test in `codegen.rs` that calls `emit()` and asserts token-stream shape.

**Location (runtime behavior):** `quartzite-core/src/traits.rs` `#[cfg(test)]` using a hand-written
`Object` impl (mirrors the existing `DummyObject` pattern).

**Scenarios (runtime):**
- `emit_signal("clicked", &[])` on a type with a `clicked: Signal<()>` fires the slot.
- `emit_signal("unknown", &[])` returns `None`, slot not called.
- `emit_signal("clicked", &[Value::Int(1)])` returns `None` (arity mismatch), slot not called.

### Task 3 — `SignalConnectionError`

**Location:** `quartzite-core/src/connect.rs` `#[cfg(test)]`

**Scenarios:**
- Each variant's `Display` format matches the `#[error]` string.
- `Clone`, `PartialEq`, `Debug` derived correctly.

### Task 4 — `connect_signal_to_signal`

**Location:** `quartzite-core/src/connect.rs` `#[cfg(test)]`

**Entry point:** `connect_signal_to_signal`

**Scenarios (unit):**
- Unknown `from_signal` → `Err(UnknownFromSignal)`.
- Unknown `to_signal` → `Err(UnknownToSignal)`.
- Arity mismatch (source has 1 param, target has 2) → `Err(ArityMismatch)`.
- Type name mismatch at index 0 → `Err(TypeMismatch { index: 0, ... })`.
- Happy path, `Direct`: returns `Ok(ConnectionId)`, emitting `from_signal` fires `to_signal`.
- Happy path, `Auto` same-thread: slot called synchronously.
- After `Arc` drop: subsequent `from_signal` emission silently skips (no panic).
- After `disconnect(id)`: subsequent `from_signal` emission does not call `to_signal`.

### Task 5 — `connect_signals` (typed path)

**Location:** `quartzite-core/src/connect.rs` `#[cfg(test)]`

**Entry point:** `connect_signals`

**Scenarios:** same as Task 4 but via the typed API; no string names for `from_signal_name` in the
actual signal field selector.

### Task 7 — Integration tests

**Location:** `tests/signal_to_signal.rs` (new workspace-level integration test)

**Required-features:** `derive`, `std`

**Fixtures:** Two or three `#[derive(Extend, DeriveObject)] / #[object_impl]` types with matching
signals.

**Scenarios:**
- AC1/AC5: Direct connection, same thread — emitting fires slot.
- AC2: Unknown signal name on `connect_signal_to_signal` → `Err`.
- AC3: Successful connection returns `Ok(ConnectionId)`.
- AC4: Arity/type mismatch on connect → `Err`.
- AC6: Auto connection, cross-thread — posted to dispatcher (use a `TestDispatcher` similar to
  the one in `signal.rs` tests, but install via `set_queued_dispatcher`; mark test `#[serial]`).
- AC7: Drop the `Arc<Mutex<to>>` after connecting; emit → no panic.
- AC8: `disconnect(id)` stops forwarding.
- AC9: Three-object chain A→B→C (B has matching signal that re-emits to C) — emit A's signal,
  verify C's slot fires.
- AC10: Typed API (`connect_signals`) satisfies AC5–AC8 without string names.
- AC11: All public items reachable via `use quartzite::prelude::*`.

**`serial_test` requirement:** The cross-thread Auto test (AC6) needs the process-wide
`QueuedDispatcher` installed.  Use `#[serial]` to avoid conflicts with other tests in the same
binary.  The integration test binary is separate from unit tests so the `OnceLock` is fresh.

## Open questions

- None.
