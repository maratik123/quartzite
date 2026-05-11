# Design: Multi-window support

**Issue:** #53
**Date:** 2026-05-11
**Spec:** [`2026-05-11-multi-window-support.spec.md`](2026-05-11-multi-window-support.spec.md)

## Approach

The current `quartzite_renderer::WindowedApplication` is a 90-line wrapper that
owns `Application` + `winit::EventLoop<()>` and forwards `run(handler)` to
`event_loop.run_app(handler)`. There is no window-creation API anywhere in
the crate, no per-window state, and no `wgpu::Surface` instantiation in real
code (`VelloPainter` is a skeleton; `RenderHarness` uses an offscreen texture).
This design adds the multi-window scaffold inside `quartzite-renderer` —
`quartzite-runtime` is not touched (key decision Q1).

### Shape of the user-facing API

The user supplies a `WindowedAppHandler` (a new, narrow trait owned by this
crate) instead of a raw `winit::ApplicationHandler`. The trait gives the user
hooks to plug into the multi-window lifecycle without re-implementing the
window/event dispatch fan-out. The internal `WrappedHandler` struct
implements `winit::ApplicationHandler` for the framework, owns the window
registry, and invokes the user's hooks at the right moments.

```text
WindowedApplication::run(user_handler: impl WindowedAppHandler)
└─ winit::EventLoop::run_app(&mut WrappedHandler { user_handler, registry, … })
   ├─ resumed(&ActiveEventLoop)       → user_handler.on_start(&mut WindowRegistry)
   │                                    (user calls registry.try_create_window(...) here)
   ├─ window_event(window_id, evt)    → look up registry entry; dispatch to widget root
   │   └─ CloseRequested              → drop surface + winit Window; if registry empty & quit_on_last_window_closed: event_loop.exit()
   └─ user_event / about_to_wait / …  → forwarded to user_handler with &mut WindowRegistry
```

The user never implements `winit::ApplicationHandler` directly. This is a
strict break from the pre-spec public API (`pub fn run(impl ApplicationHandler)`),
which AGENTS.md § *API Stability* permits (pre-publish, no downstream clients,
clean break, no compat shim).

### `WindowRegistry` is the multiplex point

A `WindowRegistry` value is threaded through the user's hooks (passed by
`&mut`). The public type has **no lifetime parameter** — it is owned by the
`WrappedHandler` for the entire `run_app` lifetime and the
`&ActiveEventLoop` borrow is held via an **erased raw-pointer slot**
(Option B from review feedback). The lifetime-parameterised alternative
(`WindowRegistry<'a>`) was rejected because it would propagate `'a` through
every user-facing signature (`WindowedAppHandler::on_start(&mut self, registry: &mut WindowRegistry<'a>)`,
the trait declaration itself, every `WindowEntry`/closure that touches the
registry) without buying real safety — `WindowRegistry: !Send + !Sync`
already prevents the only realistic misuse path (handing the registry to
another thread that outlives the callback).

`WindowRegistry` owns:

- `windows: IndexMap<winit::window::WindowId, WindowEntry>` — registry storage.
- `quit_on_last_window_closed: bool` — copied from the builder at `WrappedHandler` construction.
- `active_loop: Cell<*const ActiveEventLoop>` — transient slot for the
  `&ActiveEventLoop` available during a `winit::ApplicationHandler` callback.
  Set to a non-null pointer by `WrappedHandler` before dispatching into a
  user callback and cleared (set to `ptr::null()`) immediately on return.
- `_not_send_sync: PhantomData<*const ()>` — makes `WindowRegistry` neither
  `Send` nor `Sync`. The raw-pointer field already auto-implies `!Send +
  !Sync`, but a named marker keeps the intent visible to readers and
  survives accidental refactors.

The slot is read by `try_create_window` only:

