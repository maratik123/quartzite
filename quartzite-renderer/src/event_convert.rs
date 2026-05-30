//! Conversion helpers from winit events to quartzite-events types.

use quartzite_events::{
    Key, KeyEvent, KeyEventKind, KeyModifier, KeyModifiers, MouseButton, MouseButtons, MouseEvent,
    MouseEventKind,
};
use quartzite_geometry::{Point, Size};
use winit::event::{ElementState, MouseButton as WinitMouseButton};
use winit::keyboard::{Key as WinitKey, NamedKey};

/// Converts a `PhysicalSize<u32>` to [`Size`], saturating at [`i32::MAX`].
///
/// `Size::new` takes `i32` values; winit's resize event carries `PhysicalSize<u32>`.
/// Plain `as` casting would silently flip values > 2 147 483 647 to negative,
/// violating `Size`'s soft contract that width and height are ≥ 0. Saturation
/// is the safer choice: displays larger than ~2.1 G pixels per axis don't exist
/// in practice, so the saturating path is unreachable on real hardware.
///
/// # Parameters
///
/// - `size`: the winit physical size to convert; each axis saturates to
///   [`i32::MAX`] if it exceeds `i32::MAX`.
///
/// # Examples
///
/// ```
/// use quartzite_renderer::event_convert::size_from_physical;
/// use quartzite_geometry::Size;
/// use winit::dpi::PhysicalSize;
///
/// assert_eq!(size_from_physical(PhysicalSize::new(800_u32, 600_u32)), Size::new(800, 600));
/// assert_eq!(size_from_physical(PhysicalSize::new(u32::MAX, 0_u32)), Size::new(i32::MAX, 0));
/// ```
pub fn size_from_physical(size: winit::dpi::PhysicalSize<u32>) -> Size {
    let w = i32::try_from(size.width).unwrap_or(i32::MAX);
    let h = i32::try_from(size.height).unwrap_or(i32::MAX);
    Size::new(w, h)
}

/// Converts a winit [`MouseButton`][WinitMouseButton] to a quartzite [`MouseButton`] set.
///
/// Returns [`MouseButtons::empty()`] for `Other` buttons (no quartzite equivalent).
pub(crate) fn mouse_button_from_winit(btn: WinitMouseButton) -> MouseButtons {
    match btn {
        WinitMouseButton::Left => MouseButton::Left.into(),
        WinitMouseButton::Right => MouseButton::Right.into(),
        WinitMouseButton::Middle => MouseButton::Middle.into(),
        WinitMouseButton::Back => MouseButton::Back.into(),
        WinitMouseButton::Forward => MouseButton::Forward.into(),
        WinitMouseButton::Other(_) => MouseButtons::empty(),
    }
}

/// Constructs a quartzite [`MouseEvent`] from a winit `MouseInput` event.
#[allow(
    clippy::cast_possible_truncation,
    reason = "deliberate truncation within known bounds"
)]
pub(crate) fn mouse_event_from_winit(
    state: ElementState,
    button: WinitMouseButton,
    position: winit::dpi::PhysicalPosition<f64>,
    current_buttons: MouseButtons,
) -> MouseEvent {
    let kind = match state {
        ElementState::Pressed => MouseEventKind::Press,
        ElementState::Released => MouseEventKind::Release,
    };
    let quartzite_btn = mouse_button_from_winit(button);
    let pos = Point::new(position.x as i32, position.y as i32);
    MouseEvent::new(
        pos,
        pos,
        quartzite_btn,
        current_buttons,
        KeyModifiers::empty(),
        kind,
    )
}

