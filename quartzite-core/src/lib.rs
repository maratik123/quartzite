#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = document_features::document_features!()]

#[cfg(not(feature = "std"))]
extern crate alloc;

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

pub use id::{ConnectionId, ObjectId};
pub use meta::{
    EnumEntry, EnumMeta, MetaObject, MethodMeta, ParamMeta, PropertyFlags, PropertyMeta, SignalMeta,
};
pub use object_base::ObjectBase;
pub use receiver_guard::ReceiverGuard;
pub use signal::{ConnectionType, Signal};
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub use signal::{
    DispatcherAlreadySet, QueuedDispatcher, queued_dispatcher, set_queued_dispatcher,
};
pub use traits::{AsObject, Object, ObjectExt, SignalCallback};
pub use value::{CustomValue, FromValue, IntoValue, TypeError, Value, WeakObjectRef};
