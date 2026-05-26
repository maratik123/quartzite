//! [`Button`] — clickable push button with optional toggle behaviour.

use quartzite_core::{Signal, emit};
use quartzite_macros::{Extend, Object, object_impl};

use crate::{WidgetBase, widget_base::AsWidget};

/// A clickable push button.
///
/// When `checkable` is `false` (the default), each `click()` emits `clicked(false)`.
/// When `checkable` is `true`, each `click()` toggles `checked` and emits
/// `toggled(new_checked)` followed by `clicked(new_checked)`.
///
/// # Examples
///
/// ```
/// use quartzite_core::{Object, Value};
/// use quartzite_widgets::Button;
///
/// let btn = Button::new("OK".into());
/// assert_eq!(btn.read_property("text"), Some(Value::String("OK".into())));
/// assert_eq!(btn.meta_object().class_name, "Button");
/// ```
#[derive(Extend, Object)]
#[widget_view(variant = "Button")]
pub struct Button {
    #[base]
    widget_base: WidgetBase,
    /// Display text of the button.
    #[property(notify = text_changed)]
    pub text: String,
    /// Whether this button behaves as a toggle (checkable).
    #[property]
    pub checkable: bool,
    /// Current checked state; only meaningful when `checkable` is `true`.
    #[property]
    pub checked: bool,
    /// Emitted when the text changes; carries the new text.
    #[signal]
    pub text_changed: Signal<(String,)>,
    /// Emitted on click; carries `checked` (or `false` for non-checkable buttons).
    #[signal]
    pub clicked: Signal<(bool,)>,
    /// Emitted when `checked` changes (only for checkable buttons).
    #[signal]
    pub toggled: Signal<(bool,)>,
}

impl Button {
    /// Creates a [`Button`] with the given `text`.
    ///
    /// # Parameters
    ///
    /// - `text`: initial display text.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Button;
    ///
    /// let btn = Button::new("Cancel".into());
    /// assert_eq!(btn.text, "Cancel");
    /// assert!(!btn.checkable);
    /// ```
    #[inline]
    pub fn new(text: String) -> Self {
        Self {
            widget_base: WidgetBase::new(),
            text,
            checkable: false,
            checked: false,
            text_changed: Signal::default(),
            clicked: Signal::default(),
            toggled: Signal::default(),
        }
    }
}

#[object_impl]
impl Button {
    /// Simulates a button click.
    ///
    /// For non-checkable buttons, emits `clicked(false)`.
    /// For checkable buttons, toggles `checked`, then emits `toggled(new_checked)`
    /// and `clicked(new_checked)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Button;
    ///
    /// let mut btn = Button::new("OK".into());
    /// btn.click(); // emits clicked(false)
    /// ```
    #[slot]
    pub fn click(&mut self) {
        if self.checkable {
            let new_checked = !self.checked;
            self.checked = new_checked;
            emit!(self.toggled, &(new_checked,));
            emit!(self.clicked, &(new_checked,));
        } else {
            emit!(self.clicked, &(false,));
        }
    }

    /// Sets the button text, emitting `text_changed` if the value differs.
    ///
    /// # Parameters
    ///
    /// - `text`: new display text.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Button;
    ///
    /// let mut btn = Button::new("OK".into());
    /// btn.set_text("Cancel".into());
    /// assert_eq!(btn.text, "Cancel");
    /// ```
    #[slot]
    pub fn set_text(&mut self, text: String) {
        if self.text != text {
            self.text = text;
            emit!(self.text_changed, &(self.text.clone(),));
        }
    }

    /// Sets the checked state, emitting `toggled` if the value changes (only when checkable).
    ///
    /// # Parameters
    ///
    /// - `checked`: new checked state.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::Button;
    ///
    /// let mut btn = Button::new("Toggle".into());
    /// btn.checkable = true;
    /// btn.set_checked(true);
    /// assert!(btn.checked);
    /// ```
    #[slot]
    pub fn set_checked(&mut self, checked: bool) {
        if self.checkable && self.checked != checked {
            self.checked = checked;
            emit!(self.toggled, &(checked,));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;

    use quartzite_core::{Object, Value};

    use super::*;

    #[test]
    fn read_property_text_ok() {
        let btn = Button::new("OK".into());
        assert_eq!(btn.read_property("text"), Some(Value::String("OK".into())));
    }

    #[test]
    fn class_name_is_button() {
        let btn = Button::new("OK".into());
        assert_eq!(btn.meta_object().class_name, "Button");
    }

    #[test]
    fn click_emits_clicked_false_when_not_checkable() {
        let mut btn = Button::new("OK".into());
        let received: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
        let recv2 = Arc::clone(&received);
        btn.clicked.connect(move |args: &(bool,)| {
            *recv2.lock() = Some(args.0);
        });
        btn.click();
        assert_eq!(*received.lock(), Some(false));
    }

    #[test]
    fn write_property_text_updates_and_emits() {
        let mut btn = Button::new("OK".into());
        let fired: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let fired2 = Arc::clone(&fired);
        btn.text_changed.connect(move |args: &(String,)| {
            *fired2.lock() = Some(args.0.clone());
        });
        assert!(btn.write_property("text", Value::String("Cancel".into())));
        assert_eq!(btn.text, "Cancel");
        assert_eq!(*fired.lock(), Some("Cancel".into()));
    }

    #[test]
    fn set_text_slot_emits_text_changed() {
        let mut btn = Button::new("OK".into());
        let fired: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let fired2 = Arc::clone(&fired);
        btn.text_changed.connect(move |args: &(String,)| {
            *fired2.lock() = Some(args.0.clone());
        });
        btn.set_text("Cancel".into());
        assert_eq!(*fired.lock(), Some("Cancel".into()));
    }

    #[test]
    fn set_text_no_emit_when_same() {
        let mut btn = Button::new("OK".into());
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count2 = Arc::clone(&count);
        btn.text_changed.connect(move |_: &(String,)| {
            *count2.lock() += 1;
        });
        btn.set_text("OK".into());
        assert_eq!(*count.lock(), 0);
    }

    #[test]
    fn widget_view_returns_button_variant() {
        let btn = Button::new("OK".into());
        assert!(matches!(btn.widget_view(), crate::WidgetView::Button(_)));
    }

    #[test]
    fn checkable_click_toggles_checked_and_emits_toggled() {
        let mut btn = Button::new("Toggle".into());
        btn.checkable = true;
        let toggled: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
        let t2 = Arc::clone(&toggled);
        btn.toggled.connect(move |args: &(bool,)| {
            *t2.lock() = Some(args.0);
        });
        btn.click();
        assert!(btn.checked);
        assert_eq!(*toggled.lock(), Some(true));
    }
}
