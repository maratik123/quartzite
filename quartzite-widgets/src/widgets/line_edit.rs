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
#[derive(Debug, Extend, Object)]
#[widget_view(variant = "LineEdit")]
pub struct LineEdit {
    /// Base widget — delegates geometry, state, focus policy, and object core.
    #[base]
    pub widget_base: WidgetBase,
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
    /// Byte offset of the insertion caret within `text`.
    ///
    /// Always in `0..=text.len()`. Managed by [`set_caret`](LineEdit::set_caret).
    pub caret: usize,
    /// Byte offset of the selection anchor within `text`, or `None` when
    /// there is no active selection.
    ///
    /// Always in `0..=text.len()` when `Some`. The selected range is
    /// `min(caret, anchor)..max(caret, anchor)`. Managed by
    /// [`set_selection_anchor`](LineEdit::set_selection_anchor).
    pub selection_anchor: Option<usize>,
    /// Emitted when the caret position or selection anchor changes.
    #[signal]
    pub selection_changed: Signal<()>,
}

impl LineEdit {
    /// Creates a new empty [`LineEdit`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::LineEdit;
    ///
    /// let edit = LineEdit::new();
    /// assert!(edit.text.is_empty());
    /// assert!(!edit.read_only);
    /// assert_eq!(edit.caret, 0);
    /// assert!(edit.selection_anchor.is_none());
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
            caret: 0,
            selection_anchor: None,
            selection_changed: Signal::default(),
        }
    }

    /// Returns the normalised selection byte range `(start, end)`, or `None`.
    ///
    /// Returns `None` when there is no selection anchor, or when the anchor
    /// equals the caret (zero-length selection).  When a range is returned it
    /// is always normalised: `start <= end`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::LineEdit;
    ///
    /// let mut edit = LineEdit::new();
    /// edit.text = "hello".into();
    /// edit.caret = 2;
    /// edit.selection_anchor = Some(5);
    /// assert_eq!(edit.selection_range(), Some((2, 5)));
    ///
    /// // Zero-length selection returns None.
    /// edit.selection_anchor = Some(2);
    /// assert_eq!(edit.selection_range(), None);
    /// ```
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        if anchor == self.caret {
            None
        } else {
            Some((self.caret.min(anchor), self.caret.max(anchor)))
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
    /// assert_eq!(edit.caret, 0);
    /// assert!(edit.selection_anchor.is_none());
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

    /// Moves the caret to `caret`, clamped to `0..=text.len()`.
    ///
    /// Emits `selection_changed` when the resolved state (caret position **and**
    /// selection range) actually changes.  No-op when the widget is read-only or
    /// when the new resolved state equals the existing one.
    ///
    /// # Parameters
    ///
    /// - `caret`: desired byte offset of the insertion caret.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::LineEdit;
    ///
    /// let mut edit = LineEdit::new();
    /// edit.text = "hello".into();
    /// edit.set_caret(100); // clamps to text.len()
    /// assert_eq!(edit.caret, 5);
    /// ```
    #[slot]
    pub fn set_caret(&mut self, caret: usize) {
        if self.read_only {
            return;
        }
        let clamped = caret.min(self.text.len());
        if clamped == self.caret {
            return;
        }
        self.caret = clamped;
        emit!(self.selection_changed, &());
    }
}

