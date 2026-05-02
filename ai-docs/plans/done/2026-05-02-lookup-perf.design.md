# Design: Lookup Performance & API Improvements

**Issue:** user description
**Date:** 2026-05-02

## Approach

Three independent improvements, each self-contained. They can be developed on
sub-branches and landed in any order; there are no cross-dependencies.

### 1 — IndexMap for Signal slot storage (AC1–AC3)

Replace the three `Vec` fields in `Signal<Args>` with `IndexMap<ConnectionId, …>`.
`ConnectionId` becomes the map key, and the `id` field is removed from `SlotEntry`
and the inner structs.

**Why IndexMap?** `IndexMap` preserves insertion order for iteration (required for
deterministic emission), and gives O(1) `shift_remove` for disconnect. The
`no_std`-compatible `indexmap = { version = "2", default-features = false }` uses
`hashbrown` internally — no `std` gate needed. `ConnectionId` already derives
`Hash + Eq` (verified in `quartzite-core/src/id.rs` line 76), so no prerequisite
step is needed before switching to `IndexMap`.

**emit / SingleShot cleanup:** Replace the current index-walking loop with a
two-pass approach:

1. Iterate over all entries and call every callback.
2. After the loop, call `retain(|_, e| e.conn_type != SingleShot)`.

This is safe because `emit` already takes `&mut self`, preventing re-entrant
mutation.

**disconnect:** `slots.shift_remove(&id)` — O(1). Same for `queued_slots` and
`auto_slots`.

**Rejected alternative:** keeping `Vec` with a `swap_remove` on position found by
linear scan — O(n) and breaks insertion order.

**`DynQueuedSlot::id()` and `DynAutoSlot::id()` removal:** With the map key as the
ID, these trait methods become dead code. They are removed from the traits and the
concrete impls.

---

### 2 — Name index in ObjectTree (AC4–AC9)

Add a `by_name: HashMap<String, Vec<ObjectId>>` field alongside `forward`.
`ObjectBase::name` becomes `Option<String>`, with `None` meaning unnamed.

**Naming API:**
- `ObjectBase::new()` → `name: None`
- `ObjectBase::named(s)` → `name: Some(s.into())`
- `ObjectBase::name()` getter → `Option<&str>` (replaces the public `pub name: String` field)
- `ObjectTree::rename(id, impl Into<String>)` → sets `name` to `Some(name)`, updates `by_name`
- `ObjectTree::clear_name(id)` → sets `name` to `None`, removes from `by_name`
- `insert` reads the initial name from `obj.object_base().name` and registers it
- `remove_node` removes the object from `by_name`

**find_by_name:** signature changes to `fn find_by_name(&self, name: &str) -> &[ObjectId]`,
returning a slice of the vec in `by_name`, or `&[]` when absent (no allocation).
This is a breaking change; all callers (listed below) must be updated.

**Duplicate names:** multiple objects may share a name; the `Vec<ObjectId>` stores
them in insertion order.

**Unnamed / empty-string distinction:** `None` is never inserted into `by_name`.
`Some("")` is inserted under the key `""`. Thus `find_by_name("")` returns only
objects explicitly named `""`, never unnamed objects.

**`ObjectBase::name` visibility:** Change from `pub name: String` to a private
`name: Option<String>` field with `pub fn name(&self) -> Option<&str>`.
`ObjectExt::set_name` is **removed** from `traits.rs`. With `name: Option<String>`
as a private field, there is no safe direct mutation path. All name changes must go
through `ObjectTree::rename` or `ObjectTree::clear_name`. No production callers exist
for `set_name` (confirmed: only its own definition at `traits.rs:162-163`).

**Callers of `find_by_name` that need updating (return type change):**
- `quartzite-runtime/src/object_tree.rs` — `pub fn find_by_name` (definition)
- `quartzite-runtime/src/object_tree.rs` test module — `find_by_name_returns_correct_id`, `find_by_name_returns_none_when_absent`
- `quartzite-runtime/tests/object_tree.rs` — same two tests