/// Converts a winit [`Key`][WinitKey] to a quartzite [`Key`].
///
/// Returns `None` for keys with no quartzite mapping (e.g. IME keys, dead
/// keys, unrecognised named keys).
pub(crate) fn key_from_winit(key: &WinitKey) -> Option<Key> {
    match key {
        WinitKey::Character(s) => match s.as_str() {
            "a" | "A" => Some(Key::A),
            "b" | "B" => Some(Key::B),
            "c" | "C" => Some(Key::C),
            "d" | "D" => Some(Key::D),
            "e" | "E" => Some(Key::E),
            "f" | "F" => Some(Key::F),
            "g" | "G" => Some(Key::G),
            "h" | "H" => Some(Key::H),
            "i" | "I" => Some(Key::I),
            "j" | "J" => Some(Key::J),
            "k" | "K" => Some(Key::K),
            "l" | "L" => Some(Key::L),
            "m" | "M" => Some(Key::M),
            "n" | "N" => Some(Key::N),
            "o" | "O" => Some(Key::O),
            "p" | "P" => Some(Key::P),
            "q" | "Q" => Some(Key::Q),
            "r" | "R" => Some(Key::R),
            "s" | "S" => Some(Key::S),
            "t" | "T" => Some(Key::T),
            "u" | "U" => Some(Key::U),
            "v" | "V" => Some(Key::V),
            "w" | "W" => Some(Key::W),
            "x" | "X" => Some(Key::X),
            "y" | "Y" => Some(Key::Y),
            "z" | "Z" => Some(Key::Z),
            "0" => Some(Key::Num0),
            "1" => Some(Key::Num1),
            "2" => Some(Key::Num2),
            "3" => Some(Key::Num3),
            "4" => Some(Key::Num4),
            "5" => Some(Key::Num5),
            "6" => Some(Key::Num6),
            "7" => Some(Key::Num7),
            "8" => Some(Key::Num8),
            "9" => Some(Key::Num9),
            _ => None,
        },
        WinitKey::Named(named) => match named {
            NamedKey::Enter => Some(Key::Return),
            NamedKey::Escape => Some(Key::Escape),
            NamedKey::Backspace => Some(Key::Backspace),
            NamedKey::Tab => Some(Key::Tab),
            NamedKey::Space => Some(Key::Space),
            NamedKey::Delete => Some(Key::Delete),
            NamedKey::Insert => Some(Key::Insert),
            NamedKey::Home => Some(Key::Home),
            NamedKey::End => Some(Key::End),
            NamedKey::PageUp => Some(Key::PageUp),
            NamedKey::PageDown => Some(Key::PageDown),
            NamedKey::ArrowLeft => Some(Key::Left),
            NamedKey::ArrowRight => Some(Key::Right),
            NamedKey::ArrowUp => Some(Key::Up),
            NamedKey::ArrowDown => Some(Key::Down),
            NamedKey::F1 => Some(Key::F1),
            NamedKey::F2 => Some(Key::F2),
            NamedKey::F3 => Some(Key::F3),
            NamedKey::F4 => Some(Key::F4),
            NamedKey::F5 => Some(Key::F5),
            NamedKey::F6 => Some(Key::F6),
            NamedKey::F7 => Some(Key::F7),
            NamedKey::F8 => Some(Key::F8),
            NamedKey::F9 => Some(Key::F9),
            NamedKey::F10 => Some(Key::F10),
            NamedKey::F11 => Some(Key::F11),
            NamedKey::F12 => Some(Key::F12),
            _ => None,
        },
        _ => None,
    }
}

/// Converts winit modifier state to quartzite [`KeyModifiers`].
pub(crate) fn modifiers_from_winit(mods: winit::event::Modifiers) -> KeyModifiers {
    let state = mods.state();
    let mut result = KeyModifiers::empty();
    if state.shift_key() {
        result |= KeyModifier::Shift;
    }
    if state.control_key() {
        result |= KeyModifier::Ctrl;
    }
    if state.alt_key() {
        result |= KeyModifier::Alt;
    }
    if state.super_key() {
        result |= KeyModifier::Meta;
    }
    result
}

