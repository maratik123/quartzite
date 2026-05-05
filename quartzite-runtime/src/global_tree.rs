//! Process-global [`ObjectTree`](crate::ObjectTree) accessor registered by [`Application`].
use std::{
    ptr,
    sync::{
        Mutex,
        atomic::{AtomicPtr, Ordering},
    },
};

use crate::object_tree::ObjectTree;

static TREE_PTR: AtomicPtr<Mutex<ObjectTree>> = AtomicPtr::new(ptr::null_mut());

#[inline]
pub(crate) fn register(tree: *const Mutex<ObjectTree>) {
    TREE_PTR.store(tree as *mut _, Ordering::Release);
}

#[inline]
pub(crate) fn deregister() {
    TREE_PTR.store(ptr::null_mut(), Ordering::Release);
}

/// Calls `f` with a shared reference to the active [`ObjectTree`] and returns
/// the result wrapped in `Some`, or returns `None` if no
/// [`Application`](crate::Application) is currently live.
///
/// # Parameters
///
/// - `f`: closure that receives a shared reference to the active tree.
///
/// # Panics
///
/// Panics if the internal `ObjectTree` mutex is poisoned (i.e., another thread
/// panicked while holding the lock). Under normal operation this does not occur.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::try_with_tree;
///
/// // Returns None when called before Application::new()
/// assert!(try_with_tree(|_tree| ()).is_none());
/// ```
pub fn try_with_tree<R>(f: impl FnOnce(&ObjectTree) -> R) -> Option<R> {
    let ptr = TREE_PTR.load(Ordering::Acquire);
    if ptr.is_null() {
        return None;
    }
    // SAFETY: The pointer was stored by `register`, called exclusively from
    // `Application::new`. It points to `ApplicationInner::object_tree` which
    // lives inside `Arc<ApplicationInner>` held by `APP: OnceLock`, never
    // cleared for the process lifetime. A non-null Acquire load guarantees the
    // pointer is still valid. The project is single-threaded in v1; the
    // Release store in `deregister` precedes any access to the pointee after
    // the `Application` handle is dropped.
    let guard = unsafe { &*ptr }.lock().unwrap();
    Some(f(&guard))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_with_tree_returns_none_when_not_registered() {
        // Unit tests never call Application::new(), so the pointer is null
        // (set to null_mut() in the static initialiser).
        assert!(try_with_tree(|_| ()).is_none());
    }
}