**Callers of `ObjectBase::name` / `ObjectExt::name()` that need updating:**
- `quartzite-core/src/traits.rs` — `fn name(&self) -> &str` / `fn set_name` (change to `Option<&str>`)
- `quartzite-core/src/traits.rs` test `object_ext_name_round_trip`
- `quartzite-core/src/object_base.rs` test `named_sets_name`
- `quartzite-runtime/tests/object_tree.rs` — `LogObj::drop` accesses `self.base.name.clone()`
- `quartzite-runtime/src/object_tree.rs` — `with` doc example clones `obj.object_base().name`
- `quartzite-runtime/src/object_tree.rs` — `find_by_name` implementation body

---

### 3 — match-based lookup for static meta accessors (AC10–AC13)

Add fn-pointer fields to `MetaObject` and `EnumMeta`. The macro generates the
dispatch functions; the struct stores pointers to them.

**Why fn pointers instead of closures?** Both structs are `Copy + 'static` (stored in
`static` items). Fn pointers (`fn(&str) -> Option<…>`) are `Copy + 'static`; closures
with captured state are not.

**`MetaObject` new fields:**

```rust
pub lookup_property:  fn(&str) -> Option<PropertyMeta>,
pub lookup_signal:    fn(&str) -> Option<SignalMeta>,
pub lookup_method:    fn(&str) -> Option<MethodMeta>,
pub lookup_enum:      fn(&str) -> Option<EnumMeta>,
```

The existing `property()`/`signal()`/`method()`/`enum_meta()` methods delegate to
these fn pointers instead of calling `iter().find()`.

**`EnumMeta` new fields:**

```rust
pub lookup_entry_by_name:  fn(&str)  -> Option<EnumEntry>,
pub lookup_entry_by_value: fn(i64)   -> Option<EnumEntry>,
```

The existing `entry_by_name()` / `entry_by_value()` methods delegate to these.

**`MetaObject::new()` — open question resolved:** extend the signature with four
additional fn-pointer parameters rather than switching to struct-literal construction.
Rationale: `MetaObject::new` is called in many places (test files, `STUB_META` statics,
macro-generated code) — all as a `const fn`. Adding parameters preserves the `const`
call path and avoids a global struct-literal migration. All call sites are updated.

The macro emits a regular `fn` dispatch function for each lookup (not `const fn` —
Rust stable does not support `const fn` with `match` on `&str` comparisons). For
empty slices the function body is just `fn(_: &str) -> Option<…> { None }` — a
shared static helper avoids code bloat (see task 3b below).

**`EnumMeta::new()` — same approach:** add two fn-pointer parameters.

**EnumMeta construction in `#[meta_enum]`:** currently uses struct literal syntax
(`::quartzite_core::EnumMeta { name: …, entries: … }`). Switch to `EnumMeta::new(…)`
with the two new fn-pointer parameters, and add the generated dispatch functions.

**Callers of `MetaObject::new` that need updating:**
- `quartzite-core/src/traits.rs` — `DUMMY_META` static
- `quartzite-core/src/meta.rs` — docs + tests (14 call sites)
- `quartzite-runtime/src/factory.rs` — `TEST_META`
- `quartzite-runtime/src/object_tree.rs` — `STUB_META`
- `quartzite-runtime/tests/object_tree.rs` — `STUB_META`, `LOG_META`
- `quartzite-runtime/tests/factory.rs` — `FOO_META`
- `quartzite-macros/src/object_impl/codegen.rs` — generated `MetaObject::new(…)` call

