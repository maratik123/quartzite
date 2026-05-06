//! Process-singleton `Application` and `ApplicationError`.
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;

use crate::{connection_table::ConnectionTable, event_loop::EventLoop, object_tree::ObjectTree};

/// Error returned by [`Application::new`] when it fails.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::{Application, ApplicationError};
///
/// let _first = Application::new().expect("first call succeeds");
/// match Application::new() {
///     Err(ApplicationError::AlreadyExists) => {}
///     _ => panic!("second call must fail with AlreadyExists"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplicationError {
    /// An `Application` instance already exists in this process.
    #[error("Application already exists")]
    AlreadyExists,
}

struct ApplicationInner {
    /// `Mutex` (not `RwLock`) because `ObjectTree: Send` but not `Sync`.
    /// `RwLock<T>` requires `T: Send + Sync`; `Mutex<T>` only requires `T: Send`.
    object_tree: Mutex<ObjectTree>,
    event_loop: Arc<EventLoop>,
    connection_table: Arc<ConnectionTable>,
    main_thread_id: std::thread::ThreadId,
}

static APP: OnceLock<Arc<ApplicationInner>> = OnceLock::new();

/// Singleton entry point for the quartzite runtime.
///
/// Owns the [`ObjectTree`] and [`EventLoop`]. Creates the [`ConnectionTable`] and
/// installs it as the process-wide queued dispatcher.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::Application;
///
/// let app = Application::new().expect("only one Application per process");
/// app.quit();
/// ```
pub struct Application(Arc<ApplicationInner>);

impl Application {
    /// Creates the application singleton and installs the queued dispatcher and object factory.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError::AlreadyExists`] if an `Application` has already been
    /// installed in this process. Only one `Application` may exist per process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().expect("only one Application per process");
    /// ```
    pub fn new() -> Result<Self, ApplicationError> {
        let main_thread_id = std::thread::current().id();
        let event_loop = Arc::new(EventLoop::new());
        let connection_table = ConnectionTable::new();
        let inner = Arc::new(ApplicationInner {
            object_tree: Mutex::new(ObjectTree::new()),
            event_loop,
            connection_table,
            main_thread_id,
        });

        APP.set(Arc::clone(&inner))
            .map_err(|_| ApplicationError::AlreadyExists)?;

        // Install the main-thread event loop in the registry so queued signals
        // targeting objects on this thread are routed correctly.
        // APP.set above guarantees no Application existed before, so the only way
        // this can fail is if the caller pre-installed a loop on this thread —
        // that is a caller bug; ignore it rather than failing Application::new().
        let _ = Arc::clone(&inner.event_loop).install_for_current_thread();

        // Register ConnectionTable as the queued dispatcher. Ignore if already
        // set — only one Application can exist per process, so the only way
        // this can fail is if the dispatcher was set before Application::new()
        // (e.g. in tests that call set_queued_dispatcher directly).
        let _ = quartzite_core::set_queued_dispatcher(
            Arc::clone(&inner.connection_table) as Arc<dyn quartzite_core::QueuedDispatcher>
        );

        // Install the process-wide factory. Ignore FactoryAlreadySet — same rationale
        // as the dispatcher above. Subsequent Application::new() calls (if somehow
        // possible) share the first factory via OnceLock semantics.
        let _ = crate::factory::ObjectFactory::install(crate::factory::ObjectFactory::new());

        // Mark the global tree as live so ObjectTreeExt::parent/children work.
        crate::global_tree::register();

        Ok(Application(inner))
    }

    /// Returns a handle to the global application, or `None` if it has not been installed yet.
    ///
    /// Calling [`Application::new`] is required before this returns `Some`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// if let Some(app) = Application::global() {
    ///     app.quit();
    /// }
    /// ```
    #[inline]
    pub fn global() -> Option<Application> {
        APP.get().map(|inner| Application(Arc::clone(inner)))
    }

