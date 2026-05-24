//! Runtime layer for quartzite: event loop, object tree, connection table, timers, and threads.
//!
//! Entry point is [`Application::builder`](application::Application::builder), which returns an
//! [`ApplicationBuilder`] that installs the singleton via
//! [`build`](application_builder::ApplicationBuilder::build). Call
//! [`Application::exec`](application::Application::exec) on the main thread to run the event loop.
//!
//! ## Logging
//!
//! quartzite emits diagnostics via [`tracing`]. When the `std` feature is enabled (the default),
//! the `tracing/log` bridge is active: any [`log`](https://docs.rs/log)-compatible subscriber
//! (e.g. [`env_logger`](https://docs.rs/env_logger)) automatically receives quartzite's
//! diagnostics. Install the subscriber before creating an [`Application`]:
//!
//! ```rust,no_run
//! env_logger::init(); // or any other log-compatible subscriber
//! // … create Application via Application::new(), run event loop
//! ```

pub mod application;
pub mod application_builder;
pub mod connection_table;
pub mod event_loop;
pub mod factory;
pub(crate) mod global_tree;
pub mod loop_registry;
pub(crate) mod object_id;
pub mod object_ref;
pub mod object_tree;
pub mod object_tree_ext;
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod snapshot;
pub mod thread_pool;
pub mod timer;
pub mod timer_drivers;

pub use application::try_with_tree;
pub use application::{Application, ApplicationError, TreeAccessError};
pub use application_builder::ApplicationBuilder;
pub use connection_table::ConnectionTable;
pub use event_loop::EventLoop;
pub use factory::{FactoryAlreadySet, ObjectFactory};
pub use loop_registry::LoopAlreadyInstalled;
pub use object_ref::{ObjectRef, WeakRef};
pub use object_tree::ObjectTree;
pub use object_tree_ext::ObjectTreeExt;
pub use thread_pool::ThreadPool;
pub use timer::{AppDriver, PoolDriver, ThreadDriver, Timer, TimerConfig, TimerDriver};
