//! Core object model, signal/slot system, and reflection metadata for quartzite.
//!
//! This crate is `no_std`-compatible when the `std` feature is disabled (requires `alloc`).
//! All runtime-specific functionality (event loops, timers, thread pools) lives in
//! `quartzite-runtime`.
//!
//! # Feature flags
#![doc = document_features::document_features!()]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod args_to_values;
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod connect;
pub mod id;
pub mod meta;
pub mod object_base;
pub mod receiver_guard;
pub mod signal;
pub mod traits;
pub mod value;

/// Hidden module used by quartzite-macros generated code to access types
/// that are portable across `std` and `no_std + alloc` configurations.
#[doc(hidden)]
pub mod __macro {
    #[cfg(not(feature = "std"))]
    pub use alloc::sync::Arc;
    #[cfg(feature = "std")]
    pub use std::sync::Arc;
}

// --- Top-level re-exports ---

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub use connect::{SignalConnectionError, connect_signal_to_signal, connect_signals};
/// Re-exported solely for use by quartzite proc-macro generated code; not part of the public API.
#[doc(hidden)]
pub use enumflags2;
pub use id::{ConnectionId, ObjectId};
pub use meta::{
    EnumEntry, EnumMeta, MetaObject, MethodMeta, ParamMeta, PropertyFlag, PropertyFlags,
    PropertyMeta, SignalMeta,
};
pub use object_base::ObjectBase;
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub use parking_lot::Mutex;
pub use receiver_guard::ReceiverGuard;
pub use signal::{ConnectionType, Signal};
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub use signal::{
    DispatcherAlreadySet, QueuedDispatcher, queued_dispatcher, set_queued_dispatcher,
};
pub use traits::{AsObject, Object, ObjectExt, SignalCallback};
pub use value::{CustomValue, FromValue, IntoValue, TypeError, Value, WeakObjectRef};
