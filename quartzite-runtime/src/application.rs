//! Process-singleton `Application` and `ApplicationError`.
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;
use quartzite_core::{
    ObjectBase,
    id::ConnectionId,
    meta::{MetaObject, MethodMeta},
    signal::ConnectionType,
    traits::SignalCallback,
    value::Value,
};

use crate::{
    application_builder::ApplicationBuilder, connection_table::ConnectionTable,
    event_loop::EventLoop, object_tree::ObjectTree,
};

/// Error returned by [`ApplicationBuilder::build`] when it fails.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::{Application, ApplicationError};
///
/// let _first = Application::builder().build().expect("first call succeeds");
/// match Application::builder().build() {
///     Err(ApplicationError::AlreadyExists) => {}
///     _ => panic!("second call must fail with AlreadyExists"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplicationError {
    /// An [`Application`] instance already exists in this process.
    #[error("Application already exists")]
    AlreadyExists,
}

struct ApplicationInner {
    /// Core object data (id, name, thread affinity, signal-block flag).
    base: ObjectBase,
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
/// Construct via [`Application::builder()`]; see [`ApplicationBuilder`] for details.
///
/// The class name `"Application"` is reserved for this framework-managed singleton.
/// User-created objects should not use `"Application"` as their `meta_object().class_name`.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::Application;
///
/// let app = Application::builder().build().expect("only one Application per process");
/// app.quit();
/// ```
pub struct Application(Arc<ApplicationInner>);

// ── Static MetaObject for Application ──────────────────────────────────────

static APP_METHODS: [MethodMeta; 1] = [MethodMeta::new("quit", &[], "()")];
static APP_META: MetaObject = MetaObject::new(
    "Application",
    &[],
    &[],
    &APP_METHODS,
    &[],
    quartzite_core::meta::noop_lookup_property,
    quartzite_core::meta::noop_lookup_signal,
    |name| {
        if name == "quit" {
            Some(APP_METHODS[0])
        } else {
            None
        }
    },
    quartzite_core::meta::noop_lookup_enum,
);

// ── AsObject + Object impls (b1 hand-rolled) ────────────────────────────────

impl quartzite_core::AsObject for Application {
    #[inline]
    fn object_base(&self) -> &ObjectBase {
        &self.0.base
    }

    /// Returns a mutable reference to the underlying [`ObjectBase`].
    ///
    /// # Panics
    ///
    /// Always panics. `Application` holds an `Arc<ApplicationInner>` (shared, not mutable).
    /// Mutating the base through a shared handle is not supported. Use
    /// `ObjectTree::rename` / `ObjectTree::clear_name` for name changes.
    fn object_base_mut(&mut self) -> &mut ObjectBase {
        panic!(
            "Application singleton's ObjectBase cannot be mutated through the shared handle; \
             use ObjectTree::rename for name changes"
        )
    }

    #[inline]
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

impl quartzite_core::Object for Application {
    #[inline]
    fn meta_object(&self) -> &'static MetaObject {
        &APP_META
    }

    fn read_property(&self, _name: &str) -> Option<Value> {
        None
    }

    fn write_property(&mut self, _name: &str, _val: Value) -> bool {
        false
    }

    fn invoke_method(&mut self, name: &str, args: &[Value]) -> Option<Value> {
        match name {
            "quit" => {
                if !args.is_empty() {
                    return None;
                }
                self.quit();
                Some(Value::Null)
            }
            _ => None,
        }
    }

    fn connect_signal(
        &mut self,
        _signal: &str,
        _callback: SignalCallback,
        _conn_type: ConnectionType,
    ) -> Option<ConnectionId> {
        None
    }

    fn emit_signal(&mut self, _signal: &str, _args: &[Value]) -> Option<()> {
        None
    }
}

// ── Application inherent methods ─────────────────────────────────────────────

impl Application {
    /// Returns a builder for constructing the [`Application`] singleton.
    ///
    /// Use [`ApplicationBuilder::build`] to install the singleton. The default builder
    /// produces a **tickless** application.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::builder().build().expect("only one Application per process");
    /// ```
    #[inline]
    pub const fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }

    /// Creates a new [`Application`] singleton with the tickless default.
    ///
    /// Shorthand for `Application::builder().build()`.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError::AlreadyExists`] if an [`Application`] has already been
    /// installed in this process. Only one [`Application`] may exist per process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::new().expect("only one Application per process");
    /// ```
    #[inline]
    pub fn new() -> Result<Self, ApplicationError> {
        Self::builder().build()
    }

    /// Internal: constructs the singleton from an already-configured `EventLoop`.
    ///
    /// Called by [`ApplicationBuilder::build`]. Not part of the public API.
    pub(crate) fn build_from(event_loop: EventLoop) -> Result<Self, ApplicationError> {
        let main_thread_id = std::thread::current().id();
        let event_loop = Arc::new(event_loop);
        let connection_table = ConnectionTable::new();
        let inner = Arc::new(ApplicationInner {
            base: ObjectBase::new(),
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
        // that is a caller bug; ignore it rather than failing build().
        let _ = Arc::clone(&inner.event_loop).install_for_current_thread();

        // Register ConnectionTable as the queued dispatcher. Ignore if already
        // set — only one Application can exist per process, so the only way
        // this can fail is if the dispatcher was set before build()
        // (e.g. in tests that call set_queued_dispatcher directly).
        let _ = quartzite_core::set_queued_dispatcher(
            Arc::clone(&inner.connection_table) as Arc<dyn quartzite_core::QueuedDispatcher>
        );

        // Install the process-wide factory. Ignore FactoryAlreadySet — same rationale
        // as the dispatcher above.
        let _ = crate::factory::ObjectFactory::install(crate::factory::ObjectFactory::new());

        // Mark the global tree as live so ObjectTreeExt::parent/children work.
        crate::global_tree::register();

        Ok(Self(inner))
    }

    /// Returns a handle to the global application, or `None` if it has not been installed yet.
    ///
    /// Calling [`Application::builder().build()`](ApplicationBuilder::build) is required
    /// before this returns `Some`.
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
    pub fn global() -> Option<Self> {
        APP.get().map(|inner| Self(Arc::clone(inner)))
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
    /// let app = Application::builder().build().unwrap();
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
    /// let app = Application::builder().build().unwrap();
    /// // Post a quit before exec() so the loop exits immediately.
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
    /// Safe to call from any context — closures, signal callbacks, cross-thread handlers —
    /// because it only requires a shared reference. Reflection-based invocation via
    /// `invoke_method("quit", &[])` also dispatches here.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::builder().build().unwrap();
    /// let app2 = Application::global().unwrap();
    /// app.post_event(Box::new(move || app2.quit()));
    /// app.exec();
    /// ```
    #[inline]
    pub fn quit(&self) {
        self.0.event_loop.request_stop();
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
    /// let app = Application::builder().build().unwrap();
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
    /// let app = Application::builder().build().unwrap();
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
    /// let app = Application::builder().build().unwrap();
    /// let _el = app.event_loop();
    /// ```
    #[inline]
    pub fn event_loop(&self) -> &Arc<EventLoop> {
        &self.0.event_loop
    }

    /// Returns the [`ThreadId`](std::thread::ThreadId) of the thread that called
    /// [`ApplicationBuilder::build`].
    ///
    /// The main-thread [`EventLoop`] is registered for this thread automatically.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::builder().build().unwrap();
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
