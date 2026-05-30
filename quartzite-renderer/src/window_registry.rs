//! [`WindowRegistry`] — per-process map of live windows.

use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr;
use std::sync::Arc;

use indexmap::IndexMap;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::RendererError;
use crate::window_id::WindowId;
use crate::window_root::WidgetRoot;

/// A single live window entry.
///
/// Field declaration order is critical: `surface` **must** be declared before
/// `window` so Rust's struct-field drop order (declaration order, first to
/// last) drops the wgpu surface before the winit window. The surface borrows
/// the window's raw handle at creation time; dropping it after the window
/// would be a use-after-free.
///
/// Both `surface` and `window` are `Option` so that `#[cfg(test)]` code can
/// create dispatch-only entries without a real display. `None` entries drop
/// safely (no-op) and preserve the declaration-order drop guarantee.
pub(crate) struct WindowEntry {
    /// The wgpu surface bound to this window, or `None` in test-only entries.
    ///
    /// `Surface<'static>` is safe because the `Arc<Window>` in the same entry
    /// is guaranteed to outlive the surface — ensured by declaration order.
    ///
    /// Not yet read directly: the surface presentation path is wired in a
    /// follow-up spec once `VelloPainter` is complete.
    #[allow(dead_code)]
    pub(crate) surface: Option<wgpu::Surface<'static>>,
    /// The winit window handle, or `None` in test-only entries.
    ///
    /// Kept in an `Arc` so `wgpu::Surface<'static>` can hold a clone of the
    /// raw-handle source (giving us `'static`).
    ///
    /// Not yet read directly: the window handle is accessed indirectly through
    /// the surface presentation path (follow-up spec).
    #[allow(dead_code)]
    pub(crate) window: Option<Arc<Window>>,
    /// The widget tree root that receives events for this window.
    pub(crate) root: Box<dyn WidgetRoot>,
}

/// Map of live windows for a [`WindowedApplication`] run.
///
/// `WindowRegistry` is **not** `Send` or `Sync` — it holds a transient
/// `*const ActiveEventLoop` slot that is only valid while a winit callback is
/// executing on the event-loop thread. The raw-pointer field auto-implies
/// `!Send + !Sync`; the `PhantomData<*const ()>` marker field makes the intent
/// explicit and protects against accidental refactors.
///
/// [`WindowedApplication`]: crate::application::WindowedApplication
pub struct WindowRegistry {
    pub(crate) windows: IndexMap<winit::window::WindowId, WindowEntry>,
    pub(crate) quit_on_last_window_closed: bool,
    pub(crate) instance: wgpu::Instance,
    /// Transient slot: non-null while a winit `ApplicationHandler` callback is
    /// executing; null otherwise.
    ///
    /// # Safety invariants
    ///
    /// 1. **Set-clear bracket.** Only [`WrappedHandler`] writes this field.
    ///    Before handing `&mut WindowRegistry` to user code the handler sets
    ///    the pointer to the current `&ActiveEventLoop`; an [`ActiveLoopGuard`]
    ///    clears it to `null` on `Drop`, even on panic.
    /// 2. **Single-threaded.** `WindowRegistry: !Send + !Sync` (enforced by
    ///    this raw-pointer field and `_not_send_sync`) prevents the registry
    ///    from crossing a thread boundary; the pointed-to `ActiveEventLoop`
    ///    lives on the same thread and is valid for the entire callback.
    /// 3. **Read-only inside `try_create_window`.** The `unsafe { &*ptr }`
    ///    conversion is performed only after a non-null check; the resulting
    ///    shared reference lives only for the body of `try_create_window`.
    /// 4. **Never null when read.** A null slot returns
    ///    `Err(RendererError::OutsideCallback)` before any deref.
    ///
    /// [`WrappedHandler`]: crate::wrapped_handler::WrappedHandler
    /// [`ActiveLoopGuard`]: crate::wrapped_handler::ActiveLoopGuard
    pub(crate) active_loop: Cell<*const ActiveEventLoop>,
    _not_send_sync: PhantomData<*const ()>,
}

impl WindowRegistry {
    /// Creates an empty registry.
    #[inline]
    pub(crate) fn new(quit_on_last_window_closed: bool, instance: wgpu::Instance) -> Self {
        Self {
            windows: IndexMap::new(),
            quit_on_last_window_closed,
            instance,
            active_loop: Cell::new(ptr::null()),
            _not_send_sync: PhantomData,
        }
    }

