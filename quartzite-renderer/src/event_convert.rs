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
    let key = key_from_winit(&event.logical_key)?;
    let text = event
        .text
        .as_ref()
        .map(|s| s.to_string())
        .unwrap_or_default();
    let kind = match event.state {
        ElementState::Pressed => KeyEventKind::Press,
        ElementState::Released => KeyEventKind::Release,
    };
    Some(KeyEvent::new(key, text, modifiers, event.repeat, kind))
}

#[cfg(test)]
mod tests {
    use winit::dpi::PhysicalSize;

    use super::*;

    #[test]
    fn size_from_physical_normal() {
        assert_eq!(
            size_from_physical(PhysicalSize::new(800_u32, 600_u32)),
            Size::new(800, 600)
        );
    }

    #[test]
    fn size_from_physical_saturates() {
        assert_eq!(
            size_from_physical(PhysicalSize::new(u32::MAX, 0_u32)),
            Size::new(i32::MAX, 0)
        );
    }

    #[test]
    fn mouse_button_left_maps() {
        let btns = mouse_button_from_winit(WinitMouseButton::Left);
        assert!(btns.contains(MouseButton::Left));
    }

    #[test]
    fn mouse_button_other_maps_to_empty() {
        let btns = mouse_button_from_winit(WinitMouseButton::Other(99));
        assert!(btns.is_empty());
    }
}
