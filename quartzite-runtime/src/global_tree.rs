//! Process-global liveness flag for the [`ObjectTree`](crate::ObjectTree) accessor.
//!
//! Set to `true` by [`ApplicationBuilder::build`](crate::ApplicationBuilder::build) and cleared by
//! `Drop for Application`. The flag gates [`try_with_tree`](crate::try_with_tree).
use std::sync::atomic::{AtomicBool, Ordering};

static TREE_LIVE: AtomicBool = AtomicBool::new(false);

#[inline]
pub(crate) fn register() {
    TREE_LIVE.store(true, Ordering::Release);
}

#[inline]
pub(crate) fn deregister() {
    TREE_LIVE.store(false, Ordering::Release);
}

#[inline]
pub(crate) fn is_live() -> bool {
    TREE_LIVE.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_live_returns_false_when_not_registered() {
        // Unit tests never call Application::builder().build(), so TREE_LIVE stays false
        // (initialised to false in the static initialiser).
        assert!(!is_live());
    }
}
