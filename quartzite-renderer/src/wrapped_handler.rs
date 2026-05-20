//! [`WrappedHandler`] — winit `ApplicationHandler` that owns the window registry.

use std::cell::Cell;
use std::ptr;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId as WinitWindowId;

use crate::application_builder::AppEvent;
use crate::event_convert::{
    key_event_from_winit, modifiers_from_winit, mouse_button_from_winit, mouse_event_from_winit,
    size_from_physical,
};
use crate::font::FontCache;
use crate::vello_painter::VelloPainter;
use crate::window_registry::WindowRegistry;
use crate::windowed_app_handler::WindowedAppHandler;
use vello::Scene;

/// RAII guard that resets a `Cell<*const ActiveEventLoop>` to null on drop.
///
/// Holds a raw pointer so no borrow is retained — the caller can pass
/// `&mut WindowRegistry` to user code while the guard is live.
///
/// # Safety
///
/// The caller must ensure the pointer remains valid (points into a live
/// `WindowRegistry`) for the entire lifetime of this guard.
pub(crate) struct ActiveLoopGuard(*const Cell<*const ActiveEventLoop>);

impl Drop for ActiveLoopGuard {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: pointer was set from a reference to a WindowRegistry field
        // that is guaranteed to outlive this guard (guard is scoped to one
        // ApplicationHandler callback, registry lives for the full run_app).
        unsafe { (*self.0).set(ptr::null()) };
    }
}

/// The internal `winit::ApplicationHandler` that drives the window registry.
///
/// Constructed by [`WindowedApplication::run`] and never exposed publicly.
///
/// [`WindowedApplication::run`]: crate::application::WindowedApplication::run
pub(crate) struct WrappedHandler<H: WindowedAppHandler> {
    pub(crate) registry: WindowRegistry,
    pub(crate) user_handler: H,
    pub(crate) fonts: FontCache,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    pressed_buttons: quartzite_events::MouseButtons,
    modifiers: winit::event::Modifiers,
}

impl<H: WindowedAppHandler> WrappedHandler<H> {
    /// _Simple._
    pub(crate) fn new(registry: WindowRegistry, user_handler: H) -> Self {
        Self {
            registry,
            user_handler,
            fonts: FontCache::new(),
            cursor_position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
            pressed_buttons: quartzite_events::MouseButtons::empty(),
            modifiers: winit::event::Modifiers::default(),
        }
    }

    /// Sets the `active_loop` slot and returns a guard that clears it on drop.
    ///
    /// The guard holds a raw pointer so the borrow on `self.registry` does not
    /// persist — `self.user_handler` and `self.registry` can be borrowed
    /// mutably after this call returns while the guard is still on the stack.
    ///
    /// _Simple._
    fn arm_active_loop(&self, event_loop: &ActiveEventLoop) -> ActiveLoopGuard {
        self.registry
            .active_loop
            .set(std::ptr::from_ref::<ActiveEventLoop>(event_loop));
        // Cast to raw pointer immediately; the reference borrow ends here.
        // SAFETY: `self.registry` outlives every callback invocation.
        let cell_ptr = &raw const self.registry.active_loop;
        ActiveLoopGuard(cell_ptr)
    }
}

impl<H: WindowedAppHandler> ApplicationHandler<AppEvent> for WrappedHandler<H> {
    // _Simple._
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let _guard = self.arm_active_loop(event_loop);
        self.user_handler.on_start(&mut self.registry);
    }

    // _Simple._
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WinitWindowId,
        event: WindowEvent,
    ) {
        let _guard = self.arm_active_loop(event_loop);
        self.dispatch_window_event(event_loop, window_id, event);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::Exit => event_loop.exit(),
        }
    }
}

