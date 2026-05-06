# Project Context: quartzite

> Read on demand. Keep this file up to date as the project evolves.

## Purpose

`quartzite` is a **GUI/object framework for Rust**, usable from Rust and (eventually) Python. It implements an object tree, property system, signals/slots, meta-object reflection, widget hierarchy, and layout engine — all in idiomatic Rust with no binary or ABI compatibility requirement with any other framework.

## Crate Layout

| Crate | Purpose |
|---|---|
| `quartzite-core` | ObjectBase, AsObject, Object, ObjectExt, Value, Signal, MetaObject — no_std compatible |
| `quartzite-macros` | Proc-macro crate: `#[derive(Extend)]`, `#[derive(Object)]`, `#[object_part]`, `#[object_impl]` |
| `quartzite-runtime` | Application, EventLoop, ObjectTree, ObjectRef, Timer, ObjectTreeExt, try_with_tree |
| `quartzite-geometry` | Point/PointF, Size/SizeF, Rect/RectF, Margins — no_std, no alloc |
| `quartzite-events` | Event\<T\>, MouseEvent, KeyEvent, EventFilter\<T\>, ResizeEvent, CloseEvent, TimerEvent — no_std + alloc |
| `quartzite-paint` | Painter, Color, Font, Pen, Brush, Image, Path |
| `quartzite-style` | Style trait, Palette, StyleRegistry |
| `quartzite-widgets` | WidgetBase, WidgetExt, Layout, Button, Label, LineEdit, … |
| `quartzite` (workspace root + facade) | Re-exports sub-crates as `quartzite::core`, `quartzite::macros` (optional; `derive` feature, on by default), `quartzite::runtime`; curated `prelude` module; `examples/` at workspace root ✅ |

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
- Single-inheritance model — replaced by traits + composition
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
| `ObjectTreeExt` | no | Parent/child accessors (`parent`, `children`, `parent_in`, `children_in`) — blanket impl in `quartzite-runtime` |
| `As{X}` | yes | Generated per hierarchy level by `#[derive(Extend)] #[root]` |
| `{X}Ext` | no | Convenience methods per level (blanket impl) |

### Macro System

| Macro | Input | Output |
|---|---|---|
| `#[derive(Extend)]` | struct with `#[root]`/`#[base]`/`#[mixin]` | `As{TypeName}` trait, delegation impls |
| `#[derive(Object)]` | struct with `#[prop]`, `#[signal]` fields | property + signal metadata arrays, partial `impl Object` |
| `#[object_part]` | impl block with `#[slot]`/`#[invokable]` methods (inherent or trait impl); for multi-block accumulation | accumulates methods into thread-local; emits only the cleaned impl block |
| `#[object_impl]` | impl block with `#[slot]`/`#[invokable]` methods; no flags | auto-detects mode: empty accumulator → sole (full output); non-empty → terminal (drain + merge + full output) |

### Ownership Model

**Arena/SlotMap** (`ObjectId` handles, `slotmap::SlotMap<DefaultKey, Box<dyn Object>>` central store). Objects are owned by `ObjectTree`; callers hold `ObjectId` (logical u64 identity) or `ObjectRef<T>` / `WeakRef<T>` typed wrappers. `ObjectTree` is wrapped in `Mutex<ObjectTree>` in `Application` — `Object: Send` supertrait ensures soundness without unsafe.

### Signal/Slot Lifetime Safety

`ReceiverGuard` pattern: every `ObjectBase` holds `Arc<ReceiverGuard>`. Incoming connections hold `Weak<ReceiverGuard>`. When the object is dropped, all weak refs break and queued slot calls are silently discarded.

### Value System

`Value` enum: `Null | Bool | Int(i64) | Float(f64) | String | List | Map | Bytes | Duration(core::time::Duration) | Object(WeakRef) | Custom(Arc<dyn CustomValue>)`. `Duration` is a first-class variant (no_std-safe; `core::time::Duration` is available without `std`). `usize` implements `FromValue`/`IntoValue` via the existing `impl_int_checked!` macro.

## Key Design Decisions

