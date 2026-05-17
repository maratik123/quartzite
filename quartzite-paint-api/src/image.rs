//! [`Image`] — raw RGBA8 pixel buffer used by `Painter::draw_image`.
//!
//! [`Image`] lives in `quartzite-paint-api` (rather than `quartzite-paint`) so the
//! [`Painter`](crate::Painter) trait can name `&Image` directly without a circular
//! re-export through the higher-level paint crate.

use alloc::vec::Vec;

/// A raw RGBA8 pixel buffer.
///
/// Pixels are stored row-major with no padding: `pixels.len() == width * height * 4`.
/// Each pixel is four bytes — `R`, `G`, `B`, `A` — straight (non-premultiplied) alpha.
///
/// V1 supports only RGBA8. PNG / JPEG / etc. decoding is deferred to a follow-up
/// plan; construct an [`Image`] from already-decoded pixels via [`Image::try_new`].
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::Image;
///
/// let img = Image::try_new(2, 2, vec![0u8; 16]).expect("valid 2x2 RGBA buffer");
/// assert_eq!(img.width(), 2);
/// assert_eq!(img.height(), 2);
/// assert_eq!(img.pixels().len(), 16);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Image {
    /// Creates an image from raw RGBA8 pixels, validating the buffer length.
    ///
    /// # Parameters
    ///
    /// - `width`: image width in pixels.
    /// - `height`: image height in pixels.
    /// - `pixels`: row-major RGBA8 buffer. Must have length `width * height * 4`.
    ///
    /// # Errors
    ///
    /// - [`ImageError::Overflow`] if `width * height * 4` overflows `usize`
    ///   (only possible on 32-bit targets with very large dimensions).
    /// - [`ImageError::PixelLengthMismatch`] if `pixels.len()` does not equal
    ///   `width * height * 4`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Image, ImageError};
    ///
    /// // Valid 2x2 RGBA buffer.
    /// let ok = Image::try_new(2, 2, vec![0u8; 16]);
    /// assert!(ok.is_ok());
    ///
    /// // Wrong length is rejected.
    /// let err = Image::try_new(2, 2, vec![0u8; 15]);
    /// assert!(matches!(err, Err(ImageError::PixelLengthMismatch { expected: 16, actual: 15 })));
    /// ```
    pub fn try_new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, ImageError> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or(ImageError::Overflow)?;
        if pixels.len() != expected {
            return Err(ImageError::PixelLengthMismatch {
                expected,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Returns the image width in pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Image;
    /// let img = Image::try_new(4, 3, vec![0u8; 48]).unwrap();
    /// assert_eq!(img.width(), 4);
    /// ```
    #[inline]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the image height in pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Image;
    /// let img = Image::try_new(4, 3, vec![0u8; 48]).unwrap();
    /// assert_eq!(img.height(), 3);
    /// ```
    #[inline]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the raw RGBA8 pixel buffer.
    ///
    /// Length is guaranteed to equal `width() * height() * 4`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::Image;
    /// let img = Image::try_new(1, 1, vec![255, 0, 0, 255]).unwrap();
    /// assert_eq!(img.pixels(), &[255, 0, 0, 255]);
    /// ```
    #[inline]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

/// Construction-time errors from [`Image::try_new`].
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{Image, ImageError};
///
/// let err = Image::try_new(2, 2, vec![0u8; 15]).unwrap_err();
/// assert!(matches!(err, ImageError::PixelLengthMismatch { .. }));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ImageError {
    /// The pixel buffer length did not match `width * height * 4`.
    #[error("pixel buffer length {actual} does not match expected {expected}")]
    PixelLengthMismatch {
        /// The expected buffer length, `width * height * 4`.
        expected: usize,
        /// The actual length of the supplied buffer.
        actual: usize,
    },

    /// `width * height * 4` overflowed `usize` (32-bit targets only).
    #[error("image dimensions overflow usize")]
    Overflow,
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    #[test]
    fn try_new_accepts_correct_length() {
        let img = Image::try_new(2, 2, vec![0u8; 16]).expect("valid");
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.pixels().len(), 16);
    }

    #[test]
    fn try_new_rejects_short_buffer() {
        let err = Image::try_new(2, 2, vec![0u8; 15]).unwrap_err();
        assert_eq!(
            err,
            ImageError::PixelLengthMismatch {
                expected: 16,
                actual: 15,
            }
        );
    }

    #[test]
    fn try_new_rejects_long_buffer() {
        let err = Image::try_new(2, 2, vec![0u8; 17]).unwrap_err();
        assert_eq!(
            err,
            ImageError::PixelLengthMismatch {
                expected: 16,
                actual: 17,
            }
        );
    }

    #[test]
    fn try_new_zero_zero_empty() {
        let img = Image::try_new(0, 0, vec![]).expect("zero-sized image is valid");
        assert_eq!(img.width(), 0);
        assert_eq!(img.height(), 0);
        assert_eq!(img.pixels().len(), 0);
    }

    #[test]
    fn pixels_round_trip() {
        let buf = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let img = Image::try_new(2, 1, buf.clone()).unwrap();
        assert_eq!(img.pixels(), buf.as_slice());
    }

    #[test]
    fn image_error_display() {
        let e = ImageError::PixelLengthMismatch {
            expected: 16,
            actual: 15,
        };
        assert_eq!(
            e.to_string(),
            "pixel buffer length 15 does not match expected 16"
        );
        assert_eq!(
            ImageError::Overflow.to_string(),
            "image dimensions overflow usize"
        );
    }
}
