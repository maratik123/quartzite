//! Internal helper trait for converting signal argument tuples to `Vec<Value>`.
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::value::{IntoValue, Value};

/// Internal helper: converts a signal argument tuple to a `Vec<Value>`.
///
/// Implemented for tuple arities 0–8. Used by [`connect_signals`] to forward
/// typed signal arguments through the dynamic `emit_signal` path.
///
/// [`connect_signals`]: crate::connect::connect_signals
#[doc(hidden)]
pub trait ArgsToValues: 'static {
    /// Converts this argument tuple to an owned `Vec<Value>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::args_to_values::ArgsToValues;
    ///
    /// let v = (42i32,).to_values();
    /// assert_eq!(v.len(), 1);
    /// ```
    fn to_values(&self) -> Vec<Value>;
}

impl ArgsToValues for () {
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![]
    }
}

macro_rules! impl_args_to_values {
    ($($T:ident: $idx:tt),+) => {
        impl<$($T: IntoValue + Clone + 'static),+> ArgsToValues for ($($T,)+) {
            #[inline]
            fn to_values(&self) -> Vec<Value> {
                vec![$(IntoValue::into_value(self.$idx.clone()),)+]
            }
        }
    };
}

impl_args_to_values!(A: 0);
impl_args_to_values!(A: 0, B: 1);
impl_args_to_values!(A: 0, B: 1, C: 2);
impl_args_to_values!(A: 0, B: 1, C: 2, D: 3);
impl_args_to_values!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_args_to_values!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_args_to_values!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_args_to_values!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn unit_produces_empty_vec() {
        let v = ().to_values();
        assert!(v.is_empty());
    }

    #[test]
    fn single_elem_tuple_produces_one_value() {
        let v = (42i32,).to_values();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], Value::Int(42));
    }

    #[test]
    fn two_elem_tuple_produces_two_values() {
        let v = (1i32, true).to_values();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], Value::Int(1));
        assert_eq!(v[1], Value::Bool(true));
    }

    #[test]
    fn borrows_do_not_move_original() {
        let args = (7i32,);
        let v = args.to_values();
        // args is still accessible after to_values() (takes &self)
        assert_eq!(args.0, 7);
        assert_eq!(v.len(), 1);
    }
}
