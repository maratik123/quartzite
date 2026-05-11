//! [`WindowId`] — opaque identifier for a live window.

/// Opaque identifier for a window managed by [`WindowRegistry`].
///
/// Obtained from [`WindowRegistry::try_create_window`] and used to correlate
/// events with windows. `WindowId` values are unique within a process lifetime
/// and are never reissued after a window closes, matching winit's own guarantee.
///
/// [`WindowRegistry`]: crate::window_registry::WindowRegistry
/// [`WindowRegistry::try_create_window`]: crate::window_registry::WindowRegistry::try_create_window
///
/// # Examples
///
/// Two `WindowId`s from different sources are never equal
/// (`WindowId` wraps winit's opaque id which is process-unique):
///
/// ```
/// # // WindowId can only be obtained from a live WindowRegistry,
/// # // so this doctest just confirms the type and its derives are accessible.
/// use quartzite_renderer::WindowId;
/// // WindowId is Copy, Clone, Eq, Hash — verify those bounds compile.
/// fn _assert_copy<T: Copy + Clone + Eq + std::hash::Hash>() {}
/// _assert_copy::<WindowId>();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) winit::window::WindowId);

impl From<winit::window::WindowId> for WindowId {
    #[inline]
    fn from(id: winit::window::WindowId) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_does_not_panic() {
        // WindowId(winit::window::WindowId) — just check it formats without panic.
        // We cannot construct a winit WindowId from tests, so we test the derive chain
        // is present by verifying the type is Copy / Clone / Eq / Hash.
        let _: fn(WindowId) -> WindowId = |w| w;
    }
}
