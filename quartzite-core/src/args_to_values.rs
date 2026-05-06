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
        Vec::new()
    }
}

impl<A: IntoValue + Clone + 'static> ArgsToValues for (A,) {
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![IntoValue::into_value(self.0.clone())]
    }
}

impl<A: IntoValue + Clone + 'static, B: IntoValue + Clone + 'static> ArgsToValues for (A, B) {
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![
            IntoValue::into_value(self.0.clone()),
            IntoValue::into_value(self.1.clone()),
        ]
    }
}

impl<A: IntoValue + Clone + 'static, B: IntoValue + Clone + 'static, C: IntoValue + Clone + 'static>
    ArgsToValues for (A, B, C)
{
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![
            IntoValue::into_value(self.0.clone()),
            IntoValue::into_value(self.1.clone()),
            IntoValue::into_value(self.2.clone()),
        ]
    }
}

impl<
    A: IntoValue + Clone + 'static,
    B: IntoValue + Clone + 'static,
    C: IntoValue + Clone + 'static,
    D: IntoValue + Clone + 'static,
> ArgsToValues for (A, B, C, D)
{
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![
            IntoValue::into_value(self.0.clone()),
            IntoValue::into_value(self.1.clone()),
            IntoValue::into_value(self.2.clone()),
            IntoValue::into_value(self.3.clone()),
        ]
    }
}

impl<
    A: IntoValue + Clone + 'static,
    B: IntoValue + Clone + 'static,
    C: IntoValue + Clone + 'static,
    D: IntoValue + Clone + 'static,
    E: IntoValue + Clone + 'static,
> ArgsToValues for (A, B, C, D, E)
{
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![
            IntoValue::into_value(self.0.clone()),
            IntoValue::into_value(self.1.clone()),
            IntoValue::into_value(self.2.clone()),
            IntoValue::into_value(self.3.clone()),
            IntoValue::into_value(self.4.clone()),
        ]
    }
}

impl<
    A: IntoValue + Clone + 'static,
    B: IntoValue + Clone + 'static,
    C: IntoValue + Clone + 'static,
    D: IntoValue + Clone + 'static,
    E: IntoValue + Clone + 'static,
    F: IntoValue + Clone + 'static,
> ArgsToValues for (A, B, C, D, E, F)
{
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![
            IntoValue::into_value(self.0.clone()),
            IntoValue::into_value(self.1.clone()),
            IntoValue::into_value(self.2.clone()),
            IntoValue::into_value(self.3.clone()),
            IntoValue::into_value(self.4.clone()),
            IntoValue::into_value(self.5.clone()),
        ]
    }
}

impl<
    A: IntoValue + Clone + 'static,
    B: IntoValue + Clone + 'static,
    C: IntoValue + Clone + 'static,
    D: IntoValue + Clone + 'static,
    E: IntoValue + Clone + 'static,
    F: IntoValue + Clone + 'static,
    G: IntoValue + Clone + 'static,
> ArgsToValues for (A, B, C, D, E, F, G)
{
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![
            IntoValue::into_value(self.0.clone()),
            IntoValue::into_value(self.1.clone()),
            IntoValue::into_value(self.2.clone()),
            IntoValue::into_value(self.3.clone()),
            IntoValue::into_value(self.4.clone()),
            IntoValue::into_value(self.5.clone()),
            IntoValue::into_value(self.6.clone()),
        ]
    }
}

impl<
    A: IntoValue + Clone + 'static,
    B: IntoValue + Clone + 'static,
    C: IntoValue + Clone + 'static,
    D: IntoValue + Clone + 'static,
    E: IntoValue + Clone + 'static,
    F: IntoValue + Clone + 'static,
    G: IntoValue + Clone + 'static,
    H: IntoValue + Clone + 'static,
> ArgsToValues for (A, B, C, D, E, F, G, H)
{
    #[inline]
    fn to_values(&self) -> Vec<Value> {
        vec![
            IntoValue::into_value(self.0.clone()),
            IntoValue::into_value(self.1.clone()),
            IntoValue::into_value(self.2.clone()),
            IntoValue::into_value(self.3.clone()),
            IntoValue::into_value(self.4.clone()),
            IntoValue::into_value(self.5.clone()),
            IntoValue::into_value(self.6.clone()),
            IntoValue::into_value(self.7.clone()),
        ]
    }
}

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