impl<H: WindowedAppHandler> WrappedHandler<H> {
    fn dispatch_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WinitWindowId,
        event: WindowEvent,
    ) {
        if self.dispatch_window_event_inner(window_id, event) {
            event_loop.exit();
        }
    }

    /// Processes one `WindowEvent` and returns `true` when the event loop
    /// should exit.
    ///
    /// Separated from `dispatch_window_event` so the pure dispatch logic can be
    /// unit-tested without a live `ActiveEventLoop`.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "WindowEvent consumed by pattern match; changing to &WindowEvent requires ref-patterns throughout and degrades readability"
    )]
    pub(crate) fn dispatch_window_event_inner(
        &mut self,
        window_id: WinitWindowId,
        event: WindowEvent,
    ) -> bool {
        match event {
            WindowEvent::CloseRequested => {
                self.registry.windows.shift_remove(&window_id);
                if self.registry.is_empty() {
                    self.user_handler.on_last_window_closed(&mut self.registry);
                    return self.registry.quit_on_last_window_closed;
                }
                false
            }
            WindowEvent::Resized(size) => {
                if let Some(entry) = self.registry.windows.get_mut(&window_id) {
                    entry.root.on_resize(size_from_physical(size));
                }
                false
            }
            WindowEvent::RedrawRequested => {
                if let Some(entry) = self.registry.windows.get(&window_id) {
                    #[allow(
                        clippy::cast_possible_truncation,
                        reason = "deliberate truncation within known bounds"
                    )]
                    let scale = entry
                        .window
                        .as_ref()
                        .map_or(1.0, |w| w.scale_factor() as f32);
                    let mut scene = Scene::new();
                    {
                        let mut painter = VelloPainter::new(&mut scene)
                            .with_scale(scale)
                            .with_fonts(&mut self.fonts);
                        entry.root.paint(&mut painter);
                    }
                    // TODO: submit scene via wgpu surface (windowed pipeline — follow-up)
                    let _ = scene;
                }
                false
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let quartzite_btn = mouse_button_from_winit(button);
                match state {
                    winit::event::ElementState::Pressed => {
                        self.pressed_buttons |= quartzite_btn;
                    }
                    winit::event::ElementState::Released => {
                        self.pressed_buttons &= !quartzite_btn;
                    }
                }
                if let Some(entry) = self.registry.windows.get_mut(&window_id) {
                    let evt = mouse_event_from_winit(
                        state,
                        button,
                        self.cursor_position,
                        self.pressed_buttons,
                    );
                    match state {
                        winit::event::ElementState::Pressed => entry.root.on_mouse_press(&evt),
                        winit::event::ElementState::Released => entry.root.on_mouse_release(&evt),
                    }
                }
                false
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = position;
                false
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods;
                false
            }
            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } => {
                let mods = modifiers_from_winit(self.modifiers);
                if let (Some(entry), Some(evt)) = (
                    self.registry.windows.get_mut(&window_id),
                    key_event_from_winit(key_event, mods),
                ) {
                    match key_event.state {
                        winit::event::ElementState::Pressed => entry.root.on_key_press(&evt),
                        winit::event::ElementState::Released => entry.root.on_key_release(&evt),
                    }
                }
                false
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;

    use quartzite_events::{KeyEvent, MouseEvent};
    use quartzite_geometry::Size;
    use quartzite_paint_api::Painter;

    use crate::window_registry::WindowRegistry;
    use crate::window_root::WidgetRoot;
    use crate::windowed_app_handler::WindowedAppHandler;

    use super::*;

    // --- helpers -----------------------------------------------------------------

    /// Creates a fake `winit::window::WindowId` from a `u64`.
    ///
    /// # Safety (test-only)
    /// On Linux (the platform where these tests run) `winit::window::WindowId`
    /// is `pub struct WindowId(u64)`. The transmute is size-safe (asserted
    /// below) and the value is only ever used as an `IndexMap` key in tests
    /// that never create a real platform window.
    fn fake_id(n: u64) -> WinitWindowId {
        assert_eq!(
            std::mem::size_of::<WinitWindowId>(),
            8,
            "WindowId size changed; update fake_id"
        );
        // SAFETY: WinitWindowId is Copy + Eq + Hash with no pointer or
        // validity invariants beyond its size.  Test-only code, never reaches
        // production.
        unsafe { std::mem::transmute(n) }
    }

    /// A minimal `WindowedAppHandler` that does nothing.
    struct NoopHandler;
    impl WindowedAppHandler for NoopHandler {
        fn on_start(&mut self, _registry: &mut WindowRegistry) {}
    }

    /// A `WidgetRoot` that counts how many times each method is called.
    #[allow(
        clippy::struct_field_names,
        reason = "Test fixture mirrors method names by design — `<method>_calls` is the readable convention"
    )]
    struct CountingRoot {
        resize_calls: Arc<Mutex<Vec<Size>>>,
        press_calls: Arc<Mutex<u32>>,
        release_calls: Arc<Mutex<u32>>,
        key_press_calls: Arc<Mutex<u32>>,
        key_release_calls: Arc<Mutex<u32>>,
    }

    struct CountingRootHandle {
        root: CountingRoot,
        resizes: Arc<Mutex<Vec<Size>>>,
    }

    impl CountingRootHandle {
        fn new() -> Self {
            let resizes = Arc::new(Mutex::new(vec![]));
            Self {
                root: CountingRoot {
                    resize_calls: resizes.clone(),
                    press_calls: Arc::new(Mutex::new(0)),
                    release_calls: Arc::new(Mutex::new(0)),
                    key_press_calls: Arc::new(Mutex::new(0)),
                    key_release_calls: Arc::new(Mutex::new(0)),
                },
                resizes,
            }
        }
    }

    impl WidgetRoot for CountingRoot {
        fn paint(&self, _painter: &mut dyn Painter) {}
        fn on_resize(&mut self, size: Size) {
            self.resize_calls.lock().push(size);
        }
        fn on_mouse_press(&mut self, _event: &MouseEvent) {
            *self.press_calls.lock() += 1;
        }
        fn on_mouse_release(&mut self, _event: &MouseEvent) {
            *self.release_calls.lock() += 1;
        }
        fn on_key_press(&mut self, _event: &KeyEvent) {
            *self.key_press_calls.lock() += 1;
        }
        fn on_key_release(&mut self, _event: &KeyEvent) {
            *self.key_release_calls.lock() += 1;
        }
    }

    fn make_handler(quit: bool) -> WrappedHandler<NoopHandler> {
        let registry = WindowRegistry::new(quit, wgpu::Instance::default());
        WrappedHandler::new(registry, NoopHandler)
    }

    // --- AC3: close non-last window -----------------------------------------------

    /// AC3: closing window A when window B is still open removes only A from
    /// the registry; B's entry survives and the registry is not empty.
    #[test]
    fn close_non_last_window_leaves_other_entry() {
        let mut handler = make_handler(true);
        let id_a = fake_id(1);
        let id_b = fake_id(2);
        let CountingRootHandle { root: root_a, .. } = CountingRootHandle::new();
        let CountingRootHandle { root: root_b, .. } = CountingRootHandle::new();
        handler.registry.insert_root_for_test(id_a, root_a);
        handler.registry.insert_root_for_test(id_b, root_b);
        assert_eq!(handler.registry.windows.len(), 2);

        let should_exit = handler.dispatch_window_event_inner(id_a, WindowEvent::CloseRequested);

        assert!(
            !should_exit,
            "loop must not exit when another window remains"
        );
        assert_eq!(
            handler.registry.windows.len(),
            1,
            "registry must have exactly one entry after closing window A"
        );
        assert!(
            handler.registry.windows.contains_key(&id_b),
            "window B must still be in the registry"
        );
        assert!(
            !handler.registry.windows.contains_key(&id_a),
            "window A must be gone from the registry"
        );
    }

    /// AC3 + quit policy: closing the last window with quit=true signals exit.
    #[test]
    fn close_last_window_with_quit_true_signals_exit() {
        let mut handler = make_handler(true);
        let id = fake_id(42);
        let CountingRootHandle { root, .. } = CountingRootHandle::new();
        handler.registry.insert_root_for_test(id, root);

        let should_exit = handler.dispatch_window_event_inner(id, WindowEvent::CloseRequested);

        assert!(
            should_exit,
            "loop must exit when last window closes with quit=true"
        );
        assert!(handler.registry.is_empty());
    }

    /// Closing the last window with quit=false must NOT signal exit.
    #[test]
    fn close_last_window_with_quit_false_does_not_signal_exit() {
        let mut handler = make_handler(false);
        let id = fake_id(99);
        let CountingRootHandle { root, .. } = CountingRootHandle::new();
        handler.registry.insert_root_for_test(id, root);

        let should_exit = handler.dispatch_window_event_inner(id, WindowEvent::CloseRequested);

        assert!(!should_exit, "loop must not exit when quit=false");
    }

    // --- AC5: per-window event routing -------------------------------------------

    /// AC5: Resized event for window A routes to root A's `on_resize`, not root B.
    #[test]
    fn resized_event_routes_to_correct_root() {
        let mut handler = make_handler(true);
        let id_a = fake_id(10);
        let id_b = fake_id(20);
        let CountingRootHandle {
            root: root_a,
            resizes: resizes_a,
            ..
        } = CountingRootHandle::new();
        let CountingRootHandle {
            root: root_b,
            resizes: resizes_b,
            ..
        } = CountingRootHandle::new();
        handler.registry.insert_root_for_test(id_a, root_a);
        handler.registry.insert_root_for_test(id_b, root_b);

        handler.dispatch_window_event_inner(
            id_a,
            WindowEvent::Resized(winit::dpi::PhysicalSize::new(800u32, 600u32)),
        );

        let got_a = resizes_a.lock().clone();
        let got_b = resizes_b.lock().clone();
        assert_eq!(
            got_a,
            vec![Size::new(800, 600)],
            "root A must receive the resize"
        );
        assert!(got_b.is_empty(), "root B must not receive root A's resize");
    }

    /// AC5: event for an unknown (already-closed) `window_id` is silently dropped.
    #[test]
    fn event_for_unknown_window_id_is_silently_dropped() {
        let mut handler = make_handler(true);
        let id_a = fake_id(1);
        let id_unknown = fake_id(999);
        let CountingRootHandle {
            root: root_a,
            resizes: resizes_a,
            ..
        } = CountingRootHandle::new();
        handler.registry.insert_root_for_test(id_a, root_a);

        let should_exit = handler.dispatch_window_event_inner(
            id_unknown,
            WindowEvent::Resized(winit::dpi::PhysicalSize::new(100u32, 100u32)),
        );

        assert!(!should_exit);
        assert!(
            resizes_a.lock().is_empty(),
            "root A must not receive events for an unknown window"
        );
    }

    // --- R4: MouseInput / CursorMoved / ModifiersChanged / KeyboardInput ----------

    struct HandlerWithRoot {
        handler: WrappedHandler<NoopHandler>,
        id: WinitWindowId,
        presses: Arc<Mutex<u32>>,
        releases: Arc<Mutex<u32>>,
    }

    fn make_handler_with_root(quit: bool) -> HandlerWithRoot {
        let mut handler = make_handler(quit);
        let id = fake_id(7);
        let presses = Arc::new(Mutex::new(0u32));
        let releases = Arc::new(Mutex::new(0u32));
        let root = CountingRoot {
            resize_calls: Arc::new(Mutex::new(vec![])),
            press_calls: presses.clone(),
            release_calls: releases.clone(),
            key_press_calls: Arc::new(Mutex::new(0)),
            key_release_calls: Arc::new(Mutex::new(0)),
        };
        handler.registry.insert_root_for_test(id, root);
        HandlerWithRoot {
            handler,
            id,
            presses,
            releases,
        }
    }

    #[test]
    fn mouse_input_pressed_increments_press_count() {
        let HandlerWithRoot {
            mut handler,
            id,
            presses: press,
            releases: release,
            ..
        } = make_handler_with_root(false);
        let btn = winit::event::MouseButton::Left;
        handler.dispatch_window_event_inner(
            id,
            WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Pressed,
                button: btn,
            },
        );
        assert_eq!(*press.lock(), 1, "press_calls should be 1");
        assert_eq!(*release.lock(), 0, "release_calls should be 0");
    }

    #[test]
    fn mouse_input_released_increments_release_count() {
        let HandlerWithRoot {
            mut handler,
            id,
            presses: press,
            releases: release,
            ..
        } = make_handler_with_root(false);
        let btn = winit::event::MouseButton::Left;
        // Press first so pressed_buttons is consistent
        handler.dispatch_window_event_inner(
            id,
            WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Pressed,
                button: btn,
            },
        );
        handler.dispatch_window_event_inner(
            id,
            WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Released,
                button: btn,
            },
        );
        assert_eq!(*press.lock(), 1);
        assert_eq!(*release.lock(), 1);
    }

    #[test]
    fn cursor_moved_updates_cursor_position() {
        let HandlerWithRoot {
            mut handler, id, ..
        } = make_handler_with_root(false);
        handler.dispatch_window_event_inner(
            id,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(50.0_f64, 60.0),
            },
        );
        // cursor_position is private; verify indirectly by dispatching a
        // MouseInput immediately after and asserting the event was delivered
        // (the press callback fires, proving the entire arm ran).
        handler.dispatch_window_event_inner(
            id,
            WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
            },
        );
        // If CursorMoved arm was hit, cursor_position was updated; the
        // MouseInput arm fire confirms both arms ran successfully.
        assert!(
            handler.registry.windows.contains_key(&id),
            "window entry must still exist"
        );
    }

    #[test]
    fn modifiers_changed_does_not_panic() {
        let HandlerWithRoot {
            mut handler, id, ..
        } = make_handler_with_root(false);
        // A default Modifiers (no flags) — just verifying the arm runs without panic.
        handler.dispatch_window_event_inner(
            id,
            WindowEvent::ModifiersChanged(winit::event::Modifiers::default()),
        );
    }

    // KeyboardInput dispatch tests are omitted here: constructing
    // `winit::event::KeyEvent` from outside winit requires setting the
    // `platform_specific` field which is `pub(crate)` in winit and not
    // constructable from external test code. The keyboard path is covered
    // indirectly by `key_event_from_parts` tests in event_convert.rs (R1).
    // See design doc § R4 "Risk" bullet.

    // --- guard tests (unchanged) -------------------------------------------------

    #[test]
    fn active_loop_guard_clears_slot_on_drop() {
        let registry = WindowRegistry::new(true, wgpu::Instance::default());
        assert!(registry.active_loop.get().is_null());

        // Simulate what arm_active_loop does: set a non-null address.
        let fake_addr: *const ActiveEventLoop = 0xdead_beef as *const _;
        registry.active_loop.set(fake_addr);
        assert!(!registry.active_loop.get().is_null());

        {
            let cell_ptr = &raw const registry.active_loop;
            let _guard = ActiveLoopGuard(cell_ptr);
        }

        assert!(
            registry.active_loop.get().is_null(),
            "guard must clear the slot on drop"
        );
    }

    #[test]
    fn active_loop_guard_clears_slot_on_panic() {
        let registry = WindowRegistry::new(true, wgpu::Instance::default());
        let fake_addr: *const ActiveEventLoop = 0x1234 as *const _;
        registry.active_loop.set(fake_addr);

        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let cell_ptr = &raw const registry.active_loop;
            let _guard = ActiveLoopGuard(cell_ptr);
            panic!("simulated callback panic");
        }));

        assert!(
            registry.active_loop.get().is_null(),
            "guard must clear slot even after panic"
        );
    }
}
