# Design: ObjectTree query enhancements

**Issue:** #54
**Date:** 2026-05-06

## Approach

Three independent but coordinated additions:

1. **`find_by_name_in`** — a BFS (level-order) traversal of `children_map` starting at `root`. BFS naturally visits nodes level by level, so matches are collected in ascending-depth order without a sort step. Unknown `root` returns an empty `Vec` (same style as `find_by_name`). This lives entirely in `quartzite-runtime/src/object_tree.rs` with no new dependencies.

   Concretely: initialise a `VecDeque<ObjectId>` with `root`. On each iteration, pop the front node, check its name against the query (push to result if it matches), then extend the deque with that node's children (from `children_map`). Repeat until the deque is empty. Because all nodes at depth *d* are enqueued before any node at depth *d+1*, the result `Vec` is already in shallowest-first order with insertion order preserved within each level.

2. **`Option<String>` value conversions** — the signal payload `(Option<String>, Option<String>)` must round-trip through `Value`. `Value::Null` maps to `None`; `Value::String(s)` maps to `Some(s)`. Adding `IntoValue` / `FromValue` impls for `Option<String>` in `quartzite-core/src/value.rs` is the clean, reusable approach. No new `Value` variant is needed.

3. **`name_changed` built-in signal** — the signal must appear on every `#[object]` type without requiring the user to declare it. The built-in signal is stored as a real `Signal<(Option<String>, Option<String>)>` field named `name_changed` inside `ObjectBase` (not in the user struct). This is the correct placement because `ObjectBase` is the common base carried by every object, and the field can be reached uniformly via `this.object_base_mut().name_changed` without any user-struct field access.

   The codegen in `quartzite-macros/src/object/codegen.rs` synthesises a phantom `SignalField { ident: "name_changed", builtin: true, args_ty: (Option<String>, Option<String>) }` and prepends it to the user-declared `ir.signals` before generating the `__SIGNALS__`, `__connect_signal_dynamic_`, and `__emit_signal_` entries. No typed convenience wrappers (`emit_name_changed`, `connect_name_changed_auto`, `connect_name_changed_queued`) are generated — the tree calls the built-in via `obj.emit_signal("name_changed", &[...])` on `&mut dyn Object`.

   Concretely: `SignalField` gains a `builtin: bool` flag. Codegen emits entries in `__SIGNALS__` and the `__emit_signal_` dispatch arm for all signals (user + built-in). For built-in signals the `__emit_signal_` arm routes through `this.object_base_mut().name_changed` instead of `this.name_changed`. Similarly, the `__connect_signal_dynamic_` arm for the built-in routes through `this.object_base_mut().name_changed`. The `emit_<sig>` / `connect_<sig>_auto` / `connect_<sig>_queued` wrappers are only emitted for `!builtin` signals.

   Because the built-in is prepended, the signal slice is never empty once synthesis runs. The existing `if signals.is_empty()` guards in `emit_signal_wrappers`, `emit_connect_auto_wrappers`, and `emit_connect_queued_wrappers` test the full slice including built-ins, so they would never fire — these guards must instead check whether any `!builtin` signal exists (i.e., `signals.iter().any(|s| !s.builtin)`). This ensures the wrapper `impl` blocks are only emitted when there are actual user-declared signals.

4. **Signal emission in `rename` / `clear_name`** — after the index update, `ObjectTree` calls `self.with_mut(id, |obj| obj.emit_signal("name_changed", &[old_val, new_val]))`. `Value::Null` represents `None`, `Value::String(s)` represents `Some(s)`. Emission happens after index mutation so observers see a consistent state (spec constraint).

   In `rename`: emission is appended after the name-index insertion, so it fires for every real rename (old name differs from new, or anonymous → named). The existing early-return for same-name no-ops and unknown-id cases already prevents spurious emission.

   In `clear_name`: the current code has three arms — `None` (id not in tree, returns early), `Some(Some(old_name))` (had a name, removes from index), and `Some(None)` (already anonymous, falls through without returning). The `Some(None)` arm does NOT return early — it falls through to `self.with_mut(...)`. To prevent emitting `name_changed` for an already-anonymous object, the signal emission call must be scoped inside the `Some(Some(old_name))` arm only (not placed after the match block), or an explicit `return` must be added for the `Some(None)` arm before the emit. The preferred approach is to add an explicit `return` in the `Some(None)` arm immediately after the no-op comment, making the control flow unambiguous.

