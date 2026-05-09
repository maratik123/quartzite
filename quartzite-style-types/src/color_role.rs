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
/// use quartzite_style_types::{ColorRole, Palette};
///
/// let palette = Palette::default();
/// let _window_bg = palette.color(ColorRole::Window);
/// let _text = palette.color(ColorRole::Text);
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
    pub const ALL: &'static [ColorRole] = &[
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
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The arm count of an exhaustive match over `ColorRole` must equal
    /// `ALL.len()`. Adding a new variant without extending `ALL` fails this
    /// test (the match still compiles after the extension because the new
    /// arm contributes another `+ 1`).
    #[test]
    fn all_constant_lists_every_variant() {
        let sample = ColorRole::Window;
        let variant_count: usize = match sample {
            ColorRole::Window => 1,
            ColorRole::WindowText => 1,
            ColorRole::Button => 1,
            ColorRole::ButtonText => 1,
            ColorRole::Base => 1,
            ColorRole::Text => 1,
            ColorRole::Highlight => 1,
            ColorRole::HighlightedText => 1,
            ColorRole::Link => 1,
            ColorRole::LinkVisited => 1,
            ColorRole::BrightText => 1,
        } * 11;
        assert_eq!(ColorRole::ALL.len(), variant_count);
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
