/// Errors from the quartzite-renderer layer.
///
/// # Examples
///
/// ```
/// use quartzite_renderer::RendererError;
/// use quartzite_paint_api::PaintError;
///
/// let e = RendererError::Paint(PaintError::SurfaceLost);
/// assert_eq!(e.to_string(), "rendering surface lost");
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RendererError {
    /// The application singleton is already live in this process.
    #[error(transparent)]
    Application(#[from] quartzite_runtime::ApplicationError),

    /// The winit event loop returned an error.
    #[error("event loop error: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),

    /// A paint/rendering error.
    #[error(transparent)]
    Paint(#[from] quartzite_paint_api::PaintError),
}

#[cfg(test)]
mod tests {
    use quartzite_paint_api::PaintError;

    use super::*;

    #[test]
    fn paint_error_display() {
        let e = RendererError::Paint(PaintError::SurfaceLost);
        assert_eq!(e.to_string(), "rendering surface lost");
    }

    #[test]
    fn paint_device_lost_display() {
        let e = RendererError::Paint(PaintError::DeviceLost);
        assert_eq!(e.to_string(), "GPU device lost");
    }
}