**Alternatives rejected:**

- *Adding a new `Value::OptString` variant* — unnecessary; `Null` / `String` already encode the two states and avoids variant bloat in a public enum.
- *Hardcoding `name_changed` as a struct field in the derive input* — would require every `#[object]` struct to declare it, defeating the "built-in" intent.
- *A separate `ObjectTree` callback registry* — more complex, requires a new data structure, and is explicitly out of scope.
- *Putting `name_changed: Signal<...>` in the user struct via codegen* — the user struct has no such field; proc-macro codegen cannot safely inject struct fields. Storing it in `ObjectBase` instead is the correct fix: `ObjectBase` is always reachable via `object_base_mut()`, and the signal lifetime is tied to the object's own lifetime.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `IntoValue` / `FromValue` impls for `Option<String>` (Null↔None, String(s)↔Some(s)) with doc, `# Examples`, tests | `quartzite-core/src/value.rs` | — |
| 2 | Add `ObjectTree::find_by_name_in(root, name) -> Vec<ObjectId>` with BFS impl (shallowest-first) and full doc | `quartzite-runtime/src/object_tree.rs` | — |
| 3 | Add `name_changed: Signal<(Option<String>, Option<String>)>` field to `ObjectBase`; add `builtin: bool` to `SignalField`; update `parse.rs` (user signals keep `builtin: false`); synthesise the built-in in `codegen.rs` pre-pending it to `ir.signals`; route built-in arms in `__emit_signal_` and `__connect_signal_dynamic_` through `this.object_base_mut().name_changed`; gate wrapper emission on `signals.iter().any(\|s\| !s.builtin)` | `quartzite-core/src/object_base.rs`, `quartzite-macros/src/object/parse.rs`, `quartzite-macros/src/object/codegen.rs` | 1 |
| 4 | Emit `name_changed` signal in `ObjectTree::rename` (after index insertion) and `ObjectTree::clear_name` (inside `Some(Some(old_name))` arm, or add explicit `return` in `Some(None)` arm then emit after match), using `Value::Null`/`Value::String` encoding | `quartzite-runtime/src/object_tree.rs` | 2, 3 |
| 5 | Tests: `find_by_name_in` scoped correctness; `name_changed` fires on rename/clear_name; no-op rename does not fire; anon→named fires; already-anonymous `clear_name` does not fire; `destroy` does not fire | `quartzite-runtime/src/object_tree.rs` `#[cfg(test)]` | 4 |
| 6 | Tests: macro codegen emits `name_changed` in `__SIGNALS__` and `__emit_signal_` dispatch for every `#[object]` type; no typed `emit_name_changed` / `connect_name_changed_auto` / `connect_name_changed_queued` wrappers generated; `__connect_signal_dynamic_` has a `"name_changed"` arm | `quartzite-macros/src/object/codegen.rs` `#[cfg(test)]` | 3 |

Six tasks, within the limit.

## Risks

- **`Option<String>` encoding convention**: callers of `connect_signal("name_changed", …)` receive `[Value::Null | Value::String, Value::Null | Value::String]` — document this clearly in the signal's generated doc comment and in the `# Parameters` section of the `emit_name_changed` hidden helper. Mitigation: the `FromValue` impl for `Option<String>` makes round-trips explicit and testable.
- **Borrow conflict in `rename`**: `rename` calls `self.with(id, …)` (shared borrow to read old name), then `Self::remove_from_by_name(&mut self.by_name, …)` (mutable borrow), then `self.with_mut(id, …)` (mutable borrow for emit). These are sequential, not overlapping — no conflict. Confirmed by the existing `rename` code structure.
- **`clear_name` no-op path**: when the object is already anonymous (`Some(None)`), the match arm falls through to `self.with_mut(...)` without returning — it does NOT early-return. An explicit `return` must be added in the `Some(None)` arm so signal emission is never reached for already-anonymous objects. Failing to add this return would violate the no-op semantics (spec: emit only when there was actually a name to clear).
- **`__SIGNALS__` ordering change**: inserting the built-in signal at index 0 (prepend) shifts user-declared signal indices by one. Since signal lookup is by name (not index), runtime behaviour is unaffected. The only risk is tests that assert `__SIGNALS__Foo[0]` — the existing codegen tests do check indices; those tests must be updated to account for the prepended entry. Mitigation: update impacted tests in task 3 together with the codegen change.
- **`no_std` path**: `Option<String>` uses `alloc::string::String` under `no_std`. The existing `String` impls in `value.rs` already use `#[cfg(not(feature = "std"))] use alloc::string::String;` — the new impl follows the same pattern. No extra work needed.
- **`ObjectBase` size increase**: adding `Signal<(Option<String>, Option<String>)>` to `ObjectBase` grows every object by the size of one `Signal`. `Signal` is typically a small reference-counted type; the cost is acceptable and no alternative placement avoids it.

