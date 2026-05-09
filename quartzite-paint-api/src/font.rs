//! [`Font`] and [`FontWeight`] — typeface description used by `Painter::draw_text`.
//!
//! The types live in `quartzite-paint-api` (rather than `quartzite-paint`) so the
//! [`Painter`](crate::Painter) trait can name `&Font` directly without a circular
//! re-export through the higher-level paint crate.

use alloc::string::String;

/// A typeface description: family, point size, weight, and style flags.
///
/// `Font` is the value passed to [`Painter::draw_text`](crate::Painter::draw_text)
/// and [`Painter::draw_text_in`](crate::Painter::draw_text_in). It is a plain
/// description object — no font loading happens at construction; backends resolve
/// the family at draw time.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{Font, FontWeight};
///
/// let f = Font::new("Arial", 12.0);
/// assert_eq!(f.family(), "Arial");
/// assert_eq!(f.size_pt(), 12.0);
/// assert_eq!(f.weight(), FontWeight::Normal);
/// assert!(!f.italic());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    family: String,
    size_pt: f32,
    weight: FontWeight,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl Font {
    /// Creates a new font with the given family and point size.
    ///
    /// All style flags default to off and the weight defaults to
    /// [`FontWeight::Normal`].
    ///
    /// # Parameters
    ///
    /// - `family`: typeface family name (e.g. `"Arial"`, `"Helvetica"`).
    ///   Accepts any value convertible into `String`.
    /// - `size_pt`: point size; `12.0` is a typical UI body size.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Font;
    ///
    /// let f = Font::new("Helvetica", 14.0);
    /// assert_eq!(f.family(), "Helvetica");
    /// assert_eq!(f.size_pt(), 14.0);
    /// ```
    ///
    /// _Simple._
    #[inline]
    pub fn new(family: impl Into<String>, size_pt: f32) -> Self {
        Self {
            family: family.into(),
            size_pt,
            weight: FontWeight::Normal,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }

    /// Returns the family name.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Font;
    /// assert_eq!(Font::new("Arial", 12.0).family(), "Arial");
    /// ```
    #[inline]
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Returns the point size.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Font;
    /// assert_eq!(Font::new("Arial", 12.0).size_pt(), 12.0);
    /// ```
    #[inline]
    pub const fn size_pt(&self) -> f32 {
        self.size_pt
    }

    /// Returns the weight.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Font, FontWeight};
    /// assert_eq!(Font::new("Arial", 12.0).weight(), FontWeight::Normal);
    /// ```
    #[inline]
    pub const fn weight(&self) -> FontWeight {
        self.weight
    }

    /// Returns whether the font is italic.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Font;
    /// assert!(!Font::new("Arial", 12.0).italic());
    /// ```
    #[inline]
    pub const fn italic(&self) -> bool {
        self.italic
    }

    /// Returns whether the font is underlined.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Font;
    /// assert!(!Font::new("Arial", 12.0).underline());
    /// ```
    #[inline]
    pub const fn underline(&self) -> bool {
        self.underline
    }

    /// Returns whether the font is struck through.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Font;
    /// assert!(!Font::new("Arial", 12.0).strikethrough());
    /// ```
    #[inline]
    pub const fn strikethrough(&self) -> bool {
        self.strikethrough
    }
}

/// Canonical CSS-style font weight.
///
/// Variant discriminants match the CSS numeric weight values (`100`–`900`).
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::FontWeight;
///
/// assert_eq!(FontWeight::Bold as u16, 700);
/// assert_eq!(FontWeight::default(), FontWeight::Normal);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum FontWeight {
    /// `100` — Thin / Hairline.
    Thin = 100,
    /// `200` — Extra Light / Ultra Light.
    ExtraLight = 200,
    /// `300` — Light.
    Light = 300,
    /// `400` — Normal / Regular. The default.
    #[default]
    Normal = 400,
    /// `500` — Medium.
    Medium = 500,
    /// `600` — Semi Bold / Demi Bold.
    SemiBold = 600,
    /// `700` — Bold.
    Bold = 700,
    /// `800` — Extra Bold / Ultra Bold.
    ExtraBold = 800,
    /// `900` — Black / Heavy.
    Black = 900,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_default_weight_normal_and_flags_off() {
        let f = Font::new("Arial", 12.0);
        assert_eq!(f.family(), "Arial");
        assert_eq!(f.size_pt(), 12.0);
        assert_eq!(f.weight(), FontWeight::Normal);
        assert!(!f.italic());
        assert!(!f.underline());
        assert!(!f.strikethrough());
    }

    #[test]
    fn font_weight_default_is_normal() {
        assert_eq!(FontWeight::default(), FontWeight::Normal);
    }

    #[test]
    fn font_weight_numeric_values() {
        assert_eq!(FontWeight::Thin as u16, 100);
        assert_eq!(FontWeight::ExtraLight as u16, 200);
        assert_eq!(FontWeight::Light as u16, 300);
        assert_eq!(FontWeight::Normal as u16, 400);
        assert_eq!(FontWeight::Medium as u16, 500);
        assert_eq!(FontWeight::SemiBold as u16, 600);
        assert_eq!(FontWeight::Bold as u16, 700);
        assert_eq!(FontWeight::ExtraBold as u16, 800);
        assert_eq!(FontWeight::Black as u16, 900);
    }

    #[test]
    fn font_clone_round_trip() {
        let a = Font::new("Helvetica", 14.0);
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(b.family(), "Helvetica");
    }

    #[test]
    fn new_accepts_owned_string() {
        let owned = String::from("DejaVu Sans");
        let f = Font::new(owned, 10.5);
        assert_eq!(f.family(), "DejaVu Sans");
        assert_eq!(f.size_pt(), 10.5);
    }
}
