//! [`Label`] — static text widget.

use quartzite_macros::{Extend, Object, object_impl};

use crate::{Alignment, WidgetBase, widget_base::AsWidget};

/// A widget that displays a single line of static text.
///
/// # Examples
///
/// ```
/// use quartzite_core::{Object, Value};
/// use quartzite_widgets::{Alignment, Label};
///
/// let label = Label::new("hello".into());
/// assert_eq!(label.read_property("text"), Some(Value::String("hello".into())));
/// assert_eq!(label.meta_object().class_name, "Label");
/// ```
#[derive(Extend, Object)]
#[widget_view(variant = "Label")]
pub struct Label {
    #[base]
    widget_base: WidgetBase,
    /// Text displayed by this label.
    #[prop]
    pub text: String,
    /// Horizontal and vertical alignment of the text.
    #[prop]
    pub alignment: Alignment,
}

impl Label {
    /// Creates a [`Label`] with the given `text` and default (left) alignment.
    ///
    /// # Parameters
    ///
    /// - `text`: initial display text.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Label;
    ///
    /// let label = Label::new("Status: OK".into());
    /// assert_eq!(label.text, "Status: OK");
    /// ```
    #[inline]
    pub fn new(text: String) -> Self {
        Self {
            widget_base: WidgetBase::new(),
            text,
            alignment: Alignment::default(),
        }
    }
}

#[object_impl]
impl Label {}

#[cfg(test)]
mod tests {
    use quartzite_core::{Object, Value};

    use super::*;

    #[test]
    fn class_name_is_label() {
        let label = Label::new("hello".into());
        assert_eq!(label.meta_object().class_name, "Label");
    }

    #[test]
    fn read_property_text() {
        let label = Label::new("hello".into());
        assert_eq!(
            label.read_property("text"),
            Some(Value::String("hello".into()))
        );
    }

    #[test]
    fn write_property_text() {
        let mut label = Label::new("hello".into());
        assert!(label.write_property("text", Value::String("world".into())));
        assert_eq!(label.text, "world");
    }

    #[test]
    fn read_property_alignment() {
        use quartzite_core::FromValue;
        let label = Label::new("x".into());
        let v = label
            .read_property("alignment")
            .expect("alignment property missing");
        assert_eq!(Alignment::from_value(v), Ok(Alignment::Left));
    }

    #[test]
    fn widget_view_returns_label_variant() {
        let label = Label::new("hi".into());
        assert!(matches!(label.widget_view(), crate::WidgetView::Label(_)));
    }
}
