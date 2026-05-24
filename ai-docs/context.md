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
| `quartzite-paint-api` | Thin `no_std` shared paint vocabulary: `Color` (with `with_alpha`), `Pen`, `Brush`/`BrushKind` (`Solid`, `LinearGradient`, `RadialGradient`, `Custom(peniko::Gradient)` — `#[non_exhaustive]`, `Clone+Debug+PartialEq`, no `Copy`), `Font`/`FontWeight`, `Image`/`ImageError`, `Path`/`Segment`, 13-method `Painter` trait (adds `text_carets` + `text_visual_lines`), `PaintError`; `TextCaretCursor` + `TextVisualLineCursor` borrowed cursor traits + `TextVisualLine` POD for pixel-snapped caret/wrap-line queries; `peniko 0.6` dep (no-std, libm) for `Custom` variant — no other platform deps; used by `quartzite-paint`, `quartzite-renderer`, `quartzite-widgets`, and `quartzite-style-types` |
| `quartzite-paint` | Re-export shell: `quartzite-paint-api` types plus `Alignment` from `quartzite-geometry` plus peniko gradient types (`Gradient`, `GradientKind`, `ColorStop`, `Extend`) for `Brush::custom_gradient` callers |
| `quartzite-paint-util` | RAII painting utilities (`no_std`, `std` default feature). `TranslateGuard<'a>`: wraps `&'a mut dyn Painter`, calls `save`+`translate(origin)` on construction, `restore` on drop. Panic-safe; `#[inline]` on all methods. Depends on `quartzite-paint-api` + `quartzite-geometry`. Used by `quartzite-style-dispatch` to replace the inline `save`/`translate`/`restore` triplet. |
| `quartzite-renderer` | Windowed rendering backend — `vello` + `wgpu` + `winit`; `WindowedApplication`, `WindowedAppHandler` (`on_start`, `on_last_window_closed`), `WindowRegistry` (`IndexMap`-backed fan-out, `try_create_window`, `windows()`), `WindowedApplicationBuilder` (Linux `with_any_thread` for test threads); `VelloPainter` (full `Painter` impl — text via `parley` + `skrifa`; transform/clip stack; `draw_path` converts `quartzite_paint_api::Segment` to `kurbo`; implements `TextCaretCursor` + `TextVisualLineCursor` backed by parley layout pipeline); `FontCache` (`parley::FontContext` + `LayoutContext<[u8; 4]>` per pipeline entry point); `RenderHarness` (offscreen `image::RgbaImage` harness for snapshot tests; bypasses winit) + `RenderHarnessBuilder` (`scale_factor(f32)` for HiDPI); `RendererError`. **wgpu pinned to 29** to match vello 0.9 (the harness passes wgpu types into `vello::Renderer::render_to_texture` so versions must unify). New deps: `parley` + `skrifa`. |
| `quartzite-style-types` | Leaf crate (no_std + alloc): `Palette` (2D `[[Color; 3]; 12]` with eager Hover/Pressed derivation) + `ColorRole` (12 variants incl. `FocusRing`) + `ColorGroup` (`Normal`/`Hover`/`Pressed`). Depends only on `quartzite-paint-api` for `Color`. |
| `quartzite-style` | Downstream `std` crate: `Style` trait (with `caret_visible_now` + `prefers_reduced_motion` methods), `StyleRegistry`, `StyleClock` (blink-phase helper; `StyleClock::pinned` for snapshot tests), `DefaultStyle` (clock-bearing struct impl; flat fill/1-px-outline rendering for Button/Label/TextEdit/ScrollArea/Container/LineEdit; `paint_caret`/`paint_selection` helpers for TextEdit in `default_style/text_edit.rs`; `paint_caret_line_edit`/`paint_selection_line_edit` helpers for LineEdit in `default_style/line_edit.rs`; `start_blink_timer` behind `runtime-blink` default-on feature; opt-in registration). Re-exports `Palette`, `ColorRole`, and `ColorGroup` from `quartzite-style-types`. |
| `quartzite-style-dispatch` | Bridge crate: `dispatch_paint` free function + `WidgetResolver` trait (immutable, paint-time). Walks widget tree depth-first, calls `Style::draw_widget` once per visible node; `TranslateGuard` (from `quartzite-paint-util`) for coordinate transforms; no-op when no style installed; `tracing::warn!` on resolver miss. Depends on `quartzite-core`, `quartzite-paint-api`, `quartzite-paint-util`, `quartzite-style`, `quartzite-widgets` — no `quartzite-renderer` dep (no cycle). |
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
| `TextCaretCursor`/`TextVisualLineCursor` design | Two borrowed cursor traits on `Painter` returning `&mut dyn` tied to `&mut self` — no Box allocation, object-safe, lifetime enforces single-borrow-at-a-time. `text_carets(text, font) -> &mut dyn TextCaretCursor`; `text_visual_lines(text, font, wrap_width) -> &mut dyn TextVisualLineCursor`. See [`ai-docs/key-decisions.md` → TextCaretCursor design](key-decisions.md#textcaretcursor-design). |
| `StyleClock` blink mechanism | Half-period of 530 ms; phase formula `(elapsed_ms / 530).is_multiple_of(2)`. `StyleClock::pinned(bool)` for deterministic tests. Owned by `DefaultStyle`; exposed via `Style::caret_visible_now()` + `Style::prefers_reduced_motion()`. `Timer`-driven invalidation via `DefaultStyle::start_blink_timer` (default-on `runtime-blink` feature). |
| `TextEdit` caret/selection state | `caret: usize` (default 0), `selection_anchor: Option<usize>` (default None). `set_caret` is a `#[slot]`; `set_selection_anchor` is a plain `pub fn` (Option<usize> lacks `FromValue`). Both clamp to text length and no-op when read_only or unchanged. `selection_range()` returns `Option<(min,max)>` byte range. |
| `style ↔ widgets` Cargo cycle resolution | Two-crate split: `quartzite-style-types` is the leaf (`Palette`, `ColorRole`); `quartzite-style` is downstream (`Style` trait, `StyleRegistry`). `quartzite-widgets` re-exports `Palette`/`ColorRole` from the leaf only — never depends on `quartzite-style`. Enforced mechanically by `quartzite-widgets/tests/no_style_dep.rs`, which shells out to `cargo tree -p quartzite-widgets --prefix none --no-dedupe` and asserts no line starts with `"quartzite-style "` (trailing space anchors against `quartzite-style-types`). |
| `StyleRegistry` storage | `static OnceLock<Mutex<Option<&'static dyn Style>>>` + `Box::leak`; non-panicking `try_style()` accessor. See [`ai-docs/key-decisions.md` → `StyleRegistry` storage](key-decisions.md#styleregistry-storage). |
| `Style` trait surface | Generic-only — single method `fn draw_widget(&self, widget: &dyn AsWidget, painter: &mut dyn Painter, palette: &Palette)`. Trait bounds `Send + Sync` (required by global registry). No per-widget primitive methods on the trait — concrete `Style` impls call `widget.widget_view()` and pattern-match the **Hybrid `Paint<W>`** dispatch mechanism: typed `Paint<W>` impls per widget type; `WidgetView::Other` arm is documented silent no-op for unknown widgets. See [`ai-docs/key-decisions.md` → `Style` dispatch mechanism](key-decisions.md#style-dispatch-mechanism). |
| `dispatch_paint` / `WidgetResolver` design | Free function (not a method) in a bridge crate; keeps the widget crate free of style deps. `WidgetResolver` trait (immutable `resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>`) is separate from the layout crate's mutable `WidgetResolver`; closure blanket impl for tests. `style: &'static dyn Style` passed down the call stack (taken once from `StyleRegistry::try_style()` at entry). Children enumerated via `AsWidget::children()` (`WidgetChildren<'_>` enum — `Slice`/`Optional`/`Empty`); the former `children_of` downcast removed in #373. |
| `Painter` trait shape (paint-api) | Pass-through (NOT stateful): pen/brush passed as call args rather than stored on the painter. 11 methods total: `draw_rect`/`fill_rect`/`draw_line`/`draw_text`/`draw_text_in`/`draw_image`/`draw_path` (drawing), `clip_rect`/`translate` (transform), `save`/`restore` (state stack — implementation delegated to concrete backend, no internal stack in the trait). Object-safe: no generics, no `Self` returns, no associated types. `draw_text_in` takes two `Alignment` parameters: `h_align` then `v_align` (both horizontal-first in the parameter list — see `# Parameter order` in the trait rustdoc). `Alignment::Justify` is invalid on the vertical axis (debug_assert; falls back to `Left`/top in release). |
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
11. `quartzite-style-types` + `quartzite-style` — leaf (`Palette`, `ColorRole`, `DARK_PALETTE`) + downstream (`Style` trait, `StyleRegistry`) ✅ (paint-style #47; dark palette #488). Two-crate split exists to break the `style ↔ widgets` Cargo cycle; widgets depends on the leaf only, enforced by a `cargo tree -p quartzite-widgets` integration test.
12. `DefaultStyle` concrete implementation ✅ (#290, default-style-content). Flat fill/1-px-outline rendering for Button/Label/TextEdit/ScrollArea with palette-driven colours; checked/disabled/read-only state variants.
16. `DefaultStyle` Container + LineEdit arms ✅ (#318, container-lineedit-rendering). Container: Window fill + WindowText 1px outline. LineEdit: Base fill, read-only WindowText overlay, Text 1px outline, single-line text with half-alpha placeholder when text empty. Test block extracted to `default_style_tests.rs` (sibling `#[path]` module) to keep prod file under 1000 lines.
17. `DefaultStyle` read-only overlay fix ✅ (#458). Replaced `disabled(Window)` overlay (invisible on `Palette::default` where `Window == Base == WHITE`) with `WindowText.with_alpha(0.10)`; read-only text dims to `Text.with_alpha(0.65)`. Applies to both `TextEdit` and `LineEdit`. `text_edit_read_only.png` golden regenerated.
13. `DefaultStyle` GPU snapshot tests ✅ (#297, default-style-snapshot-tests). Seven pixel-level goldens in `quartzite-style/tests/snapshots/shared/` via `RenderHarness`; `quartzite-style/tests/support/mod.rs` mirrors the `quartzite-widgets` helper contract (snapshot-helper sync group). `harness_or_skip` lifted into both `tests/support/mod.rs` modules.
14. `Button` hover/pressed/focused visual states ✅ (#316). `WidgetBase` gained three `pub bool` fields (`hovered`, `pressed`, `focused`) (later collapsed into `pub state: BitFlags<WidgetState>` — see #480 entry in Maintenance plans); `WidgetExt` gains six accessors and four updated event-handler defaults (`on_mouse_press`/`on_mouse_release`/`on_focus_in`/`on_focus_out` now mutate the matching flag). `DefaultStyle::draw_button` consumes the flags: hover/pressed = per-palette `ColorGroup`-derived colours, focused = 2 px `FocusRing` outline (additive, full-alpha). `Color::blend(self, other, t)` added to `quartzite_paint_api::Color` as a reusable `const fn` lerp. Three new snapshot goldens in `shared/`.
18. `ColorGroup` axis + `FocusRing` role ✅ (#402). `quartzite-style-types` gains `ColorGroup` enum (Normal/Hover/Pressed) and `ColorRole::FocusRing` (12th variant). `Palette` refactored to 2D `[[Color; 3]; 12]` storage with eager derivation in `const fn new()` (Hover: blend 6% toward WindowText, Pressed: blend 16%). `DARK_PALETTE` updated with per-state seeds. All `DefaultStyle` palette call sites migrated to 2-arg API. 4 snapshot goldens regenerated (`button_hovered/pressed` light+dark).
19. `Label` / `TextEdit` / `ScrollArea` hover/pressed/focused visual states ✅ (#403). Generalises the Button #316 state-rendering pattern across the three remaining interactive widgets using the post-#402 two-axis palette API. Private `#[inline] const fn state_group(pressed, hovered) -> ColorGroup` extracted into `default_style.rs` (Button impl refactored to call it — same skeleton, no behaviour change). Each new `impl Paint<W> for DefaultStyle` reads `enabled` / `hovered` / `pressed` / `focused`, swaps roles on `pressed` (Label/TextEdit text → `HighlightedText`, fill → `Highlight`; ScrollArea fill → `Highlight`, outline → `HighlightedText`), widens outline to 2 px `FocusRing × Normal` on `focused` (full-alpha — `FocusRing` exempt from `maybe_disabled`, matching design-system additive-overlay rule + Button). TextEdit read-only overlay paint-order preserved (`FillRect(state_fill)` → `FillRect(overlay)` → `DrawRect(outline)` → `DrawTextIn`); read-only text dim composes over state-resolved text colour. 12 new snapshot goldens (9 light + 3 dark `dark_*_focused`). `default_style.rs` 280 → 358 lines.
20. `LineEdit` hover/pressed/focused visual states + disabled-axis parity with TextEdit ✅ (#406 folds in #407).
25. `LineEdit` caret + selection rendering (#405). `LineEdit` gains `caret: usize`, `selection_anchor: Option<usize>`, `selection_changed: Signal<()>`, `selection_range()` helper, `set_caret` slot, and `set_selection_anchor` plain method — identical selection model to `TextEdit`. `impl Paint<LineEdit> for DefaultStyle` extracted into `default_style/line_edit.rs` (sibling to `text_edit.rs`); `paint_caret_line_edit` uses vertical centring (`caret_y = geom.top() + (geom.size().height() - line_height) / 2`) rather than cursor-reported `line_top`; `paint_selection_line_edit` uses single-rect approach (no `text_visual_lines`, only `text_carets`) since LineEdit is single-line. All seams reused from #317 unchanged. 5 light + 5 dark snapshot goldens. 13 new symbolic-AC tests in `default_style_tests.rs`.
24. `TextEdit` caret + selection rendering (#317). `TextCaretCursor`/`TextVisualLineCursor` cursor traits on `Painter`; `StyleClock` blink helper + `caret_visible_now`/`prefers_reduced_motion` on `Style` trait; `DefaultStyle` converts from unit struct to clock-bearing struct; `paint_caret`/`paint_selection` helpers in `default_style/text_edit.rs`; `TextEdit` gains `caret`, `selection_anchor`, `selection_changed` signal; VelloPainter parley-backed cursor impls; `runtime-blink` feature wires `DefaultStyle::start_blink_timer`; 4 light + 4 dark GPU snapshot goldens; `MockTimerDriver` blink-invalidation integration test. Brings `impl Paint<LineEdit> for DefaultStyle` to parity with the post-#403 widgets using the shared `state_group(pressed, hovered)` selector + post-#402 two-axis palette API. State-resolved + `maybe_disabled`-wrapped `(fill_color, text_color, outline_color_idle)` flow through the preserved 3-arm placeholder/read-only/text-content ladder; placeholder is drawn at `disabled(maybe_disabled(palette.color(text_role, group), enabled))` so its dim composes orthogonally on top of the state-resolved text colour (enabled-placeholder ≈ `× 0.5`, disabled-placeholder ≈ `× 0.25`). `FocusRing` 2 px outline exempt from `maybe_disabled` (full alpha always). **#407 fold-in:** the pre-spec `impl Paint<LineEdit>` applied `maybe_disabled` to none of its colours — disabled-LineEdit was visually identical to enabled-LineEdit; post-spec it matches TextEdit's halved-alpha treatment on `Base` fill + `Text` outline + `Text` glyphs. 10 new recording-painter tests + 9 new snapshot goldens (7 light + 2 dark `dark_line_edit_{idle,focused}`); `line_edit_disabled.png` anchors the visible #407 change. `default_style.rs` 358 → 388 lines.
15. `quartzite-style-dispatch` ✅ (#312, renderer-style-dispatch). Bridge crate: `dispatch_paint(root, resolver, painter, palette)` free function walks the widget tree depth-first (parent-before-child), calling `Style::draw_widget` once per visible node; no-op when no style is installed; `save`/`translate`/`restore` for coordinate transforms via `TranslateGuard` (see #410); `tracing::warn!` on resolver miss. `WidgetResolver` trait (immutable paint-time) + closure blanket impl. `test-support` Cargo feature on `quartzite-style` exposes `StyleRegistry::clear_for_test()` to integration tests. **Facade feature** ✅ (#393): `style-dispatch` feature in `quartzite/Cargo.toml` re-exports `quartzite-style-dispatch` as `quartzite::style_dispatch`; chains `style` + `widgets` prerequisites (single-switch end-to-end usability).
22. `quartzite-paint-util` ✅ (#410, raii-guard-painter-transform). New `no_std` utility crate with public `TranslateGuard<'a>` RAII type: constructor calls `Painter::save` + `Painter::translate(origin)`; `Drop` calls `Painter::restore` exactly once; `painter(&mut self) -> &mut dyn Painter` accessor (chosen over `DerefMut` for explicit object-safety). `#[inline]` on all methods. Panic-safe via standard RAII. Replaces the inline `save`/`translate`/`restore` triplet in `quartzite-style-dispatch::dispatch::visit`. `std` is the default feature (enables panic-safety integration test via `catch_unwind`).
21. `WidgetExt::paint` collapsed into `Style::draw_widget` ✅ (#409). Removed the no-op `paint` default method from `WidgetExt` (zero in-tree overrides existed). Migrated the sole in-tree caller — `snapshot_widget` test helper in `quartzite-widgets/tests/support/mod.rs` — to drive `DefaultStyle.draw_widget(widget, p, &Palette::default())` directly, matching the sync-group sibling pattern from `quartzite-style/tests/snapshots.rs`. `quartzite-widgets/tests/no_style_dep.rs` updated to scope its `cargo tree` assertion to `--edges=normal` (production edges only) to permit `quartzite-style` as a dev-dependency without breaking the cycle-break guard. Single paint dispatch path: `caller → dispatch_paint → Style::draw_widget → Paint<W>` (production); `test harness → Style::draw_widget` (snapshot tests).
26. Design-system conformance audit + vertical-alignment paint-API axis (#555). `Painter::draw_text_in` extended with `h_align + v_align: Alignment` (clean break; all impls updated in lockstep). Button + Label text now vertically centred (`Alignment::Center` on v_align); TextEdit explicitly top-anchored (`Left`); LineEdit PR #554 smaller-rect recipe removed in favour of the new v_align param. `ai-docs/design-conformance-audit.md` documents the full widget × rule conformance matrix. 18 golden PNGs regenerated (button + label light/dark). AC8 symbolic test `button_and_label_use_vertical_centre` + rendering-level v_align test added. `Alignment::Justify` on the vertical axis: debug_assert + Left fallback in release.
23. `signals_blocked` serde persistence ✅ (#39). `ObjectSnapshot` gained `pub signals_blocked: bool` with `#[serde(default)]`; `capture_object` reads the flag from the source object; `restore_object` calls `block_signals()` when the snapshot value is `true`. Decision: persist (not reset); schema version stays at `1` — additive evolution via `#[serde(default)]`. 4 new tests (persist-true unit + integration, persist-false unit, v1-payload-missing-key `→ false`). Docs updated at 4 sites to move `signals_blocked` from "NOT preserved" to "preserved".

### Maintenance plans (cross-cutting)

Cross-cutting plans that touched multiple crates and aren't part of the dependency tree above. All ✅ implemented; full canonical list with per-plan summaries lives in [`plans-summary.md`](plans-summary.md#maintenance-plans-cross-cutting--reference).

## Open Questions

- Async/await integration strategy — `futures-util` was evaluated and **deferred**: the runtime uses raw `std::thread` + `mpsc` channels with no async call sites. `futures-util` should be added only once an async executor strategy (tokio, async-std, or custom) is decided. Tracked in #89.
- Accessibility (a11y) support
- No-std support scope (core only, or further? — geometry + events are also no_std + alloc)
