//! [`LineEdit`] — single-line text input widget.

use quartzite_core::{Signal, emit};
use quartzite_macros::{Extend, Object, object_impl};

use crate::{WidgetBase, widget_base::AsWidget};

/// A single-line text input field.
///
/// # Examples
///
/// ```
/// use quartzite_core::{Object, Value};
/// use quartzite_widgets::LineEdit;
///
/// let edit = LineEdit::new();
/// assert_eq!(edit.read_property("text"), Some(Value::String(String::new())));
/// assert_eq!(edit.meta_object().class_name, "LineEdit");
/// ```
#[derive(Extend, Object)]
pub struct LineEdit {
    #[base]
    widget_base: WidgetBase,
    /// Current text content.
    #[prop(notify = text_changed)]
    pub text: String,
    /// Placeholder text shown when the field is empty.
    #[prop]
    pub placeholder: String,
    /// Whether the field is read-only.
    #[prop]
    pub read_only: bool,
    /// Emitted when the text changes; carries the new text.
    #[signal]
    pub text_changed: Signal<(String,)>,
    /// Emitted when the user presses Enter/Return.
    #[signal]
    pub return_pressed: Signal<()>,
}

impl LineEdit {
    /// Creates a new empty `LineEdit`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::LineEdit;
    ///
    /// let edit = LineEdit::new();
    /// assert!(edit.text.is_empty());
    /// assert!(!edit.read_only);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            widget_base: WidgetBase::new(),
            text: String::new(),
            placeholder: String::new(),
            read_only: false,
            text_changed: Signal::default(),
            return_pressed: Signal::default(),
        }
    }
}

impl Default for LineEdit {
    /// Returns a new empty `LineEdit`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::LineEdit;
    ///
    /// let edit = LineEdit::default();
    /// assert!(edit.text.is_empty());
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[object_impl]
impl LineEdit {
    /// Sets the text, emitting `text_changed` if the value differs.
    ///
    /// Has no effect when `read_only` is `true`.
    ///
    /// # Parameters
    ///
    /// - `text`: new text content.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::LineEdit;
    ///
    /// let mut edit = LineEdit::new();
    /// edit.set_text("hello".into());
    /// assert_eq!(edit.text, "hello");
    /// ```
    #[slot]
    pub fn set_text(&mut self, text: String) {
        if !self.read_only && self.text != text {
            self.text = text;
            emit!(self.text_changed, &(self.text.clone(),));
        }
    }

    /// Clears the text, emitting `text_changed` if the field was non-empty.
    ///
    /// Has no effect when `read_only` is `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::LineEdit;
    ///
    /// let mut edit = LineEdit::new();
    /// edit.text = "hello".into();
    /// edit.clear();
    /// assert!(edit.text.is_empty());
    /// ```
    #[slot]
    pub fn clear(&mut self) {
        if !self.read_only && !self.text.is_empty() {
            self.text = String::new();
            emit!(self.text_changed, &(String::new(),));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use quartzite_core::{Object, Value};

    use super::*;

    #[test]
    fn class_name_is_line_edit() {
        let edit = LineEdit::new();
        assert_eq!(edit.meta_object().class_name, "LineEdit");
    }

    #[test]
    fn read_property_text_empty() {
        let edit = LineEdit::new();
        assert_eq!(
            edit.read_property("text"),
            Some(Value::String(String::new()))
        );
    }

    #[test]
    fn set_text_emits_text_changed() {
        let mut edit = LineEdit::new();
        let received: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let recv2 = Arc::clone(&received);
        edit.text_changed.connect(move |args: &(String,)| {
            *recv2.lock().unwrap() = Some(args.0.clone());
        });
        edit.set_text("hello".into());
        assert_eq!(edit.text, "hello");
        assert_eq!(*received.lock().unwrap(), Some("hello".into()));
    }

    #[test]
    fn set_text_no_emit_when_same() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.text_changed.connect(move |_: &(String,)| {
            *count2.lock().unwrap() += 1;
        });
        edit.set_text("hello".into());
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn set_text_no_emit_when_read_only() {
        let mut edit = LineEdit::new();
        edit.read_only = true;
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.text_changed.connect(move |_: &(String,)| {
            *count2.lock().unwrap() += 1;
        });
        edit.set_text("hello".into());
        assert_eq!(edit.text, "");
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn clear_emits_text_changed() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        let received: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let recv2 = Arc::clone(&received);
        edit.text_changed.connect(move |args: &(String,)| {
            *recv2.lock().unwrap() = Some(args.0.clone());
        });
        edit.clear();
        assert!(edit.text.is_empty());
        assert_eq!(*received.lock().unwrap(), Some(String::new()));
    }

    #[test]
    fn write_property_text_updates_and_emits() {
        let mut edit = LineEdit::new();
        let fired: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let fired2 = Arc::clone(&fired);
        edit.text_changed.connect(move |args: &(String,)| {
            *fired2.lock().unwrap() = Some(args.0.clone());
        });
        assert!(edit.write_property("text", Value::String("world".into())));
        assert_eq!(edit.text, "world");
        assert_eq!(*fired.lock().unwrap(), Some("world".into()));
    }
}
