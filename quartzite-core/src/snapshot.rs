//! Serialization snapshot types for the quartzite object system.
//!
//! ## Serialization
//!
//! This module is the entry point for the quartzite snapshot layer. It provides
//! three levels of granularity:
//!
//! 1. **Property layer** — [`Value`](crate::value::Value) itself implements `serde::Serialize` /
//!    `serde::Deserialize` (enabled by the `serde` cargo feature). Any
//!    `serde`-compatible backend can be used.
//!
//! 2. **Object layer** — [`ObjectSnapshot`] captures all
//!    [`Stored`](crate::meta::PropertyFlag::Stored) properties of a single
//!    object (produced by `quartzite_runtime::snapshot::capture_object`,
//!    restored by `quartzite_runtime::snapshot::restore_object`).
//!
//! 3. **Tree layer** — [`TreeSnapshot`] captures an entire
//!    `ObjectTree` including parent/child
//!    relationships (produced by `quartzite_runtime::snapshot::capture_tree`,
//!    restored by `quartzite_runtime::snapshot::restore_tree`).
//!
//! The `signals_blocked` flag on each object **is** serialized and preserved
//! across restore — an object whose signals were blocked before capture will
//! have signals blocked after restore.
//!
//! ## Transient state — what is NOT serialized
//!
//! | State | Behaviour on restore |
//! |---|---|
//! | `ConnectionTable` entries (signal connections) | **Dropped.** Connections hold runtime closures with no portable representation. Caller re-establishes them after restore. |
//! | Non-`Stored` properties | **Skipped.** Properties without [`PropertyFlag::Stored`](crate::meta::PropertyFlag::Stored) are not included in the snapshot. |
//!
//! ## `Value::Custom` round-trip
//!
//! [`Value::Custom`](crate::value::Value::Custom) payloads are round-tripped via
//! [`typetag`](https://crates.io/crates/typetag). Every concrete
//! [`CustomValue`](crate::value::CustomValue) implementation that should survive
//! serialization must be annotated:
//!
//! ```ignore
//! #[derive(serde::Serialize, serde::Deserialize)]
//! struct MyPayload { v: i64 }
//!
//! #[typetag::serde]
//! impl quartzite_core::value::CustomValue for MyPayload {
//!     // ...
//! }
//! ```
//!
//! Without the annotation, the concrete type cannot be deserialized (the runtime
//! registry has no entry for it). Types that do not opt in are left out of the
//! round-trip.
//!
//! ## Signal reconnection (v2)
//!
//! Re-establishing signal connections after restore is deferred to a future
//! version. When it lands, the runtime reflection system (`#[meta_method]`-annotated
//! slots) will be used to reconnect by name. Tracked in a separate issue once this
//! layer ships.

pub mod object;
pub mod tree;

pub use object::ObjectSnapshot;
pub use tree::{ObjectNode, TreeSnapshot};

/// Schema version written into every [`TreeSnapshot`].
///
/// Deserializing a payload with a higher version returns
/// [`DeserializeError::UnsupportedVersion`].
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Errors that can occur while serializing a snapshot.
///
/// # Examples
///
/// ```
/// use quartzite_core::snapshot::SerializeError;
///
/// let err = SerializeError::PropertyMissing {
///     class_name: "MyObj".into(),
///     property: "count".into(),
/// };
/// assert!(err.to_string().contains("MyObj"));
/// assert!(err.to_string().contains("count"));
///
/// let err2 = SerializeError::ObjectNotInTree { id: 42 };
/// assert!(err2.to_string().contains("42"));
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum SerializeError {
    /// `read_property` returned `None` for a property that is listed as
    /// [`Stored`](crate::meta::PropertyFlag::Stored) in the object's meta.
    ///
    /// This indicates a meta-system invariant violation — the meta declared the
    /// property as readable and stored, but the object did not return a value
    /// for it.
    #[error(
        "object '{class_name}' has no value for stored property '{property}' (meta invariant violated)"
    )]
    PropertyMissing {
        /// The class name of the object that produced the error.
        class_name: String,
        /// The property name that was expected but missing.
        property: String,
    },

    /// The requested [`ObjectId`](crate::ObjectId) was not found in the tree during capture.
    ///
    /// This occurs when `capture_tree` is called with an [`ObjectId`](crate::ObjectId) that is
    /// not present in the provided `ObjectTree`.
    #[error("object id {id} is not present in the tree")]
    ObjectNotInTree {
        /// The raw `u64` of the [`ObjectId`](crate::ObjectId) that was not found.
        id: u64,
    },
}

