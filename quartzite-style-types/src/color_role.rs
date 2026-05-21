//! [`ColorRole`] — the semantic role a color plays in a [`Palette`].
//!
//! Each role names a slot in the palette (window background, button face,
//! text on a highlighted region, etc.) rather than a concrete colour. Concrete
//! values are looked up via [`Palette::color`].
//!
//! [`Palette`]: crate::Palette
//! [`Palette::color`]: crate::Palette::color

/// Semantic role of a colour in a [`Palette`](crate::Palette).
///
/// The variants name *what the colour is for*, not what it looks like — a
/// dark theme and a light theme share these roles but populate them with
/// different RGBA values. Roles are stable identifiers that widgets request
/// at draw time; the active palette resolves the role to a concrete
/// [`Color`](quartzite_paint_api::Color).
///
/// # Examples
///
/// ```
/// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
///
/// let palette = Palette::default();
/// let _window_bg = palette.color(ColorRole::Window, ColorGroup::Normal);
/// let _text = palette.color(ColorRole::Text, ColorGroup::Normal);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ColorRole {
    /// Background of windows and top-level surfaces.
    Window,
    /// Foreground text drawn on a [`Window`](Self::Window) background.
    WindowText,
    /// Background of push buttons and similar pressable surfaces.
    Button,
    /// Foreground text drawn on a [`Button`](Self::Button) background.
    ButtonText,
    /// Background of editable text widgets (inputs, text areas).
    Base,
    /// Foreground text drawn on a [`Base`](Self::Base) background.
    Text,
    /// Selection / highlight background colour.
    Highlight,
    /// Foreground text drawn on a [`Highlight`](Self::Highlight) background.
    HighlightedText,
    /// Foreground colour for unvisited hyperlinks.
    Link,
    /// Foreground colour for visited hyperlinks.
    LinkVisited,
    /// High-contrast text used to attract attention (e.g. on a coloured banner).
    BrightText,
    /// Stroke colour for the keyboard-focus outline drawn around a focused widget.
    FocusRing,
}

impl ColorRole {
    /// Every variant of [`ColorRole`], in declaration order.
    ///
    /// Used by [`Palette::default`](crate::Palette::default) to seed every slot
    /// and by tests that loop over all roles (rather than enumerating each
    /// variant by hand). Adding a new variant requires updating this constant —
    /// the `all_constant_lists_every_variant` unit test enforces that via an
    /// exhaustive `match` whose arm count must equal `ALL.len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style_types::ColorRole;
    ///
    /// assert!(ColorRole::ALL.contains(&ColorRole::Window));
    /// assert_eq!(ColorRole::ALL.first().copied(), Some(ColorRole::Window));
    /// ```
    pub const ALL: &'static [Self] = &[
        Self::Window,
        Self::WindowText,
        Self::Button,
        Self::ButtonText,
        Self::Base,
        Self::Text,
        Self::Highlight,
        Self::HighlightedText,
        Self::Link,
        Self::LinkVisited,
        Self::BrightText,
        Self::FocusRing,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ALL` lists every variant exactly once. Loops over the slice and runs
    /// an exhaustive `match` per role to count it — every arm is therefore
    /// executed, which both proves the slice contents and forces the test to
    /// fail to compile if a new variant is added without an arm here.
    #[test]
    fn all_constant_lists_every_variant() {
        let mut count = 0_usize;
        for role in ColorRole::ALL {
            match role {
                ColorRole::Window
                | ColorRole::WindowText
                | ColorRole::Button
                | ColorRole::ButtonText
                | ColorRole::Base
                | ColorRole::Text
                | ColorRole::Highlight
                | ColorRole::HighlightedText
                | ColorRole::Link
                | ColorRole::LinkVisited
                | ColorRole::BrightText
                | ColorRole::FocusRing => count += 1,
            }
        }
        assert_eq!(ColorRole::ALL.len(), count);
        assert_eq!(count, 12);
    }

    #[test]
    fn all_contains_each_named_role() {
        for role in [
            ColorRole::Window,
            ColorRole::WindowText,
            ColorRole::Button,
            ColorRole::ButtonText,
            ColorRole::Base,
            ColorRole::Text,
            ColorRole::Highlight,
            ColorRole::HighlightedText,
            ColorRole::Link,
            ColorRole::LinkVisited,
            ColorRole::BrightText,
            ColorRole::FocusRing,
        ] {
            assert!(ColorRole::ALL.contains(&role), "ALL is missing {role:?}");
        }
    }

    #[test]
    fn equality_and_hashing_work() {
        use core::hash::{BuildHasher, Hasher};
        // BuildHasherDefault is unavailable in core; rely on a plain Hasher
        // pair to confirm Hash is implemented and produces the same bytes
        // for equal values.
        struct ConstHasher(u64);
        impl Hasher for ConstHasher {
            fn finish(&self) -> u64 {
                self.0
            }
            fn write(&mut self, bytes: &[u8]) {
                for &b in bytes {
                    self.0 = self.0.wrapping_mul(31).wrapping_add(u64::from(b));
                }
            }
        }
        struct ConstBuilder;
        impl BuildHasher for ConstBuilder {
            type Hasher = ConstHasher;
            fn build_hasher(&self) -> ConstHasher {
                ConstHasher(0)
            }
        }
        let b = ConstBuilder;
        assert_eq!(b.hash_one(ColorRole::Window), b.hash_one(ColorRole::Window));
        assert_eq!(ColorRole::Window, ColorRole::Window);
        assert_ne!(ColorRole::Window, ColorRole::Text);
    }
}
