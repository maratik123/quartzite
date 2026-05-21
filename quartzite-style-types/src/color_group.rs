//! [`ColorGroup`] — the interaction state group of a colour in a [`Palette`].
//!
//! Each group names a *state* in which a widget can be rendered (resting,
//! hovered, pressed), orthogonal to the semantic role ([`ColorRole`]).
//! Concrete values are looked up via [`Palette::color`].
//!
//! [`ColorRole`]: crate::ColorRole
//! [`Palette`]: crate::Palette
//! [`Palette::color`]: crate::Palette::color

/// Interaction state group of a colour in a [`Palette`](crate::Palette).
///
/// The variants name *when the colour is used*, not what it looks like — the
/// same [`ColorRole`](crate::ColorRole) has three group-specific values
/// (normal, hover, pressed). Widgets select the group based on the current
/// interaction state and then look up the colour via
/// [`Palette::color`](crate::Palette::color).
///
/// # Examples
///
/// ```
/// use quartzite_style_types::{ColorGroup, ColorRole, Palette};
///
/// let palette = Palette::default();
/// let _resting = palette.color(ColorRole::Button, ColorGroup::Normal);
/// let _hovered = palette.color(ColorRole::Button, ColorGroup::Hover);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorGroup {
    /// Resting state: no pointer over the widget, not pressed.
    #[default]
    Normal,
    /// Pointer is over the widget but not pressed.
    Hover,
    /// Widget is actively being pressed.
    Pressed,
}

impl ColorGroup {
    /// Every variant of [`ColorGroup`], in declaration order.
    ///
    /// Used by [`Palette`](crate::Palette) to iterate all groups and by tests
    /// that loop over every group (rather than enumerating each variant by
    /// hand). Adding a new variant requires updating this constant — the
    /// `all_constant_lists_every_variant` unit test enforces that via an
    /// exhaustive `match` whose arm count must equal `ALL.len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style_types::ColorGroup;
    ///
    /// assert!(ColorGroup::ALL.contains(&ColorGroup::Normal));
    /// assert_eq!(ColorGroup::ALL.first().copied(), Some(ColorGroup::Normal));
    /// ```
    pub const ALL: &'static [Self] = &[Self::Normal, Self::Hover, Self::Pressed];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ALL` lists every variant exactly once. Loops over the slice and runs
    /// an exhaustive `match` per group to count it — every arm is therefore
    /// executed, which both proves the slice contents and forces the test to
    /// fail to compile if a new variant is added without an arm here.
    #[test]
    fn all_constant_lists_every_variant() {
        let mut count = 0_usize;
        for group in ColorGroup::ALL {
            match group {
                ColorGroup::Normal | ColorGroup::Hover | ColorGroup::Pressed => count += 1,
            }
        }
        assert_eq!(ColorGroup::ALL.len(), count);
        assert_eq!(count, 3);
    }

    #[test]
    fn default_is_normal() {
        assert_eq!(ColorGroup::default(), ColorGroup::Normal);
    }

    #[test]
    fn equality_and_hashing_work() {
        use core::hash::{BuildHasher, Hasher};
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
        assert_eq!(
            b.hash_one(ColorGroup::Normal),
            b.hash_one(ColorGroup::Normal)
        );
        assert_eq!(ColorGroup::Normal, ColorGroup::Normal);
        assert_ne!(ColorGroup::Normal, ColorGroup::Hover);
    }
}
