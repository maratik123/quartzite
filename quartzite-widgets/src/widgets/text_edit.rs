//! [`TextEdit`] — multi-line text editor widget.

use quartzite_core::{Signal, emit};
use quartzite_macros::{Extend, Object, object_impl};

use crate::{WidgetBase, widget_base::AsWidget};

/// A multi-line rich text editor.
///
/// # Examples
///
/// ```
/// use quartzite_core::{Object, Value};
/// use quartzite_widgets::TextEdit;
///
/// let edit = TextEdit::new();
/// assert_eq!(edit.read_property("plain_text"), Some(Value::String(String::new())));
/// assert_eq!(edit.meta_object().class_name, "TextEdit");
/// ```
#[derive(Extend, Object)]
pub struct TextEdit {
    #[base]
    widget_base: WidgetBase,
    /// Plain-text content of the editor.
    #[prop(notify = text_changed)]
    pub plain_text: String,
    /// Whether the editor is read-only.
    #[prop]
    pub read_only: bool,
    /// Emitted when the content changes; carries the new plain text.
    #[signal]
    pub text_changed: Signal<(String,)>,
}

impl TextEdit {
    /// Creates a new empty [`TextEdit`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::TextEdit;
    ///
    /// let edit = TextEdit::new();
    /// assert!(edit.plain_text.is_empty());
    /// assert!(!edit.read_only);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            widget_base: WidgetBase::new(),
            plain_text: String::new(),
            read_only: false,
            text_changed: Signal::default(),
        }
    }
}

impl Default for TextEdit {
    /// Returns a new empty `TextEdit`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::TextEdit;
    ///
    /// let edit = TextEdit::default();
    /// assert!(edit.plain_text.is_empty());
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[object_impl]
impl TextEdit {
    /// Sets the plain-text content, emitting `text_changed` if the value differs.
    ///
    /// Has no effect when `read_only` is `true`.
    ///
    /// # Parameters
    ///
    /// - `text`: new plain-text content.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::TextEdit;
    ///
    /// let mut edit = TextEdit::new();
    /// edit.set_plain_text("hello\nworld".into());
    /// assert_eq!(edit.plain_text, "hello\nworld");
    /// ```
    #[slot]
    pub fn set_plain_text(&mut self, text: String) {
        if !self.read_only && self.plain_text != text {
            self.plain_text = text;
            emit!(self.text_changed, &(self.plain_text.clone(),));
        }
    }

    /// Clears all content, emitting `text_changed` if the editor was non-empty.
    ///
    /// Has no effect when `read_only` is `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::TextEdit;
    ///
    /// let mut edit = TextEdit::new();
    /// edit.plain_text = "hello".into();
    /// edit.clear();
    /// assert!(edit.plain_text.is_empty());
    /// ```
    #[slot]
    pub fn clear(&mut self) {
        if !self.read_only && !self.plain_text.is_empty() {
            self.plain_text = String::new();
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
    fn class_name_is_text_edit() {
        let edit = TextEdit::new();
        assert_eq!(edit.meta_object().class_name, "TextEdit");
    }

    #[test]
    fn read_property_plain_text_empty() {
        let edit = TextEdit::new();
        assert_eq!(
            edit.read_property("plain_text"),
            Some(Value::String(String::new()))
        );
    }

    #[test]
    fn set_plain_text_emits_text_changed() {
        let mut edit = TextEdit::new();
        let received: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let recv2 = Arc::clone(&received);
        edit.text_changed.connect(move |args: &(String,)| {
            *recv2.lock().unwrap() = Some(args.0.clone());
        });
        edit.set_plain_text("hello\nworld".into());
        assert_eq!(edit.plain_text, "hello\nworld");
        assert_eq!(*received.lock().unwrap(), Some("hello\nworld".into()));
    }

    #[test]
    fn set_plain_text_no_emit_when_same() {
        let mut edit = TextEdit::new();
        edit.plain_text = "hello".into();
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.text_changed.connect(move |_: &(String,)| {
            *count2.lock().unwrap() += 1;
        });
        edit.set_plain_text("hello".into());
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn set_plain_text_no_emit_when_read_only() {
        let mut edit = TextEdit::new();
        edit.read_only = true;
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.text_changed.connect(move |_: &(String,)| {
            *count2.lock().unwrap() += 1;
        });
        edit.set_plain_text("hello".into());
        assert_eq!(edit.plain_text, "");
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn clear_emits_text_changed() {
        let mut edit = TextEdit::new();
        edit.plain_text = "some text".into();
        let received: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let recv2 = Arc::clone(&received);
        edit.text_changed.connect(move |args: &(String,)| {
            *recv2.lock().unwrap() = Some(args.0.clone());
        });
        edit.clear();
        assert!(edit.plain_text.is_empty());
        assert_eq!(*received.lock().unwrap(), Some(String::new()));
    }
}
