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
| `quartzite-geometry` | Point/PointF, Size/SizeF, Rect/RectF, Margins, Alignment — no_std (now with `quartzite-core` + `quartzite-macros` deps for `MetaEnum` derive on `Alignment`, default-features = false to preserve `no_std`) |
| `quartzite-event-types` | Event\<T\>, EventType\<T\>, EventFilter\<T\>, KeyEventKind, MouseEventKind, TimerEvent — no_std + alloc; intermediate crate between `quartzite-core` and `quartzite-events` |
| `quartzite-events` | MouseEvent, KeyEvent, ResizeEvent, CloseEvent; re-exports from `quartzite-event-types` — no_std + alloc |
| `quartzite-paint-api` | Thin `no_std` shared paint vocabulary: `Color` (with `with_alpha`), `Pen`, `Brush`/`BrushKind`, `Font`/`FontWeight`, `Image`/`ImageError`, `Path`/`Segment`, 11-method `Painter` trait, `PaintError` — no platform deps; used by `quartzite-paint`, `quartzite-renderer`, `quartzite-widgets`, and `quartzite-style-types` |
| `quartzite-paint` | Re-export shell: `quartzite-paint-api` types plus `Alignment` from `quartzite-geometry` — backend-agnostic, no renderer dep |
| `quartzite-renderer` | Windowed rendering backend — `vello` + `wgpu` + `winit`; `WindowedApplication` (owns `Application` + `EventLoop<AppEvent>`; multi-window via `WindowRegistry`; configurable last-window-quit policy; `event_proxy() -> EventLoopProxy<AppEvent>` for cross-thread `AppEvent::Exit`); `WindowedAppHandler` trait (`on_start`, `on_last_window_closed`); `WindowRegistry` (`IndexMap`-backed fan-out, `try_create_window`, `windows()`); `WindowedApplicationBuilder` (Linux `with_any_thread` for test threads); `VelloPainter` (full `Painter` impl — all 11 methods live; text via `parley` + `skrifa`; transform/clip stack with `save`/`restore`; `draw_image` uses `vello::peniko::Image`; `draw_path` converts `quartzite_paint_api::Segment` to `kurbo`); `FontCache` (`parley::FontContext` + `LayoutContext<[u8; 4]>` — owned per pipeline entry point, borrowed mutably per frame for text shaping; persists system-font discovery across frames); `RenderHarness` (offscreen test-only headless harness producing `image::RgbaImage` for snapshot tests; bypasses winit); `RenderHarnessBuilder` (builder pattern replacing `RenderHarness::new`; `scale_factor(f32)` for HiDPI testing); `RendererError`. **wgpu pinned to 28** to match vello 0.8 (vello transitively requires wgpu 28 — the harness passes wgpu types into `vello::Renderer::render_to_texture` so the versions must unify). New deps: `parley` (text layout) + `skrifa` (font metrics). |
| `quartzite-style-types` | Leaf crate (no_std + alloc): `Palette` + `ColorRole` (+ `ColorRole::ALL`). Depends only on `quartzite-paint-api` for `Color`. Exists to break the otherwise-impossible `style ↔ widgets` Cargo cycle (widgets depends on this leaf, never on `quartzite-style`). |
| `quartzite-style` | Downstream `std` crate: `Style` trait (`Send + Sync`, generic-only `draw_widget(&dyn AsWidget, &mut dyn Painter, &Palette)`), `StyleRegistry` (`OnceLock<Mutex<Option<&'static dyn Style>>>` storage with `Box::leak`, non-panicking `try_style() -> Option<&'static dyn Style>`, lock-poisoning recovery via `unwrap_or_else(\|e\| e.into_inner())`). Re-exports `Palette` and `ColorRole` from `quartzite-style-types`. |
| `quartzite-widgets` | WidgetBase, WidgetExt, Layout (`BoxLayout`, `GridLayout`), `Label`, `Button`, `LineEdit`, `TextEdit`, `ScrollArea`, `Container` — widget system ✅ (#46) |
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
| `ObjectTree::find_by_name_in` | Returns `Vec<ObjectId>` of descendants-or-self of `root` whose name matches, in BFS (shallowest-first) order. Returns empty if `root` is not in the tree or no match is found. |
| `ObjectBase::name_changed` | Built-in `Signal<(Option<String>, Option<String>)>` field on every object. Payload: `(old_name, new_name)` where `None` = anonymous. Emitted by `ObjectTree::rename` / `ObjectTree::clear_name` after the index is updated. Not emitted by `destroy`. `Value::Null` encodes `None`; `Value::String(s)` encodes `Some(s)`. |
| `name_changed` codegen | `#[derive(Object)]` synthesises a phantom `SignalField { ident: "name_changed", builtin: true }` prepended to every object's signal slice. Routes dispatch through `this.object_base_mut().name_changed` in `__emit_signal_` and `__connect_signal_dynamic_`. No typed `emit_name_changed` / `connect_name_changed_*` wrappers generated — use `emit_signal("name_changed", &[old_val, new_val])` or `connect_signal("name_changed", cb)`. |
| `Option<String>` Value encoding | `IntoValue`/`FromValue` impls: `None` ↔ `Value::Null`, `Some(s)` ↔ `Value::String(s)`. Used by `name_changed` signal args. |
| Process-global tree accessor | `static TREE_PTR: AtomicPtr<Mutex<ObjectTree>>` in `quartzite-runtime::global_tree`. `Application::new` calls `register(&inner.object_tree)`; `Drop for Application` calls `deregister()`. `try_with_tree<R>(f)` returns `Option<R>` — `None` when no Application is live. `ObjectTreeExt::parent/children` use this; calling from inside a `try_with_tree` closure would deadlock (same Mutex). v1 is single-threaded; the AtomicPtr is safe for future multi-thread use. |
| `ObjectBase::new_with_id` | Creates an `ObjectBase` with a caller-supplied `ObjectId`. Caller must ensure uniqueness; duplicate IDs cause incorrect tree lookups. Used for deserialization / test stubs. |
| Snapshot layer (`serde` feature) | Three-level serialization: property layer (`Value` serde impls, hand-written because `Arc<dyn CustomValue>` can't auto-derive; `typetag` for `CustomValue` registry), object layer (`ObjectSnapshot { class_name, properties: BTreeMap<String, Value> }` — only `PropertyFlag::Stored` properties captured), tree layer (`TreeSnapshot { schema_version: u32, root: ObjectNode }` — full parent/child structure). `CURRENT_SCHEMA_VERSION = 1`; `TreeSnapshot::validate_version()` returns `DeserializeError::UnsupportedVersion` when payload version > supported. Restore uses `ObjectFactory::global()` to construct fresh instances. Signal connections and `signals_blocked` are NOT serialized; both reset to defaults after restore. `Value::Object(WeakObjectRef)` round-trips by numeric id (no cross-snapshot remap in v1). Public API: `capture_object`, `restore_object`, `capture_tree`, `restore_tree` in `quartzite_runtime::snapshot` (gated behind `serde` feature); re-exported in `quartzite::snapshot`. |
| MetaObject / EnumMeta lookup | fn-pointer fields — O(1) match dispatch. Property/signal lookup fns generated in hidden mod by `#[derive(Object)]`; method/enum lookup fns generated by `#[object_impl]`; EnumMeta entry fns generated by `#[meta_enum]`. |
| `ObjectBase::signals_blocked` | Private `bool` field; toggled via `block_signals()` / `unblock_signals()`. Use `emit!(self.signal_field, &args)` macro to suppress emission automatically; `Signal::emit` is unconditional. |
| `emit!` macro | `quartzite_core::emit!(receiver.field, &args)` — binds `__blocked` via `let` (releasing the immutable borrow), then calls `receiver.field.emit(&args)` inside `if !__blocked`. Available via `use quartzite::prelude::*`. Cannot be used with `Arc<Mutex<Signal>>` (e.g. `Timer::tick`); use explicit `if !blocked` guard there. |
| `emit_<signal>` codegen | `#[derive(Object)]` generates `pub fn emit_<signal>(&mut self, arg0: T0, ...)` methods (flattened tuple args) on the struct. Body delegates to `emit!(self.#field, &(args,))` — the macro handles the `signals_blocked` guard. |
| `connect_<signal>_auto` codegen | `#[derive(Object)]` generates `pub fn connect_<signal>_auto(&mut self, receiver: &ObjectBase, f: F)` methods (gated `#[cfg(feature = "std")]`, `#[inline]`) on the struct. Extracts `receiver.thread_id` and `Arc::downgrade(receiver.receiver_guard())` internally; delegates to `Signal::connect_auto`. |
| Multi-block `#[object_impl]` | `#[object_part]` accumulates into `thread_local!` HashMap keyed by `(CARGO_PKG_NAME, type_name)` as span-free `StoredMethod` strings (spans are only valid within one macro invocation); `#[object_impl]` auto-detects terminal mode via `accumulator::peek` and drains + merges on non-empty. No explicit flags needed. |
| Generic `#[derive(Extend)]` | Non-root generic structs supported via `split_for_impl()` with minimal bounds (no bounds propagated from struct definition). Root + generic rejected at parse time (trait return type would be ill-formed). |
| `connect_<signal>_queued` codegen | `#[derive(Object)]` generates `pub fn connect_<signal>_queued(&mut self, receiver: &ObjectBase, f: F)` methods (gated `#[cfg(feature = "std")]`, `#[inline]`) on the struct. Extracts `receiver.thread_id` and `Arc::downgrade(receiver.receiver_guard())` internally; delegates to `Signal::connect_queued(receiver_thread_id, f, guard)` (`thread_id` first, then `f`). |
| Per-thread `LoopRegistry` | Process-wide singleton `LazyLock<RwLock<HashMap<ThreadId, Arc<EventLoop>>>>` in `quartzite-runtime`. `EventLoop::install_for_current_thread(self: Arc<Self>) -> Result<(), LoopAlreadyInstalled>` and `EventLoop::uninstall_for_current_thread()` (static). RAII `RegistryGuard` inserted by `EventLoop::run()` for panic-safe cleanup. `EventLoop::spawn(f) -> (Arc<EventLoop>, JoinHandle<()>)` easy-install convenience. `Application::new()` auto-installs its loop; `Application::main_thread_id()` returns main-thread `ThreadId`. |
| `QueuedDispatcher::post` signature | `fn post(&self, target: ThreadId, f: Box<dyn FnOnce() + Send + 'static>)` — explicit thread routing. `ConnectionTable` looks up `LoopRegistry::get(target)` and posts to the found loop; emits `tracing::warn!` + drops `f` if no loop is registered (documented on trait). |
| `quartzite-geometry` no_std | Pure `no_std` with no alloc — all types are `Copy` stack values. `f32::round()` unavailable in no_std; `libm::roundf` is used instead (always-on dep, no opt-out). `PointF → Point` rounds half-away-from-zero. |
| `ObjectId`/`ConnectionId` ordering | `#[derive(PartialOrd, Ord)]` on both types (wrap `u64`; allocation order). Enables `BinaryHeap<Reverse<(Instant, ObjectId)>>` in `PoolDriver` without a separate `Ord` wrapper. Tests with `*Id` sort keys use native `<`/`sorted_unstable()` directly. |
| `Timer` pluggable driver | `TimerDriver` trait with `start(TimerConfig, callback)` + `stop(ObjectId)`. Three built-in drivers: `ThreadDriver` (one thread per timer, `park_timeout` + `unpark`), `AppDriver` (posts `Box<dyn FnOnce>` via `Application::global()`), `PoolDriver` (single background thread + `BinaryHeap<Reverse<(Instant, ObjectId)>>` + single `Mutex<PoolState>` + `Condvar`). |
| `Timer::tick` signal isolation | `tick: Arc<Mutex<Signal<(usize,)>>>` is NOT declared with `#[signal]`. The same `Arc` is shared between `Timer` and `TimerState::signal` so driver-thread emissions reach the same `Signal` instance as user-facing `connect_tick*` slots. Using `#[signal]` would create a second `Signal` instance; slots would never fire. |
| `signals_blocked` two-copy sync | Driver callback reads `TimerState::signals_blocked: AtomicBool` (no `&mut Timer` access). `Timer::block_signals()` / `unblock_signals()` update both `base.signals_blocked` and `TimerState::signals_blocked`. Direct `base.block_signals()` bypasses driver sync — documented limitation; callers must use `Timer::block_signals()`. |
| Signal-to-signal connections | Two APIs in `quartzite-core::connect` (std-only, in `quartzite::prelude`): `connect_signal_to_signal(&mut dyn Object, …, Arc<Mutex<dyn Object>>, …)` (dynamic, any `Object`) and `connect_signals<From, To, Args>(…, fn(&mut From) → &mut Signal<Args>, …)` (typed, avoids `&[Value]` round-trip for Auto/Queued). Validation at connection time via `SignalMeta::params` arity + `type_name` string comparison. `Object::emit_signal(&mut self, name, &[Value])` method added to the `Object` trait — proc-macro generates a hidden helper `__emit_signal_T` that dispatches by name and converts `Value` back to typed args via `FromValue`. Liveness via `Weak<Mutex<dyn Object>>` inside the forwarding closure. |
| `Timer` implements `Object` | `Timer` manually implements `AsObject` + `Object` (quartzite-runtime has no quartzite-macros dep). `read_property`/`write_property` dispatch `"interval"` → `Value::Duration` and `"single_shot"` → `Value::Bool`. Static `TIMER_META: MetaObject` with two `PropertyMeta` entries. |
| `quartzite-events` no_std | `no_std + alloc` — needs `String` for `KeyEvent::text`. `MouseButton` and `KeyModifiers` use `bitflags!` (u8). |
| `EventType<T>` shape | Nested enums `Key(KeyEventKind)`, `Mouse(MouseEventKind)` — discriminate kind without downcasting. Generic `T: 'static + Send + Sync = ()` for `User(T)` payload (winit style; app commits to one type; zero allocation, zero downcast). `Event<T>` is object-safe for fixed `T`. |
| `PropertyFlags` representation | `pub type PropertyFlags = BitFlags<PropertyFlag>` via `enumflags2`. `PropertyFlag` is a `#[bitflags(default = Readable \| Writable \| Stored \| Designable)] #[repr(u8)]` enum. Named constructors (`none`, `read_write`, `read_only`) are `const fn` on `impl PropertyFlag`. Proc-macro codegen uses `make_bitflags!(PropertyFlag::{…})` via a `use ::quartzite::core::PropertyFlag;` import inside the generated hidden module; `enumflags2` is `#[doc(hidden)]` re-exported from `quartzite-core` for that path. |
| `style ↔ widgets` Cargo cycle resolution | Two-crate split: `quartzite-style-types` is the leaf (`Palette`, `ColorRole`); `quartzite-style` is downstream (`Style` trait, `StyleRegistry`). `quartzite-widgets` re-exports `Palette`/`ColorRole` from the leaf only — never depends on `quartzite-style`. Enforced mechanically by `quartzite-widgets/tests/no_style_dep.rs`, which shells out to `cargo tree -p quartzite-widgets --prefix none --no-dedupe` and asserts no line starts with `"quartzite-style "` (trailing space anchors against `quartzite-style-types`). |
| `StyleRegistry` storage | `static REGISTRY: OnceLock<Mutex<Option<&'static dyn Style>>>` in `quartzite-style/src/registry.rs`. `set_style(Box<dyn Style>)` calls `Box::leak` to obtain a `'static` reference; replacement leaks the prior box (acceptable for a process-lifetime registry — documented). Default accessor is non-panicking `try_style() -> Option<&'static dyn Style>` per AGENTS.md § *API Naming*. Lock-poisoning recovered via `lock().unwrap_or_else(\|e\| e.into_inner())`. `#[cfg(test)] pub(crate) fn clear_for_test()` and `poison_for_test()` helpers expose the slot for `serial_test` ordering. |
| `Style` trait surface | Generic-only — single method `fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette)`. Trait bounds `Send + Sync` (required by global registry). No per-widget primitive methods (no `draw_button`/`draw_label`/etc.) on the trait — concrete `Style` impls dispatch on widget type internally via downcast or visitor. |
| `Painter` trait shape (paint-api) | Pass-through (NOT stateful): pen/brush passed as call args rather than stored on the painter. 11 methods total: `draw_rect`/`fill_rect`/`draw_line`/`draw_text`/`draw_text_in`/`draw_image`/`draw_path` (drawing), `clip_rect`/`translate` (transform), `save`/`restore` (state stack — implementation delegated to concrete backend, no internal stack in the trait). Object-safe: no generics, no `Self` returns, no associated types. |
| `Color` representation | `f32` channels in `[0.0, 1.0]` (NOT u8). `Color::new(r,g,b,a)`, named constants (`BLACK`/`WHITE`/`RED`/`GREEN`/`BLUE`/`TRANSPARENT`), `with_alpha(a: f32) -> Color` const fn for builder-style alpha replacement. |
| `Path` representation | `Vec<Segment>` builder pattern in `quartzite-paint-api`; methods `move_to(p)`/`line_to(p)`/`cubic_to(c1, c2, p)`/`arc_to{centre, radii, start_angle, sweep_angle}`/`close()` return `&mut Self`. `Segment` is `#[non_exhaustive]` enum (`MoveTo`/`LineTo`/`CubicTo`/`ArcTo`/`Close`). `arc_to` semantics: centre-and-radii, radians, positive sweep CCW. |
| `Image` representation | `quartzite-paint-api::Image { width: u32, height: u32, pixels: Vec<u8> }` — RGBA8 row-major, no per-pixel padding. Constructed via `Image::try_new(w, h, pixels) -> Result<Image, ImageError>` with `checked_mul` overflow guard. `ImageError` is `#[non_exhaustive]` thiserror with `PixelLengthMismatch { expected, actual }` and `Overflow` variants (latter for usize overflow on 32-bit targets). |
| `Alignment` location | `quartzite-geometry::Alignment` (moved from `quartzite-widgets` in paint-style #47). `quartzite-widgets::Alignment` is now a `pub use` re-export — `TypeId::of::<widgets::Alignment>() == TypeId::of::<geometry::Alignment>()`. `MetaEnum` derive preserved; `quartzite-geometry` gained `quartzite-core` (`default-features = false`) and `quartzite-macros` deps to keep the derive expanding. Crate stays `no_std`. |
| Multi-window `ActiveEventLoop` slot | `Cell<*const ActiveEventLoop>` in `WindowRegistry`; `PhantomData<*const ()>` makes `!Send + !Sync` explicit. `WrappedHandler::arm_active_loop` sets the pointer before handing `&mut WindowRegistry` to user code; `ActiveLoopGuard(raw_ptr)` clears it on drop. Raw pointer (not reference) avoids borrow-checker conflict: `&mut self.registry` can coexist with the guard on the stack. Non-null deref only after null check inside `try_create_window`. |
| `WindowEntry` drop order | `surface: wgpu::Surface<'static>` declared **before** `window: Arc<Window>` in `WindowEntry` — Rust's struct-field drop order (first-to-last) drops the surface before the window; reverse order is use-after-free because the surface holds the window's raw handle. `Arc<Window>` as the handle source gives `wgpu::Surface<'static>`. |
| `quit_on_last_window_closed` policy | Configurable via `WindowedApplicationBuilder::quit_on_last_window_closed(bool)` (default `true`). When `true`, `WrappedHandler::dispatch_window_event` calls `event_loop.exit()` on `CloseRequested` when registry becomes empty. When `false`, the user's `on_last_window_closed` hook fires but the loop continues; exit requires an explicit `AppEvent::Exit` via `event_proxy()`. |
| `AppEvent` / proxy exit | `pub enum AppEvent { Exit }` drives `EventLoop<AppEvent>`. `WindowedApplication::event_proxy() -> EventLoopProxy<AppEvent>` exposes the proxy for cross-thread exit requests. `user_event` in `WrappedHandler` calls `event_loop.exit()` on `AppEvent::Exit`. The proxy-based exit is the only cross-thread exit path; `winit` provides no synthesise-`WindowEvent` API. |

## Plans (Implementation Order)

Crate-level plans:

1. `quartzite-core` — core types + traits + signal + value ✅
2. `quartzite-macros` — Extend + Object + object_impl derive macros ✅
3. `quartzite-runtime` — Application, EventLoop, ObjectTree ✅
4. `quartzite` (facade) — prelude re-exports, sub-crate re-exports, Cargo metadata, docs.rs config ✅
5. `examples/` — runnable API examples at workspace root (hello_object, signals_slots, object_tree, timer) ✅
6. `quartzite-geometry` + `quartzite-events` — geometry primitives + event model ✅
7. `quartzite-paint-api` — thin no_std shared paint types + `Painter` trait; wired into `quartzite` facade ✅ (graphics-stack #73; extended in paint-style #47 with `Color::with_alpha`, `Font`/`FontWeight`, `Image`/`ImageError`, `Path`/`Segment`, and four new `Painter` methods `draw_text`/`draw_text_in`/`draw_image`/`draw_path`)
8. `quartzite-paint` — re-export shell over `quartzite-paint-api` + `Alignment` from `quartzite-geometry` ✅ (graphics-stack #73; full vocabulary completed in paint-style #47)
9. `quartzite-renderer` — `WindowedApplication` + `VelloPainter` (vello+wgpu+winit) ✅ (graphics-stack #73; new `Painter` methods land as no-op stubs in paint-style #47; real implementations + text rendering via parley/skrifa in renderer-painter-impls #289/#277)
10. `quartzite-widgets` — WidgetBase, WidgetExt, BoxLayout, GridLayout, Label, Button, LineEdit, TextEdit, ScrollArea, Container ✅ (#46; refactored in paint-style #47 to drop local `Alignment`/`Font`/`Palette` and re-export from upstream)
11. `quartzite-style-types` + `quartzite-style` — leaf (`Palette`, `ColorRole`) + downstream (`Style` trait, `StyleRegistry`) ✅ (paint-style #47). Two-crate split exists to break the `style ↔ widgets` Cargo cycle; widgets depends on the leaf only, enforced by a `cargo tree -p quartzite-widgets` integration test.

### Maintenance plans (cross-cutting)

Cross-cutting plans that touched multiple crates and aren't part of the dependency tree above. All ✅ implemented; full detail in linked spec.

- **project-docs** — README description block, comprehensive `quartzite` facade `lib.rs` rustdoc, `CONTRIBUTING.md`, auto-generated `ROADMAP.md` from `ai-docs/plans/INDEX.md` via `scripts/gen-roadmap.sh`, `roadmap-sync` CI gate. crates.io / docs.rs badges + versioning-policy section deferred to #136. [spec](plans/done/2026-05-08-project-docs.spec.md)
- **macro-object-bench** — criterion benchmarks for macro-derived objects in the root `quartzite` facade crate; mirrors `quartzite-core/benches/signal_property.rs` (6 bench groups) using `#[derive(Extend, DeriveObject)]` + `#[object_impl]` fixture. [spec](plans/done/2026-05-07-macro-object-bench.spec.md)
- **criterion-benchmarks** — `quartzite-core/benches/signal_property.rs` + `quartzite-runtime/benches/object_tree.rs` + Bencher CI workflows (3 workflows). [spec](plans/done/2026-05-07-criterion-benchmarks.spec.md)
- **codegen-inline-concrete-trait-impls** — restored `#[inline]` on concrete-struct trait-impl method emissions in all three macro codegen modules; codegen branches on whether the user struct introduces type/const params. 11 new codegen tests. [spec](plans/done/2026-05-07-codegen-inline-concrete-trait-impls.spec.md)
- **codegen-simple-marker** — dropped `#[inline]` from generated trait-impl methods (PR #120 follow-up); added missing `/// _Simple._` to `IntoValue::into_value` trait declaration. Trait-declaration tag is the sole canonical signal; generated impls rely on rustdoc inheritance. [spec](plans/done/2026-05-07-codegen-simple-marker.spec.md)
- **code-style-extraction** — lifted AGENTS.md `## Code Style` body into `ai-docs/code-style.md` mirroring `doc-convention.md`'s shape; AGENTS.md retains a 10-bullet index linking to cluster anchors. `code-style` added as a new `Escalated?` target. [spec](plans/done/2026-05-07-code-style-extraction.spec.md)
- **generic-fn-split** — applied "Generic-fn split for binary size" to `ObjectTree::rename`, `ObjectFactory::register`, `Timer::named`, `ObjectBase::named`; outer fn carries `/// _Simple._` and delegates to nested `fn inner`. [spec](plans/done/2026-05-07-generic-fn-split.spec.md)
- **recursive-inline-annotations** — annotation-only sweep applying the recursive `#[inline]` / `_Simple._` rule across the workspace; cascade verified quiescent. [spec](plans/done/2026-05-07-recursive-inline-annotations.spec.md)
- **tracing-spans** — `*_span!` guards replace bare `debug!`/`trace!` announcements in `object_tree`, `timer`, `event_loop`, `timer_drivers`; `signal::emit` + `event_loop::post` gated behind `verbose-tracing` cargo feature. [spec](plans/done/2026-05-06-tracing-spans.spec.md)
- **signal-to-signal** — new `quartzite-core::connect` module: `Object::emit_signal`, `ArgsToValues`, `connect_signal_to_signal`, `connect_signals`, `SignalConnectionError`; std-only; re-exported via `quartzite::prelude`. [spec](plans/done/2026-05-06-signal-to-signal.spec.md)
- **per-thread-event-loops** — `LoopRegistry` singleton, `EventLoop::install/uninstall_for_current_thread`, `EventLoop::spawn`, RAII `RegistryGuard`; `QueuedDispatcher::post` gains `target: ThreadId`; `ConnectionTable` routes via registry. [spec](plans/done/2026-05-06-per-thread-event-loops.spec.md)
- **doc-convention** — workspace-wide doc-comment convention from RFC 1574 + deterministic.space; strict section order; clippy `missing_errors_doc` / `missing_panics_doc` / `missing_safety_doc` / `doc_markdown` enabled across all crates. Canonical reference at [`ai-docs/doc-convention.md`](doc-convention.md). [spec](plans/done/2026-05-05-doc-convention.spec.md)
- **thiserror-migration** — `ApplicationError`, `FactoryAlreadySet`, `DispatcherAlreadySet`, `TypeError` migrated to `#[derive(thiserror::Error)]`; `thiserror = "2"` added to `quartzite-core`. [spec](plans/done/2026-05-05-thiserror-migration.spec.md)
- **tracing-itertools** — `tracing = "0.1"` added to `quartzite-core` (no_std-gated) + `quartzite-runtime`; `tracing/log` feature enabled; `itertools = "0.14"` as dev-dep; `env_logger` in examples. [spec](plans/done/2026-05-05-tracing-itertools.spec.md)
- **signal-emit-rename** — `Signal::emit_unless_blocked` removed; `Signal::emit` is now unconditional; `emit!` macro owns the `signals_blocked` guard; generated wrappers use `emit!`. [spec](plans/done/2026-05-05-signal-emit-rename.spec.md)
- **receiver-guard-auto** — `Weak<ReceiverGuard>` for Auto connections + `connect_<signal>_auto` codegen. [spec](plans/done/2026-05-03-receiver-guard-auto.spec.md)
- **connect-queued-codegen** — `connect_<signal>_queued` typed wrappers. [spec](plans/done/2026-05-03-connect-queued-codegen.spec.md)
- **enumflags2-property-flags** — `PropertyFlags` replaced by `BitFlags<PropertyFlag>` backed by `u8` via `enumflags2`. [spec](plans/done/2026-05-03-enumflags2-property-flags.spec.md)
- **objectbase-debug-rename-factory** — `ObjectBase: Debug`, `rename` no-op, `ObjectFactory` singleton. [spec](plans/done/2026-05-03-objectbase-debug-rename-factory.spec.md)
- **code-quality-cleanup** — code-quality sweep across `quartzite-macros`, `quartzite-runtime`, `quartzite-core`. [spec](plans/done/2026-05-02-code-quality-cleanup.spec.md)
- **docs-and-facade** — facade-crate documentation pass. [spec](plans/done/2026-05-02-docs-and-facade.spec.md)
- **public-api-docs** — public-API doctest pass; 47 new doctests. [spec](plans/done/2026-05-02-public-api-docs.spec.md)
- **lookup-perf** — O(1) signal disconnect, name index, match-based meta lookup; 21 new tests. [spec](plans/done/2026-05-02-lookup-perf.spec.md)
- **inline-simple-fns** — `#[inline]` on simple non-generic fns. [spec](plans/done/2026-05-02-inline-simple-fns.spec.md)
- **examples-crate** — `quartzite-examples` crate with runnable API examples. [spec](plans/done/2026-05-02-examples-crate.spec.md)
- **signals-blocked** — typed emit wrappers + `signals_blocked` guard. [spec](plans/done/2026-05-02-signals-blocked.spec.md)
- **auto-connection** — signal/slot Auto connection extension. [spec](plans/done/2026-05-01-auto-connection.spec.md)
- **geometry-events** — `quartzite-geometry` + `quartzite-events` crates. [spec](plans/done/2026-05-01-geometry-events.spec.md)

## Open Questions

- Async/await integration strategy — `futures-util` was evaluated and **deferred**: the runtime uses raw `std::thread` + `mpsc` channels with no async call sites. `futures-util` should be added only once an async executor strategy (tokio, async-std, or custom) is decided. Tracked in #89.
- Accessibility (a11y) support
- No-std support scope (core only, or further? — geometry + events are also no_std + alloc)
