//! Font context and text-layout scratch space used by [`VelloPainter`].
//!
//! [`VelloPainter`]: crate::vello_painter::VelloPainter

use parley::{FontContext, LayoutContext};

/// Owns the parley font-loading and layout-scratch resources.
///
/// One `FontCache` per pipeline entry point ([`RenderHarness`] or
/// [`WrappedHandler`]). Text shaping is done in [`VelloPainter`] each frame by
/// borrowing the cache mutably; the font-context data (loaded system fonts,
/// family-name resolution) persists across frames.
///
/// [`RenderHarness`]: crate::render_harness::RenderHarness
/// [`WrappedHandler`]: crate::wrapped_handler::WrappedHandler
/// [`VelloPainter`]: crate::vello_painter::VelloPainter
pub struct FontCache {
    /// Owns the system-font collection and resolved family-name tables.
    pub(crate) font_cx: FontContext,
    /// Scratch space reused across layout calls (amortises heap allocation).
    pub(crate) layout_cx: LayoutContext<[u8; 4]>,
}

impl FontCache {
    /// Creates a new `FontCache` backed by the system font collection.
    ///
    /// Constructing a `FontCache` scans available system fonts via the
    /// platform font discovery backend (fontconfig on Linux,
    /// CoreText on macOS, DirectWrite on Windows).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_renderer::font::FontCache;
    ///
    /// let _cache = FontCache::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
        }
    }
}

impl Default for FontCache {
    /// _Simple._
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_cache_constructs_without_panic() {
        let _cache = FontCache::new();
    }

    #[test]
    fn font_cache_default_constructs_without_panic() {
        let _cache = FontCache::default();
    }
}