| Question | Decision |
|---|---|
| ABI compatibility | Out of scope |
| Code generation | Proc macros only, no external tool |
| Inheritance model | Traits + composition; `#[base]` + blanket impls |
| Hierarchy root marker | `#[root]` explicit attribute |
| Multiple bases | One `#[base]` + N `#[mixin]`; two `#[base]` → compile error |
| `AsObject` vs `Object` | Separate: `AsObject` = pure accessor; `Object` = meta-system |
| Crate naming | `quartzite-*`; workspace root doubles as the `quartzite` facade crate (no `quartzite/` subdirectory) |
| Python interop | Deferred; reflection layer designed to enable it later |
| Macro codegen paths | `crate_root()` in `quartzite-macros` uses `proc_macro_crate` to resolve the actual crate path at expansion time (facade-first: `quartzite → quartzite-core → ::quartzite_core` fallback). Handles crate renaming transparently. |
| `derive` feature | `quartzite-macros` is an optional dep gated on the `derive` feature (on by default); disable to skip proc-macro compilation in macro-free or `no_std` builds |
| Object ownership | Arena/SlotMap — `ObjectTree` + `ObjectId` + `Mutex<ObjectTree>` in Application |
| `ConnectionType::Auto` | Same-thread → Direct (sync call, args cloned); cross-thread → Queued (post to dispatcher). `ThreadId` and `Weak<ReceiverGuard>` captured at connect time. Guard checked on both paths before dispatch — slot silently skipped when receiver destroyed. Requires `Args: Clone + Send + 'static`. Gated on `feature = "std"`. |
| Signal slot storage | `IndexMap<ConnectionId, …>` — insertion-order preserved; O(1) disconnect via `shift_remove`. `std` feature propagates to `indexmap/std`; `no_std` uses `hashbrown::DefaultHashBuilder` type alias. |
| `ObjectBase: Debug` | `#[derive(Debug)]` on both `ReceiverGuard` and `ObjectBase`. Auto-derive; Rust's derive respects the `#[cfg(feature = "std")]` gate on `thread_id`. May be replaced by a manual impl later. |
| `ObjectBase::name` | `Option<String>`; `None` = unnamed (not indexed); `Some("")` = explicitly named `""` (indexed). Mutation only via `ObjectTree::rename` / `ObjectTree::clear_name`. |
| `ObjectTree::rename` no-op | When new name equals current name (`Some(x) == Some(x)`), returns immediately with no mutation and no event. Anonymous (`None`) → `""` is a real rename, not a no-op. |
| `ObjectFactory` singleton | `static FACTORY: OnceLock<Arc<RwLock<ObjectFactory>>>`. Access via `ObjectFactory::install(factory)` (first call wins; returns `Err(FactoryAlreadySet)` on subsequent calls) and `ObjectFactory::global() -> Option<Arc<RwLock<ObjectFactory>>>`. `Application::new` calls `install` and ignores `FactoryAlreadySet`; does not own the factory. |
| `ObjectTree::find_by_name` | Returns `&[ObjectId]` — all objects with that name, insertion order. Backed by `HashMap<String, Vec<ObjectId>>` index. |
| Process-global tree accessor | `static TREE_PTR: AtomicPtr<Mutex<ObjectTree>>` in `quartzite-runtime::global_tree`. `Application::new` calls `register(&inner.object_tree)`; `Drop for Application` calls `deregister()`. `try_with_tree<R>(f)` returns `Option<R>` — `None` when no Application is live. `ObjectTreeExt::parent/children` use this; calling from inside a `try_with_tree` closure would deadlock (same Mutex). v1 is single-threaded; the AtomicPtr is safe for future multi-thread use. |
| `ObjectBase::new_with_id` | Creates an `ObjectBase` with a caller-supplied `ObjectId`. Caller must ensure uniqueness; duplicate IDs cause incorrect tree lookups. Used for deserialization / test stubs. |
| MetaObject / EnumMeta lookup | fn-pointer fields — O(1) match dispatch. Property/signal lookup fns generated in hidden mod by `#[derive(Object)]`; method/enum lookup fns generated by `#[object_impl]`; EnumMeta entry fns generated by `#[meta_enum]`. |
| `ObjectBase::signals_blocked` | Private `bool` field; toggled via `block_signals()` / `unblock_signals()`. Use `emit!(self.signal_field, &args)` macro to suppress emission automatically; `Signal::emit` is unconditional. |
| `emit!` macro | `quartzite_core::emit!(receiver.field, &args)` — binds `__blocked` via `let` (releasing the immutable borrow), then calls `receiver.field.emit(&args)` inside `if !__blocked`. Available via `use quartzite::prelude::*`. Cannot be used with `Arc<Mutex<Signal>>` (e.g. `Timer::tick`); use explicit `if !blocked` guard there. |
| `emit_<signal>` codegen | `#[derive(Object)]` generates `pub fn emit_<signal>(&mut self, arg0: T0, ...)` methods (flattened tuple args) on the struct. Body delegates to `emit!(self.#field, &(args,))` — the macro handles the `signals_blocked` guard. |
| `connect_<signal>_auto` codegen | `#[derive(Object)]` generates `pub fn connect_<signal>_auto(&mut self, receiver: &ObjectBase, f: F)` methods (gated `#[cfg(feature = "std")]`, `#[inline]`) on the struct. Extracts `receiver.thread_id` and `Arc::downgrade(receiver.receiver_guard())` internally; delegates to `Signal::connect_auto`. |
| Multi-block `#[object_impl]` | `#[object_part]` accumulates into `thread_local!` HashMap keyed by `(CARGO_PKG_NAME, type_name)` as span-free `StoredMethod` strings (spans are only valid within one macro invocation); `#[object_impl]` auto-detects terminal mode via `accumulator::peek` and drains + merges on non-empty. No explicit flags needed. |
| Generic `#[derive(Extend)]` | Non-root generic structs supported via `split_for_impl()` with minimal bounds (no bounds propagated from struct definition). Root + generic rejected at parse time (trait return type would be ill-formed). |
| `connect_<signal>_queued` codegen | `#[derive(Object)]` generates `pub fn connect_<signal>_queued(&mut self, receiver: &ObjectBase, f: F)` methods (gated `#[cfg(feature = "std")]`, `#[inline]`) on the struct. Extracts `Arc::downgrade(receiver.receiver_guard())` internally; delegates to `Signal::connect_queued(f, guard)` (`f` first — opposite order from `connect_auto`). No `thread_id` argument needed. |
| `quartzite-geometry` no_std | Pure `no_std` with no alloc — all types are `Copy` stack values. `f32::round()` unavailable in no_std; `libm::roundf` is used instead (always-on dep, no opt-out). `PointF → Point` rounds half-away-from-zero. |
| `ObjectId`/`ConnectionId` ordering | `#[derive(PartialOrd, Ord)]` on both types (wrap `u64`; allocation order). Enables `BinaryHeap<Reverse<(Instant, ObjectId)>>` in `PoolDriver` without a separate `Ord` wrapper. Tests with `*Id` sort keys use native `<`/`sorted_unstable()` directly. |
| `Timer` pluggable driver | `TimerDriver` trait with `start(TimerConfig, callback)` + `stop(ObjectId)`. Three built-in drivers: `ThreadDriver` (one thread per timer, `park_timeout` + `unpark`), `AppDriver` (posts `Box<dyn FnOnce>` via `Application::global()`), `PoolDriver` (single background thread + `BinaryHeap<Reverse<(Instant, ObjectId)>>` + single `Mutex<PoolState>` + `Condvar`). |
| `Timer::tick` signal isolation | `tick: Arc<Mutex<Signal<(usize,)>>>` is NOT declared with `#[signal]`. The same `Arc` is shared between `Timer` and `TimerState::signal` so driver-thread emissions reach the same `Signal` instance as user-facing `connect_tick*` slots. Using `#[signal]` would create a second `Signal` instance; slots would never fire. |
| `signals_blocked` two-copy sync | Driver callback reads `TimerState::signals_blocked: AtomicBool` (no `&mut Timer` access). `Timer::block_signals()` / `unblock_signals()` update both `base.signals_blocked` and `TimerState::signals_blocked`. Direct `base.block_signals()` bypasses driver sync — documented limitation; callers must use `Timer::block_signals()`. |
| `Timer` implements `Object` | `Timer` manually implements `AsObject` + `Object` (quartzite-runtime has no quartzite-macros dep). `read_property`/`write_property` dispatch `"interval"` → `Value::Duration` and `"single_shot"` → `Value::Bool`. Static `TIMER_META: MetaObject` with two `PropertyMeta` entries. |
| `quartzite-events` no_std | `no_std + alloc` — needs `String` for `KeyEvent::text`. `MouseButton` and `KeyModifiers` use `bitflags!` (u8). |
| `EventType<T>` shape | Nested enums `Key(KeyEventKind)`, `Mouse(MouseEventKind)` — discriminate kind without downcasting. Generic `T: 'static + Send + Sync = ()` for `User(T)` payload (winit style; app commits to one type; zero allocation, zero downcast). `Event<T>` is object-safe for fixed `T`. |
| `PropertyFlags` representation | `pub type PropertyFlags = BitFlags<PropertyFlag>` via `enumflags2`. `PropertyFlag` is a `#[bitflags(default = Readable \| Writable \| Stored \| Designable)] #[repr(u8)]` enum. Named constructors (`none`, `read_write`, `read_only`) are `const fn` on `impl PropertyFlag`. Proc-macro codegen uses `make_bitflags!(PropertyFlag::{…})` via a `use ::quartzite::core::PropertyFlag;` import inside the generated hidden module; `enumflags2` is `#[doc(hidden)]` re-exported from `quartzite-core` for that path. |