```rust
impl WindowRegistry {
    /// Creates a new window and registers it with this registry.
    ///
    /// # Errors
    ///
    /// Returns `RendererError::OutsideCallback` when called outside an
    /// `ApplicationHandler` callback (the internal `&ActiveEventLoop` slot
    /// is null), or `RendererError::Winit(e)` when winit's
    /// `ActiveEventLoop::create_window` fails.
    pub fn try_create_window(
        &mut self,
        root: impl WidgetRoot,
    ) -> Result<WindowId, RendererError> {
        let ptr = self.active_loop.get();
        if ptr.is_null() {
            return Err(RendererError::OutsideCallback);
        }
        // SAFETY: see invariants on `WindowRegistry::active_loop`.
        let event_loop = unsafe { &*ptr };
        // ... call event_loop.create_window(...), build WindowEntry, insert.
    }
}
```

#### `WindowRegistry::active_loop` — `# Safety` invariants

The raw-pointer slot is the only `unsafe` surface introduced by this design
and is governed by these documented invariants (transferred verbatim into
the rustdoc on the slot field and on `try_create_window`):

1. **Set-clear bracket.** `WrappedHandler` is the only writer. Before any
   `winit::ApplicationHandler` callback that hands `&mut WindowRegistry` to
   user code, the handler executes:

   ```rust
   self.registry.active_loop.set(event_loop as *const ActiveEventLoop);
   let _guard = ActiveLoopGuard(&self.registry); // resets to null on drop, panic-safe
   // ... invoke user callback with &mut self.registry ...
   ```

   The `ActiveLoopGuard` resets the slot to `ptr::null()` on `Drop`,
   guaranteeing the slot is cleared even when the user callback panics.

2. **Single-threaded.** `WindowRegistry: !Send + !Sync` is enforced by
   `PhantomData<*const ()>`, so the registry never crosses a thread
   boundary; the pointed-to `ActiveEventLoop` lives on the same thread and
   is valid for the entire callback duration. The slot can never be read on
   a thread other than the one that set it.

3. **Read-only inside `try_create_window`.** The `unsafe { &*ptr }`
   conversion is performed only after a non-null check; the resulting
   shared reference lives only for the body of `try_create_window` and is
   never stored beyond that body. No aliasing rules are violated: winit's
   own callback contract gives us exclusive access to `&ActiveEventLoop`
   for the duration of the callback.

4. **Never null when read.** A null slot triggers
   `Err(RendererError::OutsideCallback)` — there is no path that
   dereferences a null pointer.

A unit test exercises invariant (4) by constructing a `WindowRegistry`
outside any callback and asserting `try_create_window` returns
`OutsideCallback`. Invariant (1)'s panic-safety is exercised by a test that
panics inside a user callback and asserts the slot is null after unwind
(catch_unwind + drop check).

A `WindowEntry` owns:

- `window: Arc<winit::window::Window>` (winit gives `Arc<Window>` from
  `ActiveEventLoop::create_window`; `Arc` lets us share the raw handle into
  `wgpu::Surface` and any future per-window painter).
- `surface: wgpu::Surface<'static>` (created with `unsafe { instance.create_surface(window.clone()) }` —
  the `'static` lifetime is achieved by holding the `Arc<Window>` alongside).
  Note: with the `wgpu 28` API surface, `create_surface` accepts a
  `'static`-bounded raw window handle, so no `unsafe` block is required at
  the call site — verified at implementation time.
- `root: Box<dyn WidgetRoot>` (the widget root supplied by the caller —
  see "Widget root abstraction" below).

### Widget root abstraction

The renderer crate cannot take a `WidgetExt` trait bound: `quartzite-widgets`
depends on `quartzite-renderer`-adjacent types (it's downstream in the dep
graph). The same constraint already drove `RenderHarness::render_widget` to
take a closure (see `render_harness.rs` line 195–211 comment).

This design follows the same pattern. The renderer defines a tiny local
trait:

```rust
pub trait WidgetRoot: 'static {
    fn paint(&self, painter: &mut dyn Painter);
    fn on_resize(&mut self, size: Size);
    fn on_mouse_press(&mut self, event: &MouseEvent);
    fn on_mouse_release(&mut self, event: &MouseEvent);
    fn on_key_press(&mut self, event: &KeyEvent);
    fn on_key_release(&mut self, event: &KeyEvent);
}
```

