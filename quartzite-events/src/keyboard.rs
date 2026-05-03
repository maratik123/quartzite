use alloc::string::String;

use crate::event::{Event, EventType, KeyEventKind};

bitflags::bitflags! {
    /// Keyboard modifier keys active at the time of an event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::KeyModifiers;
    ///
    /// let mods = KeyModifiers::CTRL | KeyModifiers::SHIFT;
    /// assert!(mods.contains(KeyModifiers::CTRL));
    /// assert!(mods.contains(KeyModifiers::SHIFT));
    /// assert!(!mods.contains(KeyModifiers::ALT));
    /// ```
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct KeyModifiers: u8 {
        /// The Shift key.
        const SHIFT = 0b0000_0001;
        /// The Ctrl key.
        const CTRL  = 0b0000_0010;
        /// The Alt / Option key.
        const ALT   = 0b0000_0100;
        /// The Meta / Super / Windows / Command key.
        const META  = 0b0000_1000;
    }
}

/// A platform-independent key identifier.
///
/// Ordering is declaration order — stable within a binary but not semantically meaningful.
///
/// # Examples
///
/// ```
/// use quartzite_events::Key;
///
/// let mut keys = vec![Key::B, Key::A, Key::C];
/// keys.sort();
/// assert_eq!(keys[0], Key::A);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Key {
    /// The `A` key.
    A,
    /// The `B` key.
    B,
    /// The `C` key.
    C,
    /// The `D` key.
    D,
    /// The `E` key.
    E,
    /// The `F` key.
    F,
    /// The `G` key.
    G,
    /// The `H` key.
    H,
    /// The `I` key.
    I,
    /// The `J` key.
    J,
    /// The `K` key.
    K,
    /// The `L` key.
    L,
    /// The `M` key.
    M,
    /// The `N` key.
    N,
    /// The `O` key.
    O,
    /// The `P` key.
    P,
    /// The `Q` key.
    Q,
    /// The `R` key.
    R,
    /// The `S` key.
    S,
    /// The `T` key.
    T,
    /// The `U` key.
    U,
    /// The `V` key.
    V,
    /// The `W` key.
    W,
    /// The `X` key.
    X,
    /// The `Y` key.
    Y,
    /// The `Z` key.
    Z,
    /// The `0` key.
    Num0,
    /// The `1` key.
    Num1,
    /// The `2` key.
    Num2,
    /// The `3` key.
    Num3,
    /// The `4` key.
    Num4,
    /// The `5` key.
    Num5,
    /// The `6` key.
    Num6,
    /// The `7` key.
    Num7,
    /// The `8` key.
    Num8,
    /// The `9` key.
    Num9,
    /// The `F1` key.
    F1,
    /// The `F2` key.
    F2,
    /// The `F3` key.
    F3,
    /// The `F4` key.
    F4,
    /// The `F5` key.
    F5,
    /// The `F6` key.
    F6,
    /// The `F7` key.
    F7,
    /// The `F8` key.
    F8,
    /// The `F9` key.
    F9,
    /// The `F10` key.
    F10,
    /// The `F11` key.
    F11,
    /// The `F12` key.
    F12,
    /// The Return / Enter key.
    Return,
    /// The Escape key.
    Escape,
    /// The Backspace key.
    Backspace,
    /// The Tab key.
    Tab,
    /// The Space bar.
    Space,
    /// The Delete key.
    Delete,
    /// The Insert key.
    Insert,
    /// The Home key.
    Home,
    /// The End key.
    End,
    /// The Page Up key.
    PageUp,
    /// The Page Down key.
    PageDown,
    /// The left arrow key.
    Left,
    /// The right arrow key.
    Right,
    /// The up arrow key.
    Up,
    /// The down arrow key.
    Down,
}

/// A keyboard input event.
///
/// # Examples
///
/// ```
/// use quartzite_events::{Key, KeyEvent, KeyEventKind, KeyModifiers};
///
/// let e = KeyEvent::new(Key::Return, String::from("\n"), KeyModifiers::empty(), false, KeyEventKind::Press);
/// assert_eq!(e.key(), Key::Return);
/// assert!(!e.is_repeat());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    key: Key,
    text: String,
    modifiers: KeyModifiers,
    is_repeat: bool,
    kind: KeyEventKind,
}

impl KeyEvent {
    /// Creates a new keyboard event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{Key, KeyEvent, KeyEventKind, KeyModifiers};
    ///
    /// let e = KeyEvent::new(Key::A, String::from("a"), KeyModifiers::empty(), false, KeyEventKind::Press);
    /// assert_eq!(e.key(), Key::A);
    /// ```
    #[inline]
    pub fn new(
        key: Key,
        text: String,
        modifiers: KeyModifiers,
        is_repeat: bool,
        kind: KeyEventKind,
    ) -> Self {
        Self {
            key,
            text,
            modifiers,
            is_repeat,
            kind,
        }
    }