## Plans (Implementation Order)

Crate-level plans:

1. `quartzite-core` — core types + traits + signal + value ✅
2. `quartzite-macros` — Extend + Object + object_impl derive macros ✅
3. `quartzite-runtime` — Application, EventLoop, ObjectTree ✅
4. `quartzite` (facade) — prelude re-exports, sub-crate re-exports, Cargo metadata, docs.rs config ✅
5. `examples/` — runnable API examples at workspace root (hello_object, signals_slots, object_tree, timer) ✅
6. `quartzite-geometry` + `quartzite-events` — geometry primitives + event model ✅
7. `quartzite-widgets` — WidgetBase + concrete widgets + layouts
8. `quartzite-paint` + `quartzite-style` — painter + theming

Maintenance plans (cross-cutting, all ✅): auto-connection (signal/slot extension), code-quality-cleanup, docs-and-facade, public-api-docs, lookup-perf (O(1) signal disconnect, name index, match-based meta lookup), inline-simple-fns (`#[inline]` on simple non-generic fns), examples-crate (runnable API examples), signals-blocked (typed emit wrappers + `signals_blocked` guard), receiver-guard-auto (`Weak<ReceiverGuard>` for Auto connections + `connect_<signal>_auto` codegen), connect-queued-codegen (`connect_<signal>_queued` typed wrappers), enumflags2-property-flags (`PropertyFlags` replaced by `BitFlags<PropertyFlag>` backed by `u8` via `enumflags2`), geometry-events (quartzite-geometry + quartzite-events crates), objectbase-debug-rename-factory (`ObjectBase: Debug`, `rename` no-op, `ObjectFactory` singleton), doc-convention (workspace-wide doc-comment convention from RFC 1574 + deterministic.space; `# Parameters` on every `pub fn` with ≥1 arg; strict section order; clippy `missing_errors_doc`/`missing_panics_doc`/`missing_safety_doc`/`doc_markdown` enabled across all crates; canonical reference at `ai-docs/doc-convention.md`), thiserror-migration (`ApplicationError`, `FactoryAlreadySet`, `DispatcherAlreadySet`, `TypeError` migrated to `#[derive(thiserror::Error)]`; `thiserror = "2"` added to `quartzite-core`; `TypeError` gains `core::error::Error` impl), tracing-itertools (`tracing = "0.1"` added to `quartzite-core` (no_std-gated) + `quartzite-runtime`; `tracing/log` feature enabled so `log`-compatible subscribers receive diagnostics; `itertools = "0.14"` added as dev-dep; `env_logger` in examples), signal-emit-rename (`Signal::emit_unless_blocked` removed; `Signal::emit` is now unconditional — no `blocked` arg; `emit!` macro owns the guard; generated wrappers use `emit!`; `timer.rs` uses explicit `if !blocked` guards for its `Arc<Mutex<Signal>>` path).

## Open Questions

- Async/await integration strategy — `futures-util` was evaluated and **deferred**: the runtime uses raw `std::thread` + `mpsc` channels with no async call sites. `futures-util` should be added only once an async executor strategy (tokio, async-std, or custom) is decided. Tracked in #89.
- Accessibility (a11y) support
- No-std support scope (core only, or further? — geometry + events are also no_std + alloc)