/// Errors that can occur while deserializing a snapshot.
///
/// # Examples
///
/// ```
/// use quartzite_core::snapshot::{DeserializeError, CURRENT_SCHEMA_VERSION};
///
/// let err = DeserializeError::UnsupportedVersion { found: 99, supported: CURRENT_SCHEMA_VERSION };
/// assert!(err.to_string().contains("99"));
///
/// let err2 = DeserializeError::UnknownClass { name: "Foo".into() };
/// assert!(err2.to_string().contains("Foo"));
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum DeserializeError {
    /// The snapshot was produced by a newer version of the schema.
    #[error("unsupported schema version {found} (supported up to {supported})")]
    UnsupportedVersion {
        /// The version found in the payload.
        found: u32,
        /// The maximum [`u32`] version this enum variant signals as supported; equal to [`CURRENT_SCHEMA_VERSION`].
        supported: u32,
    },

    /// No factory constructor is registered for the class name stored in the snapshot.
    #[error("unknown class '{name}' — register a constructor in ObjectFactory before restoring")]
    UnknownClass {
        /// The class name that was not found in the factory registry.
        name: String,
    },

    /// The process-wide `ObjectFactory` has not been installed.
    ///
    /// Call `ObjectFactory::install` (or create an `Application`) before restoring.
    #[error(
        "ObjectFactory is not installed — call ObjectFactory::install before restoring a snapshot"
    )]
    FactoryMissing,

    /// `write_property` returned `false` for a property in the snapshot.
    ///
    /// This typically means the serialized value's type does not match the
    /// property's expected type, or the property is read-only.
    #[error(
        "writing property '{property}' on '{class_name}' was rejected (type mismatch or read-only)"
    )]
    WriteRejected {
        /// The class name of the object whose property write was rejected.
        class_name: String,
        /// The property name that was rejected.
        property: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeMap;
    #[cfg(feature = "std")]
    use std::collections::BTreeMap;

    #[test]
    fn validate_version_ok() {
        let snap = TreeSnapshot {
            schema_version: CURRENT_SCHEMA_VERSION,
            root: ObjectNode {
                snapshot: ObjectSnapshot {
                    class_name: "R".into(),
                    properties: BTreeMap::default(),
                    signals_blocked: false,
                },
                children: vec![],
                object_id: 0,
            },
        };
        assert!(snap.validate_version().is_ok());
    }

    #[test]
    fn validate_version_rejects_future() {
        let snap = TreeSnapshot {
            schema_version: u32::MAX,
            root: ObjectNode {
                snapshot: ObjectSnapshot {
                    class_name: "R".into(),
                    properties: BTreeMap::default(),
                    signals_blocked: false,
                },
                children: vec![],
                object_id: 0,
            },
        };
        assert_eq!(
            snap.validate_version(),
            Err(DeserializeError::UnsupportedVersion {
                found: u32::MAX,
                supported: CURRENT_SCHEMA_VERSION,
            })
        );
    }

    #[test]
    fn serialize_error_display() {
        let err = SerializeError::PropertyMissing {
            class_name: "MyClass".into(),
            property: "myProp".into(),
        };
        let s = err.to_string();
        assert!(s.contains("MyClass"), "display: {s}");
        assert!(s.contains("myProp"), "display: {s}");
    }

    #[test]
    fn serialize_error_display_object_not_in_tree() {
        let err = SerializeError::ObjectNotInTree { id: 42 };
        assert!(err.to_string().contains("42"));
    }

    #[test]
    fn deserialize_error_display_unsupported_version() {
        let err = DeserializeError::UnsupportedVersion {
            found: 99,
            supported: CURRENT_SCHEMA_VERSION,
        };
        let s = err.to_string();
        assert!(s.contains("99"), "display: {s}");
        assert!(s.contains('1'), "display: {s}");
    }

    #[test]
    fn deserialize_error_display_unknown_class() {
        let err = DeserializeError::UnknownClass {
            name: "FooBar".into(),
        };
        assert!(err.to_string().contains("FooBar"));
    }

    #[test]
    fn deserialize_error_display_factory_missing() {
        let err = DeserializeError::FactoryMissing;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn deserialize_error_display_write_rejected() {
        let err = DeserializeError::WriteRejected {
            class_name: "Cls".into(),
            property: "prop".into(),
        };
        let s = err.to_string();
        assert!(s.contains("Cls"), "display: {s}");
        assert!(s.contains("prop"), "display: {s}");
    }
}
