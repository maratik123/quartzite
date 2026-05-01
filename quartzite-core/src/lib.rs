#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod id;
pub mod meta;
pub mod object_base;
pub mod receiver_guard;
pub mod signal;
pub mod traits;
pub mod value;

// --- Top-level re-exports ---

pub use id::{ConnectionId, ObjectId};
pub use meta::{
    EnumEntry, EnumMeta, MetaObject, MethodMeta, ParamMeta, PropertyFlags, PropertyMeta, SignalMeta,
};
pub use object_base::ObjectBase;
pub use receiver_guard::ReceiverGuard;
pub use signal::{ConnectionType, Signal};
pub use traits::{AsObject, Object, ObjectExt, SignalCallback};
pub use value::{CustomValue, FromValue, IntoValue, TypeError, Value};
