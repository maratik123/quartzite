//! Snapshot capture and restore for objects and object trees.
//!
//! This module provides three levels of granularity:
//!
//! 1. **Object layer** — [`capture_object`] / [`restore_object`] operate on a
//!    single [`Object`](quartzite_core::Object) and its
//!    [`Stored`](quartzite_core::meta::PropertyFlag::Stored) properties.
//!
//! 2. **Tree layer** — [`capture_tree`] / [`restore_tree`] snapshot an entire
//!    [`ObjectTree`](crate::ObjectTree) including parent/child relationships.
//!    Intra-tree [`WeakObjectRef`](quartzite_core::value::WeakObjectRef)
//!    payloads are remapped to the fresh [`ObjectId`](quartzite_core::ObjectId)s
//!    minted during restore.
//!
//! ## What is NOT preserved
//!
//! | State | After restore |
//! |---|---|
//! | Signal connections | Dropped (re-establish after restore) |
//! | `signals_blocked` flag | Reset to `false` |
//! | Non-`Stored` properties | Keep their default-constructed values |
//! | `Value::Custom` cross-refs | Opaque — not remapped |

pub mod object;
pub mod tree;

pub use object::{capture_object, restore_object};
pub use tree::{capture_tree, restore_tree};