For all hand-written statics with empty slices, pass the shared no-op helpers.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `indexmap` dependency to `quartzite-core/Cargo.toml` | `quartzite-core/Cargo.toml` | — |
| 2 | Replace `Vec<SlotEntry>` / `Vec<DynQueuedSlot>` / `Vec<DynAutoSlot>` with `IndexMap`; remove `id` from `SlotEntry` and inner structs; remove `id()` from `DynQueuedSlot`/`DynAutoSlot`; rewrite `connect_typed`, `connect_queued`, `connect_auto`, `disconnect`, `emit` | `quartzite-core/src/signal.rs` | 1 |
| 3a | Add fn-pointer fields to `MetaObject`; extend `MetaObject::new()` with four fn-pointer params; update `property()`, `signal()`, `method()`, `enum_meta()` to delegate | `quartzite-core/src/meta.rs` | — |
| 3b | Add fn-pointer fields to `EnumMeta`; extend `EnumMeta::new()` with two fn-pointer params; update `entry_by_name()` / `entry_by_value()` to delegate; add `no_op` helper functions | `quartzite-core/src/meta.rs` | 3a |
| 3c | Update all hand-written `MetaObject::new(…)` / `EnumMeta::new(…)` call sites outside macros to pass no-op fn pointers | `quartzite-core/src/traits.rs`, `quartzite-core/src/meta.rs` (tests), `quartzite-runtime/src/factory.rs`, `quartzite-runtime/src/object_tree.rs`, `quartzite-runtime/tests/object_tree.rs`, `quartzite-runtime/tests/factory.rs` | 3b |
| 3d | Emit match-based lookup functions from `#[object_impl]`; pass them to `MetaObject::new(…)` | `quartzite-macros/src/object_impl/codegen.rs` | 3c |
| 3e | Emit match-based lookup functions from `#[meta_enum]`; switch from struct literal to `EnumMeta::new(…)`; pass fn pointers | `quartzite-macros/src/meta_enum/codegen.rs` | 3b |
| 4a | Change `ObjectBase::name` to `Option<String>`; update `new()`, `named()`; add `pub fn name(&self) -> Option<&str>` getter; update `ObjectExt::name()` return type; remove `ObjectExt::set_name`. **Note:** the two `AsObject` doc examples in `traits.rs` (lines 33 and 45) access `base.name` and assign to `obj.object_base_mut().name` — both break when `name` is privatised and changed to `Option<String>`. These examples must be updated here (or in task 4d): replace `base.name` with `base.name()`, and replace the direct setter with a comment noting that mutation goes through `ObjectTree::rename`. | `quartzite-core/src/object_base.rs`, `quartzite-core/src/traits.rs` | — |
| 4b | Add `by_name: HashMap<String, Vec<ObjectId>>` to `ObjectTree`; update `new()`, `insert`, `remove_node` to maintain the index | `quartzite-runtime/src/object_tree.rs` | 4a |
| 4c | Add `ObjectTree::rename` and `ObjectTree::clear_name`; change `find_by_name` return type to `&[ObjectId]` | `quartzite-runtime/src/object_tree.rs` | 4b |
| 4d | Update all callers of `find_by_name`, `name()`, direct `base.name` accesses, and affected tests | `quartzite-runtime/src/object_tree.rs` (doc example at line 95 `obj.object_base().name.clone()`; inline test at ~line 315 with direct `.name` access; test module), `quartzite-runtime/tests/object_tree.rs`, `quartzite-core/src/object_base.rs` (tests), `quartzite-core/src/traits.rs` (AsObject doc examples, lines 33 and 45) | 4c |

## Risks

- **Breaking `find_by_name` return type (`Option<ObjectId>` → `&[ObjectId]`):** callers
  using `Some(id)` pattern will fail to compile until updated — caught by `cargo build`,
  not a silent regression.
- **Breaking `MetaObject::new` arity:** all 14+ call sites fail to compile until updated.
  Task 3c addresses them all; `cargo build` catches any missed sites.
- **`ObjectBase::name` type change (`String` → `Option<String>`):** direct field access
  (`base.name`) is pub today; after the change it becomes private, forcing callers to the
  getter. Callers in `LogObj::drop` and the `with` doc example must be migrated.
  `ObjectExt::name()` return type change (`&str` → `Option<&str>`) is also breaking.
- **IndexMap `no_std` correctness:** `indexmap` v2 with `default-features = false` compiles
  without `std`; verified via the crate's own docs. The `std`-gated fields (`queued_slots`,
  `auto_slots`) use `#[cfg(feature = "std")]` as before.
- **`EnumMeta` fn pointer in `#[meta_enum]` — currently struct literal:** switching to
  `EnumMeta::new(…)` requires the `const fn` to accept fn-pointer parameters. All fields
  in `EnumMeta::new` must remain `const`-compatible; fn pointers are `const`-compatible.
- **Emission order preservation in `Signal` after IndexMap migration:** `IndexMap`
  preserves insertion order; existing tests (`all_pre_connected_direct_slots_fire`) will
  catch any regression. The `retain`-based SingleShot cleanup does not reorder entries.