impl LineEdit {
    /// Sets the selection anchor to `anchor`, clamped to `0..=text.len()`.
    ///
    /// `None` clears the selection anchor (no active selection).  Emits
    /// `selection_changed` when the resolved state changes.  No-op when the
    /// widget is read-only or when the new resolved state equals the existing one.
    ///
    /// Not a `#[slot]` because `Option<usize>` does not implement
    /// [`quartzite_core::value::FromValue`] and cannot participate in dynamic
    /// invocation via [`quartzite_core::Object::invoke_method`].
    ///
    /// # Parameters
    ///
    /// - `anchor`: new anchor byte offset, or `None` to clear the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::LineEdit;
    ///
    /// let mut edit = LineEdit::new();
    /// edit.text = "hello".into();
    /// edit.set_selection_anchor(Some(3));
    /// assert_eq!(edit.selection_anchor, Some(3));
    /// edit.set_selection_anchor(None);
    /// assert!(edit.selection_anchor.is_none());
    /// ```
    pub fn set_selection_anchor(&mut self, anchor: Option<usize>) {
        if self.read_only {
            return;
        }
        let clamped = anchor.map(|a| a.min(self.text.len()));
        if clamped == self.selection_anchor {
            return;
        }
        self.selection_anchor = clamped;
        emit!(self.selection_changed, &());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;

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
            *recv2.lock() = Some(args.0.clone());
        });
        edit.set_text("hello".into());
        assert_eq!(edit.text, "hello");
        assert_eq!(*received.lock(), Some("hello".into()));
    }

    #[test]
    fn set_text_no_emit_when_same() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.text_changed.connect(move |_: &(String,)| {
            *count2.lock() += 1;
        });
        edit.set_text("hello".into());
        assert_eq!(*count.lock(), 0);
    }

    #[test]
    fn set_text_no_emit_when_read_only() {
        let mut edit = LineEdit::new();
        edit.read_only = true;
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.text_changed.connect(move |_: &(String,)| {
            *count2.lock() += 1;
        });
        edit.set_text("hello".into());
        assert_eq!(edit.text, "");
        assert_eq!(*count.lock(), 0);
    }

    #[test]
    fn clear_emits_text_changed() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        let received: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let recv2 = Arc::clone(&received);
        edit.text_changed.connect(move |args: &(String,)| {
            *recv2.lock() = Some(args.0.clone());
        });
        edit.clear();
        assert!(edit.text.is_empty());
        assert_eq!(*received.lock(), Some(String::new()));
    }

    #[test]
    fn write_property_text_updates_and_emits() {
        let mut edit = LineEdit::new();
        let fired: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let fired2 = Arc::clone(&fired);
        edit.text_changed.connect(move |args: &(String,)| {
            *fired2.lock() = Some(args.0.clone());
        });
        assert!(edit.write_property("text", Value::String("world".into())));
        assert_eq!(edit.text, "world");
        assert_eq!(*fired.lock(), Some("world".into()));
    }

    #[test]
    fn widget_view_returns_line_edit_variant() {
        let edit = LineEdit::new();
        assert!(matches!(edit.widget_view(), crate::WidgetView::LineEdit(_)));
    }

    // ── Caret + selection field tests ─────────────────────────────────────

    #[test]
    fn default_caret_is_zero_and_anchor_is_none() {
        let edit = LineEdit::new();
        assert_eq!(edit.caret, 0);
        assert!(edit.selection_anchor.is_none());
    }

    #[test]
    fn set_caret_clamps_to_text_len() {
        let mut edit = LineEdit::new();
        edit.text = "abc".into();
        edit.set_caret(999);
        assert_eq!(edit.caret, 3);
    }

    #[test]
    fn set_caret_no_emit_when_unchanged() {
        let mut edit = LineEdit::new();
        edit.text = "abc".into();
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.selection_changed.connect(move |(): &()| {
            *count2.lock() += 1;
        });
        // First call: changes caret 0 → 2, emits once.
        edit.set_caret(2);
        assert_eq!(*count.lock(), 1);
        // Second call: caret is already 2, should not emit again.
        edit.set_caret(2);
        assert_eq!(*count.lock(), 1);
    }

    #[test]
    fn set_caret_no_emit_when_read_only() {
        let mut edit = LineEdit::new();
        edit.text = "abc".into();
        edit.read_only = true;
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.selection_changed.connect(move |(): &()| {
            *count2.lock() += 1;
        });
        edit.set_caret(2);
        // Field must not change and signal must not fire.
        assert_eq!(edit.caret, 0);
        assert_eq!(*count.lock(), 0);
    }

    #[test]
    fn set_selection_anchor_some_then_none_emits_twice() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.selection_changed.connect(move |(): &()| {
            *count2.lock() += 1;
        });
        edit.set_selection_anchor(Some(3));
        assert_eq!(*count.lock(), 1);
        edit.set_selection_anchor(None);
        assert_eq!(*count.lock(), 2);
    }

    #[test]
    fn selection_range_returns_normalised_pair() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        edit.caret = 2;
        edit.selection_anchor = Some(5);
        assert_eq!(edit.selection_range(), Some((2, 5)));
    }

    #[test]
    fn selection_range_normalises_reversed() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        edit.caret = 5;
        edit.selection_anchor = Some(2);
        assert_eq!(edit.selection_range(), Some((2, 5)));
    }

    #[test]
    fn selection_range_none_when_zero_length() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        edit.caret = 3;
        edit.selection_anchor = Some(3);
        assert_eq!(edit.selection_range(), None);
    }

    #[test]
    fn selection_range_none_when_anchor_none() {
        let mut edit = LineEdit::new();
        edit.text = "hello".into();
        edit.caret = 2;
        edit.selection_anchor = None;
        assert_eq!(edit.selection_range(), None);
    }

    #[test]
    fn set_selection_anchor_clamps_to_text_len() {
        let mut edit = LineEdit::new();
        edit.text = "abc".into();
        edit.set_selection_anchor(Some(999));
        assert_eq!(edit.selection_anchor, Some(3));
    }

    #[test]
    fn set_selection_anchor_no_emit_when_read_only() {
        let mut edit = LineEdit::new();
        edit.text = "abc".into();
        edit.read_only = true;
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        edit.selection_changed.connect(move |(): &()| {
            *count2.lock() += 1;
        });
        edit.set_selection_anchor(Some(2));
        assert!(edit.selection_anchor.is_none());
        assert_eq!(*count.lock(), 0);
    }
}