    /// Returns the logical key identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{Key, KeyEvent, KeyEventKind, KeyModifiers};
    ///
    /// let e = KeyEvent::new(Key::Escape, String::new(), KeyModifiers::empty(), false, KeyEventKind::Press);
    /// assert_eq!(e.key(), Key::Escape);
    /// ```
    #[inline]
    pub fn key(&self) -> Key {
        self.key
    }

    /// Returns the UTF-8 text produced by this key press (empty for non-printable keys).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{Key, KeyEvent, KeyEventKind, KeyModifiers};
    ///
    /// let e = KeyEvent::new(Key::A, String::from("a"), KeyModifiers::empty(), false, KeyEventKind::Press);
    /// assert_eq!(e.text(), "a");
    /// ```
    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the active keyboard modifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{Key, KeyEvent, KeyEventKind, KeyModifiers};
    ///
    /// let e = KeyEvent::new(Key::A, String::from("A"), KeyModifiers::SHIFT, false, KeyEventKind::Press);
    /// assert!(e.modifiers().contains(KeyModifiers::SHIFT));
    /// ```
    #[inline]
    pub fn modifiers(&self) -> KeyModifiers {
        self.modifiers
    }

    /// Returns `true` if this is an auto-repeated key press.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{Key, KeyEvent, KeyEventKind, KeyModifiers};
    ///
    /// let e = KeyEvent::new(Key::Space, String::from(" "), KeyModifiers::empty(), true, KeyEventKind::Press);
    /// assert!(e.is_repeat());
    /// ```
    #[inline]
    pub fn is_repeat(&self) -> bool {
        self.is_repeat
    }

    /// Returns the specific kind of key event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_events::{Key, KeyEvent, KeyEventKind, KeyModifiers};
    ///
    /// let e = KeyEvent::new(Key::A, String::new(), KeyModifiers::empty(), false, KeyEventKind::Release);
    /// assert_eq!(e.kind(), KeyEventKind::Release);
    /// ```
    #[inline]
    pub fn kind(&self) -> KeyEventKind {
        self.kind
    }
}

impl<T: 'static + Send + Sync> Event<T> for KeyEvent {
    #[inline]
    fn event_type(&self) -> EventType<T> {
        EventType::Key(self.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::{BTreeMap, BTreeSet};

    fn press(key: Key) -> KeyEvent {
        KeyEvent::new(
            key,
            String::new(),
            KeyModifiers::empty(),
            false,
            KeyEventKind::Press,
        )
    }

    #[test]
    fn key_modifiers_ctrl_shift() {
        let mods = KeyModifiers::CTRL | KeyModifiers::SHIFT;
        assert!(mods.contains(KeyModifiers::CTRL));
        assert!(mods.contains(KeyModifiers::SHIFT));
        assert!(!mods.contains(KeyModifiers::ALT));
    }

    #[test]
    fn key_in_btreemap() {
        let mut map = BTreeMap::new();
        map.insert(Key::Return, "confirm");
        assert_eq!(map[&Key::Return], "confirm");
    }

    #[test]
    fn key_in_hashmap() {
        let mut map = hashbrown::HashMap::new();
        map.insert(Key::Return, "confirm");
        assert_eq!(map[&Key::Return], "confirm");
    }

    #[test]
    fn key_sortable() {
        let mut keys = alloc::vec![Key::B, Key::A, Key::C];
        keys.sort();
        assert_eq!(keys[0], Key::A);
    }

    #[test]
    fn key_in_btreeset() {
        let mut set = BTreeSet::new();
        set.insert(Key::Escape);
        set.insert(Key::Return);
        assert!(set.contains(&Key::Escape));
    }

    #[test]
    fn key_event_press_type() {
        let e = press(Key::A);
        assert_eq!(e.event_type(), EventType::<()>::Key(KeyEventKind::Press));
    }

    #[test]
    fn key_event_release_type() {
        let e = KeyEvent::new(
            Key::A,
            String::new(),
            KeyModifiers::empty(),
            false,
            KeyEventKind::Release,
        );
        assert_eq!(e.event_type(), EventType::<()>::Key(KeyEventKind::Release));
    }

    #[test]
    fn key_event_is_repeat() {
        let e = KeyEvent::new(
            Key::Space,
            String::from(" "),
            KeyModifiers::empty(),
            true,
            KeyEventKind::Press,
        );
        assert!(e.is_repeat());
    }
}