## Test Design

### Task 1 — `Option<String>` conversions (`quartzite-core/src/value.rs` `#[cfg(test)]`)

- Entry points: `Option::<String>::from_value`, `Option::<String>::into_value`
- Scenarios:
  - `None.into_value()` → `Value::Null`
  - `Some("x".to_owned()).into_value()` → `Value::String("x")`
  - `Option::<String>::from_value(Value::Null)` → `Ok(None)`
  - `Option::<String>::from_value(Value::String("x"))` → `Ok(Some("x"))`
  - `Option::<String>::from_value(Value::Int(1))` → `Err(TypeError { expected: "String or Null", … })`
  - Round-trip: `None` and `Some("hello")` survive `into_value → from_value`
- Fixtures: none needed (pure function tests)

### Task 2 — `find_by_name_in` (`quartzite-runtime/src/object_tree.rs` `#[cfg(test)]`)

- Entry point: `ObjectTree::find_by_name_in`
- Scenarios:
  - Match on root itself (root name == queried name) → `[root]`
  - Match on deep descendant, non-matching sibling excluded
  - Name exists outside subtree but not inside → returns empty
  - Unknown `root` → returns empty
  - Multiple matches within subtree → all returned
  - Subtree with no names → returns empty
  - Same name at depth 0 (root) and depth 2 (grandchild) → root appears first in result
  - Two matches at the same depth → both returned, in children-insertion order (stable within level)
- Fixtures: reuse existing `StubObject::named` helper

### Task 5 — signal emission tests (`quartzite-runtime/src/object_tree.rs` `#[cfg(test)]`)

These tests require a `StubObject` that actually records received signal emissions. A `RecordingObject` fixture (wraps `StubObject`, tracks `Vec<(String, Vec<Value>)>` of `emit_signal` calls) is needed. The existing `StubObject::emit_signal` returns `None` unconditionally — a new `RecordingObject` must override that to capture calls.

- Entry points: `ObjectTree::rename`, `ObjectTree::clear_name`
- Scenarios (map to AC4–AC9):
  - `rename(id, "new")` where old name was "old" → one emission, args `[Value::String("old"), Value::String("new")]`
  - `rename(id, "same")` (no-op) → no emission (AC7)
  - `rename` on anonymous object → args `[Value::Null, Value::String("new")]` (AC8)
  - `clear_name(id)` where old name was "old" → args `[Value::String("old"), Value::Null]` (AC6)
  - `clear_name` on already-anonymous object → no emission (AC analogue of no-op rule)
  - `destroy(id)` → no `name_changed` emission (AC9)
- Fixtures: `RecordingObject` struct implementing `Object`; `emit_signal` appends to an internal `Vec`; connected via `ObjectTree::with_mut` before asserting

### Task 6 — macro codegen tests (`quartzite-macros/src/object/codegen.rs` `#[cfg(test)]`)

- Entry point: `codegen(parse(…))` as string
- Scenarios:
  - `__SIGNALS__Foo` slice contains `"name_changed"` entry for a struct with no user signals
  - `__SIGNALS__Foo` slice contains `"name_changed"` AND a user signal for a struct with one user signal
  - `__emit_signal_Foo` dispatch function has a `"name_changed"` arm
  - No `pub fn emit_name_changed` wrapper is emitted
  - `connect_signal_dynamic` has a `"name_changed"` arm
  - No `pub fn connect_name_changed_auto` / `_queued` wrapper is emitted

## Open questions

_(none)_
