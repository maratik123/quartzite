//! Tree-layer snapshot types.
use serde::{Deserialize, Serialize};

use super::object::ObjectSnapshot;

/// A single node in a [`TreeSnapshot`], holding an [`ObjectSnapshot`] and its children.
///
/// # Examples
///
/// ```
/// use quartzite_core::snapshot::{ObjectNode, ObjectSnapshot};
///
/// let node = ObjectNode {
///     snapshot: ObjectSnapshot {
///         class_name: "Root".into(),
///         properties: Default::default(),
///         signals_blocked: false,
///     },
///     children: vec![],
///     object_id: 0,
/// };
/// assert!(node.children.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectNode {
    /// The snapshot of the object at this node.
    pub snapshot: ObjectSnapshot,
    /// Ordered child nodes.
    pub children: Vec<ObjectNode>,
    /// The original [`ObjectId`](crate::ObjectId) raw u64 of this object at capture time.
    ///
    /// Used by `quartzite_runtime::snapshot::restore_tree` to build an
    /// `old_id → new_id` remap table so that intra-tree `Value::Object`
    /// payloads are updated to the new IDs after restore.
    /// Set to `0` when the node was not captured as part of a tree snapshot.
    #[serde(default)]
    pub object_id: u64,
}

/// A serializable snapshot of an entire `ObjectTree`.
///
/// Produced by `quartzite_runtime::snapshot::capture_tree` and consumed by
/// `quartzite_runtime::snapshot::restore_tree`.
///
/// # Examples
///
/// ```
/// use quartzite_core::snapshot::{ObjectNode, ObjectSnapshot, TreeSnapshot, CURRENT_SCHEMA_VERSION};
///
/// let snap = TreeSnapshot {
///     schema_version: CURRENT_SCHEMA_VERSION,
///     root: ObjectNode {
///         snapshot: ObjectSnapshot { class_name: "Root".into(), properties: Default::default(), signals_blocked: false },
///         children: vec![],
///         object_id: 0,
///     },
/// };
/// assert_eq!(snap.schema_version, 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeSnapshot {
    /// Schema version. Used by [`TreeSnapshot::validate_version`] to reject
    /// payloads from a future (unsupported) version.
    pub schema_version: u32,
    /// The root node of the tree.
    pub root: ObjectNode,
}

impl TreeSnapshot {
    /// Validates that `self.schema_version` is at or below [`CURRENT_SCHEMA_VERSION`](crate::snapshot::CURRENT_SCHEMA_VERSION).
    ///
    /// # Errors
    ///
    /// Returns [`DeserializeError::UnsupportedVersion`](crate::snapshot::DeserializeError::UnsupportedVersion)
    /// when `schema_version` exceeds [`CURRENT_SCHEMA_VERSION`](crate::snapshot::CURRENT_SCHEMA_VERSION).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::snapshot::{DeserializeError, ObjectNode, ObjectSnapshot, TreeSnapshot, CURRENT_SCHEMA_VERSION};
    ///
    /// let snap = TreeSnapshot {
    ///     schema_version: CURRENT_SCHEMA_VERSION,
    ///     root: ObjectNode {
    ///         snapshot: ObjectSnapshot { class_name: "R".into(), properties: Default::default(), signals_blocked: false },
    ///         children: vec![],
    ///         object_id: 0,
    ///     },
    /// };
    /// assert!(snap.validate_version().is_ok());
    ///
    /// let bad = TreeSnapshot { schema_version: u32::MAX, ..snap };
    /// assert!(matches!(
    ///     bad.validate_version(),
    ///     Err(DeserializeError::UnsupportedVersion { found: u32::MAX, supported: 1 })
    /// ));
    /// ```
    pub fn validate_version(&self) -> Result<&Self, crate::snapshot::DeserializeError> {
        if self.schema_version > crate::snapshot::CURRENT_SCHEMA_VERSION {
            Err(crate::snapshot::DeserializeError::UnsupportedVersion {
                found: self.schema_version,
                supported: crate::snapshot::CURRENT_SCHEMA_VERSION,
            })
        } else {
            Ok(self)
        }
    }
}
