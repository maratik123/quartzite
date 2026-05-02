# inline-simple-fns

**Source:** user description
**Date:** 2026-05-02

## Scope

Add `#[inline]` to every simple, non-generic function across `quartzite-core`,
`quartzite-runtime`, and `quartzite-macros` codegen. "Simple" means: no internal branches
or loops, at most one function call, and inlining does not lead to binary bloat.

Functions that will be monomorphized (generic functions, blanket-impl trait methods) are
**excluded** — the compiler already has their bodies for inlining.

### Hand-written functions

| File | Functions |
|------|-----------|
| `quartzite-core/src/id.rs` | `ObjectId::{new, raw}`, `ObjectId::default`, `ConnectionId::{new, raw}`, `ConnectionId::default` |
| `quartzite-core/src/object_base.rs` | `ObjectBase::{id, name, receiver_guard, set_name_raw}`, `ObjectBase::default` |
| `quartzite-core/src/meta.rs` | `PropertyFlags::{none, read_write, read_only}`, `PropertyFlags::default`, `PropertyMeta::new`, `ParamMeta::new`, `SignalMeta::new`, `MethodMeta::new`, `EnumEntry::new`, `EnumMeta::new`, `EnumMeta::{entry_by_name, entry_by_value}`, `MetaObject::new`, `MetaObject::{property, signal, method, enum_meta}`, all six `noop_lookup_*` free functions |
| `quartzite-core/src/signal.rs` | `queued_dispatcher` (free function) |
| `quartzite-runtime/src/event_loop.rs` | `EventLoop::{is_running, sender}`, `EventLoop::default` |
| `quartzite-runtime/src/timer.rs` | `Timer::is_running` |
| `quartzite-runtime/src/factory.rs` | `ObjectFactory::default` (production impl only) |
| `quartzite-runtime/src/object_tree.rs` | `ObjectTree::default` (production impl only) |

### Proc-macro generated functions (codegen changes)

| Codegen file | Generated functions to annotate |
|---|---|
| `quartzite-macros/src/extend/codegen.rs` | `As{Self}` self-ref accessor pair (`fn acc(&self) { self }`, `fn acc_mut(&mut self) { self }`); `AsObject::{object_base, object_base_mut, as_any, as_any_mut}`; parent-chain delegation pair; mixin leaf accessor pair |
| `quartzite-macros/src/object_impl/codegen.rs` | `Object::{meta_object, read_property, write_property, invoke_method, connect_signal}`; `__meta_init_Foo()` |
| `quartzite-macros/src/meta_enum/codegen.rs` | `IntoValue::into_value` |

Generated functions with `match` branches are **excluded**: `__lookup_*`,
`__read_property_*`, `__write_property_*`, `__connect_signal_dynamic_*`, `FromValue::from_value`.

## Out of scope

- Generic functions and blanket-impl trait methods (`ObjectRef<T>`, `WeakRef<T>`, `Signal<Args>`,
  `ObjectExt` default methods) — monomorphized, no cross-crate inlining benefit
- Test-only `AsObject` impls inside `#[cfg(test)]` modules
- Any generated or hand-written function with branches, loops, or more than one function call

## Deferred

None.

## Key decisions

| Question | Decision |
|---|---|
| Include `ObjectId::new` / `ConnectionId::new`? | Yes — single `fetch_add` call, no branches/loops |
| Include `Default::default` wrappers? | Yes — single call, eliminates one indirection level, no bloat |
| Include `EventLoop::sender` (clone)? | Yes — `Sender::clone` is an Arc increment, ~2 instructions |
| Include `EventLoop::is_running` / `Timer::is_running` (atomic load)? | Yes — single atomic instruction |
| Include generic / monomorphized functions? | No — compiler already has body; `#[inline]` is redundant |
| Include proc-macro generated simple functions? | Yes — they are non-generic delegation shims that cross crate boundaries |
| `IntoValue::into_value` from `#[MetaEnum]`? | Yes — `Value::Int(self as i64)`, no branches |
| `FromValue::from_value` from `#[MetaEnum]`? | No — has `if let` + `match` (branches) |
| `__lookup_*` generated functions? | No — all have `match` arms (branches) |

## Technical constraints

- All functions must remain compilable: `cargo build` must pass after each file edit.
- `cargo clippy -- -D warnings` must pass: clippy does not warn about `#[inline]` on simple
  functions, but do not introduce any new lint violations.
- `#![deny(missing_docs)]` is in effect; do not add or remove doc comments.
- `cargo fmt` must be run after edits; `#[inline]` goes on the line immediately before `pub`/`fn`.
- Codegen changes emit `#[inline]` inside `quote!` blocks; the attribute must appear on the
  line immediately before the `fn` keyword in the generated token stream.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Every hand-written function listed in the Scope table has `#[inline]` added |
| AC2 | Every simple generated function listed in the Scope table is emitted with `#[inline]` by its codegen |
| AC3 | No `#[inline]` is added to generated functions with branches (`__lookup_*`, `__read_property_*`, `__write_property_*`, `__connect_signal_dynamic_*`, `FromValue::from_value`) |
| AC4 | No `#[inline]` is added to generic functions, blanket-impl methods, or test-only code |
| AC5 | `cargo build` succeeds with no errors or warnings |
| AC6 | `cargo clippy -- -D warnings` passes clean |
| AC7 | `cargo fmt -- --check` reports no formatting drift |
| AC8 | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` passes clean |
| AC9 | `cargo test` — all existing tests remain green; codegen tests verify the `#[inline]` token is present in emitted output |
| AC10 | `AGENTS.md` Code Style section documents the `#[inline]` rule for simple functions so future contributors apply it consistently |

## Open questions

None.