The `paint` receiver is `&self` to match `WidgetExt::paint(&self, …)` in
`quartzite-widgets`. Rationale: state mutation belongs to the widget's own
lifecycle hooks (`on_resize`, `on_mouse_press`, etc., which already take
`&mut self`); `paint` is a read-only projection of the widget's current
state onto the `Painter`. The dispatch loop holds `&mut WindowEntry` for
the registry slot but reborrows immutably when calling `paint`, matching
the trait's contract. (If a future widget genuinely needs interior
mutability during paint — for caching tessellated paths, etc. — the
canonical Rust answer is `Cell` / `RefCell` inside the widget, not
escalating the trait receiver to `&mut self`.)

(Exact method set matches the spec's "existing `WidgetExt` hooks" list.) The
trait lives in `quartzite-renderer::window_root`. Test code and downstream
widget callers wrap their `WidgetExt` type in a closure-based adaptor — the
adaptor lives in `quartzite-widgets/tests/support/mod.rs` (test-side) and a
documented pattern lives next to the trait. No paths cross.

`try_create_window` accepts `impl WidgetRoot` (or `Box<dyn WidgetRoot>`); the
registry stores it as `Box<dyn WidgetRoot>`.

### Rejected alternatives

1. **Add a window-creation method directly on `Application`.** Rejected by
   spec key decision Q1 — `quartzite-runtime` must not gain a winit / wgpu
   dependency.
2. **Expose raw `winit::ApplicationHandler` and let the user manage the
   registry.** Rejected because every user would re-implement the same
   fan-out logic; spec AC5 (per-window dispatch) is the whole point of this
   milestone.
3. **`try_create_window` as a method on `WindowedApplication` (not on a
   registry handed to the user in callbacks).** Rejected because winit
   requires `&ActiveEventLoop` to create windows, and that reference only
   exists inside `ApplicationHandler` callbacks. The registry-in-callback
   pattern is the only one that gives us the `ActiveEventLoop` cleanly.
4. **One `WindowedApplication` per window.** Rejected by spec — single
   `Application` singleton, single winit `EventLoop`.
5. **Use winit's `EventLoopProxy` for cross-thread `try_create_window`.**
   Out of scope for this milestone (open question Q "sync or async" in spec
   resolves: sync). A proxy-based async create is a strict superset that can
   be added later without breaking AC1–AC9.

### Builder shape

`WindowedApplicationBuilder` is a small struct with one field today
(`quit_on_last_window_closed: bool`, defaulting to `true`). It exists primarily
to satisfy AC7 and to give future window-app-level options a stable extension
point. `WindowedApplication::builder()` returns it; `.build()` consumes it and
constructs the `WindowedApplication`. The existing `WindowedApplication::new()`
is **removed** (pre-publish, AGENTS.md § *API Stability*: clean break, no compat
shim) — callers migrate to `WindowedApplication::builder().build()`. This
satisfies spec key decision row "Whether the existing `WindowedApplication::new()`
is retained" (design picks).

`build()` returns `Result<WindowedApplication, RendererError>` — same fail
modes as today's `new()` (singleton already taken; winit `EventLoop::new()`
failed). The builder itself never fails — `build()` is where errors surface.

The builder type is `#[must_use]` per AC7.

### `WindowId` newtype

A wrapper `pub struct WindowId(winit::window::WindowId)` lives in
`quartzite-renderer::window_id`. Rationale:

- Keeps `winit` from leaking into `quartzite-renderer`'s public type
  signatures (`windows() -> Vec<WindowId>` returns the newtype).
- Allows future opaqueness changes (e.g. internal index assignment) without
  another API break.
- Derives `Debug`, `Clone`, `Copy`, `Eq`, `Hash`, `PartialEq` — matches
  winit's own derives on `winit::window::WindowId`.
- Implements `From<winit::window::WindowId>` and `as_winit(&self) -> winit::window::WindowId`
  internally for the dispatch loop; both are `pub(crate)` to keep the wrapper
  opaque to callers.

Reissuing closed-window IDs: winit documents `WindowId` as opaque and unique
within a process lifetime. The design documents this on the newtype's
rustdoc (spec open-question item).

