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

/// Re-export of [`quartzite_core`] for direct access to core types and as a
/// migration target when switching from this facade to the sub-crate directly.
pub use quartzite_core;

/// Re-export of [`quartzite_runtime`] for direct access to runtime types and
/// as a migration target when switching from this facade to the sub-crate directly.
pub use quartzite_runtime;

/// Curated set of commonly used quartzite types, re-exported for convenience.
///
/// Import with `use quartzite::prelude::*;` to bring the core object model,
/// signal types, and runtime into scope without needing to reference individual
/// sub-crates.
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
