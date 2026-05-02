#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![doc = document_features::document_features!()]
//!
//! ## Getting started
//!
//! Import the prelude to bring the most commonly used types into scope:
//!
//! ```rust
//! use quartzite::prelude::*;
//! ```
//!
//! The `quartzite-macros` crate also exports `MetaEnum` for enum reflection; it
//! is available directly from `quartzite_macros::MetaEnum` for users who need it.

/// Core object model, signals, and reflection types — re-exported from [`quartzite_core`].
///
/// Prefer `use quartzite::core::Foo` over adding a direct `quartzite-core` dependency.
pub mod core {
    pub use quartzite_core::*;
}

/// Application runtime, event loop, and object tree — re-exported from [`quartzite_runtime`].
///
/// Prefer `use quartzite::runtime::Foo` over adding a direct `quartzite-runtime` dependency.
pub mod runtime {
    pub use quartzite_runtime::*;
}

/// Curated set of types needed for typical usage — one glob covers a working import.
///
/// Use `use quartzite::prelude::*;` to get the object model, signal types, derive
/// macros, and runtime in one line. For production code that values legibility,
/// import from [`quartzite_core`] or [`quartzite_runtime`] explicitly instead.
pub mod prelude {
    // quartzite-core: object model
    pub use quartzite_core::{
        AsObject, ConnectionId, Object, ObjectBase, ObjectExt, ObjectId, SignalCallback, Value,
        WeakObjectRef,
    };
    // quartzite-core: signals
    pub use quartzite_core::signal::{ConnectionType, Signal};
    // quartzite-core: std-only dispatcher API
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub use quartzite_core::{
        DispatcherAlreadySet, QueuedDispatcher, queued_dispatcher, set_queued_dispatcher,
    };

    // quartzite-macros: derive macros
    // `Object` is re-exported as `DeriveObject` to avoid shadowing the `Object` trait above.
    pub use quartzite_macros::{Extend, Object as DeriveObject, object_impl};

    // quartzite-runtime
    pub use quartzite_runtime::{
        Application, ApplicationError, EventLoop, ObjectRef, ObjectTree, Timer, WeakRef,
    };
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn prelude_compiles() {
        let _: ObjectId = ObjectBase::new().id();
    }
}