    /// Posts a closure to run on the event-loop thread.
    ///
    /// # Parameters
    ///
    /// - `f`: closure to run on the event-loop thread; runs in FIFO order with other
    ///   posted closures.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().unwrap();
    /// app.post_event(Box::new(|| println!("on event-loop thread")));
    /// ```
    #[inline]
    pub fn post_event(&self, f: Box<dyn FnOnce() + Send>) {
        self.0.event_loop.post(f);
    }

    /// Runs the event loop, blocking the calling thread until [`quit`](Self::quit) is called.
    ///
    /// # Panics
    ///
    /// If a posted closure panics, the panic propagates through `exec` to its caller.
    /// In normal use `exec` is called once on the main thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().unwrap();
    /// // post a quit event immediately so the loop exits right away in tests
    /// let app2 = Application::global().unwrap();
    /// app.post_event(Box::new(move || app2.quit()));
    /// app.exec();
    /// ```
    #[inline]
    pub fn exec(&self) {
        self.0.event_loop.run();
    }

    /// Stops the event loop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().unwrap();
    /// app.quit();
    /// ```
    #[inline]
    pub fn quit(&self) {
        self.0.event_loop.stop();
    }

    /// Returns a reference to the process-wide object tree.
    ///
    /// Lock the mutex before accessing the tree from any thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().unwrap();
    /// let _tree = app.object_tree().lock();
    /// ```
    #[inline]
    pub fn object_tree(&self) -> &Mutex<ObjectTree> {
        &self.0.object_tree
    }

    /// Returns a reference to the process-wide connection table.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().unwrap();
    /// let _table = app.connection_table();
    /// ```
    #[inline]
    pub fn connection_table(&self) -> &Arc<ConnectionTable> {
        &self.0.connection_table
    }

    /// Returns a reference to the process-wide event loop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().unwrap();
    /// let _el = app.event_loop();
    /// ```
    #[inline]
    pub fn event_loop(&self) -> &Arc<EventLoop> {
        &self.0.event_loop
    }

    /// Returns the [`ThreadId`](std::thread::ThreadId) of the thread that called
    /// [`Application::new`].
    ///
    /// The main-thread [`EventLoop`] is registered for this thread automatically.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().unwrap();
    /// assert_eq!(app.main_thread_id(), std::thread::current().id());
    /// ```
    #[inline]
    pub fn main_thread_id(&self) -> std::thread::ThreadId {
        self.0.main_thread_id
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        crate::global_tree::deregister();
    }
}

/// Error returned by [`try_with_tree`] and [`ObjectTreeExt`](crate::ObjectTreeExt) global
/// methods when no [`Application`] is currently live in this process.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::{try_with_tree, TreeAccessError};
///
/// assert_eq!(try_with_tree(|_| ()), Err(TreeAccessError));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("no Application is currently live")]
pub struct TreeAccessError;

/// Calls `f` with a shared reference to the active [`ObjectTree`] and returns
/// the result, or [`TreeAccessError`] if no [`Application`] is currently live.
///
/// # Parameters
///
/// - `f`: closure that receives a shared reference to the active tree.
///
/// # Errors
///
/// Returns [`TreeAccessError`] when no [`Application`] is live in this process.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::{try_with_tree, TreeAccessError};
///
/// assert_eq!(try_with_tree(|_tree| ()), Err(TreeAccessError));
/// ```
pub fn try_with_tree<R>(f: impl FnOnce(&ObjectTree) -> R) -> Result<R, TreeAccessError> {
    if !crate::global_tree::is_live() {
        return Err(TreeAccessError);
    }
    let guard = APP.get().ok_or(TreeAccessError)?.object_tree.lock();
    Ok(f(&guard))
}

#[cfg(test)]
mod tests {
    // Application singleton tests live in tests/application.rs (isolated binary)
    // to avoid OnceLock conflicts between test cases in the same process.
}