    /// Creates a new window, registers it, and returns its [`WindowId`].
    ///
    /// Must be called from within a [`WindowedAppHandler`] callback (i.e. while
    /// the winit event loop is active). Calling it outside a callback returns
    /// [`RendererError::OutsideCallback`].
    ///
    /// # Parameters
    ///
    /// - `root`: the widget tree root that will receive events for the new
    ///   window; ownership is transferred to the registry.
    ///
    /// # Errors
    ///
    /// - [`RendererError::OutsideCallback`] — called outside a winit
    ///   `ApplicationHandler` callback (the internal `&ActiveEventLoop` slot
    ///   is null).
    /// - [`RendererError::OsError`] — winit's
    ///   [`ActiveEventLoop::create_window`] failed (e.g. platform error).
    /// - [`RendererError::Surface`] — wgpu surface creation failed.
    ///
    /// [`WindowedAppHandler`]: crate::windowed_app_handler::WindowedAppHandler
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_renderer::{
    ///     WindowedApplication, WindowedAppHandler, WindowRegistry, WidgetRoot,
    /// };
    /// use quartzite_events::{KeyEvent, MouseEvent};
    /// use quartzite_geometry::Size;
    /// use quartzite_paint_api::Painter;
    ///
    /// struct MyRoot;
    /// impl WidgetRoot for MyRoot {
    ///     fn paint(&self, _p: &mut dyn Painter) {}
    ///     fn on_resize(&mut self, _s: Size) {}
    ///     fn on_mouse_press(&mut self, _e: &MouseEvent) {}
    ///     fn on_mouse_release(&mut self, _e: &MouseEvent) {}
    ///     fn on_key_press(&mut self, _e: &KeyEvent) {}
    ///     fn on_key_release(&mut self, _e: &KeyEvent) {}
    /// }
    ///
    /// struct MyHandler;
    /// impl WindowedAppHandler for MyHandler {
    ///     fn on_start(&mut self, registry: &mut WindowRegistry) {
    ///         registry.try_create_window(MyRoot).unwrap();
    ///     }
    /// }
    /// ```
    pub fn try_create_window(&mut self, root: impl WidgetRoot) -> Result<WindowId, RendererError> {
        let ptr = self.active_loop.get();
        if ptr.is_null() {
            return Err(RendererError::OutsideCallback);
        }
        // SAFETY: invariants on `active_loop`:
        // (1) set-clear bracket in WrappedHandler guarantees non-null here,
        // (2) !Send+!Sync prevents cross-thread access,
        // (3) the ref lives only for this function body,
        // (4) null check above prevents deref of null.
        let event_loop = unsafe { &*ptr };
        let window = Arc::new(
            event_loop
                .create_window(winit::window::WindowAttributes::default().with_title("quartzite"))
                .map_err(RendererError::OsError)?,
        );
        let surface = self
            .instance
            .create_surface(window.clone())
            .map_err(RendererError::Surface)?;
        let winit_id = window.id();
        let entry = WindowEntry {
            surface: Some(surface),
            window: Some(window),
            root: Box::new(root),
        };
        self.windows.insert(winit_id, entry);
        Ok(WindowId(winit_id))
    }

    /// Returns an iterator over the [`WindowId`]s of all currently live windows.
    ///
    /// Iteration order is insertion order (backed by [`IndexMap`]). The order
    /// is stable within one [`WindowedApplication::run`] invocation.
    ///
    /// [`WindowedApplication::run`]: crate::application::WindowedApplication::run
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use quartzite_renderer::{WindowRegistry, WindowId};
    /// fn list_windows(registry: &WindowRegistry) -> Vec<WindowId> {
    ///     registry.windows().collect()
    /// }
    /// ```
    #[inline]
    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.windows.keys().copied().map(WindowId)
    }

    /// Returns `true` if there are no live windows.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Inserts a dispatch-only entry into the registry without a real window or
    /// surface.
    ///
    /// Used by unit tests in `wrapped_handler` to exercise dispatch logic
    /// without requiring a live display server or `ActiveEventLoop`.
    /// The `window` and `surface` fields of the created entry are `None`; the
    /// entry **must not** trigger `RedrawRequested` (which would call
    /// `VelloPainter` against a null surface) — only resize, input, and close
    /// paths are exercised by tests that use this helper.
    #[cfg(test)]
    pub(crate) fn insert_root_for_test(
        &mut self,
        winit_id: winit::window::WindowId,
        root: impl WidgetRoot,
    ) {
        let entry = WindowEntry {
            surface: None,
            window: None,
            root: Box::new(root),
        };
        self.windows.insert(winit_id, entry);
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn try_create_window_outside_callback_returns_err() {
        // Build a registry without setting the active_loop slot (slot stays null).
        // try_create_window must return OutsideCallback without dereferencing the null.
        // We use a dummy instance and root to confirm the null check fires first.
        struct Noop;
        impl WidgetRoot for Noop {
            fn paint(&self, _: &mut dyn quartzite_paint_api::Painter) {}
            fn on_resize(&mut self, _: quartzite_geometry::Size) {}
            fn on_mouse_press(&mut self, _: &quartzite_events::MouseEvent) {}
            fn on_mouse_release(&mut self, _: &quartzite_events::MouseEvent) {}
            fn on_key_press(&mut self, _: &quartzite_events::KeyEvent) {}
            fn on_key_release(&mut self, _: &quartzite_events::KeyEvent) {}
        }

        let mut registry = WindowRegistry::new(true, wgpu::Instance::default());
        // active_loop slot is null — must get OutsideCallback
        let result = registry.try_create_window(Noop);
        assert_matches!(
            result,
            Err(RendererError::OutsideCallback),
            "expected OutsideCallback, got {result:?}"
        );
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = WindowRegistry::new(true, wgpu::Instance::default());
        assert!(registry.is_empty());
        assert_eq!(registry.windows().count(), 0);
    }
}
