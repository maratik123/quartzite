//! Object-layer snapshot type.
#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String};
#[cfg(feature = "std")]
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::value::Value;

/// A serializable snapshot of a single object's [`Stored`](crate::meta::PropertyFlag::Stored)
/// properties and signal-blocking state.
///
/// Produced by the capture functions in `quartzite-runtime::snapshot` and consumed by the
/// restore functions in the same module. The snapshot records the class name so that the
/// runtime factory can reconstruct the concrete type on restore, and persists
/// `signals_blocked` so the flag survives the round-trip.
///
/// # Examples
///
/// ```
/// use quartzite_core::snapshot::ObjectSnapshot;
/// use quartzite_core::value::Value;
///
/// let snap = ObjectSnapshot {
///     class_name: "MyObject".into(),
///     properties: [("count".into(), Value::Int(0))].into_iter().collect(),
///     signals_blocked: false,
/// };
/// assert_eq!(snap.class_name, "MyObject");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectSnapshot {
    /// The class name used to reconstruct the object via the runtime factory.
    pub class_name: String,
    /// The [`Stored`](crate::meta::PropertyFlag::Stored) property values keyed by name.
    pub properties: BTreeMap<String, Value>,
    /// Whether signal emission was blocked on the object at capture time.
    #[serde(default)]
    pub signals_blocked: bool,
}
