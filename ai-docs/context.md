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
| `quartzite-paint-api` | Thin `no_std` shared paint vocabulary: `Color` (with `with_alpha`), `Pen`, `Brush`/`BrushKind` (`Solid`, `LinearGradient`, `RadialGradient`, `Custom(peniko::Gradient)` — `#[non_exhaustive]`, `Clone+Debug+PartialEq`, no `Copy`), `Font`/`FontWeight`, `Image`/`ImageError`, `Path`/`Segment`, 11-method `Painter` trait, `PaintError`; `peniko 0.6` dep (no-std, libm) for `Custom` variant — no other platform deps; used by `quartzite-paint`, `quartzite-renderer`, `quartzite-widgets`, and `quartzite-style-types` |
| `quartzite-paint` | Re-export shell: `quartzite-paint-api` types plus `Alignment` from `quartzite-geometry` plus peniko gradient types (`Gradient`, `GradientKind`, `ColorStop`, `Extend`) for `Brush::custom_gradient` callers |
| `quartzite-renderer` | Windowed rendering backend — `vello` + `wgpu` + `winit`; `WindowedApplication`, `WindowedAppHandler` (`on_start`, `on_last_window_closed`), `WindowRegistry` (`IndexMap`-backed fan-out, `try_create_window`, `windows()`), `WindowedApplicationBuilder` (Linux `with_any_thread` for test threads); `VelloPainter` (full `Painter` impl — text via `parley` + `skrifa`; transform/clip stack; `draw_path` converts `quartzite_paint_api::Segment` to `kurbo`); `FontCache` (`parley::FontContext` + `LayoutContext<[u8; 4]>` per pipeline entry point); `RenderHarness` (offscreen `image::RgbaImage` harness for snapshot tests; bypasses winit) + `RenderHarnessBuilder` (`scale_factor(f32)` for HiDPI); `RendererError`. **wgpu pinned to 29** to match vello 0.9 (the harness passes wgpu types into `vello::Renderer::render_to_texture` so versions must unify). New deps: `parley` + `skrifa`. |
| `quartzite-style-types` | Leaf crate (no_std + alloc): `Palette` + `ColorRole` (+ `ColorRole::ALL`). Depends only on `quartzite-paint-api` for `Color`. |
| `quartzite-style` | Downstream `std` crate: `Style` trait, `StyleRegistry`, `DefaultStyle` (zero-sized concrete impl; flat fill/1-px-outline rendering for Button/Label/TextEdit/ScrollArea/Container/LineEdit; opt-in registration). Re-exports `Palette` and `ColorRole` from `quartzite-style-types`. |
| `quartzite-style-dispatch` | Bridge crate: `dispatch_paint` free function + `WidgetResolver` trait (immutable, paint-time). Walks widget tree depth-first, calls `Style::draw_widget` once per visible node; `save`/`translate`/`restore` for coordinate transforms; no-op when no style installed; `tracing::warn!` on resolver miss. Depends on `quartzite-core`, `quartzite-paint-api`, `quartzite-style`, `quartzite-widgets` — no `quartzite-renderer` dep (no cycle). |
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
| Macro codegen paths | `quartzite-macros` resolves the crate path at expansion time via `proc_macro_crate` (facade-first fallback chain). See [`ai-docs/key-decisions.md` → Macro codegen paths](key-decisions.md#macro-codegen-paths). |
| `derive` feature | `quartzite-macros` is an optional dep gated on the `derive` feature (on by default); disable to skip proc-macro compilation in macro-free or `no_std` builds |
| Object ownership | Arena/SlotMap — `ObjectTree` + `ObjectId` + `Mutex<ObjectTree>` in Application |
| `ConnectionType::Auto` | Same-thread → Direct; cross-thread → Queued; `Weak<ReceiverGuard>` checked on dispatch; std-gated. See [`ai-docs/key-decisions.md` → `ConnectionType::Auto`](key-decisions.md#connectiontypeauto). |
| Signal slot storage | `IndexMap<ConnectionId, …>` — insertion-order preserved; O(1) disconnect via `shift_remove`. See [`ai-docs/key-decisions.md` → Signal slot storage](key-decisions.md#signal-slot-storage). |
| `ObjectBase: Debug` | `#[derive(Debug)]` on `ObjectBase` and `ReceiverGuard`; the `thread_id` field auto-respects the `std` cfg-gate. See [`ai-docs/key-decisions.md` → `ObjectBase: Debug`](key-decisions.md#objectbase-debug). |
| `ObjectBase::name` | `Option<String>`; `None` = unnamed (not indexed); `Some("")` = explicitly named `""` (indexed). Mutation only via `ObjectTree::rename` / `ObjectTree::clear_name`. |
| `ObjectTree::rename` no-op | Same-name rename returns immediately (no mutation, no event). See [`ai-docs/key-decisions.md` → `ObjectTree::rename` no-op](key-decisions.md#objecttreerename-no-op). |
| `ObjectFactory` singleton | `static OnceLock<Arc<RwLock<ObjectFactory>>>` with first-call-wins `install`. See [`ai-docs/key-decisions.md` → `ObjectFactory` singleton](key-decisions.md#objectfactory-singleton). |
| `ObjectTree::find_by_name` | Returns `&[ObjectId]` — all objects with that name, insertion order. Backed by `HashMap<String, Vec<ObjectId>>` index. |
| `ObjectTree::find_by_name_in` | Returns `Vec<ObjectId>` of descendants-or-self of `root` whose name matches, in BFS (shallowest-first) order. Returns empty if `root` is not in the tree or no match is found. |
| `ObjectBase::name_changed` | Built-in `Signal<(Option<String>, Option<String>)>` on every object; payload is `(old_name, new_name)`. See [`ai-docs/key-decisions.md` → `ObjectBase::name_changed`](key-decisions.md#objectbasename_changed). |
| `name_changed` codegen | `#[derive(Object)]` synthesises a phantom built-in `SignalField` for `name_changed`; no typed wrappers — use dynamic `emit_signal`/`connect_signal`. See [`ai-docs/key-decisions.md` → `name_changed` codegen](key-decisions.md#name_changed-codegen). |
| `Option<String>` Value encoding | `IntoValue`/`FromValue` impls: `None` ↔ `Value::Null`, `Some(s)` ↔ `Value::String(s)`. See [`ai-docs/key-decisions.md` → `Option<String>` Value encoding](key-decisions.md#optionstring-value-encoding). |
| Process-global tree accessor | `AtomicPtr<Mutex<ObjectTree>>` + `try_with_tree<R>(f) -> Option<R>` (None when no Application live). See [`ai-docs/key-decisions.md` → Process-global tree accessor](key-decisions.md#process-global-tree-accessor). |
| `ObjectBase::new_with_id` | Caller-supplied `ObjectId`; uniqueness is the caller's contract. See [`ai-docs/key-decisions.md` → `ObjectBase::new_with_id`](key-decisions.md#objectbasenew_with_id). |
| Snapshot layer (`serde` feature) | Three-level (`Value` / `ObjectSnapshot` / `TreeSnapshot`) under `serde` feature; restore via `ObjectFactory::global()`. See [`ai-docs/key-decisions.md` → Snapshot layer (`serde` feature)](key-decisions.md#snapshot-layer-serde-feature). |
| MetaObject / EnumMeta lookup | fn-pointer fields — O(1) match dispatch; lookup fns generated in hidden mods. See [`ai-docs/key-decisions.md` → MetaObject / EnumMeta lookup](key-decisions.md#metaobject--enummeta-lookup). |
| `ObjectBase::signals_blocked` | Private `bool` field; toggled via `block_signals()` / `unblock_signals()`. Use `emit!(self.signal_field, &args)` macro to suppress emission automatically; `Signal::emit` is unconditional. |
| `emit!` macro | `quartzite_core::emit!(receiver.field, &args)` — binds `__blocked` via `let`, then `if !__blocked { …emit(&args) }`. See [`ai-docs/key-decisions.md` → `emit!` macro](key-decisions.md#emit-macro). |
| `emit_<signal>` codegen | `#[derive(Object)]` generates `emit_<signal>(&mut self, …)` flat-args wrappers delegating to `emit!`. See [`ai-docs/key-decisions.md` → `emit_<signal>` codegen](key-decisions.md#emit_signal-codegen). |
| `connect_<signal>_auto` codegen | `#[derive(Object)]` generates `connect_<signal>_auto(&mut self, receiver: &ObjectBase, f: F)` (std-gated). See [`ai-docs/key-decisions.md` → `connect_<signal>_auto` codegen](key-decisions.md#connect_signal_auto-codegen). |
| Multi-block `#[object_impl]` | `#[object_part]` accumulates into a `thread_local!` HashMap; `#[object_impl]` auto-detects terminal mode and drains. See [`ai-docs/key-decisions.md` → Multi-block `#[object_impl]`](key-decisions.md#multi-block-object_impl). |
| Generic `#[derive(Extend)]` | Non-root generic structs supported via `split_for_impl()` with minimal bounds; root + generic rejected at parse time. See [`ai-docs/key-decisions.md` → Generic `#[derive(Extend)]`](key-decisions.md#generic-deriveextend). |
| `connect_<signal>_queued` codegen | `#[derive(Object)]` generates `connect_<signal>_queued(&mut self, receiver: &ObjectBase, f: F)` (std-gated). See [`ai-docs/key-decisions.md` → `connect_<signal>_queued` codegen](key-decisions.md#connect_signal_queued-codegen). |
| Per-thread `LoopRegistry` | `LazyLock<RwLock<HashMap<ThreadId, Arc<EventLoop>>>>`; `install/uninstall_for_current_thread` + RAII `RegistryGuard`. See [`ai-docs/key-decisions.md` → Per-thread `LoopRegistry`](key-decisions.md#per-thread-loopregistry). |
| `QueuedDispatcher::post` signature | `fn post(&self, target: ThreadId, f: Box<dyn FnOnce() + Send + 'static>)` — explicit thread routing. `ConnectionTable` looks up `LoopRegistry::get(target)` and posts to the found loop; emits `tracing::warn!` + drops `f` if no loop is registered (documented on trait). |
| `quartzite-geometry` no_std | Pure `no_std` with no alloc — all types are `Copy` stack values. `f32::round()` unavailable in no_std; `libm::roundf` is used instead (always-on dep, no opt-out). `PointF → Point` rounds half-away-from-zero. |
| `ObjectId`/`ConnectionId` ordering | `#[derive(PartialOrd, Ord)]` on both `u64` newtypes (allocation-order); used by `PoolDriver` heap. See [`ai-docs/key-decisions.md` → `ObjectId`/`ConnectionId` ordering](key-decisions.md#objectidconnectionid-ordering). |
| `Timer` pluggable driver | `TimerDriver` trait with `start(TimerConfig, callback)` + `stop(ObjectId)`. Three built-in drivers: `ThreadDriver` (one thread per timer, `park_timeout` + `unpark`), `AppDriver` (posts `Box<dyn FnOnce>` via `Application::global()`), `PoolDriver` (single background thread + `BinaryHeap<Reverse<(Instant, ObjectId)>>` + single `Mutex<PoolState>` + `Condvar`). |
| `Timer::tick` signal isolation | `tick: Arc<Mutex<Signal<(usize,)>>>` shared between `Timer` and `TimerState::signal` — NOT `#[signal]` (would create a duplicate). See [`ai-docs/key-decisions.md` → `Timer::tick` signal isolation](key-decisions.md#timertick-signal-isolation). |
| `signals_blocked` two-copy sync | Driver-side `TimerState::signals_blocked: AtomicBool` mirrors `base.signals_blocked`; both updated by `Timer::block_signals()`. See [`ai-docs/key-decisions.md` → `signals_blocked` two-copy sync](key-decisions.md#signals_blocked-two-copy-sync). |
| Signal-to-signal connections | Two APIs in `quartzite-core::connect`: `connect_signal_to_signal` (dynamic) and `connect_signals` (typed). See [`ai-docs/key-decisions.md` → Signal-to-signal connections](key-decisions.md#signal-to-signal-connections). |
| `Timer` implements `Object` | `Timer` manually implements `AsObject` + `Object` (runtime has no macros dep); static `TIMER_META: MetaObject`. See [`ai-docs/key-decisions.md` → `Timer` implements `Object`](key-decisions.md#timer-implements-object). |
| `quartzite-events` no_std | `no_std + alloc` — needs `String` for `KeyEvent::text`. `MouseButton` and `KeyModifiers` use `bitflags!` (u8). |
| `EventType<T>` shape | Nested enums `Key(KeyEventKind)`, `Mouse(MouseEventKind)`; generic `T: 'static + Send + Sync = ()` for `User(T)` payload (winit style). See [`ai-docs/key-decisions.md` → `EventType<T>` shape](key-decisions.md#eventtypet-shape). |
| `PropertyFlags` representation | `pub type PropertyFlags = BitFlags<PropertyFlag>` via `enumflags2`; codegen uses `make_bitflags!`. See [`ai-docs/key-decisions.md` → `PropertyFlags` representation](key-decisions.md#propertyflags-representation). |
| `style ↔ widgets` Cargo cycle resolution | Two-crate split: `quartzite-style-types` is the leaf (`Palette`, `ColorRole`); `quartzite-style` is downstream (`Style` trait, `StyleRegistry`). `quartzite-widgets` re-exports `Palette`/`ColorRole` from the leaf only — never depends on `quartzite-style`. Enforced mechanically by `quartzite-widgets/tests/no_style_dep.rs`, which shells out to `cargo tree -p quartzite-widgets --prefix none --no-dedupe` and asserts no line starts with `"quartzite-style "` (trailing space anchors against `quartzite-style-types`). |
| `StyleRegistry` storage | `static OnceLock<Mutex<Option<&'static dyn Style>>>` + `Box::leak`; non-panicking `try_style()` accessor. See [`ai-docs/key-decisions.md` → `StyleRegistry` storage](key-decisions.md#styleregistry-storage). |
| `Style` trait surface | Generic-only — single method `fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette)`. Trait bounds `Send + Sync` (required by global registry). No per-widget primitive methods on the trait — concrete `Style` impls call `widget.widget_view()` and pattern-match the **Hybrid `Paint<W>`** dispatch mechanism: typed `Paint<W>` impls per widget type; `WidgetView::Other` arm is documented silent no-op for unknown widgets. See [`ai-docs/key-decisions.md` → `Style` dispatch mechanism](key-decisions.md#style-dispatch-mechanism). |
| `dispatch_paint` / `WidgetResolver` design | Free function (not a method) in a bridge crate; keeps the widget crate free of style deps. `WidgetResolver` trait (immutable `resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>`) is separate from the layout crate's mutable `WidgetResolver`; closure blanket impl for tests. `style: &'static dyn Style` passed down the call stack (taken once from `StyleRegistry::try_style()` at entry). Children enumerated via `AsWidget::children()` (`WidgetChildren<'_>` enum — `Slice`/`Optional`/`Empty`); the former `children_of` downcast removed in #373. |
| `Painter` trait shape (paint-api) | Pass-through (NOT stateful): pen/brush passed as call args rather than stored on the painter. 11 methods total: `draw_rect`/`fill_rect`/`draw_line`/`draw_text`/`draw_text_in`/`draw_image`/`draw_path` (drawing), `clip_rect`/`translate` (transform), `save`/`restore` (state stack — implementation delegated to concrete backend, no internal stack in the trait). Object-safe: no generics, no `Self` returns, no associated types. |
| `Color` representation | `f32` channels in `[0.0, 1.0]` (NOT u8). `Color::new(r,g,b,a)`, named constants (`BLACK`/`WHITE`/`RED`/`GREEN`/`BLUE`/`TRANSPARENT`), `with_alpha(a: f32) -> Color` const fn for builder-style alpha replacement. |
| `Path` representation | `Vec<Segment>` builder in `quartzite-paint-api`; `move_to`/`line_to`/`cubic_to`/`arc_to`/`close` return `&mut Self`. See [`ai-docs/key-decisions.md` → `Path` representation](key-decisions.md#path-representation). |
| `Image` representation | `Image { width, height, pixels: Vec<u8> }` RGBA8 row-major; `Image::try_new` returns `Result<_, ImageError>`. See [`ai-docs/key-decisions.md` → `Image` representation](key-decisions.md#image-representation). |
| `Alignment` location | `quartzite-geometry::Alignment` (moved in paint-style #47); `quartzite-widgets::Alignment` is a `pub use` re-export. See [`ai-docs/key-decisions.md` → `Alignment` location](key-decisions.md#alignment-location). |
| Multi-window `ActiveEventLoop` slot | `Cell<*const ActiveEventLoop>` in `WindowRegistry` armed by `WrappedHandler` for the user-code callback. See [`ai-docs/key-decisions.md` → Multi-window `ActiveEventLoop` slot](key-decisions.md#multi-window-activeeventloop-slot). |
| `WindowEntry` drop order | `surface: wgpu::Surface<'static>` declared before `window: Arc<Window>` so the surface drops first — reverse order is use-after-free. See [`ai-docs/key-decisions.md` → `WindowEntry` drop order](key-decisions.md#windowentry-drop-order). |
| `quit_on_last_window_closed` policy | Configurable on `WindowedApplicationBuilder` (default `true`); off-mode keeps the loop alive until `AppEvent::Exit`. See [`ai-docs/key-decisions.md` → `quit_on_last_window_closed` policy](key-decisions.md#quit_on_last_window_closed-policy). |
| `AppEvent` / proxy exit | `EventLoop<AppEvent>` + `event_proxy() -> EventLoopProxy<AppEvent>` is the only cross-thread exit path (`winit` has no synthesise-`WindowEvent` API). See [`ai-docs/key-decisions.md` → `AppEvent` / proxy exit](key-decisions.md#appevent--proxy-exit). |

## Plans (Implementation Order)

Crate-level plans:

1. `quartzite-core` — core types + traits + signal + value ✅
2. `quartzite-macros` — Extend + Object + object_impl derive macros ✅
3. `quartzite-runtime` — Application, EventLoop, ObjectTree ✅
4. `quartzite` (facade) — prelude re-exports, sub-crate re-exports, Cargo metadata, docs.rs config ✅
5. `examples/` — runnable API examples at workspace root (hello_object, signals_slots, object_tree, timer) ✅
6. `quartzite-geometry` + `quartzite-events` — geometry primitives + event model ✅
7. `quartzite-paint-api` — thin no_std shared paint types + `Painter` trait; wired into `quartzite` facade ✅ (graphics-stack #73; extended in paint-style #47 with `Color::with_alpha`, `Font`/`FontWeight`, `Image`/`ImageError`, `Path`/`Segment`, and four new `Painter` methods `draw_text`/`draw_text_in`/`draw_image`/`draw_path`; gradient variants #281 add `BrushKind::LinearGradient`, `RadialGradient`, `Custom(peniko::Gradient)`)
8. `quartzite-paint` — re-export shell over `quartzite-paint-api` + `Alignment` from `quartzite-geometry` + peniko gradient re-exports ✅ (graphics-stack #73; full vocabulary completed in paint-style #47; peniko re-exports added in #281)
9. `quartzite-renderer` — `WindowedApplication` + `VelloPainter` (vello+wgpu+winit) ✅ (graphics-stack #73; new `Painter` methods land as no-op stubs in paint-style #47; real implementations + text rendering via parley/skrifa in renderer-painter-impls #289/#277)
10. `quartzite-widgets` — WidgetBase, WidgetExt, BoxLayout, GridLayout, Label, Button, LineEdit, TextEdit, ScrollArea, Container ✅ (#46; refactored in paint-style #47 to drop local `Alignment`/`Font`/`Palette` and re-export from upstream)
11. `quartzite-style-types` + `quartzite-style` — leaf (`Palette`, `ColorRole`) + downstream (`Style` trait, `StyleRegistry`) ✅ (paint-style #47). Two-crate split exists to break the `style ↔ widgets` Cargo cycle; widgets depends on the leaf only, enforced by a `cargo tree -p quartzite-widgets` integration test.
12. `DefaultStyle` concrete implementation ✅ (#290, default-style-content). Flat fill/1-px-outline rendering for Button/Label/TextEdit/ScrollArea with palette-driven colours; checked/disabled/read-only state variants.
16. `DefaultStyle` Container + LineEdit arms ✅ (#318, container-lineedit-rendering). Container: Window fill + WindowText 1px outline. LineEdit: Base fill, read-only WindowText overlay, Text 1px outline, single-line text with half-alpha placeholder when text empty. Test block extracted to `default_style_tests.rs` (sibling `#[path]` module) to keep prod file under 1000 lines.
17. `DefaultStyle` read-only overlay fix ✅ (#458). Replaced `disabled(Window)` overlay (invisible on `Palette::default` where `Window == Base == WHITE`) with `WindowText.with_alpha(0.10)`; read-only text dims to `Text.with_alpha(0.65)`. Applies to both `TextEdit` and `LineEdit`. `text_edit_read_only.png` golden regenerated.
13. `DefaultStyle` GPU snapshot tests ✅ (#297, default-style-snapshot-tests). Seven pixel-level goldens in `quartzite-style/tests/snapshots/shared/` via `RenderHarness`; `quartzite-style/tests/support/mod.rs` mirrors the `quartzite-widgets` helper contract (snapshot-helper sync group). `harness_or_skip` lifted into both `tests/support/mod.rs` modules.
14. `Button` hover/pressed/focused visual states ✅ (#316). `WidgetBase` gained three `pub bool` fields (`hovered`, `pressed`, `focused`) (later collapsed into `pub state: BitFlags<WidgetState>` — see #480 entry in Maintenance plans); `WidgetExt` gains six accessors and four updated event-handler defaults (`on_mouse_press`/`on_mouse_release`/`on_focus_in`/`on_focus_out` now mutate the matching flag). `DefaultStyle::draw_button` consumes the flags: hover = 25% blend toward Highlight, pressed = Highlight role-swap, focused = 2 px Highlight outline (additive). `Color::blend(self, other, t)` added to `quartzite_paint_api::Color` as a reusable `const fn` lerp. Three new snapshot goldens in `shared/`.
15. `quartzite-style-dispatch` ✅ (#312, renderer-style-dispatch). Bridge crate: `dispatch_paint(root, resolver, painter, palette)` free function walks the widget tree depth-first (parent-before-child), calling `Style::draw_widget` once per visible node; no-op when no style is installed; `save`/`translate`/`restore` for coordinate transforms; `tracing::warn!` on resolver miss. `WidgetResolver` trait (immutable paint-time) + closure blanket impl. `test-support` Cargo feature on `quartzite-style` exposes `StyleRegistry::clear_for_test()` to integration tests.

### Maintenance plans (cross-cutting)

Cross-cutting plans that touched multiple crates and aren't part of the dependency tree above. All ✅ implemented; full detail in linked spec.

- **fix-missing-const-for-fn** — promoted all 59 `missing_const_for_fn` lint hits to `const fn` across 26 files (14 in `quartzite-core`, 10 in `quartzite-events`, 9 in `quartzite-renderer`, 10 in paint/style cluster, 16 across widgets/runtime/geometry/macros-tests/examples); cascade-unlock sites in `quartzite-geometry/src/margins.rs` and additional renderer/runtime sites promoted beyond the original survey; single per-item `#[allow(…, reason = "libm::roundf is not const fn on the no_std path")]` for `round_f32`; workspace-level `missing_const_for_fn = "allow"` removed from `Cargo.toml`. [spec](plans/done/2026-05-19-fix-missing-const-for-fn.spec.md)
- **const-default-fn** — added `pub const fn new() -> Self` to 4 Group (A) types: `Path` (`quartzite-paint-api`), `CloseEvent` (`quartzite-events`), `Palette` (`quartzite-style-types`), `DefaultStyle` (`quartzite-style`). Callers gain a `const fn` path usable in `const`/`static` initialisers. `Path`, `CloseEvent`, `DefaultStyle` keep `#[derive(Default)]` (derived body is correct); `Palette` uses an explicit `impl Default { Self::new() }` (body not derivable). 18 Group (B) types (atomics, `Arc`, `mpsc`, FFI) left untouched — zero liftable. Geometry types (`Point`, `PointF`, `Rect`, `RectF`, `Size`, `SizeF`, `Margins`) already conformant (multi-arg `pub const fn new(…)` already existed). [spec](plans/done/2026-05-19-const-default-fn.spec.md)
- **widgetbase-bool-bitflags** — collapsed 6 `pub bool` fields on `WidgetBase` (`visible`, `enabled`, `pending_update`, `hovered`, `pressed`, `focused`) into a single `pub state: BitFlags<WidgetState>` backed by `enumflags2`; `WidgetState` re-exported from `quartzite-widgets`; `WidgetExt` accessor signatures unchanged; `#[allow(clippy::struct_excessive_bools)]` removed. [spec](plans/done/2026-05-19-widgetbase-bool-bitflags.spec.md)
- **audit-workspace-clippy-allows** — audited all 15 workspace-level `allow` entries (11 audits) from PR #423; 9 removed and narrowed to per-item `#[allow(clippy::..., reason = "…")]` at ~100 sites across 8 crates; 6 kept with refreshed hit-count comments; `option_if_let_else` 4 rewrites + `needless_pass_by_value` local fix. Key insight: `[lints] workspace = true` and per-crate `[lints.clippy]` are mutually exclusive in Cargo. [spec](plans/done/2026-05-18-audit-workspace-clippy-allows.spec.md)
- **escalate-clippy-warns-deny** — escalated three workspace clippy lints from `warn` to `deny` at the declaration level in root `Cargo.toml`'s `[workspace.lints.clippy]` table: `undocumented_unsafe_blocks`, `large_stack_frames`, `large_stack_arrays`. Behavioural delta: local non-flagged `cargo clippy` now hard-fails on a violation of any of the three (previously, only `cargo clippy -- -D warnings` in CI did). The `pedantic` and `nursery` group entries remain at `warn` (escalating them is a footgun across toolchain bumps); the `-D warnings` CI flag stays as belt-and-braces. In-tree hit count for all three lints was 0 pre-edit and remains 0 post-edit. Option-3 / full sweep: the `undocumented_unsafe_blocks` text mirrors were synchronised across `AGENTS.md`, `ai-docs/code-style.md` (×2), `.claude/agents/self-review.md`, `.claude/agents/review-findings.md`, and `.claude/skills/task/reference.md`; the `large_stack_*` prose at `ai-docs/code-style.md:44–46` also updated. [spec](plans/done/2026-05-18-escalate-clippy-warns-deny.spec.md)
- **wire-design-system-context** — wired the `design-system/` folder into the Claude Code agent prompts as a **conditional** context source (loaded on demand for visual work, never auto-imported). Four touchpoints: `AGENTS.md` (new `## Design system` pointer section after `## Project`, +799 B; new `## Agent Docs` row for the pointer skill); `.claude/agents/design.md` (bullet under `## Read before designing` naming `design-system/SKILL.md` + `README.md`, qualified to UI-touching `quartzite-widgets` paint / `quartzite-style` / user-facing surfaces); `.claude/agents/design-review.md` (Step 2 `**Read context**` extension + Step 3 severity-rubric clause flagging deviations from documented visual rules — outline width, radius, derivation formulas, focus overlay — as `major`, same tier as the existing handoff-grouping check); new `.claude/skills/ui-design/SKILL.md` (`name: ui-design`, ~10-line pointer body, slash-discoverable counterpart to `design-system/SKILL.md` whose `name: quartzite-design` is not). Pointer-only throughout — AGENTS.md grows ≤ 1 024 B and CLAUDE.md is unchanged (no `@<file>` auto-import added). [spec](plans/done/2026-05-18-wire-design-system-context.spec.md)
- **ci-clippy-bench-targets** — widened the `Clippy` CI step from `cargo clippy --workspace -- -D warnings` to `cargo clippy --workspace --all-targets -- -D warnings` so bench, integration-test, and example targets are linted on every PR; fixes pre-existing `let_underscore_lock` + `similar_names` violations surfaced by the widened invocation; developer lint command propagated to AGENTS.md, README.md, and 6 skill files. [spec](plans/done/2026-05-18-ci-clippy-bench-targets.spec.md)
- **workspace-lints-lift** — lifted the 7 per-crate doc-lint directives (`#![deny(missing_docs)]`, `#![deny(rustdoc::broken_intra_doc_links)]`, `#![warn(clippy::undocumented_unsafe_blocks)]`, plus 4 `clippy::missing_*_doc` / `doc_markdown` warns) from every `lib.rs` into root `Cargo.toml`'s `[workspace.lints.{rust,rustdoc,clippy}]` tables; each member crate inherits via the pre-existing `[lints] workspace = true`. The 4 `clippy::missing_*_doc` / `doc_markdown` warns turned out to be pedantic-implied (AC6 live pre-check: pedantic-suppression diff + `jq` cross-validation both ZERO) and were dropped; only `undocumented_unsafe_blocks` is independently declared. Workspace lints have no per-target gating, so `missing_docs = "deny"` now also covers integration test crates + `examples/*.rs`; 17 test files + 4 example files received one-line `//!` crate-docs to satisfy the broadened enforcement (spec amended mid-Step-8). AGENTS.md *Linter posture* + *Documentation* rows and the corresponding `ai-docs/code-style.md` + `ai-docs/doc-convention.md` sections updated to describe the workspace-level mechanism. [spec](plans/done/2026-05-17-workspace-lints-lift.spec.md)
- **tighten-clippy-pedantic-nursery** — enabled `clippy::pedantic` + `clippy::nursery` (both `warn`, `priority = -1`) plus `clippy::large_stack_frames` + `clippy::large_stack_arrays` workspace-wide via `[workspace.lints.clippy]` in root `Cargo.toml`; each member crate opts in with `[lints] workspace = true`; size thresholds materialised in `clippy.toml`. 17-entry workspace-level allow-list (each with `#`-justifying comment); ~243 Bucket B+C+D fixes (mechanical refactors + judgement-call rewrites). AGENTS.md *Linter posture* + `ai-docs/code-style.md § Linter posture` updated to point at the new mechanism. [spec](plans/done/2026-05-17-tighten-clippy-pedantic-nursery.spec.md)
- **draw-widget-type-system-redesign** — replaced `DefaultStyle::draw_widget` downcast chain with **Hybrid `Paint<W>`** dispatch: `WidgetView<'a>` borrowed enum (one variant per built-in + `#[non_exhaustive]` `Other(&dyn AsWidget)`) on the widget side; `Paint<W>` trait in `quartzite-style`; `#[widget_view(variant = "…")]` proc-macro attribute on each built-in; `AsWidget::children()` replaces `children_of` downcast in `quartzite-style-dispatch`. Third-party widgets integrate without modifying `quartzite-widgets`. [spec](plans/done/2026-05-16-draw-widget-type-system-redesign.spec.md)
- **cleanup-progress-issue-derive** — `.claude/skills/pr-merged/scripts/cleanup-progress.sh` now derives the spec-lookup key from the issue number resolved from the merged PR body (not the PR number); idempotent, with stderr warnings on miss. [spec](plans/done/2026-05-13-cleanup-progress-issue-derive.spec.md)
- **shrink-agents-md** — `AGENTS.md` reduced 40,572 → ~31,466 chars; long narrative extracted to `ai-docs/workflow.md` and `ai-docs/corrections-log.md`; all 8 AXIOMs preserved verbatim. [spec](plans/done/2026-05-13-shrink-agents-md.spec.md)
- **project-docs** — README description block, facade-crate `lib.rs` rustdoc, `CONTRIBUTING.md`, auto-generated `ROADMAP.md` via `scripts/gen-roadmap.sh`, `roadmap-sync` CI gate. [spec](plans/done/2026-05-08-project-docs.spec.md)
- **macro-object-bench** — criterion benchmarks for macro-derived objects in the root `quartzite` facade; mirrors the `quartzite-core` 6-group fixture using `#[derive(Extend, DeriveObject)]` + `#[object_impl]`. [spec](plans/done/2026-05-07-macro-object-bench.spec.md)
- **criterion-benchmarks** — `quartzite-core/benches/signal_property.rs` + `quartzite-runtime/benches/object_tree.rs` + 3 Bencher CI workflows. [spec](plans/done/2026-05-07-criterion-benchmarks.spec.md)
- **codegen-inline-concrete-trait-impls** — restored `#[inline]` on concrete-struct trait-impl method emissions across all three codegen modules; branches on user-struct generics. [spec](plans/done/2026-05-07-codegen-inline-concrete-trait-impls.spec.md)
- **codegen-simple-marker** — dropped `#[inline]` from generated trait-impl methods; trait-declaration `/// _Simple._` tag is the sole canonical signal, generated impls inherit via rustdoc. [spec](plans/done/2026-05-07-codegen-simple-marker.spec.md)
- **code-style-extraction** — lifted AGENTS.md `## Code Style` body into `ai-docs/code-style.md`; AGENTS.md retains a bullet index; `code-style` added as a new `Escalated?` target. [spec](plans/done/2026-05-07-code-style-extraction.spec.md)
- **generic-fn-split** — applied "Generic-fn split for binary size" to `ObjectTree::rename`, `ObjectFactory::register`, `Timer::named`, `ObjectBase::named`; outer `_Simple._`, body in nested `fn inner`. [spec](plans/done/2026-05-07-generic-fn-split.spec.md)
- **recursive-inline-annotations** — annotation-only sweep applying the recursive `#[inline]` / `_Simple._` rule across the workspace; cascade verified quiescent. [spec](plans/done/2026-05-07-recursive-inline-annotations.spec.md)
- **tracing-spans** — `*_span!` guards replace bare `debug!`/`trace!` in `object_tree`, `timer`, `event_loop`, `timer_drivers`; high-frequency emit/post gated on `verbose-tracing` feature. [spec](plans/done/2026-05-06-tracing-spans.spec.md)
- **signal-to-signal** — `quartzite-core::connect` module: `Object::emit_signal`, `ArgsToValues`, `connect_signal_to_signal`, `connect_signals`, `SignalConnectionError`; std-only. [spec](plans/done/2026-05-06-signal-to-signal.spec.md)
- **per-thread-event-loops** — `LoopRegistry` singleton + `EventLoop::install/uninstall_for_current_thread` + `spawn` + RAII `RegistryGuard`; `QueuedDispatcher::post` gains `target: ThreadId`. [spec](plans/done/2026-05-06-per-thread-event-loops.spec.md)
- **doc-convention** — workspace doc-comment convention (RFC 1574 + deterministic.space); clippy `missing_errors_doc` / `missing_panics_doc` / `missing_safety_doc` / `doc_markdown` enabled across all crates. [spec](plans/done/2026-05-05-doc-convention.spec.md)
- **thiserror-migration** — `ApplicationError`, `FactoryAlreadySet`, `DispatcherAlreadySet`, `TypeError` migrated to `#[derive(thiserror::Error)]`. [spec](plans/done/2026-05-05-thiserror-migration.spec.md)
- **tracing-itertools** — `tracing` added to `quartzite-core` + `quartzite-runtime` (`tracing/log` feature); `itertools` dev-dep; `env_logger` in examples. [spec](plans/done/2026-05-05-tracing-itertools.spec.md)
- **signal-emit-rename** — `Signal::emit` is now unconditional; `Signal::emit_unless_blocked` removed; `emit!` macro owns the `signals_blocked` guard. [spec](plans/done/2026-05-05-signal-emit-rename.spec.md)
- **receiver-guard-auto** — `Weak<ReceiverGuard>` for Auto connections + `connect_<signal>_auto` codegen. [spec](plans/done/2026-05-03-receiver-guard-auto.spec.md)
- **connect-queued-codegen** — `connect_<signal>_queued` typed wrappers from `#[derive(Object)]`. [spec](plans/done/2026-05-03-connect-queued-codegen.spec.md)
- **enumflags2-property-flags** — `PropertyFlags` replaced by `BitFlags<PropertyFlag>` backed by `u8` via `enumflags2`; `make_bitflags!` codegen; `enumflags2` re-exported `#[doc(hidden)]` from `quartzite-core`. [spec](plans/done/2026-05-03-enumflags2-property-flags.spec.md)
- **objectbase-debug-rename-factory** — `ObjectBase: Debug`, `rename` no-op, `ObjectFactory` global singleton. [spec](plans/done/2026-05-03-objectbase-debug-rename-factory.spec.md)
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
