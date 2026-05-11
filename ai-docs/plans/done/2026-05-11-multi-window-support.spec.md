# Multi-window support

**Source:** issue #53
**Date:** 2026-05-11
**Tracked in:** #53

## Scope

1. Multi-window registry + creation API on `quartzite_renderer::WindowedApplication`: a callable `try_create_window(...) -> Result<WindowId, _>` plus a `windows()` accessor that lists currently-live windows. `quartzite-runtime` stays headless (no `winit` / `wgpu` dependency added).
2. Window lifecycle wired into the winit event loop already owned by `WindowedApplication`:
   - `WindowEvent::CloseRequested` removes the window from the registry and drops its `wgpu::Surface` + winit `Window`.
   - Last-window-quit policy: builder-configurable via `.quit_on_last_window_closed(bool)`, **default `true`** (event loop exits when the last live window closes). Setting `false` keeps the loop running so the user code can re-create windows later or shut down on a different signal.
3. Per-window event routing: winit events arriving for a window are dispatched to that window's widget root via the existing `WidgetExt` hooks (`paint`, `on_resize`, mouse / key callbacks). Within a window, the widget tree's existing hit-testing decides which widget receives the event — multi-window scope is strictly "which window did this event arrive at, and which root receives it".
4. Tests at the `quartzite-renderer` crate covering: open ≥ 2 windows; close a non-first window (other windows survive, app keeps running); close the last window with `quit_on_last_window_closed = true` (loop exits) and with `quit_on_last_window_closed = false` (loop stays running); per-window event routing dispatches scoped events to the correct widget root.

## Out of scope

