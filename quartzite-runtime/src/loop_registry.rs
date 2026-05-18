//! Process-wide per-thread [`EventLoop`] registry.
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
    thread::ThreadId,
};

use parking_lot::RwLock;

use crate::event_loop::EventLoop;

static REGISTRY: LazyLock<RwLock<HashMap<ThreadId, Arc<EventLoop>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Error returned when a thread already has an installed [`EventLoop`].
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use quartzite_runtime::{EventLoop, LoopAlreadyInstalled};
///
/// let el = Arc::new(EventLoop::new());
/// el.clone().install_for_current_thread().unwrap();
/// assert_eq!(
///     el.install_for_current_thread(),
///     Err(LoopAlreadyInstalled),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("an EventLoop is already installed for this thread")]
pub struct LoopAlreadyInstalled;

/// RAII guard that removes the current thread's registry entry on drop.
///
/// Entered inside [`EventLoop::run`] so the registry stays clean even if a closure panics.
pub(crate) struct RegistryGuard;

impl Drop for RegistryGuard {
    fn drop(&mut self) {
        LoopRegistry::uninstall(std::thread::current().id());
    }
}

/// Process-wide registry mapping each thread's [`ThreadId`] to its installed [`EventLoop`].
///
/// Worker threads register via [`EventLoop::install_for_current_thread`];
/// [`ConnectionTable`](crate::connection_table::ConnectionTable) looks up the appropriate
/// loop when routing queued signal invocations.
pub(crate) struct LoopRegistry;

impl LoopRegistry {
    /// Registers `loop_` for thread `id`.
    ///
    /// Returns `Err` without modifying the registry if an entry already exists for `id`.
    #[allow(
        clippy::significant_drop_tightening,
        reason = "MutexGuard held intentionally to keep critical section atomic"
    )]
    pub(crate) fn install(id: ThreadId, loop_: Arc<EventLoop>) -> Result<(), LoopAlreadyInstalled> {
        let mut map = REGISTRY.write();
        if map.contains_key(&id) {
            return Err(LoopAlreadyInstalled);
        }
        map.insert(id, loop_);
        Ok(())
    }

    /// Removes the registry entry for `id`. No-op if no entry exists.
    #[inline]
    pub(crate) fn uninstall(id: ThreadId) {
        REGISTRY.write().remove(&id);
    }

    /// Returns the registered loop for `id`, or `None` if no loop is installed for that thread.
    #[inline]
    pub(crate) fn get(id: ThreadId) -> Option<Arc<EventLoop>> {
        REGISTRY.read().get(&id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, thread};

    /// Spawn a thread, capture its `ThreadId`, join, and return it.
    fn other_thread_id() -> ThreadId {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || tx.send(thread::current().id()).unwrap())
            .join()
            .unwrap();
        rx.recv().unwrap()
    }

    #[test]
    fn install_and_get() {
        let id = other_thread_id();
        let el = Arc::new(EventLoop::new());
        LoopRegistry::install(id, Arc::clone(&el)).unwrap();
        assert!(LoopRegistry::get(id).is_some());
        LoopRegistry::uninstall(id);
    }

    #[test]
    fn double_install_returns_error() {
        let id = other_thread_id();
        let el = Arc::new(EventLoop::new());
        LoopRegistry::install(id, Arc::clone(&el)).unwrap();
        assert_eq!(
            LoopRegistry::install(id, Arc::clone(&el)),
            Err(LoopAlreadyInstalled)
        );
        LoopRegistry::uninstall(id);
    }

    #[test]
    fn uninstall_removes_entry() {
        let id = other_thread_id();
        let el = Arc::new(EventLoop::new());
        LoopRegistry::install(id, Arc::clone(&el)).unwrap();
        LoopRegistry::uninstall(id);
        assert!(LoopRegistry::get(id).is_none());
    }

    #[test]
    fn uninstall_noop_when_absent() {
        let id = other_thread_id();
        LoopRegistry::uninstall(id); // must not panic
    }

    #[test]
    fn thread_b_cannot_see_thread_a_entry() {
        let id_a = other_thread_id();
        let id_b = other_thread_id();
        let el = Arc::new(EventLoop::new());
        LoopRegistry::install(id_a, Arc::clone(&el)).unwrap();
        assert!(LoopRegistry::get(id_b).is_none());
        LoopRegistry::uninstall(id_a);
    }
}