/// Constructs a quartzite [`KeyEvent`] from a winit `KeyboardInput` event.
///
/// Returns `None` if the key has no quartzite mapping.
pub(crate) fn key_event_from_winit(
    event: &winit::event::KeyEvent,
    modifiers: KeyModifiers,
) -> Option<KeyEvent> {
    key_event_from_parts(
        &event.logical_key,
        event.text.as_deref(),
        event.state,
        event.repeat,
        modifiers,
    )
}

/// Core of [`key_event_from_winit`], operating on decomposed fields.
///
/// Split out so that tests can supply their own `logical_key` / `state` /
/// `repeat` without needing to construct `winit::event::KeyEvent` (which has
/// a `pub(crate)` platform-specific field).
pub(crate) fn key_event_from_parts(
    logical_key: &WinitKey,
    text: Option<&str>,
    state: ElementState,
    repeat: bool,
    modifiers: KeyModifiers,
) -> Option<KeyEvent> {
    let key = key_from_winit(logical_key)?;
    let text = text.map(str::to_string).unwrap_or_default();
    let kind = match state {
        ElementState::Pressed => KeyEventKind::Press,
        ElementState::Released => KeyEventKind::Release,
    };
    Some(KeyEvent::new(key, text, modifiers, repeat, kind))
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use rstest::rstest;
    use winit::dpi::{PhysicalPosition, PhysicalSize};
    use winit::keyboard::{ModifiersState, NamedKey, NativeKey};

    use super::*;

    // --- size_from_physical ---

    #[test]
    fn size_from_physical_normal() {
        assert_eq!(
            size_from_physical(PhysicalSize::new(800_u32, 600_u32)),
            Size::new(800, 600)
        );
    }

    #[test]
    fn size_from_physical_saturates_width() {
        assert_eq!(
            size_from_physical(PhysicalSize::new(u32::MAX, 0_u32)),
            Size::new(i32::MAX, 0)
        );
    }

    #[test]
    fn size_from_physical_saturates_height() {
        assert_eq!(
            size_from_physical(PhysicalSize::new(0_u32, u32::MAX)),
            Size::new(0, i32::MAX)
        );
    }

    // --- mouse_button_from_winit ---

    #[rstest]
    #[case(WinitMouseButton::Left, MouseButton::Left)]
    #[case(WinitMouseButton::Right, MouseButton::Right)]
    #[case(WinitMouseButton::Middle, MouseButton::Middle)]
    #[case(WinitMouseButton::Back, MouseButton::Back)]
    #[case(WinitMouseButton::Forward, MouseButton::Forward)]
    fn mouse_button_maps(#[case] btn: WinitMouseButton, #[case] expected: MouseButton) {
        assert!(mouse_button_from_winit(btn).contains(expected));
    }

    #[test]
    fn mouse_button_other_maps_to_empty() {
        assert!(mouse_button_from_winit(WinitMouseButton::Other(99)).is_empty());
    }

    // --- mouse_event_from_winit ---

    #[test]
    fn mouse_event_pressed() {
        let pos = PhysicalPosition::new(10.0_f64, 20.0);
        let ev = mouse_event_from_winit(
            ElementState::Pressed,
            WinitMouseButton::Left,
            pos,
            MouseButtons::empty(),
        );
        assert_matches!(ev.kind(), MouseEventKind::Press);
        assert!(ev.event_button().contains(MouseButton::Left));
        assert_eq!(ev.position(), Point::new(10, 20));
    }

    #[test]
    fn mouse_event_released() {
        let pos = PhysicalPosition::new(5.0_f64, 15.0);
        let ev = mouse_event_from_winit(
            ElementState::Released,
            WinitMouseButton::Right,
            pos,
            MouseButton::Right.into(),
        );
        assert_matches!(ev.kind(), MouseEventKind::Release);
        assert!(ev.event_button().contains(MouseButton::Right));
    }

    // --- key_from_winit: character keys ---

    #[rstest]
    #[case("a", Key::A)]
    #[case("A", Key::A)]
    #[case("b", Key::B)]
    #[case("B", Key::B)]
    #[case("c", Key::C)]
    #[case("d", Key::D)]
    #[case("e", Key::E)]
    #[case("f", Key::F)]
    #[case("g", Key::G)]
    #[case("h", Key::H)]
    #[case("i", Key::I)]
    #[case("j", Key::J)]
    #[case("k", Key::K)]
    #[case("l", Key::L)]
    #[case("m", Key::M)]
    #[case("n", Key::N)]
    #[case("o", Key::O)]
    #[case("p", Key::P)]
    #[case("q", Key::Q)]
    #[case("r", Key::R)]
    #[case("s", Key::S)]
    #[case("t", Key::T)]
    #[case("u", Key::U)]
    #[case("v", Key::V)]
    #[case("w", Key::W)]
    #[case("x", Key::X)]
    #[case("y", Key::Y)]
    #[case("z", Key::Z)]
    #[case("0", Key::Num0)]
    #[case("1", Key::Num1)]
    #[case("2", Key::Num2)]
    #[case("3", Key::Num3)]
    #[case("4", Key::Num4)]
    #[case("5", Key::Num5)]
    #[case("6", Key::Num6)]
    #[case("7", Key::Num7)]
    #[case("8", Key::Num8)]
    #[case("9", Key::Num9)]
    fn character_key_maps(#[case] ch: &str, #[case] expected: Key) {
        assert_eq!(
            key_from_winit(&WinitKey::Character(ch.into())),
            Some(expected)
        );
    }

    #[test]
    fn character_key_unmapped_returns_none() {
        assert_eq!(key_from_winit(&WinitKey::Character("!".into())), None);
    }

    // --- key_from_winit: named keys ---

    #[rstest]
    #[case(NamedKey::Enter, Key::Return)]
    #[case(NamedKey::Escape, Key::Escape)]
    #[case(NamedKey::Backspace, Key::Backspace)]
    #[case(NamedKey::Tab, Key::Tab)]
    #[case(NamedKey::Space, Key::Space)]
    #[case(NamedKey::Delete, Key::Delete)]
    #[case(NamedKey::Insert, Key::Insert)]
    #[case(NamedKey::Home, Key::Home)]
    #[case(NamedKey::End, Key::End)]
    #[case(NamedKey::PageUp, Key::PageUp)]
    #[case(NamedKey::PageDown, Key::PageDown)]
    #[case(NamedKey::ArrowLeft, Key::Left)]
    #[case(NamedKey::ArrowRight, Key::Right)]
    #[case(NamedKey::ArrowUp, Key::Up)]
    #[case(NamedKey::ArrowDown, Key::Down)]
    #[case(NamedKey::F1, Key::F1)]
    #[case(NamedKey::F2, Key::F2)]
    #[case(NamedKey::F3, Key::F3)]
    #[case(NamedKey::F4, Key::F4)]
    #[case(NamedKey::F5, Key::F5)]
    #[case(NamedKey::F6, Key::F6)]
    #[case(NamedKey::F7, Key::F7)]
    #[case(NamedKey::F8, Key::F8)]
    #[case(NamedKey::F9, Key::F9)]
    #[case(NamedKey::F10, Key::F10)]
    #[case(NamedKey::F11, Key::F11)]
    #[case(NamedKey::F12, Key::F12)]
    fn named_key_maps(#[case] named: NamedKey, #[case] expected: Key) {
        assert_eq!(key_from_winit(&WinitKey::Named(named)), Some(expected));
    }

    #[test]
    fn named_key_unmapped_returns_none() {
        assert_eq!(key_from_winit(&WinitKey::Named(NamedKey::Hyper)), None);
    }

    #[test]
    fn unidentified_key_returns_none() {
        assert_eq!(
            key_from_winit(&WinitKey::Unidentified(NativeKey::Unidentified)),
            None
        );
    }

    // --- modifiers_from_winit ---

    #[test]
    fn modifiers_empty() {
        let mods = modifiers_from_winit(winit::event::Modifiers::from(ModifiersState::empty()));
        assert!(mods.is_empty());
    }

    #[test]
    fn modifiers_shift() {
        let mods = modifiers_from_winit(winit::event::Modifiers::from(ModifiersState::SHIFT));
        assert!(mods.contains(KeyModifier::Shift));
        assert!(!mods.contains(KeyModifier::Ctrl));
    }

    #[test]
    fn modifiers_ctrl() {
        let mods = modifiers_from_winit(winit::event::Modifiers::from(ModifiersState::CONTROL));
        assert!(mods.contains(KeyModifier::Ctrl));
        assert!(!mods.contains(KeyModifier::Shift));
    }

    #[test]
    fn modifiers_alt() {
        let mods = modifiers_from_winit(winit::event::Modifiers::from(ModifiersState::ALT));
        assert!(mods.contains(KeyModifier::Alt));
    }

    #[test]
    fn modifiers_meta() {
        let mods = modifiers_from_winit(winit::event::Modifiers::from(ModifiersState::SUPER));
        assert!(mods.contains(KeyModifier::Meta));
    }

    #[test]
    fn modifiers_combined_shift_ctrl() {
        let mods = modifiers_from_winit(winit::event::Modifiers::from(
            ModifiersState::SHIFT | ModifiersState::CONTROL,
        ));
        assert!(mods.contains(KeyModifier::Shift));
        assert!(mods.contains(KeyModifier::Ctrl));
        assert!(!mods.contains(KeyModifier::Alt));
    }

    // --- key_event_from_parts ---

    #[test]
    fn key_event_parts_mapped_pressed() {
        let ev = key_event_from_parts(
            &WinitKey::Character("a".into()),
            Some("a"),
            ElementState::Pressed,
            false,
            KeyModifiers::empty(),
        );
        let ev = ev.expect("should map 'a'");
        assert_eq!(ev.key(), Key::A);
        assert_eq!(ev.text(), "a");
        assert_matches!(ev.kind(), KeyEventKind::Press);
        assert!(!ev.is_repeat());
    }

    #[test]
    fn key_event_parts_released() {
        let ev = key_event_from_parts(
            &WinitKey::Named(NamedKey::Enter),
            None,
            ElementState::Released,
            false,
            KeyModifiers::empty(),
        );
        let ev = ev.expect("Enter should map");
        assert_matches!(ev.kind(), KeyEventKind::Release);
        assert_eq!(ev.text(), "");
    }

    #[test]
    fn key_event_parts_repeat_propagates() {
        let ev = key_event_from_parts(
            &WinitKey::Character("b".into()),
            None,
            ElementState::Pressed,
            true,
            KeyModifiers::empty(),
        )
        .expect("should map");
        assert!(ev.is_repeat());
    }

    #[test]
    fn key_event_parts_unmapped_returns_none() {
        let result = key_event_from_parts(
            &WinitKey::Named(NamedKey::Hyper),
            None,
            ElementState::Pressed,
            false,
            KeyModifiers::empty(),
        );
        assert!(result.is_none());
    }

    #[test]
    fn key_event_parts_modifiers_propagate() {
        let mods: KeyModifiers = KeyModifier::Shift.into();
        let ev = key_event_from_parts(
            &WinitKey::Character("c".into()),
            None,
            ElementState::Pressed,
            false,
            mods,
        )
        .expect("should map");
        assert!(ev.modifiers().contains(KeyModifier::Shift));
    }

    // --- key_event_from_winit (smoke through wrapping fn) ---

    // Coverage of the `key_event_from_winit` wrapper itself is obtained via
    // wrapped_handler tests (subtask R4) which inject `WindowEvent::KeyboardInput`.
    // The inner logic is fully covered by `key_event_from_parts` tests above.
}
