/// Errors from the paint/renderer layer.
///
/// In v1, rendering errors are non-recoverable and `VelloPainter` panics or
/// logs on failure rather than returning this type.  `PaintError` is reserved
/// for a future API version when [`Painter`](crate::Painter) methods gain
/// `Result` return types.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::PaintError;
///
/// let e = PaintError::SurfaceLost;
/// assert_eq!(e.to_string(), "rendering surface lost");
/// ```
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum PaintError {
    /// The rendering surface was lost (e.g. window minimised on some backends).
    #[error("rendering surface lost")]
    SurfaceLost,

    /// The GPU device was lost and the renderer must be recreated.
    #[error("GPU device lost")]
    DeviceLost,

    /// A generic renderer error with a human-readable message.
    #[error("renderer error: {0}")]
    Other(&'static str),
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn surface_lost_display() {
        assert_eq!(
            PaintError::SurfaceLost.to_string(),
            "rendering surface lost"
        );
    }

    #[test]
    fn device_lost_display() {
        assert_eq!(PaintError::DeviceLost.to_string(), "GPU device lost");
    }

    #[test]
    fn other_display() {
        assert_eq!(
            PaintError::Other("oops").to_string(),
            "renderer error: oops"
        );
    }
}
