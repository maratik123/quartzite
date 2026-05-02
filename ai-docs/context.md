# Project Context: quartzite

> Read on demand. Keep this file up to date as the project evolves.

## Purpose

`quartzite` is a **GUI/object framework for Rust**, usable from Rust and (eventually) Python. It implements an object tree, property system, signals/slots, meta-object reflection, widget hierarchy, and layout engine — all in idiomatic Rust with no binary or ABI compatibility requirement with any other framework.

## Crate Layout

| Crate | Purpose |
|---|---|
| `quartzite-core` | ObjectBase, AsObject, Object, ObjectExt, Value, Signal, MetaObject — no_std compatible |
| `quartzite-macros` | Proc-macro crate: `#[derive(Extend)]`, `#[derive(Object)]`, `#[object_impl]` |
| `quartzite-runtime` | Application, EventLoop, ObjectTree, ObjectRef, Timer |
| `quartzite-geometry` | Point, Size, Rect, Margins |
| `quartzite-events` | Event, MouseEvent, KeyEvent, EventFilter |
| `quartzite-paint` | Painter, Color, Font, Pen, Brush, Image, Path |
| `quartzite-style` | Style trait, Palette, StyleRegistry |
| `quartzite-widgets` | WidgetBase, WidgetExt, Layout, Button, Label, LineEdit, … |
| `quartzite` (facade) | Re-exports all crates; curated `prelude` module ✅ |

Python interop (`quartzite-python` via PyO3) is **deferred** — the reflection layer must not block it.

## Concept Mapping

| Concept | quartzite implementation |
|---|---|
| Object tree | `ObjectBase` + `AsObject` trait + arena ownership |
| Property system | `#[prop]` on struct fields; proc-macro generates metadata |
| Signals & slots | `Signal<Args>` struct; type-safe at compile time |
| Meta-object / reflection | `MetaObject` static struct; `Object` trait for runtime dispatch |
| Widget base | `WidgetBase` struct + `AsWidget` / `WidgetExt` traits |
| Layouts | `Layout` trait; `BoxLayout`, `GridLayout` |
| Dynamic value container | `Value` enum |

## Out of Scope

- Binary/ABI compatibility with other frameworks
- C++ single-inheritance model — replaced by traits + composition
- External code generation — replaced by proc macros
- Declarative/scripting layer — not a goal for v1; deferred

## Core Architecture

### Object Hierarchy (no inheritance)

Every type in the hierarchy uses **composition + proc-macro delegation**:

```
ObjectBase   ←  WidgetBase   ←  Button
   ↑               ↑               ↑
AsObject        AsWidget        AsWidget (generated)
                                AsObject (via blanket impl)
```

- `#[derive(Extend)] #[root]` on a struct → generates `As{TypeName}` trait + self-referential impl
- `#[base]` on a field → macro generates delegation impl for the parent trait
- `#[mixin]` on a field → adds leaf trait only; no ancestor propagation (prevents diamond problem)
- Blanket impls chain the hierarchy upward; user never manually writes delegation code

### Trait Layers

| Trait | Object-safe? | Purpose |
|---|---|---|
| `AsObject` | yes | Pure accessor: `object_base()` + `as_any()` |
| `Object` | yes | Meta-system: `read_property`, `write_property`, `invoke_method`, `connect_signal` |
| `ObjectExt` | no | Convenience methods (blanket impl on all `AsObject` types) |
| `As{X}` | yes | Generated per hierarchy level by `#[derive(Extend)] #[root]` |
| `{X}Ext` | no | Convenience methods per level (blanket impl) |

### Macro System

| Macro | Input | Output |
|---|---|---|
| `#[derive(Extend)]` | struct with `#[root]`/`#[base]`/`#[mixin]` | `As{TypeName}` trait, delegation impls |
| `#[derive(Object)]` | struct with `#[prop]`, `#[signal]` fields | property + signal metadata arrays, partial `impl Object` |
| `#[object_impl]` | impl block with `#[slot]` methods | method metadata, final `MetaObject` static, complete `impl Object` |