### Event dispatch

`WrappedHandler::window_event(_, window_id, event)` looks up
`registry.windows.get_mut(&window_id)` and dispatches:

| winit event variant | dispatch action |
|---|---|
| `CloseRequested` | drop the entry; check last-window-quit |
| `Resized(PhysicalSize { width, height })` | convert `u32 → i32` via saturating `i32::try_from(width).unwrap_or(i32::MAX)` (and the same for `height`) in `event_convert::size_from_physical`; call `root.on_resize(Size::new(w, h))`. The conversion is saturating rather than `as` truncation because `as u32 → i32` flips into negative values for displays larger than ~2.1 G pixels per axis; `Size::new` documents `width >= 0` as a soft contract so saturation preserves it. Document the conversion choice next to the function. |
| `RedrawRequested` | call `root.paint(&mut painter)` against `&root` (receiver is `&self`) — painter is a `VelloPainter` instance (skeleton today; the redraw path lights up when `VelloPainter` is filled in). The widget-tree `paint` call is the existing seam; nothing in this milestone wires the wgpu surface present path (that's a separate spec). |
| `MouseInput { state, button, .. }` | call `on_mouse_press` / `on_mouse_release` with a `MouseEvent` constructed from the winit event |
| `KeyboardInput { event: KeyEvent { logical_key, state, .. }, .. }` | call `on_key_press` / `on_key_release` |
| anything else | no-op (logged at `trace`) |

If `window_id` is not in the registry (race between close and a pending
event), the event is dropped silently. This is the documented winit pattern.

### Last-window-quit policy

After every `CloseRequested` dispatch:

```text
if registry.is_empty() && quit_on_last_window_closed:
    active_event_loop.exit()
```

`event_loop.exit()` causes `run_app` to return `Ok(())`. With
`quit_on_last_window_closed = false`, the loop continues running an empty
registry until the user issues an explicit exit (e.g. `Application::quit`
posting an event that calls `event_loop.exit()` via a captured proxy, or a
future direct exit channel). The test suite uses winit's `EventLoopProxy`
+ a custom user event for the opt-out test (AC4b).

To support the proxy path, the design changes `EventLoop<()>` to
`EventLoop<AppEvent>` where:

```rust
enum AppEvent {
    Exit,
    // room for future cross-thread requests
}
```

`WindowedApplication` keeps a `proxy: EventLoopProxy<AppEvent>` accessible via
`WindowedApplication::event_proxy()`. The wrapped handler's `user_event` arm
matches `AppEvent::Exit → active_event_loop.exit()`. The AC4b test sends
`AppEvent::Exit` from inside the `on_start` callback (which fires from the
main thread inside `resumed`), proving the loop exits when explicitly told.

### `windows()` accessor

`WindowedApplication::windows()` does **not** exist — the `WindowedApplication`
value is consumed by `run`, so `windows()` returning a snapshot post-`run` is
meaningless. Instead, the registry's `windows()` accessor is the public
surface, callable from the user's `WindowedAppHandler` callbacks:

```rust
impl WindowRegistry {
    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ { … }
}
```

Returns an unspecified-order iterator over current `WindowId`s — backed by a
`HashMap::keys()` clone, which is stable within one `run` invocation per
spec AC2 ("order unspecified but stable across one `run`"). The "stable
across one `run`" wording is slightly stretched here — the HashMap iteration
order is *not* guaranteed to be stable across calls to `windows()` within
one `run`. The design resolves this by switching the storage to
`indexmap::IndexMap` so insertion-order iteration holds within a `run` (and
matches the spec wording). `indexmap` is already in the transitive Cargo.lock
(brought in by `wgpu`); no new top-level dependency.

### `WindowedAppHandler` trait

```rust
pub trait WindowedAppHandler {
    /// Called once the event loop is ready to create windows. Equivalent to
    /// `ApplicationHandler::resumed`. Most users create their initial windows here.
    fn on_start(&mut self, registry: &mut WindowRegistry);

    /// Called whenever the registry becomes empty.
    /// Default no-op. Useful for opt-out callers (`quit_on_last_window_closed = false`).
    fn on_last_window_closed(&mut self, _registry: &mut WindowRegistry) {}
}
```

Intentionally narrow: the multi-window scaffold's job is to route winit
events into per-window widget roots. User code that needs richer hooks
(e.g. `about_to_wait` for animation) can be added behind a v2 method with
a default no-op.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `quartzite-events = { path = "../quartzite-events" }` to `quartzite-renderer/Cargo.toml` `[dependencies]` (no cycle: `quartzite-widgets` depends on `quartzite-renderer` only as a `dev-dependency`). Introduce `WindowId` newtype + `WidgetRoot` trait + `WindowRegistry` skeleton (no winit wiring yet — types only, with rustdoc, no behaviour). | `quartzite-renderer/Cargo.toml`, `quartzite-renderer/src/window_id.rs` (new), `quartzite-renderer/src/window_root.rs` (new), `quartzite-renderer/src/window_registry.rs` (new), `quartzite-renderer/src/lib.rs` | — |
| 2 | Introduce `WindowedApplicationBuilder` + `AppEvent`; refactor `WindowedApplication` to construct via `builder().build()`; remove `WindowedApplication::new()`; expose `event_proxy()`. Update `tests/application.rs` to use the builder. | `quartzite-renderer/src/application.rs`, `quartzite-renderer/src/application_builder.rs` (new), `quartzite-renderer/tests/application.rs` | 1 |
| 3 | Implement `WrappedHandler` + integrate `WindowRegistry::try_create_window` using `&ActiveEventLoop` (transient slot pattern); wire `WindowEvent::CloseRequested` teardown and last-window-quit policy. | `quartzite-renderer/src/wrapped_handler.rs` (new), `quartzite-renderer/src/window_registry.rs`, `quartzite-renderer/src/application.rs` | 2 |
| 4 | Implement per-window event dispatch (`Resized`, `RedrawRequested`, `MouseInput`, `KeyboardInput`) routing to `WidgetRoot` methods; construct `quartzite_events::MouseEvent` / `KeyEvent` from winit equivalents. | `quartzite-renderer/src/wrapped_handler.rs`, `quartzite-renderer/src/event_convert.rs` (new) | 3 |
| 5 | Add `WindowedAppHandler` trait, change `WindowedApplication::run` signature to `run(self, handler: impl WindowedAppHandler) -> Result<(), RendererError>`. Update `xvfb_smoke.rs` to either skip or migrate (the test does not need multi-window — it boots an `EventLoop` directly, so it's largely untouched; the `WindowedApplication::run`-using path becomes the new shape but the existing test bypasses `WindowedApplication`). | `quartzite-renderer/src/application.rs`, `quartzite-renderer/src/windowed_app_handler.rs` (new), `quartzite-renderer/src/lib.rs` | 4 |
| 6 | Integration tests: AC1–AC5 covered via `quartzite-renderer/tests/multi_window.rs` under the `xvfb` gating pattern (Linux + `with_any_thread`). AC4a (default-quit), AC4b (proxy-driven exit), AC3 (close non-last), AC1+AC2 (open ≥ 2 + `windows()` accessor), AC5 (per-window routing — test uses two roots that record `(WindowId, event)` pairs into a shared `Arc<Mutex<Vec<_>>>`). Shared test helpers live under `tests/support/mod.rs` (subdirectory module, **not** a sibling `tests/support.rs` — Cargo would compile a sibling file as its own integration-test binary). Each integration test file declares `mod support;` at the top and refers to helpers via `support::…`. | `quartzite-renderer/tests/multi_window.rs` (new), `quartzite-renderer/tests/support/mod.rs` (new — shared test helpers) | 5 |
| 7 | Doc gate + clippy gate: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo clippy --workspace -- -D warnings`, `cargo build -p quartzite --no-default-features`. Fix doc + clippy violations; add `# Examples` blocks to every new public item per AGENTS.md. | all touched files in 1–5 | 6 |

7 tasks — at the spec's "split if > 7" threshold; the decomposition stays
together as one PR because tasks 1–5 are tightly coupled (each one only
compiles after the prior task lands).

## Risks

- **`indexmap` dependency direct vs. transitive.** Adding it as a direct
  Cargo.toml dependency makes `windows()` ordering an explicit contract. If
  upstream `wgpu` drops `indexmap` later, the renderer keeps its own version.
  *Mitigation:* add as a direct dep with `0.x` pin per AGENTS.md *Dependency
  Versions*; verify max stable via `curl -sS https://crates.io/api/v1/crates/indexmap | jq -r '.crate.max_stable_version'` at implementation time.
- **`unsafe` surface for `wgpu::Surface` creation.** wgpu 28's
  `Instance::create_surface(window)` is safe given a `'static`-bounded
  `Arc<Window>` (winit 0.30 returns `Arc<Window>` from
  `ActiveEventLoop::create_window`). *Mitigation:* implementation-time check
  — if the API requires `unsafe`, add `# Safety` per AGENTS.md and document
  the `'static` lifetime via the held `Arc`.
- **`WindowedApplication::run` signature break.** The existing public
  `run(impl ApplicationHandler)` is replaced with `run(impl WindowedAppHandler)`.
  *Mitigation:* pre-publish, AGENTS.md § *API Stability* permits the clean
  break; no compat shim. Test in `tests/application.rs` migrates.
- **xvfb test flakiness.** Multi-window integration tests touch the full
  winit + display path; xvfb does not honour all winit features uniformly.
  *Mitigation:* use the existing `xvfb_smoke.rs` gating pattern
  (`with_any_thread` X11 + Wayland extensions; `SKIP_RENDER_SNAPSHOT`
  bail-out for environments without display). Tests assert clean exit, not
  pixel content.
- **Widget root paint without surface.** `VelloPainter` is a no-op
  skeleton; calls to `root.paint(&mut painter)` produce no pixels today.
  *Mitigation:* the AC5 test asserts on dispatch *occurrence*
  (`Arc<Mutex<Vec<…>>>` records the call), not on rendered output. Real
  surface presentation lights up in a separate future spec.
- **Event-loop user event type change.** Switching `EventLoop<()>` to
  `EventLoop<AppEvent>` is observable in handler signatures (winit
  generic). *Mitigation:* the user's handler is now `WindowedAppHandler`,
  not `winit::ApplicationHandler` — the user never sees the
  `EventLoop<AppEvent>` generic.
- **Registry borrowing in `try_create_window`.** The `&ActiveEventLoop`
  borrow needs to be valid only inside callbacks; storing it in a transient
  registry slot risks misuse if a callback hands the registry off
  cross-thread. *Mitigation:* `WindowRegistry: !Send + !Sync` (raw-pointer
  field + `PhantomData<*const ()>`); the slot is
  `Cell<*const ActiveEventLoop>` set to a non-null pointer before each user
  callback and cleared to `ptr::null()` on return via an
  `ActiveLoopGuard` (Drop-based, panic-safe); `try_create_window` returns
  `Err(RendererError::OutsideCallback)` when the slot is null. The
  `unsafe { &*ptr }` deref carries a `# Safety` block citing the four
  invariants in the *`WindowRegistry::active_loop` — `# Safety` invariants*
  section (set-clear bracket, single-threaded, read-only inside
  `try_create_window`, never null when read). Unit tests exercise the null
  branch and the panic-safety of the guard's `Drop`.
- **Per-window `wgpu::Surface` lifetime tied to `Arc<Window>`.** Surface
  must drop *before* the window. *Mitigation:* `Drop` order on
  `WindowEntry` — `surface: wgpu::Surface<'static>` listed before
  `window: Arc<Window>` so struct-field drop order matches the constraint.
  Document and unit-test (Drop instrumentation in a test fixture).

## Test Design

### Task 1 — `WindowId`, `WidgetRoot`, `WindowRegistry` skeleton

- Location: `quartzite-renderer/src/window_id.rs` `#[cfg(test)]` module.
- Entry points: `WindowId::from(winit::window::WindowId)`, derived traits.
- Scenarios: Hash + Eq roundtrip; Debug renders an opaque token; Copy works.
- Fixtures: none — winit `WindowId` is constructable via winit test API
  (or skipped if not — derive tests are enough).

### Task 2 — Builder + AppEvent

- Location: `quartzite-renderer/src/application_builder.rs` `#[cfg(test)]`
  module; integration sanity in `tests/application.rs`.
- Entry points: `WindowedApplicationBuilder::default()`,
  `WindowedApplicationBuilder::quit_on_last_window_closed(bool)`,
  `.build()`.
- Scenarios:
  - Default builder has `quit_on_last_window_closed = true`.
  - `quit_on_last_window_closed(false)` sets the flag.
  - `.build()` consumed builder; type is `#[must_use]`.
  - `.build()` returns `Err(RendererError::Application(AlreadyExists))` when
    `Application` singleton is taken (mirror of existing test pattern).
- Fixtures: existing `tests/application.rs` pattern.

### Task 3 — Wrapped handler + registry + close + last-window-quit

- Location: `quartzite-renderer/src/wrapped_handler.rs` `#[cfg(test)]`
  for pure-logic unit tests; integration in `tests/multi_window.rs`.
- Entry points: `WrappedHandler::window_event(_, _, CloseRequested)`,
  `WindowRegistry::try_create_window`, registry `is_empty()` quit check.
- Scenarios:
  - Pure-logic test: a fake registry with two entries, drop one — registry
    still has one entry, `is_empty() == false`.
  - Pure-logic test: drop last entry with `quit_on_last_window_closed = true`
    → "exit requested" flag set on a test double of `ActiveEventLoop`
    (use a small `enum DispatchSink { Real(&'a ActiveEventLoop), Test(Vec<…>) }`
    or factor the side-effect into a `fn request_exit` injection).
  - Integration: AC3 — open 2, close 1, assert second one persists, loop
    keeps running until explicit exit.
- Fixtures: `tests/support/mod.rs` exposes a `RecordingRoot` impl of
  `WidgetRoot` that records `(WindowId, event_kind)` into
  `Arc<Mutex<Vec<_>>>`. Each test file pulls it in with `mod support;`.

### Task 4 — Per-window event dispatch

- Location: `quartzite-renderer/src/wrapped_handler.rs` `#[cfg(test)]`
  module; integration in `tests/multi_window.rs`.
- Entry points: `WrappedHandler::window_event` for each event variant;
  `event_convert::mouse_event_from_winit`, `key_event_from_winit`.
- Scenarios:
  - `Resized(800, 600)` for `WindowId(A)` → root A receives `on_resize(Size{800,600})`; root B does not.
  - `MouseInput { state: Pressed, button: Left }` → root receives `on_mouse_press` with kind = press, button = left.
  - `MouseInput { state: Released, button: Right }` → `on_mouse_release` with right.
  - `KeyboardInput { state: Pressed, … }` → `on_key_press`.
  - `RedrawRequested` → `paint(&self, …)` called with a `&mut dyn Painter`; the test verifies the recording root saw the call (the `&self` receiver works through a `Cell<bool>` painted-flag inside `RecordingRoot`).
  - Event for an unknown `WindowId` (already-closed) — silently dropped, no panic.
- Fixtures: `RecordingRoot` (above).

### Task 5 — `WindowedAppHandler` trait + `run` signature

- Location: doctest in `application.rs`; integration in
  `tests/multi_window.rs`.
- Entry points: `WindowedApplication::run(impl WindowedAppHandler)`.
- Scenarios: doctest constructs a `WindowedApplication` via builder and
  shows the new run signature (no_run / no actual display needed).

### Task 6 — Acceptance criteria coverage

`quartzite-renderer/tests/multi_window.rs` — one Linux-only `#[cfg]` block,
mirroring `xvfb_smoke.rs` style:

- `test_open_two_windows_close_one_keeps_running` (AC1 + AC3): handler
  creates two windows in `on_start`, posts `CloseRequested` for one via
  `EventLoopProxy` indirection (or sends a synthetic event), asserts the
  remaining window is still in `registry.windows()` and the loop has not
  exited; then explicit-exit via `AppEvent::Exit`.
- `test_default_quit_on_last_close` (AC4a): default builder, one window,
  close → `run` returns `Ok(())`.
- `test_opt_out_keeps_loop_alive` (AC4b): builder with
  `.quit_on_last_window_closed(false)`, one window, close, then send
  `AppEvent::Exit` via proxy → `run` returns `Ok(())`, and a per-test atomic
  records that the loop was still running between the close and the explicit
  exit (set in `on_last_window_closed`).
- `test_windows_accessor_lists_live_set` (AC2): handler captures
  `registry.windows().collect::<Vec<_>>()` before close and after; asserts
  pre = 2 entries, post = 1 entry.
- `test_event_routed_to_correct_root` (AC5): two `RecordingRoot`s in
  separate windows; test synthesises a `WindowEvent` for window A (using
  winit's internal event-dispatch path is not possible from outside; use
  the `WrappedHandler::window_event` direct call in a unit-test boundary
  on the same file — gated to Linux for parity with the rest, or moved to a
  `#[cfg(test)] mod` inside `wrapped_handler.rs` if no display work is
  needed for the assertion).
- Fixtures: `tests/support/mod.rs` exposes (each test file declares
  `mod support;` to pull these in):
  - `RecordingRoot { records: Arc<Mutex<Vec<RootEventRecord>>> }`.
  - `build_app_with(quit_on_last_window_closed: bool) -> WindowedApplication`
    that mirrors the `xvfb_smoke.rs` worker-thread escape (`with_any_thread`).
  - `proxy_send_exit(proxy: &EventLoopProxy<AppEvent>)`.

### Task 7 — Doc + clippy + no-default-features

- `cargo doc` doc-link audit: every new `pub` item has a `# Examples` block
  per AGENTS.md (single-line public items need it; multi-line items SHOULD
  per the same rule).
- `cargo clippy --workspace -- -D warnings`.
- `cargo build -p quartzite --no-default-features` — verifies the
  `quartzite` facade still compiles without the renderer (the renderer is
  not a default-feature dep of `quartzite`, so this is automatic, but the
  build gates it).

## Open questions

- **Whether `WidgetRoot` should be folded into a closure adaptor** (like
  `RenderHarness::render_widget`) instead of a named trait. Argument for
  closure: parity with the harness; one less type to learn. Argument
  against: every widget hook becomes a separate closure stored in a
  `WindowEntry`, making the entry struct cumbersome; six closures × N
  windows is mechanical. **Design picks: named trait** for ergonomics, but
  flag the question for review.
- **Whether `on_last_window_closed` is even useful in this milestone.** It
  is purely a courtesy hook for opt-out callers and adds API surface.
  Removing it shrinks the trait to one method (`on_start`); the AC4b test
  can use an atomic flag set inside the test's `WidgetRoot::on_resize` or
  similar instead. **Design picks: include it** because AC4b's test design
  reads more cleanly with the hook, but flag for review.
- **Whether `try_create_window` should accept window-level configuration**
  (title, initial size, decorated/undecorated). Spec is silent. Minimal
  shape today: `try_create_window(&mut self, root: impl WidgetRoot) -> Result<WindowId, _>`
  with sane defaults (e.g., `WindowAttributes::default().with_title("quartzite")`).
  A future spec adds a builder for window attributes; current design exposes
  no attribute knobs. Flag for review — if reviewers want `title`/`size` in
  v1, add a `WindowAttributes` arg now to avoid a second break.
- **`WindowRegistry: !Send + !Sync` enforcement vs. ergonomics.** Both
  negative auto-traits prevent misuse (the held `*const ActiveEventLoop`
  slot is single-threaded by construction; see invariant 2 of the
  `active_loop` `# Safety` block) but block future patterns where the
  registry might be cloned into a worker thread for cross-thread
  `try_create_window`. The future-proof escape hatch is winit's
  `EventLoopProxy<AppEvent>`, which is `Send` by design and is already
  surfaced via `WindowedApplication::event_proxy()` — cross-thread create
  requests can be funnelled through it later without changing the
  registry's auto-trait posture. Confirm this is OK for v1.
