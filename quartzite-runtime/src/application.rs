//! Process-singleton `Application` and `ApplicationError`.
use std::sync::{Arc, Mutex, OnceLock};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    /// An `Application` instance already exists in this process.
    AlreadyExists,
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::AlreadyExists => write!(f, "Application already exists"),
        }
    }
}

impl std::error::Error for ApplicationError {}

struct ApplicationInner {
    /// `Mutex` (not `RwLock`) because `ObjectTree: Send` but not `Sync`.
    /// `RwLock<T>` requires `T: Send + Sync`; `Mutex<T>` only requires `T: Send`.
    object_tree: Mutex<ObjectTree>,
    event_loop: Arc<EventLoop>,
    connection_table: Arc<ConnectionTable>,
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
        let event_loop = Arc::new(EventLoop::new());
        let connection_table = ConnectionTable::new(Arc::clone(&event_loop));
        let inner = Arc::new(ApplicationInner {
            object_tree: Mutex::new(ObjectTree::new()),
            event_loop,
            connection_table,
        });

        APP.set(Arc::clone(&inner))
            .map_err(|_| ApplicationError::AlreadyExists)?;

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
    /// Panics if another thread is already inside [`exec`](Self::exec) for the same
    /// application — the underlying receiver mutex is already held. In normal use
    /// `exec` is called once on the main thread.
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
    /// let _tree = app.object_tree().lock().unwrap();
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
}

impl Drop for Application {
    fn drop(&mut self) {
        crate::global_tree::deregister();
    }
}

/// Calls `f` with a shared reference to the active [`ObjectTree`] and returns
/// the result wrapped in `Some`, or returns `None` if no [`Application`] is
/// currently live or if the tree mutex is poisoned.
///
/// # Parameters
///
/// - `f`: closure that receives a shared reference to the active tree.
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
    if !crate::global_tree::is_live() {
        return None;
    }
    let guard = APP.get()?.object_tree.lock().ok()?;
    Some(f(&guard))
}

#[cfg(test)]
mod tests {
    // Application singleton tests live in tests/application.rs (isolated binary)
    // to avoid OnceLock conflicts between test cases in the same process.
}