- Modal / parent-child window relationships (no `set_parent` / `is_modal` in this milestone).
- Multiple winit `EventLoop`s — single winit loop multiplexes all windows, matching how winit itself models multi-window apps.
- Per-window `Application` instances. The process-singleton `Application` from `quartzite-runtime` remains a singleton; multi-window means many windows, not many `Application`s.
- Non-winit / off-screen window backends. `RenderHarness` (offscreen) is unaffected and is not "a window".
- Cross-window focus orchestration policy beyond "the window that received the winit event owns dispatch". Cross-window tab traversal, click-to-focus across windows, and global focus tracking are deferred.
- Window-level menu bars, dialogs, dock widgets — listed under `quartzite-widgets` v2 backlog (issue #46 carry-over).

## Deferred

- Window state persistence (size / position restoration across runs) | needs a settings layer | separate issue needed: yes (file under `widget-backlog.md` if not already)
- Multi-monitor placement APIs (`set_monitor`, fullscreen-on-display-N) | winit exposes the primitives but no widget-side consumer yet | separate issue needed: yes
- Per-window cursor / IME state plumbing | not requested in issue #53 | separate issue needed: yes
- Pluggable backend for headless / alternative windowing (smithay direct, sdl) | single-backend (winit) per #73 | covered by existing pluggable-backend deferral

## Key decisions

| Question | Decision |
|---|---|
| Q1: Where do `try_create_window(...)` and `windows()` live? | **`quartzite_renderer::WindowedApplication`** (round 1 answer). `quartzite-runtime` stays graphics-free; no `winit` / `wgpu` dependency added there. |
| Q2: Default policy when the last window closes? | **Configurable, default exit** (round 1 answer). Builder method `WindowedApplication::builder().quit_on_last_window_closed(bool)` with default `true`. `false` keeps the event loop alive after the last close. |
| API naming for the fallible creator | `try_create_window` returning `Result<WindowId, _>` (AGENTS.md § *API Naming*: non-panicking default; no panicking `create_window` sibling unless explicitly requested). |
| API naming for `_unchecked` | Not applicable — no `unsafe` fns in this scope. |
| `WindowId` type | Newtype around `winit::window::WindowId` (opaque to callers). Design phase picks the concrete shape (newtype struct vs. `pub use` re-export). |
| Primary-window concept | None — all windows are peers. The previous "primary or none" model is replaced by an unordered registry. First-created has no semantic privilege. |
| Window-to-widget-root binding | `try_create_window` accepts (or returns a handle that accepts) a widget root; per-window dispatch routes to that root. Concrete signature picked in design (builder vs. args). |
| Builder-vs-constructor shape | `WindowedApplication` gains a builder (`WindowedApplication::builder() -> WindowedApplicationBuilder`) carrying the `quit_on_last_window_closed` flag and any future window-app-level options. The existing `WindowedApplication::new()` either becomes shorthand for `builder().build()` or is replaced by the builder entirely — AGENTS.md § *API Stability* permits the break. Concrete shape (retain `new()` vs. drop it; chained vs. final-`build()`) picked in design. |
| Event-loop ownership | One winit `EventLoop` (the existing `WindowedApplication`-owned loop). Per-window registries multiplex inside it. |
| Compat shims | None. `WindowedApplication`'s current single-window pipeline is freely refactored — AGENTS.md § *API Stability* (pre-publish, no downstream clients). |
| Tracking-issue posture | Issue #53's "Blocked by" list (#73, #46) is now stale — both are closed / merged. The spec is unblocked; the issue body is not rewritten (AGENTS.md). A scope-change comment on #53 may be posted from the implementation PR. |

## Technical constraints

- `WindowedApplication` already owns `winit::event_loop::EventLoop<()>` and exposes `run_app(handler: impl ApplicationHandler)`. The multi-window registry must live behind a handler that wraps the user's `ApplicationHandler`, so user code keeps the existing entry point shape.
- `quartzite-runtime` must not gain a `winit` / `wgpu` dependency unless Q1 resolves to "runtime owns the API"; even then, the API surface must remain feature-gated to preserve the headless build (`quartzite-runtime` is the headless CLI / daemon / test entry per #73's design).
- `Application` is a process singleton (`OnceLock` in `quartzite-runtime/src/application.rs`). Multi-window does **not** lift that constraint.
- Window dispatch closures cross into the runtime's main-thread event loop via `Application::post_event`; pre-existing thread-affinity rules (AGENTS.md § *Per-thread event loops* via #51-derived design) apply.
- `WindowEvent::CloseRequested` is the close hook (not `Destroyed`); resource teardown (`wgpu::Surface` drop, vello painter scene drop) happens deterministically before the registry entry is removed.
- The `quartzite-widgets` `WidgetExt` hooks (`on_show`, `on_hide`, `on_resize`, mouse / key) are the routing targets — no new trait methods are added in this milestone.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `WindowedApplication::try_create_window(...) -> Result<WindowId, _>` exists; calling it ≥ 2 times within a single `WindowedApplication::run` registers ≥ 2 live windows. |
| AC2 | `WindowedApplication::windows()` returns the current set of live `WindowId`s; the order is unspecified but stable across one `run` invocation. |
| AC3 | Closing a non-last window via `WindowEvent::CloseRequested` removes that window from the registry, drops its `wgpu::Surface`, and does **not** stop the event loop; remaining windows continue receiving events. |
| AC4a | Default behaviour: when no builder override is set, closing the last open window exits the event loop. A test constructs a `WindowedApplication` via the default builder, opens one window, closes it, and asserts `run_app` returns `Ok(())`. |
| AC4b | Opt-out behaviour: when `WindowedApplication::builder().quit_on_last_window_closed(false).build()` is used, closing the last open window leaves the event loop running; the test then issues an explicit exit signal (e.g. winit `EventLoopProxy` or `Application::quit`) and asserts the loop exits cleanly. |
| AC5 | A winit event scoped to a specific window (`WindowEvent { window_id, .. }`) is dispatched to that window's widget root and not to other windows' roots. |
| AC6 | `cargo test -p quartzite-renderer` covers AC1–AC5. Tests requiring a display follow the existing `xvfb` gating pattern in `quartzite-renderer/tests/xvfb_smoke.rs`. |
| AC7 | `WindowedApplication::builder()` exists, returns a builder carrying at least the `quit_on_last_window_closed` flag, and produces a `WindowedApplication` via `.build()`. Builder is `#[must_use]` and documented. |
| AC8 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` and `cargo clippy --workspace -- -D warnings` pass with the new API surface. |
| AC9 | `cargo build -p quartzite --no-default-features` still builds (no_std / derive-free path unaffected). `quartzite-runtime`'s dependency graph is unchanged (no winit / wgpu added). |

## Open questions

| Item | Why deferred |
|---|---|
| Per-window scale-factor / DPI policy | Winit exposes `scale_factor`; the widget layout system does not yet consume it. Filed against widgets backlog. |
| Window-level keyboard focus model across multiple windows on click-to-focus platforms | Outside the dispatch-routing scope of this milestone; needs a focus-state design that touches `quartzite-widgets`. |
| Whether closed-window `WindowId` values may be re-issued | Winit guarantees uniqueness within a process; design phase confirms and documents. |
| Whether `try_create_window` is sync or async | The current `WindowedApplication::run` is fully sync; design phase confirms (default: sync; winit `Window` creation is sync inside `ApplicationHandler::resumed`). |
| Exact handle shape exposed to user callbacks for calling `try_create_window` mid-loop | Winit's `ApplicationHandler::resumed`/event callbacks receive `&ActiveEventLoop`; the spec leaves it to the design phase whether `WindowedApplication` exposes a shared `WindowRegistry` handle, threads it through the user's handler, or wraps the user handler. All three satisfy the ACs. |
| Whether the existing `WindowedApplication::new()` constructor is retained as a shorthand for `builder().build()` or removed | Sugar question; design picks. Both options satisfy AC7. |