## Test Design

### Task 2 — Signal IndexMap

Location: `quartzite-core/src/signal.rs` `#[cfg(test)]` module

New / updated tests:
- **`disconnect_is_o1_and_preserves_order`** — connect three slots (A, B, C), disconnect B,
  emit, verify A and C fire in that order and B does not fire.
- **`single_shot_removed_by_retain`** *(augments, does not replace, `single_shot_called_once`)* —
  connect one SingleShot and one Direct slot, emit twice; verify SingleShot fires exactly
  once and Direct fires exactly twice. This test specifically targets the two-pass `retain`
  path: it confirms that after the first emit removes the SingleShot entry via `retain`,
  a second emit does not invoke the slot again.
- **`disconnect_nonexistent_id_is_noop`** — already exists; still passes.
- Existing `emit_calls_connected_direct_slots`, `single_shot_called_once`,
  `auto_*` tests all continue to pass (regression).

Fixtures: same `Arc<AtomicU32>` / `Arc<AtomicBool>` pattern as existing tests.

### Task 3a/3b — MetaObject / EnumMeta fn pointer fields

Location: `quartzite-core/src/meta.rs` `#[cfg(test)]` module

New tests:
- **`meta_object_property_lookup_via_fn_pointer`** — build a `MetaObject` with a
  concrete lookup fn; call `property("count")` and `property("missing")`.
- **`enum_meta_entry_by_name_via_fn_pointer`** — same for `entry_by_name`.
- **`enum_meta_entry_by_value_via_fn_pointer`** — same for `entry_by_value`.
- No-op path: construct with `no_op` helpers and verify `None` is returned.

### Task 3d — macro lookup generation

Location: `quartzite-macros/src/object_impl/codegen.rs` `#[cfg(test)]` module

New tests (token-string assertions, not compile-check):
- `lookup_property_fn_emitted` — verify `__lookup_property_Foo` function appears in output.
- `lookup_signal_fn_emitted` — same for `__lookup_signal_Foo`.
- Match arm present for each declared property/signal name.
- Wildcard `_ => None` arm present.

### Task 3e — meta_enum lookup generation

Location: `quartzite-macros/src/meta_enum/codegen.rs` `#[cfg(test)]` module

New tests:
- `lookup_entry_by_name_fn_emitted` — token-string check for `__lookup_entry_by_name_Color`.
- `lookup_entry_by_value_fn_emitted` — same for `__lookup_entry_by_value_Color`.

### Task 4b/4c — ObjectTree name index

Location: `quartzite-runtime/src/object_tree.rs` `#[cfg(test)]` module  
And: `quartzite-runtime/tests/object_tree.rs`

New tests (AC4–AC9):
- **`new_object_name_is_none`** — `ObjectBase::new()` → `name() == None`.
- **`named_object_name_is_some`** — `ObjectBase::named("foo")` → `name() == Some("foo")`.
- **`rename_updates_index`** — insert unnamed object, rename to `"bar"`, verify
  `find_by_name("bar")` contains the id and `find_by_name("")` does not.
- **`clear_name_removes_from_index`** — insert named object, clear name, verify
  `find_by_name(original_name)` is empty and `name() == None`.
- **`find_by_name_returns_all_with_same_name`** — insert two objects named `"foo"`,
  verify `find_by_name("foo")` contains both in insertion order.
- **`find_by_name_empty_string_vs_unnamed`** — insert one unnamed (`None`) and one
  explicitly named `""` object; `find_by_name("")` returns only the `""` one.
- **`destroy_removes_from_by_name`** — insert named object, destroy, verify
  `find_by_name` returns empty slice.
- **`rename_old_name_removed`** — rename from `"a"` to `"b"`; `find_by_name("a")`
  empty, `find_by_name("b")` contains id.

Fixtures: extend existing `StubObject` and `Stub` helpers to accept
`Option<&str>` for name; or add `unnamed()` constructor alongside `named()`.

## Open questions

- None. The spec's open question (`MetaObject::new()` extension vs struct literal) is
  resolved in favour of extending `new()` with fn-pointer parameters (see Approach §3).
