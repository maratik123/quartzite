#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
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
//! `MetaEnum` for enum reflection is available via [`macros::MetaEnum`] (requires the `derive`
//! feature, enabled by default).

/// Core object model, signals, and reflection types — re-exported from [`quartzite_core`].
///
/// Prefer `use quartzite::core::Foo` over adding a direct `quartzite-core` dependency.
pub mod core {
    pub use quartzite_core::*;
}

/// Derive macros and attribute macros — re-exported from [`quartzite_macros`].
///
/// Prefer `use quartzite::macros::MetaEnum` over adding a direct `quartzite-macros` dependency.
/// Most macros (`Extend`, `Object`, `object_impl`, `object_part`) are already included in [`prelude`].
///
/// Requires the `derive` feature (enabled by default).
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub mod macros {
    pub use quartzite_macros::*;
}

/// Application runtime, event loop, and object tree — re-exported from [`quartzite_runtime`].
///
/// Prefer `use quartzite::runtime::Foo` over adding a direct `quartzite-runtime` dependency.
pub mod runtime {
    pub use quartzite_runtime::*;
}

/// Geometry primitives — re-exported from [`quartzite_geometry`].
///
/// Provides integer (`Point`, `Size`, `Rect`, `Margins`) and float (`PointF`, `SizeF`, `RectF`) types.
pub mod geometry {
    pub use quartzite_geometry::*;
}

/// Event model — re-exported from [`quartzite_events`].
///
/// Provides `MouseEvent`, `KeyEvent`, `ResizeEvent`, `CloseEvent`, `TimerEvent`, `EventFilter`, and `EventType`.
pub mod events {
    pub use quartzite_events::*;
}

/// Curated set of types needed for typical usage — one glob covers a working import.
///
/// Use `use quartzite::prelude::*;` to get the object model, signal types, derive
/// macros, and runtime in one line. For production code that values legibility,
/// import from [`core`], [`macros`], or [`runtime`] explicitly instead.
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

    // quartzite-macros: derive macros (requires derive feature)
    // `Object` is re-exported as `DeriveObject` to avoid shadowing the `Object` trait above.
    #[cfg(feature = "derive")]
    #[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
    pub use quartzite_macros::{Extend, Object as DeriveObject, object_impl, object_part};

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
