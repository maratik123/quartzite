# Threading & Runtime

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| No-std validation \| design for it but don't enforce until runtime exists | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | ✅ done | |
| Multi-window support \| needs platform backend first | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | | #53 (closed) |
| Thread event loops (one loop per thread) \| defer until threading model decided | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | | #51 (closed) |
| Stale `thread_id` invalidation \| needs object-mobility / thread-affinity-change API first | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | | #52 |
| `AutoConnection` in no_std \| same gating as `Queued`; unblocked when std feature is defined | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | ✅ done | |
| Unified user event payload subcrate — needs cross-crate design | [timer-object spec](../plans/done/2026-05-05-timer-object.spec.md) |  | #261 |
| Enabling the `std` tracing feature conditionally via `quartzite-core`'s `std` feature flag — straightforward but adds Cargo feature plumbing; low value for now | [tracing-itertools spec](../plans/done/2026-05-05-tracing-itertools.spec.md) |  | untracked |
| `futures-util` integration — blocked on async strategy decision | [tracing-itertools spec](../plans/done/2026-05-05-tracing-itertools.spec.md) |  | untracked |
| Optional per-subtree `HashMap` index for O(1) scoped `find_by_name_in` — avoids DFS on large trees; requires subtree membership tracking | [object-tree-query spec](../plans/done/2026-05-06-object-tree-query.spec.md) |  | #262 |
| Stale `thread_id` invalidation on object migration — needs object-mobility API | [per-thread-event-loops spec](../plans/done/2026-05-06-per-thread-event-loops.spec.md) |  | #263 |
| Timer driver `start`/`stop` logging — needs investigation of future use patterns | [tracing-spans spec](../plans/done/2026-05-06-tracing-spans.spec.md) |  | #264 |
| Window state persistence (size / position restoration across runs) — needs a settings layer | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | #303 |
| Multi-monitor placement APIs (`set_monitor`, fullscreen-on-display-N) — winit exposes the primitives but no widget-side consumer yet | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | #304 |
| Per-window cursor / IME state plumbing — not requested in issue #53 | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | #305 |
| Pluggable backend for headless / alternative windowing (smithay direct, sdl) — single-backend (winit) per #73 | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | #306 |

## Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Thread migration — if an object moves to another thread after a connection is established, the captured `thread_id` will be stale; this requires a separate object-mobility design | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | | #52 |
| Mutable parent/children manipulation (reparenting). | [parent-children-accessors spec](../plans/done/2026-05-05-parent-children-accessors.spec.md) |  | untracked |
| Timer-wheel implementation for `PoolDriver` (min-heap single thread is sufficient) | [timer-object spec](../plans/done/2026-05-05-timer-object.spec.md) |  | untracked |
| Async timer support | [timer-object spec](../plans/done/2026-05-05-timer-object.spec.md) |  | untracked |
| Explicit `mpsc::Sender` in `start()` (superseded by `TimerDriver`) | [timer-object spec](../plans/done/2026-05-05-timer-object.spec.md) |  | untracked |
| `futures-util` — no async call sites; async/await strategy is an open design question (deferred) | [tracing-itertools spec](../plans/done/2026-05-05-tracing-itertools.spec.md) |  | untracked |
| `clap` / `clap_complete` — no CLI surface in the library | [tracing-itertools spec](../plans/done/2026-05-05-tracing-itertools.spec.md) |  | untracked |
| Changing production iterator patterns to use itertools | [tracing-itertools spec](../plans/done/2026-05-05-tracing-itertools.spec.md) |  | untracked |
| Adding `tracing-subscriber` or any concrete subscriber — that is the application's responsibility | [tracing-itertools spec](../plans/done/2026-05-05-tracing-itertools.spec.md) |  | untracked |
| Tree-level observer/watcher object (use per-object signal and connect on insert) | [object-tree-query spec](../plans/done/2026-05-06-object-tree-query.spec.md) |  | untracked |
| Wiring through `quartzite-events` (UI events crate) for name-change notifications | [object-tree-query spec](../plans/done/2026-05-06-object-tree-query.spec.md) |  | untracked |
| `destroy` does not emit `name_changed` — destruction is a separate concern | [object-tree-query spec](../plans/done/2026-05-06-object-tree-query.spec.md) |  | untracked |
| `Auto` connection improvements beyond what per-thread loops already enable | [per-thread-event-loops spec](../plans/done/2026-05-06-per-thread-event-loops.spec.md) |  | untracked |
| No-std path (event loops require `std`) | [per-thread-event-loops spec](../plans/done/2026-05-06-per-thread-event-loops.spec.md) |  | untracked |
| Pluggable async executors | [per-thread-event-loops spec](../plans/done/2026-05-06-per-thread-event-loops.spec.md) |  | untracked |
| Adding new tracing points beyond signal emit. | [tracing-spans spec](../plans/done/2026-05-06-tracing-spans.spec.md) |  | #265 |
| Other signal.rs calls (`connect`, `disconnect`) — embedded in logic, not announcements; unchanged. | [tracing-spans spec](../plans/done/2026-05-06-tracing-spans.spec.md) |  | untracked |
| Timer driver-level `start`/`stop` in `timer_drivers.rs` — deferred; significance needs future investigation. | [tracing-spans spec](../plans/done/2026-05-06-tracing-spans.spec.md) |  | untracked |
| `info!`/`warn!`/`error!` calls (none exist currently). | [tracing-spans spec](../plans/done/2026-05-06-tracing-spans.spec.md) |  | untracked |
| Modal / parent-child window relationships (no `set_parent` / `is_modal` in this milestone). | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | #307 |
| Multiple winit `EventLoop`s — single winit loop multiplexes all windows, matching how winit itself models multi-window apps. | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | untracked |
| Per-window `Application` instances. The process-singleton `Application` from `quartzite-runtime` remains a singleton; multi-window means many windows, not many `Application`s. | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | untracked |
| Non-winit / off-screen window backends. `RenderHarness` (offscreen) is unaffected and is not "a window". | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | untracked |
| Cross-window focus orchestration policy beyond "the window that received the winit event owns dispatch". Cross-window tab traversal, click-to-focus across windows, and global focus tracking are deferred. | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | #308 |

## Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Should `ConnectionTable` use `DashMap` (lock-free) or `Mutex<HashMap>`? | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | ✅ done | |
| Should the event loop be `async`-runtime-agnostic (pluggable executor) or std-thread-based only? | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | ✅ done | |
| **`QueuedDispatcher` trait location**: Proposed to live in quartzite-core behind `feature = "std"`. Alternative: define it in quartzite-runtime and have core use a raw function pointer or a `OnceLock<fn(Box<dyn FnOnce() + Send>)>`. Needs decision before Task 5. | [runtime design](../plans/done/2026-05-01-runtime.design.md) | ✅ done | |
| **`ObjectFactory` global vs. per-`Application`**: Should there be a process-wide factory singleton (like `ConnectionTable`), or one per `Application`? **Proposed**: per-`Application`, accessible via `Application::factory()`. | [runtime design](../plans/done/2026-05-01-runtime.design.md) | | #61 (closed) |
| **`ThreadPool` shutdown semantics**: Should `ThreadPool::drop` wait for in-flight tasks to complete (graceful) or abandon them? **Proposed**: graceful — close the sender and join all workers. Panic in a worker thread propagates as a resumed panic in `join()`. | [runtime design](../plans/done/2026-05-01-runtime.design.md) | ✅ done | |
| Per-window scale-factor / DPI policy — Winit exposes `scale_factor`; the widget layout system does not yet consume it. Filed against widgets backlog. | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | #309 |
| Window-level keyboard focus model across multiple windows on click-to-focus platforms — Outside the dispatch-routing scope of this milestone; needs a focus-state design that touches `quartzite-widgets`. | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | untracked |
| Whether closed-window `WindowId` values may be re-issued — Winit guarantees uniqueness within a process; design phase confirms and documents. | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | untracked |
| Whether `try_create_window` is sync or async — The current `WindowedApplication::run` is fully sync; design phase confirms (default: sync; winit `Window` creation is sync inside `ApplicationHandler::resumed`). | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | untracked |
| Exact handle shape exposed to user callbacks for calling `try_create_window` mid-loop — The design chose `WindowRegistry` threaded through `WindowedAppHandler` callbacks via `&mut`. | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | untracked |
| Whether the existing `WindowedApplication::new()` constructor is retained as a shorthand for `builder().build()` or removed — Sugar question; design picks. Both options satisfy AC7. | [multi-window-support spec](../plans/done/2026-05-11-multi-window-support.spec.md) |  | untracked |
| Whether `WidgetRoot` should be folded into a closure adaptor instead of a named trait. Design picks: named trait for ergonomics. | [multi-window-support design](../plans/done/2026-05-11-multi-window-support.design.md) |  | untracked |
| Whether `on_last_window_closed` is even useful in this milestone. Design picks: include it for AC4b test clarity. | [multi-window-support design](../plans/done/2026-05-11-multi-window-support.design.md) |  | untracked |
| Whether `try_create_window` should accept window-level configuration (title, initial size, decorated/undecorated). Spec is silent; future spec adds a `WindowAttributes` arg. | [multi-window-support design](../plans/done/2026-05-11-multi-window-support.design.md) |  | #310 |
| `WindowRegistry: !Send + !Sync` enforcement vs. ergonomics — blocks future cross-thread `try_create_window`; escape hatch is `EventLoopProxy<AppEvent>`. | [multi-window-support design](../plans/done/2026-05-11-multi-window-support.design.md) |  | untracked |
