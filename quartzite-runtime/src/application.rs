//! Process-singleton `Application` and `ApplicationError`.
use std::sync::{Arc, Mutex, OnceLock};

use crate::{connection_table::ConnectionTable, event_loop::EventLoop, object_tree::ObjectTree};

/// Error returned by [`Application::new`] when it fails.
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
    /// Mutex (not RwLock) because `ObjectTree: Send` but not `Sync`.
    /// `RwLock<T>` requires `T: Send + Sync`; `Mutex<T>` only requires `T: Send`.
    object_tree: Mutex<ObjectTree>,
    event_loop: Arc<EventLoop>,
    connection_table: Arc<ConnectionTable>,
}

static APP: OnceLock<Arc<ApplicationInner>> = OnceLock::new();

/// Singleton entry point for the quartzite runtime.
///
/// Owns the `ObjectTree` and `EventLoop`. Creates the `ConnectionTable` and
/// installs it as the process-wide `QueuedDispatcher`.
pub struct Application(Arc<ApplicationInner>);

impl Application {
    /// Create the application. Returns `Err(AlreadyExists)` if called more than once.
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

        Ok(Application(inner))
    }

    /// Access the global application without consuming it.
    ///
    /// Returns `None` if [`Application::new`] has not been called yet.
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

    /// Post a closure to run on the event-loop thread.
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

    /// Run the event loop, blocking the calling thread until `quit()` is called.
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

    /// Stop the event loop.
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

#[cfg(test)]
mod tests {
    // Application singleton tests live in tests/application.rs (isolated binary)
    // to avoid OnceLock conflicts between test cases in the same process.
}