### Ownership Model

**Arena/SlotMap** (`ObjectId` handles, `slotmap::SlotMap<DefaultKey, Box<dyn Object>>` central store). Objects are owned by `ObjectTree`; callers hold `ObjectId` (logical u64 identity) or `ObjectRef<T>` / `WeakRef<T>` typed wrappers. `ObjectTree` is wrapped in `Mutex<ObjectTree>` in `Application` — `Object: Send` supertrait ensures soundness without unsafe.

### Signal/Slot Lifetime Safety

`ReceiverGuard` pattern: every `ObjectBase` holds `Arc<ReceiverGuard>`. Incoming connections hold `Weak<ReceiverGuard>`. When the object is dropped, all weak refs break and queued slot calls are silently discarded.

### Value System

`Value` enum: `Null | Bool | Int(i64) | Float(f64) | String | List | Map | Bytes | Object(WeakRef) | Custom(Arc<dyn CustomValue>)`.

## Key Design Decisions

| Question | Decision |
|---|---|
| ABI compatibility | Out of scope |
| Code generation | Proc macros only, no external tool |
| Inheritance model | Traits + composition; `#[base]` + blanket impls |
| Hierarchy root marker | `#[root]` explicit attribute |
| Multiple bases | One `#[base]` + N `#[mixin]`; two `#[base]` → compile error |
| `AsObject` vs `Object` | Separate: `AsObject` = pure accessor; `Object` = meta-system |
| Crate naming | `quartzite-*` |
| Python interop | Deferred; reflection layer designed to enable it later |
| Object ownership | Arena/SlotMap — `ObjectTree` + `ObjectId` + `Mutex<ObjectTree>` in Application |
| `ConnectionType::Auto` | Same-thread → Direct (sync call, args cloned); cross-thread → Queued (post to dispatcher). `ThreadId` captured at connect time; requires `Args: Clone + Send + 'static`. Gated on `feature = "std"`. |
| Signal slot storage | `IndexMap<ConnectionId, …>` — insertion-order preserved; O(1) disconnect via `shift_remove`. `std` feature propagates to `indexmap/std`; `no_std` uses `hashbrown::DefaultHashBuilder` type alias. |
| `ObjectBase::name` | `Option<String>`; `None` = unnamed (not indexed); `Some("")` = explicitly named `""` (indexed). Mutation only via `ObjectTree::rename` / `ObjectTree::clear_name`. |
| `ObjectTree::find_by_name` | Returns `&[ObjectId]` — all objects with that name, insertion order. Backed by `HashMap<String, Vec<ObjectId>>` index. |
| MetaObject / EnumMeta lookup | fn-pointer fields — O(1) match dispatch. Property/signal lookup fns generated in hidden mod by `#[derive(Object)]`; method/enum lookup fns generated by `#[object_impl]`; EnumMeta entry fns generated by `#[meta_enum]`. |

## Plans (Implementation Order)

Crate-level plans:

1. `quartzite-core` — core types + traits + signal + value ✅
2. `quartzite-macros` — Extend + Object + object_impl derive macros ✅
3. `quartzite-runtime` — Application, EventLoop, ObjectTree ✅
4. `quartzite` (facade) — prelude re-exports, Cargo metadata, docs.rs config ✅
5. `quartzite-geometry` + `quartzite-events` — geometry primitives + event model
6. `quartzite-widgets` — WidgetBase + concrete widgets + layouts
7. `quartzite-paint` + `quartzite-style` — painter + theming

Maintenance plans (cross-cutting, all ✅): auto-connection (signal/slot extension), code-quality-cleanup, docs-and-facade, public-api-docs, lookup-perf (O(1) signal disconnect, name index, match-based meta lookup), inline-simple-fns (`#[inline]` on simple non-generic fns).

## Open Questions

- Async/await integration strategy
- Accessibility (a11y) support
- No-std support scope (core only, or further?)
